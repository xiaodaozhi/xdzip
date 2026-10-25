#!/usr/bin/env python3
"""Collect source-pinned license notices for private developer-review builds (stdlib only)."""
import json
import hashlib
from pathlib import Path
import shutil
import subprocess
import sys
import runpy
import argparse
from sbom import source_bom
from runtime_rust import runtime_candidates
from native_runtime_materials import prepare

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('output', type=Path)
parser.add_argument('--sparkle-sdk', type=Path)
arguments = parser.parse_args()
out = arguments.output
out.mkdir(parents=True, exist_ok=True)
native_materials = prepare(out, {'win32': 'windows', 'darwin': 'macos'}.get(sys.platform, 'linux'))
notice_files = []
notice_files.extend(out/item['file'] for item in native_materials['files'])
native_dependencies = []
from native_codec_materials import notices as codec_notices
codec_files, codec_dependencies = codec_notices(out)
notice_files += codec_files
native_dependencies += codec_dependencies
if sys.platform == 'darwin':
    from sparkle_bundle import configuration, write_notice
    sparkle_sdk = arguments.sparkle_sdk or root/'build/desktop/_deps/sparkle-2.10.0'
    notice_files += write_notice(sparkle_sdk.resolve(), out)
    native_dependencies.append(configuration())
shutil.copyfile(root / 'LICENSE', out / 'xdzip-Apache-2.0.txt')
notice_files.append(out / 'xdzip-Apache-2.0.txt')
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
    notices = [p for p in source.iterdir() if p.is_file() and p.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'COPYRIGHT', 'NOTICE'))]
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
        notice_files.append(target / notice.name)
    inventory.append({k: package[k] for k in ('name', 'version', 'license', 'repository')})
shutil.copytree(root / 'third_party/qt', out / 'Qt-6.8.2', dirs_exist_ok=True)
notice_files += [out/'Qt-6.8.2'/p.relative_to(root/'third_party/qt') for p in (root/'third_party/qt').rglob('*') if p.is_file()]
shutil.copyfile(root / 'design/assets/fonts/OFL.txt', out / 'Xdzip-Design-Sans-SC-OFL.txt')
notice_files.append(out / 'Xdzip-Design-Sans-SC-OFL.txt')
(out / 'rust-dependencies.json').write_text(json.dumps(inventory, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
(out / 'target.json').write_text(json.dumps({'rust_host': host, 'scope': 'Host-resolved workspace dependencies, including build tools; Cargo.lock retains all targets'}, indent=2) + '\n', encoding='utf-8')

version = runpy.run_path(str(root/'scripts/sync-version.py'))['version_info']()
display_version = version.pop('display_version')
# Map Cargo's path-bearing opaque IDs to stable references before writing evidence.
references = {p['id']: 'cargo:'+p['name']+'@'+p['version']+':'+hashlib.sha256((p['source'] or 'workspace').encode()).hexdigest()[:12]
              for p in metadata['packages'] if p['id'] in resolved}
assets = []
for relative, license_id in [('design/assets/fonts/xdzip-design-sans-sc.woff2','OFL-1.1'),
                             ('design/assets/fonts/xdzip-design-sans-sc.ttf','OFL-1.1'),
                             ('design/assets/brand/xdzip.svg','Apache-2.0'),
                             ('design/assets/brand/xdzip-zip.svg','Apache-2.0')]:
    assets.append({'path':relative, 'license':license_id, 'sha256':hashlib.sha256((root/relative).read_bytes()).hexdigest()})
revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
source_inventory = {
    'schema_version': 1, 'product': 'xdzip', 'version_source': version,
    'display_version': display_version, 'revision': revision, 'dirty': bool(subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=normal'], cwd=root, text=True)), 'rust_host': host,
    'rust_packages': [dict({k: p[k] for k in ('name', 'version', 'license', 'repository')}, bom_ref=references[p['id']], source_kind='registry' if (p['source'] or '').startswith('registry+') else 'git' if p['source'] else 'workspace')
                      for p in metadata['packages'] if p['id'] in resolved],
    'rust_dependencies': [{'ref':references[n['id']], 'dependsOn':sorted({references[d['pkg']] for d in n['deps'] if d['pkg'] in references})}
                          for n in metadata['resolve']['nodes'] if n['id'] in references],
    'rust_runtime_candidates': runtime_candidates(metadata, references),
    'assets': assets,
    'qt_declared_product_modules': ['Core', 'Gui', 'Network', 'Qml', 'Quick', 'QuickControls2', 'Widgets'],
    'qt_version': '6.8.2',
    'native_dependencies': native_dependencies,
    'font': json.loads((root / 'design/assets/fonts/manifest.json').read_text()),
    'notice_hashes': [{'file': p.relative_to(out).as_posix(), 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()}
                      for p in sorted(notice_files)],
    'limits': ['Source-resolved host build inventory, including build tools, not an installed artifact SBOM.',
               'Declared Qt product modules do not inventory all transitive deployed libraries or plugins.',
               'Signed packages, deployed dependency completeness and licensing obligations still require release review.']
}
(out / 'build-inventory.json').write_text(json.dumps(source_inventory, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
(out / 'source-build.cdx.json').write_text(json.dumps(source_bom(source_inventory), indent=2) + '\n', encoding='utf-8')
print(f'Collected {len(inventory)} external Rust notice sets and a versioned source build inventory')
