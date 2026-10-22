#!/usr/bin/env python3
"""Attribute trusted runtime files using SDK SPDX evidence, without compliance claims."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import urllib.request
from deployment_audit import digest, native, within

ROOT = Path(__file__).resolve().parents[1]
QT_VERSION = '6.8.2'


def bounded(root, relative):
    path = PurePosixPath(relative)
    if path.is_absolute() or '..' in path.parts or '\\' in relative or (path.parts and ':' in path.parts[0]):
        raise ValueError('Unbounded SDK evidence path')
    result = root / path
    if not within(result, root):
        raise ValueError('SDK evidence symlink escapes root')
    return result


def load_qt(sdk):
    packages, dependencies, file_index, documents, extracted = {}, {}, {}, [], {}
    paths = sorted((sdk / 'sbom').glob('*-6.8.2.spdx.json'))
    if not paths:
        raise ValueError('Pinned Qt SDK has no SPDX build documents')
    for path in paths:
        if not within(path, sdk):
            raise ValueError('SDK SPDX document escapes root')
        repo = path.name.removesuffix('-6.8.2.spdx.json')
        if repo == 'qttools':
            continue  # SDK build/test tools are not product runtime components.
        doc = json.loads(path.read_text(encoding='utf-8'))
        if doc.get('spdxVersion') != 'SPDX-2.3' or not any(
                p.get('name') == repo and p.get('versionInfo') == QT_VERSION
                for p in doc.get('packages', [])):
            raise ValueError('Qt SPDX identity/version mismatch')
        def ref(value):
            if value.startswith('DocumentRef-') and ':' in value:
                other, value = value.removeprefix('DocumentRef-').split(':', 1)
                return other + ':' + value
            return repo + ':' + value
        documents.append({'file': path.name, 'sha256': digest(path), 'repository': repo,
                          'namespace': doc['documentNamespace']})
        for p in doc['packages']:
            key = ref(p['SPDXID'])
            if key in packages:
                raise ValueError('Duplicate upstream SPDX package')
            packages[key] = {'ref': key, 'name': p['name'], 'version': p.get('versionInfo'),
                'repository': repo, 'license_declared': p.get('licenseDeclared', 'NOASSERTION'),
                'license_concluded_by_upstream': p.get('licenseConcluded', 'NOASSERTION'),
                'copyright': p.get('copyrightText', 'NOASSERTION'),
                'source': p.get('downloadLocation', 'NOASSERTION')}
        owners = {}
        for r in doc.get('relationships', []):
            if r['relationshipType'] == 'CONTAINS':
                owners.setdefault(r['relatedSpdxElement'], set()).add(ref(r['spdxElementId']))
            elif r['relationshipType'] == 'DEPENDS_ON':
                dependencies.setdefault(ref(r['spdxElementId']), set()).add(ref(r['relatedSpdxElement']))
        for f in doc.get('files', []):
            relative = f['fileName'].removeprefix('./')
            bounded(sdk, relative)
            if relative in file_index:
                raise ValueError('Ambiguous SDK SPDX file ownership')
            file_index[relative] = {'owners': sorted(owners.get(f['SPDXID'], set())),
                'upstream_checksums': f.get('checksums', []), 'repository': repo}
        for license_info in doc.get('hasExtractedLicensingInfos', []):
            key = repo + ':' + license_info['licenseId']
            extracted[key] = license_info['extractedText']
    return packages, dependencies, file_index, documents, extracted


def license_materials(output, cache):
    config = json.loads((ROOT / 'scripts/qt-license-sources.json').read_text())
    materials = {}
    target = output / 'license-texts'; target.mkdir(parents=True, exist_ok=True)
    cache.mkdir(parents=True, exist_ok=True)
    for record in config['licenses']:
        # Only reviewed configuration URLs may be fetched, never SPDX-provided URLs.
        qt_source = re.fullmatch(r'https://raw\.githubusercontent\.com/qt/[a-z]+/[0-9a-f]{40}/[A-Za-z0-9_./+-]+\.txt', record['url'])
        spdx_source = re.fullmatch(r'https://raw\.githubusercontent\.com/spdx/license-list-data/[0-9a-f]{40}/text/[A-Za-z0-9.+-]+\.txt', record['url'])
        if not (qt_source or spdx_source) or '..' in record['url']:
            raise ValueError('Unpinned license source')
        path = cache / (record['id'] + '.txt')
        if not path.exists():
            data = urllib.request.urlopen(record['url'], timeout=30).read()
        else:
            data = path.read_bytes()
        if hashlib.sha256(data).hexdigest() != record['sha256']:
            raise ValueError('Qt license text hash mismatch')
        path.write_bytes(data); shutil.copyfile(path, target / path.name)
        materials[record['id']] = dict(record, file='license-texts/' + path.name)
    return materials


def closure(seeds, packages, dependencies):
    selected, missing = set(), set()
    pending = list(seeds)
    while pending:
        key = pending.pop()
        if key in selected or key in missing:
            continue
        if key not in packages:
            missing.add(key); continue
        selected.add(key)
        pending.extend(dependencies.get(key, []))
    return selected, missing


def attribute(audit, root, sdk, deployment=None):
    if audit['status'] != 'passed':
        raise ValueError('Cannot attribute failed deployment audit')
    packages, dependencies, index, documents, extracted = load_qt(sdk)
    provenance = {p['path']: p for p in (deployment or {}).get('sdk_files', [])}
    by_name = {}
    for relative in index:
        by_name.setdefault(PurePosixPath(relative).name, []).append(relative)
    files, seeds, gaps = [], set(), []
    for item in audit['files'] + audit['external_files']:
        external = item in audit['external_files']
        relative = item['path']
        if not external and digest(bounded(root, relative)) != item['sha256']:
            raise ValueError('Artifact changed after deployment audit')
        source, evidence = None, None
        if external and relative.startswith('qt-sdk/'):
            source = relative.removeprefix('qt-sdk/'); evidence = 'sdk-byte-identity'
        elif not external and relative in provenance:
            p = provenance[relative]; source = p['sdk_path']; evidence = 'deployment-copy-and-rewrite-record'
            if p['artifact_sha256'] != item['sha256'] or digest(bounded(sdk, source)) != p['sdk_sha256']:
                raise ValueError('Stale deployment copy provenance')
            if audit['platform'] == 'macos':
                from macos_attribution import compare_images, sdk_relative
                expected = sdk_relative(relative)
                if not expected or bounded(sdk, source).resolve() != bounded(sdk, expected).resolve():
                    raise ValueError('Unexpected macOS SDK ownership path')
                if p.get('comparison') == 'sdk-byte-identity':
                    if item['sha256'] != p['sdk_sha256']:
                        raise ValueError('macOS SDK byte identity mismatch')
                elif p.get('comparison') == 'mach-o-sections-uuid-and-imports':
                    fingerprint = compare_images(bounded(root, relative).read_bytes(), bounded(sdk, source).read_bytes())
                    if fingerprint != p.get('fingerprint'):
                        raise ValueError('Stale macOS section comparison')
                else:
                    raise ValueError('Unverified macOS deployment comparison')
                evidence = p['comparison']
        elif not external:
            candidate = bounded(sdk, relative)
            if candidate.is_file() and digest(candidate) == item['sha256']:
                source = relative; evidence = 'sdk-byte-identity'
            else:
                candidates = [p for p in by_name.get(PurePosixPath(relative).name, []) if bounded(sdk, p).is_file()]
                identical = [p for p in candidates if digest(bounded(sdk, p)) == item['sha256']]
                if identical:
                    source = identical[0]; evidence = 'sdk-byte-identity'
                elif len(candidates) == 1:
                    source = candidates[0]; evidence = 'filename-only-unverified-rewrite'
        actual = bounded(sdk, source) if source else bounded(root, relative) if not external else None
        if source:
            sdk_file = bounded(sdk, source)
            if not sdk_file.is_file():
                raise ValueError('Missing attributed SDK file')
            sdk_hash = digest(sdk_file)
            if evidence == 'sdk-byte-identity' and sdk_hash != item['sha256']:
                raise ValueError('SDK/audit byte identity mismatch')
            entry = index.get(source)
            owners = entry['owners'] if entry else []
            if not entry and source.startswith(('qml/', 'translations/')):
                repo = 'qtdeclarative' if source.startswith('qml/') else 'qttranslations'
                key = repo + ':SPDXRef-Package-' + repo
                if key in packages:owners = [key]
            seeds.update(owners)
            checks = []
            for checksum in (entry or {}).get('upstream_checksums', []):
                algorithm = checksum['algorithm'].lower().replace('-', '')
                if algorithm in {'sha1', 'sha256'}:
                    checks.append({'algorithm': checksum['algorithm'], 'expected': checksum['checksumValue'],
                        'matches_sdk': hashlib.new(algorithm, sdk_file.read_bytes()).hexdigest() == checksum['checksumValue']})
            files.append({'path': relative if external else 'artifact/' + relative,
                'sha256': item['sha256'], 'sdk_path': source, 'sdk_sha256': sdk_hash,
                'evidence': evidence, 'owners': owners, 'upstream_checksums': checks})
            if evidence == 'filename-only-unverified-rewrite':
                gaps.append({'kind': 'unverified-rewrite', 'file': relative})
            if any(not c['matches_sdk'] for c in checks):
                gaps.append({'kind': 'upstream-sdk-checksum-difference', 'file': source})
            if entry and not owners:
                raise ValueError('SPDX runtime file has no package owner')
            if not entry and actual and native(actual, audit['platform']):
                gaps.append({'kind': 'sdk-native-without-spdx-file', 'file': source})
        elif actual and actual.is_file() and native(actual, audit['platform']):
            # Includes the product and platform redistributables; do not guess their supplier.
            gaps.append({'kind': 'native-outside-qt-attribution', 'file': relative})
    selected, missing = closure(seeds, packages, dependencies)
    for key in sorted(missing):
        gaps.append({'kind': 'upstream-package-reference-missing', 'ref': key})
    for key in sorted(selected):
        p = packages[key]
        if p['license_concluded_by_upstream'] in {'NOASSERTION', 'NONE', ''}:
            gaps.append({'kind': 'upstream-license-unasserted', 'ref': key})
    return {'schema_version': 1, 'status': 'evidence-generated-release-review-required',
        'identity': audit['identity'], 'platform': audit['platform'], 'mode': audit['mode'],
        'qt_version': QT_VERSION, 'audit_sha256': hashlib.sha256(json.dumps(audit,sort_keys=True,separators=(',',':')).encode()).hexdigest(),
        'upstream_documents': documents, 'files': files,
        'components': [packages[k] for k in sorted(selected)],
        'dependencies': [{'ref': k, 'dependsOn': sorted(dependencies.get(k, set()) & selected)} for k in sorted(selected)],
        'gaps': gaps, 'extracted_license_texts': {k: v for k, v in extracted.items() if k.split(':')[0] in {packages[s]['repository'] for s in selected}},
        'limits': ['SDK SPDX ownership is upstream evidence, not a legal conclusion or proof of unmodified upstream bytes.',
            'Filename-only matches and upstream checksum differences require review; deployment rewrite records are separate evidence.',
            'Non-native QML/resources can lack individual SPDX ownership; embedded assets, static Rust exact linkage and OS/redistributable licenses need separate review.',
            'Corresponding source archives, relinking/install materials and applicable redistribution obligations remain release checks.']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--audit', type=Path, required=True); parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--qt-prefix', type=Path, required=True); parser.add_argument('--deployment', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--license-cache', type=Path, default=ROOT / 'build/review/qt-license-cache')
    parser.add_argument('--require-complete', action='store_true')
    args = parser.parse_args()
    if within(args.output, args.root):
        parser.error('Attribution report must be outside the audited artifact')
    report = attribute(json.loads(args.audit.read_text()), args.root.resolve(), args.qt_prefix.resolve(),
        json.loads(args.deployment.read_text()) if args.deployment else None)
    materials = license_materials(args.output.parent / 'runtime-notices', args.license_cache)
    report['license_materials'] = materials
    report['license_materials_base'] = 'runtime-notices'
    for component in report['components']:
        expression = component['license_concluded_by_upstream']
        for license_id in re.findall(r'[A-Za-z0-9][A-Za-z0-9.+-]*', expression):
            if license_id in {'OR', 'AND', 'WITH', 'NOASSERTION', 'NONE'}:
                continue
            if license_id not in materials and component['repository'] + ':' + license_id not in report['extracted_license_texts']:
                report['gaps'].append({'kind': 'license-text-missing', 'ref': component['ref'], 'license': license_id})
    # These are explicit release blockers even if every known Qt license has a text.
    report['release_blockers'] = ['Exact static Rust linkage and embedded resources attribution',
        'Corresponding source/relinking delivery and platform redistributable obligations',
        'Manual license choice and compliance review']
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(f'Runtime attribution: {len(report["files"])} SDK files, {len(report["components"])} upstream components, {len(report["gaps"])} review gaps; release incomplete')
    if args.require_complete and (report['gaps'] or report['release_blockers']):
        raise SystemExit('Release attribution remains incomplete')


if __name__ == '__main__':
    main()
