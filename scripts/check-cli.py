#!/usr/bin/env python3
"""Exercise real headless CLI commands against owned archive inputs."""
from __future__ import annotations
import argparse
import hashlib
import gzip
import json
import lzma
import os
from pathlib import Path
import signal
import shutil
import subprocess
import tarfile
import time
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def check_desktop_worker(binary, worker, work, environment, source):
    """Exercise fixed deployment lookup without the ordinary worker overrides."""
    root=work/'desktop-worker-layout';root.mkdir()
    mac=platform_name()=='macos'
    cli=root/('Owned.app/Contents/Helpers/cli' if mac else 'cli');cli.mkdir(parents=True)
    gui=root/'Owned.app/Contents/MacOS/xdzip' if mac else root/binary.name
    gui.parent.mkdir(parents=True,exist_ok=True);gui.write_bytes(b'owned desktop identity fixture')
    executable=cli/binary.name;shutil.copy2(binary,executable)
    shared=gui.with_name(worker.name) if mac or os.name=='nt' else root/'bin'/worker.name
    shared.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(worker,shared)
    marker=root/'Owned.app/Contents/Resources/xdzip-cli-context.json' if mac else cli/'xdzip-cli-context.json'
    marker.parent.mkdir(parents=True,exist_ok=True)
    record=dict(schema_version=1,kind='desktop',desktop_relative=(
        '../../MacOS/xdzip' if mac else '../'+binary.name),worker_relative=(
        '../../MacOS/'+worker.name if mac else '../'+worker.name if os.name=='nt' else '../bin/'+worker.name))
    marker.write_text(json.dumps(record))
    home=root/'home';home.mkdir(mode=0o700)
    env=dict(environment,HOME=str(home),LOCALAPPDATA=str(home),XDG_DATA_HOME=str(home))
    for key in ('XDZIP_NATIVE_CODEC','XDZIP_NATIVE_CODEC_WORKER'):env.pop(key,None)
    def command(*args,code=0):
        p=subprocess.run([str(executable),'--json',*map(str,args)],env=env,capture_output=True,timeout=30)
        assert p.returncode==code,(p.returncode,p.stdout[:1500],p.stderr[:1500])
        return json.loads(p.stdout)
    archive=root/'shared.7z'
    command('create',archive,source);command('test',archive)
    command('extract',archive,'-C',root/'extracted')
    assert (root/'extracted'/source.name).read_bytes()==source.read_bytes()
    original=record['worker_relative'];record['worker_relative']='../../foreign-worker'
    marker.write_text(json.dumps(record))
    assert command('list',archive,code=3)['error_code']=='invalid_installation'
    record['worker_relative']=original;marker.write_text(json.dumps(record))
    held=shared.with_name('held-worker');shared.rename(held)
    assert command('list',archive,code=3)['error_code']=='invalid_installation'
    held.rename(shared)
    marker.unlink()
    command('create',root/'missing-local.7z',source,code=1)
    assert not (root/'missing-local.7z').exists()
    shutil.copy2(shared,cli/worker.name)
    command('create',root/'standalone.7z',source);command('test',root/'standalone.7z')


