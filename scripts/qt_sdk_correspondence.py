#!/usr/bin/env python3
"""Bind SDK declarations/configuration and known consumer patches to pinned source bytes.

This is an evidence collector, not a Qt rebuild or a compliance approval.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import tarfile
from deployment_audit import digest, within

ROOT = Path(__file__).resolve().parents[1]


def safe_path(name):
    path = PurePosixPath(name.removeprefix('./'))
    if path.is_absolute() or '..' in path.parts or '\\' in name or ':' in name:
        raise ValueError('Unsafe source-relative path')
    return path.as_posix()


def source_checksums(path):
    """Read REUSE tag/value file sections without retaining upstream private paths."""
    records = {}
    name = None
    with path.open(encoding='utf-8') as stream:
        for line in stream:
            if line.startswith('FileName: '):
                if name is not None:
                    raise ValueError('Source file record has no SHA1 checksum')
                # Windows REUSE declarations use native relative separators.
                # Normalize only declaration names; archive members stay POSIX-only.
                name = safe_path(line[len('FileName: '):].strip().replace('\\', '/'))
            elif line.startswith('FileChecksum: SHA1: ') and name:
                value = line.split(': ', 2)[2].strip().lower()
                if not re.fullmatch('[0-9a-f]{40}', value) or name in records:
                    raise ValueError('Invalid or duplicate source checksum')
                records[name] = value
                name = None
    if name is not None:
        raise ValueError('Source file record has no SHA1 checksum')
    if not records:
        raise ValueError('Empty source SPDX checksum inventory')
    return records


def compare_sources(archive_path, item, declaration=None):
    if digest(archive_path) != item['sha256']:
        raise ValueError('Pinned source archive checksum mismatch')
    expected = source_checksums(declaration) if declaration else {}
    matched, different, absent = [], [], []
    archive_only = 0
    prefix = item['file'].removesuffix('.tar.xz')+'/'
    found_tag = None
    with tarfile.open(archive_path, 'r:xz') as archive:
        seen = set()
        for member in archive:
            if not member.name.startswith(prefix):
                if member.isdir() and member.name == prefix[:-1]:
                    continue
                raise ValueError('Unexpected source archive layout')
            name = safe_path(member.name[len(prefix):])
            if name in seen:
                raise ValueError('Duplicate source archive member')
            seen.add(name)
            if not member.isfile():
                continue
            if name == '.tag' or name in expected:
                content = archive.extractfile(member).read()
                if name == '.tag':
                    found_tag = content.decode('ascii').strip()
                if name in expected:
                    actual = hashlib.sha1(content).hexdigest()
                    if actual == expected[name]:
                        matched.append(name)
                    else:
                        different.append({'file': name, 'sdk_source_sha1': expected[name], 'archive_sha1': actual})
            if name not in expected:
                archive_only += 1
        absent = sorted(set(expected)-seen)
    if found_tag != item['source_commit']:
        raise ValueError('Pinned source commit marker mismatch')
    return {'module': item['module'], 'archive_sha256': item['sha256'], 'archive_commit': found_tag,
            'sdk_source_spdx_sha256': digest(declaration) if declaration else None,
            'matched_files': len(matched), 'different_files': different, 'sdk_only_files': absent,
            'archive_only_files': archive_only,
            'status': 'matching-declared-files' if expected and not different and not absent else 'incomplete'}


def replay_patch(before, patch):
    lines = before.decode('utf-8').splitlines(keepends=True)
    output, cursor = [], 0
    changes = patch.decode('utf-8').splitlines(keepends=True)
    index = 2
    while index < len(changes):
        hunk = re.fullmatch(r'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@[^\n]*\n?', changes[index])
        if not hunk:
            raise ValueError('Unsupported SDK patch hunk')
        start = int(hunk[1])-1
        if start < cursor:
            raise ValueError('Overlapping SDK patch hunk')
        output.extend(lines[cursor:start])
        cursor = start
        old_count = new_count = 0
        index += 1
        while index < len(changes) and not changes[index].startswith('@@ '):
            line = changes[index]
            if line[0] not in ' +-':
                raise ValueError('Unsupported SDK patch line')
            if line[0] in ' -':
                if cursor >= len(lines) or lines[cursor] != line[1:]:
                    raise ValueError('SDK patch context mismatch')
                cursor += 1
                old_count += 1
            if line[0] in ' +':
                output.append(line[1:])
                new_count += 1
            index += 1
        if old_count != int(hunk[2] or 1) or new_count != int(hunk[4] or 1):
            raise ValueError('SDK patch hunk count mismatch')
    output.extend(lines[cursor:])
    return ''.join(output).encode('utf-8')


def patch_status(prefix, item, archive_path):
    patch = ROOT/item['patch']
    if not within(patch, ROOT/'scripts/qt-patches') or digest(patch) != item['patch_sha256']:
        raise ValueError('Pinned SDK patch checksum mismatch')
    with tarfile.open(archive_path, 'r:xz') as archive:
        member = archive.getmember(archive_path.name.removesuffix('.tar.xz')+'/'+safe_path(item['source_path']))
        if not member.isfile():
            raise ValueError('SDK patch base is not a regular source file')
        before = archive.extractfile(member).read()
    if hashlib.sha256(before).hexdigest() != item['before_sha256'] or hashlib.sha256(
            replay_patch(before, patch.read_bytes())).hexdigest() != item['after_sha256']:
        raise ValueError('SDK patch does not reproduce pinned helper bytes')
    relative = safe_path(item['sdk_path'])
    path = prefix/relative
    if not within(path.resolve(), prefix) or not path.is_file():
        raise ValueError('SDK patch target missing or outside SDK')
    raw = path.read_bytes()
    actual = hashlib.sha256(raw).hexdigest()
    normalized = hashlib.sha256(raw.replace(b'\r\n', b'\n')).hexdigest()
    if normalized == item['before_sha256']:
        state = 'upstream-unmodified'
    elif normalized == item['after_sha256']:
        state = 'known-local-patch'
    else:
        raise ValueError('Unrecognized SDK consumer helper modification, SHA256='+actual+', LF-SHA256='+normalized)
    return {'id': item['id'], 'sdk_path': relative, 'state': state, 'sdk_sha256': actual,
            'patch_sha256': item['patch_sha256'], 'scope': item['scope'],
            'line_endings': 'LF' if actual == normalized else 'CRLF-normalized-for-comparison',
            'lf_sha256': normalized}


def sanitize(text):
    # SDK build comments carry the upstream maintainer's absolute paths. Keep
    # flags/order while exporting placeholders; raw evidence is bound by hash.
    return re.sub(r'(?<![\w:])(?:[A-Za-z]:[/\\]|/(?:Users|home|Applications|Library|usr|opt|build|src_dir)/)[^\s;"\']*',
                  '<upstream-path>', text)


def collect(prefix, cache, fetch=False):
    config = json.loads((ROOT/'scripts/corresponding-sources.json').read_text())
    spec = importlib.util.spec_from_file_location('delivery', ROOT/'scripts/prepare-source-delivery.py')
    delivery = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(delivery)
    modules = []
    for item in config['archives']:
        archive = cache/item['file']
        if fetch:
            cache.mkdir(parents=True, exist_ok=True)
            archive = delivery.archive_source(item, cache)
        document = prefix/'sbom'/f"{item['module']}-{config['qt_version']}.spdx.json"
        data = json.loads(document.read_text(encoding='utf-8'))
        candidates = [p for p in data['packages'] if p.get('name') == item['module']]
        if len(candidates) != 1 or candidates[0].get('versionInfo') != config['qt_version']:
            raise ValueError('SDK module version/identity mismatch')
        package = candidates[0]
        source_spdx = prefix/'sbom'/f"{item['module']}-{config['qt_version']}.source.spdx"
        result = compare_sources(archive, item, source_spdx if source_spdx.exists() else None)
        result.update(sdk_build_spdx_sha256=digest(document),
                      declared_build=sanitize(package.get('comment', '')),
                      compiler=[{k: p[k] for k in ['name', 'versionInfo'] if k in p}
                                for p in data['packages'] if p.get('SPDXID') == 'SPDXRef-compiler'])
        modules.append(result)
    # Read actual installed feature/config inputs on all platforms, including
    # macOS framework headers via their SDK-contained aliases. No source mutation.
    inputs = []
    paths = set((prefix/'modules').glob('*.json')) | set((prefix/'mkspecs').glob('qconfig*.pri'))
    for base in [prefix/'include', prefix/'lib']:
        paths.update(base.rglob('*config*.h'))
    for path in sorted(paths):
        if not path.is_file() or not within(path.resolve(), prefix):
            continue
        relative = path.relative_to(prefix).as_posix()
        if path.suffix == '.h':
            defines = [line for line in path.read_text(encoding='utf-8').splitlines()
                       if re.match(r'#\s*define\s+QT_(?:FEATURE_|VERSION|NAMESPACE|SHARED|STATIC|NO_DEBUG)', line)]
            if not defines:
                continue
            content = '\n'.join(defines)
        elif path.suffix == '.json':
            metadata = json.loads(path.read_text(encoding='utf-8'))
            if metadata.get('version') != config['qt_version']:
                raise ValueError('SDK installed module version mismatch')
            content = json.dumps(metadata, sort_keys=True)
        else:
            content = path.read_text(encoding='utf-8')
        inputs.append({'file': relative, 'sha256': digest(path), 'configuration': sanitize(content)})
    if not inputs:
        raise ValueError('SDK configuration inventory missing')
    patches = [patch_status(prefix, item, cache/next(a['file'] for a in config['archives'] if a['module'] == item['module'])) for item in config.get('sdk_patches', [])
               if (prefix/item['sdk_path']).exists()]
    return {'schema_version': 1, 'status': 'evidence-collected-not-build-equivalence',
            'qt_version': config['qt_version'],
            'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
            'source_dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT).strip()),
            'source_config_sha256': digest(ROOT/'scripts/corresponding-sources.json'),
            'modules': modules, 'installed_configuration': inputs,
            'consumer_patches': patches,
            'license_text_provenance': json.loads((ROOT/'scripts/qt-license-sources.json').read_text())['source_commits'],
            'license_provenance_scope': 'License text retrieval commits only; archive commits are recorded per module.',
            'gaps': ['SDK source SPDX covers declared files only; omitted/generated files and source changes require review.',
                     'Installed configuration and upstream configure comments do not include the full original CMake cache, environment or toolchain inputs.',
                     'SDK metadata architecture is the upstream configure host/default; it does not prove the actual runtime binary slices.',
                     'Known consumer helper is checked; the complete SDK modification history and runtime binary derivation remain unverified.',
                     'No Qt source rebuild, runtime binary equivalence, license approval or independent security review is claimed.']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--cache', type=Path, default=ROOT/'build/source-cache')
    parser.add_argument('--fetch', action='store_true')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if not within(args.output.resolve(), ROOT/'build') or args.output.exists():
        parser.error('Use a new output file under build/')
    report = collect(args.qt_prefix.resolve(), args.cache.resolve(), args.fetch)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print('Collected Qt SDK/source correspondence evidence; build equivalence remains unverified')


if __name__ == '__main__':
    main()
