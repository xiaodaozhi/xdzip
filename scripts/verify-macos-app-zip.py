# scripts/verify-macos-app-zip.py

"""Validate the preserved signed application after extracting the published ZIP."""
import argparse
from contextlib import contextmanager
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
from macos_bundle import validate_optimized_bundle
from macos_launch_services import temporary_bundle_path, transient_registrations

ROOT = Path(__file__).resolve().parents[1]


@contextmanager
def temporary_extraction_directory(path):
    path = Path(path)
    if path.exists() or path.is_symlink() or not path.resolve().is_relative_to(ROOT/'build'):
        raise ValueError('Extraction verification needs a new directory under build/')
    path.mkdir(mode=0o700)
    identity = path.stat()
    try:
        yield path
    finally:
        if not path.parent.resolve().is_relative_to(ROOT/'build'):
            raise RuntimeError('Extraction verification directory escaped build/')
        if path.is_symlink() or not path.is_dir():
            raise RuntimeError('Extraction verification directory changed identity')
        current = path.stat()
        if (current.st_dev, current.st_ino) != (identity.st_dev, identity.st_ino):
            raise RuntimeError('Extraction verification directory changed identity')
        shutil.rmtree(path)


def bundle_manifest(app):
    """Inspect actual nodes without following framework or plugin symlinks."""
    if not app.is_dir() or app.is_symlink():
        raise ValueError('Application bundle root is not a real directory')
    entries = {}
    for directory, directories, files in os.walk(app, followlinks=False):
        for name in directories + files:
            path = Path(directory)/name
            relative = path.relative_to(app).as_posix()
            info = path.lstat()
            if stat.S_ISLNK(info.st_mode):
                entries[relative] = ('link', os.readlink(path))
            elif stat.S_ISDIR(info.st_mode):
                entries[relative] = ('directory',)
            elif stat.S_ISREG(info.st_mode):
                digest = hashlib.sha256()
                with path.open('rb') as source:
                    for chunk in iter(lambda: source.read(1024 * 1024), b''):
                        digest.update(chunk)
                entries[relative] = ('file', stat.S_IMODE(info.st_mode), digest.hexdigest())
            else:
                raise ValueError('Unexpected special object in application bundle')
    return entries


def compare_bundles(reference, extracted):
    expected = bundle_manifest(reference)
    actual = bundle_manifest(extracted)
    if not expected or expected.keys() != actual.keys():
        raise ValueError('Application bundle entry sets differ')
    for name, entry in expected.items():
        found = actual[name]
        if entry[0] == 'file':
            # The extraction contract retains safe Unix read/execute bits and
            # removes group/other writes and special permission bits.
            wanted = ('file', 0o600 | (entry[1] & 0o155), entry[2])
        else:
            wanted = entry
        if found != wanted:
            raise ValueError('Application bundle content, link or file mode differs: '+name)
    return len(actual)


def verify_product_extraction(verifier, binary, archive, reference, output, env):
    destination = output/'product-extraction'
    # The release suffix identifies a normal wrapper directory, not another .app.
    if not archive.name.lower().endswith('.app.zip') or len(archive.name) <= len('.app.zip'):
        raise ValueError('Expected a named .app.zip release archive')
    wrapper = destination/archive.name[:-len('.app.zip')]
    app = wrapper/'xdzip.app'
    probe_env = dict(env, XDZIP_PROBE_OPERATION='extract', XDZIP_PROBE_CREATE_FOLDER='1',
                     XDZIP_PROBE_EXTRACT_TARGET=str(destination), XDZIP_PROBE_IDLE_MS='0')
    text = verifier.run_product(binary, ['--desktop-probe', str(archive)], probe_env,
                                output, 'app-zip-product-extraction', timeout=180)
    events = [json.loads(line.removeprefix('XDZIP_PROBE ')) for line in text.splitlines()
              if line.startswith('XDZIP_PROBE ')]
    by = {event['event']: event for event in events}
    required = {'first_frame', 'open_ready', 'browse_finished', 'extract_requested',
                'extract_finished', 'quit_requested'}
    if (not required.issubset(by) or by['first_frame'].get('architecture') != 'arm64'
            or by['first_frame'].get('backend') != 'metal'
            or by['first_frame'].get('runtime_available') is not True
            or by['extract_finished'].get('status') != 'succeeded'):
        raise ValueError('Product did not complete native application ZIP extraction')
    if wrapper.is_symlink() or not wrapper.is_dir() or list(destination.iterdir()) != [wrapper]:
        raise ValueError('Application ZIP did not produce the expected normal wrapper folder')
    count = compare_bundles(reference, app)
    result = subprocess.run(['codesign', '--verify', '--deep', '--strict', str(app)],
                            capture_output=True, text=True)
    if result.returncode:
        raise ValueError('Extracted application failed deep strict signature verification')
    verifier.run_product(app/'Contents/MacOS/xdzip', ['--smoke-test'], env, output,
                         'extracted-app-cocoa-metal')
    frame = by['first_frame']
    product_identity = {key: frame[key] for key in ('version', 'build', 'revision', 'dirty')}
    return {'status': 'passed', 'product_identity': product_identity, 'compared_entries': count,
            'signature': 'deep-strict', 'extracted_app_smoke_test': 'passed'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--product', type=Path, help='Optional locally built xdzip executable')
    args = parser.parse_args()
    output = args.output.resolve()
    if sys.platform != 'darwin' or not output.is_relative_to(ROOT/'build') or output.exists():
        raise ValueError('Use native macOS and a new output directory under build/')
    output.mkdir(parents=True)
    app = output/'xdzip.app'
    destination = output/'product-extraction'
    # Remove registrations while even a wrongly named output bundle still exists.
    with temporary_bundle_path(app), temporary_extraction_directory(destination), transient_registrations(output):
        subprocess.run(['ditto', '-x', '-k', str(args.archive.resolve()), str(output)], check=True)
        validate_optimized_bundle(app)
        subprocess.run(['codesign', '--verify', '--deep', '--strict', str(app)], check=True)
        spec = importlib.util.spec_from_file_location('dmg_verifier', ROOT/'scripts/verify-macos-dmg.py')
        verifier = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(verifier)
        env = verifier.clean_environment(output)
        binary = app/'Contents/MacOS/xdzip'
        identity = subprocess.check_output([str(binary), '--version'], env=env, text=True).strip()
        subprocess.run([str(binary), '--smoke-test'], env=env, timeout=30, check=True)
        product = args.product.resolve(strict=True) if args.product else binary
        extraction = verify_product_extraction(verifier, product, args.archive.resolve(), app,
                                               output, env)
    (output/'verification.json').write_text(json.dumps(dict(status='passed', identity=identity,
        signing='ad-hoc', developer_id_signed=False, notarized=False,
        product_extraction=extraction), indent=2)+'\n')


if __name__ == '__main__': main()
