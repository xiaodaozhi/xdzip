# scripts/sign-macos-update.py

"""Sign a staged Mac asset in an isolated step, without exposing secrets to builds."""
import argparse
from pathlib import Path
import runpy

from sparkle_bundle import ROOT, sign_staged_archive


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--architecture', choices=['arm64', 'x86_64'], default='arm64')
    args = parser.parse_args()
    directory = args.directory.resolve()
    if not directory.is_relative_to((ROOT/'build').resolve()):
        parser.error('Update assets must be staged under build/')
    version = runpy.run_path(str(ROOT/'scripts/sync-version.py'))['version_info']()
    signed = sign_staged_archive(directory, version['display_version'], version['build'], architecture=args.architecture)
    print('Mac update archive verified and signed' if signed else
          'Mac update signing is not configured; candidate cannot be published as authenticated')


if __name__ == '__main__':
    main()
