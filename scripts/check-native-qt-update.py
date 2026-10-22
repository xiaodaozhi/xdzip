# scripts/check-native-qt-update.py

"""Complete real Qt -> signed native installer -> real Qt restart regression."""
import argparse
import configparser
import csv
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from qt_update_fixture import prepare, feed, verify_restart, load
from update_fixture import dump, run, await_file


def await_restart(session, timeout=240):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        if (session/'fixture-error.json').is_file():
            raise RuntimeError((session/'fixture-error.json').read_text())
        for name in ('200-state.json','201-state.json'):
            path=session/name
            if path.is_file():
                record=json.loads(path.read_text())
                if record['state'] in ('error','recovery_required'):
                    raise RuntimeError('Real Qt update failed: '+record['error'])
        if (session/'201-started.json').is_file():
            return json.loads((session/'201-started.json').read_text())
        time.sleep(.1)
    raise TimeoutError('Real Qt update/restart did not finish')


def wait_exit(pid, timeout=30):
    if sys.platform=='win32':
        import ctypes
        from ctypes import wintypes
        kernel=ctypes.WinDLL('kernel32',use_last_error=True)
        kernel.OpenProcess.restype=wintypes.HANDLE
        kernel.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD]
        kernel.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD]
        kernel.CloseHandle.argtypes=[wintypes.HANDLE]
        handle=kernel.OpenProcess(0x100000,False,pid)
        if not handle:return
        try:
            if kernel.WaitForSingleObject(handle,int(timeout*1000))!=0:
                raise TimeoutError('Owned Qt process did not shut down')
        finally:
            kernel.CloseHandle(handle)
    else:
        import select
        try:fd=os.pidfd_open(pid)
        except ProcessLookupError:return
        try:
            poll=select.poll();poll.register(fd,select.POLLIN)
            if not poll.poll(int(timeout*1000)):
                raise TimeoutError('Owned Qt process did not shut down')
        finally:
            os.close(fd)


