# scripts/check-windows-update.py

"""Real Inno updater protocol regression, never permitted on a developer host."""
import argparse
import csv
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from update_fixture import ROOT, OLD, NEW, run, dump, digest, build_helpers, compile_product, manifest, request, await_file


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--offline', action='store_true')
    args = parser.parse_args()
    if sys.platform != 'win32' or os.environ.get('CI') != 'true' or os.environ.get('GITHUB_ACTIONS') != 'true' or os.environ.get('RUNNER_ENVIRONMENT') != 'github-hosted':
        parser.error('Real installation requires a disposable GitHub-hosted Windows runner')
    import winreg
    for key in ('Software\\xdzip\\Install', 'Software\\Classes\\xdzip.Archive'):
        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, key):
                parser.error('Refusing to replace an existing xdzip installation')
        except FileNotFoundError:
            pass
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    helper, signing = build_helpers(output, args.offline)
    spec = importlib.util.spec_from_file_location('windows_builder', ROOT / 'scripts/build-windows-installer.py')
    builder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(builder)
    sid = next(csv.reader([subprocess.check_output(['whoami', '/user', '/fo', 'csv', '/nh'], text=True).strip()]))[1]
    def private(path):
        run(['icacls', path, '/inheritance:r', '/grant:r', '*' + sid + ':(OI)(CI)F'], stdout=subprocess.DEVNULL)
    def package(label, identity, executable_identity=None):
        folder = output / label
        payload = folder / 'payload'
        payload.mkdir(parents=True)
        compile_product(payload / 'xdzip.exe', executable_identity or identity)
        shutil.copy2(helper, payload / 'xdzip-update-helper.exe')
        licenses = payload / 'share/notices/Microsoft-VC-runtime'
        licenses.mkdir(parents=True)
        shutil.copy2(ROOT / 'build/windows-runtime/runtime/Microsoft-VC-runtime-license.rtf', licenses / 'license.rtf')
        notice = ROOT / 'apps/desktop/packaging/windows/Windows-runtime-NOTICE.txt'
        (licenses.parent / notice.name).write_bytes(b'\xef\xbb\xbf' + notice.read_bytes())
        builder.runtime_agreement(payload)
        audit = dict(status='passed', platform='windows', mode='deployed', identity=identity,
                     files=[{'path': p.relative_to(payload).as_posix(), 'sha256': digest(p)} for p in payload.rglob('*') if p.is_file()])
        path = folder / ('xdzip-' + identity['version'] + '-windows-x86_64-setup.exe')
        text, _ = builder.render(payload, audit, path, ROOT / 'build/installer-toolchain/ChineseSimplified.isl')
        # Only this owned test installer automatically confirms its wizard. The
        # exact production LicenseFile and notice remain compiled and validated;
        # neither the production template nor helper receives a silent bypass.
        text += '''
procedure CurPageChanged(CurPageID: Integer);
begin
  if CurPageID = wpLicense then WizardForm.LicenseAcceptedRadio.Checked := True;
  if (CurPageID <> wpInstalling) and WizardForm.NextButton.Enabled then
    WizardForm.NextButton.OnClick(WizardForm.NextButton);
end;
'''
        path.with_suffix('.files').write_text(builder.payload_manifest(audit), encoding='utf-8')
        path.with_suffix('.iss').write_text(text, encoding='utf-8-sig')
        run([ROOT / 'build/installer-toolchain/compiler/ISCC.exe', path.with_suffix('.iss')], stdout=subprocess.DEVNULL)
        manifest(path, 'windows', signing, identity)
        return path
    old = package('old', OLD)
    new = package('new', NEW)
    mismatched = package('identity-mismatch', NEW, OLD)
    install_root = Path(os.environ['RUNNER_TEMP']) / ('xdzip updater owned fixture ' + str(os.getpid()))
    install_root.mkdir()
    private(install_root)
    restart = Path(os.environ['USERPROFILE']) / 'xdzip-update-fixture-restarted.json'
    if restart.exists():
        raise RuntimeError('Refusing existing restart evidence')
    results = []
    def install(package):
        run([package, '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/DIR=' + str(install_root)])
    def scenario(label, candidate, expected='', cancel=False, corrupt=False):
        install(old)
        restart.unlink(missing_ok=True)
        job = output / ('job-' + label)
        job.mkdir()
        private(job)
        for src in (candidate, candidate.parent / 'release.json', candidate.parent / 'release.json.sig'):
            shutil.copy2(src, job / src.name)
        if corrupt:
            signature = json.loads((job / 'release.json.sig').read_text())
            signature['signature'] = '00' * 64
            dump(job / 'release.json.sig', signature)
        shutil.copy2(helper, job / 'xdzip-update-helper.exe')
        stop = job / 'exit-parent'
        parent = subprocess.Popen([str(install_root / 'xdzip.exe'), '--fixture-parent', str(stop)])
        time.sleep(0.15)
        if parent.poll() is not None:
            raise RuntimeError('Fixture parent did not start')
        transaction = request(job, parent.pid, install_root, job / candidate.name)
        log = (output / (label + '.log')).open('w')
        process = subprocess.Popen([str(job / 'xdzip-update-helper.exe'), '--request', str(job / 'request.json')], stdout=log, stderr=subprocess.STDOUT)
        try:
            if expected != 'untrusted_signature':
                ready = await_file(job / 'ready.json', process)
                assert ready['transaction'] == transaction
                assert subprocess.run([str(install_root / 'xdzip.exe'), '--gate-probe']).returncode == 11
                if cancel:
                    dump(job / 'cancel.json', {'transaction': transaction})
                else:
                    dump(job / 'proceed.json', {'transaction': transaction})
                    stop.touch()
                    parent.wait(timeout=10)
            receipt = await_file(job / 'result.json', timeout=180)
            assert receipt == {'transaction': transaction, 'error': expected}, receipt
            process.wait(timeout=15)
            if not expected:
                resumed = await_file(restart, timeout=20)
                assert resumed['version'] == NEW['version'] and resumed['build'] == NEW['build'], resumed
            assert subprocess.run([str(install_root / 'xdzip.exe'), '--gate-probe']).returncode == 0
            results.append(dict(case=label, status='passed', error=expected))
        finally:
            if (job / 'result.json').is_file():
                shutil.copy2(job / 'result.json', output / (label + '-receipt.json'))
            stop.touch()
            parent.wait(timeout=10)
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=10)
            log.close()
    try:
        scenario('cancel', new, 'cancelled', cancel=True)
        scenario('bad-signature', new, 'untrusted_signature', corrupt=True)
        scenario('wrong-installed-identity', mismatched, 'installed_identity_mismatch')
        # An independently repeated verified installer repairs the injected old
        # binary before the subsequent complete parent/helper/installer restart.
        install(new)
        actual = json.loads(subprocess.check_output([str(install_root / 'xdzip.exe'), '--version-json'], text=True))
        assert actual['build'] == NEW['build']
        results.append(dict(case='identity-failure-recovery', status='passed'))
        scenario('real-installer-upgrade', new)
        scenario('second-upgrade', new)
        dump(output / 'result.json', dict(status='passed', production_trust_modified=False,
             scope='Owned native fixture, real interactive Inno with test-only wizard driver; not full Qt upgrade or manual license acceptance', cases=results))
    finally:
        uninstaller = install_root / 'unins000.exe'
        if uninstaller.exists():
            run([uninstaller, '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART'])
        restart.unlink(missing_ok=True)


if __name__ == '__main__':
    main()
