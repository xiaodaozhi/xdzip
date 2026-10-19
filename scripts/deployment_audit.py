#!/usr/bin/env python3
"""Read-only native dependency/file audit of an explicitly trusted local build.

Linux ldd must only inspect our own freshly built executables, never user inputs.
This is a loader/import snapshot, not a trace of all dynamically loaded plugins.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as file:
        for chunk in iter(lambda: file.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def within(path, root):
    return path.resolve().is_relative_to(root.resolve())


def run(*command):
    # Tool failures are summarized without printing paths from private SDKs.
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode:
        raise ValueError(f'{Path(command[0]).name} could not inspect a trusted binary')
    return result.stdout


def mac_imports(text):
    return [line.strip().split(' (compatibility version', 1)[0]
            for line in text.splitlines()[1:] if line.startswith(('\t', ' '))]


def mac_rpaths(text):
    return re.findall(r'cmd LC_RPATH\s+cmdsize \d+\s+path (.+?) \(offset \d+\)', text)


def windows_imports(text):
    # dumpbin also lists delay-load imports; avoid localized section headings.
    return sorted(set(re.findall(r'^\s*([\w.+-]+\.dll)\s*$', text, re.M | re.I)))


def linux_imports(text):
    result = []
    for line in text.splitlines():
        line = line.strip()
        if '=> not found' in line:
            result.append((line.split('=>')[0].strip(), None))
        elif '=>' in line:
            name, location = line.split('=>', 1)
            result.append((name.strip(), Path(location.strip().rsplit(' (', 1)[0])))
        elif line.startswith('/'):
            value = line.rsplit(' (', 1)[0]
            result.append((Path(value).name, Path(value)))
    return result


def native(path, platform):
    with path.open('rb') as file:
        magic = file.read(4)
    return (magic[:2] == b'MZ' if platform == 'windows' else
            magic == b'\x7fELF' if platform == 'linux' else
            magic in (b'\xcf\xfa\xed\xfe', b'\xfe\xed\xfa\xcf', b'\xca\xfe\xba\xbe', b'\xbe\xba\xfe\xca'))


def audit(root, app, qt_prefix, platform, mode):
    root, app = root.resolve(), app.resolve()
    qt_prefix = qt_prefix.resolve() if qt_prefix else None
    if not root.is_dir() or not app.is_file() or not within(app, root) or not native(app, platform):
        raise ValueError('Application must be a native binary within the declared artifact root')
    files, binaries = [], []
    for path in sorted(root.rglob('*')):
        if path.is_symlink() and not within(path, root):
            raise ValueError('Artifact contains a symlink outside its declared root')
        if not path.is_file():
            continue
        item = {'path': path.relative_to(root).as_posix(), 'bytes': path.stat().st_size,
                'sha256': digest(path), 'executable': bool(path.stat().st_mode & 0o111)}
        if path.is_symlink():
            item['symlink_target'] = os.path.relpath(path.resolve(), path.parent).replace('\\', '/')
        files.append(item)
        if native(path, platform):
            binaries.append(path.resolve())
    edges, inspected, external = [], set(), {}
    pending = list(dict.fromkeys(binaries))
    system_root = Path(os.environ.get('SystemRoot', 'C:/Windows')).resolve()

    def location(path):
        path = path.resolve()
        if within(path, root):
            return 'artifact/' + path.relative_to(root).as_posix(), 'artifact'
        if qt_prefix and within(path, qt_prefix):
            return 'qt-sdk/' + path.relative_to(qt_prefix).as_posix(), 'qt-sdk'
        system = (str(path).startswith(('/System/Library/', '/usr/lib/')) if platform == 'macos' else
                  within(path, system_root) if platform == 'windows' else
                  str(path).startswith(('/lib/', '/lib64/', '/usr/lib/', '/usr/lib64/')) and not path.name.startswith('libQt'))
        if system:
            return 'system/' + path.name, 'system'
        # A hash avoids leaking source usernames or conflating equal filenames.
        return 'external/' + digest(path)[:16] + '/' + path.name, 'external'

    while pending:
        binary = pending.pop()
        if binary in inspected:
            continue
        inspected.add(binary)
        source, _ = location(binary)
        if platform == 'macos':
            imports = [(name, None) for name in mac_imports(run('otool', '-L', str(binary)))]
            rpaths = mac_rpaths(run('otool', '-l', str(binary)))
            def expand(value):
                return Path(value.replace('@loader_path', str(binary.parent)).replace('@executable_path', str(app.parent)))
            search = [expand(p) for p in rpaths] + [root/'Contents/Frameworks']
            if qt_prefix:
                search.append(qt_prefix/'lib')
        elif platform == 'windows':
            imports = [(name, None) for name in windows_imports(run('dumpbin', '/nologo', '/dependents', str(binary))) ]
        else:
            imports = linux_imports(run('ldd', str(binary)))
        for name, resolved in imports:
            if platform == 'macos':
                if name.startswith(('/System/Library/', '/usr/lib/')):
                    edges.append({'from': source, 'import': Path(name).name, 'scope': 'system'})
                    continue # System dyld-cache images need not exist as files.
                candidates = ([base/name[len('@rpath/'):] for base in search] if name.startswith('@rpath/') else [expand(name)])
                resolved = next((p.resolve() for p in candidates if p.is_file()), None)
            elif platform == 'windows':
                if name.lower().startswith(('api-ms-win-', 'ext-ms-win-')):
                    edges.append({'from': source, 'import': name, 'scope': 'system-api-contract'})
                    continue
                candidates = [binary.parent/name, app.parent/name, system_root/'System32'/name]
                # Windows lookup is case-insensitive, including on parser-test hosts.
                resolved = next((p.resolve() for p in candidates if p.is_file()), None)
                if resolved is None:
                    for parent in [binary.parent, app.parent, system_root/'System32']:
                        if parent.is_dir():
                            resolved = next((p.resolve() for p in parent.iterdir() if p.name.lower() == name.lower()), None)
                            if resolved:
                                break
            if resolved is None or not resolved.is_file():
                edges.append({'from': source, 'import': Path(name).name, 'scope': 'unresolved'})
                continue
            target, scope = location(resolved)
            edges.append({'from': source, 'import': Path(name).name, 'to': target, 'scope': scope})
            if scope in ('qt-sdk', 'external'):
                external[target] = {'path': target, 'sha256': digest(resolved), 'bytes': resolved.stat().st_size}
            if scope in ('artifact', 'qt-sdk') and resolved not in inspected:
                pending.append(resolved)
    unresolved = sum(e['scope'] == 'unresolved' for e in edges)
    portable = not unresolved and not external
    return {'schema_version': 1, 'platform': platform, 'mode': mode,
            'app': app.relative_to(root).as_posix(), 'files': files, 'binary_imports': edges,
            'external_files': sorted(external.values(), key=lambda x:x['path']),
            'unresolved_imports': unresolved, 'uninspected_external_libraries': sum(not key.startswith('qt-sdk/') for key in external), 'sdk_independent_imports': portable,
            'status': 'passed' if not unresolved and (mode == 'sdk' or portable) else 'failed',
            'limits': ['Recursion is limited to the artifact and declared Qt SDK; other external libraries are hashed but not recursively inspected.',
                       'Native imports only; QML/plugins loaded by name and OS ABI compatibility require runtime and clean-machine checks.',
                       'SDK mode describes a developer build, not an independently installed application.',
                       'File hashes do not establish component identity, license compliance, signatures or security review.']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--qt-prefix', type=Path)
    parser.add_argument('--mode', choices=['sdk', 'deployed'], required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if within(args.output, args.root):
        parser.error('Write audit output outside the artifact root to avoid self-inventory')
    platform = 'windows' if sys.platform == 'win32' else 'macos' if sys.platform == 'darwin' else 'linux'
    try:
        report = audit(args.root, args.app, args.qt_prefix, platform, args.mode)
        cli = run(str(args.app.resolve()), '--version').strip()
        match = re.fullmatch(r'xdzip (\S+) \(build (\d+), ([0-9a-f]{12})(, dirty)?\)', cli)
        if not match:
            raise ValueError('Application has no verifiable build identity')
        report['identity'] = {'version':match[1], 'build':int(match[2]), 'revision':match[3], 'dirty':bool(match[4])}
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2)+'\n')
    except (ValueError, OSError):
        raise SystemExit('Deployment audit failed to inspect the declared trusted artifact; no readiness claim made')
    print(f'Deployment imports: {report["status"]}, {len(report["files"])} files, {report["unresolved_imports"]} unresolved, SDK-independent={report["sdk_independent_imports"]}')
    raise SystemExit(report['status'] != 'passed')

if __name__ == '__main__':
    main()
