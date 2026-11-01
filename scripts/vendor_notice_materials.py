#!/usr/bin/env python3
"""Bind supplemental upstream notices to the exact Cargo vendor packages."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import re

ROOT = Path(__file__).resolve().parents[1]
CONVENTIONAL = re.compile(r'(?:LICENSE|LICENCE|COPYING|COPYRIGHT|NOTICE)(?:[._-][A-Z0-9][A-Z0-9._+-]*)?')


def relative(value):
    """Accept portable relative names without drive prefixes or parent traversal."""
    if not isinstance(value, str) or not value or '\\' in value or ':' in value:
        raise ValueError('Invalid relative notice path')
    path = PurePosixPath(value)
    if path.is_absolute() or any(part in ('.', '..') for part in path.parts):
        raise ValueError('Invalid relative notice path')
    return path


def source_file(root, value):
    parts = relative(value).parts
    current = root
    for part in parts:
        current = current / part
        if current.is_symlink():
            raise ValueError('Symlink in notice source')
    if not current.is_file() or not current.resolve().is_relative_to(root.resolve()):
        raise ValueError('Notice source is missing or escapes its root')
    return current


def package_fields(raw):
    """Read the three plain-string fields from a pinned Cargo-generated package block."""
    text = raw.decode('utf-8')
    match = re.search(r'(?ms)^\[package\]\s*\n(.*?)(?=^\[|\Z)', text)
    if not match:
        raise ValueError('Missing Cargo package block')
    values = {}
    for key in ('name', 'version', 'license'):
        fields = re.findall(r'(?m)^' + key + r'\s*=\s*("(?:[^"\\]|\\.)*")\s*$', match.group(1))
        if len(fields) != 1:
            raise ValueError('Unsupported Cargo package identity')
        values[key] = json.loads(fields[0])
    return values


def collect(vendor, output, *, root=ROOT, config_path=None):
    """Validate all missing notice paths before writing a separate supplement directory."""
    vendor = Path(vendor)
    output = Path(output)
    root = Path(root)
    config_path = Path(config_path) if config_path else root/'scripts/rust-vendor-notice-sources.json'
    if not vendor.is_dir() or vendor.is_symlink():
        raise ValueError('Cargo vendor directory is missing or is a symlink')
    if output.exists() or output.is_symlink():
        raise ValueError('Supplement output already exists')
    config_bytes = config_path.read_bytes()
    config = json.loads(config_bytes)
    if config.get('schema_version') != 1 or not isinstance(config.get('packages'), list):
        raise ValueError('Unsupported supplemental notice manifest')
    pinned = {}
    for item in config['packages']:
        directory = item['vendor_directory']
        if (len(relative(directory).parts) != 1 or directory != item['name'] + '-' + item['version']
                or directory.casefold() in pinned):
            raise ValueError('Invalid or duplicate vendor identity')
        pinned[directory.casefold()] = item

    missing = []
    for package in sorted(vendor.iterdir()):
        if package.is_symlink():
            raise ValueError('Symlink in Cargo vendor root')
        if not package.is_dir():
            continue
        manifest = package/'Cargo.toml'
        if not manifest.is_file() or manifest.is_symlink():
            raise ValueError('Invalid Cargo vendor package')
        conventional = [path for path in package.iterdir()
                        if path.is_file() and not path.is_symlink()
                        and CONVENTIONAL.fullmatch(path.name.upper())]
        if not conventional:
            if package.name.casefold() not in pinned:
                raise ValueError('Unresolved vendor notice path: ' + package.name)
            missing.append((package, pinned[package.name.casefold()]))

    prepared = []
    for package, item in missing:
        if package.name != item['vendor_directory']:
            raise ValueError('Vendor directory case or identity mismatch')
        cargo = (package/'Cargo.toml').read_bytes()
        if hashlib.sha256(cargo).hexdigest() != item['manifest_sha256']:
            raise ValueError('Vendor package manifest checksum mismatch')
        fields = package_fields(cargo)
        if fields != {'name': item['name'], 'version': item['version'],
                      'license': item['license_expression']}:
            raise ValueError('Vendor package metadata mismatch')
        vcs_path = package/'.cargo_vcs_info.json'
        if vcs_path.is_symlink() or not vcs_path.is_file():
            raise ValueError('Vendor source revision is missing')
        vcs = json.loads(vcs_path.read_text(encoding='utf-8'))
        revision = item['upstream_revision']
        if (not re.fullmatch('[0-9a-f]{40}', revision)
                or vcs.get('git', {}).get('sha1') != revision
                or vcs.get('path_in_vcs') != item['upstream_package_path']):
            raise ValueError('Vendor source revision mismatch')
        repository = item['upstream_repository']
        if not re.fullmatch(r'https://github\.com/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository):
            raise ValueError('Invalid upstream repository')
        names = set()
        notices = []
        for notice in item['notices']:
            name = notice['name']
            if len(relative(name).parts) != 1 or name.casefold() in names:
                raise ValueError('Invalid or duplicate upstream notice')
            names.add(name.casefold())
            repo_file = relative(notice['repo_file'])
            if repo_file.parts[:2] != ('third_party', 'rust-vendor-notices'):
                raise ValueError('Notice source outside pinned materials')
            source = source_file(root, repo_file.as_posix())
            raw = source.read_bytes()
            sha256 = hashlib.sha256(raw).hexdigest()
            blob = hashlib.sha1(b'blob ' + str(len(raw)).encode('ascii') + b'\0' + raw).hexdigest()
            if sha256 != notice['sha256'] or blob != notice['upstream_git_blob']:
                raise ValueError('Pinned upstream notice checksum mismatch')
            expected_url = repository + '/blob/' + revision + '/' + name
            if notice['upstream_url'] != expected_url:
                raise ValueError('Pinned upstream notice URL mismatch')
            existing = package/name
            if existing.exists() and (existing.is_symlink() or not existing.is_file()
                                      or existing.read_bytes() != raw):
                raise ValueError('Existing vendor notice differs from the pinned source')
            notices.append({'name': name, 'sha256': sha256, 'upstream_git_blob': blob,
                            'upstream_url': expected_url, 'bytes': raw})
        if not notices:
            raise ValueError('Pinned vendor package has no notices')
        prepared.append((item, notices))

    output.mkdir(parents=True)
    records = []
    for item, notices in prepared:
        target = output/item['vendor_directory']
        target.mkdir()
        files = []
        for notice in notices:
            data = notice.pop('bytes')
            path = target/notice['name']
            with path.open('xb') as stream:
                stream.write(data)
            files.append(dict(notice, file=(Path(item['vendor_directory'])/notice['name']).as_posix()))
        records.append({key: item[key] for key in ('name', 'version', 'vendor_directory',
                       'license_expression', 'manifest_sha256', 'upstream_repository',
                       'upstream_revision', 'upstream_package_path')} | {'notices': files})
    index = {'schema_version': 1, 'source_manifest_sha256': hashlib.sha256(config_bytes).hexdigest(),
             'scope': 'Supplemental upstream notices for source-distributed vendor packages; not license approval.',
             'packages': records}
    (output/'index.json').write_text(json.dumps(index, ensure_ascii=False, indent=2) + '\n',
                                     encoding='utf-8', newline='\n')
    return index
