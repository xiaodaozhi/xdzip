"""Match protected update keys and sign only prepared, disposable upgrade fixtures.

This entry point never builds, generates private keys, publishes, or accepts a
manifest pathname. The workflow prepares every input without signing authority;
only the already built update tool receives the one secret needed by a command.
"""

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import plistlib
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import zipfile
import zlib

from release_protocol import validate
from release_updates import appcast_bytes


ROOT = Path(__file__).resolve().parents[1]
PRIVATE = ('XDZIP_UPDATE_SIGNING_KEY_HEX', 'XDZIP_SPARKLE_SIGNING_KEY_HEX')
KEY_ID = 'xdzip-release-1'
VERSION = '0.2.0-alpha.2'
BUILD = 201
HEX = re.compile(r'[0-9a-f]{64}')
COMMIT = re.compile(r'[0-9a-f]{40}')
PREPARED = 'production-update-prepared.json'
PUBLIC_FILES = ('update-trust.json', 'sparkle-update-key.json')


class AdmissionError(Exception):
    """A fixed, public error code; never include subprocess output or paths."""


def require(condition, code):
    if not condition:
        raise AdmissionError(code)


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            result.update(block)
    return result.hexdigest()


def regular(path, limit=None):
    metadata = path.lstat()
    require(stat.S_ISREG(metadata.st_mode) and not path.is_symlink(), 'invalid_regular_input')
    require(metadata.st_nlink == 1, 'hardlinked_input_rejected')
    if limit is not None:
        require(metadata.st_size <= limit, 'input_size_limit')
    return path


def owned(path, parent):
    """Reject both final symlinks and symlinked directory components."""
    path = Path(os.path.abspath(path))
    parent = Path(os.path.abspath(parent))
    require(path != parent and path.is_relative_to(parent), 'fixture_path_outside_build')
    cursor = path
    while cursor != parent:
        require(not cursor.is_symlink(), 'fixture_symlink_rejected')
        cursor = cursor.parent
    require(not parent.is_symlink(), 'fixture_symlink_rejected')
    return path


def relative(directory, value):
    require(isinstance(value, str) and '\\' not in value, 'invalid_artifact_path')
    pure = PurePosixPath(value)
    require(not pure.is_absolute() and pure.as_posix() == value and
            all(part not in ('', '.', '..') for part in pure.parts) and ':' not in value,
            'invalid_artifact_path')
    return owned(directory.joinpath(*pure.parts), directory)


def load(path, limit=256 * 1024):
    # Duplicate keys can otherwise change what the preparer and signer see.
    def pairs(items):
        value = {}
        for key, item in items:
            require(key not in value, 'duplicate_json_key')
            value[key] = item
        return value
    return json.loads(regular(path, limit).read_bytes(), object_pairs_hook=pairs)


def clean_environment(secret=None):
    # No GH token, dynamic-loader override, compiler hook or second private key is
    # inherited by the tool. These native OS locations are not signing inputs.
    allowed = ('SystemRoot', 'SYSTEMROOT', 'WINDIR', 'TEMP', 'TMP', 'TMPDIR')
    value = {name: os.environ[name] for name in allowed if name in os.environ}
    value['PATH'] = os.defpath
    if secret:
        key = os.environ.get(secret, '')
        require(bool(HEX.fullmatch(key)), 'signing_key_missing_or_invalid')
        value[secret] = key
    return value


def source_commit():
    git = shutil.which('git')
    require(git is not None, 'source_identity_unavailable')
    result = subprocess.run([git, '-C', str(ROOT), 'rev-parse', 'HEAD'],
                            env=clean_environment(), capture_output=True, timeout=30)
    require(result.returncode == 0, 'source_identity_unavailable')
    commit = result.stdout.decode('ascii').strip()
    require(bool(COMMIT.fullmatch(commit)), 'source_identity_invalid')
    if os.environ.get('GITHUB_ACTIONS') == 'true':
        require(os.environ.get('GITHUB_SHA') == commit, 'workflow_source_mismatch')
    return commit


