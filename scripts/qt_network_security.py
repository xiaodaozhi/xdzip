"""Pinned QtNetwork security build, preserving the installed SDK's other modules."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
RECEIPT = 'xdzip-network-security.json'
MACOS_NETWORK_TOOLS = ('libexec/qmlimportscanner', 'libexec/qmlcachegen', 'bin/qmllint')
BUILDER_FILES = ('scripts/qt_network_security.py', 'scripts/prepare-qt-network.py',
                 'scripts/check-qt-network.py', 'tools/qt-network-security/CMakeLists.txt',
                 'tools/qt-network-security/main.cpp', 'scripts/update_proxy_fixture.py',
                 'scripts/qt_sdk_correspondence.py',
                 'scripts/corresponding-sources.json', 'scripts/qt-patches/clang-arm-acle.patch')


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def configuration():
    data = json.loads((ROOT/'scripts/qt-network-security.json').read_bytes())
    sources = json.loads((ROOT/'scripts/corresponding-sources.json').read_bytes())
    source = next(x for x in sources['archives'] if x['module'] == 'qtbase')
    if data['qt_version'] != sources['qt_version']:
        raise ValueError('Qt security source version mismatch')
    for item in sources.get('sdk_patches', []):
        if item['patch'] == 'scripts/qt-patches/clang-arm-acle.patch' and digest(ROOT/item['patch']) != item['patch_sha256']:
            raise ValueError('Pinned Clang compatibility patch changed')
    for patch in data['patches']:
        path = ROOT/patch['file']
        if not path.resolve().is_relative_to(ROOT/'patches/qt') or digest(path) != patch['sha256']:
            raise ValueError('Qt security patch identity mismatch')
    inputs = {name: digest(ROOT/name) for name in BUILDER_FILES}
    inputs.update({x['file']: x['sha256'] for x in data['patches']})
    inputs['scripts/qt-network-security.json'] = digest(ROOT/'scripts/qt-network-security.json')
    inputs['source'] = source['sha256']
    return data, source, hashlib.sha256(json.dumps(inputs, sort_keys=True).encode()).hexdigest()


def library(prefix, module, system=None):
    system = system or sys.platform
    relative = (f'lib/Qt{module}.framework/Versions/A/Qt{module}' if system == 'darwin' else
                f'bin/Qt6{module}.dll' if system == 'win32' else f'lib/libQt6{module}.so.6.8.2')
    path = Path(prefix)/relative
    if not path.is_file() or not path.resolve().is_relative_to(Path(prefix).resolve()):
        raise ValueError('Required SDK module missing or outside its prefix: '+module)
    return path


def features(prefix):
    base = (Path(prefix)/'lib/QtNetwork.framework/Versions/A/Headers' if sys.platform == 'darwin'
            else Path(prefix)/'include/QtNetwork')
    files = [base/'qtnetwork-config.h', base/'6.8.2/QtNetwork/private/qtnetwork-config_p.h']
    result = {}
    for path in files:
        for name, value in re.findall(r'^\s*#\s*define\s+QT_FEATURE_(\w+)\s+(-?1)\s*$', path.read_text(), re.M):
            if name in result and result[name] != value:
                raise ValueError('Conflicting QtNetwork feature')
            result[name] = value
    if result.get('ssl') != '1' or result.get('http') != '1':
        raise ValueError('QtNetwork must retain TLS and HTTP')
    return result


def core_features(prefix):
    base = (Path(prefix)/'lib/QtCore.framework/Versions/A/Headers' if sys.platform == 'darwin'
            else Path(prefix)/'include/QtCore')
    result = {}
    for name in ('qconfig.h', 'qtcore-config.h', '6.8.2/QtCore/private/qconfig_p.h',
                 '6.8.2/QtCore/private/qtcore-config_p.h'):
        path = base/name
        if path.is_file():
            result.update(re.findall(r'^\s*#\s*define\s+QT_FEATURE_(\w+)\s+(-?1)\s*$', path.read_text(), re.M))
    return result


def identity(prefix, *, required=True):
    path = Path(prefix)/RECEIPT
    if not path.is_file():
        if required:
            raise ValueError('Prepare the pinned QtNetwork security build before using this SDK')
        return None
    if path.is_symlink():
        raise ValueError('QtNetwork security receipt must be a regular file')
    config, source, inputs = configuration()
    data = json.loads(path.read_bytes())
    if not isinstance(data, dict) or not isinstance(data.get('verification'), dict):
        raise ValueError('Invalid QtNetwork security receipt')
    if (data.get('schema_version') != 1 or data.get('variant') != config['variant'] or
            data.get('inputs_sha256') != inputs or data.get('source_sha256') != source['sha256'] or
            data.get('core_sha256') != digest(library(prefix, 'Core')) or
            data.get('core_features') != core_features(prefix) or
            data.get('network_sha256') != digest(library(prefix, 'Network')) or
            data.get('patches') != config['patches'] or data.get('features') != features(prefix) or
            data['verification'].get('network_sha256') != data.get('network_sha256') or
            data['verification'].get('status') != 'passed'):
        raise ValueError('QtNetwork security receipt differs from its inputs or actual SDK binaries')
    tools = data.get('sdk_tools', [])
    expected_tools = set(MACOS_NETWORK_TOOLS) if sys.platform == 'darwin' else set()
    if {item['sdk_path'] for item in tools} != expected_tools:
        raise ValueError('QtNetwork SDK tool seal inventory differs')
    for item in tools:
        tool = Path(prefix)/item['sdk_path']
        if not tool.resolve().is_relative_to(Path(prefix).resolve()) or digest(tool) != item['sha256']:
            raise ValueError('QtNetwork SDK tool changed after preparation')
    return data


def apply_patch_files(files, patch):
    """Replay exact, uniquely matched hunks; never use fuzzy external patch tools."""
    sections = re.split(r'^diff --git ', patch.decode(), flags=re.M)[1:]
    if not sections:
        raise ValueError('Missing Qt patch sections')
    for section in sections:
        lines = ('diff --git '+section).splitlines(keepends=True)
        match = re.fullmatch(r'diff --git a/(\S+) b/(\S+)\n', lines[0])
        if (not match or match[1] != match[2] or '..' in match[1].split('/') or
                not match[1].startswith(('src/network/', 'tests/auto/network/'))):
            raise ValueError('Qt patch escapes its bounded network scope')
        name = match[1]
        if name not in files:
            raise ValueError('Qt patch input missing: '+name)
        original = files[name].decode().splitlines(keepends=True)
        cursor = index = 0
        hunks = 0
        while index < len(lines):
            if not lines[index].startswith('@@ '):
                index += 1
                continue
            hunk = re.match(r'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@', lines[index])
            if not hunk:
                raise ValueError('Invalid Qt patch hunk')
            index += 1
            before, after = [], []
            while index < len(lines) and lines[index][:1] in (' ', '+', '-'):
                line = lines[index]
                if line.startswith(('--- ', '+++ ')):
                    break
                if line[0] in ' -':
                    before.append(line[1:])
                if line[0] in ' +':
                    after.append(line[1:])
                index += 1
            if not before or len(before) != int(hunk[2] or 1) or len(after) != int(hunk[4] or 1):
                raise ValueError('Qt patch hunk count mismatch')
            matches = [i for i in range(cursor, len(original)-len(before)+1)
                       if original[i:i+len(before)] == before]
            if len(matches) != 1:
                raise ValueError('Qt patch context must match exactly once: '+name)
            offset = matches[0]
            original[offset:offset+len(before)] = after
            cursor = offset+len(after)
            hunks += 1
        if not hunks:
            raise ValueError('Qt patch contains no supported hunks')
        files[name] = ''.join(original).encode()


def prepare_source(config, source, cache, work):
    cache.mkdir(parents=True, exist_ok=True)
    archive = cache/source['file']
    if not archive.exists():
        temporary = archive.with_suffix(archive.suffix+'.download')
        try:
            with urllib.request.urlopen(source['url'], timeout=60) as reply, temporary.open('wb') as stream:
                shutil.copyfileobj(reply, stream)
            if digest(temporary) != source['sha256']:
                raise ValueError('Qt source download checksum mismatch')
            os.replace(temporary, archive)
        finally:
            temporary.unlink(missing_ok=True)
    if digest(archive) != source['sha256']:
        raise ValueError('Qt source archive checksum mismatch')
    directory = work/source['file'].removesuffix('.tar.xz')
    prefix = directory.name+'/'
    patches = [ROOT/x['file'] for x in config['patches']]
    names = {m[0] for path in patches for m in re.findall(r'^diff --git a/(\S+) b/(\S+)$', path.read_text(), re.M)}
    with tarfile.open(archive) as stream:
        changed = {name: stream.extractfile(prefix+name).read() for name in names}
        for path in patches:
            apply_patch_files(changed, path.read_bytes())
        if sys.platform == 'darwin':
            # Existing pinned build-only compatibility for current native Clang.
            from qt_sdk_correspondence import replay_patch
            name = 'src/corelib/thread/qyieldcpu.h'
            changed[name] = replay_patch(stream.extractfile(prefix+name).read(),
                                       (ROOT/'scripts/qt-patches/clang-arm-acle.patch').read_bytes())
        if not directory.exists():
            stream.extractall(work, filter='data')
        known = set()
        for member in stream.getmembers():
            if member.isdir():
                continue
            if not member.name.startswith(prefix):
                raise ValueError('Unexpected Qt source layout')
            relative = member.name[len(prefix):]
            known.add(relative)
            target = directory/relative
            if not target.resolve().is_relative_to(directory.resolve()):
                raise ValueError('Qt source member escapes extraction')
            if member.isfile():
                before = stream.extractfile(member).read()
                expected = changed.get(relative, before)
                if target.is_symlink() or not target.is_file():
                    raise ValueError('Qt source member replaced')
                actual = target.read_bytes()
                if actual == before and actual != expected:
                    target.write_bytes(expected)
                elif actual != expected:
                    raise ValueError('Unexpected modified Qt source member: '+relative)
            elif member.issym() and (not target.is_symlink() or os.readlink(target) != member.linkname):
                raise ValueError('Qt source link replaced')
        actual = {p.relative_to(directory).as_posix() for p in directory.rglob('*') if p.is_file() or p.is_symlink()}
        if actual != known:
            raise ValueError('Qt source contains missing or extra inputs')
    return directory


def run(arguments, log):
    environment = dict(os.environ)
    for secret in ('XDZIP_UPDATE_SIGNING_KEY_HEX', 'XDZIP_SPARKLE_SIGNING_KEY_HEX'):
        environment.pop(secret, None)
    result = subprocess.run(list(map(str, arguments)), capture_output=True, env=environment)
    raw = (result.stdout+result.stderr).replace(str(ROOT).encode(), b'<workspace>')
    raw = raw.replace(str(Path.home()).encode(), b'<user>')
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_bytes(raw)
    if result.returncode:
        raise ValueError('QtNetwork command failed; inspect '+log.name)
    return result.stdout.decode(errors='replace')


def openssl_headers(config, cache, work):
    """Generate pinned OpenSSL 3 declarations; never build or ship its libraries."""
    item = config['openssl_headers']
    archive = Path(cache)/item['file']
    if not archive.exists():
        temporary = archive.with_suffix('.download')
        try:
            with urllib.request.urlopen(item['url'], timeout=60) as reply, temporary.open('wb') as stream:
                shutil.copyfileobj(reply, stream)
            if digest(temporary) != item['sha256']:
                raise ValueError('OpenSSL declaration source checksum mismatch')
            os.replace(temporary, archive)
        finally:
            temporary.unlink(missing_ok=True)
    if digest(archive) != item['sha256']:
        raise ValueError('OpenSSL declaration source checksum mismatch')
    target = {'darwin': 'darwin64-arm64-cc', 'win32': 'VC-WIN64A', 'linux': 'linux-x86_64'}.get(sys.platform)
    if not target:
        raise ValueError('Unsupported native QtNetwork security build platform')
    directory = work/item['file'].removesuffix('.tar.gz')
    record = work/'headers-receipt.json'
    if record.exists():
        data = json.loads(record.read_bytes())
        if not data.get('headers'):
            raise ValueError('Missing generated OpenSSL declaration inventory')
        for name in data['headers']:
            path = directory/name
            if not name.startswith('include/') or not path.resolve().is_relative_to(directory) or path.is_symlink():
                raise ValueError('Unbounded OpenSSL declaration inventory')
        if (data['source_sha256'] != item['sha256'] or data['target'] != target or
                any(digest(directory/name) != expected for name, expected in data['headers'].items())):
            raise ValueError('Generated OpenSSL declaration inputs changed; use a fresh owned build directory')
        return directory/'include'
    with tarfile.open(archive) as stream:
        if not directory.exists():
            stream.extractall(work, filter='data')
        for member in stream:
            if member.isfile():
                path = work/member.name
                if (not path.resolve().is_relative_to(directory) or path.is_symlink() or not path.is_file() or
                        path.read_bytes() != stream.extractfile(member).read()):
                    raise ValueError('OpenSSL declaration source input changed')
    # Configure writes generated files only. The make target expands templates;
    # no crypto library or executable is compiled or added to the distribution.
    environment = dict(os.environ)
    for secret in ('XDZIP_UPDATE_SIGNING_KEY_HEX', 'XDZIP_SPARKLE_SIGNING_KEY_HEX'):
        environment.pop(secret, None)
    for label, command in [('configure', ['perl', 'Configure', target, 'no-tests', 'no-asm']),
                           ('generate', ['nmake' if sys.platform == 'win32' else 'make', 'build_generated'])]:
        result = subprocess.run(command, cwd=directory, env=environment, capture_output=True)
        raw = (result.stdout+result.stderr).replace(str(ROOT).encode(), b'<workspace>').replace(str(Path.home()).encode(), b'<user>')
        (work/('headers-'+label+'.log')).write_bytes(raw)
        if result.returncode:
            raise ValueError('OpenSSL declaration generation failed: '+label)
    data = dict(source_sha256=item['sha256'], target=target,
                headers={p.relative_to(directory).as_posix(): digest(p) for p in (directory/'include').rglob('*.h')})
    record.write_text(json.dumps(data, indent=2)+'\n')
    return directory/'include'


def symbol_names(path, *, undefined=False):
    if sys.platform == 'win32':
        output = subprocess.check_output(['dumpbin', '/nologo', '/imports' if undefined else '/exports', str(path)], text=True)
        pattern = r'^\s+[0-9A-Fa-f]+\s+(\S+)\s*$' if undefined else r'^\s+\d+\s+[0-9A-Fa-f]+\s+[0-9A-Fa-f]+\s+(\S+)'
        return set(re.findall(pattern, output, re.M))
    args = ['nm', '-arch', 'arm64', '-gu' if undefined else '-gU'] if sys.platform == 'darwin' else ['nm', '-D', '--undefined-only' if undefined else '--defined-only']
    output = subprocess.check_output(args+[str(path)], text=True, stderr=subprocess.DEVNULL)
    return {line.split()[-1].replace('@@', '@') for line in output.splitlines() if line.split() and not line.endswith(':')}


def check_abi(prefix, built):
    original = symbol_names(library(prefix, 'Network'))
    candidate = symbol_names(library(built, 'Network'))
    imports = symbol_names(library(built, 'Network'), undefined=True)
    built_core = symbol_names(library(built, 'Core'))
    sdk_core = symbol_names(library(prefix, 'Core'))
    missing_exports = original-candidate
    missing_core = (imports & built_core)-sdk_core
    if missing_exports or missing_core:
        raise ValueError('QtNetwork ABI differs: '+json.dumps({'missing_network': sorted(missing_exports), 'missing_core': sorted(missing_core)}))
    return {'network_exports_preserved': len(original), 'core_imports_confirmed': len(imports & built_core)}


def build(prefix, work, source, parallel, headers):
    expected_features = features(prefix)
    flags = ['-DCMAKE_BUILD_TYPE=Release', '-DQT_BUILD_TESTS=OFF', '-DQT_BUILD_EXAMPLES=OFF',
             '-DBUILD_WITH_PCH=OFF', '-DWARNINGS_ARE_ERRORS=OFF', '-DCMAKE_POLICY_VERSION_MINIMUM=3.16',
             '-DCMAKE_INSTALL_PREFIX='+str(work/'stage'), '-DFEATURE_openssl=ON',
             '-DFEATURE_openssl_runtime=ON', '-DOPENSSL_INCLUDE_DIR='+str(headers)]
    flags += ['-DFEATURE_'+name+'=OFF' for name in ('gui', 'widgets', 'sql', 'xml', 'dbus', 'concurrent', 'testlib', 'printsupport')]
    flags += ['-DFEATURE_'+name+'='+('ON' if value == '1' else 'OFF') for name, value in sorted(expected_features.items())]
    existing_core = core_features(prefix)
    for name in ('system_zlib', 'system_pcre2', 'icu', 'glib', 'zstd'):
        if name in existing_core:
            flags += ['-DFEATURE_'+name+'='+('ON' if existing_core[name] == '1' else 'OFF')]
    if sys.platform == 'darwin':
        flags += ['-DCMAKE_OSX_ARCHITECTURES=arm64', '-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0',
                  '-DQT_FORCE_WARN_APPLE_SDK_AND_XCODE_CHECK=ON']
    run(['cmake', '--fresh', '-S', source, '-B', work/'obj', '-G', 'Ninja', *flags], work/'configure.log')
    run(['cmake', '--build', work/'obj', '--target', 'Network', '--parallel', parallel], work/'build.log')
    abi = check_abi(prefix, work/'obj')
    if (work/'stage').exists():
        shutil.rmtree(work/'stage')
    run(['cmake', '--install', work/'obj/src/network', '--prefix', work/'stage'], work/'install.log')
    if features(work/'stage') != expected_features:
        raise ValueError('Source-built QtNetwork features differ from the installed SDK')
    return flags, abi


def prepare(prefix, work, cache, parallel=3):
    prefix, work = Path(prefix).resolve(), Path(work).resolve()
    if not work.is_relative_to(ROOT/'build') or work == ROOT/'build' or not 1 <= parallel <= 12:
        raise ValueError('Use an owned build/ directory and 1–12 jobs')
    config, source, inputs = configuration()
    if (prefix/RECEIPT).exists():
        try:
            return identity(prefix)
        except ValueError:
            pass
    library(prefix, 'Core'); network = library(prefix, 'Network')
    work.mkdir(parents=True, exist_ok=True)
    lock = work/'prepare.lock'
    descriptor = os.open(lock, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    os.close(descriptor)
    try:
        patched = prepare_source(config, source, Path(cache), work)
        headers = openssl_headers(config, Path(cache), work)
        flags, abi = build(prefix, work, patched, parallel, headers)
        previous = json.loads((prefix/RECEIPT).read_bytes()) if (prefix/RECEIPT).is_file() else None
        original_sha = previous.get('upstream_network_sha256') if previous else digest(network)
        backup = work/'rollback-network.bin'; shutil.copy2(network, backup)
        tools_before = []
        if sys.platform == 'darwin':
            previous_tools = {item['sdk_path']: item for item in (previous or {}).get('sdk_tools', [])}
            for index, relative in enumerate(MACOS_NETWORK_TOOLS):
                tool = prefix/relative
                if tool.is_symlink() or not tool.resolve().is_relative_to(prefix) or not tool.is_file():
                    raise ValueError('Required QML SDK tool missing or replaced')
                saved = work/('rollback-tool-'+str(index)+'.bin'); shutil.copy2(tool, saved)
                before = digest(tool)
                upstream = previous_tools.get(relative, {}).get('upstream_sha256', before)
                tools_before.append((tool, saved, upstream))
        receipt_before = (prefix/RECEIPT).read_bytes() if (prefix/RECEIPT).is_file() else None
        signature = network.parent/'_CodeSignature/CodeResources'
        signature_before = signature.read_bytes() if signature.exists() else None
        replacement = network.with_name(network.name+'.xdzip-new')
        try:
            # Atomic file commit occurs only after source, feature and ABI checks.
            # Exceptions restore the prior binary/seal/receipt. A killed process
            # cannot yield a valid receipt: all consumers verify actual hashes.
            shutil.copy2(library(work/'stage', 'Network'), replacement)
            os.replace(replacement, network)
            (prefix/RECEIPT).unlink(missing_ok=True)
            if sys.platform == 'darwin':
                run(['codesign', '--force', '--sign', '-', '--timestamp=none', network.parents[2]], work/'seal.log')
                run(['codesign', '--verify', '--strict', network.parents[2]], work/'seal-check.log')
                for index, (tool, _, _) in enumerate(tools_before):
                    # Qt's vendor-signed build tools use library validation and
                    # cannot load a locally rebuilt framework with another team.
                    # Only these three SDK executables are locally re-sealed;
                    # product and runtime hardening policies are not changed.
                    run(['codesign', '--force', '--sign', '-', '--options', '0', '--timestamp=none', tool],
                        work/('tool-seal-'+str(index)+'.log'))
                    run(['codesign', '--verify', '--strict', tool], work/('tool-seal-check-'+str(index)+'.log'))
                    smoke = work/'tool-smoke'; smoke.mkdir(exist_ok=True)
                    arguments = ['-rootPath', smoke] if tool.name == 'qmlimportscanner' else ['--help']
                    run([tool, *arguments], work/('tool-start-'+str(index)+'.log'))
            run([sys.executable, ROOT/'scripts/check-qt-network.py', '--qt-prefix', prefix,
                 '--source', patched, '--output', work/'verification'], work/'verification.log')
            verification = json.loads((work/'verification/result.json').read_bytes())
            data = dict(schema_version=1, variant=config['variant'], inputs_sha256=inputs,
                        source_sha256=source['sha256'], patches=config['patches'],
                        core_sha256=digest(library(prefix, 'Core')), network_sha256=digest(network),
                        core_features=core_features(prefix),
                        upstream_network_sha256=original_sha, features=features(prefix), abi=abi,
                        sdk_tools=[dict(sdk_path=tool.relative_to(prefix).as_posix(), upstream_sha256=upstream,
                                        sha256=digest(tool), seal='ad-hoc, build tool only')
                                   for tool, _, upstream in tools_before],
                        configure_flags=[x.replace(str(work), '<build>') for x in flags], verification=verification)
            temporary = prefix/(RECEIPT+'.new')
            temporary.write_text(json.dumps(data, indent=2)+'\n')
            os.replace(temporary, prefix/RECEIPT)
            return identity(prefix)
        except BaseException:
            shutil.copy2(backup, replacement); os.replace(replacement, network)
            for tool, saved, _ in tools_before:
                shutil.copy2(saved, tool)
            if signature_before is not None:
                signature.parent.mkdir(exist_ok=True); signature.write_bytes(signature_before)
            if receipt_before is not None:
                (prefix/RECEIPT).write_bytes(receipt_before)
            else:
                (prefix/RECEIPT).unlink(missing_ok=True)
            raise
        finally:
            replacement.unlink(missing_ok=True)
    finally:
        lock.unlink(missing_ok=True)
