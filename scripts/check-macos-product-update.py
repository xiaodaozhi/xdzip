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
import re
import shutil
import stat
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
from release_updates import appcast_bytes, verify_release_updates

PRODUCT = 'org.xdzip.desktop'
LSREGISTER = '/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister'
PRIVATE_KEY_NAMES = ('XDZIP_UPDATE_SIGNING_KEY_HEX', 'XDZIP_SPARKLE_SIGNING_KEY_HEX')
PUBLIC_RESOURCES = ('update-trust.json', 'sparkle-update-key.json')


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


def private_source(output, production_trust=False):
    source = output/'source'
    archive = output/'source.tar'
    run(['git', '-C', ROOT, 'archive', '--format=tar', '--output', archive, 'HEAD'])
    source.mkdir()
    with tarfile.open(archive) as stream:
        stream.extractall(source, filter='data')
    archive.unlink()
    if not production_trust:
        dump(source/'resources/update-trust.json', KEYRING)
        dump(source/'resources/sparkle-update-key.json', {'schema_version': 1, 'public_key': PUBLIC})
    else:
        for name in PUBLIC_RESOURCES:
            if (source/'resources'/name).read_bytes() != (ROOT/'resources'/name).read_bytes():
                raise RuntimeError('Production public resource differs from the committed source')
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


def prepare(output, qt, session, app_name, production_trust=False):
    source = private_source(output, production_trust)
    run(['cargo', 'fetch', '--locked', '--manifest-path', source/'Cargo.toml'], output/'cargo-fetch.log')
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


def unsigned_candidate(output, new):
    directory = output/'owned'; directory.mkdir()
    version = new['identity']['version']
    package = directory/('xdzip-'+version+'-macos-arm64.app.zip')
    run(['ditto', '-c', '-k', '--sequesterRsrc', '--keepParent', new['app'], package])
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
    return manifest,package


def signed_candidate(output, new, signing, sdk):
    manifest,package = unsigned_candidate(output, new)
    directory = output/'owned'
    env = environment(); env['XDZIP_SPARKLE_SIGNING_KEY_HEX'] = SEED
    public = output/'source/resources/sparkle-update-key.json'
    metadata = directory/'macos-update.json'
    run([signing, 'sign-archive', '--archive', package, '--public-key', public,
         '--version', new['identity']['version'], '--build', '201', '--output', metadata], env=env)
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


def require_no_production_secrets():
    # Preparation and product execution must never inherit signing authority.
    # The separate protected signer is the only process allowed to consume it.
    if any(name.upper() in PRIVATE_KEY_NAMES for name in os.environ):
        raise RuntimeError('Production preparation and execution require an environment without signing keys')


def public_resource_hashes():
    return {'resources/'+name:digest(ROOT/'resources'/name) for name in PUBLIC_RESOURCES}


def production_file(output, relative):
    if not isinstance(relative, str) or '\\' in relative:
        raise RuntimeError('Invalid prepared production path')
    path = output/relative
    if Path(relative).is_absolute() or path.resolve() != path.absolute() or not path.resolve().is_relative_to(output):
        raise RuntimeError('Prepared production path escaped its owned output')
    if not path.is_file() or path.is_symlink():
        raise RuntimeError('Prepared production input is not a regular file')
    return path


def production_prepared(output, products, signing):
    """Freeze only public build inputs; final signatures are made in another step."""
    require_no_production_secrets()
    manifest,package = unsigned_candidate(output, products['new'])
    resource_dir = output/'resources';resource_dir.mkdir()
    for resource in PUBLIC_RESOURCES:
        shutil.copyfile(ROOT/'resources'/resource, resource_dir/resource)
    tool_dir=output/'signing';tool_dir.mkdir()
    tool=tool_dir/'xdzip-update-tool';shutil.copy2(signing,tool)
    expected_key=base64.b64encode(bytes.fromhex(json.loads((resource_dir/'sparkle-update-key.json').read_bytes())['public_key'])).decode('ascii')
    recorded_products={}
    for label,product in products.items():
        app=product['app'];plist=app/'Contents/Info.plist'
        info=plistlib.loads(plist.read_bytes())
        if info.get('SUPublicEDKey') != expected_key:
            raise RuntimeError('Prepared application does not embed the production Sparkle public key')
        recorded_products[label]=dict(identity=product['identity'],commit=product['commit'],
            app=app.relative_to(output).as_posix(),SUPublicEDKey=info['SUPublicEDKey'],
            binary_sha256=digest(app/'Contents/MacOS/xdzip'),plist_sha256=digest(plist))
    inputs=[output/'owned/release.json',package,tool]+[resource_dir/name for name in PUBLIC_RESOURCES]
    record=dict(schema_version=1,platform='macos',source_commit=run(['git','-C',ROOT,'rev-parse','HEAD']).decode().strip(),
        tool=dict(path='signing/xdzip-update-tool',sha256=digest(tool)),
        artifacts={path.relative_to(output).as_posix():dict(bytes=path.stat().st_size,sha256=digest(path)) for path in inputs},
        products=recorded_products)
    dump(output/'production-update-prepared.json',record)
    return record