def check(binary, worker, output, payload=None, install_fixture=False, default_worker=False):
    binary, worker, output = binary.resolve(), worker.resolve(), output.resolve()
    if not output.is_relative_to(ROOT/'build') or output == ROOT/'build':
        raise ValueError('CLI evidence must be under build/')
    output.mkdir(parents=True, exist_ok=True)
    work = output/('cases-'+str(os.getpid())+'-'+str(time.time_ns()))
    work.mkdir(mode=0o700)
    if os.name == 'nt':
        import csv
        sid = next(csv.reader([subprocess.check_output(['whoami','/user','/fo','csv','/nh'],text=True).strip()]))[1]
        subprocess.run(['icacls',str(work),'/inheritance:r','/grant:r','*'+sid+':(OI)(CI)F'],stdout=subprocess.DEVNULL,check=True)
    env = os.environ.copy()
    for key in ['DISPLAY','WAYLAND_DISPLAY','QT_PLUGIN_PATH','QT_QPA_PLATFORM_PLUGIN_PATH',
                'QML_IMPORT_PATH','QML2_IMPORT_PATH','LD_LIBRARY_PATH','DYLD_LIBRARY_PATH']:
        env.pop(key, None)
    for key in ('XDZIP_NATIVE_CODEC','XDZIP_NATIVE_CODEC_WORKER'):
        if default_worker:env.pop(key,None)
        else:env[key]=str(worker)
    results = []

    def command(*arguments, code=0, stdin=None, cwd=work):
        process = subprocess.run([str(binary), *map(str,arguments)], input=stdin,
                                 capture_output=True, cwd=cwd, env=env, timeout=60)
        if process.returncode != code:
            raise AssertionError(f'CLI {arguments[0]}: expected exit {code}, actual {process.returncode}, result '+process.stdout.decode('utf-8', errors='replace')[:1500])
        return process

    def invoke(*arguments, code=0, stdin=None):
        process = command('--json', *arguments, code=code, stdin=stdin)
        value = json.loads(process.stdout)
        assert set(value) == {'schema_version','command','status','error_code','data','summary','warnings'}
        assert value['schema_version'] == 1
        return value

    def record(name):
        results.append(dict(case=name,status='passed'))

    try:
        imports = {}
        for name, path in [('cli',binary),('worker',worker)]:
            if platform_name() == 'macos':
                data = subprocess.check_output(['otool','-L',str(path)],text=True)
            elif os.name == 'nt':
                data = subprocess.check_output(['dumpbin','/dependents',str(path)],text=True,errors='replace')
            else:
                data = subprocess.check_output(['readelf','-d',str(path)],text=True)
            assert 'Qt6' not in data and 'QtCore' not in data and 'Sparkle' not in data
            imports[name] = 'no-qt-or-sparkle-imports'
        identity = json.loads(command('--version-json').stdout)
        if install_fixture:
            payload = work/'installation-fixture'
            fixture_bin = payload/'bin';fixture_bin.mkdir(parents=True)
            fixture_files = [(binary,binary.name),(worker,worker.name),(binary.parent/('xdzip-launcher.exe' if os.name=='nt' else 'xdzip-launcher'),'xdzip-launcher.exe' if os.name=='nt' else 'xdzip-launcher')]
            files = []
            for source, name in fixture_files:
                shutil.copy2(source,fixture_bin/name)
                files.append(dict(path='bin/'+name,sha256=hashlib.sha256((fixture_bin/name).read_bytes()).hexdigest(),executable=True))
            (payload/'cli-payload.json').write_text(json.dumps(dict(schema_version=1,product='xdzip',identity=identity,files=files))+'\n')
        assert command('--version').stdout.startswith(('xdzip '+identity['version']).encode())
        assert b'Examples' in command('--help','--language','en-US').stdout
        assert '示例'.encode() in command('--language','zh-CN','--help').stdout
        command()
        assert invoke('missing-command',code=2)['error_code'] == 'invalid_arguments'
        record('version_help_bilingual_and_parse_exit')
        source = work/'输入 files';source.mkdir()
        (source/'你好.txt').write_bytes('归档内容\n'.encode())
        (source/'empty').mkdir()
        (source/'.DS_Store').write_bytes(b'ignored metadata')
        (source/'__MACOSX').mkdir();(source/'__MACOSX'/'ignored').write_bytes(b'ignored')
        (work/'--input.txt').write_bytes(b'literal file')
        for suffix in ['zip','tar','tar.gz','tgz','7z']:
            archive = work/('roundtrip.'+suffix)
            invoke('create',archive,source)
            listed = invoke('list',archive)
            names = [entry['path'] for entry in listed['data']['entries']]
            assert '输入 files/你好.txt' in names
            assert not any('.DS_Store' in (name or '') or '__MACOSX' in (name or '') for name in names)
            tested = invoke('test',archive)
            if suffix == 'tar':
                assert tested['data']['check_kind'] == 'structure_and_readability'
            target = work/('extracted-'+suffix)
            invoke('extract',archive,'输入 files/你好.txt','-C',target)
            assert (target/'输入 files/你好.txt').read_bytes() == (source/'你好.txt').read_bytes()
            added = work/('added-'+suffix+'.txt');added.write_bytes(b'new contents')
            invoke('add',archive,added,'--into','输入 files')
            target2 = work/('added-output-'+suffix)
            invoke('extract',archive,'-C',target2)
            assert (target2/'输入 files'/added.name).read_bytes() == b'new contents'
            if suffix == 'zip':
                with zipfile.ZipFile(archive) as reader:
                    assert reader.read('输入 files/你好.txt') == (source/'你好.txt').read_bytes()
            if suffix in ['tar','tar.gz','tgz']:
                with tarfile.open(archive) as reader:
                    assert reader.extractfile('输入 files/你好.txt').read() == (source/'你好.txt').read_bytes()
        record('zip_tar_targz_7z_roundtrip_add_and_independent_readers')
        single = source/'你好.txt'
        for level in ['store','fast','normal','best']:
            archive = work/('single-'+level+'.gz')
            invoke('create',archive,single,'--level',level)
            assert gzip.decompress(archive.read_bytes()) == single.read_bytes()
            listed = invoke('list',archive)['data']['entries']
            assert len(listed) == 1 and listed[0]['path'] == 'single-'+level
            invoke('test',archive)
            target = work/('gzip-output-'+level)
            invoke('extract',archive,'-C',target)
            assert (target/('single-'+level)).read_bytes() == single.read_bytes()
        invoke('create',work/'refused.gz',source,code=1)
        assert not (work/'refused.gz').exists()
        for suffix in ['xpi','apk','ipa','cbz']:
            alias = work/('package.'+suffix)
            shutil.copyfile(ROOT/'tests/fixtures/zip/normal.zip',alias)
            invoke('list',alias)
            invoke('test',alias)
        record('gzip_single_file_levels_and_zip_aliases')
        phrase = b'abracadabra abracadabra\n' * 6
        for name, kind, output_name in [
            ('phrase.bz2','supported_content','phrase'), ('phrase.lzma','structure_and_readability','phrase'),
            ('phrase.lz','supported_content','phrase'), ('files.tbz2','supported_content','hello.txt'),
            ('files.tar.lzma','structure_and_readability','hello.txt'), ('files.tlz','supported_content','hello.txt'),
            ('phrase.zst','format_declared_checks','phrase'), ('files.tar.zst','format_declared_checks','hello.txt'),
            ('phrase.lz4','format_declared_checks','phrase'), ('files.tar.lz4','format_declared_checks','hello.txt'),
            ('phrase-crc64.xz','format_declared_checks','phrase-crc64'), ('files.txz','format_declared_checks','hello.txt'),
            ('zipx-deflate64.zipx','supported_content','hello.txt'), ('zipx-bzip2.zipx','supported_content','hello.txt'),
            ('zipx-lzma.zipx','supported_content','hello.txt'), ('zipx-xz.zipx','supported_content','hello.txt'),
        ]:
            archive = ROOT/'tests/fixtures/streams'/name
            listed = invoke('list',archive)['data']['entries']
            assert len(listed) == 1 and listed[0]['path'] == output_name
            tested = invoke('test',archive)
            assert tested['data']['check_kind'] == kind
            target = work/('filtered-output-'+name)
            invoke('extract',archive,'-C',target)
            assert (target/output_name).read_bytes() == phrase
        for suffix in ['bz2','lzma','lz']:
            refused = work/('read-only.'+suffix)
            assert invoke('create',refused,single,code=2)['error_code'] == 'unsupported_format'
            assert not refused.exists()
        record('bzip2_lzma_lzip_and_tar_filters_read_only')
        for suffix in ['xz','tar.xz','txz']:
            for level in ['fast','normal','best']:
                archive=work/('created-'+level+'.'+suffix)
                invoke('create',archive,single if suffix=='xz' else source,'--level',level)
                invoke('test',archive)
                if suffix=='xz':
                    assert lzma.decompress(archive.read_bytes())==single.read_bytes()
                else:
                    with tarfile.open(archive,'r:xz') as independent:
                        assert independent.extractfile(source.name+'/'+single.name).read()==single.read_bytes()
                target=work/('created-output-'+suffix+'-'+level)
                invoke('extract',archive,'-C',target)
                assert (target/('created-'+level if suffix=='xz' else source.name+'/'+single.name)).read_bytes()==single.read_bytes()
            refused=work/('store-refused.'+suffix)
            assert invoke('create',refused,single,'--level','store',code=2)['error_code']=='unsupported_options'
            assert not refused.exists()
        record('xz_txz_lzma2_levels_and_independent_readers')
        for suffix in ['lz4','tar.lz4']:
            for level in ['fast','normal','best']:
                archive=work/('created-'+level+'.'+suffix)
                invoke('create',archive,single if suffix=='lz4' else source,'--level',level)
                invoke('test',archive)
                reference=os.environ.get('XDZIP_LZ4_REFERENCE')
                if reference:
                    decoded=subprocess.run([reference,'-d','-c',str(archive)],capture_output=True,check=True).stdout
                    if suffix=='lz4':
                        assert decoded==single.read_bytes()
                    else:
                        import io
                        with tarfile.open(fileobj=io.BytesIO(decoded),mode='r:') as independent:
                            assert independent.extractfile(source.name+'/'+single.name).read()==single.read_bytes()
                target=work/('created-output-'+suffix+'-'+level)
                invoke('extract',archive,'-C',target)
                assert (target/('created-'+level if suffix=='lz4' else source.name+'/'+single.name)).read_bytes()==single.read_bytes()
            refused=work/('store-refused.'+suffix)
            assert invoke('create',refused,single,'--level','store',code=2)['error_code']=='unsupported_options'
            assert not refused.exists()
        record('lz4_standard_frame_levels_and_tar_composition')
        zstd_source=work/'zstd-source'
        zstd_source.mkdir()
        zstd_single=zstd_source/'data.bin'
        zstd_single.write_bytes(b'CLI real Zstandard payload for independent decoding\n'*12000)
        for suffix in ['zst','tar.zst','tzst']:
            for level in ['fast','normal','best']:
                archive=work/('created-'+level+'.'+suffix)
                invoke('create',archive,zstd_single if suffix=='zst' else zstd_source,'--level',level)
                invoke('test',archive)
                reference=os.environ.get('XDZIP_ZSTD_REFERENCE_PYTHON')
                if reference:
                    decoded=subprocess.run([reference,'-c',"import compression.zstd as z,sys; assert z.zstd_version_info==(1,5,7); sys.stdout.buffer.write(z.decompress(open(sys.argv[1],'rb').read()))",str(archive)],capture_output=True,check=True).stdout
                    if suffix=='zst':
                        assert decoded==zstd_single.read_bytes()
                    else:
                        import io
                        with tarfile.open(fileobj=io.BytesIO(decoded),mode='r:') as independent:
                            assert independent.extractfile(zstd_source.name+'/'+zstd_single.name).read()==zstd_single.read_bytes()
                target=work/('created-output-'+suffix+'-'+level)
                invoke('extract',archive,'-C',target)
                assert (target/('created-'+level if suffix=='zst' else zstd_source.name+'/'+zstd_single.name)).read_bytes()==zstd_single.read_bytes()
            refused=work/('store-refused.'+suffix)
            assert invoke('create',refused,zstd_single,'--level','store',code=2)['error_code']=='unsupported_options'
            assert not refused.exists()
        record('zstandard_real_multiblock_levels_and_tar_compositions')
        for suffix in ['lzh','lha']:
            for level in ['fast','normal','best']:
                archive=work/f'created-{level}.{suffix}'
                invoke('create',archive,source,'--level',level)
                listed=invoke('list',archive)['data']
                assert listed['format']=='LZH'
                assert all(row['directory'] or row['method']==5 for row in listed['entries'])
                assert invoke('test',archive)['data']['check_kind']=='format_declared_checks'
                target=work/f'lzh-created-output-{level}-{suffix}'
                invoke('extract',archive,'-C',target)
                assert (target/source.name/'你好.txt').read_bytes()==(source/'你好.txt').read_bytes()
                before=hashlib.sha256(archive.read_bytes()).hexdigest()
                invoke('add',archive,single,code=1)
                assert hashlib.sha256(archive.read_bytes()).hexdigest()==before
            refused=work/f'lzh-store-refused.{suffix}'
            assert invoke('create',refused,single,'--level','store',code=2)['error_code']=='unsupported_options'
            assert not refused.exists()
        record('lzh_real_lh5_levels_unicode_directories_full_check_and_atomic_creation')
        zero=work/'zero-source.bin'
        zero.write_bytes(bytes(4096))
        for suffix in ['gz','xz','lz4','zst']:
            archive=work/('zeros.'+suffix)
            invoke('create',archive,zero)
            listed=invoke('list',archive)['data']['entries']
            assert len(listed)==1 and listed[0]['path']=='zeros' and listed[0]['size']==4096
            invoke('test',archive)
            target=work/('zeros-output-'+suffix)
            invoke('extract',archive,'-C',target)
            assert (target/'zeros').read_bytes()==bytes(4096)
            empty=work/('empty.tar.'+suffix)
            shutil.copyfile(archive,empty)
            assert invoke('list',empty)['data']['entries']==[]
        unknown=work/'empty.unknown'
        shutil.copyfile(work/'zeros.gz',unknown)
        assert invoke('list',unknown)['data']['entries']==[]
        record('zero_filled_ordinary_streams_keep_named_files')
        deb=ROOT/'tests/fixtures/deb/control-gz-data-zst.deb'
        entries=invoke('list',deb)['data']['entries']
        assert len(entries)==9 and [entry['id'] for entry in entries]==list(range(1,10))
        assert all(entry['packed_size'] is not None if entry['method']==0 else entry['packed_size'] is None for entry in entries)
        assert invoke('test',deb)['data']['check_kind']=='format_declared_checks'
        deb_target=work/'deb-all'
        invoke('extract',deb,'-C',deb_target)
        assert (deb_target/'members/debian-binary').read_bytes()==b'2.0\n'
        assert (deb_target/'control/control').read_bytes().startswith(b'Package: xdzip-owned-fixture\n')
        assert (deb_target/'control/postinst').read_bytes()==b'#!/bin/sh\necho owned-test-only\n'
        assert (deb_target/'data/usr/share/xdzip/hello.txt').read_bytes()==phrase
        assert (deb_target/'data/usr/share/xdzip/资料.txt').read_bytes()==b'owned UTF-8 content\n'
        assert (deb_target/'data/usr/share/xdzip/empty.bin').read_bytes()==b''
        deb_selected=work/'deb-selected'
        invoke('extract',deb,'data/','-C',deb_selected)
        assert (deb_selected/'data/usr/share/xdzip/hello.txt').read_bytes()==phrase
        assert not (deb_selected/'members').exists() and not (deb_selected/'control').exists()
        duplicate_deb=ROOT/'tests/fixtures/deb/duplicates.deb'
        matching=[entry for entry in invoke('list',duplicate_deb)['data']['entries'] if entry['path']=='data/usr/share/xdzip/hello.txt']
        assert len(matching)==2 and matching[0]['id']!=matching[1]['id']
        invoke('extract',duplicate_deb,'--entry-id',matching[1]['id'],'-C',work/'deb-physical')
        assert (work/'deb-physical/data/usr/share/xdzip/hello.txt').read_bytes()==b'second physical record\n'
        before=hashlib.sha256(deb.read_bytes()).hexdigest()
        invoke('add',deb,source,code=1)
        assert hashlib.sha256(deb.read_bytes()).hexdigest()==before
        record('deb_raw_and_nested_views_physical_selection_and_readonly_boundary')
        for name in ['lh0-h0.lzh','lh1-h1.lzh','lh2-h2.lzh','lh3-h3.lzh',
                     'lh4-h0.lzh','lh5-h2.lzh','lh6-h1.lzh','lh7-h3.lzh',
                     'lzs-h2.lzh','lz4-h0.lzh','lz5-h2.lzh','named-utf8.lzh','named-shiftjis.lzh']:
            archive=ROOT/'tests/fixtures/lha'/name
            listed=invoke('list',archive)['data']
            assert listed['format']=='LZH'
            assert invoke('test',archive)['data']['check_kind']=='format_declared_checks'
            entry=listed['entries'][0]
            target=work/('lha-'+name)
            invoke('extract',archive,'--entry-id',entry['id'],'-C',target)
            assert (target/entry['path']).read_bytes()==phrase
        archive=ROOT/'tests/fixtures/lha/lh5-h2.lzh'
        before=hashlib.sha256(archive.read_bytes()).hexdigest()
        invoke('add',archive,source,code=1)
        assert hashlib.sha256(archive.read_bytes()).hexdigest()==before
        record('lha_all_admitted_methods_header_levels_decoded_names_and_readonly_boundary')
        for name,expected in [
            ('pm0-allbytes.pma', bytes(range(256))),
            ('pm2-boundary-1024.pma', (bytes(range(256)) + b'abracadabra abracadabra\n'*128)[:1024]),
            ('pm2-deep28.pma', bytes(range(39,31,-1))*16),
        ]:
            archive=ROOT/'tests/fixtures/pma'/name
            listed=invoke('list',archive)['data']
            assert listed['format']=='PMA' and len(listed['entries'])==1
            assert invoke('test',archive)['data']['check_kind']=='format_declared_checks'
            entry=listed['entries'][0]
            assert entry['path']=='HELLO.BIN' and entry['size']==len(expected)
            target=work/('pma-'+name)
            invoke('extract',archive,'--entry-id',entry['id'],'-C',target)
            assert (target/'HELLO.BIN').read_bytes()==expected
        empty=invoke('list',ROOT/'tests/fixtures/pma/empty.pma')['data']
        assert empty['format']=='PMA' and empty['entries']==[]
        before=hashlib.sha256(archive.read_bytes()).hexdigest()
        invoke('add',archive,source,code=1)
        assert hashlib.sha256(archive.read_bytes()).hexdigest()==before
        record('pma_pm0_pm2_final_controls_deep_codes_empty_and_readonly_boundary')
        known=(ROOT/'tests/fixtures/pma/pm1-known.bin').read_bytes()
        for name,expected in [('pm1-selector-17.pma',known[17*256:18*256]),('pm1-long-overlap.pma',b' '*24704)]:
            archive=ROOT/'tests/fixtures/pma'/name
            listed=invoke('list',archive)['data']
            assert listed['format']=='PMA' and len(listed['entries'])==1
            assert invoke('test',archive)['data']['check_kind']=='format_declared_checks'
            target=work/('pma-'+name)
            invoke('extract',archive,'--entry-id',listed['entries'][0]['id'],'-C',target)
            assert (target/'HELLO.BIN').read_bytes()==expected
        record('pma_pm1_repeated_literal_branches_overlap_full_check_and_selected_extraction')
        for asar_name in ['ordinary.asar','without-integrity.asar','shared.asar','executable.asar','empty.asar']:
            asar=ROOT/'tests/fixtures/asar'/asar_name
            listed=invoke('list',asar)['data']
            assert listed['format']=='ASAR'
            assert invoke('test',asar)['data']['check_kind']=='format_declared_checks'
            if asar_name=='empty.asar':
                assert listed['entries']==[]
                continue
            asar_target=work/('asar-'+asar_name)
            invoke('extract',asar,'-C',asar_target)
            if asar_name=='executable.asar':
                assert (asar_target/'run.txt').read_bytes()==phrase
            else:
                assert (asar_target/'hello.txt').read_bytes()==phrase
                assert (asar_target/'copy.txt').read_bytes()==phrase
                assert (asar_target/'empty-dir').is_dir()
            if asar_name in ['ordinary.asar','without-integrity.asar']:
                assert (asar_target/'assets/资料.txt').read_bytes()=='owned UTF-8 ASAR content\n资料\n'.encode()
                assert (asar_target/'empty.bin').read_bytes()==b''
        asar=ROOT/'tests/fixtures/asar/shared.asar'
        entries=invoke('list',asar)['data']['entries']
        invoke('extract',asar,'--entry-id',entries[1]['id'],'-C',work/'asar-selected')
        assert (work/'asar-selected/copy.txt').read_bytes()==phrase
        assert not (work/'asar-selected/hello.txt').exists()
        before=hashlib.sha256(asar.read_bytes()).hexdigest()
        invoke('add',asar,source,code=1)
        assert hashlib.sha256(asar.read_bytes()).hexdigest()==before
        for name in ['external.asar','link.asar']:
            refused=ROOT/'tests/fixtures/asar'/name
            assert invoke('list',refused)['data']['entries'][0]['warning']
            failed=invoke('extract',refused,'-C',work/('refused-'+name),code=1)
            assert failed['summary'].get('committed_files',0)==0
            target=work/('refused-'+name)
            assert not target.exists() or (target.is_dir() and not list(target.iterdir()))
        record('asar_shared_ranges_declared_hashes_selection_and_readonly_boundary')

        uu_payload=b'Owned UU/XXE payload.\n'*17+bytes(range(256))
        for name in ['ordinary.uue','ordinary.xxe','spaces.uu','tails.uue','tails.xxe','empty.uue','empty.xxe','base64.uue']:
            envelope=ROOT/'tests/fixtures/uu'/name
            listed=invoke('list',envelope)['data']
            assert listed['format']==('XXE' if name.endswith('xxe') else 'UU')
            assert invoke('test',envelope)['data']['check_kind']=='structure_and_readability'
            target=work/('uu-'+name)
            invoke('extract',envelope,'-C',target)
            expected=b'' if name.startswith('empty') else bytes(range(46)) if name.startswith('tails') else uu_payload
            assert (target/('data.bin' if name=='base64.uue' else 'hello.bin')).read_bytes()==expected
            before=hashlib.sha256(envelope.read_bytes()).hexdigest()
            invoke('add',envelope,source,code=1)
            assert hashlib.sha256(envelope.read_bytes()).hexdigest()==before
        multi=ROOT/'tests/fixtures/uu/multi.uue'
        entries=invoke('list',multi)['data']['entries']
        assert len(entries)==3 and entries[0]['path']==entries[1]['path'] and entries[0]['id']!=entries[1]['id']
        invoke('extract',multi,'--entry-id',entries[1]['id'],'-C',work/'uu-selected')
        assert (work/'uu-selected/hello.bin').read_bytes()==bytes(range(46))
        assert len(list((work/'uu-selected').iterdir()))==1
        record('uu_xxe_base64_profiles_physical_selection_and_readonly_boundary')

        for name,expected in [('phrase.Z',phrase),('empty.Z',b''),('nonblock.Z',b'ABABABA'),('clear.Z',b'ABABCDCD'),('zeros.Z',bytes(4096)),('empty-tar.Z',bytes(1024)),('files.taz',phrase),('empty.taz',None)]:
            compressed=ROOT/'tests/fixtures/compress'/name
            listed=invoke('list',compressed)['data']
            tar=name.endswith('taz')
            assert listed['format']==('TAR.Z' if tar else 'Z')
            assert invoke('test',compressed)['data']['check_kind']=='structure_and_readability'
            if expected is None:
                assert not listed['entries']
                continue
            assert len(listed['entries'])==1 and (listed['entries'][0]['packed_size'] is None)==tar
            target=work/('compress-'+name)
            invoke('extract',compressed,'-C',target)
            assert (target/listed['entries'][0]['path']).read_bytes()==expected
            before=hashlib.sha256(compressed.read_bytes()).hexdigest()
            invoke('add',compressed,source,code=1)
            assert hashlib.sha256(compressed.read_bytes()).hexdigest()==before
        invoke('list',ROOT/'tests/fixtures/compress/bad-header.Z',code=1)
        record('unix_compress_real_lzw_empty_streams_tar_and_readonly_boundary')

        marker=bytearray(512);marker[254:259]=b'ustar'
        for name,expected in [('q11-text.br',phrase),('files.tar.br',phrase),('metadata.br',phrase),('empty.br',b''),('raw.br',b'Z'),('raw-tar-marker.br',bytes(marker)),('zeros.br',bytes(4096)),('empty.tar.br',None)]:
            compressed=ROOT/'tests/fixtures/brotli'/name
            listed=invoke('list',compressed)['data'];tar=name.endswith('.tar.br')
            assert listed['format']==('TAR.BR' if tar else 'Brotli')
            assert invoke('test',compressed)['data']['check_kind']=='structure_and_readability'
            if expected is None:
                assert not listed['entries'];continue
            row=listed['entries'][0];assert (row['packed_size'] is None)==tar
            target=work/('brotli-'+name)
            invoke('extract',compressed,'--entry-id',row['id'],'-C',target)
            assert (target/row['path']).read_bytes()==expected
            before=hashlib.sha256(compressed.read_bytes()).hexdigest();invoke('add',compressed,source,code=1)
            assert hashlib.sha256(compressed.read_bytes()).hexdigest()==before
        invoke('list',ROOT/'tests/fixtures/brotli/bad-header.br',code=1)
        for suffix in ['br','tar.br']:
            refused=work/('read-only.'+suffix)
            assert invoke('create',refused,single,code=2)['error_code']=='unsupported_format'
            assert not refused.exists()
        record('brotli_strict_headerless_streams_tar_selection_and_readonly_boundary')

        for version in range(4):
            for profile in ['phrase','empty','tail17','unicode']:
                encrypted=ROOT/'tests/fixtures/aescrypt'/f'v{version}-{profile}.aes'
                secret=('owned-資料-🔑' if profile=='unicode' else 'owned-aes-fixture').encode()+b'\n'
                assert invoke('list',encrypted,code=3)['error_code']=='interaction_required'
                wrong=invoke('list',encrypted,'--password-stdin',stdin=b'incorrect\n',code=1)
                assert wrong['error_code'] in ['wrong_password','password_or_integrity']
                listed=invoke('list',encrypted,'--password-stdin',stdin=secret)['data']
                assert listed['format']=='AES Crypt' and len(listed['entries'])==1
                row=listed['entries'][0]
                expected=b'' if profile=='empty' else bytes(range(17)) if profile=='tail17' else phrase
                assert row['size']==len(expected) and row['packed_size']==encrypted.stat().st_size
                assert invoke('test',encrypted,'--password-stdin',stdin=secret)['data']['check_kind']=='format_declared_checks'
                target=work/f'aes-{version}-{profile}'
                result=command('extract',encrypted,'--entry-id',row['id'],'-C',target,'--password-stdin','--json',stdin=secret)
                assert secret.strip() not in result.stdout+result.stderr
                assert (target/encrypted.stem).read_bytes()==expected
                before=hashlib.sha256(encrypted.read_bytes()).hexdigest()
                invoke('add',encrypted,source,code=3)
                assert hashlib.sha256(encrypted.read_bytes()).hexdigest()==before
        damaged=work/'damaged.aes';data=bytearray((ROOT/'tests/fixtures/aescrypt/v3-phrase.aes').read_bytes());data[-1]^=1;damaged.write_bytes(data)
        assert invoke('extract',damaged,'-C',work/'aes-unpublished','--password-stdin',stdin=b'owned-aes-fixture\n',code=1)['status']=='failed'
        assert not (work/'aes-unpublished').exists()
        record('aescrypt_all_versions_opening_credentials_authenticated_selection_and_readonly_boundary')



        check_desktop_worker(binary,worker,work,env,source/'你好.txt')
        record('desktop_shared_and_standalone_default_worker_lookup')
        invoke('create',work/'literal.zip','--','--input.txt')
        assert invoke('list',work/'literal.zip')['data']['entries'][0]['path'] == '--input.txt'
        duplicate = ROOT/'tests/fixtures/zip/normal.zip'
        complete = invoke('list', duplicate)['data']['entries']
        page = invoke('list', duplicate, '--offset', '1', '--limit', '1')['data']
        assert page['entries'] == complete[1:2] and page['offset'] == 1
        assert page['total'] == len(complete) and page['next_offset'] == 2
        assert invoke('list', duplicate, '--offset', str(len(complete)))['data']['entries'] == []
        assert invoke('list', duplicate, '--limit', '0', code=2)['error_code'] == 'invalid_arguments'
        record('bounded_physical_listing_pages_and_argument_limits')
        assert invoke('extract',duplicate,'notes.txt','-C',work/'ambiguous',code=1)['error_code'] == 'ambiguous_member'
        assert not (work/'ambiguous').exists()
        invoke('extract',duplicate,'--entry-id','3','-C',work/'physical')
        assert (work/'physical'/'notes.txt').read_bytes() == b'first'
        assert invoke('extract',duplicate,'missing','-C',work/'missing',code=1)['error_code'] == 'member_not_found'
        assert not (work/'missing').exists()
        record('literal_paths_physical_ids_and_zero_write_selection_failure')
        archive = work/'literal.zip';target = work/'conflict';target.mkdir()
        (target/'--input.txt').write_bytes(b'keep')
        conflict = invoke('extract',archive,'-C',target,code=3)
        assert conflict['error_code'] == 'interaction_required'
        assert (target/'--input.txt').read_bytes() == b'keep'
        skipped = invoke('extract',archive,'-C',target,'--conflict','skip')
        assert skipped['summary']['skipped_files'] == 1
        invoke('extract',archive,'-C',target,'--conflict','rename')
        assert (target/'--input.txt').read_bytes() == b'keep'
        invoke('extract',archive,'-C',target,'--conflict','overwrite')
        assert (target/'--input.txt').read_bytes() == b'literal file'
        before = hashlib.sha256(archive.read_bytes()).hexdigest()
        invoke('add',archive,source,'--into','missing-directory',code=1)
        assert hashlib.sha256(archive.read_bytes()).hexdigest() == before
        invoke('create',archive,archive,'--conflict','overwrite',code=1)
        assert hashlib.sha256(archive.read_bytes()).hexdigest() == before
        record('conflict_prompts_skip_rename_overwrite_and_source_protection')
        secret = b'owned-cli-fixture-password\n'
        for suffix, extra in [('zip',[]),('7z',['--encrypt-names','--solid'])]:
            archive = work/('encrypted.'+suffix)
            created = command('create',archive,source,'--encrypt','--password-stdin',*extra,'--json',stdin=secret)
            assert secret.strip() not in created.stdout+created.stderr
            invoke('test',archive,'--password-stdin',stdin=secret)
            invoke('extract',archive,'-C',work/('secret-output-'+suffix),'--password-stdin',stdin=secret)
            before = hashlib.sha256(archive.read_bytes()).hexdigest()
            invoke('add',archive,source,code=3 if suffix == '7z' else 1)
            assert hashlib.sha256(archive.read_bytes()).hexdigest() == before
        wrong = invoke('test',work/'encrypted.zip','--password-stdin',stdin=b'wrong\n',code=1)
        assert wrong['status'] == 'failed'
        with (work/'unreadable-stdin').open('wb') as unreadable:
            refused = subprocess.run([str(binary),'test',str(work/'encrypted.zip'),'--password-stdin','--json'],stdin=unreadable,capture_output=True,env=env,timeout=10)
            assert refused.returncode == 3 and json.loads(refused.stdout)['error_code'] == 'interaction_required'
        record('password_stdin_aes_header_encryption_and_edit_rejection')
        if os.name != 'nt':
            import errno
            import pty
            import select
            master, slave = pty.openpty()
            transcript = bytearray()
            child = subprocess.Popen([str(binary),'--language','en-US','create',str(work/'terminal.zip'),str(source),'--encrypt'],stdin=slave,stdout=slave,stderr=slave,env=env,cwd=work)
            os.close(slave)
            try:
                deadline = time.monotonic()+30
                sent = 0
                while time.monotonic() < deadline:
                    if select.select([master],[],[],0.1)[0]:
                        try:
                            block = os.read(master,4096)
                        except OSError as error:
                            if error.errno == errno.EIO:
                                break
                            raise
                        if not block:
                            break
                        transcript.extend(block)
                    prompt = b'Password: ' if sent == 0 else b'Confirm password: '
                    if sent < 2 and prompt in transcript:
                        os.write(master,secret)
                        sent += 1
                    if child.poll() is not None:
                        break
                assert child.wait(timeout=5) == 0 and sent == 2
                assert secret.strip() not in transcript
            finally:
                if child.poll() is None:
                    child.kill();child.wait()
                os.close(master)
            invoke('test',work/'terminal.zip','--password-stdin',stdin=secret)
            record('native_terminal_hidden_password_and_confirmation')
        else:
            shell_env = dict(env,XDZIP_CLI_TEST_BIN=str(binary))
            for shell in [['cmd','/d','/c',subprocess.list2cmdline([str(binary),'--json','missing-command'])],['powershell','-NoProfile','-NonInteractive','-Command','& $env:XDZIP_CLI_TEST_BIN --json missing-command; exit $LASTEXITCODE']]:
                called = subprocess.run(shell,capture_output=True,env=shell_env,timeout=30)
                assert called.returncode == 2, called.stderr.decode(errors='replace')
                assert json.loads(called.stdout)['error_code'] == 'invalid_arguments'
            record('cmd_and_powershell_wait_and_propagate_exit')
        invoke('create',work/'split.zip',source,'--volume-size','128')
        assert (work/'split.zip.001').is_file()
        invoke('test',work/'split.zip.001')
        invoke('extract',work/'split.zip.001','-C',work/'split-output')
        corrupt = work/'unsafe.zip'
        with zipfile.ZipFile(corrupt,'w') as writer:
            writer.writestr('../escape',b'never publish')
        assert b'[warning]' in command('list',corrupt).stdout
        invoke('extract',corrupt,'-C',work/'unsafe-output',code=1)
        assert not (work/'escape').exists()
        for suffix in ['rar','cab','iso','udf']:
            invoke('create',work/('unsupported.'+suffix),source,code=2)
        record('split_archive_and_unsafe_or_unsupported_inputs')
        readonly = [('CAB',ROOT/'tests/fixtures/cab-iso/named.cab'),('RAR',ROOT/'tests/fixtures/native-archives/rar7-store-small.rar')]
        for kind,name in [('ISO','iso9660.iso.gz'),('UDF','udf.iso.gz')]:
            image = work/(kind.lower()+'.iso')
            with gzip.open(ROOT/'tests/fixtures/cab-iso'/name,'rb') as stream:
                image.write_bytes(stream.read())
            readonly.append((kind,image))
        for kind,archive in readonly:
            copied = work/('readonly-'+archive.name)
            copied.write_bytes(archive.read_bytes())
            before = hashlib.sha256(copied.read_bytes()).hexdigest()
            listing = invoke('list',copied)
            assert listing['data']['format'] == kind
            safe_ids = [str(entry['id']) for entry in listing['data']['entries'] if entry['path'] is not None and not entry['warning'] and not entry['directory'] and not entry['link'] and entry['payload_supported']]
            assert safe_ids
            selected = [part for entry_id in safe_ids for part in ['--entry-id',entry_id]]
            invoke('extract',copied,*selected,'-C',work/('readonly-output-'+kind))
            verified = invoke('test',copied,code=1 if kind == 'UDF' else 0)
            if kind == 'UDF':
                assert verified['error_code'] == 'unsupported_feature'
                assert b'files committed' in command('--language','en-US','test',copied,code=1).stderr
            if kind == 'RAR':
                assert (work/('readonly-output-'+kind)/'sample.txt').read_bytes() == b'RAR newest algorithm verification sample'
            else:
                expected_name = 'HELLO.TXT' if kind == 'ISO' else 'hello.txt'
                assert (work/('readonly-output-'+kind)/expected_name).read_bytes() == b'xdzip read-only archive fixture\n'
            invoke('add',copied,source,code=1)
            assert hashlib.sha256(copied.read_bytes()).hexdigest() == before
        record('cab_rar_iso_udf_readonly_commands_and_unchanged_rejection')

        assert invoke('update','--status',code=3)['error_code'] == 'manual_install'
        portable_bin = work/'versions/portable/bin';portable_bin.mkdir(parents=True)
        portable = portable_bin/binary.name;shutil.copy2(binary,portable)
        portable_status = subprocess.run([str(portable),'update','--status','--json'],capture_output=True,env=env,timeout=10)
        assert portable_status.returncode == 3 and json.loads(portable_status.stdout)['error_code'] == 'manual_install'
        assert invoke('update','--check','--proxy','http://user:secret@localhost',code=2)['error_code'] == 'invalid_proxy'
        record('portable_update_and_proxy_credential_rejection')
        if os.name != 'nt':
            slow = work/'cancellation.bin';slow.write_bytes(os.urandom(8*1024*1024))
            child = subprocess.Popen([str(binary),'create',str(work/'cancel.7z'),str(slow),'--json'],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
            time.sleep(0.1)
            child.send_signal(signal.SIGINT)
            stdout,stderr = child.communicate(timeout=30)
            assert child.returncode == 130, (child.returncode,stdout[:1000])
            assert json.loads(stdout)['status'] == 'cancelled'
            assert not (work/'cancel.7z').exists()
            slow.unlink()
            record('native_sigint_waits_for_cleanup')
        if payload is not None:
            payload = payload.resolve()
            if os.name == 'nt':
                system = Path(os.environ['SystemRoot'])
                env['PATH'] = os.pathsep.join(map(str,[payload/'bin',system/'System32',system]))
            entry_binary = payload/'bin'/binary.name
            bin_dir, install_dir = work/'commands', work/'managed'
            result = subprocess.run([str(entry_binary),'setup','--bin-dir',str(bin_dir),'--install-dir',str(install_dir),'--json'],capture_output=True,env=env,timeout=60)
            assert result.returncode == 0, result.stdout[:1000]
            launcher = bin_dir/binary.name
            actual = json.loads(subprocess.check_output([str(launcher),'--version-json'],env=env))
            assert actual['version'] == identity['version']
            collision = subprocess.run([str(entry_binary),'setup','--bin-dir',str(bin_dir),'--install-dir',str(install_dir),'--json'],capture_output=True,env=env,timeout=30)
            assert collision.returncode == 3
            original = launcher.read_bytes()
            launcher.write_bytes(original+b'changed-registration-fixture')
            protected = subprocess.run([str(entry_binary),'setup','--remove','--bin-dir',str(bin_dir),'--json'],capture_output=True,env=env,timeout=30)
            assert protected.returncode == 3 and launcher.exists()
            launcher.write_bytes(original)
            remove = subprocess.run([str(launcher),'setup','--remove','--bin-dir',str(bin_dir),'--json'],capture_output=True,env=env,timeout=30)
            assert remove.returncode == 0 and not launcher.exists(), remove.stdout[:1500]
            assert (install_dir/'current.json').is_file()
            if os.name == 'nt':
                pointer = json.loads((install_dir/'current.json').read_text())
                installed = install_dir/'versions'/pointer['runtime']/'bin'/binary.name
                restored = subprocess.run([str(installed),'setup','--bin-dir',str(bin_dir),'--json'],capture_output=True,env=env,timeout=30)
                assert restored.returncode == 0, restored.stdout[:1500]
                assert not list((bin_dir/'.xdzip-retired').glob('*.exe'))
                removed = subprocess.run([str(installed),'setup','--remove','--bin-dir',str(bin_dir),'--json'],capture_output=True,env=env,timeout=30)
                assert removed.returncode == 0 and not launcher.exists(), removed.stdout[:1500]
            record('managed_install_launcher_collision_and_owned_removal')
        report = dict(schema_version=1,status='passed',identity=identity,platform=platform_name(),headless=True,imports=imports,executable_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),worker_sha256=hashlib.sha256(worker.read_bytes()).hexdigest(),results=results)
    except Exception as error:
        report = dict(schema_version=1,status='failed',platform=platform_name(),results=results,error=type(error).__name__)
        (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
        raise
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print('CLI verification passed: '+str(len(results))+' scenarios')


def platform_name():
    import sys
    return {'darwin':'macos','win32':'windows'}.get(sys.platform,'linux')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--worker',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--payload',type=Path)
    parser.add_argument('--default-worker',action='store_true',help='Clear worker overrides and exercise the deployed lookup')
    parser.add_argument('--install-fixture',action='store_true',help='Test setup using owned copies of these native binaries, without creating a release archive')
    args = parser.parse_args()
    check(args.binary,args.worker,args.output,args.payload,args.install_fixture,args.default_worker)


if __name__ == '__main__':
    main()
