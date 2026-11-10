#!/usr/bin/env python3
"""Exercise real headless CLI commands against owned archive inputs."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import tarfile
import time
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def check(binary, worker, output, payload=None):
    binary, worker, output = binary.resolve(), worker.resolve(), output.resolve()
    if not output.is_relative_to(ROOT/'build') or output == ROOT/'build':
        raise ValueError('CLI evidence must be under build/')
    output.mkdir(parents=True, exist_ok=True)
    work = output/('cases-'+str(os.getpid())+'-'+str(time.time_ns()))
    work.mkdir(mode=0o700)
    env = os.environ.copy()
    for key in ['DISPLAY','WAYLAND_DISPLAY','QT_PLUGIN_PATH','QT_QPA_PLATFORM_PLUGIN_PATH',
                'QML_IMPORT_PATH','QML2_IMPORT_PATH','LD_LIBRARY_PATH','DYLD_LIBRARY_PATH']:
        env.pop(key, None)
    env['XDZIP_NATIVE_CODEC'] = str(worker)
    env['XDZIP_NATIVE_CODEC_WORKER'] = str(worker)
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
        invoke('create',work/'literal.zip','--','--input.txt')
        assert invoke('list',work/'literal.zip')['data']['entries'][0]['path'] == '--input.txt'
        duplicate = ROOT/'tests/fixtures/zip/normal.zip'
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
        import gzip
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
            assert remove.returncode == 0 and not launcher.exists()
            assert (install_dir/'current.json').is_file()
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
    args = parser.parse_args()
    check(args.binary,args.worker,args.output,args.payload)


if __name__ == '__main__':
    main()
