#!/usr/bin/env python3
"""Collect release evidence while the final signed deployment still exists."""
import argparse
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
import sys

from deployment_audit import ROOT, digest, within
from release_evidence import compare_macho_sections, verify_bindings


def collect(source, deployed, qt, output):
    if sys.platform != 'darwin':
        raise ValueError('Release deployment evidence requires native macOS')
    source, deployed, qt, output = (p.resolve() for p in (source, deployed, qt, output))
    if not all(within(p, ROOT/'build') for p in (source, deployed, output)):
        raise ValueError('Use owned application and evidence paths under build/')
    compiled = source/'Contents/MacOS/xdzip'
    final = deployed/'Contents/MacOS/xdzip'
    build = source.parents[2]
    link_map = source.parent/'xdzip-link.map'
    metrics = build/'tools/process-metrics/xdzip-process-metrics'
    fixture = source.parent/'m2-flat.zip'
    libraries = sorted((source.parent/'rust').rglob('libxdzip_qt_bridge.a'))
    if len(libraries) != 1 or not all(p.is_file() for p in (compiled, final, link_map, metrics, fixture)):
        raise ValueError('Missing unique compiled link inputs or native probe tools')
    names = ['source-build.cdx.json', 'artifact-files.cdx.json', 'qt-sdk-correspondence.json',
             'qt-binary-differences.json', 'desktop-performance.json', 'runtime-resources.json',
             'linkage-evidence.json', 'release-deployment-evidence.json', 'release-evidence-commands.log']
    if any((output/name).exists() for name in names):
        raise ValueError('Use a fresh release evidence output')
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    if subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT).strip():
        raise ValueError('Release evidence requires clean source')
    if any(p.stat().st_size > 256 * 1024 * 1024 for p in (compiled, final)):
        raise ValueError('Unsupported release application image size')
    match = compare_macho_sections(compiled.read_bytes(), final.read_bytes())
    audit = json.loads((output/'deployment-audit.json').read_text())
    expected = next(row['sha256'] for row in audit['files'] if row['path'] == audit['app'])
    if match['deployed_sha256'] != expected:
        raise ValueError('Deployment image changed before evidence collection')
    source_bom = deployed/'Contents/Resources/notices/source-build.cdx.json'
    file_bom = output/f'xdzip-{audit["identity"]["version"]}-macos-arm64.cdx.json'
    shutil.copy2(source_bom, output/'source-build.cdx.json')
    shutil.copy2(file_bom, output/'artifact-files.cdx.json')
    environment = os.environ.copy()
    environment['CARGO_NET_OFFLINE'] = 'true'
    with (output/'release-evidence-commands.log').open('x', encoding='utf-8') as log:
        def run(script, *arguments, env=None):
            log.write('Running '+script+'\n')
            result = subprocess.run([sys.executable, str(ROOT/'scripts'/script), *map(str, arguments)],
                                    cwd=ROOT, env=environment if env is None else env,
                                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                    text=True, encoding='utf-8', errors='replace')
            diagnostics = result.stdout.replace(str(ROOT), '<workspace>').replace(str(Path.home()), '<home>')
            log.write(diagnostics[-4*1024*1024:])
            log.flush()
            if result.returncode:
                raise ValueError(script+' failed; inspect release-evidence-commands.log')
        run('qt_sdk_correspondence.py', '--qt-prefix', qt, '--fetch',
            '--output', output/'qt-sdk-correspondence.json')
        run('classify-qt-binary-differences.py', '--attribution', output/'runtime-attribution.json',
            '--qt-prefix', qt, '--output', output/'qt-binary-differences.json')
        run('validate-sbom.py', output/'source-build.cdx.json', output/'artifact-files.cdx.json')
        # Reuse the package verifier's complete SDK/diagnostic override scrub.
        verifier = runpy.run_path(str(ROOT/'scripts/verify-macos-dmg.py'))
        runtime = verifier['clean_environment'](output/'release-probe')
        runtime['XDZIP_PROBE_RESOURCE_MANIFEST'] = '1'
        run('measure-desktop.py', '--app', final, '--fixture', fixture, '--metrics', metrics,
            '--runs', '1', '--require-backend', 'metal', '--output', output/'desktop-performance.json', env=runtime)
        run('check-runtime-resources.py', '--probe', output/'desktop-performance.json',
            '--output', output/'runtime-resources.json')
        run('linkage-evidence.py', '--app', compiled, '--map', link_map, '--rust-library', libraries[0],
            '--output', output/'linkage-evidence.json')
    def read(name):
        return json.loads((output/name).read_text())
    verify_bindings(revision, audit, read('runtime-resources.json'), read('linkage-evidence.json'),
                    read('qt-sdk-correspondence.json'), match)
    if digest(final) != expected or digest(compiled) != match['compiled_sha256']:
        raise ValueError('Application image changed during evidence collection')
    report = {'schema_version': 1, 'status': 'passed', 'revision': revision, 'platform': 'macos',
              'deployment_audit_sha256': digest(output/'deployment-audit.json'),
              'compiled_app_sha256': match['compiled_sha256'], 'deployed_app_sha256': expected,
              'code_data_sections': match,
              'records': [{'file': name, 'sha256': digest(output/name)} for name in names if name.endswith('.json')
                          and name != 'release-deployment-evidence.json'],
              'limits': ['Compiled link inputs and deployed resource bytes have distinct executable hashes.',
                         'Non-debug code/data section equivalence does not prove complete binary or SDK build equivalence.',
                         'One Cocoa/Metal probe is not reference-host performance or human visual acceptance.',
                         'Original attribution gaps, incomplete SBOM and license choices remain separate.'],
              'release_approved': False, 'manual_acceptance_claimed': False, 'reference_budget_approved': False}
    (output/'release-deployment-evidence.json').write_text(json.dumps(report, indent=2)+'\n')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-app', type=Path, required=True)
    parser.add_argument('--deployed-app', type=Path, required=True)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--output-dir', type=Path, required=True)
    args = parser.parse_args()
    collect(args.source_app, args.deployed_app, args.qt_prefix, args.output_dir)
    print('Native release resources and compiled link inputs bound to the exact deployed application')


if __name__ == '__main__':
    main()
