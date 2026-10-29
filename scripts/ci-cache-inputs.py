#!/usr/bin/env python3
"""Verify bounded dependency inputs; never cache products, credentials or verdicts."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import stat
import sys

ROOT = Path(__file__).resolve().parents[1]
MAX_FILES = 100000
MAX_BYTES = 3 * 1024 ** 3


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def identity(scope, root=ROOT, env=None):
    env = os.environ if env is None else env
    policy = json.loads((root / 'scripts/ci-cache-policy.json').read_text(encoding='utf-8'))
    if policy['schema_version'] != 1 or scope not in policy['scopes']:
        raise ValueError('Unknown cache policy or scope')
    config = policy['scopes'][scope]
    sources = []
    for name in policy['common_inputs'] + config['inputs']:
        sources.append({'file': name, 'sha256': digest(root / name)})
    data = {'scope': scope, 'os': env.get('RUNNER_OS', platform.system()),
            'arch': env.get('RUNNER_ARCH', platform.machine()),
            'image_os': env.get('ImageOS', ''), 'image_version': env.get('ImageVersion', ''),
            'python': '.'.join(platform.python_version_tuple()[:2]), 'inputs': sources}
    fingerprint = hashlib.sha256(json.dumps(data, sort_keys=True).encode()).hexdigest()
    return {'key': 'xdzip-' + policy['version'] + '-' + scope + '-' + fingerprint,
            'fingerprint': fingerprint, 'scope': scope, 'host': data,
            'root_labels': config['roots']}


def roots(root=ROOT, env=None):
    env = os.environ if env is None else env
    cargo = Path(env.get('CARGO_HOME') or Path.home() / '.cargo')
    return {'cargo-archives': cargo / 'registry/cache', 'cargo-git': cargo / 'git/db',
            'pip': root / 'build/ci-input-cache/pip',
            'playwright': root / 'build/ci-input-cache/playwright',
            'qt-sources': root / 'build/source-cache'}


def is_link(path):
    return path.is_symlink() or (hasattr(path, 'is_junction') and path.is_junction())


def inventory(selected, paths):
    records = []
    total = 0
    for label in selected:
        base = paths[label]
        if is_link(base):
            raise ValueError('Cache root is a link')
        if not base.exists():
            continue
        if not base.is_dir():
            raise ValueError('Cache root is not a directory')
        for current, folders, files in os.walk(base, followlinks=False):
            for name in folders + files:
                path = Path(current) / name
                if is_link(path) or not path.resolve().is_relative_to(base.resolve()):
                    raise ValueError('Cache link or path escape')
            for name in files:
                path = Path(current) / name
                metadata = path.stat()
                if not stat.S_ISREG(metadata.st_mode):
                    raise ValueError('Cache special object')
                total += metadata.st_size
                if len(records) >= MAX_FILES or total > MAX_BYTES:
                    raise ValueError('Cache inventory budget exceeded')
                records.append({'root': label, 'file': path.relative_to(base).as_posix(),
                                'bytes': metadata.st_size, 'sha256': digest(path)})
    return sorted(records, key=lambda r: (r['root'], r['file']))


def seal(plan, paths, env):
    if (env.get('CI') != 'true' or env.get('GITHUB_EVENT_NAME') != 'push'
            or env.get('GITHUB_REF') != 'refs/heads/main'):
        raise ValueError('Only trusted main push may save CI input caches')
    return {'schema_version': 1, 'key': plan['key'], 'scope': plan['scope'],
            'host': plan['host'], 'repository': env.get('GITHUB_REPOSITORY', ''),
            'producer_event': 'push', 'producer_ref': 'refs/heads/main',
            'producer_run': env.get('GITHUB_RUN_ID', ''),
            'files': inventory(plan['root_labels'], paths)}


def verify(plan, paths, record, env):
    if (record.get('schema_version') != 1 or record.get('key') != plan['key']
            or record.get('scope') != plan['scope'] or record.get('host') != plan['host']
            or record.get('repository') != env.get('GITHUB_REPOSITORY', '')
            or record.get('producer_event') != 'push'
            or record.get('producer_ref') != 'refs/heads/main'):
        raise ValueError('Cache identity or trusted provenance differs')
    files = record.get('files')
    if not isinstance(files, list) or len(files) > MAX_FILES:
        raise ValueError('Invalid cache inventory')
    for item in files:
        relative = PurePosixPath(item.get('file', ''))
        if (item.get('root') not in plan['root_labels'] or not relative.parts
                or relative.is_absolute() or '..' in relative.parts
                or '\\' in item['file'] or ':' in item['file']
                or not re.fullmatch('[0-9a-f]{64}', item.get('sha256', ''))
                or type(item.get('bytes')) is not int or item['bytes'] < 0):
            raise ValueError('Unsafe cache inventory entry')
    actual = inventory(plan['root_labels'], paths)
    if actual != files:
        raise ValueError('Cached dependency bytes or file set differ')
    return len(actual)


def github_output(name, value):
    path = os.environ.get('GITHUB_OUTPUT')
    if not path:
        raise ValueError('CI output channel is missing')
    with Path(path).open('a', encoding='utf-8') as output:
        if '\n' in value:
            delimiter = 'xdzip_' + hashlib.sha256(value.encode()).hexdigest()
            output.write(name + '<<' + delimiter + '\n' + value + '\n' + delimiter + '\n')
        else:
            output.write(name + '=' + value + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['plan', 'verify', 'seal'])
    parser.add_argument('--scope', required=True, choices=['foundation', 'qt', 'html'])
    parser.add_argument('--matched-key', default='')
    args = parser.parse_args()
    if os.environ.get('CI') != 'true' or os.environ.get('GITHUB_ACTIONS') != 'true':
        raise ValueError('Cache orchestration is restricted to GitHub CI')
    plan = identity(args.scope)
    paths = roots()
    manifest = ROOT / 'build/ci-input-cache/manifests' / (args.scope + '.json')
    if args.operation == 'plan':
        github_output('key', plan['key'])
        github_output('paths', '\n'.join(str(paths[r]) for r in plan['root_labels']) + '\n' + str(manifest))
    elif args.operation == 'verify':
        hit = bool(args.matched_key)
        if hit:
            if args.matched_key != plan['key']:
                raise ValueError('Partial cache key is not accepted')
            count = verify(plan, paths, json.loads(manifest.read_text(encoding='utf-8')), os.environ)
        else:
            count = 0
        target = ROOT / 'build/review' / ('ci-cache-' + args.scope + '.json')
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(json.dumps({'schema_version': 1, 'scope': args.scope,
                          'cache_hit': hit, 'key': plan['key'], 'verified_files': count,
                          'status': 'verified' if hit else 'cold-cache'}, indent=2) + '\n', encoding='utf-8')
    else:
        record = seal(plan, paths, os.environ)
        manifest.parent.mkdir(parents=True, exist_ok=True)
        manifest.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
        github_output('save', 'true' if record['files'] else 'false')
    print(json.dumps({'scope': args.scope, 'operation': args.operation, 'status': 'passed'}))


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError) as error:
        # Paths and cache payloads may contain host information; keep them out of logs.
        print(json.dumps({'status': 'failed', 'error_type': type(error).__name__}), file=sys.stderr)
        raise SystemExit(1)
