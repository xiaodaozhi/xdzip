#!/usr/bin/env python3
"""Independent TAR/TAR.GZ interoperability and rejection checks on owned fixtures."""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import os
import re
import sys
import tarfile
import tempfile
import threading
import time
import zlib
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[1]


def capture(command, *, cwd=None, env=None, timeout=90, limit=262144):
    """Bound both retained output and the lifetime of this owned test process."""
    try:
        process=subprocess.Popen([str(value) for value in command],cwd=cwd,env=env,stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,stderr=subprocess.PIPE,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
    except OSError as error:
        return SimpleNamespace(returncode=None,stdout=b'',stderr=b'',failure='spawn_'+type(error).__name__)
    buffers=[bytearray(),bytearray()];lock=threading.Lock();overflow=threading.Event();count=0
    def drain(stream,index):
        nonlocal count
        while chunk:=stream.read(4096):
            with lock:
                retained=min(len(chunk),limit-count)
                buffers[index].extend(chunk[:retained]);count+=retained
                if retained<len(chunk):overflow.set()
    readers=[threading.Thread(target=drain,args=(stream,index),daemon=True)
             for index,stream in enumerate((process.stdout,process.stderr))]
    for reader in readers:reader.start()
    deadline=time.monotonic()+timeout;failure=None
    while process.poll() is None:
        if overflow.is_set() or time.monotonic()>=deadline:
            failure='output_limit' if overflow.is_set() else 'timeout'
            process.kill()  # Only this Popen-owned fixture/tool, never another process.
            break
        time.sleep(.01)
    process.wait(timeout=5)
    for reader in readers:reader.join(timeout=5)
    if any(reader.is_alive() for reader in readers):failure='output_pipe_timeout'
    else:
        process.stdout.close();process.stderr.close()
    if overflow.is_set():failure='output_limit'
    return SimpleNamespace(returncode=process.returncode,stdout=bytes(buffers[0]),stderr=bytes(buffers[1]),failure=failure)


def independent_reader(root,record):
    """Admit a reader using Python's reference archive before product creation."""
    reference=root/'python-reader-reference.tar'
    files={'资料/内容.txt':b'owned Unicode reference\n','资料/🧪.txt':b'owned non-BMP reference\n'}
    with tarfile.open(reference,'w',format=tarfile.PAX_FORMAT) as stream:
        for name in ('资料/','资料/空目录/'):
            item=tarfile.TarInfo(name);item.type=tarfile.DIRTYPE;item.mode=0o755;stream.addfile(item)
        for name,data in files.items():
            item=tarfile.TarInfo(name);item.size=len(data);stream.addfile(item,io.BytesIO(data))
    record.update(reference_format='Python tarfile PAX UTF-8',
                  reference_sha256=hashlib.sha256(reference.read_bytes()).hexdigest())
    compressed=root/'python-reader-reference.tar.gz'
    compressed.write_bytes(gzip.compress(reference.read_bytes(),mtime=0))
    record['reference_gzip_sha256']=hashlib.sha256(compressed.read_bytes()).hexdigest()
    candidates=[]
    found=shutil.which('tar')
    if found:candidates.append((Path(found).resolve(),'PATH'))
    if sys.platform=='win32' and (git:=shutil.which('git')):
        # Git is an existing source/CI tool, not a new deployed dependency.
        for parent in list(Path(git).resolve().parents)[:4]:
            candidate=parent/'usr/bin/tar.exe'
            if candidate.is_file() and not any(candidate.resolve()==value[0] for value in candidates):
                candidates.append((candidate.resolve(),'Git for Windows/usr/bin'))
                break
    admitted=[]
    for index,(tool,origin) in enumerate(candidates):
        info={'basename':tool.name,'origin':origin,'qualified':False,
              'path_key':hashlib.sha256(str(tool).encode()).hexdigest()}
        if sys.platform=='win32' and tool==Path(os.environ.get('SystemRoot','C:/Windows'),'System32/tar.exe').resolve():
            info['source_path']='<SystemRoot>/System32/'+tool.name
        elif origin.startswith('Git'):info['source_path']='<Git>/usr/bin/'+tool.name
        else:info['source_path']='<external>/'+tool.name
        record['candidates'].append(info)
        try:
            reader_env=None
            if origin.startswith('Git'):
                # GNU tar launches the existing sibling gzip. Only this reader's
                # child environment changes; the product probes retain PATH.
                reader_env={**os.environ,'PATH':str(tool.parent)+os.pathsep+os.environ.get('PATH','')}
                info['child_path_prefix']='<Git>/usr/bin'
                compressor=tool.parent/'gzip.exe'
                compressor_version=capture([compressor,'--version'],env=reader_env,timeout=15,limit=16384)
                info['gzip']={'source_path':'<Git>/usr/bin/gzip.exe',
                    'sha256':hashlib.sha256(compressor.read_bytes()).hexdigest(),
                    'version':compressor_version.stdout.decode('utf-8','replace').splitlines()[:1],
                    'version_returncode':compressor_version.returncode,'version_failure':compressor_version.failure}
            info['sha256']=hashlib.sha256(tool.read_bytes()).hexdigest()
            version=capture([tool,'--version'],env=reader_env,timeout=15,limit=16384)
            info.update(version=version.stdout.decode('utf-8','replace').splitlines()[:1],
                        version_returncode=version.returncode,version_failure=version.failure)
            expected={name:hashlib.sha256(data).hexdigest() for name,data in files.items()}
            info['reference_checks']=[]
            for kind,archive in (('tar',reference),('tar.gz',compressed)):
                destination=root/('reader-reference-'+str(index)+'-'+kind);destination.mkdir()
                result=capture([tool,'-xf',archive.name,'-C',destination.name],cwd=root,env=reader_env,timeout=15)
                actual={path.relative_to(destination).as_posix():hashlib.sha256(path.read_bytes()).hexdigest()
                        for path in destination.rglob('*') if path.is_file()}
                directories={path.relative_to(destination).as_posix() for path in destination.rglob('*') if path.is_dir()}
                directory_match=directories=={'资料','资料/空目录'}
                info['reference_checks'].append({'kind':kind,'returncode':result.returncode,'limit_failure':result.failure,
                    'unicode_names_and_bytes':actual==expected,'empty_directory':directory_match,
                    'stderr':result.stderr.decode('utf-8','replace')[-4096:],
                    'passed':not result.failure and result.returncode==0 and actual==expected and directory_match})
            qualified=not version.failure and version.returncode==0 and all(item['passed'] for item in info['reference_checks'])
            if 'gzip' in info:qualified=qualified and not compressor_version.failure and compressor_version.returncode==0
            info['qualified']=qualified
            if qualified:admitted.append((tool,reader_env,info))
        except Exception as error:
            info['admission_failure']=type(error).__name__
    if not admitted:raise RuntimeError('No independent reader passed the Python Unicode PAX reference; see evidence')
    tool,reader_env,selected=admitted[0]
    record['selected']=selected
    record['selection_reason']='Fixed before product creation after full Python PAX TAR and TAR.GZ Unicode/name/byte/empty-directory checks'
    return tool,reader_env


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--create-binary', type=Path, required=True)
    parser.add_argument('--extract-binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    create, extract = args.create_binary.resolve(), args.extract_binary.resolve()
    checks = [];reader_record={'candidates':[]};active_archive=None;phase='native_temp_base';base=None;root=None
    def safe_text(value):
        for path,label in ((root,'<fixture>'),(base,'<native-temp>'),(ROOT,'<repository>'),
                           (create,'<create-probe>'),(extract,'<extract-probe>')):
            if path:
                for variant in (str(path),str(path).replace('\\','/')):value=value.replace(variant,label)
        return value[-4096:]
    def fail(reason,**details):
        args.output.parent.mkdir(parents=True,exist_ok=True)
        failure={'phase':phase,'reason':reason,**details}
        if active_archive and active_archive.is_file() and active_archive.stat().st_size<=1048576:
            saved=args.output.with_name(args.output.stem+'-failed-input.bin')
            shutil.copyfile(active_archive,saved)
            failure.update(input_file=saved.name,input_sha256=hashlib.sha256(saved.read_bytes()).hexdigest())
        args.output.write_text(json.dumps({'status':'failed','passed':len(checks),'checks':checks,
            'independent_reader':reader_record,'failure':failure},ensure_ascii=True,indent=2)+'\n',encoding='utf-8')
        raise RuntimeError('TAR interoperability failed in '+phase+'; bounded evidence: '+args.output.name)
    def require(condition,reason):
        if not condition:fail(reason)
    def run(command, succeeds=True, **kwargs):
        result = capture(command,**kwargs)
        if result.failure or (result.returncode == 0) != succeeds:
            fail('subprocess exit or resource limit',tool=Path(command[0]).name,returncode=result.returncode,
                 limit_failure=result.failure,stderr=safe_text(result.stderr.decode('utf-8','replace')))
        return result
    def digest_tree(root):
        return {p.relative_to(root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                for p in root.rglob('*') if p.is_file()}
    base=run([create,'--native-temp-base'],timeout=15,limit=16384).stdout.decode('utf-8').strip()
    with tempfile.TemporaryDirectory(prefix='xdzip-tar-interop-', dir=base) as tmp:
        root = Path(tmp)
        source = root/'资料'
        (source/'空目录').mkdir(parents=True)
        files = {'内容.txt': '真实 TAR 与 GZIP\n'.encode()*2048, 'empty.bin': b'', '长名称'*20+'.txt': bytes(range(256))*128}
        for name, data in files.items():
            (source/name).write_bytes(data)
        expected = digest_tree(source)
        if sys.platform=='win32':
            import ctypes
            reader_record['windows_ansi_codepage']=ctypes.windll.kernel32.GetACP()
        reader_record['locale_environment']={name:(value if value is None or re.fullmatch(r'[A-Za-z0-9_.@-]{0,128}',value)
            else '<non-locale-value>') for name in ('LANG','LC_ALL','LC_CTYPE') for value in [os.environ.get(name)]}
        phase='reader_admission'
        try:system_tar,reader_env=independent_reader(root,reader_record)
        except Exception as error:fail('No independent reader qualified',exception=type(error).__name__)
        for suffix, method, level in [('.tar', 0, 6), ('.tar.gz', 8, 0), ('.tar.gz', 8, 1), ('.tar.gz', 8, 6), ('.tar.gz', 8, 9)]:
            output = root/f'created-{len(checks)}'
            output.mkdir()
            archive = output/('archive'+suffix)
            active_archive=archive;phase='xdzip creates '+suffix+' level '+str(level)
            run([create, source, output, archive.name, method, level])
            with tarfile.open(archive) as stream:
                records = stream.getmembers()
                require(all(r.isfile() or r.isdir() for r in records),'Python found a non-file/directory record')
                actual = {Path(r.name).relative_to('资料').as_posix(): hashlib.sha256(stream.extractfile(r).read()).hexdigest() for r in records if r.isfile()}
                require(actual == expected,'Python name/content mismatch')
                require(any(r.isdir() and r.name.rstrip('/') == '资料/空目录' for r in records),'Python empty-directory mismatch')
            system_output = output/'system'
            system_output.mkdir()
            # Relative input/output avoids GNU tar treating a Windows drive
            # colon as a remote archive. The admitted executable stays fixed.
            run([system_tar, '-xf', archive.name, '-C', system_output.name],cwd=output,env=reader_env)
            require(digest_tree(system_output/'资料') == expected,'Independent reader name/content mismatch')
            require((system_output/'资料/空目录').is_dir(),'Independent reader empty-directory mismatch')
            checks.append({'case': f'xdzip creates {suffix} level {level}', 'python_bytes': True, 'system_tar_bytes': True})
        for fmt in [tarfile.USTAR_FORMAT, tarfile.GNU_FORMAT, tarfile.PAX_FORMAT]:
            # USTAR has a short component ceiling. Other variants exercise GNU/PAX paths.
            manifest = files if fmt != tarfile.USTAR_FORMAT else {'内容.txt': files['内容.txt'], 'empty.bin': b''}
            payload = io.BytesIO()
            with tarfile.open(fileobj=payload, mode='w', format=fmt) as stream:
                marker = tarfile.TarInfo('./'); marker.type = tarfile.DIRTYPE; stream.addfile(marker)
                empty = tarfile.TarInfo('./空目录/'); empty.type = tarfile.DIRTYPE; stream.addfile(empty)
                for name, data in manifest.items():
                    info = tarfile.TarInfo('./'+name); info.size = len(data); info.mtime = 1700000000
                    stream.addfile(info, io.BytesIO(data))
            plain = payload.getvalue()
            mid = len(plain)//2
            variants = [('tar', plain), ('gzip', gzip.compress(plain)), ('multi-member', gzip.compress(plain[:mid])+gzip.compress(plain[mid:]))]
            for label, strategy in [('fixed', zlib.Z_FIXED), ('huffman-only', zlib.Z_HUFFMAN_ONLY), ('rle', zlib.Z_RLE), ('filtered', zlib.Z_FILTERED)]:
                encoder = zlib.compressobj(6, zlib.DEFLATED, 31, strategy=strategy)
                variants.append(('gzip-' + label, encoder.compress(plain) + encoder.flush()))
            for mode, encoded in variants:
                archive = root/f'python-{len(checks)}.data'
                archive.write_bytes(encoded)
                active_archive=archive;phase=f'Python format {fmt} -> xdzip {mode}'
                target = root/f'extracted-{len(checks)}'; target.mkdir()
                run([extract, archive, target, '--generic'])
                require(digest_tree(target) == {n: hashlib.sha256(v).hexdigest() for n,v in manifest.items()},'xdzip name/content mismatch')
                require((target/'空目录').is_dir(),'xdzip empty-directory mismatch')
                checks.append({'case': f'Python format {fmt} -> xdzip {mode}', 'all_bytes': True})
        # git archive emits a global PAX comment containing its commit identity.
        active_archive=None;phase='Git global PAX reference creation'
        git_tar = run(['git', '-C', ROOT, 'archive', 'HEAD', 'README.md']).stdout
        git_readme = run(['git', '-C', ROOT, 'show', 'HEAD:README.md']).stdout
        require(git_tar[156:157] == b'g','Git reference lacks its global PAX record')
        for mode, encoded in [('tar', git_tar), ('gzip', gzip.compress(git_tar))]:
            archive = root/f'git-source-{mode}.data'; archive.write_bytes(encoded)
            active_archive=archive;phase='Git global PAX comment -> xdzip '+mode
            target = root/f'git-source-{mode}'; target.mkdir()
            run([extract, archive, target, '--generic'])
            require(digest_tree(target) == {'README.md':hashlib.sha256(git_readme).hexdigest()},'Git reference content mismatch')
            checks.append({'case': f'Git global PAX comment -> xdzip {mode}', 'all_bytes':True})
        invalid = []
        for name, kind in [('../escape', tarfile.REGTYPE), ('link', tarfile.SYMTYPE), ('hardlink', tarfile.LNKTYPE), ('device', tarfile.CHRTYPE), ('fifo', tarfile.FIFOTYPE)]:
            payload = io.BytesIO()
            with tarfile.open(fileobj=payload, mode='w', format=tarfile.GNU_FORMAT) as stream:
                info=tarfile.TarInfo(name); info.type=kind; info.linkname='../../outside';stream.addfile(info)
            invalid.append((name, payload.getvalue()))
        zipped = bytearray(gzip.compress(plain)); zipped[-8] ^= 1
        invalid.extend([('gzip CRC', bytes(zipped)), ('truncated', plain[:600]), ('tail garbage', plain+b'X')])
        for name, data in invalid:
            archive=root/f'invalid-{len(checks)}'; archive.write_bytes(data)
            active_archive=archive;phase='reject '+name
            target=root/f'rejected-{len(checks)}';target.mkdir()
            run([extract, archive, target, '--generic'], succeeds=False)
            require(not any(target.iterdir()) and not (root/'escape').exists(),'Rejected archive wrote output')
            checks.append({'case': 'reject '+name, 'no_output': True})
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps({'status':'passed','passed': len(checks), 'checks': checks,
        'independent_reader':reader_record}, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    print(f'TAR interoperability: {len(checks)} cases passed')


if __name__ == '__main__':
    main()
