"""Read source provenance for native desktop and CLI build entry points."""

import json
import os
from pathlib import Path
import re
import subprocess
import sys


def source_identity(root):
    root = Path(root).resolve()

    def git(*args):
        return subprocess.check_output(
            ['git', '--no-optional-locks', *args], cwd=root, stderr=subprocess.DEVNULL)

    try:
        checkout = Path(os.fsdecode(git('rev-parse', '--show-toplevel').rstrip(b'\r\n')))
        if checkout.resolve() == root:
            revision = git('rev-parse', '--short=12', 'HEAD').decode('ascii').strip()
            if re.fullmatch('[0-9a-f]{12,40}', revision):
                return {'revision': revision, 'dirty': bool(
                    git('status', '--porcelain', '-z', '--untracked-files=normal'))}
    except (OSError, ValueError, subprocess.CalledProcessError):
        pass
    return {'revision': 'unknown', 'dirty': False}


if __name__ == '__main__':
    print(json.dumps(source_identity(sys.argv[1])))