def public_resources():
    ring = load(ROOT / 'resources/update-trust.json', 65536)
    sparkle = load(ROOT / 'resources/sparkle-update-key.json', 4096)
    require(set(ring) == {'schema_version', 'keys'} and ring['schema_version'] == 1 and
            isinstance(ring['keys'], list) and len(ring['keys']) == 1,
            'production_keyring_shape')
    key = ring['keys'][0]
    require(set(key) == {'key_id', 'public_key'} and key['key_id'] == KEY_ID and
            isinstance(key['public_key'], str) and bool(HEX.fullmatch(key['public_key'])),
            'production_keyring_shape')
    require(set(sparkle) == {'schema_version', 'public_key'} and sparkle['schema_version'] == 1 and
            isinstance(sparkle['public_key'], str) and bool(HEX.fullmatch(sparkle['public_key'])),
            'production_sparkle_shape')
    require(key['public_key'] != sparkle['public_key'], 'independent_keys_required')
    require(os.environ.get('XDZIP_UPDATE_KEY_ID') == KEY_ID, 'production_key_id_mismatch')
    return ring, sparkle


class Tool:
    def __init__(self, path, expected):
        self.path = path
        self.expected = expected

    def run(self, arguments, secret=None):
        require(digest(regular(self.path, 64 * 1024 * 1024)) == self.expected, 'signer_changed')
        result = subprocess.run([str(self.path), *map(str, arguments)],
                                env=clean_environment(secret), capture_output=True, timeout=300)
        # A failing process may print its environment. Never echo either stream.
        require(result.returncode == 0, 'compiled_signing_tool_failed')


def match_keys(tool, parent):
    ring, sparkle = public_resources()
    with tempfile.TemporaryDirectory(prefix='public-key-match-', dir=parent) as temporary:
        directory = Path(temporary)
        general_path = directory / 'update-trust.json'
        sparkle_path = directory / 'sparkle-update-key.json'
        tool.run(['public-key', '--key-id', KEY_ID, '--output', general_path], PRIVATE[0])
        tool.run(['sparkle-public-key', '--output', sparkle_path], PRIVATE[1])
        require(load(general_path, 65536) == ring, 'production_update_key_mismatch')
        require(load(sparkle_path, 4096) == sparkle, 'production_sparkle_key_mismatch')
    return {'key_id': KEY_ID, 'general_public_key_matched': True,
            'sparkle_public_key_matched': True,
            'general_public_resource_sha256': digest(ROOT / 'resources/update-trust.json'),
            'sparkle_public_resource_sha256': digest(ROOT / 'resources/sparkle-update-key.json')}


def artifact(directory, record, name):
    require(name in record['artifacts'], 'required_prepared_artifact_missing')
    path = relative(directory, name)
    expected = record['artifacts'][name]
    require(isinstance(expected, dict) and set(expected) == {'bytes', 'sha256'} and
            type(expected['bytes']) is int and 0 < expected['bytes'] <= 2 * 1024 ** 3 and
            isinstance(expected['sha256'], str) and bool(HEX.fullmatch(expected['sha256'])),
            'invalid_artifact_record')
    regular(path, 2 * 1024 ** 3)
    require(path.stat().st_size == expected['bytes'] and digest(path) == expected['sha256'],
            'prepared_artifact_changed')
    return path


def product_identity(value, label):
    require(isinstance(value, dict) and isinstance(value.get('identity'), dict), 'product_identity_missing')
    identity = value['identity']
    commit = value.get('commit', '')
    require(isinstance(commit, str) and bool(COMMIT.fullmatch(commit)), 'product_commit_invalid')
    number, build = (1, 200) if label == 'old' else (2, BUILD)
    require(identity.get('version') == '0.2.0-alpha.' + str(number) and
            type(identity.get('build')) is int and identity['build'] == build and
            identity.get('revision') == commit[:12] and identity.get('dirty') is False,
            'product_identity_mismatch')
    return commit