def load_production_prepared(output):
    require_no_production_secrets()
    record_path=production_file(output,'production-update-prepared.json')
    if record_path.stat().st_size>64*1024:raise RuntimeError('Prepared production record exceeds its bound')
    record=json.loads(record_path.read_bytes())
    source_commit=run(['git','-C',ROOT,'rev-parse','HEAD']).decode().strip()
    if (set(record)!={'schema_version','platform','source_commit','tool','artifacts','products'} or
            record.get('schema_version')!=1 or record.get('platform')!='macos' or record.get('source_commit')!=source_commit):
        raise RuntimeError('Prepared production source or trust identity changed')
    expected_paths={'owned/release.json','owned/xdzip-0.2.0-alpha.2-macos-arm64.app.zip',
                    'signing/xdzip-update-tool','resources/update-trust.json','resources/sparkle-update-key.json'}
    if set(record.get('artifacts',{}))!=expected_paths:
        raise RuntimeError('Unexpected prepared production inputs')
    for relative,expected in record['artifacts'].items():
        path=production_file(output,relative)
        if expected!={'bytes':path.stat().st_size,'sha256':digest(path)}:
            raise RuntimeError('Prepared production input changed')
    for name in PUBLIC_RESOURCES:
        if (output/'resources'/name).read_bytes()!=(ROOT/'resources'/name).read_bytes():
            raise RuntimeError('Prepared production trust differs from the source')
    tool=production_file(output,'signing/xdzip-update-tool')
    if record.get('tool')!={'path':'signing/xdzip-update-tool','sha256':digest(tool)}:
        raise RuntimeError('Prepared production tool changed')
    expected_key=base64.b64encode(bytes.fromhex(json.loads((output/'resources/sparkle-update-key.json').read_bytes())['public_key'])).decode('ascii')
    products={}
    if set(record.get('products',{}))!={'old','new'}:raise RuntimeError('Prepared products are incomplete')
    old_path=record['products']['old'].get('app','')
    app_match=re.fullmatch('old/(xdzip-owned-update-更新-([0-9a-f]{32})\\.app)',old_path)
    if not app_match:raise RuntimeError('Prepared application name is not owned')
    app_name,token=app_match.groups()
    for label,number in (('old',1),('new',2)):
        product=record['products'][label]
        expected_app=label+'/'+app_name
        if product.get('app')!=expected_app:raise RuntimeError('Prepared application path changed')
        plist=production_file(output,expected_app+'/Contents/Info.plist')
        binary=production_file(output,expected_app+'/Contents/MacOS/xdzip')
        info=plistlib.loads(plist.read_bytes())
        if (digest(plist)!=product.get('plist_sha256') or digest(binary)!=product.get('binary_sha256') or
                info.get('SUPublicEDKey')!=expected_key or product.get('SUPublicEDKey')!=expected_key or
                not re.fullmatch('[0-9a-f]{40}',product.get('commit','')) or
                product.get('identity')!={'product':'xdzip','version':'0.2.0-alpha.'+str(number),'build':199+number,
                                         'revision':product['commit'][:12],'dirty':False}):
            raise RuntimeError('Prepared production application identity changed')
        products[label]=dict(identity=product['identity'],commit=product['commit'],app=output/expected_app)
    manifest=json.loads((output/'owned/release.json').read_bytes())
    if (manifest.get('commit')!=products['new']['commit'] or manifest.get('version')!='0.2.0-alpha.2' or manifest.get('build')!=201):
        raise RuntimeError('Prepared manifest does not match the new product')
    # These checks execute with no production keys, before any application or
    # native installer is launched. The installer will independently verify again.
    verify_release_updates(output/'owned',manifest,tool=tool,keyring=output/'resources/update-trust.json',
                           public_key=output/'resources/sparkle-update-key.json')
    record.update(token=token,app_name=app_name)
    return record,products,manifest,output/'owned/xdzip-0.2.0-alpha.2-macos-arm64.app.zip'


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


