# scripts/release_protocol.py

"""Immutable GitHub release manifest contract, shared by packaging and future clients."""
import hashlib
import json
from pathlib import Path
import re

TARGETS = {
    ('windows', 'x86_64', 'zip'): ('windows-x86_64.zip', {'windows': '11'}),
    ('windows', 'x86_64', 'exe'): ('windows-x86_64-setup.exe', {'windows': '11'}),
    ('macos', 'arm64', 'app.zip'): ('macos-arm64.app.zip', {'macos': '14.0'}),
    ('macos', 'arm64', 'dmg'): ('macos-arm64.dmg', {'macos': '14.0'}),
    ('linux', 'x86_64', 'tar.gz'): ('linux-x86_64.tar.gz', {'distribution': 'ubuntu', 'version': '24.04', 'glibc': '2.39', 'display_server': 'X11'}),
    ('linux', 'x86_64', 'deb'): ('ubuntu24.04-x86_64.deb', {'distribution': 'ubuntu', 'version': '24.04', 'glibc': '2.39', 'display_server': 'X11'}),
}
CHANNEL_ORDER = {'alpha': 0, 'beta': 1, 'rc': 2, 'stable': 3}


def sha256(path):
    result = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for data in iter(lambda: stream.read(1024 * 1024), b''):
            result.update(data)
    return result.hexdigest()


def semantic_version(value, allow_development=False):
    channels = 'dev|alpha|beta|rc' if allow_development else 'alpha|beta|rc'
    match = re.fullmatch(r'(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-('+channels+r')\.([1-9]\d*))?', value)
    if not match:
        raise ValueError('Unsupported semantic version')
    major, minor, patch, channel, number = match.groups()
    if any(int(part) > 65535 for part in (major, minor, patch)) or int(number or 0) > 65535:
        raise ValueError('Version components exceed the native identity range')
    return (int(major), int(minor), int(patch), -1 if channel == 'dev' else CHANNEL_ORDER[channel or 'stable'], int(number or 0))


def compare_versions(left, right):
    """Match the product ABI: -1/0/1, or 2 for unsupported input."""
    try:
        left = semantic_version(left, allow_development=True)
        right = semantic_version(right, allow_development=True)
        return (left > right) - (left < right)
    except (TypeError, ValueError):
        return 2


def release_tag_eligible(tag, current, current_channel):
    """Channel progression is independent of a higher base version."""
    try:
        if not tag.startswith('v'):
            return False
        candidate = semantic_version(tag[1:])
        current_version = semantic_version(current, allow_development=True)
        rank = -1 if current_channel == 'development' else CHANNEL_ORDER.get(current_channel)
        return rank is not None and current_version[3] == rank and candidate > current_version and candidate[3] >= rank
    except (TypeError, ValueError):
        return False


def validate(manifest, directory=None):
    if manifest.get('schema_version') != 1 or manifest.get('product') != 'xdzip':
        raise ValueError('Unsupported release manifest')
    version = manifest['version']
    parsed = semantic_version(version)
    channel = manifest['channel']
    if channel not in CHANNEL_ORDER or parsed[3] != CHANNEL_ORDER[channel]:
        raise ValueError('Channel disagrees with semantic version')
    if manifest['tag'] != 'v' + version or not re.fullmatch('[a-f0-9]{40}', manifest['commit']):
        raise ValueError('Tag or full commit identity mismatch')
    if type(manifest['build']) is not int or not 1 <= manifest['build'] <= 65535:
        raise ValueError('Invalid build number')
    repository = manifest['repository']
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository):
        raise ValueError('Invalid GitHub repository')
    seen = set()
    for asset in manifest['assets']:
        target = (asset['platform'], asset['architecture'], asset['package_type'])
        if target in seen or target not in TARGETS:
            raise ValueError('Duplicate or unsupported target')
        seen.add(target)
        suffix, requirements = TARGETS[target]
        name = 'xdzip-' + version + '-' + suffix
        if asset['filename'] != name or asset['minimum_system'] != requirements:
            raise ValueError('Asset identity or minimum system mismatch')
        url = f'https://github.com/{repository}/releases/download/v{version}/{name}'
        if asset['url'] != url or not re.fullmatch('[a-f0-9]{64}', asset['sha256']):
            raise ValueError('Unpinned download URL or invalid SHA-256')
        if type(asset['bytes']) is not int or asset['bytes'] <= 0:
            raise ValueError('Empty asset')
        if directory is not None:
            path = Path(directory) / name
            if path.is_symlink() or not path.is_file() or path.stat().st_size != asset['bytes'] or sha256(path) != asset['sha256']:
                raise ValueError('Missing or corrupted release asset')
    if seen != set(TARGETS):
        raise ValueError('Incomplete cross-platform release')
    appcast = manifest.get('macos_appcast_url', '')
    if appcast and appcast != f'https://github.com/{repository}/releases/download/v{version}/appcast.xml':
        raise ValueError('Unpinned macOS appcast URL')
    return manifest


def create(directory, version, repository, commit):
    display = version['display_version']
    result = dict(schema_version=1, product='xdzip', version=display, channel=version['channel'],
                  build=version['build'], tag='v'+display, commit=commit, repository=repository, assets=[])
    for (platform, architecture, package_type), (suffix, requirements) in TARGETS.items():
        name = f'xdzip-{display}-{suffix}'
        path = Path(directory)/name
        result['assets'].append(dict(filename=name, platform=platform, architecture=architecture,
            package_type=package_type, minimum_system=requirements, bytes=path.stat().st_size,
            sha256=sha256(path), url=f'https://github.com/{repository}/releases/download/v{display}/{name}'))
    return validate(result, directory)


def select(manifest, current, allowed_channels, platform, architecture, package_type):
    """Future client selection. Callers must still verify downloaded bytes before applying."""
    validate(manifest)
    if manifest['channel'] not in allowed_channels or semantic_version(manifest['version']) <= semantic_version(current, allow_development=True):
        return None
    return next((asset for asset in manifest['assets'] if (asset['platform'], asset['architecture'], asset['package_type']) ==
                 (platform, architecture, package_type)), None)
