# scripts/qt_update_fixture.py

"""Build two real private Qt products, retaining production verification and handoff."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
from update_fixture import ROOT, KEYRING, run, dump, digest, manifest, qa_installer_text
from deployment_audit import native


def load(path, name):
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def prepare(output, sdk, session):
    if os.environ.get('XDZIP_ISOLATED_UPDATE_FIXTURE')!='1':
        raise RuntimeError('Private Qt fixture requires explicit isolated regression build')
    output=output.resolve();sdk=sdk.resolve()
    output.mkdir(parents=True,exist_ok=False)
    source=Path(tempfile.mkdtemp(prefix='xdzip-qt-update-' if sys.platform=='linux' else 'qtu-',
                                 dir=None if sys.platform=='linux' else ROOT/'build'))
    archive=output/'owned-source.tar'
    with archive.open('wb') as stream:
        run(['git','-C',ROOT,'archive','--format=tar','HEAD'],stdout=stream)
    with tarfile.open(archive) as stream:
        stream.extractall(source,filter='data')
    archive.unlink()
    # Also allow local compile validation before these exact test-only files have
    # been committed. Dedicated CI otherwise uses the immutable checked-out SHA.
    for relative in ('apps/desktop/CMakeLists.txt','apps/desktop/browser_main.cpp','apps/desktop/UpdateController.cpp',
                     'apps/desktop/tests/native_update_fixture.cpp','apps/desktop/tests/native_update_fixture.h'):
        shutil.copy2(ROOT/relative,source/relative)
        (source/relative).touch()
    shutil.copy2(ROOT/'crates/xdzip-update-helper/src/linux.rs',source/'crates/xdzip-update-helper/src/linux.rs')
    (source/'crates/xdzip-update-helper/src/linux.rs').touch()
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
    results={}
    with (output/'prepare.log').open('w',encoding='utf-8') as log:
        def checked(argv,**kwargs):
            log.write('Running '+str(argv[0])+'\n');log.flush()
            return run(argv,env=env,stdout=log,stderr=subprocess.STDOUT,**kwargs)
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
        # Build the shared, Qt-free signer in this private committed source tree.
        checked([cargo,'build','--locked','--manifest-path',source/'Cargo.toml','--target-dir',source/'build/signer',
                 '-p','xdzip-update','--bin','xdzip-update-tool'])
        signer=source/('build/signer/debug/xdzip-update-tool'+('.exe' if sys.platform=='win32' else ''))
        new=output/'new'/results['new']['package']
        manifest(new,'windows' if sys.platform=='win32' else 'linux',signer,results['new']['identity'])
    value={'schema_version':1,'scope':'Private real browser_main/QML builds with public fixture trust; not product packages',
           'production_trust_modified':False,'session_directory':str(session),**results}
    dump(output/'prepared.json',value)
    password_fixture=ROOT/'tests/fixtures/extract/password-retry.zip'
    assert digest(password_fixture)=='cdb1a34b259be86e1670c58d26fb328e3e2dfe6b71defaa6f333393bb4156d41'
    shutil.copyfile(password_fixture,output/'password-retry.zip')
    return value


def feed(prepared, output, session):
    session.mkdir(parents=True,exist_ok=False)
    new=output/'new'/prepared['new']['package']
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
    if prepared.get('password_task'):
        shutil.copyfile(output/'password-retry.zip',session/'password-retry.zip')
        dump(session/'password-scenario.json',{'owned':True})


def verify_restart(record, prepared):
    expected=prepared['new']['identity']
    for key in ('version','build','revision'):
        assert record[key]==expected[key],record
    assert record['state']=='updated' and not record['error'],record
    assert record['primary_windows']==1,record
    assert record['theme']=='dark' and record['text_scale']==1.25 and record['density']=='compact',record
    assert record['language']=='en-US' and record['active_language']=='en-US',record
