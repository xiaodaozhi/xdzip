"""Verify independent optical fixtures through the Rust codec and real host service.

Generated inputs and reports stay in build/. The official 7-Zip CLI is an
optional independent test reference. The host worker and the preserved official
reference worker are identified separately and must have different contents.
"""
import argparse
import gzip
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import unicodedata

ROOT = Path(__file__).resolve().parents[1]
HELLO = b'xdzip read-only archive fixture\n'
NESTED = 'Unicode payload: \u4e2d\u6587\n'.encode('utf-8')
UNICODE = {'hello.txt': HELLO, 'nested/data.txt': NESTED,
           'zero.txt': b'', '\u4e2d\u6587.txt': NESTED}
MAX_FIXTURE_BYTES = 8 * 1024**2
MAX_LARGE_FIXTURE_BYTES = 256 * 1024**2
MAX_UDFTOOLS_COMPRESSED_BYTES = 512 * 1024
REFERENCE_REJECTION_SHA256 = {
    'sparable-201.udf': 'e95b996d52d95e0fe3119ee52ba2163ccea9e74c265ed3b77b2c623efdcede6f',
    'vat150.udf': '3b79b05a3230b51404f3b66a3e36b8f382e9f4755cf4233a2dd6c317bba49728',
    'vat201.udf': 'ad3522d7f53ffd98ba74f7caac709600c8d2d8b403ed7a4321e2ecfaeda7d099',
}


