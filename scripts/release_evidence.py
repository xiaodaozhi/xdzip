"""Bind compiled link inputs and actual deployed macOS application evidence."""
import hashlib
from macos_attribution import compare_images, image_identity


def macho_sections(data):
    """Apply the established section/UUID/import parser to a bounded app image."""
    if len(data) < 32 or len(data) > 256 * 1024 * 1024 or data[:4] != b'\xcf\xfa\xed\xfe':
        raise ValueError('Expected a bounded thin arm64 Mach-O image')
    identity = image_identity(data)
    sections, seen = [], set()
    for segment, name, address, length, flags, checksum in identity['sections']:
        key = (segment, name)
        if key in seen:
            raise ValueError('Duplicate Mach-O section')
        seen.add(key)
        sections.append({'segment': bytes.fromhex(segment).rstrip(b'\0').decode('ascii'),
                         'section': bytes.fromhex(name).rstrip(b'\0').decode('ascii'),
                         'address': address, 'bytes': length, 'flags': flags,
                         'zero_fill': (flags & 0xFF) in (0x1, 0xC, 0x12), 'sha256': checksum})
    if not any(s['segment'] == '__TEXT' and s['section'] == '__text' and s['bytes'] for s in sections):
        raise ValueError('Missing Mach-O application code section')
    return sections


def compare_macho_sections(compiled, deployed):
    before = macho_sections(compiled)
    macho_sections(deployed)
    try:
        fingerprint = compare_images(deployed, compiled)
    except ValueError as error:
        raise ValueError('Deployment changed compiled sections, UUID or imports') from error
    return {'status': 'exact-code-data-section-match', 'sections': before,
            'comparison': 'mach-o-sections-uuid-and-imports', 'fingerprint': fingerprint,
            'compiled_sha256': hashlib.sha256(compiled).hexdigest(),
            'deployed_sha256': hashlib.sha256(deployed).hexdigest(),
            'scope': 'Established arm64 section/address/flag, UUID and normalized import comparison; load-command layout, LINKEDIT and signatures remain distinct.'}


def verify_bindings(revision, audit, resources, linkage, sdk, section_match):
    if len(revision) != 40 or any(c not in '0123456789abcdef' for c in revision):
        raise ValueError('Expected a full candidate revision')
    identity = audit['identity']
    if (audit['status'] != 'passed' or audit['mode'] != 'deployed' or audit['platform'] != 'macos'
            or identity['dirty'] is not False or identity['revision'] != revision[:12]):
        raise ValueError('Deployment audit belongs to another candidate')
    rows = [row for row in audit['files'] if row['path'] == audit['app']]
    if len(rows) != 1:
        raise ValueError('Deployment audit has no unique application image')
    expected = rows[0]['sha256']
    fingerprint = section_match.get('fingerprint')
    if (section_match.get('status') != 'exact-code-data-section-match'
            or section_match.get('comparison') != 'mach-o-sections-uuid-and-imports'
            or not isinstance(fingerprint, str) or len(fingerprint) != 64
            or any(c not in '0123456789abcdef' for c in fingerprint)
            or section_match['deployed_sha256'] != expected):
        raise ValueError('Code/data proof belongs to another deployed image')
    if resources['app_sha256'] != expected:
        raise ValueError('Resource proof belongs to another deployed image')
    resource_identity = resources['identity']
    if resource_identity['dirty'] is not False or resource_identity['revision'] != revision[:12]:
        raise ValueError('Resource proof belongs to another candidate')
    if linkage['app_sha256'] != section_match['compiled_sha256']:
        raise ValueError('Link-input proof belongs to another compiled image')
    if sdk['revision'] != revision or sdk['source_dirty'] is not False:
        raise ValueError('SDK source proof belongs to another candidate')
