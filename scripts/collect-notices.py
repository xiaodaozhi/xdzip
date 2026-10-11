#!/usr/bin/env python3
"""Collect source-pinned license notices for private developer-review builds (stdlib only)."""
import json
from pathlib import Path
import shutil
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
out = Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)
shutil.copyfile(root / 'LICENSE', out / 'xdzip-Apache-2.0.txt')
metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1'], cwd=root, text=True))
inventory = []
for package in metadata['packages']:
    if package['source'] is None:
        continue
    source = Path(package['manifest_path']).parent
    target = out / f'{package["name"]}-{package["version"]}'
    target.mkdir(exist_ok=True)
    notices = [p for p in source.iterdir() if p.is_file() and p.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'NOTICE'))]
    if not notices:
        raise SystemExit(f'Missing upstream notices: {package["name"]}')
    for notice in notices:
        shutil.copyfile(notice, target / notice.name)
    inventory.append({k: package[k] for k in ('name', 'version', 'license', 'repository')})
shutil.copytree(root / 'third_party/qt', out / 'Qt-6.8.2', dirs_exist_ok=True)
(out / 'rust-dependencies.json').write_text(json.dumps(inventory, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
