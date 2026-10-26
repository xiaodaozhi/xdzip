# scripts/qt_update_fixture.py

"""Build two real private Qt products, retaining production verification and handoff."""
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from update_fixture import ROOT, KEYRING, run, dump, digest, manifest, qa_installer_text
from deployment_audit import native


def require_no_signing_authority():
    if any(name in os.environ for name in
           ('XDZIP_UPDATE_SIGNING_KEY_HEX','XDZIP_SPARKLE_SIGNING_KEY_HEX')):
        raise RuntimeError('Product preparation and execution must not receive signing secrets')


def production_trust_check(output, prepared):
    """Verify frozen inputs and real signatures before starting any installer."""
    require_no_signing_authority()
    path=output/'production-update-prepared.json'
    record=json.loads(path.read_text(encoding='utf-8'))
    expected={'schema_version','platform','source_commit','tool','artifacts','products'}
    if set(record)!=expected or record['schema_version']!=1:
        raise RuntimeError('Invalid production update preparation record')
    if (not prepared.get('production_trust') or record['platform']!=('windows' if sys.platform=='win32' else 'linux') or
            re.fullmatch('[0-9a-f]{40}',record['source_commit']) is None or
            os.environ.get('GITHUB_SHA')!=record['source_commit'] or prepared['source_commit']!=record['source_commit']):
        raise RuntimeError('Production update source or mode does not match this workflow')
    if record['products']!=prepared['production_products']:
        raise RuntimeError('Production product identities changed after preparation')
    suffix='.exe' if sys.platform=='win32' else ''
    package_suffix='windows-x86_64-setup.exe' if sys.platform=='win32' else 'ubuntu24.04-x86_64.deb'
    required={'signing/xdzip-update-tool'+suffix,'resources/update-trust.json','resources/sparkle-update-key.json',
              'new/release.json','failure/release.json'}
    if set(record['products'])!={'old','new','failure'}:
        raise RuntimeError('Production update products are incomplete')
    for label,product in record['products'].items():
        number=1 if label=='old' else 2
        version='0.2.0-alpha.'+str(number)
        package='xdzip-'+version+'-'+package_suffix
        expected_identity={'version':version,'build':199+number,'revision':product['commit'][:12],'dirty':False}
        if (re.fullmatch('[0-9a-f]{40}',product['commit']) is None or product['identity']!=expected_identity or
                prepared[label]['package']!=package):
            raise RuntimeError('Production fixture identity or package changed')
        required.add(label+'/'+package)
    if set(record['artifacts'])!=required:
        raise RuntimeError('Production update preparation does not freeze every required input')
    if record['products']['new']['commit']!=record['products']['failure']['commit']:
        raise RuntimeError('Failure installer no longer contains the same new product')
    if record['tool']!={'path':'signing/xdzip-update-tool'+suffix,
                        'sha256':record['artifacts']['signing/xdzip-update-tool'+suffix]['sha256']}:
        raise RuntimeError('Unexpected production signature verifier')
    for relative, expected_file in record['artifacts'].items():
        candidate=output/relative
        if (Path(relative).is_absolute() or '..' in Path(relative).parts or
                candidate.is_symlink() or not candidate.is_file() or
                not candidate.resolve().is_relative_to(output.resolve()) or
                candidate.stat().st_size!=expected_file['bytes'] or digest(candidate)!=expected_file['sha256']):
            raise RuntimeError('Production update input changed after preparation')
    for name in ('update-trust.json','sparkle-update-key.json'):
        if digest(output/'resources'/name)!=digest(ROOT/'resources'/name):
            raise RuntimeError('Production public resource does not match checked-out trust')
    tool=output/record['tool']['path']
    if digest(tool)!=record['tool']['sha256']:
        raise RuntimeError('Production signature verifier changed')
    signatures=[]
    for label in ('new','failure'):
        directory=output/label
        release=json.loads((directory/'release.json').read_text(encoding='utf-8'))
        identity=record['products'][label]
        if any(release[key]!=value for key,value in
               {'product':'xdzip','repository':'xiaodaozhi/xdzip','version':identity['identity']['version'],
                'build':identity['identity']['build'],'commit':identity['commit']}.items()):
            raise RuntimeError('Signed candidate differs from the frozen product identity')
        selected=[asset for asset in release['assets'] if asset['platform']==record['platform'] and
                  asset['package_type']==('exe' if sys.platform=='win32' else 'deb')]
        package=directory/prepared[label]['package']
        if (len(selected)!=1 or selected[0]['filename']!=package.name or
                selected[0]['bytes']!=package.stat().st_size or selected[0]['sha256']!=digest(package)):
            raise RuntimeError('Signed candidate does not describe the frozen installer')
        signature=directory/'release.json.sig'
        if signature.is_symlink() or not signature.is_file() or signature.stat().st_size>4096:
            raise RuntimeError('A protected-step production signature is required')
        run([tool,'verify','--manifest',directory/'release.json','--keyring',ROOT/'resources/update-trust.json',
             '--signature',signature],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        signatures.append({'manifest':label+'/release.json','signature_sha256':digest(signature)})
    result={'schema_version':1,'status':'passed','source_commit':record['source_commit'],
            'platform':record['platform'],'production_trust':True,
            'public_resources':{name:digest(ROOT/'resources'/name) for name in
                                ('update-trust.json','sparkle-update-key.json')},
            'signing_secrets_received':False,'signatures':signatures}
    dump(output/'production-trust-verification.json',result)
    return result


def load(path, name):
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def windows_copy_pause(text, payload, session):
    """Pause the owned new installer after its EXE, before the remaining Qt payload."""
    original='Source: "'+str(payload)+'\\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs'
    if text.count(original)!=1:
        raise RuntimeError('Owned installer no longer has the expected root payload entry')
    replacement=('Source: "'+str(payload)+'\\xdzip.exe"; DestDir: "{app}"; Flags: ignoreversion; AfterInstall: OwnedPauseAfterExe\n'
                 +original+'; Excludes: "xdzip.exe"; BeforeInstall: OwnedRemainingFiles')
    def literal(value):return "'"+str(value).replace("'","''")+"'"
    return text.replace(original,replacement,1)+'''
function OwnedInstallerPid(): LongWord;
external 'GetCurrentProcessId@kernel32.dll stdcall';

procedure OwnedPauseAfterExe();
var I: Integer;
begin
  { This private entry has replaced xdzip.exe; no remaining payload entry ran. }
  if not SaveStringToFile('''+literal(session/'installer-copy-paused.json.tmp')+''',
    '{"schema_version":1,"stage":"after_xdzip_before_remaining_files","qt_sdk":"6.8.2","installer_pid":' +
    IntToStr(OwnedInstallerPid()) + '}', False) then
    RaiseException('Owned installer pause marker could not be written');
  if not RenameFile('''+literal(session/'installer-copy-paused.json.tmp')+''',
    '''+literal(session/'installer-copy-paused.json')+''') then
    RaiseException('Owned installer pause marker could not be published');
  for I := 1 to 1200 do begin
    if FileExists('''+literal(session/'installer-copy-release.json')+''') then exit;
    Sleep(50);
  end;
  RaiseException('Owned installer copy pause timed out');
end;

procedure OwnedRemainingFiles();
begin
  if not FileExists('''+literal(session/'installer-copy-resumed.json')+''') then
    if not SaveStringToFile('''+literal(session/'installer-copy-resumed.json')+''',
      '{"stage":"remaining_payload_started"}', False) then
      RaiseException('Owned remaining-copy marker could not be written');
end;
'''


def prepare(output, sdk, session, *, production_trust=False):
    if os.environ.get('XDZIP_ISOLATED_UPDATE_FIXTURE')!='1':
        raise RuntimeError('Private Qt fixture requires explicit isolated regression build')
    if production_trust:
        require_no_signing_authority()
    source_commit=subprocess.check_output(['git','-c','safe.directory='+str(ROOT),'-C',ROOT,'rev-parse','HEAD'],text=True).strip()
    if production_trust and subprocess.check_output(
            ['git','-c','safe.directory='+str(ROOT),'-C',ROOT,'status','--porcelain','--untracked-files=no'],text=True).strip():
        raise RuntimeError('Production trust preparation requires clean tracked source')
    output=output.resolve();sdk=sdk.resolve()
    output.mkdir(parents=True,exist_ok=False)
    source=Path(tempfile.mkdtemp(prefix='xdzip-qt-update-' if sys.platform=='linux' else 'qtu-',
                                 dir=None if sys.platform=='linux' else ROOT/'build'))
    archive=output/'owned-source.tar'
    with archive.open('wb') as stream:
        # The known read-only checkout is owned by the runner, while the SDK
        # container builds as root. Trust this exact source for this read only.
        run(['git','-c','safe.directory='+str(ROOT),'-C',ROOT,'archive','--format=tar','HEAD'],stdout=stream)
    with tarfile.open(archive) as stream:
        stream.extractall(source,filter='data')
    archive.unlink()
    # Also allow local compile validation before these exact test-only files have
    # been committed. Dedicated CI otherwise uses the immutable checked-out SHA.
    for relative in ('apps/desktop/CMakeLists.txt','apps/desktop/browser_main.cpp','apps/desktop/UpdateController.cpp',
                     'apps/desktop/DesktopIntegration.cpp',
                     'apps/desktop/tests/native_update_fixture.cpp','apps/desktop/tests/native_update_fixture.h'):
        shutil.copy2(ROOT/relative,source/relative)
        (source/relative).touch()
    shutil.copy2(ROOT/'crates/xdzip-update-helper/src/linux.rs',source/'crates/xdzip-update-helper/src/linux.rs')
    (source/'crates/xdzip-update-helper/src/linux.rs').touch()
    if production_trust:
        for name in ('update-trust.json','sparkle-update-key.json'):
            if digest(source/'resources'/name)!=digest(ROOT/'resources'/name):
                raise RuntimeError('Archived production trust differs from checked-out source')
    else:
        dump(source/'resources/update-trust.json',KEYRING)
    run(['git','-C',source,'init','--quiet'])
    run(['git','-C',source,'config','user.name','xdzip owned update fixture'])
    run(['git','-C',source,'config','user.email','fixture@xdzip.invalid'])
    run(['git','-C',source,'config','core.hooksPath',str(source/'.git/no-hooks')])
    build=source/'build/desktop'
    env=os.environ.copy()
    # Cargo metadata paths are UTF-8 even under a legacy Windows console locale.
    env['PYTHONUTF8']='1'
    cargo=shutil.which('cargo');cmake=shutil.which('cmake')
    if not cargo or not cmake:
        raise RuntimeError('Private Qt build requires Cargo and CMake on the build environment PATH')
    for name in ('XDZIP_UPDATE_SIGNING_KEY_HEX','XDZIP_SPARKLE_SIGNING_KEY_HEX','CARGO_ENCODED_RUSTFLAGS','RUSTFLAGS'):
        env.pop(name,None)
    qt_version=subprocess.check_output([str(sdk/('bin/qmake.exe' if sys.platform=='win32' else 'bin/qmake')),
                                        '-query','QT_VERSION'],env=env,text=True).strip()
    if qt_version!='6.8.2':
        raise RuntimeError('The native update regression requires the declared Qt SDK 6.8.2')
    results={}
    with (output/'prepare.log').open('w',encoding='utf-8') as log:
        def checked(argv,*,child_env=None,**kwargs):
            log.write('Running '+str(argv[0])+'\n');log.flush()
            return run(argv,env=env if child_env is None else child_env,stdout=log,stderr=subprocess.STDOUT,**kwargs)
        # The native C-helper test need not fetch the archive/Qt bridge crates.
        # Warm the complete locked workspace while this build phase has network;
        # the later SDK-free runtime remains offline.
        checked([cargo,'fetch','--locked','--manifest-path',source/'Cargo.toml'])
        for number,build_number in ((1,200),(2,201)):
            version='0.2.0-alpha.'+str(number)
            dump(source/'VERSION.json',{'version':'0.2.0','channel':'alpha','build':build_number,'prerelease_number':number})
            checked([sys.executable,source/'scripts/sync-version.py'])
            with open(os.devnull,'w') as null:
                run([cargo,'metadata','--manifest-path',source/'Cargo.toml','--offline','--format-version','1'],env=env,stdout=null,stderr=log)
            checked(['git','-C',source,'add','--all'])
            checked(['git','-C',source,'-c','commit.gpgsign=false','commit','--allow-empty','--quiet','-m','Owned native update fixture '+version])
            commit=subprocess.check_output(['git','-C',str(source),'rev-parse','HEAD'],text=True).strip()
            checked([cmake,'-S',source,'-B',build,'-G','Ninja','-DCMAKE_BUILD_TYPE=Release',
                     '-DCMAKE_PREFIX_PATH='+str(sdk),'-DBUILD_TESTING=OFF','-DXDZIP_BUILD_QT_PROBE=OFF',
                     '-DXDZIP_BUILD_DESKTOP_PREVIEW=ON',
                     '-DXDZIP_NATIVE_UPDATE_FIXTURE=ON','-DXDZIP_NATIVE_UPDATE_FIXTURE_DIRECTORY='+str(session).replace('\\','/')])
            checked([cmake,'--build',build,'--target','xdzip-browser','--parallel','2'])
            label='old' if number==1 else 'new'
            folder=source/'build'/label;folder.mkdir()
            payload=folder/'payload'
            notices=folder/'notices'
            checked([sys.executable,source/'scripts/collect-notices.py',notices])
            if sys.platform=='win32':
                payload.mkdir()
                for name in ('xdzip.exe','xdzip-update-helper.exe'):
                    shutil.copy2(build/'apps/desktop'/name,payload/name)
                shutil.copytree(notices,payload/'share/notices')
                checked([sys.executable,source/'scripts/deploy_windows.py','--root',payload,'--qt-prefix',sdk,
                         '--qml-dir',source/'apps/desktop/ui','--qml-import',sdk/'qml',
                         '--runtime',ROOT/'build/windows-runtime/runtime'])
                app=payload/'xdzip.exe'
            else:
                checked([sys.executable,source/'scripts/deploy_linux.py','--app',build/'apps/desktop/xdzip',
                         '--qt-prefix',sdk,'--output',payload,'--notices',notices])
                # A local Windows bind-mounted Linux SDK exposes synthetic 0777
                # modes. Only normalize this owned private fixture payload to the
                # same modes produced by the native Ubuntu SDK in CI.
                for path in payload.rglob('*'):
                    path.chmod(0o755 if path.is_dir() or native(path,'linux') else 0o644)
                (payload/'xdzip').chmod(0o755)
                app=payload/'bin/xdzip'
            audit_path=folder/'audit.json'
            checked([sys.executable,source/'scripts/deployment_audit.py','--root',payload,'--app',app,
                     '--mode','deployed','--output',audit_path])
            audit=json.loads(audit_path.read_text())
            identity=audit['identity']
            assert identity=={'version':version,'build':build_number,'revision':commit[:12],'dirty':False},identity
            identity['commit']=commit
            asset=output/label;asset.mkdir()
            if sys.platform=='win32':
                builder=load(source/'scripts/build-windows-installer.py','qt_windows_builder')
                package=asset/('xdzip-'+version+'-windows-x86_64-setup.exe')
                text,_=builder.render(payload,audit,package,ROOT/'build/installer-toolchain/ChineseSimplified.isl')
                package.with_suffix('.files').write_text(builder.payload_manifest(audit),encoding='utf-8')
                if number==2:text=windows_copy_pause(text,payload,session)
                package.with_suffix('.iss').write_text(qa_installer_text(text),encoding='utf-8-sig')
                checked([ROOT/'build/installer-toolchain/compiler/ISCC.exe',package.with_suffix('.iss')])
            else:
                builder=load(source/'scripts/build-linux-installer.py','qt_linux_builder')
                stage=folder/'deb-stage'
                record=builder.package_tree(payload,audit,stage)
                package=asset/('xdzip-'+version+'-ubuntu24.04-x86_64.deb')
                checked(['dpkg-deb','-Zgzip','-z1','--root-owner-group','--build',stage,package])
                dump(asset/'deployment.json',json.loads((payload/'share/deployment.json').read_text()))
                identity['runtime_id']=record['runtime_id']
            dump(asset/'identity.json',identity)
            results[label]={'package':package.name,'identity':identity}
            if number==2:
                # The same real Qt payload enters an actual failing installer.
                # Only this private fixture changes installer control flow.
                failed=output/'failure';failed.mkdir()
                failed_package=failed/package.name
                if sys.platform=='win32':
                    failed_text,_=builder.render(payload,audit,failed_package,ROOT/'build/installer-toolchain/ChineseSimplified.isl')
                    failed_package.with_suffix('.files').write_text(builder.payload_manifest(audit),encoding='utf-8')
                    failed_package.with_suffix('.iss').write_text(qa_installer_text(failed_text,'startup-failure'),encoding='utf-8-sig')
                    checked([ROOT/'build/installer-toolchain/compiler/ISCC.exe',failed_package.with_suffix('.iss')])
                else:
                    failed_stage=folder/'failure-deb-stage'
                    builder.package_tree(payload,audit,failed_stage)
                    hook=failed_stage/'DEBIAN/postinst'
                    # Run the genuine postinst first, including clearing its
                    # launch gate. dpkg then records a real half-configured new
                    # payload, which ordinary startup must identify honestly.
                    with hook.open('a') as stream:
                        stream.write('\nif test "$1" = configure && ! test -f /run/xdzip/qt-postinst-failed; then\n'
                                     ' install -m 0644 /dev/null /run/xdzip/qt-postinst-failed\n exit 85\nfi\n')
                    checked(['dpkg-deb','-Zgzip','-z1','--root-owner-group','--build',failed_stage,failed_package])
                dump(failed/'identity.json',identity)
                results['failure']={'package':failed_package.name,'identity':identity,'error':'installation_failed'}
        # Build the shared, Qt-free signer in this private committed source tree.
        # It runs later without the compiler environment or installed product.
        # Only this final tool build changes CRT selection; product builds above
        # keep their deployment-audited app-local runtimes.
        signer_environment=env.copy()
        if sys.platform=='win32':
            signer_environment['RUSTFLAGS']='-C target-feature=+crt-static'
        checked([cargo,'build','--locked','--manifest-path',source/'Cargo.toml','--target-dir',source/'build/signer',
                 '-p','xdzip-update','--bin','xdzip-update-tool'],child_env=signer_environment)
        signer=source/('build/signer/debug/xdzip-update-tool'+('.exe' if sys.platform=='win32' else ''))
        new=output/'new'/results['new']['package']
        manifest(new,'windows' if sys.platform=='win32' else 'linux',signer,results['new']['identity'],sign=not production_trust)
        manifest(output/'failure'/results['failure']['package'],
                 'windows' if sys.platform=='win32' else 'linux',signer,results['failure']['identity'],sign=not production_trust)
    production_products={label:{'identity':{key:result['identity'][key] for key in ('version','build','revision','dirty')},
                                'commit':result['identity']['commit']} for label,result in results.items()}
    if production_trust:
        signing=output/'signing';signing.mkdir()
        copied=signing/signer.name;shutil.copy2(signer,copied)
        resources=output/'resources';resources.mkdir()
        for name in ('update-trust.json','sparkle-update-key.json'):
            shutil.copy2(source/'resources'/name,resources/name)
        frozen=[copied,resources/'update-trust.json',resources/'sparkle-update-key.json',
                output/'new/release.json',output/'failure/release.json',
                *(output/label/result['package'] for label,result in results.items())]
        artifacts={path.relative_to(output).as_posix():{'bytes':path.stat().st_size,'sha256':digest(path)} for path in frozen}
        dump(output/'production-update-prepared.json',{'schema_version':1,'platform':'windows' if sys.platform=='win32' else 'linux',
             'source_commit':source_commit,'tool':{'path':copied.relative_to(output).as_posix(),'sha256':digest(copied)},
             'artifacts':artifacts,'products':production_products})
    value={'schema_version':1,'scope':'Private real browser_main/QML builds with public fixture trust; not product packages',
           'production_trust_modified':False,'session_directory':str(session),
           'qt_sdk_version':qt_version,'windows_copy_pause':sys.platform=='win32',**results}
    if production_trust:
        value.update(scope='Private real browser_main/QML builds with unchanged production public trust; not released packages',
                     production_trust=True,source_commit=source_commit,production_products=production_products)
    dump(output/'prepared.json',value)
    password_fixture=ROOT/'tests/fixtures/extract/password-retry.zip'
    assert digest(password_fixture)=='cdb1a34b259be86e1670c58d26fb328e3e2dfe6b71defaa6f333393bb4156d41'
    shutil.copyfile(password_fixture,output/'password-retry.zip')
    return value


def feed(prepared, output, session, candidate='new', reset=False):
    session.mkdir(parents=True,exist_ok=reset)
    if reset:
        # This directory was created by this exact fixture. Preserve only its
        # explicit owned-profile marker between independent ordinary processes.
        for path in session.glob('*.json'):
            if path.name!='profile-initialized.json':path.unlink()
    new=output/candidate/prepared[candidate]['package']
    for path in (new,new.parent/'release.json',new.parent/'release.json.sig'):
        shutil.copy2(path,session/path.name)
    release=json.loads((session/'release.json').read_text())
    base='https://github.com/xiaodaozhi/xdzip/releases/download/'+release['tag']+'/'
    assets=[{'name':a['filename'],'browser_download_url':a['url'],'size':a['bytes']} for a in release['assets']]
    assets += [{'name':name,'browser_download_url':base+name} for name in ('release.json','release.json.sig')]
    dump(session/'releases.json',[{'tag_name':release['tag'],'draft':False,'prerelease':True,
                                  'html_url':'https://github.com/xiaodaozhi/xdzip/releases/tag/'+release['tag'],'assets':assets}])
    routes={base+name:name for name in ('release.json','release.json.sig',new.name)}
    routes['https://api.github.com/repos/xiaodaozhi/xdzip/releases?per_page=100&page=1']='releases.json'
    dump(session/'routes.json',routes)
    if prepared.get('password_task') and candidate=='new':
        shutil.copyfile(output/'password-retry.zip',session/'password-retry.zip')
        dump(session/'password-scenario.json',{'owned':True})
        dump(session/'normal-quit-scenario.json',{'owned':True})


def verify_restart(record, prepared):
    expected=prepared['new']['identity']
    for key in ('version','build','revision'):
        assert record[key]==expected[key],record
    assert record['state']=='updated' and not record['error'],record
    assert record['primary_windows']==1,record
    assert record['theme']=='dark' and record['text_scale']==1.25 and record['density']=='compact',record
    assert record['language']=='en-US' and record['active_language']=='en-US',record
