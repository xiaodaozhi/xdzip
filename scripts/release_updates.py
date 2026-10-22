# scripts/release_updates.py

"""Authenticated update attachments; private keys are supplied only at execution."""

from __future__ import annotations

import base64
from functools import lru_cache
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import xml.etree.ElementTree as ET

from release_protocol import sha256, validate

ROOT = Path(__file__).resolve().parents[1]
SPARKLE = 'http://www.andymatuschak.org/xml-namespaces/sparkle'
ET.register_namespace('sparkle', SPARKLE)


def signing_tool(tool: Path | None = None) -> list[str]:
    if tool is not None:
        return [str(tool)]
    return [str(build_signing_tool())]


@lru_cache(maxsize=1)
def build_signing_tool() -> Path:
    # Build scripts and compiler subprocesses need neither signing nor publishing
    # authority. Scrub a copy so the later signer and publisher retain theirs.
    environment = os.environ.copy()
    private_names = {
        'XDZIP_UPDATE_SIGNING_KEY_HEX', 'XDZIP_SPARKLE_SIGNING_KEY_HEX',
        'GH_TOKEN', 'GITHUB_TOKEN', 'GH_ENTERPRISE_TOKEN', 'GITHUB_ENTERPRISE_TOKEN',
        'GIT_ASKPASS', 'SSH_ASKPASS', 'SSH_ASKPASS_REQUIRE', 'SSH_AUTH_SOCK', 'SSH_AGENT_PID',
        'GIT_SSH', 'GIT_SSH_COMMAND', 'GIT_CONFIG', 'CARGO_REGISTRY_TOKEN',
    }
    for name in tuple(environment):
        normalized = name.upper()
        if (normalized in private_names or normalized.startswith('GIT_CONFIG_')
                or (normalized.startswith('CARGO_REGISTRIES_') and normalized.endswith('_TOKEN'))):
            del environment[name]
    result = subprocess.run(['cargo', 'build', '--locked', '--quiet', '--release', '-p', 'xdzip-update',
                             '--bin', 'xdzip-update-tool'], cwd=ROOT, env=environment, capture_output=True)
    if result.returncode:
        raise ValueError('Unable to build the update signing tool')
    return ROOT/'target/release'/('xdzip-update-tool.exe' if os.name == 'nt' else 'xdzip-update-tool')


def run_tool(arguments: list[str], tool: Path | None = None) -> None:
    # argv contains only public package metadata and paths, never private keys.
    # Capture diagnostics: a failed external signer must not print environment data.
    result = subprocess.run(signing_tool(tool) + arguments, cwd=ROOT, capture_output=True)
    if result.returncode:
        raise ValueError('Update signing or verification failed')


def appcast_bytes(manifest: dict, metadata: dict) -> bytes:
    """Embed a short version description, with no client fetch of external notes.

    Full release notes remain in the reviewed GitHub Release body.
    """
    validate(manifest)
    asset = next(a for a in manifest['assets'] if a['platform'] == 'macos' and a['package_type'] == 'app.zip')
    expected_keys = {'schema_version', 'version', 'build', 'filename', 'bytes', 'sha256', 'ed_signature'}
    if set(metadata) != expected_keys or metadata['schema_version'] != 1:
        raise ValueError('Invalid macOS update metadata')
    if any(metadata[key] != (manifest[key] if key in ('version', 'build') else asset[key])
           for key in ('version', 'build', 'filename', 'bytes', 'sha256')):
        raise ValueError('macOS update metadata does not describe the frozen archive')
    try:
        signature = base64.b64decode(metadata['ed_signature'], validate=True)
    except (ValueError, TypeError) as error:
        raise ValueError('Invalid Sparkle archive signature') from error
    if len(signature) != 64:
        raise ValueError('Invalid Sparkle archive signature')
    rss = ET.Element('rss', {'version': '2.0'})
    channel = ET.SubElement(rss, 'channel')
    ET.SubElement(channel, 'title').text = 'xdzip'
    ET.SubElement(channel, 'link').text = f'https://github.com/{manifest["repository"]}'
    ET.SubElement(channel, 'description').text = 'xdzip updates'
    item = ET.SubElement(channel, 'item')
    ET.SubElement(item, 'title').text = 'xdzip ' + manifest['version']
    ET.SubElement(item, f'{{{SPARKLE}}}version').text = str(manifest['build'])
    ET.SubElement(item, f'{{{SPARKLE}}}shortVersionString').text = manifest['version']
    ET.SubElement(item, f'{{{SPARKLE}}}minimumSystemVersion').text = '14.0'
    if manifest['channel'] != 'stable':
        ET.SubElement(item, f'{{{SPARKLE}}}channel').text = manifest['channel']
    ET.SubElement(item, 'description').text = 'Verified xdzip update, ' + manifest['version']
    ET.SubElement(item, 'enclosure', {'url': asset['url'], 'length': str(asset['bytes']),
                  'type': 'application/octet-stream', f'{{{SPARKLE}}}edSignature': metadata['ed_signature']})
    ET.indent(rss)
    return ET.tostring(rss, encoding='utf-8', xml_declaration=True) + b'\n'


