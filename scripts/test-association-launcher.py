#!/usr/bin/env python3
"""Check that the Windows launcher rejects malformed commands before dispatch."""
import argparse
from pathlib import Path
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', required=True, type=Path)
    args = parser.parse_args()
    if sys.platform != 'win32':
        parser.error('Requires Windows')
    for arguments in ([], ['--'], ['wrong', 'fixture.zip'], ['--', 'fixture.zip', 'extra']):
        result = subprocess.run([str(args.exe.resolve()), *arguments], timeout=10,
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if result.returncode != 2:
            raise RuntimeError('Association launcher accepted an invalid command')
    print('Windows association launcher rejects malformed commands')


if __name__ == '__main__':
    main()
