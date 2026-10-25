# scripts/check-advanced-creation.py

"""Bidirectional AES ZIP and numbered split interoperability on owned fixtures.

Only synthetic fixtures and a fixed fixture secret are used. The non-shipping
probes use their explicit test channels. 7-Zip gets passwords through its prompt,
using a terminal or redirected stdin, never command arguments. Neither product
creation nor extraction invokes these subprocesses or exposes a password
CLI/environment input channel.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import threading
import time
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def independent_outcome(code, transcript, password, responses, succeeds):
    """Only stable diagnostic labels may escape the private prompt transcript."""
    known_errors = (b'Wrong password', b'No files', b'Is not archive', b'Unknown switch',
                    b'Data Error', b'System ERROR', b'Break signaled',
                    b'passwords are not identical', b'Invalid argument', b'E_INVALIDARG',
                    b'E_FAIL', b'Not implemented', b'Can not open', b'Can not create')
    markers = [value.decode('ascii') for value in known_errors if value.lower() in transcript.lower()]
    if password is not None and responses == 0:
        raise RuntimeError(f'Independent fixture did not exercise password input, exit {code}, markers {markers!r}')
    if (code == 0) != succeeds:
        raise RuntimeError(f'Independent fixture outcome mismatch, exit {code}, prompts {responses}, markers {markers!r}')
    return code


def seven_zip_windows(command, password, cwd, succeeds):
    """Exercise the official Windows 7-Zip prompt with ASCII fixture passwords.

    Keep only a bounded private tail; prompts are answered as they arrive, not
    placed in argv or environment. The reader drains output while the child waits.
    """
    if password is not None and not password.isascii():
        raise RuntimeError('Windows reference prompt coverage requires an ASCII fixture password')
    command = [*command, '-sccUTF-8']
    child = subprocess.Popen(command, cwd=cwd, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                             stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW)
    state = dict(tail=b'', pending=b'', responses=0, failure=None)

    def read_output():
        try:
            while chunk := child.stdout.read1(8192):
                state['tail'] = (state['tail'] + chunk)[-65536:]
                state['pending'] = (state['pending'] + chunk)[-65536:]
                while match := re.search(rb'(?:Enter|Verify) password[^:\r\n]*:', state['pending'], re.I):
                    if password is None or state['responses'] >= 2:
                        state['failure'] = 'Unexpected independent fixture password prompt'
                        child.kill()
                        return
                    child.stdin.write(password.encode('utf-8') + b'\n')
                    child.stdin.flush()
                    state['responses'] += 1
                    state['pending'] = state['pending'][match.end():]
        except (OSError, ValueError):
            state['failure'] = 'Independent fixture password pipe failed'
            if child.poll() is None:
                child.kill()

    reader = threading.Thread(target=read_output, daemon=True)
    reader.start()
    try:
        try:
            code = child.wait(timeout=90)
        except subprocess.TimeoutExpired:
            raise RuntimeError('Independent fixture tool exceeded its time limit') from None
        reader.join(timeout=5)
        if reader.is_alive():
            raise RuntimeError('Independent fixture output did not terminate')
        if state['failure']:
            raise RuntimeError(state['failure'])
        return independent_outcome(code, state['tail'], password, state['responses'], succeeds)
    finally:
        if child.poll() is None:
            child.kill()
        child.wait(timeout=5)
        reader.join(timeout=5)
        child.stdin.close()
        child.stdout.close()


def seven_zip(command, password=None, cwd=None, succeeds=True):
    """Drive the platform-specific native password prompt.

    The transcript deliberately stays local to this function and is never printed,
    included in exceptions, or written into a report, even on tool failure.
    """
    if any(value.startswith('-p') and value != '-p' for value in command):
        raise RuntimeError('Passwords must use the fixture prompt channel')
    if os.name == 'nt':
        return seven_zip_windows(command, password, cwd, succeeds)
    import errno
    import pty
    import select
    import signal
    import termios
    pid, terminal = pty.fork()
    if pid == 0:
        try:
            if cwd is not None:
                os.chdir(cwd)
            os.execvpe(command[0], command, os.environ)
        except BaseException:
            os._exit(127)
    status = None
    pending = b''
    responses = 0
    deadline = time.monotonic() + 90
    try:
        while status is None:
            if time.monotonic() > deadline:
                raise RuntimeError('Independent fixture tool exceeded its time limit')
            ready, _, _ = select.select([terminal], [], [], 0.1)
            if ready:
                try:
                    chunk = os.read(terminal, 8192)
                except OSError as error:
                    if error.errno != errno.EIO:
                        raise
                    chunk = b''
                pending = (pending + chunk)[-65536:]
                while match := re.search(rb'(?:Enter|Verify) password[^:\r\n]*:', pending, re.I):
                    if password is None or responses >= 2:
                        raise RuntimeError('Unexpected independent fixture password prompt')
                    flags = termios.tcgetattr(terminal)
                    flags[3] &= ~termios.ECHO
                    termios.tcsetattr(terminal, termios.TCSANOW, flags)
                    os.write(terminal, password.encode('utf-8') + b'\n')
                    responses += 1
                    pending = pending[match.end():]
            done, result = os.waitpid(pid, os.WNOHANG)
            if done:
                status = result
        while select.select([terminal], [], [], 0)[0]:
            try:
                chunk = os.read(terminal, 8192)
            except OSError as error:
                if error.errno != errno.EIO:
                    raise
                break
            if not chunk:
                break
            pending = (pending + chunk)[-65536:]
        code = os.waitstatus_to_exitcode(status)
        return independent_outcome(code, pending, password, responses, succeeds)
    finally:
        if status is None:
            try:
                os.kill(pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            os.waitpid(pid, 0)
        os.close(terminal)


def manifest(root):
    return {path.relative_to(root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in root.rglob('*') if path.is_file()}


def directories(root):
    return {path.relative_to(root).as_posix() for path in root.rglob('*') if path.is_dir()}


def check_aes_profile(archive, method):
    """Inspect external metadata independently, without decrypting with Python."""
    with zipfile.ZipFile(archive) as stream:
        files = [entry for entry in stream.infolist() if not entry.is_dir()]
        assert files and all(entry.compress_type == 99 and entry.flag_bits & 1 for entry in files)
        nonempty_methods = set()
        for entry in files:
            extra, fields = entry.extra, []
            while extra:
                assert len(extra) >= 4
                tag = int.from_bytes(extra[:2], 'little')
                size = int.from_bytes(extra[2:4], 'little')
                assert len(extra) >= 4 + size
                if tag == 0x9901:
                    fields.append(extra[4:4+size])
                extra = extra[4+size:]
            assert len(fields) == 1 and len(fields[0]) == 7
            field = fields[0]
            assert int.from_bytes(field[:2], 'little') in (1, 2)
            assert field[2:4] == b'AE' and field[4] == 3
            actual = int.from_bytes(field[5:], 'little')
            assert actual in (0, 8)
            if entry.file_size:
                nonempty_methods.add(actual)
            if int.from_bytes(field[:2], 'little') == 2:
                assert entry.CRC == 0
        # 7-Zip can choose Store for incompressible members even with Deflate
        # requested. A nonempty member must exercise the requested method.
        assert method in nonempty_methods


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--extract-binary', type=Path, required=True)
    parser.add_argument('--reader', default=shutil.which('7zz') or shutil.which('7z'))
    parser.add_argument('--skip-infozip', action='store_true',
                        help='Explicitly omit the two Info-ZIP PKZIP reference cases on Windows')
    parser.add_argument('--output', type=Path, default=ROOT/'build/review/advanced-creation.json')
    args = parser.parse_args()
    if not args.reader:
        raise SystemExit('Independent 7-Zip is required; install outside the repository')
    if args.skip_infozip and os.name != 'nt':
        parser.error('--skip-infozip is limited to Windows; Unix runs require Info-ZIP')
    args.reader = str(Path(args.reader).resolve())
    binary = str(args.binary.resolve())
    extract = str(args.extract_binary.resolve())
    native_temp = subprocess.check_output([binary, '--native-temp-base'], text=True).strip()
    cases = []
    skipped = []
    with tempfile.TemporaryDirectory(prefix='xdzip-aes-split-', dir=native_temp) as owned:
        root = Path(owned)
        source = root/'input'
        source.mkdir()
        (source/'empty.txt').write_bytes(b'')
        (source/'空目录').mkdir()
        (source/'中文.txt').write_bytes(('AES and split ZIP，UTF-8\n'*5000).encode())
        (source/'random.bin').write_bytes(hashlib.shake_256(b'public synthetic fixture').digest(200_000))
        # Redirected Windows 7za prompts did not accept tested UTF-8/GBK input.
        # Keep this reference limitation explicit; Unix terminal checks retain
        # Unicode passwords. Rust independent ciphertext tests cover Unicode
        # separately and are not counted as executions of this script.
        password = 'public fixture ascii' if os.name == 'nt' else 'public fixture 密码'
        expected = manifest(source)
        expected_directories = directories(source)
        def extract_with_xdzip(archive, target, secret=password, profile=source):
            target.mkdir()
            result = subprocess.run([extract, str(archive), str(target), '--generic', '--fixture-password-stdin'],
                                    input=secret.encode('utf-8') if secret is not None else b'', capture_output=True, timeout=90)
            if result.returncode:
                raise RuntimeError('Owned AES extraction fixture failed')
            report = json.loads(result.stdout)
            assert report['password_prompts'] == int(secret is not None)
            assert report['files'] == len(manifest(profile))
            assert manifest(target/'input') == manifest(profile)
            assert directories(target/'input') == directories(profile)
            assert {path.name for path in target.iterdir()} == {'input'}
        for method in [0, 8]:
            for split in [False, True]:
                for encrypt in [False, True]:
                    out = root/f'out-{method}-{split}-{encrypt}'
                    out.mkdir()
                    command = [binary,str(source),str(out),'fixture.zip',str(method),'6',str(65536 if split else 0)]
                    if encrypt:
                        command.append('--fixture-password-stdin')
                    result = subprocess.run(command, input=password.encode('utf-8') if encrypt else None,
                                            capture_output=True, timeout=90)
                    if result.returncode:
                        raise RuntimeError('Owned creation fixture failed')
                    archive = out/('fixture.zip.001' if split else 'fixture.zip')
                    target = root/f'decoded-{method}-{split}-{encrypt}'
                    command = [args.reader, 'x', str(archive), '-o'+str(target)]
                    seven_zip(command, password if encrypt else None)
                    for path in source.rglob('*'):
                        decoded = target/'input'/path.relative_to(source)
                        assert decoded.is_dir() if path.is_dir() else decoded.read_bytes() == path.read_bytes(), 'Independent payload mismatch'
                    assert manifest(target/'input') == expected
                    assert directories(target/'input') == expected_directories
                    assert {path.name for path in target.iterdir()} == {'input'}
                    assert not any(p.name.startswith('.xdzip-work-') for p in out.iterdir())
                    parts = sorted(out.glob('fixture.zip.*')) if split else [archive]
                    if split:
                        assert len(parts) > 1
                        assert all(0 < p.stat().st_size <= 65536 for p in parts)
                    if encrypt:
                        seven_zip([args.reader, 't', str(archive)], 'wrong fixture', succeeds=False)
                        if not split:
                            check_aes_profile(archive, method)
                    extract_with_xdzip(archive, root/f'xdzip-decoded-{method}-{split}-{encrypt}', password if encrypt else None)
                    cases.append(dict(method=method, split=split, encrypted=encrypt, parts=len(parts),
                                      direction='xdzip_to_7zip', all_bytes=True,
                                      password_encoding=('ascii' if os.name == 'nt' else 'utf8') if encrypt else None,
                                      xdzip_roundtrip=True, passed=True))
            external = root/f'external-{method}.zip'
            # 7-Zip's ZIP writer rejects non-ASCII passwords on some hosts. The
            # Unix xdzip -> 7-Zip cases retain a UTF-8 password; Windows prompt
            # coverage is deliberately ASCII as recorded in the JSON evidence.
            external_password = hashlib.sha256(b'owned external AES fixture').hexdigest()[:32]
            seven_zip([args.reader, 'a', '-tzip', '-mem=AES256', '-mm='+('Copy' if method == 0 else 'Deflate'),
                       '-p', str(external), 'input'], external_password, cwd=root)
            check_aes_profile(external, method)
            extract_with_xdzip(external, root/f'external-decoded-{method}', external_password)
            cases.append(dict(method=method, direction='7zip_to_xdzip', encrypted=True, all_bytes=True, passed=True))
            split_external = root/f'external-split-{method}.zip'
            seven_zip([args.reader, 'a', '-tzip', '-mem=AES256', '-mm='+('Copy' if method == 0 else 'Deflate'),
                       '-v64k', '-p', str(split_external), 'input'], external_password, cwd=root)
            extract_with_xdzip(Path(str(split_external)+'.002'), root/f'external-split-decoded-{method}', external_password)
            cases.append(dict(method=method, direction='7zip_to_xdzip', split=True, encrypted=True, all_bytes=True, passed=True))
            if args.skip_infozip:
                skipped.append(dict(method=method, direction='infozip_to_xdzip', split=True, layout='pkzip',
                                    executed=False, reason='Explicit --skip-infozip: Windows reference tool is not installed'))
                continue
            zip_tool = shutil.which('zip')
            if not zip_tool:
                raise RuntimeError('Info-ZIP is required for independent PKZIP volume coverage')
            pkzip = root/f'pkzip-{method}.zip'
            # Info-ZIP's legacy Unicode extra-field profile is outside the
            # product's UTF-8-flag policy. Independently exercise disk layout
            # using ASCII names; the other directions retain Unicode coverage.
            pk_work = root/f'infozip-{method}'
            pk_input = pk_work/'input'
            pk_input.mkdir(parents=True)
            for index, path in enumerate(sorted(source.iterdir())):
                if path.is_dir():
                    (pk_input/f'directory-{index}').mkdir()
                else:
                    (pk_input/f'file-{index}.bin').write_bytes(path.read_bytes())
            result = subprocess.run([zip_tool, '-0' if method == 0 else '-6', '-s', '64k', '-r', str(pkzip), 'input'],
                                    cwd=pk_work, capture_output=True, timeout=90)
            if result.returncode:
                raise RuntimeError('Owned Info-ZIP split fixture creation failed')
            extract_with_xdzip(pkzip.with_suffix('.z01'), root/f'pkzip-decoded-{method}', None, pk_input)
            cases.append(dict(method=method, direction='infozip_to_xdzip', split=True, layout='pkzip', all_bytes=True, passed=True))
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(dict(status='passed', cases=cases, skipped_cases=skipped,
                                         independent_reader=Path(args.reader).name,
                                         unicode_password_prompt=dict(
                                             executed=os.name != 'nt',
                                             reason=('Official Windows 7za redirected prompt did not accept tested UTF-8/GBK '
                                                     'input; this script exercises ASCII passwords. Unicode independent '
                                                     'ciphertext and read/write tests are separate Rust evidence.'
                                                     if os.name == 'nt' else
                                                     'UTF-8 passwords exercised in encrypted xdzip_to_7zip cases')),
                                         password_transport=('redirected_stdin_and_fixture_stdin' if os.name == 'nt'
                                                             else 'terminal_and_fixture_stdin')), indent=2)+'\n',encoding='utf-8')
    print(f'Independent ZIP interoperability: {len(cases)} cases passed, {len(skipped)} explicitly not executed')


if __name__ == '__main__': main()
