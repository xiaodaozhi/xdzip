# scripts/update_fixture.py

"""Shared disposable updater fixtures. The public RFC 8032 key is never production trust."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]
SEED = '9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60'
PUBLIC = 'd75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a'
KEYRING = {'schema_version': 1, 'keys': [{'key_id': 'owned-fixture', 'public_key': PUBLIC}]}
OLD = {'version': '0.2.0-alpha.1', 'build': 200, 'revision': 'a' * 12, 'dirty': False}
NEW = {'version': '0.2.0-alpha.2', 'build': 201, 'revision': 'b' * 12, 'dirty': False}


def run(argv, **kwargs):
    return subprocess.run([str(p) for p in argv], check=True, **kwargs)


def dump(path, value):
    path.write_text(json.dumps(value, sort_keys=True) + '\n', encoding='utf-8')


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(128 * 1024), b''):
            result.update(block)
    return result.hexdigest()


def build_helpers(output, offline=False):
    source = output / 'test-source'
    source.mkdir()
    # Keep workspace resolution identical, but exclude all generated artifacts.
    for name in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml'):
        shutil.copy2(ROOT / name, source / name)
    for name in ('crates', 'tools'):
        shutil.copytree(ROOT / name, source / name,
                        ignore=shutil.ignore_patterns('target', 'build', 'dist'))
    (source / 'resources').mkdir()
    dump(source / 'resources/update-trust.json', KEYRING)
    env = os.environ.copy()
    for key in ('XDZIP_UPDATE_SIGNING_KEY_HEX', 'XDZIP_SPARKLE_SIGNING_KEY_HEX', 'CARGO_ENCODED_RUSTFLAGS'):
        env.pop(key, None)
    if os.name == 'nt':
        env['RUSTFLAGS'] = '-C target-feature=+crt-static'
    command = ['cargo', 'build', '--locked', '--manifest-path', source / 'Cargo.toml',
               '--target-dir', output / 'rust', '-p', 'xdzip-update-helper', '-p', 'xdzip-update', '--bins']
    if offline:
        command.append('--offline')
    with (output / 'build.log').open('w', encoding='utf-8') as log:
        run(command, env=env, stdout=log, stderr=subprocess.STDOUT)
    suffix = '.exe' if os.name == 'nt' else ''
    return output / ('rust/debug/xdzip-update-helper' + suffix), output / ('rust/debug/xdzip-update-tool' + suffix)


def compile_product(output, identity):
    source = ROOT / 'tests/fixtures/native/update_product.c'
    if os.name == 'nt':
        argv = ['cl', '/nologo', '/MT', source, '/Fe:' + str(output), '/Fo:' + str(output.with_suffix('.obj'))]
        defines = ['/D']
    else:
        argv = ['cc', '-std=gnu11', '-Wall', '-Wextra', '-Werror', source, '-o', output]
        defines = ['-D']
    for name, value in (('VERSION', identity['version']), ('BUILD', str(identity['build'])), ('REVISION', identity['revision'])):
        argv.append(defines[0] + 'FIXTURE_' + name + '="' + value + '"')
    run(argv)


def manifest(package, platform, tool, identity=NEW):
    version = identity['version']
    base = 'https://github.com/xiaodaozhi/xdzip/releases/download/v' + version + '/'
    linux = {'distribution': 'ubuntu', 'version': '24.04', 'glibc': '2.39', 'display_server': 'X11'}
    targets = [('windows', 'x86_64', 'zip', 'windows-x86_64.zip', {'windows': '11'}),
               ('windows', 'x86_64', 'exe', 'windows-x86_64-setup.exe', {'windows': '11'}),
               ('macos', 'arm64', 'app.zip', 'macos-arm64.app.zip', {'macos': '14.0'}),
               ('macos', 'arm64', 'dmg', 'macos-arm64.dmg', {'macos': '14.0'}),
               ('linux', 'x86_64', 'tar.gz', 'linux-x86_64.tar.gz', linux),
               ('linux', 'x86_64', 'deb', 'ubuntu24.04-x86_64.deb', linux)]
    assets = []
    for target, arch, kind, suffix, minimum in targets:
        filename = 'xdzip-' + version + '-' + suffix
        selected = target == platform and kind == ('exe' if platform == 'windows' else 'deb')
        size, sha = (package.stat().st_size, digest(package)) if selected else (1, hashlib.sha256(b'x').hexdigest())
        assets.append(dict(filename=filename, platform=target, architecture=arch, package_type=kind,
                           minimum_system=minimum, bytes=size, sha256=sha, url=base + filename))
    value = dict(schema_version=1, product='xdzip', version=version, channel='alpha', build=identity['build'],
                 tag='v' + version, commit=identity.get('commit',identity['revision'] + '0' * 28), repository='xiaodaozhi/xdzip', assets=assets)
    path = package.parent / 'release.json'
    dump(path, value)
    ring = package.parent / 'test-keyring.json'
    dump(ring, KEYRING)
    env = os.environ.copy()
    env['XDZIP_UPDATE_SIGNING_KEY_HEX'] = SEED
    run([tool, 'sign', '--manifest', path, '--keyring', ring, '--key-id', 'owned-fixture',
         '--output', package.parent / 'release.json.sig'], env=env)
    ring.unlink()
    return path


def qa_installer_text(text):
    """Drive only owned test installers; retain the real supplier agreement."""
    return text.replace('[Setup]\n','[Setup]\nShowLanguageDialog=no\nSetupLogging=yes\n',1)+'''
procedure CurPageChanged(CurPageID: Integer);
begin
  Log(Format('Owned fixture wizard page: %d', [CurPageID]));
  if CurPageID = wpLicense then begin
    WizardForm.LicenseAcceptedRadio.Checked := True;
    WizardForm.LicenseAcceptedRadio.OnClick(WizardForm.LicenseAcceptedRadio);
  end;
  { wpReady rejects synchronous clicks before Show; queue the native button event. }
  if (CurPageID <> wpInstalling) and WizardForm.NextButton.Enabled then
    PostMessage(WizardForm.NextButton.Handle, $00F5, 0, 0);
end;
'''


def request(job, parent_pid, install_root, package):
    transaction = uuid.uuid4().hex
    dump(job / 'request.json', dict(schema_version=1, transaction=transaction, parent_pid=parent_pid,
         current_version=OLD['version'], current_build=OLD['build'], install_root=str(install_root),
         package_file=str(package), manifest_file=str(job / 'release.json'), signature_file=str(job / 'release.json.sig')))
    return transaction


def await_file(path, process=None, timeout=60):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.is_file():
            return json.loads(path.read_text(encoding='utf-8'))
        if process and process.poll() is not None:
            raise RuntimeError('Fixture process exited before ' + path.name)
        time.sleep(0.05)
    raise TimeoutError('Fixture timed out waiting for ' + path.name)