def exercise(output, prepared, session, launch, executable, association_check, sentinel):
    old=None;new=None;result=None;failure=None;stage='launch_old';cleanup=[]
    log=(output/'qt-product.log').open('w',encoding='utf-8')
    try:
        old=launch([executable],stdout=log,stderr=subprocess.STDOUT)
        started=await_file(session/'200-started.json',old,timeout=40)
        assert started['primary_windows']==1,started
        profile=Path(started['settings_path'])
        config=configparser.ConfigParser();config.read(profile,encoding='utf-8')
        assert config.get('ownedFixture','sentinel')=='preserve-me'
        baseline={section:dict(config[section]) for section in config.sections() if section!='update'}
        stage='single_instance_old'
        # This goes through normal main/SingleInstance: the secondary must forward
        # and return before it creates a second QML tree or fixture record.
        peer=launch([executable],stdout=log,stderr=subprocess.STDOUT)
        assert peer.wait(timeout=20)==0
        assert json.loads((session/'200-started.json').read_text())['pid']==old.pid
        association_check()
        stage='upgrade_and_restart'
        trigger=session/'upgrade.json';dump(trigger,{'owned':True})
        if sys.platform=='linux':os.chown(trigger,19731,19731)
        new=await_restart(session)
        stage='verify_new_process'
        verify_restart(new,prepared)
        assert new['settings_path']==started['settings_path'] and new['home']==started['home'],new
        assert new['pid']!=old.pid
        assert old.wait(timeout=10)==0
        barrier=await_file(session/'200-exit-barrier.json')
        assert barrier['update_pending'] and barrier['active_tasks']==0
        assert barrier['about_check_clicked'] and barrier['about_install_clicked']
        assert barrier['duplicate_install_attempted'],barrier
        pending=barrier['pending'];job=Path(pending['directory'])
        request=json.loads((job/'request.json').read_text())
        receipt=json.loads((job/'result.json').read_text())
        assert request['transaction']==pending['transaction']==receipt['transaction']
        assert receipt['error']=='',receipt
        dump(output/'helper-request.json',request);dump(output/'helper-result.json',receipt)
        if prepared.get('password_task'):
            waiting=json.loads((session/'password-waiting.json').read_text())
            cancelled=json.loads((session/'waiting-cancelled.json').read_text())
            completed=json.loads((session/'password-completed.json').read_text())
            assert waiting['active_tasks']==cancelled['active_tasks']==1 and waiting['kind']=='check'
            assert waiting['task_id']==cancelled['task_id']==completed['task_id']
            assert waiting['prompt_id']==cancelled['prompt_id'] and cancelled['method']=='controller.cancel'
            assert cancelled['state']=='downloaded' and cancelled['password_window_modal']
            assert completed['status']=='succeeded' and completed['verified_files']=='1' and completed['verified_bytes']=='18'
            assert barrier['password_task_completed'] and barrier['waiting_cancel_controller_call']
        stage='verify_settings_and_associations'
        changed=configparser.ConfigParser();changed.read(profile,encoding='utf-8')
        for section,values in baseline.items():
            for key,value in values.items():
                assert changed.get(section,key)==value,(section,key)
        assert sentinel.read_text()=='unowned user content\n'
        association_check()
        stage='single_instance_new'
        peer=launch([executable],stdout=log,stderr=subprocess.STDOUT)
        assert peer.wait(timeout=20)==0
        time.sleep(.3)
        current=json.loads((session/'201-state.json').read_text())
        assert current['pid']==new['pid'] and current['primary_windows']==1,current
        if sys.platform=='linux':
            assert new['uid']==19731,new
            for name in ('XDG_CONFIG_HOME','XDG_DATA_HOME','XDG_CACHE_HOME','XDG_STATE_HOME'):
                assert new[name] and new[name]==started[name],(name,new)
        result={'status':'passed','scope':'Real private browser_main, actual About mouse input, signed verification/join/handoff, native installer and ordinary SingleInstance restart; real AES Check wait/cancel/resume before upgrade' if prepared.get('password_task') else 'Real private browser_main, actual About mouse input, signed verification/join/handoff, native installer and ordinary SingleInstance restart; idle tasks only',
             'production_trust_modified':False,'old':started,'new':new,'exit_barrier':barrier,
             'settings_preserved':True,'default_association_preserved':True,'single_instance_before_and_after':True,
             'duplicate_install_preserved_transaction':True,
             'real_password_check_wait_cancel_resume':bool(prepared.get('password_task')),
             'association_scope':'Classic ProgID plus unchanged UserChoice' if sys.platform=='win32' else 'Custom XDG MIME default and GIO candidate',
             'limits':['Cancel waiting uses a direct controller call while the real password window is application-modal; About mouse input is never injected through the modal.' if prepared.get('password_task') else 'Busy/password scenarios are not part of this idle run.',
                       'Physical desktop, accessibility, normal quit cancellation and manual license/authorization acceptance are separate.',
                       'Windows Server CI uses a test-only minimum-system override; Windows 11 native acceptance remains separate.' if sys.platform=='win32' else 'Ubuntu minimum-system check is unchanged.']}
    except Exception as error:
        failure={'status':'failed','stage':stage,'error':type(error).__name__,'detail':str(error),
                 'production_trust_modified':False}
        raise
    finally:
        # Export the last observable state before any possibly failing cleanup.
        try:
            for path in session.glob('*.json'):
                shutil.copyfile(path,output/('session-'+path.name))
            if sys.platform=='win32':
                diagnostics=load(Path(__file__).with_name('check-windows-update.py'),'owned_inno_diagnostics')
                dump(output/'installer-windows.json',diagnostics.installer_windows())
                for index,path in enumerate((session/'installer-temp').glob('Setup Log*.txt')):
                    shutil.copyfile(path,output/('installer-'+str(index)+'.log'))
            if new is None and (session/'201-state.json').is_file():
                new=json.loads((session/'201-state.json').read_text())
        except Exception as error:
            cleanup.append({'stage':'capture','error':type(error).__name__,'detail':str(error)})
        try:
            stop=session/'stop.json';dump(stop,{'owned':True})
            if sys.platform=='linux':os.chown(stop,19731,19731)
            if new:wait_exit(new['pid'])
            elif old and old.poll() is None:
                try:old.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    # Only this Popen-owned process is terminated after a failed
                    # assertion, never an unrelated process or user application.
                    old.terminate();old.wait(timeout=10)
        except Exception as error:
            cleanup.append({'stage':'shutdown','error':type(error).__name__,'detail':str(error)})
        finally:
            log.close()
            if failure or cleanup:
                record=failure or {'status':'failed','stage':'cleanup','error':'cleanup_failed'}
                record['cleanup_errors']=cleanup
                dump(output/'result.json',record)
            elif result:
                dump(output/'result.json',result)
        if cleanup and failure is None:
            raise RuntimeError('Owned Qt cleanup failed; see result.json')


