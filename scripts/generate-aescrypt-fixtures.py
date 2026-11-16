#!/usr/bin/env python3
"""Generate owned AES Crypt v0-v3 inputs using two independent crypto implementations."""
import argparse
import hashlib
import hmac
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / 'tests/fixtures/aescrypt'
ARCHIVE_SHA256 = '5edea8662cc04eb5fa53ce716df3de4bb28237ff0a2e951dfdc2795c851be7dd'
MANIFEST = '''[package]
name = "xdzip-aescrypt-reference"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[dependencies]
aescry = { path = "source/aescry-1.0.0" }
'''


def password(profile):
    return 'owned-資料-🔑' if profile == 'unicode' else 'p' * 128 + '資料-🔑' if profile == 'long' else 'owned-aes-fixture'


def independent_decode(data, text_password):
    from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
    if data[:3] != b'AES' or data[3] not in range(4):
        raise ValueError('Invalid owned header')
    version, at = data[3], 5
    if version >= 2:
        while True:
            size = int.from_bytes(data[at:at + 2], 'big')
            at += 2
            if not size:
                break
            at += size
    if version == 3:
        iterations = int.from_bytes(data[at:at + 4], 'big')
        at += 4
    iv = data[at:at + 16]
    at += 16
    if version == 3:
        key = hashlib.pbkdf2_hmac('sha512', text_password.encode(), iv, iterations, 32)
    else:
        key = iv + bytes(16)
        encoded_password = text_password.encode('utf-16le')
        for _ in range(8192):
            key = hashlib.sha256(key + encoded_password).digest()
    if version:
        wrapped = data[at:at + 48]
        at += 48
        tag = data[at:at + 32]
        at += 32
        if not hmac.compare_digest(hmac.new(key, wrapped + (b'\x03' if version == 3 else b''), 'sha256').digest(), tag):
            raise ValueError('Independent session HMAC mismatch')
        decoder = Cipher(algorithms.AES(key), modes.CBC(iv)).decryptor()
        session = decoder.update(wrapped) + decoder.finalize()
        iv, key = session[:16], session[16:]
    ciphertext = data[at:-33 if version in (1, 2) else -32]
    if not hmac.compare_digest(hmac.new(key, ciphertext, 'sha256').digest(), data[-32:]):
        raise ValueError('Independent data HMAC mismatch')
    decoder = Cipher(algorithms.AES(key), modes.CBC(iv)).decryptor()
    plain = decoder.update(ciphertext) + decoder.finalize()
    if version == 3:
        count = plain[-1]
        if not 1 <= count <= 16 or plain[-count:] != bytes([count]) * count:
            raise ValueError('Independent PKCS7 mismatch')
        return plain[:-count]
    count = data[4] if version == 0 else data[-33]
    if count not in range(16):
        raise ValueError('Invalid owned modulo')
    return plain[:len(plain) - (16 - count if count else 0)]


