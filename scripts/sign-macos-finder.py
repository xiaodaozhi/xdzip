#!/usr/bin/env python3
"""Sign an owned local build for Finder validation, without deployment or DMG."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys
from sparkle_bundle import signing_metadata

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    app, output = args.app.resolve(), args.output.resolve()
    if sys.platform != 'darwin' or not app.is_relative_to(ROOT/'build') or not output.is_relative_to(ROOT/'build'):
        raise ValueError('Local native signing uses owned build/ paths only')
    spec = importlib.util.spec_from_file_location('macdmg', ROOT/'scripts/build-macos-dmg.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    binary = module.validate_bundle(app)
    output.parent.mkdir(parents=True, exist_ok=True)
    work = output.parent/'finder-signing-work'
    work.mkdir(exist_ok=True)
    sparkle = app/'Contents/Frameworks/Sparkle.framework'
    metadata = signing_metadata(sparkle, module.checked) if sparkle.exists() else None
    module.sign_local(app, metadata, work)
    receipt = dict(status='passed', signing='ad-hoc', developer_id_signed=False, notarized=False,
                   architectures=module.checked(['lipo', '-archs', binary]).split(),
                   finder_extension='org.xdzip.finder-sync',
                   limits='Local integrity verification, not Gatekeeper trust or a release approval.')
    output.write_text(json.dumps(receipt, indent=2)+'\n')
    print('Native Finder ad-hoc signing and strict nested verification passed')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError) as error:
        raise SystemExit(str(error)) from None