def windows(output, prepared):
    if os.environ.get('CI')!='true' or os.environ.get('GITHUB_ACTIONS')!='true' or os.environ.get('RUNNER_ENVIRONMENT')!='github-hosted':
        raise RuntimeError('Real Qt installation requires a disposable GitHub-hosted Windows runner')
    import winreg
    def user_choice():
        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER,'Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FileExts\\.zip\\UserChoice') as key:
                return sorted([winreg.EnumValue(key,index) for index in range(winreg.QueryInfoKey(key)[1])],key=lambda value:value[0])
        except FileNotFoundError:return None
    original_choice=user_choice()
    for key in ('Software\\xdzip\\Install','Software\\Classes\\xdzip.Archive'):
        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER,key):
                raise RuntimeError('Refusing an existing xdzip installation')
        except FileNotFoundError:pass
    session=Path(prepared['session_directory'])
    feed(prepared,output,session)
    root=Path(os.environ['RUNNER_TEMP'])/('xdzip real Qt update '+str(os.getpid()))
    root.mkdir()
    sid=next(csv.reader([subprocess.check_output(['whoami','/user','/fo','csv','/nh'],text=True).strip()]))[1]
    for path in (root,session):
        run(['icacls',path,'/inheritance:r','/grant:r','*'+sid+':(OI)(CI)F'],stdout=subprocess.DEVNULL)
    installer_temp=session/'installer-temp';installer_temp.mkdir()
    desktop_env={**os.environ,'TEMP':str(installer_temp),'TMP':str(installer_temp)}
    # Own the classic extension association in this disposable runner. Installer
    # OpenWith registration must never replace its existing default ProgID.
    extension='Software\\Classes\\.zip'
    with winreg.CreateKey(winreg.HKEY_CURRENT_USER,extension) as key:
        previous,kind=winreg.QueryValueEx(key,'') if _registry_has_default(key) else (None,None)
        winreg.SetValueEx(key,'',0,winreg.REG_SZ,'xdzip.OwnedFixtureOther')
    def association():
        assert user_choice()==original_choice,'Installer modified the effective UserChoice association'
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER,extension) as key:
            assert winreg.QueryValueEx(key,'')[0]=='xdzip.OwnedFixtureOther'
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER,extension+'\\OpenWithProgids') as key:
            winreg.QueryValueEx(key,'xdzip.Archive')
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER,'Software\\Classes\\Applications\\xdzip.exe\\shell\\open\\command') as key:
            assert winreg.QueryValueEx(key,'')[0]=='"'+str(root/'xdzip.exe')+'" -- "%1"'
    try:
        package=output/'old'/prepared['old']['package']
        run([package,'/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/DIR='+str(root),
             '/LOG='+str(output/'old-install.log')],env=desktop_env,timeout=180)
        sentinel=root/'user-kept.txt';sentinel.write_text('unowned user content\n')
        exercise(output,prepared,session,lambda argv,**kwargs:subprocess.Popen(list(map(str,argv)),env=desktop_env,**kwargs),
                 root/'xdzip.exe',association,sentinel)
    finally:
        result_path=output/'result.json'
        completed=result_path.is_file() and json.loads(result_path.read_text()).get('status')=='passed'
        if completed and (root/'unins000.exe').is_file():
            run([root/'unins000.exe','/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART'],timeout=90)
        # A failed installer may still hold its own mutex. Preserve its evidence
        # and let the disposable runner reclaim it instead of blocking upload on
        # another interactive installer launched during failure cleanup.
        with winreg.CreateKey(winreg.HKEY_CURRENT_USER,extension) as key:
            if previous is None:
                try:winreg.DeleteValue(key,'')
                except FileNotFoundError:pass
            else:winreg.SetValueEx(key,'',0,kind,previous)


def _registry_has_default(key):
    import winreg
    try:winreg.QueryValueEx(key,'');return True
    except FileNotFoundError:return False