def with_extensions(data):
    if data[5:7] != bytes(2):
        raise ValueError('Reference unexpectedly added extensions')
    records = [b'CREATED_BY\x00xdzip owned input', b'FILENAME\x00../../ignored-name',
               b'urn:xdzip:owned\x00opaque', bytes(128)]
    return data[:5] + b''.join(len(record).to_bytes(2, 'big') + record for record in records) + bytes(2) + data[7:]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    ref = args.reference.resolve()
    if not ref.is_relative_to(ROOT / 'build/toolchain'):
        parser.error('Keep reference tools under build/toolchain/')
    archive = ref / 'aescry-1.0.0.crate'
    if hashlib.sha256(archive.read_bytes()).hexdigest() != ARCHIVE_SHA256:
        raise ValueError('Independent reference crate identity mismatch')
    with tarfile.open(archive) as package:
        for entry in package.getmembers():
            if not entry.isfile():
                if not entry.isdir():
                    raise ValueError('Unexpected reference member type')
                continue
            relative = Path(entry.name)
            if relative.is_absolute() or not relative.parts or relative.parts[0] != 'aescry-1.0.0':
                raise ValueError('Unexpected reference path')
            target = ref / 'source' / relative
            if not target.resolve().is_relative_to(ref / 'source') or '..' in relative.parts or target.is_symlink() or target.read_bytes() != package.extractfile(entry).read():
                raise ValueError('Reference source differs from pinned package')
    (ref / 'src').mkdir(exist_ok=True)
    shutil.copyfile(DEST / 'reference.rs', ref / 'src/main.rs')
    (ref / 'Cargo.toml').write_text(MANIFEST, encoding='utf-8')
    report = ROOT / 'build/review/format-expansion'
    report.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, CARGO_HOME=str(ROOT / 'build/toolchain/reference-cargo'), RUSTUP_TOOLCHAIN='1.99.0')
    built = subprocess.run(['cargo', 'build', '--release', '--locked', '--offline'], cwd=ref, env=env, capture_output=True)
    (report / 'm7-aes-reference-build.log').write_bytes(built.stdout + built.stderr)
    if built.returncode:
        raise ValueError('Independent reference build failed; inspect its build log')
    executable = ref / 'target/release/xdzip-aescrypt-reference.exe'
    phrase = b'abracadabra abracadabra\n' * 6
    cases = []
    for version in range(4):
        cases.extend((version, label, payload, 'ascii') for label, payload in [
            ('phrase', phrase), ('empty', b''), ('tail1', bytes(range(1))), ('tail15', bytes(range(15))),
            ('block16', bytes(range(16))), ('tail17', bytes(range(17)))])
        cases.append((version, 'unicode', phrase, 'unicode'))
    for version in [2, 3]:
        cases.append((version, 'long', phrase, 'long'))
        cases.append((version, 'extensions', phrase, 'ascii'))
    cases.extend([(3, 'default', phrase, 'default'), (3, 'iter1', bytes(range(17)), 'one'),
                  (3, 'all-bytes', bytes(range(256)), 'ascii'),
                  (3, 'multi', bytes((at * 73 ^ (at >> 8)) & 255 for at in range(131077)), 'ascii')])
    identities = {}
    with tempfile.TemporaryDirectory(prefix='aescrypt-reference-', dir=report) as temporary:
        work = Path(temporary)
        source, encoded, decoded = work / 'owned.bin', work / 'owned.aes', work / 'decoded.bin'
        for version, label, payload, profile in cases:
            name = f'v{version}-{label}.aes'
            source.write_bytes(payload)
            command = [str(executable), 'encode', str(version), profile, str(source), str(encoded)]
            run = subprocess.run(command, capture_output=True, timeout=30)
            if run.returncode:
                raise ValueError('Independent encoding failed: ' + name)
            data = encoded.read_bytes()
            if label == 'extensions':
                data = with_extensions(data)
                encoded.write_bytes(data)
            if independent_decode(data, password(profile)) != payload:
                raise ValueError('Independent Python/OpenSSL mismatch: ' + name)
            run = subprocess.run([str(executable), 'decode', str(version), profile, str(encoded), str(decoded)], capture_output=True, timeout=30)
            if run.returncode or decoded.read_bytes() != payload:
                raise ValueError('Independent AES Crypt reading mismatch: ' + name)
            target = DEST / name
            if args.check:
                if target.read_bytes() != data:
                    raise ValueError('Stale owned AES Crypt input: ' + name)
            else:
                target.write_bytes(data)
            identities[name] = dict(sha256=hashlib.sha256(data).hexdigest(), version=version,
                                    profile=profile, packed=len(data), decoded=len(payload))
    (report / 'm7-aes-reference.json').write_text(json.dumps(identities, indent=2) + '\n', encoding='utf-8')
    print(f'{len(cases)} owned AES Crypt inputs verified by aescry and Python/OpenSSL')


if __name__ == '__main__':
    try:
        main()
    except Exception as error:
        message = str(error)
        profile = os.environ.get('USERPROFILE')
        if profile:
            message = message.replace(profile, '<profile>').replace(profile.replace('\\', '/'), '<profile>')
        raise SystemExit(message) from None
