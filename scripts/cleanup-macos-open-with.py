#!/usr/bin/env python3
"""Preview or remove stale xdzip build-copy LaunchServices registrations."""
import argparse
import json
from pathlib import Path
import sys
import subprocess
from macos_launch_services import cleanup


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--keep', type=Path, required=True,
                        help='Existing primary xdzip.app to keep registered')
    parser.add_argument('--apply', action='store_true', help='Apply the previewed cleanup')
    args = parser.parse_args()
    if sys.platform != 'darwin':
        parser.error('This maintenance command requires macOS')
    try:
        result = cleanup(args.keep, args.apply)
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        # Native command exceptions contain user paths; do not print their payload.
        parser.exit(1, 'LaunchServices maintenance failed: '+type(error).__name__+'\n')
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