def linux(output, prepared):
    if os.geteuid()!=0 or not Path('/.dockerenv').exists() or os.environ.get('XDZIP_ISOLATED_UPDATE_FIXTURE')!='1':
        raise RuntimeError('Real Qt installation is restricted to a disposable root container')
    for tool in ('cc','gcc','g++','rustc','cmake','qmlimportscanner'):
        if shutil.which(tool):raise RuntimeError('Qt update runtime must not contain SDK/build tools: '+tool)
    installed=subprocess.run(['dpkg-query','-W','xdzip'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    if installed.returncode==0:raise RuntimeError('Refusing existing xdzip installation')
    run(['useradd','--create-home','--uid','19731','xdzipfixture'])
    home=Path('/home/xdzipfixture')
    session=Path(prepared['session_directory'])
    assert session==home/'qt-session'
    feed(prepared,output,session)
    for path in [session,*session.rglob('*')]:os.chown(path,19731,19731)
    settings=home/'custom-config';settings.mkdir(exist_ok=True)
    (settings/'mimeapps.list').write_text('[Default Applications]\napplication/zip=owned-other.desktop\n')
    data=home/'custom-data';applications=data/'applications';applications.mkdir(parents=True)
    (applications/'owned-other.desktop').write_text('[Desktop Entry]\nType=Application\nName=Owned other fixture\nExec=/bin/true %f\nMimeType=application/zip;\n')
    cache=home/'custom-cache';cache.mkdir()
    state=home/'custom-state';state.mkdir()
    for path in [settings,data,cache,state,applications,*settings.rglob('*'),*applications.rglob('*')]:os.chown(path,19731,19731)
    runtime=home/'runtime';runtime.mkdir(mode=0o700);os.chown(runtime,19731,19731)
    env={**os.environ,'HOME':str(home),'DISPLAY':':99','XDG_RUNTIME_DIR':str(runtime),
         'XDG_CONFIG_HOME':str(settings),'XDG_DATA_HOME':str(data),'XDG_CACHE_HOME':str(cache),'XDG_STATE_HOME':str(state),
         'QT_QPA_PLATFORM':'xcb','QT_QUICK_BACKEND':'software','LANG':'C.UTF-8'}
    for key in ('XDZIP_SETTINGS_PATH','LD_LIBRARY_PATH','QT_PLUGIN_PATH','QML_IMPORT_PATH','QML2_IMPORT_PATH'):env.pop(key,None)
    def launch(argv,**kwargs):
        return subprocess.Popen(['/usr/bin/setpriv','--reuid','19731','--regid','19731','--init-groups','--',*map(str,argv)],env=env,**kwargs)
    def association():
        assert launch(['xdg-mime','query','default','application/zip'],stdout=subprocess.PIPE,text=True).communicate(timeout=10)[0].strip()=='owned-other.desktop'
        assert 'xdzip.desktop' in launch(['gio','mime','application/zip'],stdout=subprocess.PIPE,text=True).communicate(timeout=10)[0]
    rule=Path('/etc/polkit-1/rules.d/00-xdzip-owned-fixture.rules')
    rule.write_text('polkit.addRule(function(a,s) { if(a.id === "org.xdzip.update" && s.user === "xdzipfixture") return polkit.Result.YES; });\n')
    Path('/run/dbus').mkdir(exist_ok=True)
    run(['dbus-daemon','--system','--fork'])
    polkit=next(p for p in (Path('/usr/lib/polkit-1/polkitd'),Path('/usr/libexec/polkitd')) if p.is_file())
    service=subprocess.Popen([str(polkit),'--no-debug'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    display=subprocess.Popen(['Xvfb',':99','-screen','0','1280x900x24','-ac'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    try:
        for _ in range(100):
            if Path('/tmp/.X11-unix/X99').exists():break
            time.sleep(.05)
        run(['dpkg','-i',output/'old'/prepared['old']['package']],stdout=subprocess.DEVNULL)
        sentinel=Path('/opt/xdzip/user-kept.txt');sentinel.write_text('unowned user content\n')
        exercise(output,prepared,session,launch,Path('/usr/bin/xdzip'),association,sentinel)
    finally:
        display.terminate();display.wait(timeout=5)
        service.terminate();service.wait(timeout=5)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--prepare',action='store_true')
    parser.add_argument('--qt-prefix',type=Path)
    parser.add_argument('--session-directory',type=Path)
    parser.add_argument('--password-task',action='store_true',help='Exercise a real AES Check waiting for a password, cancel only update waiting, then finish the task and upgrade')
    args=parser.parse_args();output=args.output.resolve()
    if args.prepare:
        if not args.qt_prefix or not args.session_directory:parser.error('Preparation requires Qt SDK and fixed private session directory')
        prepare(output,args.qt_prefix,args.session_directory)
        return
    prepared=json.loads((output/'prepared.json').read_text())
    prepared['password_task']=args.password_task
    try:
        if sys.platform=='win32':windows(output,prepared)
        elif sys.platform=='linux':linux(output,prepared)
        else:parser.error('This private regression supports only native Windows/Linux')
    except Exception as error:
        if (output/'result.json').exists():
            record=json.loads((output/'result.json').read_text())
            if record.get('status')=='passed':
                record.update(status='failed',stage='post_upgrade_cleanup',error=type(error).__name__,detail=str(error))
                dump(output/'result.json',record)
        else:
            dump(output/'result.json',{'status':'failed','stage':'setup','error':type(error).__name__,'detail':str(error)})
        raise


if __name__=='__main__':main()
