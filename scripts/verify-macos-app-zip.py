# scripts/verify-macos-app-zip.py

"""Validate the preserved signed application after extracting the published ZIP."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
from macos_bundle import validate_optimized_bundle
from macos_launch_services import temporary_bundle_path, transient_registrations

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if sys.platform != 'darwin' or not output.is_relative_to(ROOT/'build') or output.exists():
        raise ValueError('Use native macOS and a new output directory under build/')
    output.mkdir(parents=True)
    app = output/'xdzip.app'
    with temporary_bundle_path(app), transient_registrations(output):
        subprocess.run(['ditto', '-x', '-k', str(args.archive.resolve()), str(output)], check=True)
        validate_optimized_bundle(app)
        subprocess.run(['codesign', '--verify', '--deep', '--strict', str(app)], check=True)
        spec = importlib.util.spec_from_file_location('dmg_verifier', ROOT/'scripts/verify-macos-dmg.py')
        verifier = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(verifier)
        env = verifier.clean_environment(output)
        binary = app/'Contents/MacOS/xdzip'
        identity = subprocess.check_output([str(binary), '--version'], env=env, text=True).strip()
        subprocess.run([str(binary), '--smoke-test'], env=env, timeout=30, check=True)
    (output/'verification.json').write_text(json.dumps(dict(status='passed', identity=identity,
        signing='ad-hoc', developer_id_signed=False, notarized=False), indent=2)+'\n')


if __name__ == '__main__': main()
