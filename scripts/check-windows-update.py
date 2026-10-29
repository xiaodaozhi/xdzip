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
from update_fixture import ROOT, OLD, NEW, run, dump, digest, build_helpers, compile_product, manifest, request, await_file, qa_installer_text
from association_types import WINDOWS_ICONS


def installer_windows():
    """Read only captions from owned Inno processes on the disposable runner."""
    import ctypes
    from ctypes import wintypes
    user32=ctypes.WinDLL('user32',use_last_error=True)
    kernel32=ctypes.WinDLL('kernel32',use_last_error=True)
    kernel32.OpenProcess.restype=wintypes.HANDLE
    kernel32.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD]
    kernel32.QueryFullProcessImageNameW.argtypes=[wintypes.HANDLE,wintypes.DWORD,wintypes.LPWSTR,ctypes.POINTER(wintypes.DWORD)]
    kernel32.CloseHandle.argtypes=[wintypes.HANDLE]
    callback_type=ctypes.WINFUNCTYPE(wintypes.BOOL,wintypes.HWND,wintypes.LPARAM)
    user32.GetWindowTextW.argtypes=[wintypes.HWND,wintypes.LPWSTR,ctypes.c_int]
    user32.GetWindowThreadProcessId.argtypes=[wintypes.HWND,ctypes.POINTER(wintypes.DWORD)]
    user32.EnumWindows.argtypes=[callback_type,wintypes.LPARAM]
    user32.EnumChildWindows.argtypes=[wintypes.HWND,callback_type,wintypes.LPARAM]
    records=[]
    def caption(window):
        text=ctypes.create_unicode_buffer(2048)
        user32.GetWindowTextW(window,text,len(text))
        return text.value
    def visit(window,_):
        pid=wintypes.DWORD()
        user32.GetWindowThreadProcessId(window,ctypes.byref(pid))
        handle=kernel32.OpenProcess(0x1000,False,pid.value)
        if not handle:return True
        try:
            image=ctypes.create_unicode_buffer(32768);size=wintypes.DWORD(len(image))
            if not kernel32.QueryFullProcessImageNameW(handle,0,image,ctypes.byref(size)):return True
            name=Path(image.value).name.lower()
            if not (name.endswith('.tmp') or name.endswith('-setup.exe')):return True
            children=[]
            def child(value,_):
                text=caption(value)
                if text:children.append(text)
                return True
            user32.EnumChildWindows(window,callback_type(child),0)
            records.append({'pid':pid.value,'image':name,'caption':caption(window),'controls':children})
        finally:
            kernel32.CloseHandle(handle)
        return True
    user32.EnumWindows(callback_type(visit),0)
    return records


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
    # The protocol fixture has a small C main program, but must still carry
    # the real association command and all document icons required by Setup.
    # Application resource/visual checks belong to the complete Qt product.
    association = output / 'xdzip-association.exe'
    run(['cl', '/nologo', '/MT', '/EHsc', ROOT/'apps/desktop/AssociationLauncher.cpp',
         '/Fe:'+str(association), '/Fo:'+str(output/'association.obj'),
         '/link', '/SUBSYSTEM:WINDOWS', 'shell32.lib', 'ole32.lib'])
    spec = importlib.util.spec_from_file_location('windows_builder', ROOT / 'scripts/build-windows-installer.py')
    builder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(builder)
    sid = next(csv.reader([subprocess.check_output(['whoami', '/user', '/fo', 'csv', '/nh'], text=True).strip()]))[1]
    def private(path):
        run(['icacls', path, '/inheritance:r', '/grant:r', '*' + sid + ':(OI)(CI)F'], stdout=subprocess.DEVNULL)
    def package(label, identity, executable_identity=None, outcome='upgrade'):
        folder = output / label
        payload = folder / 'payload'
        payload.mkdir(parents=True)
        compile_product(payload / 'xdzip.exe', executable_identity or identity)
        shutil.copy2(helper, payload / 'xdzip-update-helper.exe')
        shutil.copy2(association, payload / association.name)
        for icon in WINDOWS_ICONS.values():
            shutil.copy2(ROOT/'design/assets/file-associations'/f'xdzip-{icon}.ico', payload)
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
        text=qa_installer_text(text,outcome)
        path.with_suffix('.files').write_text(builder.payload_manifest(audit), encoding='utf-8')
        path.with_suffix('.iss').write_text(text, encoding='utf-8-sig')
        run([ROOT / 'build/installer-toolchain/compiler/ISCC.exe', path.with_suffix('.iss')], stdout=subprocess.DEVNULL)
        manifest(path, 'windows', signing, identity)
        return path
    old = package('old', OLD)
    new = package('new', NEW)
    mismatched = package('identity-mismatch', NEW, OLD)
    license_cancel = package('license-cancel', NEW, outcome='cancel-license')
    setup_failure = package('setup-failure', NEW, outcome='startup-failure')
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
        installer_temp=job/'installer-temp';installer_temp.mkdir()
        process = subprocess.Popen([str(job / 'xdzip-update-helper.exe'), '--request', str(job / 'request.json')],
                                   env={**os.environ,'TEMP':str(installer_temp),'TMP':str(installer_temp)},stdout=log, stderr=subprocess.STDOUT)
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
            if expected in ('installation_cancelled','installation_failed'):
                assert not restart.exists(),'Failed or cancelled Setup restarted the application'
                actual=json.loads(subprocess.check_output([str(install_root/'xdzip.exe'),'--version-json'],text=True))
                assert actual['version']==OLD['version'] and actual['build']==OLD['build'],actual
                marker=('Owned fixture cancelled license without accepting' if expected=='installation_cancelled'
                        else 'Owned fixture injected InitializeSetup failure')
                setup_logs=[]
                for setup_log in installer_temp.glob('Setup Log*.txt'):
                    raw=setup_log.read_bytes()
                    setup_logs.append(raw.decode('utf-16' if raw.startswith(b'\xff\xfe') else 'utf-8-sig',errors='replace'))
                assert any(marker in text for text in setup_logs),'Actual Setup did not reach the intended owned failure/cancel hook'
            if not expected:
                resumed = await_file(restart, timeout=20)
                assert resumed['version'] == NEW['version'] and resumed['build'] == NEW['build'], resumed
            assert subprocess.run([str(install_root / 'xdzip.exe'), '--gate-probe']).returncode == 0
            results.append(dict(case=label, status='passed', error=expected))
        finally:
            dump(output/(label+'-installer-windows.json'),installer_windows())
            for index,setup_log in enumerate(installer_temp.glob('Setup Log*.txt')):
                shutil.copyfile(setup_log,output/(label+'-setup-'+str(index)+'.log'))
            if (job / 'result.json').is_file():
                shutil.copy2(job / 'result.json', output / (label + '-receipt.json'))
            stop.touch()
            parent.wait(timeout=10)
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=10)
            log.close()
    try:
        scenario('real-license-cancel',license_cancel,'installation_cancelled')
        scenario('real-setup-failure',setup_failure,'installation_failed')
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