def await_json(path, timeout=90, process=None, session=None, *, expected_recovery_error=None, allow_closing=False):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        if session and (session/'fixture-error.json').exists():raise RuntimeError((session/'fixture-error.json').read_text())
        if session:
            for state_path in (session/'200-state.json',session/'201-state.json'):
                if state_path.is_file():
                    state=json.loads(state_path.read_bytes())
                    if state.get('state') in ('error','recovery_required'):
                        old_product=(state_path.name=='200-state.json' and state.get('build')==200 and
                                     state.get('version')=='0.2.0-alpha.1')
                        expected_recovery=(old_product and
                                           state.get('state')=='recovery_required' and
                                           expected_recovery_error is not None and
                                           state.get('error')==expected_recovery_error)
                        expected_closing=(old_product and allow_closing and
                                          state.get('state')=='error' and state.get('error')=='closing')
                        if not expected_recovery and not expected_closing:
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


def restore_owned_flags(installed,applications,fault):
    require_owned(installed,applications)
    current=installed.lstat()
    if (not stat.S_ISDIR(current.st_mode) or current.st_uid!=os.getuid() or
        current.st_dev!=int(fault['device']) or current.st_ino!=int(fault['inode'])):
        raise RuntimeError('Refusing to restore flags on a changed installed inode')
    os.chflags(installed,int(fault['flags_before']),follow_symlinks=False)
    restored=installed.lstat()
    if (restored.st_flags!=int(fault['flags_before']) or restored.st_dev!=current.st_dev or
        restored.st_ino!=current.st_ino or restored.st_uid!=os.getuid() or not stat.S_ISDIR(restored.st_mode)):
        raise RuntimeError('Owned immutable fault flags were not restored')
    return dict(device=str(restored.st_dev),inode=str(restored.st_ino),flags=restored.st_flags,
                original_flags_restored=True,exact_fault_inode=True)


def wait_owned_helpers(pids,installed,timeout=90):
    if not pids or any(not isinstance(pid,int) or pid<=1 for pid in pids):
        raise RuntimeError('Missing native ready helper identities')
    expected=(installed/'Contents/Frameworks/Sparkle.framework/Autoupdate').resolve()
    limit=time.monotonic()+timeout
    observations=0
    env=environment();env['LC_ALL']='en_US.UTF-8'
    while True:
        observations+=1
        alive=[]
        for pid in pids:
            result=subprocess.run(['/bin/ps','-ww','-p',str(pid),'-o','comm='],capture_output=True,env=env,timeout=5)
            value=result.stdout.decode('utf-8',errors='strict').strip()
            if result.returncode==1 and not value:continue
            if result.returncode or not value or '\n' in value:raise RuntimeError('Native helper observation failed')
            if Path(value).resolve()!=expected:raise RuntimeError('Captured native helper identity changed')
            alive.append(pid)
        if not alive:return dict(captured_pids=pids,remaining_pids=[],observations=observations,gone=True)
        if time.monotonic()>limit:raise RuntimeError('Native installer did not exit without intervention')
        time.sleep(.1)


def native_log_pid(entry,pids):
    values=[entry[key] for key in ('processID','processIdentifier') if key in entry]
    if values and all(type(value) is int and value in pids and value==values[0] for value in values):return values[0]
    return None


def native_log_errors(entries):
    messages=[entry.get('eventMessage','') for entry in entries if isinstance(entry.get('eventMessage',''),str)]
    return (any('Failed to finalize installation' in text for text in messages),
            any('Operation not permitted' in text or 'Permission denied' in text or
                re.search(r'NSPOSIXErrorDomain Code=(?:1|13)\b|NSCocoaErrorDomain Code=513\b',text) for text in messages))


def native_log_matches(events,pids,installed,expected):
    matched=[];direct={};groups={};receipts=[]
    for entry in events:
        pid=native_log_pid(entry,pids);image=entry.get('processImagePath','')
        if pid is None or not image or not Path(image).is_absolute():continue
        try:same_image=os.path.samefile(image,expected)
        except OSError:same_image=False
        if same_image:direct.setdefault(pid,[]).append(entry)
        elif Path(image).name=='Autoupdate':groups.setdefault((pid,image),[]).append(entry)
    for entries in direct.values():
        if all(native_log_errors(entries)):matched.extend(entries)
    case_root=(ROOT/'build/update-regression/macos/cases').resolve()
    labels=('tampered-feed','tampered-archive','cancel-ready','cancel-ready-repeat-1','cancel-ready-repeat-2','success')
    suffix=('xdzip-update-probe.app','Contents','Frameworks','Sparkle.framework','Versions','B','Autoupdate')
    expected_sha=digest(expected)
    for (pid,image),entries in groups.items():
        catalog=Path(image).resolve()
        try:parts=catalog.relative_to(case_root).parts
        except ValueError:continue
        if len(parts)!=8 or parts[0] not in labels or parts[1:]!=suffix:continue
        try:
            owned=catalog.is_file() and catalog.stat().st_uid==os.getuid()
            catalog_sha=digest(catalog) if owned else None
        except OSError:continue
        move_chain=any(isinstance(entry.get('eventMessage'),str) and
                       ('Failed to move the old app at '+str(installed)+' to a temporary location at ') in
                       entry['eventMessage'] for entry in entries)
        finalize,permission=native_log_errors(entries)
        accepted=catalog_sha==expected_sha and move_chain and finalize and permission
        receipts.append(dict(method='captured_pid_owned_move_chain_same_sha_catalog',process_identifier=pid,
                             accepted=accepted,catalog_path=str(catalog),expected_sha256=expected_sha,
                             catalog_sha256=catalog_sha,current_owned_move_chain=move_chain,catalog_owned=owned,
                             same_process_finalize=finalize,same_process_permission=permission))
        if accepted:matched.extend(entries)
    return matched,receipts


