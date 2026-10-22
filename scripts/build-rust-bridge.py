# scripts/build-rust-bridge.py

"""Build a CXX package and restore its public headers even after Ninja clean."""

import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys

parser = argparse.ArgumentParser(description=__doc__)
for name in ('cargo', 'root', 'target-dir', 'target', 'profile', 'package'):
    parser.add_argument('--' + name, required=True)
args = parser.parse_args()
command = [args.cargo, 'build', '--locked', '-p', args.package,
           '--manifest-path', str(Path(args.root) / 'Cargo.toml'),
           '--target-dir', args.target_dir, '--target', args.target,
           '--profile', args.profile, '--message-format=json-render-diagnostics']
output = None
with subprocess.Popen(command, cwd=args.root, stdout=subprocess.PIPE, text=True, encoding='utf-8') as process:
    for line in process.stdout:
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            print(line, end='')
            continue
        if event.get('reason') == 'compiler-message':
            print(event.get('message', {}).get('rendered', ''), end='', file=sys.stderr)
        if event.get('reason') == 'build-script-executed':
            package_id = event.get('package_id', '')
            fragment = package_id.rsplit('#', 1)[-1]
            package = fragment.split('@', 1)[0] if '@' in fragment else package_id.split('#', 1)[0].rsplit('/', 1)[-1]
            if package == args.package:
                output = Path(event['out_dir']) / 'cxxbridge/include'
    result = process.wait()
if result:
    raise SystemExit(result)
if output is None or not (output / args.package).is_dir() or not (output / 'rust/cxx.h').is_file():
    raise SystemExit('Cargo did not report the selected CXX build script headers')
destination = Path(args.target_dir) / 'cxxbridge'
for directory in (output / args.package, output / 'rust'):
    for source in directory.rglob('*'):
        if not source.is_file():
            continue
        target = destination / source.relative_to(output)
        # Public CXX paths can already be symlinks to the same source file.
        if target.exists() and target.read_bytes() == source.read_bytes():
            continue
        if target.is_symlink():
            target.unlink()
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
