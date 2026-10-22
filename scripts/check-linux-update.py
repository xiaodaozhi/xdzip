# scripts/check-linux-update.py

"""Real APT updater protocol regression, restricted to a disposable Linux container."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import pwd
import shutil
import subprocess
import sys
import tempfile
import time
from update_fixture import ROOT, OLD, NEW, run, dump, digest, build_helpers, compile_product, manifest, request, await_file

UID = 19731


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--offline', action='store_true')
    args = parser.parse_args()
    if sys.platform != 'linux' or os.geteuid() != 0 or not Path('/.dockerenv').exists() or os.environ.get('XDZIP_ISOLATED_UPDATE_FIXTURE') != '1':
        parser.error('Real installation is restricted to an explicitly marked disposable root container')
    if subprocess.run(['dpkg-query', '-W', 'xdzip'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0:
        parser.error('Refusing a container with an existing xdzip installation')
    for tool in ('pkexec', 'apt-get', 'dbus-daemon', 'dpkg-deb', 'cc'):
        if not shutil.which(tool):
            parser.error('Missing container dependency: ' + tool)
    evidence = args.output.resolve()
    evidence.mkdir(parents=True, exist_ok=False)
    # Native POSIX ownership/modes cannot be emulated by a Windows bind mount.
    # Build/install fixtures on the container filesystem; export records only.
    output = Path(tempfile.mkdtemp(prefix='xdzip-update-owned-'))
    import atexit
    def export():
        for pattern in ('*.log', '*.json'):
            for path in output.glob(pattern):
                target=evidence / path.name
                shutil.copy2(path, target)
                # Only public fixture records are exported. Keep the live job
                # private, while allowing the unprivileged CI uploader to read.
                target.chmod(0o644)
    atexit.register(export)
    helper, signing = build_helpers(output, args.offline)
    try:
        pwd.getpwuid(UID)
        raise RuntimeError('Fixture UID already exists')
    except KeyError:
        run(['useradd', '--create-home', '--uid', UID, 'xdzipfixture'])
    home = Path('/home/xdzipfixture')
    def user(argv, **kwargs):
        return subprocess.Popen(['/usr/bin/setpriv', '--reuid', str(UID), '--regid', str(UID), '--init-groups', '--', *map(str, argv)],
                                env={**os.environ, 'HOME': str(home)}, **kwargs)
    # The actual pkexec chain is exercised. This rule exists only in this throwaway
    # container and authorizes this fixed public fixture UID, without a desktop agent.
    rules = Path('/etc/polkit-1/rules.d/00-xdzip-owned-fixture.rules')
    rules.parent.mkdir(parents=True, exist_ok=True)
    rules.write_text('polkit.addRule(function(action, subject) { if (action.id === "org.xdzip.update" && subject.user === "xdzipfixture") return polkit.Result.YES; });\n')
    Path('/run/dbus').mkdir(exist_ok=True)
    run(['dbus-daemon', '--system', '--fork'])
    polkit = next((p for p in (Path('/usr/lib/polkit-1/polkitd'), Path('/usr/libexec/polkitd')) if p.is_file()))
    polkit_log = (output / 'polkit.log').open('w')
    service = subprocess.Popen([str(polkit), '--no-debug'], stdout=polkit_log, stderr=subprocess.STDOUT)
    spec = importlib.util.spec_from_file_location('linux_builder', ROOT / 'scripts/build-linux-installer.py')
    builder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(builder)
    def package(label, identity, fault=None):
        folder = output / label
        payload = folder / 'payload'
        (payload / 'bin').mkdir(parents=True)
        (payload / 'share').mkdir()
        compile_product(payload / 'bin/xdzip', identity)
        shutil.copy2(helper, payload / 'bin/xdzip-update-helper')
        (payload / 'bin/xdzip').chmod(0o755)
        (payload / 'bin/xdzip-update-helper').chmod(0o755)
        launcher = payload / 'xdzip'
        launcher.write_text('#!/bin/sh\ncd "$(dirname "$0")"\nexec ./bin/xdzip "$@"\n')
        launcher.chmod(0o755)
        dump(payload / 'share/deployment.json', {'system_packages': ['libc6']})
        audit = dict(status='passed', platform='linux', mode='deployed', identity=identity,
                     files=[{'path': p.relative_to(payload).as_posix(), 'sha256': digest(p)} for p in payload.rglob('*') if p.is_file()])
        stage = folder / 'stage'
        record = builder.package_tree(payload, audit, stage)
        if fault in ('package', 'version'):
            control = stage / 'DEBIAN/control'
            text = control.read_text()
            text = text.replace('Package: xdzip\n', 'Package: owned-other\n') if fault == 'package' else text.replace('Version: ' + record['version'], 'Version: 1:0.2.0~alpha.99+git' + identity['revision'])
            control.write_text(text)
        if fault == 'postinst':
            hook = stage / 'DEBIAN/postinst'
            text = hook.read_text()
            hook.write_text(text.replace('hook=postinst', 'if ! test -e /run/xdzip/fixture-postinst-failed; then\n install -m 0644 /dev/null /run/xdzip/fixture-postinst-failed\n exit 85\nfi\nhook=postinst'))
        path = folder / ('xdzip-' + identity['version'] + '-ubuntu24.04-x86_64.deb')
        run(['dpkg-deb', '-Zgzip', '-z1', '--root-owner-group', '--build', stage, path], stdout=subprocess.DEVNULL)
        manifest(path, 'linux', signing, identity)
        return path, Path('/opt/xdzip/versions') / record['runtime_id']
    old, old_root = package('old', OLD)
    new, _ = package('new', NEW)
    wrong_name, _ = package('wrong-name', NEW, 'package')
    wrong_version, _ = package('wrong-version', NEW, 'version')
    failing, _ = package('postinst-failure', NEW, 'postinst')
    restart = home / 'xdzip-update-fixture-restarted.json'
    results = []
    def control(path, transaction):
        dump(path, {'transaction': transaction})
        os.chown(path, UID, UID)
        path.chmod(0o600)
    def reset():
        run(['dpkg', '-i', old], stdout=subprocess.DEVNULL)
        restart.unlink(missing_ok=True)
    def scenario(label, candidate, expected='', cancel=False, corrupt=False, authorized=True):
        reset()
        job = home / label
        job.mkdir(mode=0o700)
        os.chown(job, UID, UID)
        for src in (candidate, candidate.parent / 'release.json', candidate.parent / 'release.json.sig'):
            shutil.copy2(src, job / src.name)
        if corrupt:
            signature = json.loads((job / 'release.json.sig').read_text())
            signature['signature'] = '00' * 64
            dump(job / 'release.json.sig', signature)
        stop = job / 'exit-parent'
        parent = user([old_root / 'bin/xdzip', '--fixture-parent', stop])
        time.sleep(0.15)
        if parent.poll() is not None:
            raise RuntimeError('Fixture parent failed to acquire normal running gate')
        transaction = request(job, parent.pid, old_root, job / candidate.name)
        for leaf in job.iterdir():
            os.chown(leaf, UID, UID)
            leaf.chmod(0o600)
        log = (output / (label + '.log')).open('w')
        if authorized:
            process = user(['/usr/libexec/xdzip-update-helper', '--authorize', job / 'request.json'], stdout=log, stderr=subprocess.STDOUT)
        else:
            process = subprocess.Popen(['/usr/libexec/xdzip-update-helper', '--request', str(job / 'request.json')], env={**os.environ, 'PKEXEC_UID': str(UID)}, stdout=log, stderr=subprocess.STDOUT)
        try:
            if not expected or cancel or expected == 'installation_failed':
                ready = await_file(job / 'ready.json', process)
                assert ready['transaction'] == transaction
                probe = user([old_root / 'bin/xdzip', '--gate-probe'])
                assert probe.wait(timeout=5) == 11, 'New application startup bypassed update gate'
                if cancel:
                    control(job / 'cancel.json', transaction)
                else:
                    control(job / 'proceed.json', transaction)
                    stop.touch()
                    parent.wait(timeout=5)
            receipt = await_file(job / 'result.json', timeout=90)
            assert receipt == {'transaction': transaction, 'error': expected}, receipt
            process.wait(timeout=10)
            if not expected:
                resumed = await_file(restart, timeout=20)
                assert resumed['version'] == NEW['version'] and resumed['build'] == NEW['build'] and resumed['uid'] == UID, resumed
                status = subprocess.check_output(['dpkg-query', '-W', '-f=${Status}', 'xdzip'], text=True)
                assert status == 'install ok installed'
            assert not Path('/run/xdzip/updating').exists(), 'Update gate leaked'
            results.append({'case': label, 'status': 'passed', 'error': expected, 'authorized_via_pkexec': authorized})
        finally:
            if (job / 'result.json').is_file():
                shutil.copy2(job / 'result.json', output / (label + '-receipt.json'))
            stop.touch()
            parent.wait(timeout=5)
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=5)
            log.close()
    try:
        scenario('cancel', new, 'cancelled', cancel=True)
        scenario('bad-signature', new, 'untrusted_signature', corrupt=True)
        scenario('wrong-package-name', wrong_name, 'invalid_debian_package')
        scenario('wrong-package-version', wrong_version, 'invalid_debian_package')
        scenario('failed-postinst', failing, 'installation_failed')
        # Retry the same real package through dpkg/APT configuration recovery.
        recovery_log = (output / 'recovery.log').open('w')
        run(['apt-get', '-y', '--no-remove', '--no-install-recommends', 'install', failing], stdout=recovery_log, stderr=subprocess.STDOUT)
        recovery_log.close()
        assert subprocess.check_output(['dpkg-query', '-W', '-f=${Status}', 'xdzip'], text=True) == 'install ok installed'
        results.append({'case': 'failed-installation-recovery', 'status': 'passed'})
        scenario('root-request', new, authorized=False)
        scenario('authorized-upgrade', new)
        # Another old->new transaction after a complete restart proves the lock and
        # authorization chain are reusable; no no_new_privs flag may leak to restart.
        scenario('second-upgrade', new)
        dump(output / 'result.json', {'status': 'passed', 'scope': 'Owned native fixture, real pkexec/root helper/APT; not full Qt upgrade or desktop authorization UI',
                                     'production_trust_modified': False, 'cases': results})
    finally:
        service.terminate()
        service.wait(timeout=5)
        polkit_log.close()


if __name__ == '__main__':
    main()
