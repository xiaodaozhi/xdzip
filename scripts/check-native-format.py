"""Run clang-format on tracked, handwritten native sources."""

import argparse
import os
from pathlib import Path, PurePosixPath
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SUFFIXES = {'.c', '.cc', '.cpp', '.cxx', '.h', '.hh', '.hpp', '.hxx', '.m', '.mm'}


def native_sources(root):
    tracked = subprocess.check_output([
        'git', 'ls-files', '-z', '--', 'apps', 'crates', 'tools', 'tests/fixtures/native',
    ], cwd=root)
    sources = []
    for entry in tracked.split(b'\0'):
        path = PurePosixPath(os.fsdecode(entry))
        if path.suffix not in SUFFIXES:
            continue
        if path == PurePosixPath('apps/desktop/ArchiveFormatsGenerated.h'):
            continue  # Checked by the format registry generator.
        if path.is_relative_to('apps/desktop/tests/preferences-compat/previous'):
            continue  # Frozen sources used for compatibility comparisons.
        sources.append(path.as_posix())
    return sources


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--clang-format', default='clang-format')
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    sources = native_sources(ROOT)
    if not sources:
        parser.error('no tracked native sources found')
    flags = ['-i'] if args.write else ['--dry-run', '--Werror']
    # Bound command length on Windows as well as on POSIX hosts.
    for start in range(0, len(sources), 64):
        subprocess.run([args.clang_format, *flags, *sources[start:start + 64]], cwd=ROOT, check=True)


if __name__ == '__main__':
    main()