def prepared_inputs(directory):
    record = load(directory / PREPARED)
    require(isinstance(record, dict) and set(record) ==
            {'schema_version', 'platform', 'source_commit', 'tool', 'artifacts', 'products'} and
            record['schema_version'] == 1, 'invalid_prepared_record')
    require(record['source_commit'] == source_commit(), 'prepared_source_mismatch')
    platform = record['platform']
    require(platform in ('windows', 'linux', 'macos'), 'unsupported_fixture_platform')
    expected_tool = 'signing/xdzip-update-tool' + ('.exe' if platform == 'windows' else '')
    require(isinstance(record['tool'], dict) and set(record['tool']) == {'path', 'sha256'} and
            record['tool']['path'] == expected_tool, 'unexpected_signing_tool')
    require(isinstance(record['artifacts'], dict) and 5 <= len(record['artifacts']) <= 32,
            'invalid_artifact_inventory')
    # Verify the complete preparer inventory, then independently bind mandatory
    # public roots and signing tool. The inventory never chooses signing inputs.
    for name in record['artifacts']:
        artifact(directory, record, name)
    tool_path = artifact(directory, record, expected_tool)
    require(digest(tool_path) == record['tool']['sha256'], 'prepared_signer_mismatch')
    for name in PUBLIC_FILES:
        path = artifact(directory, record, 'resources/' + name)
        require(path.read_bytes() == regular(ROOT / 'resources' / name, 65536).read_bytes(),
                'prepared_public_resource_mismatch')
    labels = {'old', 'new'} if platform == 'macos' else {'old', 'new', 'failure'}
    require(isinstance(record['products'], dict) and set(record['products']) == labels,
            'prepared_products_mismatch')
    commits = {label: product_identity(value, label) for label, value in record['products'].items()}
    if platform != 'macos':
        require(commits['new'] == commits['failure'], 'failure_product_identity_mismatch')
    require(commits['old'] != commits['new'], 'distinct_upgrade_identity_required')
    return record, Tool(tool_path, record['tool']['sha256'])


def unicode_extra_name(entry, raw):
    """Check the optional Info-ZIP Unicode path against the complete raw name."""
    extra = entry.extra
    seen = False
    while extra:
        require(len(extra) >= 4, 'mac_archive_extra_field_invalid')
        kind = int.from_bytes(extra[:2], 'little')
        size = int.from_bytes(extra[2:4], 'little')
        require(len(extra) >= size + 4, 'mac_archive_extra_field_invalid')
        data, extra = extra[4:4 + size], extra[4 + size:]
        if kind != 0x7075:
            continue
        require(not seen and len(data) >= 5 and data[0] == 1,
                'mac_archive_unicode_path_invalid')
        seen = True
        require(int.from_bytes(data[1:5], 'little') == zlib.crc32(raw),
                'mac_archive_unicode_path_crc_mismatch')
        try:
            path = data[5:].decode('utf-8', errors='strict')
        except UnicodeError:
            raise AdmissionError('mac_archive_unicode_path_invalid') from None
        # A Unicode extra field cannot redirect an unrelated raw archive member
        # to a trusted app path, even if an extractor honors that extra field.
        require(path.encode('utf-8') == raw, 'mac_archive_unicode_path_mismatch')
    return seen


def mac_archive_members(archive, expected_names):
    """Match fixed complete UTF-8 names, including legacy unset UTF-8 flags.

    Some producers store UTF-8 bytes without bit 11. Python then decodes those
    bytes as CP437. Reversing that documented decoding preserves byte identity;
    it does not guess an encoding or accept a suffix/alternate app directory.
    """
    entries = archive.infolist()
    require(len(entries) <= 100000, 'mac_archive_inventory_invalid')
    selected = {name: [] for name in expected_names}
    raw_names = set()
    for entry in entries:
        require('\x00' not in entry.orig_filename and '\x00' not in entry.filename,
                'mac_archive_filename_invalid')
        try:
            raw = entry.orig_filename.encode('utf-8' if entry.flag_bits & 0x800 else 'cp437', errors='strict')
        except UnicodeError:
            raise AdmissionError('mac_archive_filename_invalid') from None
        require(raw not in raw_names, 'mac_archive_inventory_invalid')
        raw_names.add(raw)
        has_unicode_extra = unicode_extra_name(entry, raw)
        # Newer Python versions apply a verified Unicode extra path to filename,
        # while orig_filename still preserves the central-directory decoding.
        require(entry.filename == entry.orig_filename or
                (has_unicode_extra and entry.filename.encode('utf-8') == raw),
                'mac_archive_filename_invalid')
        for name in expected_names:
            if raw == name.encode('utf-8'):
                method = 'utf8-flag' if entry.flag_bits & 0x800 else 'exact-utf8-bytes-without-flag'
                selected[name].append((entry, method))
    for matches in selected.values():
        require(len(matches) == 1, 'mac_archive_identity_missing_or_ambiguous')
    return {name: matches[0] for name, matches in selected.items()}


