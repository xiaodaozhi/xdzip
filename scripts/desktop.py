#!/usr/bin/env python3
"""Shared local/CI desktop stages. Run --help before preparing optional downloads."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import urllib.request
import uuid

from desktop_environment import PYTHON_PACKAGES, QT_VERSION, check_environment, locked_rust_version
from association_types import WINDOWS_ICONS

ROOT = Path(__file__).resolve().parents[1]
TOOL_PACKAGES = tuple(name+'=='+version for name, version in PYTHON_PACKAGES.items())
AQT_PACKAGE = 'aqtinstall==3.3.0'
PLATFORMS = {
    'Darwin': ('macos', 'mac', 'clang_64', 'macos'),
    'Windows': ('windows', 'windows', 'win64_msvc2022_64', 'msvc2022_64'),
    'Linux': ('linux', 'linux', 'linux_gcc_64', 'gcc_64'),
}


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def generated_path(value, *, cache=False):
    """Never create workflow outputs in source/document or arbitrary user folders."""
    path = Path(value).resolve()
    roots = [ROOT/'build'] + ([ROOT/'.cache'] if cache else [])
    if not any(path.is_relative_to(root) and path != root for root in roots):
        raise ValueError('Use a dedicated directory under build/' +
                         (' or .cache/ for tools' if cache else ''))
    return path


def label(path):
    path = Path(path)
    try:
        return path.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return path.name


class Workflow:
    def __init__(self, args, *, system=None):
        self.args = args
        self.preflights = []
        actual = PLATFORMS.get(platform.system())
        if actual is None and system is None:
            raise ValueError('Unsupported native build host')
        self.system = system or actual[0]
        self.platform_spec = next(v for v in PLATFORMS.values() if v[0] == self.system)
        default_qt = ROOT/'.cache/Qt'/QT_VERSION/self.platform_spec[3]
        self.qt = Path(args.qt_prefix or os.environ.get('XDZIP_QT_PREFIX') or default_qt).resolve()
        self.build = generated_path(args.build_dir)
        self.output = generated_path(args.output_dir)
        # Explicit outside tool caches are allowed for CI ephemeral directories.
        # They only receive a venv when prepare is explicitly selected.
        self.tools = Path(args.tools_dir).resolve()
        if self.tools == ROOT or self.tools in ROOT.parents:
            raise ValueError('Tools directory must be a dedicated environment')
        self.tools_python = self.tools/('Scripts/python.exe' if self.system == 'windows' else 'bin/python')
        self.python = str(self.tools_python if self.tools_python.is_file() else Path(sys.executable))
        self.env = os.environ.copy()
        self.env['PATH'] = str(self.qt/'bin') + os.pathsep + self.env.get('PATH', '')
        if self.system == 'linux':
            self.env['LD_LIBRARY_PATH'] = str(self.qt/'lib')
        if self.system == 'macos':
            self.env['MACOSX_DEPLOYMENT_TARGET'] = '14.0'
        suffix = '.exe' if self.system == 'windows' else ''
        desktop = self.build/'apps/desktop'
        self.app = desktop/('xdzip.app/Contents/MacOS/xdzip' if self.system == 'macos' else 'xdzip'+suffix)
        self.tests = desktop/('xdzip-browser-tests'+suffix)
        self.runtime = self.output/self.system
        self.runtime_tools = ROOT/'build/windows-runtime'
        self.installer_tools = ROOT/'build/installer-toolchain'

    def run(self, command, *, capture=False, env=None, cwd=None):
        command = [str(x) for x in command]
        # Print only the program/stage, not private absolute SDK/source paths.
        if not capture:
            print('Running ' + Path(command[0]).name +
                  (' ' + Path(command[1]).name if len(command) > 1 and command[1].endswith('.py') else ''), flush=True)
        result = subprocess.run(command, cwd=cwd or ROOT, env=env or self.env,
                                capture_output=capture, text=True, check=False)
        if result.returncode:
            raise ValueError(Path(command[0]).name + ' failed with exit code ' + str(result.returncode))
        return result.stdout if capture else None

    def script(self, name, *args, **kwargs):
        return self.run([self.python, ROOT/'scripts'/name, *args], **kwargs)

    def deployed_env(self):
        env = self.env.copy()
        for name in ('LD_LIBRARY_PATH', 'DYLD_LIBRARY_PATH', 'QT_PLUGIN_PATH',
                     'QT_QPA_PLATFORM_PLUGIN_PATH', 'QML_IMPORT_PATH', 'QML2_IMPORT_PATH'):
            env.pop(name, None)
        env['PATH'] = os.pathsep.join(entry for entry in env.get('PATH', '').split(os.pathsep)
                                      if Path(entry).resolve() != (self.qt/'bin').resolve())
        return env

    def source(self):
        revision = self.run(['git', 'rev-parse', 'HEAD'], capture=True).strip()
        status = self.run(['git', 'status', '--porcelain', '--untracked-files=normal'], capture=True)
        diff = self.run(['git', 'diff', 'HEAD', '--binary'], capture=True)
        state = hashlib.sha256((status + diff).encode())
        # Include untracked source bytes, not just their names. Ignored build
        # outputs never participate in the source identity.
        untracked = self.run(['git', 'ls-files', '--others', '--exclude-standard', '-z'], capture=True)
        for relative in sorted(filter(None, untracked.split('\0'))):
            path = ROOT/relative
            state.update(relative.encode())
            if path.is_symlink():
                state.update(os.readlink(path).encode())
            elif path.is_file():
                state.update(bytes.fromhex(digest(path)))
        return {'revision': revision, 'dirty': bool(status), 'fingerprint': state.hexdigest()}

    def doctor(self, stage):
        report = check_environment(self.qt, stage=stage, system=self.system,
                                   python_executable=self.python, env=self.env)
        self.preflights.append({'stage': stage, **report})
        for item in report['checks']:
            print(item['status'].upper() + ' ' + item['name'] +
                  (' '+item['version'] if item.get('version') else '') +
                  (': ' + item['hint'] if item.get('hint') else ''), flush=True)
        if report['status'] != 'passed':
            raise ValueError('Environment preflight failed; fix the listed checks before ' + stage)
        return report

    def require_clean(self):
        source = self.source()
        if source['dirty']:
            raise ValueError('Packaging requires clean source. Commit intended changes, then run build again.')
        return source

    def build_receipt(self):
        path = self.build/'desktop-build.json'
        if not path.is_file():
            raise ValueError('Run the shared build stage first; no desktop-build.json receipt exists')
        receipt = json.loads(path.read_text(encoding='utf-8'))
        if receipt.get('status') != 'passed' or receipt.get('source') != self.source():
            raise ValueError('Source differs from the successful build; run build again')
        if receipt.get('configuration') != self.args.configuration or receipt.get('platform') != self.system:
            raise ValueError('Build configuration or native platform differs; run build again')
        if receipt.get('qt_sdk') != self.sdk_identity():
            raise ValueError('Qt SDK differs from the successful build; run build again')
        if not self.app.is_file() or receipt.get('product_sha256') != digest(self.app):
            raise ValueError('Product differs from the successful build; run build again')
        cache = self.build/'CMakeCache.txt'
        if not cache.is_file() or receipt.get('cmake_cache_sha256') != digest(cache):
            raise ValueError('CMake configuration changed; run build again')
        return receipt

    def sdk_identity(self):
        qmake = self.qt/'bin'/('qmake.exe' if self.system == 'windows' else 'qmake')
        return {'prefix_key': hashlib.sha256(str(self.qt).encode()).hexdigest(),
                'version': QT_VERSION, 'qmake_sha256': digest(qmake)}

    def check_cached_sdk(self, *, configured=False):
        cache = self.build/'CMakeCache.txt'
        if not cache.is_file():
            if configured:
                raise ValueError('CMake did not produce a configuration cache')
            return
        entries = re.findall(r'^(Qt6\w*_DIR):(?:PATH|UNINITIALIZED)=(.*)$',
                             cache.read_text(encoding='utf-8'), re.M)
        if configured and not any(name == 'Qt6_DIR' for name, _ in entries):
            raise ValueError('Configured build does not declare its Qt SDK')
        for _, value in entries:
            if value and not value.endswith('-NOTFOUND') and not Path(value).resolve().is_relative_to(self.qt):
                raise ValueError('CMake cached a different Qt SDK; use a new --build-dir for this --qt-prefix')

    def prepare(self):
        self.doctor('prepare')
        if self.args.install_rust:
            channel = locked_rust_version()
            self.run(['rustup', 'toolchain', 'install', channel, '--profile', 'minimal',
                      '--component', 'rustfmt', '--component', 'clippy'])
        if self.args.install_qt or self.args.packaging:
            if not self.tools_python.is_file():
                if self.tools.exists() and any(self.tools.iterdir()):
                    raise ValueError('Tools directory exists without a venv; choose a dedicated --tools-dir')
                self.run([sys.executable, '-m', 'venv', self.tools])
            self.python = str(self.tools_python)
            packages = list(TOOL_PACKAGES) + ([AQT_PACKAGE] if self.args.install_qt else [])
            self.run([self.python, '-m', 'pip', 'install', *packages])
        if self.args.install_qt and not (self.qt/'bin'/('qmake.exe' if self.system == 'windows' else 'qmake')).is_file():
            qt_root = Path(self.args.qt_root).resolve() if self.args.qt_root else self.qt.parents[1]
            expected = qt_root/QT_VERSION/self.platform_spec[3]
            if expected != self.qt:
                raise ValueError('--qt-prefix must match --qt-root/6.8.2/platform-folder when installing Qt')
            downloads = ROOT/'build/qt-install-tool'
            downloads.mkdir(parents=True, exist_ok=True)
            self.run([self.python, '-m', 'aqt', 'install-qt', self.platform_spec[1], 'desktop',
                      QT_VERSION, self.platform_spec[2], '-O', qt_root], cwd=downloads)
        self.doctor('build')
        self.run(['cargo', 'fetch', '--locked'])
        self.script('prepare-native-codecs.py')
        if self.args.packaging:
            # Fetch/cache small license and schema inputs before costly deployment.
            self.script('prepare-runtime-notices.py', '--qt-prefix', self.qt,
                        '--output', self.output/'prepared-qt-notices')
            self.prepare_schemas()
            if self.system == 'windows':
                self.script('prepare-windows-runtime.py', '--output', self.runtime_tools)
                self.run(['pwsh', '-File', ROOT/'scripts/prepare-windows-installer.ps1',
                          '-OutputDirectory', self.installer_tools])
        return {'qt_sdk': self.sdk_identity(), 'packaging_inputs_prepared': self.args.packaging}

    def prepare_schemas(self):
        cache = ROOT/'build/review/sbom-schemas'
        cache.mkdir(parents=True, exist_ok=True)
        for item in json.loads((ROOT/'scripts/sbom-schemas.json').read_text())['files']:
            target = cache/item['name']
            if not target.exists():
                data = urllib.request.urlopen(item['url'], timeout=30).read()
                if hashlib.sha256(data).hexdigest() != item['sha256']:
                    raise ValueError('Pinned schema download hash mismatch')
                target.write_bytes(data)
            if digest(target) != item['sha256']:
                raise ValueError('Cached schema hash mismatch; remove that cache entry and prepare again')

    def build_product(self):
        self.doctor('build')
        before = self.source()
        self.check_cached_sdk()
        self.script('sync-version.py', '--check')
        self.script('prepare-native-codecs.py', '--offline')
        command = ['cmake', '-S', ROOT, '-B', self.build, '-G', 'Ninja',
                   '-DCMAKE_BUILD_TYPE='+self.args.configuration, '-DXDZIP_BUILD_DESKTOP_PREVIEW=ON',
                   '-DBUILD_TESTING=ON',
                   '-DCMAKE_PREFIX_PATH='+str(self.qt)]
        if self.system == 'macos':
            command += ['-DCMAKE_OSX_ARCHITECTURES=arm64', '-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0',
                        '-DXDZIP_MACOS_HANDLER_RANK='+self.args.macos_handler_rank]
        self.run(command)
        self.check_cached_sdk(configured=True)
        self.run(['cmake', '--build', self.build, '--parallel', self.args.jobs])
        identity = self.run([self.app, '--version'], capture=True).strip()
        if self.source() != before:
            raise ValueError('Source changed during build; rebuild before continuing')
        return {'source': before, 'qt_sdk': self.sdk_identity(), 'configuration': self.args.configuration,
                'product': label(self.app), 'product_sha256': digest(self.app),
                'identity_text': identity, 'cmake_cache_sha256': digest(self.build/'CMakeCache.txt'),
                'macos_handler_rank': self.args.macos_handler_rank if self.system == 'macos' else None}

    def test(self):
        self.doctor('test')
        receipt = self.build_receipt()
        env = dict(self.env, QT_QPA_PLATFORM='offscreen', QT_QUICK_BACKEND='software')
        env['XDZIP_NATIVE_CODEC'] = str(self.app.parent / ('xdzip-native-codec.exe' if self.system == 'windows' else 'xdzip-native-codec'))
        if not self.args.skip_rust:
            self.run(['cargo', 'test', '--workspace', '--locked'], env=env)
            self.run(['cargo', 'test', '-p', 'xdzip-runtime', '--locked', 'native_backend_tests', '--', '--ignored'], env=env)
        self.script('check-ui-i18n.py', env=env)
        self.script('check-window-contracts.py', '--self-test', env=env)
        self.script('check-operation-components.py', env=env)
        self.run([self.qt/'bin'/('qmllint.exe' if self.system == 'windows' else 'qmllint'),
                  '-I', self.build/'qml', *sorted((ROOT/'apps/desktop/ui').glob('*.qml'))], env=env)
        self.run(['ctest', '--test-dir', self.build, '--no-tests=error', '--output-on-failure'], env=env)
        self.build_receipt()
        return {'source': receipt['source'], 'product_sha256': receipt['product_sha256'],
                'rust_workspace': 'delegated-to-foundation' if self.args.skip_rust else 'passed',
                'rendering': 'offscreen/software, native desktop acceptance remains separate'}

    def package_preflight(self):
        if self.system == 'macos' and os.environ.get('CI', '').lower() == 'true' and not self.args.release_distribution:
            raise ValueError('macOS CI must not deploy, sign or package an application')
        if self.system == 'windows' and self.args.configuration == 'Debug':
            raise ValueError('Windows packages require Release or RelWithDebInfo; Debug Qt deployment is not supported')
        self.require_clean()
        receipt = self.build_receipt()
        tests = self.build/'desktop-test.json'
        test_receipt = json.loads(tests.read_text(encoding='utf-8')) if tests.is_file() else {}
        if (test_receipt.get('status') != 'passed' or test_receipt.get('source') != receipt['source'] or
                test_receipt.get('product_sha256') != receipt['product_sha256']):
            raise ValueError('Run test on this successful build before packaging')
        self.doctor('package')
        if self.system == 'macos' and receipt.get('macos_handler_rank') != 'Alternate':
            raise ValueError('DMG requires build --macos-handler-rank Alternate in a separate build directory')
        # The check also ensures all host crate metadata was fetched by prepare.
        # License collection uses the same verified PATH and must not fetch late.
        self.env['CARGO_NET_OFFLINE'] = 'true'
        self.script('check-build-version.py', '--app', self.app, '--output', self.output/'build-version.json')
        if self.system == 'windows':
            for name in ('compiler/ISCC.exe', 'ChineseSimplified.isl', 'InnoSetup-license.txt'):
                if not (self.installer_tools/name).is_file():
                    raise ValueError('Run prepare --packaging first; pinned Windows installer tools are missing')
            if not (self.runtime_tools/'runtime').is_dir():
                raise ValueError('Run prepare --packaging first; validated app-local VC++ runtime is missing')
        return receipt

    def notices(self):
        notices = self.output/'notices'
        self.script('collect-notices.py', notices)
        self.script('prepare-runtime-notices.py', '--qt-prefix', self.qt, '--output', notices/'Qt-runtime')
        if self.system == 'windows':
            from native_runtime_materials import installer_notices
            installer_notices(self.installer_tools, notices)
        shutil.copy2(notices/'source-build.cdx.json', self.output/'source-build.cdx.json')
        return notices

    def runtime_package(self):
        if self.runtime.exists():
            raise ValueError('Runtime output already exists; choose a new --output-dir')
        notices = self.notices()
        if self.system == 'windows':
            self.runtime.mkdir(parents=True)
            shutil.copy2(self.app, self.runtime/'xdzip.exe')
            shutil.copy2(self.app.parent/'xdzip-update-helper.exe', self.runtime/'xdzip-update-helper.exe')
            shutil.copy2(self.app.parent/'xdzip-native-codec.exe', self.runtime/'xdzip-native-codec.exe')
            shutil.copy2(self.app.parent/'xdzip-association.exe', self.runtime/'xdzip-association.exe')
            for icon in WINDOWS_ICONS.values():
                shutil.copy2(ROOT/'design/assets/file-associations'/f'xdzip-{icon}.ico', self.runtime)
            self.script('deploy_windows.py', '--root', self.runtime, '--qt-prefix', self.qt,
                        '--qml-dir', ROOT/'apps/desktop/ui', '--qml-import', self.build/'qml',
                        '--runtime', self.runtime_tools/'runtime')
            # Keep the exact native supplier terms in both runtime and materials.
            # The source inventory remains a source inventory, not a claim that
            # these later deployment inputs were part of the Rust build graph.
            native_notices = self.runtime/'share/notices'
            for name in ('Microsoft-VC-runtime', 'Windows-SDK-graphics'):
                shutil.copytree(native_notices/name, notices/name, dirs_exist_ok=True)
            shutil.copy2(native_notices/'Windows-runtime-NOTICE.txt', notices)
            shutil.copytree(notices, self.runtime/'share/notices', dirs_exist_ok=True)
        else:
            self.script('deploy_linux.py', '--app', self.app, '--qt-prefix', self.qt,
                        '--notices', notices, '--output', self.runtime)
            shutil.copy2(self.runtime/'share/deployment.json', self.output/'linux-deployment.json')
        self.audit_runtime()
        return {'source': self.source(), 'runtime': self.system,
                'runtime_audit_sha256': digest(self.output/'deployment-audit.json')}

    def audit_runtime(self):
        app = self.runtime/('xdzip.exe' if self.system == 'windows' else 'bin/xdzip')
        audit = self.output/'deployment-audit.json'
        env = self.deployed_env()
        self.script('deployment_audit.py', '--root', self.runtime, '--app', app,
                    '--qt-prefix', self.qt, '--mode', 'deployed', '--output', audit, env=env)
        self.script('native-redistribution.py', '--root', self.runtime, '--audit', audit,
                    '--output', self.output/'native-redistribution.json', env=env)
        attribution = ['--audit', audit, '--root', self.runtime, '--qt-prefix', self.qt,
                       '--output', self.output/'runtime-attribution.json']
        if self.system == 'linux':
            attribution += ['--deployment', self.output/'linux-deployment.json']
        self.script('runtime_attribution.py', *attribution, env=env)
        self.script('classify-qt-binary-differences.py', '--attribution', self.output/'runtime-attribution.json',
                    '--qt-prefix', self.qt, '--output', self.output/'qt-binary-differences.json', env=env)
        self.script('sbom.py', '--audit', audit, '--source', self.output/'source-build.cdx.json',
                    '--output', self.output/'artifact-files.cdx.json',
                    '--attribution', self.output/'runtime-attribution.json', env=env)
        self.script('validate-sbom.py', self.output/'source-build.cdx.json', self.output/'artifact-files.cdx.json', env=env)

    def runtime_receipt(self, *, current_source=False):
        path = self.output/'package-runtime.json'
        if not path.is_file():
            raise ValueError('Run package --kind runtime first')
        data = json.loads(path.read_text(encoding='utf-8'))
        if data.get('status') != 'passed' or data.get('platform') != self.system:
            raise ValueError('No successful runtime package for this platform')
        if current_source and data.get('source') != self.source():
            raise ValueError('Runtime package source differs; use a new output directory and package again')
        audit = self.output/'deployment-audit.json'
        if not audit.is_file() or data['runtime_audit_sha256'] != digest(audit):
            raise ValueError('Runtime audit changed; package again')
        # Recompute every payload hash before creating or verifying an installer.
        from installer_payload import verify_payload
        verify_payload(self.runtime, json.loads(audit.read_text(encoding='utf-8')))
        return data

    def installer_package(self):
        self.runtime_receipt(current_source=True)
        audit = self.output/'deployment-audit.json'
        if self.system == 'windows':
            artifact = self.output/'xdzip-windows-x64-setup.exe'
            if artifact.exists():
                raise ValueError('Installer already exists; choose a new output directory')
            self.script('build-windows-installer.py', '--root', self.runtime, '--audit', audit,
                        '--iscc', self.installer_tools/'compiler/ISCC.exe',
                        '--chinese', self.installer_tools/'ChineseSimplified.isl',
                        '--output', artifact, '--report', self.output/'windows-installer.json')
            files = [artifact]
        else:
            archive = self.output/'xdzip-linux-x86_64.tar.gz'
            artifact = self.output/'xdzip-ubuntu24.04-amd64.deb'
            if archive.exists() or artifact.exists():
                raise ValueError('Installer/archive already exists; choose a new output directory')
            self.script('package_linux.py', '--root', self.runtime, '--output', archive,
                        '--report', self.output/'linux-package.json')
            self.script('build-linux-installer.py', '--root', self.runtime, '--audit', audit,
                        '--output', artifact, '--report', self.output/'linux-installer.json')
            files = [archive, artifact]
        return {'source': self.source(), 'artifacts': self.hashes(files)}

    def hashes(self, files):
        return [{'file': path.relative_to(self.output).as_posix(), 'sha256': digest(path),
                 'bytes': path.stat().st_size} for path in files]

    def save(self, name, result):
        self.output.mkdir(parents=True, exist_ok=True)
        report = {'schema_version': 1, 'status': 'passed', 'platform': self.system, **result}
        (self.output/(name+'.json')).write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
        return report

    def package(self):
        if self.system == 'macos' and self.args.kind != 'all':
            raise ValueError('macOS local packaging uses --kind all (DMG); CI uses SDK verification only')
        source = self.package_preflight()['source']
        if self.system == 'macos':
            import runpy
            version = runpy.run_path(str(ROOT/'scripts/sync-version.py'))['version_info']()
            display = version['display_version']
            dmg = self.output/('xdzip-'+display+'-macos-arm64.dmg')
            self.script('build-macos-dmg.py', '--app', self.app.parents[2], '--qt-prefix', self.qt, '--output', dmg,
                        *(['--release-distribution'] if self.args.release_distribution else []))
            self.script('validate-sbom.py', dmg.with_suffix('.cdx.json'))
            if self.require_clean() != source:
                raise ValueError('Source changed during packaging; use a new output directory and rebuild')
            files = [dmg, dmg.with_suffix('.json'), dmg.with_suffix('.cdx.json')]
            if self.args.release_distribution:
                archive = self.output/('xdzip-'+display+'-macos-arm64.app.zip')
                if not archive.is_file():
                    raise ValueError('DMG builder did not preserve the signed application ZIP')
                files.append(archive)
            return {'source': self.source(), 'artifacts': self.hashes(files)}
        if self.args.kind in ('runtime', 'all'):
            self.save('package-runtime', self.runtime_package())
        if self.args.kind in ('installer', 'all'):
            self.save('package-installer', self.installer_package())
        if self.require_clean() != source:
            raise ValueError('Source changed during packaging; use a new output directory and rebuild')
        return {'source': self.source(), 'kind': self.args.kind}

    def verify(self):
        if self.args.kind not in ('runtime', 'all'):
            raise ValueError('Verify supports --kind runtime or all')
        if self.system == 'macos' and os.environ.get('CI', '').lower() == 'true' and not self.args.release_distribution:
            raise ValueError('macOS CI does not validate a deployed application or DMG')
        self.doctor('verify')
        if self.system == 'macos':
            path = self.output/'package.json'
            if not path.is_file():
                raise ValueError('Run package before verify')
            receipt = json.loads(path.read_text(encoding='utf-8'))
            if receipt.get('status') != 'passed' or receipt.get('platform') != self.system:
                raise ValueError('No successful package for this platform')
            self.verify_hashes(receipt['artifacts'])
            images = [self.output/item['file'] for item in receipt['artifacts'] if item['file'].endswith('.dmg')]
            if len(images) != 1:
                raise ValueError('Package receipt must identify exactly one DMG')
            dmg = images[0]
            self.script('verify-macos-dmg.py', '--dmg', dmg, '--output', self.output/'verification')
            if self.args.release_distribution:
                archives = [self.output/item['file'] for item in receipt['artifacts'] if item['file'].endswith('.app.zip')]
                if len(archives) != 1: raise ValueError('Missing application ZIP')
                self.script('verify-macos-app-zip.py', '--archive', archives[0], '--output', self.output/'zip-verification')
            self.script('validate-sbom.py', dmg.with_suffix('.cdx.json'))
            return {'artifacts': receipt['artifacts'], 'deployment': 'passed',
                    'limits': 'Ad-hoc integrity only, no Developer ID or notarization'}
        runtime = self.runtime_receipt()
        if self.system == 'windows':
            required = [self.tests, self.qt/'bin/Qt6Test.dll', self.qt/'plugins/platforms/qoffscreen.dll']
            if not all(path.is_file() for path in required):
                raise ValueError('Windows runtime verification needs the built browser tests and matching Qt Test/offscreen SDK inputs')
        elif not (self.build/'apps/desktop/m2-flat.zip').is_file():
            raise ValueError('Linux runtime verification needs the owned m2-flat.zip fixture; run build and test first')
        if self.args.kind == 'all':
            path = self.output/'package-installer.json'
            if not path.is_file():
                raise ValueError('Run package --kind installer before verify --kind all')
            installer = json.loads(path.read_text(encoding='utf-8'))
            if (installer.get('status') != 'passed' or installer.get('platform') != self.system or
                    installer.get('source') != runtime.get('source')):
                raise ValueError('Installer and runtime package identities differ')
            self.verify_hashes(installer['artifacts'])
        if self.system == 'windows':
            # Each invocation owns a fresh harness; retain earlier evidence and
            # keep the verifier's refusal to reuse or delete an existing tree.
            harness = self.output/('windows-runtime-test-'+uuid.uuid4().hex)
            self.script('verify_windows_runtime.py', '--root', self.runtime, '--qt-prefix', self.qt,
                        '--tests', self.tests, '--output', harness, env=self.deployed_env())
        else:
            self.script('verify_linux_runtime.py', '--root', self.runtime,
                        '--fixture', self.build/'apps/desktop/m2-flat.zip',
                        '--output', self.output/'linux-runtime.json', env=self.deployed_env())
        self.script('validate-sbom.py', self.output/'source-build.cdx.json', self.output/'artifact-files.cdx.json')
        return {'source': runtime['source'], 'deployment': 'passed',
                'installer_lifecycle': 'CI extension only, no installation or association changes here'}

    def verify_hashes(self, items):
        for item in items:
            path = (self.output/item['file']).resolve()
            if not path.is_relative_to(self.output) or not path.is_file() or digest(path) != item['sha256']:
                raise ValueError('Packaged artifact changed or is missing')


def parser():
    value = argparse.ArgumentParser(description=__doc__)
    value.add_argument('stage', choices=['doctor', 'prepare', 'build', 'test', 'package', 'verify'])
    value.add_argument('--qt-prefix')
    value.add_argument('--qt-root', help='Qt install root, only used by prepare --install-qt')
    value.add_argument('--build-dir', default='build/desktop')
    value.add_argument('--output-dir', default='build/package')
    value.add_argument('--tools-dir', default='.cache/desktop-tools')
    value.add_argument('--configuration', choices=['Debug', 'Release', 'RelWithDebInfo'], default='Release')
    value.add_argument('--jobs', type=int, default=3)
    value.add_argument('--for-stage', choices=['prepare', 'build', 'test', 'package', 'verify'], default='build')
    value.add_argument('--install-qt', action='store_true', help='Prepare only, explicitly download pinned Qt if missing')
    value.add_argument('--install-rust', action='store_true', help='Prepare only, install locked toolchain through existing rustup')
    value.add_argument('--packaging', action='store_true', help='Prepare only, download/cache pinned packaging tools and materials')
    value.add_argument('--skip-rust', action='store_true', help='Test only, delegate Rust tests to a separate Foundation job')
    value.add_argument('--macos-handler-rank', choices=['None', 'Alternate'], default='None')
    value.add_argument('--kind', choices=['runtime', 'installer', 'all'], default='all')
    value.add_argument('--release-distribution', action='store_true', help='Dedicated release only: permit native Apple Silicon deployment, ad-hoc signing and both app ZIP/DMG')
    return value


def main(argv=None):
    args = parser().parse_args(argv)
    if args.jobs < 1:
        raise SystemExit('--jobs must be positive')
    try:
        workflow = Workflow(args)
        if args.stage == 'doctor':
            workflow.doctor(args.for_stage)
            return 0
        report_path = (workflow.build/('desktop-'+args.stage+'.json') if args.stage in ('build', 'test')
                       else workflow.output/(args.stage+'.json'))
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report = {'schema_version': 1, 'platform': workflow.system, 'stage': args.stage, 'status': 'running'}
        report_path.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
        try:
            result = {'prepare': workflow.prepare, 'build': workflow.build_product,
                      'test': workflow.test, 'package': workflow.package, 'verify': workflow.verify}[args.stage]()
            report.update(result, status='passed')
        except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
            # Child logs remain local. Do not copy exception payloads bearing SDK
            # paths, URL credentials or the user profile into a portable report.
            report.update(status='failed', failure_type=type(error).__name__)
            raise
        finally:
            report['preflights'] = workflow.preflights
            report_path.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
        print(args.stage + ' passed, receipt: ' + label(report_path))
        return 0
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        if isinstance(error, ValueError):
            print(str(error), file=sys.stderr)
        else:
            print('Stage failed ('+type(error).__name__+'); inspect local stage logs and run doctor.', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
