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
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--create-binary', type=Path, required=True)
    parser.add_argument('--extract-binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    create, extract = args.create_binary.resolve(), args.extract_binary.resolve()
    base = subprocess.check_output([str(create), '--native-temp-base'], text=True).strip()
    checks = []
    def run(command, succeeds=True):
        result = subprocess.run([str(x) for x in command], capture_output=True, timeout=90)
        if (result.returncode == 0) != succeeds:
            raise RuntimeError('interop subprocess failed its expected outcome: ' + result.stderr.decode('utf-8', 'replace')[-300:])
        return result
    def digest_tree(root):
        return {p.relative_to(root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                for p in root.rglob('*') if p.is_file()}
    with tempfile.TemporaryDirectory(prefix='xdzip-tar-interop-', dir=base) as tmp:
        root = Path(tmp)
        source = root/'资料'
        (source/'空目录').mkdir(parents=True)
        files = {'内容.txt': '真实 TAR 与 GZIP\n'.encode()*2048, 'empty.bin': b'', '长名称'*20+'.txt': bytes(range(256))*128}
        for name, data in files.items():
            (source/name).write_bytes(data)
        expected = digest_tree(source)
        for suffix, method, level in [('.tar', 0, 6), ('.tar.gz', 8, 0), ('.tar.gz', 8, 1), ('.tar.gz', 8, 6), ('.tar.gz', 8, 9)]:
            output = root/f'created-{len(checks)}'
            output.mkdir()
            archive = output/('archive'+suffix)
            run([create, source, output, archive.name, method, level])
            with tarfile.open(archive) as stream:
                records = stream.getmembers()
                assert all(r.isfile() or r.isdir() for r in records)
                actual = {Path(r.name).relative_to('资料').as_posix(): hashlib.sha256(stream.extractfile(r).read()).hexdigest() for r in records if r.isfile()}
                assert actual == expected
                assert any(r.isdir() and r.name.rstrip('/') == '资料/空目录' for r in records)
            system_tar = shutil.which('tar')
            if system_tar:
                system_output = output/'system'
                system_output.mkdir()
                run([system_tar, '-xf', archive, '-C', system_output])
                if digest_tree(system_output/'资料') != expected:
                    raise RuntimeError('system tar name/content mismatch: '+json.dumps({'expected':sorted(expected),'actual':sorted(digest_tree(system_output))},ensure_ascii=True))
            checks.append({'case': f'xdzip creates {suffix} level {level}', 'python_bytes': True, 'system_tar_bytes': bool(system_tar)})
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
            for mode, encoded in [('tar', plain), ('gzip', gzip.compress(plain)), ('multi-member', gzip.compress(plain[:mid])+gzip.compress(plain[mid:]))]:
                archive = root/f'python-{len(checks)}.data'
                archive.write_bytes(encoded)
                target = root/f'extracted-{len(checks)}'; target.mkdir()
                run([extract, archive, target, '--generic'])
                assert digest_tree(target) == {n: hashlib.sha256(v).hexdigest() for n,v in manifest.items()}
                assert (target/'空目录').is_dir()
                checks.append({'case': f'Python format {fmt} -> xdzip {mode}', 'all_bytes': True})
        # git archive emits a global PAX comment containing its commit identity.
        git_tar = run(['git', '-C', ROOT, 'archive', 'HEAD', 'README.md']).stdout
        git_readme = run(['git', '-C', ROOT, 'show', 'HEAD:README.md']).stdout
        assert git_tar[156:157] == b'g'
        for mode, encoded in [('tar', git_tar), ('gzip', gzip.compress(git_tar))]:
            archive = root/f'git-source-{mode}.data'; archive.write_bytes(encoded)
            target = root/f'git-source-{mode}'; target.mkdir()
            run([extract, archive, target, '--generic'])
            assert digest_tree(target) == {'README.md':hashlib.sha256(git_readme).hexdigest()}
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
            target=root/f'rejected-{len(checks)}';target.mkdir()
            run([extract, archive, target, '--generic'], succeeds=False)
            assert not any(target.iterdir()) and not (root/'escape').exists()
            checks.append({'case': 'reject '+name, 'no_output': True})
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps({'passed': len(checks), 'checks': checks}, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    print(f'TAR interoperability: {len(checks)} cases passed')


if __name__ == '__main__':
    main()