def inspect_mac_products(directory, record, package, inspection):
    expected_public = base64.b64encode(bytes.fromhex(public_resources()[1]['public_key'])).decode('ascii')
    name = None
    for label in ('old', 'new'):
        inspection['stage'] = 'mac_product_plist'
        value = record['products'][label]
        app = value.get('app', '')
        match = re.fullmatch(label + r'/((xdzip-owned-update-更新-)[0-9a-f]{32}\.app)', app)
        require(match is not None, 'mac_product_path_mismatch')
        if name is None:
            name = match[1]
        require(match[1] == name, 'mac_product_name_mismatch')
        plist = regular(relative(directory, app + '/Contents/Info.plist'), 256 * 1024)
        binary = regular(relative(directory, app + '/Contents/MacOS/xdzip'), 256 * 1024 * 1024)
        info = plistlib.loads(plist.read_bytes())
        require(digest(plist) == value.get('plist_sha256') and
                digest(binary) == value.get('binary_sha256'), 'mac_product_changed')
        require(value.get('SUPublicEDKey') == expected_public and
                info.get('SUPublicEDKey') == expected_public, 'mac_embedded_sparkle_key_mismatch')
        require(info.get('CFBundleIdentifier') == 'org.xdzip.desktop' and
                info.get('CFBundleShortVersionString') == '0.2.0' and
                str(info.get('CFBundleVersion')) == str(value['identity']['build']) and
                info.get('XdzipUpdateInstallable') is True,
                'mac_bundle_identity_mismatch')
    expected = {
        name + '/Contents/Info.plist': (record['products']['new']['plist_sha256'], 256 * 1024),
        name + '/Contents/MacOS/xdzip': (record['products']['new']['binary_sha256'], 256 * 1024 * 1024),
    }
    inspection['expected_names'] = list(expected)
    inspection['stage'] = 'mac_archive_open'
    with zipfile.ZipFile(package) as archive:
        inspection['stage'] = 'mac_archive_identity_lookup'
        inspection['archive_member_count'] = len(archive.infolist())
        selected = mac_archive_members(archive, expected)
        for expected_name, (expected_hash, limit) in expected.items():
            entry, method = selected[expected_name]
            member = {'expected_name': expected_name, 'zip_flags': entry.flag_bits,
                      'matching_method': method, 'bytes': entry.file_size}
            inspection['members'].append(member)
            require(entry.file_size <= limit, 'mac_archive_identity_size_limit')
            inspection['stage'] = 'mac_archive_identity_read'
            result = hashlib.sha256()
            with archive.open(entry) as stream:
                for block in iter(lambda: stream.read(1024 * 1024), b''):
                    result.update(block)
            member['sha256'] = result.hexdigest()
            require(result.hexdigest() == expected_hash, 'mac_archive_product_mismatch')
    inspection['stage'] = 'mac_archive_inspection_complete'


def mac_products(directory, record, package):
    """Bind actual apps and archive bytes, retaining only public diagnostics."""
    inspection = {'schema_version': 1, 'source_commit': record['source_commit'],
                  'stage': 'mac_product_identity', 'status': 'inspecting', 'members': []}
    failure = None
    try:
        inspection['stage'] = 'mac_archive_package_binding'
        inspection['package_sha256'] = digest(regular(package, 2 * 1024 ** 3))
        inspect_mac_products(directory, record, package, inspection)
        inspection['status'] = 'passed'
    except Exception as error:
        code = str(error) if isinstance(error, AdmissionError) else inspection['stage'] + '_failed'
        inspection.update(status='failed', error=code)
        failure = AdmissionError(code)
    try:
        destination = owned(directory / 'production-archive-inspection.json', directory)
        with destination.open('x', encoding='utf-8') as stream:
            json.dump(inspection, stream, sort_keys=True)
            stream.write('\n')
    except Exception:
        if failure is None:
            failure = AdmissionError('mac_archive_inspection_record_failed')
    if failure is not None:
        raise failure from None