def prepare_appcast(directory: Path, manifest: dict, *, required: bool = False, tool: Path | None = None,
                    public_key: Path | None = None) -> bool:
    """Bind and sign the native archive and complete feed with the Sparkle key."""
    metadata_path = directory / 'macos-update.json'
    configured = bool(os.environ.get('XDZIP_SPARKLE_SIGNING_KEY_HEX'))
    if not metadata_path.is_file() or not configured:
        if required:
            raise ValueError('Sparkle archive metadata and signing key are required')
        return False
    if metadata_path.is_symlink() or metadata_path.stat().st_size > 4096:
        raise ValueError('Invalid macOS update metadata file')
    metadata = json.loads(metadata_path.read_bytes())
    content = appcast_bytes(manifest, metadata)
    archive = directory / metadata['filename']
    if archive.is_symlink() or not archive.is_file() or archive.stat().st_size != metadata['bytes'] or sha256(archive) != metadata['sha256']:
        raise ValueError('macOS update archive changed')
    public_key = public_key or ROOT / 'resources/sparkle-update-key.json'
    run_tool(['verify-archive', '--archive', str(archive), '--public-key', str(public_key),
              '--signature', metadata['ed_signature']], tool)
    with tempfile.TemporaryDirectory(prefix='xdzip-appcast-', dir=directory.parent) as temporary:
        unsigned = Path(temporary) / 'appcast.xml'
        unsigned.write_bytes(content)
        output = directory / 'appcast.xml'
        run_tool(['sign-appcast', '--appcast', str(unsigned), '--public-key', str(public_key), '--output', str(output)], tool)
        run_tool(['verify-appcast', '--appcast', str(output), '--public-key', str(public_key)], tool)
    manifest['macos_appcast_url'] = f'https://github.com/{manifest["repository"]}/releases/download/{manifest["tag"]}/appcast.xml'
    return True


def sign_manifest(directory: Path, *, key_id: str | None, required: bool = False,
                  tool: Path | None = None, keyring: Path | None = None) -> bool:
    """Sign exact JSON bytes and independently reverify before creating checksums."""
    configured = bool(os.environ.get('XDZIP_UPDATE_SIGNING_KEY_HEX')) and bool(key_id)
    if not configured:
        if required:
            raise ValueError('Update manifest signing key and key ID are required')
        return False
    if not re.fullmatch('[A-Za-z0-9_-]{1,64}', key_id):
        raise ValueError('Invalid update key ID')
    keyring = keyring or ROOT / 'resources/update-trust.json'
    arguments = ['--manifest', str(directory / 'release.json'), '--keyring', str(keyring)]
    signature = directory / 'release.json.sig'
    run_tool(['sign', *arguments, '--key-id', key_id, '--output', str(signature)], tool)
    run_tool(['verify', *arguments, '--signature', str(signature)], tool)
    return True


def verify_release_updates(directory: Path, manifest: dict, *, tool: Path | None = None,
                           keyring: Path | None = None, public_key: Path | None = None) -> None:
    """Publication gate; neither a caller flag nor readiness JSON grants trust."""
    keyring = keyring or ROOT / 'resources/update-trust.json'
    public_key = public_key or ROOT / 'resources/sparkle-update-key.json'
    for name, limit in [('release.json.sig', 4096), ('appcast.xml', 256 * 1024 + 512), ('macos-update.json', 4096)]:
        path = directory / name
        if path.is_symlink() or not path.is_file() or path.stat().st_size > limit:
            raise ValueError('Authenticated update attachments are required')
    expected_url = f'https://github.com/{manifest["repository"]}/releases/download/{manifest["tag"]}/appcast.xml'
    if manifest.get('macos_appcast_url') != expected_url:
        raise ValueError('Authenticated candidate feed is missing')
    run_tool(['verify', '--manifest', str(directory/'release.json'), '--signature', str(directory/'release.json.sig'),
              '--keyring', str(keyring)], tool)
    run_tool(['verify-appcast', '--appcast', str(directory/'appcast.xml'), '--public-key', str(public_key)], tool)
    metadata = json.loads((directory/'macos-update.json').read_bytes())
    unsigned = appcast_bytes(manifest, metadata)
    signed = (directory/'appcast.xml').read_bytes()
    # The feed signed by the Sparkle key must describe this exact six-package
    # candidate, rather than another otherwise valid signed feed.
    if not signed.startswith(unsigned + b'<!-- sparkle-signatures:\n'):
        raise ValueError('Signed appcast does not describe this release')
    archive = directory / metadata['filename']
    run_tool(['verify-archive', '--archive', str(archive), '--public-key', str(public_key),
              '--signature', metadata['ed_signature']], tool)
