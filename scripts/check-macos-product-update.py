# scripts/check-macos-product-update.py

"""Real macOS product controller, Sparkle replacement, default profile and association regression."""
import argparse
import base64
import configparser
import ctypes
import hashlib
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import importlib.util
import json
import os
from pathlib import Path
import platform
import plistlib
import shutil
import subprocess
import sys
import tarfile
import threading
import time
import uuid

from deployment_audit import audit, mac_rpaths, native
from desktop_environment import locked_rust_version
from macos_bundle import optimize_bundle, validate_optimized_bundle
from sparkle_bundle import embed, signing_metadata
from update_fixture import ROOT, KEYRING, SEED, PUBLIC, digest, dump
from release_updates import appcast_bytes

PRODUCT = 'org.xdzip.desktop'
LSREGISTER = '/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister'


def environment():
    value = os.environ.copy()
    for name in list(value):
        if (name.startswith(('QT_', 'QML_', 'QML2_', 'DYLD_')) or name in
                ('XDZIP_SETTINGS_PATH', 'XDZIP_LANGUAGE', 'XDZIP_UPDATE_SIGNING_KEY_HEX',
                 'XDZIP_SPARKLE_SIGNING_KEY_HEX', 'XDZIP_QT_PREFIX', 'QTDIR', 'CMAKE_PREFIX_PATH',
                 'CARGO_ENCODED_RUSTFLAGS', 'RUSTFLAGS', 'LD_LIBRARY_PATH')):
            value.pop(name, None)
    value['RUSTUP_TOOLCHAIN'] = locked_rust_version()
    value['MACOSX_DEPLOYMENT_TARGET'] = '14.0'
    value['XDZIP_ISOLATED_UPDATE_FIXTURE'] = '1'
    return value


def run(arguments, log=None, *, env=None, input_data=None):
    result = subprocess.run(list(map(str, arguments)), env=env or environment(),
                            input=input_data, capture_output=True)
    if log:
        with Path(log).open('ab') as stream:
            stream.write(result.stdout+result.stderr)
    if result.returncode:
        raise RuntimeError('Owned Mac product command failed: '+Path(str(arguments[0])).name)
    return result.stdout