def candidate(directory, record, folder):
    platform = record['platform']
    manifest_path = artifact(directory, record, folder + '/release.json')
    manifest = load(manifest_path)
    validate(manifest)
    require(set(manifest) == {'schema_version', 'product', 'version', 'channel', 'build', 'tag',
                            'commit', 'repository', 'assets'} |
            ({'macos_appcast_url'} if platform == 'macos' else set()), 'unexpected_manifest_fields')
    require(manifest['version'] == VERSION and manifest['build'] == BUILD and
            manifest['commit'] == record['products']['new']['commit'] and
            manifest['repository'] == 'xiaodaozhi/xdzip', 'fixture_manifest_identity_mismatch')
    kind = {'windows': 'exe', 'linux': 'deb', 'macos': 'app.zip'}[platform]
    selected = next(asset for asset in manifest['assets'] if asset['platform'] == platform and
                    asset['package_type'] == kind)
    package = artifact(directory, record, folder + '/' + selected['filename'])
    require(selected['bytes'] == package.stat().st_size and selected['sha256'] == digest(package),
            'fixture_manifest_package_mismatch')
    # All other targets are explicit inert placeholders, never an unrelated
    # release asset smuggled through this isolated signing authorization.
    for asset in manifest['assets']:
        require(set(asset) == {'filename', 'platform', 'architecture', 'package_type',
                              'minimum_system', 'bytes', 'sha256', 'url'}, 'unexpected_asset_fields')
        if asset is not selected:
            require(asset['bytes'] == 1 and asset['sha256'] == hashlib.sha256(b'x').hexdigest(),
                    'fixture_placeholder_mismatch')
    return manifest_path, manifest, package


def sign_fixture(directory):
    prepared_sha256 = digest(regular(directory / PREPARED, 256 * 1024))
    record, tool = prepared_inputs(directory)
    platform = record['platform']
    folders = ('owned',) if platform == 'macos' else ('new', 'failure')
    candidates = [(folder, *candidate(directory, record, folder)) for folder in folders]
    outputs = [directory / folder / 'release.json.sig' for folder in folders]
    if platform == 'macos':
        outputs += [directory / 'owned' / name for name in
                    ('macos-update.json', 'appcast.xml', 'appcast-unsigned.xml')]
        outputs += [directory / 'production-archive-inspection.json']
    outputs += [directory / 'production-signing.json']
    require(all(not path.exists() and not path.is_symlink() for path in outputs), 'signing_output_exists')
    if platform == 'macos':
        mac_products(directory, record, candidates[0][3])
    matched = match_keys(tool, directory)
    ring = directory / 'resources/update-trust.json'
    sparkle = directory / 'resources/sparkle-update-key.json'
    signed = []
    for folder, manifest_path, manifest, package in candidates:
        destination = directory / folder
        if platform == 'macos':
            metadata_path = destination / 'macos-update.json'
            tool.run(['sign-archive', '--archive', package, '--public-key', sparkle,
                      '--version', VERSION, '--build', str(BUILD), '--output', metadata_path], PRIVATE[1])
            metadata = load(metadata_path, 4096)
            tool.run(['verify-archive', '--archive', package, '--public-key', sparkle,
                      '--signature', metadata['ed_signature']])
            unsigned = destination / 'appcast-unsigned.xml'
            with unsigned.open('xb') as stream:
                stream.write(appcast_bytes(manifest, metadata))
            feed = destination / 'appcast.xml'
            tool.run(['sign-appcast', '--appcast', unsigned, '--public-key', sparkle,
                      '--output', feed], PRIVATE[1])
            tool.run(['verify-appcast', '--appcast', feed, '--public-key', sparkle])
            signed.extend([{'path': 'owned/' + name, 'sha256': digest(destination / name)}
                           for name in ('macos-update.json', 'appcast.xml')])
        signature = destination / 'release.json.sig'
        tool.run(['sign', '--manifest', manifest_path, '--keyring', ring,
                  '--key-id', KEY_ID, '--output', signature], PRIVATE[0])
        tool.run(['verify', '--manifest', manifest_path, '--signature', signature, '--keyring', ring])
        signed.append({'path': folder + '/release.json.sig', 'sha256': digest(signature),
                       'manifest_sha256': digest(manifest_path), 'package_sha256': digest(package),
                       'signature_verified': True})
    # Recheck all input bytes after signing. A different preparer/runner process
    # must not replace a package or manifest between admission and completion.
    for name in record['artifacts']:
        artifact(directory, record, name)
    require(digest(regular(directory / PREPARED, 256 * 1024)) == prepared_sha256,
            'prepared_record_changed')
    summary = dict(schema_version=1, status='passed', mode='prepared-fixtures', platform=platform,
                   source_commit=record['source_commit'], signer_sha256=tool.expected,
                   prepared_sha256=prepared_sha256, public_key_match=matched,
                   signatures=signed, production_private_keys_exported=False,
                   release_created=False, packages_uploaded=False)
    with (directory / 'production-signing.json').open('x', encoding='utf-8') as stream:
        json.dump(summary, stream, sort_keys=True)
        stream.write('\n')
    return summary