def native_replacement_error(pids,installed,timeout=20):
    predicate=' OR '.join('processIdentifier == '+str(pid) for pid in pids)
    expected=(installed/'Contents/Frameworks/Sparkle.framework/Autoupdate').resolve()
    def sanitized(value):
        return str(value).replace(str(Path.home()),'$HOME').replace(str(ROOT),'$WORKSPACE')[:4096]
    deadline=time.monotonic()+timeout
    attempts=[];messages=[];unaccepted_messages=[];receipts=[];finalize=False;permission=False
    while True:
        if attempts and time.monotonic()>=deadline:break
        try:
            result=subprocess.run(['/usr/bin/log','show','--style','json','--info','--debug','--last','5m','--predicate',predicate],
                                  capture_output=True,env=environment(),timeout=max(1,min(10,deadline-time.monotonic())))
        except subprocess.TimeoutExpired as error:
            attempts.append(dict(query_error='timeout',stdout_bytes=len(error.stdout or b''),
                                 stderr=sanitized((error.stderr or b'').decode('utf-8',errors='replace'))[:2048]))
            break
        raw=result.stdout
        attempt=dict(returncode=result.returncode,stdout_bytes=len(raw),stderr=sanitized(result.stderr.decode('utf-8',errors='replace'))[:2048])
        if result.returncode or len(raw)>1024*1024:
            attempt['query_error']='nonzero_exit' if result.returncode else 'evidence_bound'
            attempts.append(attempt)
            break
        events=json.loads(raw)
        if not isinstance(events,list):raise RuntimeError('Native installer log was not an event array')
        matched,receipts=native_log_matches(events,pids,installed,expected)
        for receipt in receipts:receipt['catalog_path']=sanitized(receipt['catalog_path'])
        accepted_ids={id(entry) for entry in matched}
        unmatched_images=set();image_paths=set();unaccepted_messages=[]
        for entry in events:
            image=entry.get('processImagePath','')
            if image:image_paths.add(sanitized(image))
            if id(entry) not in accepted_ids:
                unmatched_images.add(Path(image).name[:128] if image else '<missing>')
                pid=native_log_pid(entry,pids)
                if pid is not None and image and Path(image).name=='Autoupdate':
                    unaccepted_messages.append(dict(diagnostic_only=True,accepted=False,
                        process_identifier=pid,process_image_path=sanitized(image),
                        message=sanitized(entry.get('eventMessage',''))))
        attempt.update(event_count=len(events),matched_event_count=len(matched),
                       process_image_paths=sorted(image_paths)[:16],unmatched_image_names=sorted(unmatched_images)[:16])
        attempts.append(attempt)
        messages=[entry.get('eventMessage','') for entry in matched]
        finalize,permission=native_log_errors(matched)
        if (finalize and permission) or time.monotonic()>=deadline:break
        time.sleep(min(.5,max(0,deadline-time.monotonic())))
    return dict(finalize_failure_observed=finalize,permission_failure_observed=permission,
                expected_process_image=sanitized(expected),queried_pids=pids,attempts=attempts,catalog_checks=receipts,
                logging_disabled=environment().get('OS_ACTIVITY_MODE')=='disable',
                unaccepted_autoupdate_messages=unaccepted_messages[-32:],
                messages=[sanitized(text) for text in messages[-32:]])


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-prefix',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--production-trust',action='store_true',help='Retain committed public trust and use separately signed inputs')
    parser.add_argument('--prepare',action='store_true',help='Build unsigned production fixtures without installing or signing')
    args=parser.parse_args()
    if (sys.platform!='darwin' or platform.machine()!='arm64' or os.environ.get('CI')!='true' or
        os.environ.get('GITHUB_ACTIONS')!='true' or os.environ.get('RUNNER_ENVIRONMENT')!='github-hosted'):
        parser.error('Requires an explicitly disposable GitHub-hosted Apple Silicon runner')
    output,qt=args.output.resolve(),args.qt_prefix.resolve()
    if args.prepare and not args.production_trust:parser.error('--prepare requires --production-trust')
    if args.production_trust:require_no_production_secrets()
    executing_prepared=args.production_trust and not args.prepare
    if (not output.is_relative_to((ROOT/'build').resolve()) or output==(ROOT/'build').resolve() or
            (not executing_prepared and output.exists()) or (executing_prepared and not output.is_dir())):
        parser.error('Use fresh output beneath build/ for preparation, then reuse that exact prepared output')
    prepared_record=products=manifest=archive=None
    if executing_prepared:
        prepared_record,products,manifest,archive=load_production_prepared(output)
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
    if not executing_prepared:output.mkdir(parents=True)
    token=prepared_record['token'] if executing_prepared else uuid.uuid4().hex
    name=prepared_record['app_name'] if executing_prepared else 'xdzip-owned-update-更新-'+token+'.app'
    applications.mkdir(exist_ok=True)
    installed=applications/name; require_owned(installed,applications)
    session=output/'session'
    if not executing_prepared:session.mkdir()
    elif session.is_symlink() or not session.is_dir() or any(session.iterdir()):
        parser.error('Prepared session must be an unused owned directory')
    associations=Associations()
    if executing_prepared:
        before_preparation=json.loads(production_file(output,'association-before-preparation.json').read_bytes())
    else:
        before_preparation=associations.snapshot()
        dump(output/'association-before-preparation.json',before_preparation)
    production=public_resource_hashes()
    if not executing_prepared:
        products,signing,sdk=prepare(output,qt,session,name,args.production_trust)
        if args.prepare:
            production_prepared(output,products,signing)
            print('Unsigned production-trust Mac fixtures prepared; independent protected signing is required')
            return
    # Explicitly register the owned, formal Alternate product during preparation.
    # Restore its defined pre-mutation Viewer, not an invented missing handler.
    run([LSREGISTER,'-f',products['old']['app']])
    original=associations.snapshot()
    dump(output/'association-original.json',original)
    if not original['2']:raise RuntimeError('Cannot safely restore a missing pre-mutation ZIP Viewer handler')
    if not executing_prepared:manifest,archive=signed_candidate(output,products['new'],signing,sdk)
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
    dump(session/'configuration.json',dict(marker='xdzip-owned-mac-product-update-v1',install_root=str(installed),scenario='interrupted-handoff',
         fixed_asset_url=base+archive.name,fixed_appcast_url=base+'appcast.xml',loopback_appcast_url=address+'/owned/appcast.xml',
         loopback_asset_url=address+'/owned/'+archive.name,routes={base+'release.json':'release.json',base+'release.json.sig':'release.json.sig',
         'https://api.github.com/repos/xiaodaozhi/xdzip/releases?per_page=100&page=1':'releases.json'}))
    password_fixture=ROOT/'tests/fixtures/extract/password-retry.zip'
    if digest(password_fixture)!='cdb1a34b259be86e1670c58d26fb328e3e2dfe6b71defaa6f333393bb4156d41':
        raise RuntimeError('Owned AES regression sample changed')
    shutil.copyfile(password_fixture,session/'password-retry.zip')
    other=None;old=None;new=None;log=None;record={'status':'failed'};stage='association_setup';evidence_failures=[]
    fault_flags_restored=False
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
        stage='interrupted_native_handoff'
        dump(session/'upgrade.json',{'owned':True})
        interrupted=await_json(session/'interrupted-handoff.json',timeout=240,session=session,
                               expected_recovery_error='handoff')
        if old.wait(timeout=40)!=0:raise RuntimeError('Interrupted native handoff did not quit normally')
        interrupted_quit=await_json(session/'200-normal-quit.json',session=session,expected_recovery_error='handoff')
        interrupted_barrier=await_json(session/'200-exit-barrier.json',session=session,expected_recovery_error='handoff')
        if (interrupted['state']!='recovery_required' or interrupted['error']!='handoff' or
            not interrupted['pending_present'] or interrupted_barrier['active_tasks']!=0 or
            interrupted_barrier['pending'].get('build')!=201 or
            not interrupted_quit['all_rust_joined'] or interrupted_quit['quit_signals']!=1 or
            not interrupted_quit['native_exit']['installer_seen_at_ready'] or
            not interrupted_quit['native_exit']['installer_gone_before_exit'] or
            interrupted_quit['native_exit']['committed'] or
            interrupted_quit['native_exit']['installer_observation_error']):
            raise RuntimeError('Interrupted real handoff did not preserve its receipt and settle the helper')
        interruption_record=dict(scope='Owned cancellation at the real join/commit boundary, not an OS disk replacement failure',
                                 state=interrupted,barrier=interrupted_barrier,quit=interrupted_quit)
        dump(output/'interrupted-handoff-verification.json',interruption_record)
        def change_scenario(name):
            # Only generated records in this fresh, owned session are cleared.
            leaves={'stop.json','upgrade.json','fixture-error.json','interrupted-handoff.json',
                    'password-waiting.json','waiting-cancelled.json','password-completed.json','quit-returned.json',
                    'asset-redirect.json','feed-redirect.json','replacement-fault.json','quit-progress.json',
                    'quit-first-intent.json'}
            for path in session.glob('*.json'):
                if path.name in leaves or path.name.startswith(('200-','201-','native-')):path.unlink()
            config=json.loads((session/'configuration.json').read_bytes())
            config.update(scenario=name,reuse_profile=True)
            dump(session/'configuration.json',config)
        stage='ordinary_old_recovery'
        change_scenario('recovery')
        old=subprocess.Popen([str(executable)],env=launch_env,stdout=log,stderr=subprocess.STDOUT)
        recovered=await_json(session/'200-started.json',process=old,session=session,
                             expected_recovery_error='installation_interrupted')
        if (recovered['state']!='recovery_required' or recovered['error']!='installation_interrupted' or
            recovered['pending_present'] or recovered['unknown_key']!=started['unknown_key'] or
            recovered['settings_path']!=started['settings_path'] or recovered['install_root']!=str(installed)):
            raise RuntimeError('Ordinary old startup did not consume the interrupted native attempt as recovery')
        dump(session/'stop.json',{'owned':True})
        if old.wait(timeout=20)!=0:raise RuntimeError('Recovery product did not exit normally')
        dump(output/'ordinary-recovery-verification.json',recovered)
        stage='committed_native_replacement_failure'
        change_scenario('immutable-replacement')
        old=subprocess.Popen([str(executable)],env=launch_env,stdout=log,stderr=subprocess.STDOUT)
        await_json(session/'200-started.json',process=old,session=session)
        dump(session/'upgrade.json',{'owned':True})
        native_commit=await_json(session/'200-native-commit.json',timeout=180,session=session)
        if old.wait(timeout=30)!=0:raise RuntimeError('Native committed host did not exit normally')
        failed_barrier=await_json(session/'200-exit-barrier.json',session=session)
        fault=await_json(session/'replacement-fault.json',session=session)
        ready=failed_barrier['native_exit'];pids=ready['installer_pids_at_ready']
        if (not fault['applied'] or not native_commit['committed_native_quit'] or native_commit['active_tasks']!=0 or
            not ready['installer_seen_at_ready'] or ready['installer_observation_error'] or
            failed_barrier['pending'].get('build')!=201):
            raise RuntimeError('Owned immutable fault did not reach actual native commit')
        helper_observation=wait_owned_helpers(pids,installed)
        terminal=dict(helper_observation=helper_observation)
        try:
            unchanged=json.loads(run([executable,'--version-json']))
            current=installed.lstat()
            terminal.update(identity=unchanged,old_identity_unchanged=unchanged==products['old']['identity'],
                            device=str(current.st_dev),inode=str(current.st_ino),flags=current.st_flags,
                            fault_inode_preserved=current.st_dev==int(fault['device']) and current.st_ino==int(fault['inode']))
        except Exception as error:
            terminal.update(identity_error=type(error).__name__,
                            identity_detail=str(error).replace(str(Path.home()),'$HOME').replace(str(ROOT),'$WORKSPACE')[:2048])
            dump(output/'replacement-terminal.json',terminal)
            raise
        dump(output/'replacement-terminal.json',terminal)
        if unchanged!=products['old']['identity']:raise RuntimeError('Failed replacement changed the old product')
        if not terminal['fault_inode_preserved'] or current.st_flags!=int(fault['flags_after']):
            raise RuntimeError('Committed native fault no longer belongs to the exact immutable inode')
        native_error=native_replacement_error(pids,installed)
        dump(output/'native-replacement-error.json',native_error)
        restored=restore_owned_flags(installed,applications,fault)
        fault_flags_restored=True
        dump(output/'replacement-flags-restored.json',restored)
        native_error_observed=native_error['finalize_failure_observed'] and native_error['permission_failure_observed']
        if not native_error_observed:
            # Preserve the failed evidence gate while collecting later, independent
            # recovery and task cases. An incomplete native error can never pass CI.
            evidence_failures.append(dict(stage=stage,error='native_replacement_error_evidence_incomplete'))
        replacement_failure=dict(status='passed' if native_error_observed else 'evidence_incomplete',
                                 scope='Committed Sparkle handoff with an owned immutable inode; actual final-replacement permission error required',
                                 fault=fault,commit=native_commit,barrier=failed_barrier,error=native_error,
                                 helper_observation=helper_observation,
                                 helpers_exited_without_intervention=True,old_identity=unchanged,original_flags_restored=True)
        dump(output/'replacement-failure-verification.json',replacement_failure)
        stage='ordinary_recovery_after_native_failure'
        change_scenario('recovery')
        old=subprocess.Popen([str(executable)],env=launch_env,stdout=log,stderr=subprocess.STDOUT)
        failed_recovery=await_json(session/'200-started.json',process=old,session=session,
                                   expected_recovery_error='installation_interrupted')
        if (failed_recovery['state']!='recovery_required' or failed_recovery['error']!='installation_interrupted' or
            failed_recovery['pending_present'] or failed_recovery['build']!=200 or
            failed_recovery['unknown_key']!=started['unknown_key']):
            raise RuntimeError('Ordinary startup did not report the real failed native replacement')
        dump(output/'replacement-failure-recovery.json',failed_recovery)
        dump(session/'stop.json',{'owned':True})
        if old.wait(timeout=20)!=0:raise RuntimeError('Failed-replacement recovery did not exit normally')
        stage='ordinary_quit_during_update_wait'
        change_scenario('quit-wait')
        old=subprocess.Popen([str(executable)],env=launch_env,stdout=log,stderr=subprocess.STDOUT)
        await_json(session/'200-started.json',process=old,session=session)
        dump(session/'upgrade.json',{'owned':True})
        quit_record=await_json(session/'200-normal-quit.json',timeout=180,session=session,allow_closing=True)
        if old.wait(timeout=20)!=0:raise RuntimeError('Real task quit did not complete normally')
        joined=await_json(session/'200-join.json',session=session,allow_closing=True)
        returned=json.loads((session/'quit-returned.json').read_bytes())
        quit_waiting=json.loads((session/'password-waiting.json').read_bytes())
        quit_cancelled=json.loads((session/'waiting-cancelled.json').read_bytes())
        first_intent=json.loads((session/'quit-first-intent.json').read_bytes())
        if (quit_record['quit_intent']!='QEvent::Quit through AppQuitGuard and DesktopIntegration' or
            not quit_record['return_clicked'] or not quit_record['cancel_quit_clicked'] or
            quit_record['barriers']!=0 or not quit_record['all_rust_joined'] or quit_record['quit_signals']!=1 or
            joined['active_tasks']!=0 or joined['update_pending'] or not joined['password_prompt_cleared'] or
            joined['task'].get('status')!='cancelled' or not joined['task'].get('terminal') or
            not all(first_intent.get(key) is True for key in ('password_visible','password_exposed','password_is_modal','password_owned')) or
            first_intent.get('active_tasks')!=1 or first_intent.get('state')!='waiting_tasks' or
            first_intent.get('task_id')!=quit_waiting['task_id'] or first_intent.get('prompt_id')!=quit_waiting['prompt_id'] or
            returned['state']!='waiting_tasks' or returned['active_tasks']!=1 or
            quit_waiting['session_count']!=2 or quit_waiting['idle_session']==quit_waiting['working_session'] or
            not returned.get('password_modal_restored') or returned['task_id']!=quit_waiting['task_id'] or returned['prompt_id']!=quit_waiting['prompt_id'] or
            quit_cancelled['task_id']!=quit_waiting['task_id'] or quit_cancelled['state']!='available' or
            (session/'native-start.json').exists() or (session/'200-exit-barrier.json').exists()):
            raise RuntimeError('Ordinary Return/Cancel quit did not preserve then cancel/join the real waiting task')
        ordinary_quit_record=dict(quit=quit_record,joined=joined,returned=returned,waiting=quit_waiting,
                                  cancelled=quit_cancelled,first_intent=first_intent)
        shutil.copyfile(session/'quit-first-intent.json',output/'quit-first-intent.json')
        dump(output/'ordinary-quit-verification.json',ordinary_quit_record)
        stage='busy_upgrade_start'
        change_scenario('busy-upgrade')
        old=subprocess.Popen([str(executable)],env=launch_env,stdout=log,stderr=subprocess.STDOUT)
        started=await_json(session/'200-started.json',process=old,session=session)
        peer=subprocess.Popen([str(executable)],env=launch_env,stdout=log,stderr=subprocess.STDOUT)
        if peer.wait(timeout=20)!=0:raise RuntimeError('Busy scenario SingleInstance startup failed')
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
        password_waiting=await_json(session/'password-waiting.json',session=session)
        waiting_cancelled=await_json(session/'waiting-cancelled.json',session=session)
        password_completed=await_json(session/'password-completed.json',session=session)
        if (password_waiting['state']!='waiting_tasks' or password_waiting['active_tasks']!=1 or
            password_waiting['session_count']!=2 or password_waiting['idle_session']==password_waiting['working_session'] or
            waiting_cancelled['state']!='available' or waiting_cancelled['active_tasks']!=1 or
            waiting_cancelled['task_id']!=password_waiting['task_id'] or waiting_cancelled['prompt_id']!=password_waiting['prompt_id'] or
            password_completed['task'].get('status')!='succeeded' or
            password_completed['task'].get('id')!=password_waiting['task_id'] or
            password_completed['task'].get('verifiedFiles')!='1' or password_completed['task'].get('verifiedBytes')!='18'):
            raise RuntimeError('Real AES update wait/cancel/IME resume did not preserve and verify the same task')
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
        record=dict(status='failed' if evidence_failures else 'passed',evidence_failures=evidence_failures,
                    quit_intent_schema_version=1,
                    scope='Real browser_main/About/UpdateController, formal installed identity, native Sparkle, ordinary receipt restoration and default profile',
                    production_trust_modified=False,production_signatures=args.production_trust,
                    production_trust=production,prepared_source_commit=prepared_record['source_commit'] if prepared_record else None,
                    embedded_sparkle_public_keys={label:product['SUPublicEDKey'] for label,product in prepared_record['products'].items()} if prepared_record else None,
                    old=started,new=new,exit_barrier=barrier,
                    settings_preserved=True,unknown_setting_preserved=True,single_instance_before_and_after=True,
                    zip_association_scope='Owned LaunchServices public.zip-archive Viewer default for ordinary ZIP opening; Editor/Shell checked read-only, all-roles aggregate informational only',
                    before_preparation_zip_handlers=before_preparation,
                    preparation_registration='Explicit lsregister of the owned prepared old product with formal Alternate document handlers',
                    restoration_scope='Pre-mutation handlers after private product preparation; preparation-time LaunchServices discovery is recorded, not reversed',
                    original_zip_handlers=original,owned_zip_handlers=baseline,after_zip_handlers=after_associations,
                    network_seam='Private Qt metadata transport and exact feed URL mapping; public Sparkle download-request delegate maps only the matched authenticated GitHub asset request, preserving item URL/signature/length',
                    archive_transport=redirect,
                    interrupted_handoff=interruption_record,ordinary_recovery=recovered,ordinary_quit=ordinary_quit_record,
                    native_replacement_failure=replacement_failure,native_failure_recovery=failed_recovery,
                    password_waiting=password_waiting,waiting_cancelled=waiting_cancelled,password_completed=password_completed,
                    limits=['Busy install request/cancel use controller calls while the real password window is application-modal; the final install uses About input.',
                            'The post-commit failure uses one owned UF_IMMUTABLE inode; system authorization, other OS failures and real-user desktop acceptance remain separate.',
                            'Default handlers for other archive UTIs and physical user acceptance are not claimed.'])
    except Exception as error:
        record=dict(status='failed',stage=stage,error=type(error).__name__,detail=str(error),evidence_failures=evidence_failures)
        if stage=='ordinary_quit_during_update_wait' and (session/'quit-progress.json').is_file():
            dump(output/'quit-wait-failure.json',json.loads((session/'quit-progress.json').read_bytes()))
        raise
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
            fault_path=session/'replacement-fault.json'
            if not fault_flags_restored and fault_path.is_file():
                own_fault=json.loads(fault_path.read_bytes())
                if own_fault.get('changed'):
                    restored=restore_owned_flags(installed,applications,own_fault)
                    fault_flags_restored=True
                    dump(output/'replacement-flags-restored.json',restored)
        except Exception as error:cleanup.append(dict(stage='restore_owned_inode_flags',error=type(error).__name__,detail=str(error)))
        if fault_flags_restored:record['owned_inode_flags_restored']=True
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
    if evidence_failures:raise RuntimeError('Required native replacement error evidence incomplete; independent phases recorded')
    print('Real Mac product signed update, ordinary restart, profile and ZIP handlers passed')


if __name__=='__main__':main()