def module(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def require_owned(path, parent):
    if path.is_symlink() or path.parent.resolve() != parent.resolve():
        raise RuntimeError('Owned Mac fixture path escaped its declared parent')


def private_source(output):
    source = output/'source'
    archive = output/'source.tar'
    run(['git', '-C', ROOT, 'archive', '--format=tar', '--output', archive, 'HEAD'])
    source.mkdir()
    with tarfile.open(archive) as stream:
        stream.extractall(source, filter='data')
    archive.unlink()
    dump(source/'resources/update-trust.json', KEYRING)
    dump(source/'resources/sparkle-update-key.json', {'schema_version': 1, 'public_key': PUBLIC})
    run(['git', '-C', source, 'init', '--quiet'])
    run(['git', '-C', source, 'config', 'user.name', 'xdzip owned Mac fixture'])
    run(['git', '-C', source, 'config', 'user.email', 'fixture@xdzip.invalid'])
    run(['git', '-C', source, 'config', 'core.hooksPath', source/'.git/no-hooks'])
    return source


def deploy(source, build, qt, sdk, destination, log):
    shutil.copytree(build/'apps/desktop/xdzip.app', destination, symlinks=True)
    run([qt/'bin/macdeployqt', destination, '-always-overwrite', '-no-strip',
         '-qmldir='+str(source/'apps/desktop/ui'), '-qmlimport='+str(build/'qml'), '-verbose=1'], log)
    framework = embed(sdk, destination, installable=True)
    def checked(arguments): return run(arguments, log).decode('utf-8')
    metadata = signing_metadata(framework, checked)
    info_path = destination/'Contents/Info.plist'
    info = plistlib.loads(info_path.read_bytes())
    documents = info.get('CFBundleDocumentTypes', [])
    if (info.get('CFBundleIdentifier') != PRODUCT or not info.get('XdzipUpdateInstallable') or
            not documents or any(d.get('LSHandlerRank') != 'Alternate' for d in documents)):
        raise RuntimeError('Private product must retain formal identity and Alternate document handlers')
    # The only transport exception belongs to this private, signed test build.
    info['NSAppTransportSecurity'] = {'NSAllowsLocalNetworking': True}
    info_path.write_bytes(plistlib.dumps(info, sort_keys=False))
    for driver in (destination/'Contents/PlugIns/sqldrivers').glob('*.dylib'):
        if driver.name != 'libqsqlite.dylib': driver.unlink()
    builder = module(ROOT/'scripts/build-macos-dmg.py', 'owned_mac_product_builder')
    builder.configure_translation_path(destination)
    optimize_bundle(destination)
    for path in {p.resolve() for p in destination.rglob('*') if p.is_file() and native(p, 'macos')}:
        for rpath in mac_rpaths(checked(['otool', '-l', path])):
            if rpath.startswith('/') and not rpath.startswith(('/usr/lib/', '/System/Library/')):
                checked(['install_name_tool', '-delete_rpath', rpath, path])
    builder.sign_local(destination, metadata, destination.parent)
    for path in destination.parent.glob('sparkle-helper-*.entitlements'): path.unlink()
    validate_optimized_bundle(destination)
    binary = destination/'Contents/MacOS/xdzip'
    report = audit(destination, binary, None, 'macos', 'deployed')
    if report['status'] != 'passed': raise RuntimeError('Private product is not SDK independent')
    return json.loads(run([binary, '--version-json']))


def prepare(output, qt, session, app_name):
    source = private_source(output)
    sdk = output/'sdk'
    run([sys.executable, source/'scripts/prepare-sparkle.py', '--output', sdk], output/'sdk.log')
    build = source/'build/desktop'
    products = {}
    for label, number, build_number in (('old', 1, 200), ('new', 2, 201)):
        dump(source/'VERSION.json', {'version': '0.2.0', 'channel': 'alpha', 'build': build_number, 'prerelease_number': number})
        run([sys.executable, source/'scripts/sync-version.py'], output/(label+'-build.log'))
        run(['cargo', 'update', '--workspace', '--offline', '--manifest-path', source/'Cargo.toml'], output/(label+'-build.log'))
        run(['git', '-C', source, 'add', '--all'])
        run(['git', '-C', source, '-c', 'commit.gpgsign=false', 'commit', '--quiet', '--allow-empty',
             '-m', 'Owned Mac product update '+label])
        commit = run(['git', '-C', source, 'rev-parse', 'HEAD']).decode().strip()
        run(['cmake', '-S', source, '-B', build, '-G', 'Ninja', '-DCMAKE_BUILD_TYPE=Release',
             '-DCMAKE_PREFIX_PATH='+str(qt), '-DCMAKE_OSX_ARCHITECTURES=arm64', '-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0',
             '-DXDZIP_BUILD_QT_PROBE=OFF', '-DBUILD_TESTING=OFF', '-DXDZIP_BUILD_DESKTOP_PREVIEW=ON',
             '-DXDZIP_MACOS_HANDLER_RANK=Alternate', '-DXDZIP_NATIVE_MAC_UPDATE_FIXTURE=ON',
             '-DXDZIP_NATIVE_MAC_UPDATE_FIXTURE_DIRECTORY='+str(session), '-DXDZIP_SPARKLE_SDK='+str(sdk)], output/(label+'-build.log'))
        run(['cmake', '--build', build, '--target', 'xdzip-browser', '--parallel', '3'], output/(label+'-build.log'))
        folder = output/label; folder.mkdir()
        app = folder/app_name
        identity = deploy(source, build, qt, sdk, app, output/(label+'-deployment.log'))
        expected = {'product': 'xdzip', 'version': '0.2.0-alpha.'+str(number), 'build': build_number,
                    'revision': commit[:12], 'dirty': False}
        if identity != expected: raise RuntimeError('Private product did not retain clean Release identity')
        if run(['git', '-C', source, 'status', '--porcelain']).strip():
            raise RuntimeError('Private build modified tracked source after committing identity')
        products[label] = dict(identity=identity, commit=commit, app=app)
    run(['cargo', 'build', '--locked', '--manifest-path', source/'Cargo.toml', '--target-dir', source/'build/signer',
         '-p', 'xdzip-update', '--bin', 'xdzip-update-tool'], output/'signer-build.log')
    return products, source/'build/signer/debug/xdzip-update-tool', sdk


def signed_candidate(output, new, signing, sdk):
    directory = output/'owned'; directory.mkdir()
    version = new['identity']['version']
    package = directory/('xdzip-'+version+'-macos-arm64.app.zip')
    run(['ditto', '-c', '-k', '--sequesterRsrc', '--keepParent', new['app'], package])
    env = environment(); env['XDZIP_SPARKLE_SIGNING_KEY_HEX'] = SEED
    public = output/'source/resources/sparkle-update-key.json'
    metadata = directory/'macos-update.json'
    run([signing, 'sign-archive', '--archive', package, '--public-key', public, '--version', version,
         '--build', '201', '--output', metadata], env=env)
    base = 'https://github.com/xiaodaozhi/xdzip/releases/download/v'+version+'/'
    targets = [('windows','x86_64','zip','windows-x86_64.zip',{'windows':'11'}),
               ('windows','x86_64','exe','windows-x86_64-setup.exe',{'windows':'11'}),
               ('macos','arm64','app.zip','macos-arm64.app.zip',{'macos':'14.0'}),
               ('macos','arm64','dmg','macos-arm64.dmg',{'macos':'14.0'}),
               ('linux','x86_64','tar.gz','linux-x86_64.tar.gz',{'distribution':'ubuntu','version':'24.04','glibc':'2.39','display_server':'X11'}),
               ('linux','x86_64','deb','ubuntu24.04-x86_64.deb',{'distribution':'ubuntu','version':'24.04','glibc':'2.39','display_server':'X11'})]
    assets=[]
    for system, arch, kind, suffix, minimum in targets:
        filename='xdzip-'+version+'-'+suffix
        selected=kind=='app.zip'
        assets.append(dict(filename=filename,platform=system,architecture=arch,package_type=kind,minimum_system=minimum,
                           bytes=package.stat().st_size if selected else 1,
                           sha256=digest(package) if selected else hashlib.sha256(b'x').hexdigest(),url=base+filename))
    manifest=dict(schema_version=1,product='xdzip',version=version,channel='alpha',build=201,tag='v'+version,
                  commit=new['commit'],repository='xiaodaozhi/xdzip',assets=assets,macos_appcast_url=base+'appcast.xml')
    dump(directory/'release.json',manifest)
    env['XDZIP_UPDATE_SIGNING_KEY_HEX']=SEED
    run([signing,'sign','--manifest',directory/'release.json','--keyring',output/'source/resources/update-trust.json',
         '--key-id','owned-fixture','--output',directory/'release.json.sig'],env=env)
    run([signing,'verify','--manifest',directory/'release.json','--keyring',output/'source/resources/update-trust.json',
         '--signature',directory/'release.json.sig'])
    appcast=directory/'appcast.xml'; appcast.write_bytes(appcast_bytes(manifest,json.loads(metadata.read_bytes())))
    secret=base64.b64encode(bytes.fromhex(SEED))+b'\n'
    run([sdk/'bin/sign_update','--ed-key-file','-','--disable-signing-warning',appcast],input_data=secret)
    run([signing,'verify-appcast','--appcast',appcast,'--public-key',public])
    return manifest,package


class Associations:
    def __init__(self):
        self.cf=ctypes.CDLL('/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation')
        self.ls=ctypes.CDLL('/System/Library/Frameworks/CoreServices.framework/CoreServices')
        self.cf.CFStringCreateWithCString.argtypes=[ctypes.c_void_p,ctypes.c_char_p,ctypes.c_uint32]
        self.cf.CFStringCreateWithCString.restype=ctypes.c_void_p
        self.cf.CFStringGetCString.argtypes=[ctypes.c_void_p,ctypes.c_char_p,ctypes.c_long,ctypes.c_uint32]
        self.cf.CFStringGetCString.restype=ctypes.c_bool
        self.cf.CFRelease.argtypes=[ctypes.c_void_p]
        self.ls.LSCopyDefaultRoleHandlerForContentType.argtypes=[ctypes.c_void_p,ctypes.c_uint32]
        self.ls.LSCopyDefaultRoleHandlerForContentType.restype=ctypes.c_void_p
        self.ls.LSSetDefaultRoleHandlerForContentType.argtypes=[ctypes.c_void_p,ctypes.c_uint32,ctypes.c_void_p]
        self.ls.LSSetDefaultRoleHandlerForContentType.restype=ctypes.c_int32
    def string(self,value): return self.cf.CFStringCreateWithCString(None,value.encode(),0x08000100)
    def get(self,role):
        uti=self.string('public.zip-archive')
        try: value=self.ls.LSCopyDefaultRoleHandlerForContentType(uti,role)
        finally: self.cf.CFRelease(uti)
        if not value:return ''
        try:
            data=ctypes.create_string_buffer(8192)
            if not self.cf.CFStringGetCString(value,data,len(data),0x08000100):raise RuntimeError('Default handler is not UTF-8')
            return data.value.decode()
        finally:self.cf.CFRelease(value)
    def set(self,role,identity):
        uti,value=self.string('public.zip-archive'),self.string(identity)
        try:
            status=self.ls.LSSetDefaultRoleHandlerForContentType(uti,role,value)
            if status:raise RuntimeError('Cannot set owned default ZIP Viewer handler, OSStatus='+str(status))
        finally:self.cf.CFRelease(uti);self.cf.CFRelease(value)
    def snapshot(self):return {str(role):self.get(role) for role in (0xffffffff,2,4,8)}


def other_handler(output, identity):
    app=output/'owned-other-handler.app'; (app/'Contents/MacOS').mkdir(parents=True)
    # Copy executable bytes only. System binaries carry protected BSD flags
    # which an owned fixture must neither copy nor attempt to set.
    handler=app/'Contents/MacOS/handler'
    shutil.copyfile('/usr/bin/true',handler);handler.chmod(0o755)
    info=dict(CFBundleIdentifier=identity,CFBundleExecutable='handler',CFBundleName='Owned ZIP regression handler',
              CFBundlePackageType='APPL',CFBundleVersion='1',LSUIElement=True,CFBundleDocumentTypes=[{
              'CFBundleTypeName':'Owned ZIP fixture','CFBundleTypeRole':'Viewer','LSHandlerRank':'Owner',
              'LSItemContentTypes':['public.zip-archive'],'CFBundleTypeExtensions':['zip']}])
    (app/'Contents/Info.plist').write_bytes(plistlib.dumps(info))
    run(['codesign','--force','--sign','-','--timestamp=none',app]);run([LSREGISTER,'-f',app])
    return app


def await_json(path, timeout=90, process=None, session=None):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        if session and (session/'fixture-error.json').exists():raise RuntimeError((session/'fixture-error.json').read_text())
        if session:
            for state_path in (session/'200-state.json',session/'201-state.json'):
                if state_path.is_file():
                    state=json.loads(state_path.read_bytes())
                    if state.get('state') in ('error','recovery_required'):
                        raise RuntimeError('Real Mac product update failed: '+state.get('error',''))
        if path.exists():return json.loads(path.read_bytes())
        if process and process.poll() is not None:raise RuntimeError('Owned product exited before '+path.name)
        time.sleep(.1)
    raise TimeoutError('Owned product timed out waiting for '+path.name)


def stop_pid(pid, timeout=30):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        try:os.kill(pid,0)
        except ProcessLookupError:return
        time.sleep(.1)
    raise TimeoutError('Owned new product did not exit normally')


class Handler(SimpleHTTPRequestHandler):
    def log_message(self,*_):pass
    def log_request(self,code='-',size='-'):
        with self.server.evidence_lock:
            self.server.evidence.append(dict(path=self.path[:1024],status=code))
            dump(Path(self.directory)/'http-requests.json',self.server.evidence[-64:])


def same_roles(left,right):
    # kLSRolesAll is a query over a role set, not a separately restorable slot.
    return all(left[str(role)]==right[str(role)] for role in (2,4,8))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-prefix',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    if (sys.platform!='darwin' or platform.machine()!='arm64' or os.environ.get('CI')!='true' or
        os.environ.get('GITHUB_ACTIONS')!='true' or os.environ.get('RUNNER_ENVIRONMENT')!='github-hosted'):
        parser.error('Requires an explicitly disposable GitHub-hosted Apple Silicon runner')
    output,qt=args.output.resolve(),args.qt_prefix.resolve()
    if not output.is_relative_to(ROOT/'build') or output.exists():parser.error('Use fresh output beneath build/')
    home=Path.home(); applications=home/'Applications'
    if applications.is_symlink():parser.error('Refusing a redirected Applications directory')
    roots=[home/'Library/Preferences/xdzip/xdzip',home/'Library/Application Support/xdzip/xdzip',home/'Library/Caches/xdzip/xdzip',
           home/'Library/Caches/org.xdzip.desktop',home/'Library/Application Support/org.xdzip.desktop']
    existing_parents={path.parent:path.parent.exists() for path in roots}
    preferences=home/'Library/Preferences/org.xdzip.desktop.plist'
    for path in roots+[preferences]:
        if path.exists() or path.is_symlink():parser.error('Refusing an existing xdzip profile or cache')
    for parent in (Path('/Applications'),applications):
        for app in parent.glob('*.app'):
            info=app/'Contents/Info.plist'
            if info.is_file() and plistlib.loads(info.read_bytes()).get('CFBundleIdentifier')==PRODUCT:
                parser.error('Refusing a runner with an existing xdzip installation')
    output.mkdir(parents=True)
    token=uuid.uuid4().hex
    name='xdzip-owned-update-更新-'+token+'.app'
    applications.mkdir(exist_ok=True)
    installed=applications/name; require_owned(installed,applications)
    session=output/'session';session.mkdir()
    associations=Associations();before_preparation=associations.snapshot()
    dump(output/'association-before-preparation.json',before_preparation)
    production={str(path.relative_to(ROOT)):digest(path) for path in (ROOT/'resources/update-trust.json',ROOT/'resources/sparkle-update-key.json')}
    products,signing,sdk=prepare(output,qt,session,name)
    # Explicitly register the owned, formal Alternate product during preparation.
    # Restore its defined pre-mutation Viewer, not an invented missing handler.
    run([LSREGISTER,'-f',products['old']['app']])
    original=associations.snapshot()
    dump(output/'association-original.json',original)
    if not original['2']:raise RuntimeError('Cannot safely restore a missing pre-mutation ZIP Viewer handler')
    manifest,archive=signed_candidate(output,products['new'],signing,sdk)
    server=ThreadingHTTPServer(('127.0.0.1',0),partial(Handler,directory=str(output)))
    server.evidence_lock=threading.Lock();server.evidence=[]
    thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
    address='http://127.0.0.1:'+str(server.server_port)
    base='https://github.com/xiaodaozhi/xdzip/releases/download/'+manifest['tag']+'/'
    for filename in ('release.json','release.json.sig'):shutil.copy2(output/'owned'/filename,session/filename)
    assets=[dict(name=a['filename'],browser_download_url=a['url'],size=a['bytes']) for a in manifest['assets']]
    assets += [dict(name=f,browser_download_url=base+f) for f in ('release.json','release.json.sig')]
    dump(session/'releases.json',[dict(tag_name=manifest['tag'],draft=False,prerelease=True,
         html_url='https://github.com/xiaodaozhi/xdzip/releases/tag/'+manifest['tag'],assets=assets)])
    dump(session/'configuration.json',dict(marker='xdzip-owned-mac-product-update-v1',install_root=str(installed),
         fixed_asset_url=base+archive.name,fixed_appcast_url=base+'appcast.xml',loopback_appcast_url=address+'/owned/appcast.xml',
         loopback_asset_url=address+'/owned/'+archive.name,routes={base+'release.json':'release.json',base+'release.json.sig':'release.json.sig',
         'https://api.github.com/repos/xiaodaozhi/xdzip/releases?per_page=100&page=1':'releases.json'}))
    other=None;old=None;new=None;log=None;record={'status':'failed'};stage='association_setup'
    try:
        other_id='org.xdzip.owned-zip-handler.'+token
        other=output/'owned-other-handler.app'
        other_handler(output,other_id)
        associations.set(2,other_id)
        baseline=associations.snapshot()
        dump(output/'association-baseline.json',baseline)
        if baseline['2']!=other_id:raise RuntimeError('Owned default Viewer handler did not register')
        stage='install_old'
        shutil.copytree(products['old']['app'],installed,symlinks=True)
        run([LSREGISTER,'-f',installed])
        if not same_roles(associations.snapshot(),baseline):raise RuntimeError('Old product registration changed ZIP defaults')
        log=(output/'product.log').open('wb')
        executable=installed/'Contents/MacOS/xdzip'
        launch_env=environment();launch_env['PATH']='/usr/bin:/bin:/usr/sbin:/sbin'
        stage='launch_old'
        old=subprocess.Popen([str(executable)],env=launch_env,stdout=log,stderr=subprocess.STDOUT)
        started=await_json(session/'200-started.json',process=old,session=session)
        profile=Path(started['settings_path'])
        if not profile.resolve().is_relative_to(roots[0]) or started['settings_override'] or started['primary_windows']!=1:
            raise RuntimeError('Old product did not use its ordinary default profile/window')
        baseline_profile=configparser.ConfigParser();baseline_profile.read(profile,encoding='utf-8')
        stage='single_instance_old'
        peer=subprocess.Popen([str(executable)],env=launch_env,stdout=log,stderr=subprocess.STDOUT)
        if peer.wait(timeout=20)!=0:raise RuntimeError('Old normal SingleInstance startup failed')
        stage='upgrade'
        dump(session/'upgrade.json',{'owned':True})
        new=await_json(session/'201-started.json',timeout=240,session=session)
        redirect=await_json(session/'asset-redirect.json',session=session)
        if (redirect.get('transport')!='Sparkle public willDownloadUpdate delegate' or
            redirect.get('fixed_asset_url')!=base+archive.name or
            redirect.get('loopback_asset_url')!=address+'/owned/'+archive.name):
            raise RuntimeError('Private request mapping did not bind the authenticated native asset')
        if old.wait(timeout=20)!=0:raise RuntimeError('Old process failed ordinary exit')
        barrier=await_json(session/'200-exit-barrier.json',session=session)
        if not (barrier['about_check_clicked'] and barrier['about_install_clicked'] and barrier['trusted'] and
                barrier['can_install'] and barrier['update_pending'] and barrier['active_tasks']==0 and
                barrier['pending'].get('kind')=='sparkle' and barrier['pending'].get('build')==201):
            raise RuntimeError('Real controller/join/pending handoff did not meet the product contract')
        stage='verify_restart'
        expected=products['new']['identity']
        if (any(new[key]!=expected[key] for key in ('version','build','revision','dirty')) or new['configuration']!='Release' or
            new['state']!='updated' or new['error'] or new['pid']==old.pid or new['primary_windows']!=1 or
            new['install_root']!=str(installed) or new['settings_path']!=started['settings_path'] or new['settings_override'] or
            new['home']!=started['home'] or new['theme']!='dark' or new['text_scale']!=1.25 or new['density']!='compact' or
            new['language']!='en-US' or new['active_language']!='en-US' or new['unknown_key']!='preserve-across-native-Sparkle'):
            raise RuntimeError('Ordinary new startup did not restore identity, update state and persistent settings')
        after_profile=configparser.ConfigParser();after_profile.read(profile,encoding='utf-8')
        if {s:dict(after_profile[s]) for s in after_profile.sections()}!={s:dict(baseline_profile[s]) for s in baseline_profile.sections()}:
            raise RuntimeError('Product profile changed across update')
        stage='verify_associations'
        run([LSREGISTER,'-f',installed]);after_associations=associations.snapshot()
        dump(output/'association-after-update.json',after_associations)
        if not same_roles(after_associations,baseline):raise RuntimeError('Native Sparkle replacement changed default ZIP handlers')
        run(['codesign','--verify','--deep','--strict',installed],output/'codesign.log')
        stage='single_instance_new'
        peer=subprocess.Popen([str(executable)],env=launch_env,stdout=log,stderr=subprocess.STDOUT)
        if peer.wait(timeout=20)!=0:raise RuntimeError('New ordinary SingleInstance startup failed')
        record=dict(status='passed',scope='Real browser_main/About/UpdateController, formal installed identity, native Sparkle, ordinary receipt restoration and default profile',
                    production_trust_modified=False,old=started,new=new,exit_barrier=barrier,
                    settings_preserved=True,unknown_setting_preserved=True,single_instance_before_and_after=True,
                    zip_association_scope='Owned LaunchServices public.zip-archive Viewer default for ordinary ZIP opening; Editor/Shell checked read-only, all-roles aggregate informational only',
                    before_preparation_zip_handlers=before_preparation,
                    preparation_registration='Explicit lsregister of the owned prepared old product with formal Alternate document handlers',
                    restoration_scope='Pre-mutation handlers after private product preparation; preparation-time LaunchServices discovery is recorded, not reversed',
                    original_zip_handlers=original,owned_zip_handlers=baseline,after_zip_handlers=after_associations,
                    network_seam='Private Qt metadata transport and exact feed URL mapping; public Sparkle download-request delegate maps only the matched authenticated GitHub asset request, preserving item URL/signature/length',
                    archive_transport=redirect,
                    limits=['Idle successful upgrade only; rejection/cancellation use the separate native Sparkle scenarios.',
                            'Default handlers for other archive UTIs and physical user acceptance are not claimed.'])
    except Exception as error:
        record=dict(status='failed',stage=stage,error=type(error).__name__,detail=str(error));raise
    finally:
        cleanup=[]
        try:
            if new is None and (session/'201-state.json').is_file():
                new=json.loads((session/'201-state.json').read_bytes())
            if new:
                dump(session/'stop.json',{'owned':True});stop_pid(new['pid'])
            elif old and old.poll() is None:
                dump(session/'stop.json',{'owned':True});old.wait(timeout=20)
        except Exception as error:
            cleanup.append(dict(stage='stop_owned_product',error=type(error).__name__))
            if old and old.poll() is None:old.terminate();old.wait(timeout=10)
        if log:log.close()
        try:
            current=associations.get(2)
            if current!=original['2']:
                if current in ('org.xdzip.owned-zip-handler.'+token,PRODUCT):
                    associations.set(2,original['2'])
                else:raise RuntimeError('Unowned default Viewer association changed during fixture')
            after_restore=associations.snapshot();dump(output/'association-after-restore.json',after_restore)
            if not same_roles(after_restore,original):raise RuntimeError('Could not restore pre-mutation ZIP handlers')
            record['pre_mutation_zip_handlers_restored']=True
        except Exception as error:cleanup.append(dict(stage='restore_associations',error=type(error).__name__,detail=str(error)))
        for app in (installed,other):
            if not app or not app.exists():continue
            try:
                require_owned(app,applications if app==installed else output)
                try:
                    if (app/'Contents/Info.plist').is_file():run([LSREGISTER,'-u',app])
                finally:shutil.rmtree(app)
            except Exception as error:cleanup.append(dict(stage='remove_owned_application',error=type(error).__name__))
        try:
            after_unregister=associations.snapshot();dump(output/'association-after-unregister.json',after_unregister)
            if not same_roles(after_unregister,original):raise RuntimeError('Unregistering owned applications changed pre-mutation handlers')
        except Exception as error:cleanup.append(dict(stage='verify_restored_associations',error=type(error).__name__,detail=str(error)))
        subprocess.run(['defaults','delete',PRODUCT],capture_output=True)
        for path in roots:
            try:
                if path.exists():
                    require_owned(path,path.parent);shutil.rmtree(path)
            except Exception as error:cleanup.append(dict(stage='remove_owned_profile',error=type(error).__name__))
        for parent,existed in existing_parents.items():
            if not existed and parent.is_dir() and not any(parent.iterdir()):parent.rmdir()
        for name,sha in production.items():
            if digest(ROOT/name)!=sha:cleanup.append(dict(stage='production_trust_changed',error='digest_mismatch'))
        server.shutdown();server.server_close();thread.join(timeout=5)
        if cleanup:record.update(status='failed',cleanup_errors=cleanup)
        dump(output/'verification.json',record)
        if cleanup:raise RuntimeError('Owned Mac product cleanup failed')
    print('Real Mac product signed update, ordinary restart, profile and ZIP handlers passed')


if __name__=='__main__':main()