def check_only(tool_path, output):
    commit = source_commit()
    tool_path = owned(tool_path, ROOT / 'build')
    regular(tool_path, 64 * 1024 * 1024)
    output = owned(output, ROOT / 'build')
    output.mkdir(parents=True, exist_ok=False)
    tool = Tool(tool_path, digest(tool_path))
    matched = match_keys(tool, output)
    summary = dict(schema_version=1, status='passed', mode='key-match-only', source_commit=commit,
                   signer_sha256=tool.expected, public_key_match=matched,
                   signing_performed=False, production_private_keys_exported=False,
                   release_created=False)
    with (output / 'production-key-match.json').open('x', encoding='utf-8') as stream:
        json.dump(summary, stream, sort_keys=True)
        stream.write('\n')
    return summary


def record_failure(args, summary):
    """Retain public failure evidence without invoking tools or reading secrets.

    A failed input/path check must never create an evidence directory or write
    outside build/. Exclusive creation also preserves earlier success/failure
    evidence if an operator accidentally reuses a fixture directory.
    """
    try:
        requested = args.output if args.check_only else args.directory
        if requested is None:
            return
        directory = owned(requested, ROOT / 'build')
        if not directory.is_dir():
            return
        name = 'production-key-match.json' if args.check_only else 'production-signing-result.json'
        destination = owned(directory / name, directory)
        with destination.open('x', encoding='utf-8') as stream:
            json.dump(summary, stream, sort_keys=True)
            stream.write('\n')
    except Exception:
        # Failure evidence is best effort; writing it must not hide the original
        # nonzero result or expose a raw OS exception or a user filesystem path.
        return


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check-only', action='store_true')
    parser.add_argument('--tool', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--directory', type=Path)
    args = parser.parse_args(argv)
    try:
        if args.check_only:
            require(args.tool is not None and args.output is not None and args.directory is None,
                    'invalid_check_only_arguments')
            summary = check_only(args.tool, args.output)
        else:
            require(args.directory is not None and args.tool is None and args.output is None,
                    'invalid_fixture_signing_arguments')
            directory = owned(args.directory, ROOT / 'build')
            require(directory.is_dir(), 'fixture_directory_missing')
            summary = sign_fixture(directory)
        print(json.dumps(summary, sort_keys=True))
        return 0
    except Exception as error:
        # Only AdmissionError contains our public vocabulary. Raw Python/OS/tool
        # exceptions may contain sensitive source text, environment or user paths.
        code = str(error) if isinstance(error, AdmissionError) else 'production_signing_failed'
        summary = {'schema_version': 1, 'status': 'failed', 'error': code}
        record_failure(args, summary)
        print(json.dumps(summary, sort_keys=True))
        return 1


if __name__ == '__main__':
    sys.exit(main())
