#!/usr/bin/env python3
"""CI-only container: real dpkg lifecycle and GIO launch, no SDK or default takeover."""
import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time


def run(args,**kwargs):
    result=subprocess.run(args,capture_output=True,text=True,timeout=40,**kwargs)
    if result.returncode:raise RuntimeError('Installer verification command failed: '+Path(args[0]).name+'\n'+result.stdout+result.stderr)
    return result.stdout.strip()


def main():
    home=Path('/tmp/xdzip-home');home.mkdir();os.chown(home,1000,1000)
    for relative in ['.config','.local/share/applications','probe','runtime']:
        target=home/relative;target.mkdir(parents=True,exist_ok=True)
    for p in home.rglob('*'):os.chown(p,1000,1000)
    (home/'runtime').chmod(0o700)
    config=home/'.config/mimeapps.list';config.write_text('[Default Applications]\napplication/zip=other.desktop\n');os.chown(config,1000,1000)
    other=home/'.local/share/applications/other.desktop';other.write_text('[Desktop Entry]\nType=Application\nName=Other\nExec=/bin/true %f\nMimeType=application/zip;\n');os.chown(other,1000,1000)
    settings=home/'.config/settings.ini';settings.write_text('[General]\nschemaVersion=1\n[appearance]\ntheme=dark\n[privacy]\nrecentEnabled=false\n');os.chown(settings,1000,1000)
    env=dict(os.environ,HOME=str(home),XDG_CONFIG_HOME=str(home/'.config'),XDG_DATA_HOME=str(home/'.local/share'),XDG_CURRENT_DESKTOP='GNOME',
             LANG='C.UTF-8',QT_QPA_PLATFORM='xcb',QT_QUICK_BACKEND='software',DISPLAY=':99',XDZIP_SETTINGS_PATH=str(settings),XDG_RUNTIME_DIR=str(home/'runtime'))
    for key in ['LD_LIBRARY_PATH','QT_PLUGIN_PATH','QML_IMPORT_PATH','QML2_IMPORT_PATH']:env.pop(key,None)
    user={'env':env,'user':1000,'group':1000,'extra_groups':[]}
    display=subprocess.Popen(['Xvfb',':99','-screen','0','1280x900x24','-ac'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    try:
        for _ in range(100):
            if Path('/tmp/.X11-unix/X99').exists():break
            time.sleep(.05)
        fixture=home/'中文 space ; archive.zip';fixture.write_bytes(Path('/fixture.zip').read_bytes());os.chown(fixture,1000,1000)
        phases=[]
        def verify_open(label):
            probe=home/'probe'/f'{label}.json'
            launch=dict(user);launch['env']=dict(env,XDZIP_ASSOCIATION_PROBE=str(probe))
            run(['gio','launch','/usr/share/applications/xdzip.desktop',str(fixture)],**launch)
            for _ in range(300):
                if probe.exists():break
                time.sleep(.05)
            data=json.loads(probe.read_text())
            if data['event']!='association_open' or data['records']!=100000:raise RuntimeError('Associated product did not read real archive')
            # Probe requests normal close; wait until its lifetime lock is released.
            for _ in range(100):
                with open('/run/xdzip/usage.lock') as lock:
                    try:fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);break
                    except BlockingIOError:time.sleep(.05)
            if run(['xdg-mime','query','default','application/zip'],**user)!='other.desktop':raise RuntimeError('Default handler changed')
            if 'xdzip.desktop' not in run(['gio','mime','application/zip'],**user):raise RuntimeError('MIME candidate not registered')
            phases.append(label)
        run(['dpkg','-i','/packages/qa1.deb']);verify_open('install')
        sentinel=Path('/opt/xdzip/user-kept.txt');sentinel.write_text('keep user data')
        original_settings=settings.read_bytes()
        broken=subprocess.run(['dpkg','-i','/packages/qa-failure.deb'],capture_output=True,text=True,timeout=40)
        if broken.returncode==0 or 'half-configured' not in run(['dpkg-query','-W','-f=${Status}','xdzip']):
            raise RuntimeError('Configuration failure was not exercised')
        if not Path('/run/xdzip/installing').is_file() or settings.read_bytes()!=original_settings:
            raise RuntimeError('Incomplete installation not gated or user settings changed')
        blocked=subprocess.run(['/usr/bin/xdzip','--smoke-test'],capture_output=True,text=True,timeout=20,**user)
        if blocked.returncode==0:raise RuntimeError('Half-configured runtime was allowed to launch')
        phases.append('configuration-failure-blocks-launch-preserves-settings')
        run(['dpkg','--configure','-a'])
        if Path('/run/xdzip/installing').exists() or settings.read_bytes()!=original_settings:
            raise RuntimeError('dpkg recovery did not clear gate or changed settings')
        verify_open('configure-recovery')
        run(['dpkg','-i','/packages/qa2.deb']);verify_open('upgrade')
        active=subprocess.Popen(['/usr/bin/xdzip'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,**user)
        try:
            locked=False
            for _ in range(100):
                if active.poll() is not None:raise RuntimeError('Installed active product exited early')
                with open('/run/xdzip/usage.lock') as lock:
                    try:fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
                    except BlockingIOError:locked=True;break
                time.sleep(.05)
            if not locked:raise RuntimeError('Active runtime lock missing')
            before=run(['dpkg-query','-W','-f=${Version}','xdzip'])
            busy=subprocess.run(['dpkg','-i','/packages/qa1.deb'],capture_output=True,text=True,timeout=20)
            if busy.returncode==0 or before!=run(['dpkg-query','-W','-f=${Version}','xdzip']) or active.poll() is not None:
                raise RuntimeError('Active product was replaced or terminated')
            phases.append('busy-update-rejected')
        finally:
            active.terminate();active.wait(timeout=10)
        run(['dpkg','-i','/packages/qa1.deb']);verify_open('rollback')
        run(['dpkg','-i','/packages/product.deb']);verify_open('production-package')
        run(['dpkg','--remove','xdzip']);run(['dpkg','--purge','xdzip'])
        if Path('/usr/bin/xdzip').exists() or Path('/usr/share/applications/xdzip.desktop').exists():raise RuntimeError('Installed entry points remain')
        if sentinel.read_text()!='keep user data' or 'theme=dark' not in settings.read_text():raise RuntimeError('User data removed')
        if run(['xdg-mime','query','default','application/zip'],**user)!='other.desktop':raise RuntimeError('Uninstall changed foreign default')
        phases.append('uninstall-preserves-user-data-and-default')
        print('XDZIP_INSTALL '+json.dumps({'status':'passed','platform':'linux','phases':phases,'uid':1000,'records':100000,
            'limits':'Xvfb container/GIO, not physical desktop click, GPU, IME, signed release or distribution-wide support.'}))
    finally:display.terminate();display.wait(timeout=10)

if __name__=='__main__':main()
