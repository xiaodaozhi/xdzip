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
from qt_update_fixture import prepare, feed, verify_restart, load, require_no_signing_authority, production_trust_check
from update_fixture import dump, run, await_file


def await_restart(session, timeout=240, copy_probe=None):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        if (session/'fixture-error.json').is_file():
            raise RuntimeError((session/'fixture-error.json').read_text())
        if copy_probe and (session/'installer-copy-paused.json').is_file():
            copy_probe(json.loads((session/'installer-copy-paused.json').read_text()))
            copy_probe=None
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


def windows_copy_probe(session, executable, launch, log, prepared, pause):
    """Launch the real Qt EXE while the owned Inno installer holds its copy pause."""
    import ctypes
    from ctypes import wintypes
    kernel=ctypes.WinDLL('kernel32',use_last_error=True)
    user=ctypes.WinDLL('user32',use_last_error=True)
    kernel.OpenProcess.restype=wintypes.HANDLE
    kernel.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD]
    kernel.OpenMutexW.restype=wintypes.HANDLE
    kernel.OpenMutexW.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.LPCWSTR]
    kernel.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD]
    kernel.CloseHandle.argtypes=[wintypes.HANDLE]
    callback_type=ctypes.WINFUNCTYPE(wintypes.BOOL,wintypes.HWND,wintypes.LPARAM)
    user.EnumWindows.argtypes=[callback_type,wintypes.LPARAM]
    user.IsWindowVisible.argtypes=[wintypes.HWND]
    user.GetWindowThreadProcessId.argtypes=[wintypes.HWND,ctypes.POINTER(wintypes.DWORD)]
    record={'status':'failed','stage':'after_xdzip_before_remaining_files','qt_sdk':'6.8.2'}
    installer=None;gate=None;probe=None;identity_process=None
    try:
        assert pause['schema_version']==1 and pause['stage']==record['stage'] and pause['qt_sdk']=='6.8.2',pause
        assert prepared['qt_sdk_version']=='6.8.2'
        assert not (session/'installer-copy-resumed.json').exists()
        assert not (session/'201-started.json').exists() and not (session/'201-state.json').exists()
        installer=kernel.OpenProcess(0x100000,False,pause['installer_pid'])
        gate=kernel.OpenMutexW(0x100000,False,'Local\\xdzip.desktop.updating')
        assert installer and gate and kernel.WaitForSingleObject(installer,0)==258
        # Observing a named mutex must not keep the gate alive on behalf of the
        # product. Only the genuine helper/Setup may retain it during launch.
        kernel.CloseHandle(gate);gate=None
        identity_process=launch([executable,'--version-json'],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
        identity_bytes,_=identity_process.communicate(timeout=15)
        assert identity_process.returncode==0,identity_bytes
        identity=json.loads(identity_bytes)
        expected=prepared['new']['identity']
        assert all(identity[key]==expected[key] for key in ('version','build','revision','dirty')),identity
        assert not (session/'installer-copy-resumed.json').exists()
        visible=set()
        probe=launch([executable],stdout=log,stderr=subprocess.STDOUT)
        record.update(installer_pid=pause['installer_pid'],probe_pid=probe.pid)
        @callback_type
        def observe(window,_):
            pid=wintypes.DWORD()
            user.GetWindowThreadProcessId(window,ctypes.byref(pid))
            if pid.value==probe.pid and user.IsWindowVisible(window):visible.add(int(window))
            return True
        deadline=time.monotonic()+15
        while probe.poll() is None and time.monotonic()<deadline:
            assert user.EnumWindows(observe,0)
            record['visible_window_count']=len(visible)
            time.sleep(.005)
        code=probe.wait(timeout=1)
        record.update(installer_pid=pause['installer_pid'],probe_pid=probe.pid,exit_code=code,
                      visible_window_count=len(visible),installed_executable=identity,
                      installer_still_running=kernel.WaitForSingleObject(installer,0)==258,
                      remaining_files_not_started=not (session/'installer-copy-resumed.json').exists(),
                      fixture_sessions_not_started=not (session/'201-started.json').exists() and not (session/'201-state.json').exists())
        assert code==11 and not visible,record
        assert record['installer_still_running'] and record['remaining_files_not_started'] and record['fixture_sessions_not_started'],record
        # Exit 11 is the real main's InstalledRuntimeGuard rejection, before
        # constructing ApplicationSessions/controllers or accepting archive work.
        record.update(status='passed',runtime_guard_rejected_before_sessions=True,
                      limits='Same Qt 6.8.2 payload; no atomic bundle or cross-Qt-version migration guarantee')
        return record
    except Exception as error:
        record.update(error=type(error).__name__,detail=str(error))
        raise
    finally:
        for owned in (probe,identity_process):
            try:
                if owned and owned.poll() is None:
                    owned.terminate();owned.wait(timeout=10)
            except Exception as error:
                record.setdefault('cleanup_errors',[]).append(type(error).__name__)
        if installer:kernel.CloseHandle(installer)
        if gate:kernel.CloseHandle(gate)
        try:dump(session/'installer-copy-probe.json',record)
        finally:
            # Always release this owned pause, including assertion failures, so
            # its diagnostic cannot strand the disposable real installer.
            dump(session/'installer-copy-release.json',{'owned':True})


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


def failure_recovery(output, prepared, session, launch, executable, association_check, sentinel, repair):
    """Consume a genuine failed receipt through an ordinary default controller."""
    old=None;restored=None;record={'status':'failed','stage':'failure_launch'}
    evidence=output/'failed-install-recovery';evidence.mkdir()
    log=(evidence/'qt-product.log').open('w',encoding='utf-8')
    def owned(name):
        path=session/name;dump(path,{'owned':True})
        if sys.platform=='linux':os.chown(path,19731,19731)
    try:
        old=launch([executable],stdout=log,stderr=subprocess.STDOUT)
        started=await_file(session/'200-started.json',old,timeout=40)
        assert started['pid']==old.pid and started['primary_windows']==1,started
        profile=Path(started['settings_path'])
        before=configparser.ConfigParser();before.read(profile,encoding='utf-8')
        baseline={section:dict(before[section]) for section in before.sections() if section!='update'}
        assert before.get('ownedFixture','sentinel')=='preserve-me'
        association_check()
        record['stage']='real_about_failed_install';owned('upgrade.json')
        barrier=await_file(session/'200-exit-barrier.json',timeout=90)
        assert barrier['about_check_clicked'] and barrier['about_install_clicked']
        assert barrier['update_pending'] and barrier['active_tasks']==0
        assert old.wait(timeout=20)==0
        pending=barrier['pending'];job=Path(pending['directory'])
        receipt=await_file(job/'result.json',timeout=180)
        request=json.loads((job/'request.json').read_text())
        dump(evidence/'helper-request.json',request);dump(evidence/'helper-result.json',receipt)
        for name in ('installer.log','helper.log'):
            if (job/name).is_file():shutil.copyfile(job/name,evidence/name)
        assert request['transaction']==pending['transaction']==receipt['transaction']
        assert receipt['error']==prepared['failure']['error'],receipt
        # Observing the Windows mutex must not retain its launch gate ourselves.
        deadline=time.monotonic()+30
        while True:
            if sys.platform=='win32':
                import ctypes
                from ctypes import wintypes
                kernel=ctypes.WinDLL('kernel32',use_last_error=True)
                kernel.OpenMutexW.restype=wintypes.HANDLE
                kernel.OpenMutexW.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.LPCWSTR]
                kernel.CloseHandle.argtypes=[wintypes.HANDLE]
                handle=kernel.OpenMutexW(0x100000,False,'Local\\xdzip.desktop.updating')
                gated=bool(handle)
                if handle:kernel.CloseHandle(handle)
            else:gated=Path('/run/xdzip/updating').exists()
            if not gated:break
            if time.monotonic()>deadline:raise TimeoutError('Failed helper did not release its launch gate')
            time.sleep(.05)
        assert not (session/'201-started.json').exists(),'Failed helper automatically restarted the application'
        assert json.loads((session/'200-started.json').read_text())['pid']==old.pid
        installed=launch([executable,'--version-json'],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
        identity_bytes,_=installed.communicate(timeout=20)
        assert installed.returncode==0,identity_bytes
        identity=json.loads(identity_bytes)
        expected=prepared['old' if sys.platform=='win32' else 'new']['identity']
        assert all(identity[key]==expected[key] for key in ('version','build','revision','dirty')),identity
        package_status=None
        if sys.platform=='linux':
            package_status=subprocess.check_output(['dpkg-query','-W','-f=${Status}','xdzip'],text=True)
            assert package_status=='install ok half-configured',package_status
        record['stage']='ordinary_restore_only_startup'
        (session/'upgrade.json').unlink()
        for name in ('200-started.json','200-state.json','201-started.json','201-state.json'):
            (session/name).unlink(missing_ok=True)
        owned('restore-only.json')
        # The factory returns nullptr BEFORE construction: real default-cache
        # restoration consumes this genuine pending state and helper receipt.
        restored=launch([executable],stdout=log,stderr=subprocess.STDOUT)
        recovery=await_file(session/'recovery-started.json',restored,timeout=40)
        assert recovery['pid']==restored.pid and recovery['primary_windows']==1,recovery
        assert recovery['state']=='recovery_required' and recovery['error']==receipt['error'],recovery
        assert all(recovery[key]==identity[key] for key in ('version','build','revision','dirty')),recovery
        assert recovery['active_tasks']==0 and not recovery['update_pending'] and recovery['update_barriers']==0
        assert not recovery['about_check_clicked'] and not recovery['about_install_clicked']
        consumed=configparser.ConfigParser();consumed.read(profile.parent/'update-state.ini',encoding='utf-8')
        assert not consumed.has_option('update','pending'),'Recovery did not consume the real pending receipt'
        after=configparser.ConfigParser();after.read(profile,encoding='utf-8')
        for section,values in baseline.items():
            for key,value in values.items():assert after.get(section,key)==value,(section,key)
        assert sentinel.read_text()=='unowned user content\n'
        association_check()
        owned('stop.json');assert restored.wait(timeout=20)==0
        record.update(stage='explicit_fixture_repair',old=started,exit_barrier=barrier,
                      receipt=receipt,request=request,recovery=recovery,installed_identity=identity,
                      package_status_after_failure=package_status,
                      automatic_restart=False,automatic_retry=False,automatic_rollback=False,
                      settings_preserved=True,default_associations_preserved=True)
        for path in session.glob('*.json'):shutil.copyfile(path,evidence/path.name)
        # Only the fixture repairs/resets after the recovery process exits;
        # the product never retries or rolls back an installer automatically.
        record['explicit_fixture_repair']=repair(log)
        feed(prepared,output,session,reset=True)
        if sys.platform=='linux':
            for path in [session,*session.rglob('*')]:os.chown(path,19731,19731)
        record.update(status='passed',stage='completed',
                      scope='Real About failure handoff, genuine receipt, ordinary main/default-controller recovery; explicit fixture repair/reset before independent successful upgrade')
    except Exception as error:
        record.update(error=type(error).__name__,detail=str(error))
        raise
    finally:
        try:
            if restored and restored.poll() is None:
                owned('stop.json');restored.wait(timeout=15)
            if old and old.poll() is None:
                owned('stop.json');old.wait(timeout=15)
        except Exception as error:
            record.setdefault('cleanup_errors',[]).append({'error':type(error).__name__,'detail':str(error)})
            for process in (restored,old):
                if process and process.poll() is None:process.terminate();process.wait(timeout=10)
            record['status']='failed'
        finally:
            for path in session.glob('*.json'):shutil.copyfile(path,evidence/('last-'+path.name))
            if sys.platform=='win32':
                for index,path in enumerate((session/'installer-temp').glob('Setup Log*.txt')):
                    shutil.copyfile(path,evidence/('installer-'+str(index)+'.log'))
            dump(evidence/'result.json',record);log.close()
    assert record['status']=='passed',record


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
        ordinary_quit=None
        if prepared.get('password_task'):
            stage='normal_quit_during_update_and_password_wait'
            ordinary_quit=await_file(session/'normal-quit-completed.json',old,timeout=90)
            returned=json.loads((session/'normal-quit-returned.json').read_text())
            assert ordinary_quit['status']=='passed',ordinary_quit
            assert ordinary_quit['ready_to_quit_count']==1 and ordinary_quit['all_runtimes_joined']
            assert ordinary_quit['runtime_count']>0 and ordinary_quit['task_status']=='cancelled'
            assert ordinary_quit['prompt_cleared'] and not ordinary_quit['update_pending']
            assert ordinary_quit['update_barriers']==0 and returned['active_tasks']==1
            assert returned['task_id']==ordinary_quit['task_id'] and returned['prompt_id']==ordinary_quit['prompt_id']
            assert returned['update_state']=='waiting_tasks'
            assert old.wait(timeout=15)==0
            assert not (session/'200-exit-barrier.json').exists() and not (session/'201-started.json').exists()
            state_file=profile.parent/'update-state.ini'
            pending=configparser.ConfigParser();pending.read(state_file,encoding='utf-8')
            assert not pending.has_option('update','pending'),'Normal quit persisted an install handoff'
            association_check()
            dump(output/'normal-quit.json',{'status':'passed','quit':ordinary_quit,'returned':returned,
                'scope':'Actual browser_main quit guard, AES Check and native QuitWindow return/cancel before ordinary restart; no installer handoff'})
            trigger.unlink()
            for name in ('200-started.json','200-state.json','download-requested.json','release-download.json',
                         'password-waiting.json','waiting-cancelled.json','password-completed.json'):
                (session/name).unlink(missing_ok=True)
            previous_pid=old.pid
            old=launch([executable],stdout=log,stderr=subprocess.STDOUT)
            started=await_file(session/'200-started.json',old,timeout=40)
            assert started['pid']!=previous_pid and started['state']=='idle',started
            same_profile=Path(started['settings_path']).resolve(strict=True)==profile.resolve(strict=True)
            assert same_profile and started['primary_windows']==1,{
                'same_settings_file':same_profile,'primary_windows':started['primary_windows'],
                'restart_state':started['state']}
            unchanged=configparser.ConfigParser();unchanged.read(profile,encoding='utf-8')
            for section,values in baseline.items():
                for key,value in values.items():assert unchanged.get(section,key)==value,(section,key)
            association_check()
            stage='upgrade_after_normal_quit_restart'
            dump(trigger,{'owned':True})
            if sys.platform=='linux':os.chown(trigger,19731,19731)
        copy_evidence={}
        def copy_probe(pause):
            copy_evidence.update(windows_copy_probe(session,executable,launch,log,prepared,pause))
        new=await_restart(session,copy_probe=copy_probe if sys.platform=='win32' and prepared.get('windows_copy_pause') else None)
        if sys.platform=='win32':
            assert prepared.get('windows_copy_pause') and copy_evidence.get('status')=='passed'
            assert json.loads((session/'installer-copy-resumed.json').read_text())['stage']=='remaining_payload_started'
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
             'normal_quit_during_update_wait':ordinary_quit,
             'failed_install_ordinary_startup_recovery':json.loads((output/'failed-install-recovery/result.json').read_text()),
             'installation_launch_probe':copy_evidence if sys.platform=='win32' else None,
             'association_scope':'ZIP/TAR/GZip classic defaults and unchanged UserChoice; ZIP OpenWith' if sys.platform=='win32' else 'ZIP/TAR/GZip custom XDG MIME defaults and ZIP GIO candidate',
             'limits':['Cancel waiting uses a direct controller call while the real password window is application-modal; About mouse input is never injected through the modal.' if prepared.get('password_task') else 'Busy/password scenarios are not part of this idle run.',
                       'Physical desktop, accessibility and manual license/authorization acceptance are separate.',
                       'The Windows copy-pause launch regression uses Qt 6.8.2 on both sides; it does not prove atomic bundle replacement or migration between Qt versions.',
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
    extensions=('.zip','.tar','.gz')
    def user_choice(extension):
        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER,'Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FileExts\\'+extension+'\\UserChoice') as key:
                return sorted([winreg.EnumValue(key,index) for index in range(winreg.QueryInfoKey(key)[1])],key=lambda value:value[0])
        except FileNotFoundError:return None
    original_choice={extension:user_choice(extension) for extension in extensions}
    for key in ('Software\\xdzip\\Install','Software\\Classes\\xdzip.Archive'):
        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER,key):
                raise RuntimeError('Refusing an existing xdzip installation')
        except FileNotFoundError:pass
    session=Path(prepared['session_directory'])
    feed(prepared,output,session,candidate='failure')
    root=Path(os.environ['RUNNER_TEMP'])/('xdzip real Qt update '+str(os.getpid()))
    root.mkdir()
    sid=next(csv.reader([subprocess.check_output(['whoami','/user','/fo','csv','/nh'],text=True).strip()]))[1]
    for path in (root,session):
        run(['icacls',path,'/inheritance:r','/grant:r','*'+sid+':(OI)(CI)F'],stdout=subprocess.DEVNULL)
    installer_temp=session/'installer-temp';installer_temp.mkdir()
    desktop_env={**os.environ,'TEMP':str(installer_temp),'TMP':str(installer_temp)}
    # Own the classic extension association in this disposable runner. Installer
    # OpenWith registration must never replace its existing default ProgID.
    previous_defaults={}
    for suffix in extensions:
        with winreg.CreateKey(winreg.HKEY_CURRENT_USER,'Software\\Classes\\'+suffix) as key:
            previous_defaults[suffix]=winreg.QueryValueEx(key,'') if _registry_has_default(key) else (None,None)
            winreg.SetValueEx(key,'',0,winreg.REG_SZ,'xdzip.OwnedFixtureOther')
    def association():
        for suffix in extensions:
            assert user_choice(suffix)==original_choice[suffix],'Installer modified the effective UserChoice association'
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER,'Software\\Classes\\'+suffix) as key:
                assert winreg.QueryValueEx(key,'')[0]=='xdzip.OwnedFixtureOther'
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER,'Software\\Classes\\.zip\\OpenWithProgids') as key:
            winreg.QueryValueEx(key,'xdzip.Archive.ZIP')
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER,'Software\\Classes\\Applications\\xdzip.exe\\shell\\open\\command') as key:
            assert winreg.QueryValueEx(key,'')[0]=='"'+str(root/'xdzip.exe')+'" -- "%1"'
    try:
        package=output/'old'/prepared['old']['package']
        run([package,'/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/DIR='+str(root),
             '/LOG='+str(output/'old-install.log')],env=desktop_env,timeout=180)
        sentinel=root/'user-kept.txt';sentinel.write_text('unowned user content\n')
        launch=lambda argv,**kwargs:subprocess.Popen(list(map(str,argv)),env=desktop_env,**kwargs)
        def repair(log):
            identity=json.loads(subprocess.check_output([root/'xdzip.exe','--version-json'],env=desktop_env,text=True))
            assert all(identity[key]==prepared['old']['identity'][key] for key in ('version','build','revision','dirty')),identity
            return {'action':'verified unchanged old installation; no repair needed','identity':identity}
        failure_recovery(output,prepared,session,launch,root/'xdzip.exe',association,sentinel,repair)
        exercise(output,prepared,session,launch,
                 root/'xdzip.exe',association,sentinel)
    finally:
        result_path=output/'result.json'
        completed=result_path.is_file() and json.loads(result_path.read_text()).get('status')=='passed'
        if completed and (root/'unins000.exe').is_file():
            run([root/'unins000.exe','/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART'],timeout=90)
        # A failed installer may still hold its own mutex. Preserve its evidence
        # and let the disposable runner reclaim it instead of blocking upload on
        # another interactive installer launched during failure cleanup.
        for suffix,(previous,kind) in previous_defaults.items():
            with winreg.CreateKey(winreg.HKEY_CURRENT_USER,'Software\\Classes\\'+suffix) as key:
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
    feed(prepared,output,session,candidate='failure')
    for path in [session,*session.rglob('*')]:os.chown(path,19731,19731)
    settings=home/'custom-config';settings.mkdir(exist_ok=True)
    mime_types=('application/zip','application/x-tar','application/gzip')
    (settings/'mimeapps.list').write_text('[Default Applications]\n'+''.join(mime+'=owned-other.desktop\n' for mime in mime_types))
    data=home/'custom-data';applications=data/'applications';applications.mkdir(parents=True)
    (applications/'owned-other.desktop').write_text('[Desktop Entry]\nType=Application\nName=Owned other fixture\nExec=/bin/true %f\nMimeType='+';'.join(mime_types)+';\n')
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
        for mime in mime_types:
            assert launch(['xdg-mime','query','default',mime],stdout=subprocess.PIPE,text=True).communicate(timeout=10)[0].strip()=='owned-other.desktop',mime
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
        def repair(log):
            run(['apt-get','-y','--no-remove','--no-install-recommends','install',output/'failure'/prepared['failure']['package']],stdout=log,stderr=subprocess.STDOUT,timeout=120)
            status=subprocess.check_output(['dpkg-query','-W','-f=${Status}','xdzip'],text=True)
            assert status=='install ok installed',status
            # Explicit owned reset preserves the independent original chain.
            run(['apt-get','-y','--allow-downgrades','--no-remove','--no-install-recommends','install',output/'old'/prepared['old']['package']],stdout=log,stderr=subprocess.STDOUT,timeout=120)
            identity=json.loads(launch(['/usr/bin/xdzip','--version-json'],stdout=subprocess.PIPE,text=True).communicate(timeout=20)[0])
            assert all(identity[key]==prepared['old']['identity'][key] for key in ('version','build','revision','dirty')),identity
            return {'action':'explicit APT configure repair, then explicit verified old fixture reset',
                    'repaired_package_status':status,'reset_identity':identity}
        failure_recovery(output,prepared,session,launch,Path('/usr/bin/xdzip'),association,sentinel,repair)
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
    parser.add_argument('--production-trust',action='store_true',help='Retain embedded production public keys; prepare unsigned inputs or verify protected-step signatures before installation')
    args=parser.parse_args();output=args.output.resolve()
    if args.production_trust:
        require_no_signing_authority()
    if args.prepare:
        if not args.qt_prefix or not args.session_directory:parser.error('Preparation requires Qt SDK and fixed private session directory')
        prepare(output,args.qt_prefix,args.session_directory,production_trust=args.production_trust)
        return
    prepared=json.loads((output/'prepared.json').read_text())
    if bool(prepared.get('production_trust'))!=args.production_trust:
        parser.error('Preparation and execution must use the same production trust mode')
    if args.production_trust:
        production_trust_check(output,prepared)
    prepared['password_task']=args.password_task
    try:
        if sys.platform=='win32':windows(output,prepared)
        elif sys.platform=='linux':linux(output,prepared)
        else:parser.error('This private regression supports only native Windows/Linux')
        if args.production_trust:
            path=output/'result.json';record=json.loads(path.read_text())
            record.update(production_trust=True,production_trust_verification='production-trust-verification.json')
            dump(path,record)
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
