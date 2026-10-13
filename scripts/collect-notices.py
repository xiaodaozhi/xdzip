#!/usr/bin/env python3
"""Collect source-pinned license notices for private developer-review builds (stdlib only)."""
import json
import hashlib
from pathlib import Path
import shutil
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
out = Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)
shutil.copyfile(root / 'LICENSE', out / 'xdzip-Apache-2.0.txt')
host = next(line.removeprefix('host: ') for line in subprocess.check_output(['rustc', '-vV'], text=True).splitlines() if line.startswith('host: '))
metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1', '--filter-platform', host], cwd=root, text=True))
resolved = {node['id'] for node in metadata['resolve']['nodes']}
inventory = []
for package in metadata['packages']:
    if package['source'] is None or package['id'] not in resolved:
        continue
    source = Path(package['manifest_path']).parent
    target = out / f'{package["name"]}-{package["version"]}'
    target.mkdir(exist_ok=True)
    notices = [p for p in source.iterdir() if p.is_file() and p.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'NOTICE'))]
    # This published crate places the full MIT grant and copyright in AUTHORS.
    # Pin that exception; an arbitrary author list is not a license notice.
    if not notices and (package['name'], package['version']) == ('r-efi', '6.0.0'):
        notice = source / 'AUTHORS'
        if notice.is_file() and hashlib.sha256(notice.read_bytes()).hexdigest() == 'd027e91dbc9cdbb2f1190068e498bd6b61cff022b6a032b191021ba658d96111':
            notices = [notice]
    if not notices:
        raise SystemExit(f'Missing upstream notices: {package["name"]}')
    for notice in notices:
        shutil.copyfile(notice, target / notice.name)
    inventory.append({k: package[k] for k in ('name', 'version', 'license', 'repository')})
shutil.copytree(root / 'third_party/qt', out / 'Qt-6.8.2', dirs_exist_ok=True)
shutil.copyfile(root / 'design/assets/fonts/OFL.txt', out / 'Xdzip-Design-Sans-SC-OFL.txt')
(out / 'rust-dependencies.json').write_text(json.dumps(inventory, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
(out / 'target.json').write_text(json.dumps({'rust_host': host, 'scope': 'Host-resolved workspace dependencies, including build tools; Cargo.lock retains all targets'}, indent=2) + '\n', encoding='utf-8')