def module(path):
    spec = importlib.util.spec_from_file_location(path.stem.replace('-', '_'), path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def invoke(command, env=None):
    return subprocess.run([str(arg) for arg in command], capture_output=True,
                          timeout=90, env=env)


def file_identity(path):
    size = 0
    digest = hashlib.sha256()
    with Path(path).open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            size += len(block)
            require(size <= MAX_LARGE_FIXTURE_BYTES, 'physical fixture size limit')
            digest.update(block)
    return size, digest.hexdigest()


def prepare(output):
    fixed = module(ROOT / 'scripts/prepare-cab-iso-fixtures.py').prepare(output / 'fixtures')
    cases = []

    def add(name, format, files, specials=()):
        cases.append(dict(name=name, path=str(fixed / name), format=format,
                          expected={key: value.hex() for key, value in files.items()},
                          directories=['EMPTY', 'NESTED'] if name == 'iso9660.iso' else ['empty', 'nested'],
                          specials=list(specials)))

    add('iso9660.iso', 'iso', {'HELLO.TXT': HELLO, 'NESTED/DATA.TXT': NESTED, 'ZERO.TXT': b''})
    add('joliet.iso', 'iso', UNICODE)
    add('rock-ridge.iso', 'iso', UNICODE, ['link'])
    for name in ('udf.iso', 'hybrid.iso'):
        add(name, 'udf', {**UNICODE, 'udf-only.txt': HELLO}, ['udf-link'])
    add('udf-regular.iso', 'udf', {**UNICODE, 'udf-only.txt': HELLO})
    generated = module(ROOT / 'tests/fixtures/optical/generate.py').generate(output / 'generated', fixed)
    for case in generated:
        case.setdefault('directories', [])
        case.setdefault('specials', [])
        if case['name'] == 'eltorito.iso':
            case['directories'] = ['[BOOT]']
        if case['name'] == 'udf-sparse.iso':
            case['expected'] = {key: value.hex() for key, value in
                                {**UNICODE, 'udf-only.txt': HELLO, 'hello.txt': bytes(len(HELLO))}.items()}
            case['directories'] = ['empty', 'nested']
        cases.append(case)
    cases.extend(prepare_imapi(output / 'imapi'))
    cases.extend(prepare_udftools(output / 'udftools'))
    cases.extend(prepare_makefs(output / 'makefs'))
    return cases


def expected_payloads(manifest, label):
    expected = {}
    for item in manifest['payloads']:
        if 'utf8' in item:
            data = item['utf8'].encode('utf-8')
        elif 'hex' in item:
            data = bytes.fromhex(item['hex'])
        else:
            recipe = item['recipe']
            first, last = recipe['byte_range_inclusive']
            repetitions = recipe['repetitions']
            require(0 <= first <= last <= 255 and 0 <= repetitions <= MAX_FIXTURE_BYTES // (last - first + 1),
                    label + ' payload recipe limit')
            data = bytes(range(first, last + 1)) * repetitions
        require(len(data) == item['bytes'] and len(data) <= MAX_FIXTURE_BYTES
                and hashlib.sha256(data).hexdigest() == item['sha256'], label + ' expected-payload identity mismatch')
        require(item['name'] not in expected, label + ' duplicate expected filename')
        expected[item['name']] = data.hex()
    return expected


def small_image_bytes(base, item, label, suffix):
    name = item['file']
    require(Path(name).name == name and '/' not in name and '\\' not in name and name.endswith(suffix),
            'unexpected ' + label + ' fixture filename')
    require(0 < item['compressed_bytes'] <= MAX_FIXTURE_BYTES
            and 0 < item['decoded_bytes'] <= MAX_FIXTURE_BYTES, label + ' fixture size limit')
    archive = base / name
    require(archive.stat().st_size <= MAX_FIXTURE_BYTES, label + ' compressed fixture size limit')
    with archive.open('rb') as source:
        stored = source.read(MAX_FIXTURE_BYTES + 1)
    require(len(stored) == item['compressed_bytes'] and len(stored) <= MAX_FIXTURE_BYTES
            and hashlib.sha256(stored).hexdigest() == item['sha256'], label + ' compressed fixture identity mismatch')
    with gzip.GzipFile(fileobj=io.BytesIO(stored)) as source:
        data = source.read(MAX_FIXTURE_BYTES + 1)
    require(len(data) == item['decoded_bytes'] and len(data) <= MAX_FIXTURE_BYTES
            and hashlib.sha256(data).hexdigest() == item['decoded_sha256'], label + ' decoded fixture identity mismatch')
    return name[:-3], data


def prepare_imapi(output):
    base = ROOT / 'tests/fixtures/optical/imapi'
    manifest = json.loads((base / 'sources.json').read_text(encoding='utf-8'))
    require(manifest['schema_version'] == 1, 'unsupported IMAPI fixture manifest')
    expected = expected_payloads(manifest, 'IMAPI')
    output.mkdir(parents=True, exist_ok=True)
    require(len(manifest['fixtures']) <= 16, 'IMAPI fixture count limit')
    cases = []
    for item in manifest['fixtures']:
        name, data = small_image_bytes(base, item, 'IMAPI', '.iso.gz')
        path = output / name
        path.write_bytes(data)
        cases.append(dict(name=name, path=str(path), format='udf', expected=expected,
                          directories=['nested'], specials=[], source='Microsoft Windows IMAPI2FS',
                          udf_revision=item['udf_revision'], fixture_partition_maps=item['partition_maps']))
    return cases


def prepare_makefs(output):
    base = ROOT / 'tests/fixtures/optical/makefs'
    manifest = json.loads((base / 'sources.json').read_text(encoding='utf-8'))
    require(manifest['schema_version'] == 1, 'unsupported makefs fixture manifest')
    expected = expected_payloads(manifest, 'makefs')
    output.mkdir(parents=True, exist_ok=True)
    require(0 < len(manifest['fixtures']) <= 16, 'makefs fixture count limit')
    cases = []
    for item in manifest['fixtures']:
        name, data = small_image_bytes(base, item, 'makefs', '.udf.gz')
        path = output / name
        path.write_bytes(data)
        cases.append(dict(name=name, path=str(path), format='udf', expected=expected,
                          directories=['nested'], specials=[], source='NetBSD makefs and udfclient',
                          udf_revision=item['udf_revision'], fixture_partition_maps=item['partition_maps'],
                          fixture_vat_version=item['vat_version'], fixture_tools=manifest['tools'],
                          fixture_validation_scope=manifest['validation_scope'],
                          fixture_tool_observation=item['known_tool_observation']))
    return cases


def stream_expand_fixture(archive, target, expected_size, expected_sha256):
    require(0 < expected_size <= MAX_LARGE_FIXTURE_BYTES, 'decoded optical fixture size limit')
    temporary = target.with_name(target.name + '.part')
    size = 0
    digest = hashlib.sha256()
    try:
        with gzip.open(archive, 'rb') as source, temporary.open('wb') as output:
            while True:
                block = source.read(min(1024 * 1024, MAX_LARGE_FIXTURE_BYTES - size + 1))
                if not block:
                    break
                size += len(block)
                require(size <= MAX_LARGE_FIXTURE_BYTES, 'decoded optical fixture size limit')
                digest.update(block)
                output.write(block)
        require(size == expected_size and digest.hexdigest() == expected_sha256,
                'decoded optical fixture identity mismatch')
        temporary.replace(target)
    finally:
        temporary.unlink(missing_ok=True)


def prepare_udftools(output):
    base = ROOT / 'tests/fixtures/optical/udftools'
    manifest = json.loads((base / 'sources.json').read_text(encoding='utf-8'))
    require(manifest['schema_version'] == 1, 'unsupported udftools fixture manifest')
    expected = expected_payloads(manifest, 'udftools')
    require(0 < len(manifest['fixtures']) <= 16, 'udftools fixture count limit')
    output.mkdir(parents=True, exist_ok=True)
    cases = []
    for item in manifest['fixtures']:
        name = item['file']
        require(Path(name).name == name and '/' not in name and '\\' not in name and name.endswith('.udf.gz'),
                'unexpected udftools fixture filename')
        archive = base / name
        require(archive.stat().st_size <= MAX_UDFTOOLS_COMPRESSED_BYTES, 'compressed udftools fixture size limit')
        require(file_identity(archive) == (item['compressed_bytes'], item['sha256']),
                'compressed udftools fixture identity mismatch')
        path = output / name[:-3]
        stream_expand_fixture(archive, path, item['decoded_bytes'], item['decoded_sha256'])
        cases.append(dict(name=name[:-3], path=str(path), format='udf', expected=expected,
                          directories=['nested'], specials=[], source='udftools mkudffs and udfclient',
                          udf_revision=item['udf_revision'], fixture_partition_maps=item['partition_maps'],
                          fixture_system_streams=item['system_streams']))
    return cases


def codec_case(binary, case):
    command = [binary, case['path'], case['format'], '--guards']
    if case.get('virtual'):
        command.append('--virtual')
    result = invoke(command)
    require(result.returncode == 0, case['name'] + ': codec scan/decode or guards failed: '
            + result.stderr.decode('utf-8', 'replace')[:300])
    report = json.loads(result.stdout)
    require(report['source_kind'] == ('segmented_virtual' if case.get('virtual') else 'physical_file'),
            case['name'] + ': unexpected source-stream kind')
    rows = report['rows']
    names = [bytes.fromhex(row['name_hex']).decode('utf-8') for row in rows]
    require(len(names) == len(set(names)), case['name'] + ': duplicate physical names')
    actual = {}
    specials = set()
    directories = set()
    for name, row in zip(names, rows):
        require(row['format'] == case['format'].upper(), case['name'] + ': wrong format')
        if row['directory']:
            directories.add(name.rstrip('/'))
        elif row['link'] or row['warning']:
            specials.add(name)
            require(row.get('decode_error') == 'unsupported_feature', case['name'] + ': special item decoded')
        else:
            data = bytes.fromhex(row['hex'])
            require(len(data) == row['size'], case['name'] + ': incorrect decoded length')
            actual[name] = data.hex()
            if case.get('expected_raw_names'):
                require(row['raw_name_hex'] == case['expected_raw_names'].get(name),
                        case['name'] + ': legacy raw-name identity was lost')
                require(row['decoded_name'] == name and row['name_encoding'].startswith('Legacy('),
                        case['name'] + ': legacy filename was incorrectly treated as declared Unicode')
    require(actual == case['expected'], case['name'] + ': codec payload or name mismatch: '
            + repr(sorted(set(actual) ^ set(case['expected']))))
    require(specials == set(case['specials']), case['name'] + ': special-record identity mismatch')
    # Synthetic boot-file parents need not be physical ISO directory records.
    require(directories == set(case['directories']), case['name'] + ': directory mismatch')
    if case.get('virtual'):
        require(report['source_length'] > 1 << 32, 'virtual source is not larger than 4 GiB')
        require(report['maximum_read_offset'] > 1 << 32, 'payload did not cross the 4 GiB offset')
        require(report['bytes_read'] < 8 * 1024**2, 'virtual source was read sequentially')
    details = dict(case=case['name'], passed=True, files=len(actual), special_records=len(specials),
                   source_bytes=report['source_length'], actual_input_bytes=report['bytes_read'],
                   maximum_read_offset=report['maximum_read_offset'], source_kind=report['source_kind'],
                   guards=report['guards'])
    for key in ('source', 'udf_revision', 'fixture_partition_maps', 'fixture_system_streams',
                'fixture_vat_version', 'fixture_tools', 'fixture_validation_scope',
                'fixture_tool_observation'):
        if key in case:
            details[key] = case[key]
    return details


def normalize(name):
    return unicodedata.normalize('NFC', name)


def service_case(binary, case, owned, env):
    if case.get('virtual'):
        return None
    target = owned / (case['name'] + '.out')
    target.mkdir()
    # Inputs are owned copies, so service checks do not expose fixtures to mutation.
    source = owned / case['name']
    shutil.copyfile(case['path'], source)
    before = file_identity(source)
    result = invoke([binary, source, target, '--generic'], env=env)
    require(file_identity(source) == before, case['name'] + ': host changed source')
    if case['specials']:
        require(result.returncode != 0 and b'unsupported_link' in result.stderr,
                case['name'] + ': host did not explicitly reject links')
        require(not list(target.iterdir()), case['name'] + ': rejected host task published output')
        return dict(case=case['name'], passed=True, result='unsupported_link', published_files=0)
    require(result.returncode == 0, case['name'] + ': host extraction failed: '
            + result.stderr.decode('utf-8', 'replace')[:300])
    paths = list(target.rglob('*'))
    require(not any(path.is_symlink() for path in paths), case['name'] + ': host created a link')
    actual = {normalize(path.relative_to(target).as_posix()): path.read_bytes().hex()
              for path in paths if path.is_file()}
    expected = {normalize(key): value for key, value in case['expected'].items()}
    require(actual == expected, case['name'] + ': host names or bytes mismatch')
    directories = {normalize(path.relative_to(target).as_posix()) for path in paths if path.is_dir()}
    wanted_dirs = {normalize(name) for name in case['directories']}
    for name in case['expected']:
        wanted_dirs.update(normalize(str(parent).replace('\\', '/'))
                           for parent in Path(name).parents if str(parent) != '.')
    require(directories == wanted_dirs, case['name'] + ': host directory mismatch')
    summary = json.loads(result.stdout)
    size = sum(len(bytes.fromhex(value)) for value in expected.values())
    require(summary['files'] == len(expected) and summary['bytes'] == size,
            case['name'] + ': host committed counters mismatch')
    return dict(case=case['name'], passed=True, files=summary['files'], bytes=summary['bytes'],
                directories=summary['directories'])


def reference_case(reference, case):
    if case.get('virtual'):
        return None
    for name, expected_hex in case['expected'].items():
        result = invoke([reference, 'x', '-so', '-bd', '-y', case['path'], name])
        require(result.returncode == 0, case['name'] + ': independent reference rejected file ' + name)
        require(result.stdout.hex() == expected_hex, case['name'] + ': independent reference payload mismatch ' + name)
    return dict(case=case['name'], passed=True, files=len(case['expected']))


def reference_worker_case(worker, case):
    if case.get('virtual'):
        return None
    native = module(ROOT / 'scripts/check-native-codecs.py')
    data = Path(case['path']).read_bytes()
    kwargs = {'format_id': 9 if case['format'] == 'iso' else 10}
    try:
        rows, _ = native.run_worker(worker, [data], **kwargs)
    except native.WorkerError as error:
        if case['name'] == 'udf-sparse.iso' and str(error) == 'native_error_3':
            return dict(case=case['name'], status='unsupported_by_reference',
                        error='native_error_3', reference='official_native_worker')
        if (str(error) == 'native_error_2' and case['name'] in REFERENCE_REJECTION_SHA256
                and hashlib.sha256(data).hexdigest() == REFERENCE_REJECTION_SHA256[case['name']]):
            return dict(case=case['name'], status='reference_rejected', error='native_error_2',
                        reference='official_native_worker',
                        independent_authoring_readback=case['source'])
        raise AssertionError(case['name'] + ': reference worker rejected owned fixture') from error
    actual = {}
    specials = set()
    names_by_raw = {bytes.fromhex(value): key for key, value in case.get('expected_raw_names', {}).items()}
    for row in rows:
        if row['flags'] & 1:
            continue
        name = names_by_raw.get(row['raw_name'], row['name'].replace('\\', '/'))
        if row['flags'] & 12:
            specials.add(name)
            try:
                native.run_worker(worker, [data], selected=row['index'], **kwargs)
            except native.WorkerError:
                continue
            raise AssertionError(case['name'] + ': reference decoded a special item')
        try:
            _, payload = native.run_worker(worker, [data], selected=row['index'], **kwargs)
        except native.WorkerError as error:
            if case['name'] == 'udf-sparse.iso' and str(error) == 'native_error_3':
                return dict(case=case['name'], status='unsupported_by_reference',
                            error='native_error_3', reference='official_native_worker')
            raise AssertionError(case['name'] + ': reference worker failed to decode owned file') from error
        actual[name] = payload.hex()
    require(actual == case['expected'] and specials == set(case['specials']),
            case['name'] + ': reference worker bytes or names mismatch')
    return dict(case=case['name'], passed=True, files=len(actual), reference='official_native_worker')


def negative_cases(binary, extract_binary, cases, output, owned, env):
    results = []
    fixed = {case['name']: case for case in cases}
    mutations = []
    for name in ('iso9660.iso', 'udf.iso', 'hybrid.iso'):
        case = fixed[name]
        data = Path(case['path']).read_bytes()
        # Retain the optical recognition descriptors. Cutting ISO below sector
        # 16 would leave only zeros, which are independently a valid empty TAR.
        truncated = data[:-2048] if case['format'] == 'iso' else data[:len(data) // 2]
        mutations.append((name + '-truncated', case['format'], truncated, 'invalid_archive'))
    bad_iso_size = bytearray(Path(fixed['iso9660.iso']['path']).read_bytes())
    record = bad_iso_size.index(b'HELLO.TXT;1') - 33
    struct.pack_into('<I', bad_iso_size, record + 10, 0xffffffff)
    mutations.append(('iso-both-endian-size-mismatch', 'iso', bytes(bad_iso_size), 'invalid_archive'))
    bad_udf_size = bytearray(Path(fixed['udf-regular.iso']['path']).read_bytes())
    changed = False
    for at in range(0, len(bad_udf_size), 2048):
        if (struct.unpack_from('<H', bad_udf_size, at)[0] == 261
                and bad_udf_size[at + 27] == 5
                and struct.unpack_from('<Q', bad_udf_size, at + 56)[0] == len(HELLO)):
            struct.pack_into('<Q', bad_udf_size, at + 56, len(HELLO) + 1)
            module(ROOT / 'tests/fixtures/optical/generate.py').retag(bad_udf_size, at)
            changed = True
            break
    require(changed, 'bad-size UDF fixture construction')
    mutations.append(('udf-information-length-mismatch', 'udf', bytes(bad_udf_size), 'invalid_archive'))
    hybrid = bytearray(Path(fixed['hybrid.iso']['path']).read_bytes())
    # Keep NSR02/03 recognition intact but invalidate every anchor descriptor.
    # A product that silently falls back to the valid ISO namespace would succeed.
    anchors = 0
    for at in range(0, len(hybrid), 2048):
        if hybrid[at:at + 2] == b'\x02\x00':
            hybrid[at + 4] ^= 1
            anchors += 1
    require(anchors >= 1 and (b'NSR02' in hybrid or b'NSR03' in hybrid), 'bad-UDF fixture construction')
    mutations.append(('hybrid-corrupt-udf-no-fallback', 'udf', bytes(hybrid), 'invalid_archive'))
    for name, format, data, expected_error in mutations:
        path = output / 'generated' / (name + '.iso')
        path.write_bytes(data)
        result = invoke([binary, path, format])
        require(result.returncode != 0 and expected_error.encode() in result.stderr,
                name + ': codec accepted damaged input or returned an unexpected error')
        if extract_binary:
            source = owned / path.name
            source.write_bytes(data)
            target = owned / (name + '.out')
            target.mkdir()
            host = invoke([extract_binary, source, target, '--generic'], env=env)
            require(host.returncode != 0 and expected_error.encode() in host.stderr,
                    name + ': host accepted damaged input or fell back to ISO')
            require(not list(target.iterdir()) and source.read_bytes() == data,
                    name + ': failed host task modified source or target')
        results.append(dict(case=name, passed=True, error=expected_error,
                            host_service_checked=bool(extract_binary)))
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, help='non-shipping xdzip-optical-probe')
    parser.add_argument('--extract-binary', type=Path, help='real xdzip-m3-probe host service')
    parser.add_argument('--worker', type=Path, help='migrated Rust optical worker; defaults to XDZIP_NATIVE_CODEC')
    parser.add_argument('--reference', type=Path, help='optional pinned full official 7z/7zz; reduced 7za lacks optical handlers')
    parser.add_argument('--reference-worker', type=Path, help='optional existing official optical worker, only as independent reference')
    parser.add_argument('--output', type=Path, default=ROOT / 'build/review/optical-interop')
    parser.add_argument('--prepare-only', action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    require(output.is_relative_to(ROOT / 'build') and output != ROOT / 'build', 'outputs belong below build/')
    output.mkdir(parents=True, exist_ok=True)
    cases = prepare(output)
    if args.prepare_only:
        print('Prepared owned optical fixtures; codec and host checks were not run')
        return
    require(args.binary is not None, '--binary is required for verification')
    binary = args.binary.resolve()
    extract_binary = args.extract_binary.resolve() if args.extract_binary else None
    reference = args.reference.resolve() if args.reference else None
    reference_worker = args.reference_worker.resolve() if args.reference_worker else None
    worker_value = args.worker or os.environ.get('XDZIP_NATIVE_CODEC')
    worker = Path(worker_value).resolve() if worker_value else None
    report = dict(passed=False, codec=[], host_service=[], reference=[], negative=[],
                  fixture_sources_sha256=hashlib.sha256((ROOT / 'tests/fixtures/cab-iso/sources.json').read_bytes()).hexdigest(),
                  imapi_sources_sha256=hashlib.sha256((ROOT / 'tests/fixtures/optical/imapi/sources.json').read_bytes()).hexdigest(),
                  udftools_sources_sha256=hashlib.sha256((ROOT / 'tests/fixtures/optical/udftools/sources.json').read_bytes()).hexdigest(),
                  makefs_sources_sha256=hashlib.sha256((ROOT / 'tests/fixtures/optical/makefs/sources.json').read_bytes()).hexdigest(),
                  limits=['Native desktop/Qt acceptance is separate from these codec and host-service checks.',
                          'Virtual large-source evidence does not allocate or extract a physical 4 GiB file.',
                          'ISO/UDF payload byte matching does not imply embedded checksum authentication.',
                          'Known-payload readability and the exercised host behaviors do not prove full optical-media conformance; per-fixture generator observations, including the VAT 1.50 udfinfo warning, remain in the codec case metadata.'])
    env = os.environ.copy()
    if reference_worker:
        report['reference_worker_sha256'] = hashlib.sha256(reference_worker.read_bytes()).hexdigest()
    if reference:
        report['reference_binary_sha256'] = hashlib.sha256(reference.read_bytes()).hexdigest()
    if worker:
        require(worker.is_file(), 'migrated host worker is missing')
        env['XDZIP_NATIVE_CODEC'] = str(worker)
        digest = hashlib.sha256(worker.read_bytes()).hexdigest()
        report['host_worker_sha256'] = digest
        if reference_worker:
            require(digest != hashlib.sha256(reference_worker.read_bytes()).hexdigest(),
                    'host worker must differ from the preserved official optical reference')
    try:
        if extract_binary:
            require(worker is not None, 'host-service checks require --worker or XDZIP_NATIVE_CODEC')
            base = invoke([extract_binary, '--native-temp-base'], env=env)
            require(base.returncode == 0, 'host temporary-directory query failed')
            native_base = base.stdout.decode('utf-8').strip()
        else:
            native_base = output / 'scratch'
            native_base.mkdir(exist_ok=True)
            report['limits'].append('Host service extraction, host cancellation and source identity were not checked.')
        with tempfile.TemporaryDirectory(prefix='xdzip-optical-interop-', dir=native_base) as temporary:
            owned = Path(temporary)
            for case in cases:
                report['codec'].append(codec_case(binary, case))
                if reference:
                    record = reference_case(reference, case)
                    if record:
                        report['reference'].append(record)
                if reference_worker:
                    record = reference_worker_case(reference_worker, case)
                    if record:
                        report['reference'].append(record)
                if extract_binary:
                    record = service_case(extract_binary, case, owned, env)
                    if record:
                        report['host_service'].append(record)
            report['negative'] = negative_cases(binary, extract_binary, cases, output, owned, env)
        report['passed'] = True
        report['host_service_checked'] = bool(extract_binary)
        if not reference and not reference_worker:
            report['limits'].append('Optional official-tool comparison was not run; fixed pycdlib vectors remain independent inputs.')
        if any(record.get('status') == 'unsupported_by_reference' for record in report['reference']):
            report['limits'].append('The existing official worker rejects the UDF sparse test vector as unsupported; it is not counted as a reference pass.')
        rejected = [record['case'] for record in report['reference'] if record.get('status') == 'reference_rejected']
        if rejected:
            report['limits'].append('The preserved official worker rejects these fixed independent images: '
                                    + ', '.join(rejected) + '. Their independent authoring/readback uses the recorded tools; '
                                    + 'these records are not counted as official-worker reference passes.')
        print('Optical interoperability passed: %d codec, %d host-service, %d reference passes, %d damaged-input cases'
              % (len(report['codec']), len(report['host_service']),
                 sum(record.get('passed') is True for record in report['reference']), len(report['negative'])))
    except Exception as error:
        # Only stable errors and owned logical case names enter persistent evidence.
        report['failure'] = str(error)
        raise
    finally:
        (output / 'optical-interop.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)
                                                  + '\n', encoding='utf-8')


if __name__ == '__main__':
    main()
