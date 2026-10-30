"""Bind compiled link inputs and actual deployed macOS application evidence."""
import hashlib
import struct


def macho_sections(data):
    """Read a bounded thin arm64 image, excluding file-only signing metadata.

    Deployment may rewrite load commands and __LINKEDIT signatures. Every
    declared non-debug code/data section must retain its exact bytes or zero-fill
    size; this proves that narrower relation, not complete binary equivalence.
    """
    if len(data) < 32 or len(data) > 256 * 1024 * 1024:
        raise ValueError('Unsupported Mach-O image size')
    magic, cpu, _, _, count, commands_size = struct.unpack_from('<6I', data)
    if magic != 0xFEEDFACF or cpu != 0x0100000C:
        raise ValueError('Expected a thin little-endian arm64 Mach-O image')
    end = 32 + commands_size
    if count > 65536 or end > len(data):
        raise ValueError('Invalid Mach-O load command bounds')
    cursor = 32
    sections = {}
    for _ in range(count):
        if cursor + 8 > end:
            raise ValueError('Truncated Mach-O load command')
        command, size = struct.unpack_from('<II', data, cursor)
        if size < 8 or size % 8 or cursor + size > end:
            raise ValueError('Invalid Mach-O load command size')
        if command == 0x19:  # LC_SEGMENT_64
            if size < 72:
                raise ValueError('Truncated Mach-O segment')
            segment = data[cursor + 8:cursor + 24].split(b'\0', 1)[0].decode('ascii')
            section_count = struct.unpack_from('<I', data, cursor + 64)[0]
            if 72 + section_count * 80 != size:
                raise ValueError('Invalid Mach-O section table bounds')
            for index in range(section_count):
                start = cursor + 72 + index * 80
                name = data[start:start + 16].split(b'\0', 1)[0].decode('ascii')
                declared_segment = data[start + 16:start + 32].split(b'\0', 1)[0].decode('ascii')
                if declared_segment != segment:
                    raise ValueError('Mach-O section/segment mismatch')
                if segment in ('__LINKEDIT', '__DWARF'):
                    continue
                length = struct.unpack_from('<Q', data, start + 40)[0]
                offset = struct.unpack_from('<I', data, start + 48)[0]
                flags = struct.unpack_from('<I', data, start + 64)[0]
                zero_fill = (flags & 0xFF) in (0x1, 0xC, 0x12)
                key = (segment, name)
                if key in sections:
                    raise ValueError('Duplicate Mach-O section')
                if not zero_fill and length and (offset < end or offset + length > len(data)):
                    raise ValueError('Mach-O section data escapes image')
                sections[key] = {'segment': segment, 'section': name, 'bytes': length,
                                 'zero_fill': zero_fill,
                                 'sha256': None if zero_fill else hashlib.sha256(data[offset:offset + length]).hexdigest()}
        cursor += size
    if cursor != end or ('__TEXT', '__text') not in sections or not sections[('__TEXT', '__text')]['bytes']:
        raise ValueError('Missing or incomplete Mach-O code sections')
    return [sections[key] for key in sorted(sections)]


def compare_macho_sections(compiled, deployed):
    before = macho_sections(compiled)
    if before != macho_sections(deployed):
        raise ValueError('Deployment changed compiled code/data sections')
    return {'status': 'exact-code-data-section-match', 'sections': before,
            'compiled_sha256': hashlib.sha256(compiled).hexdigest(),
            'deployed_sha256': hashlib.sha256(deployed).hexdigest(),
            'scope': 'Thin arm64 non-debug sections, including zero-fill sizes; load commands, debug and signing metadata remain distinct.'}


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
    if (section_match.get('status') != 'exact-code-data-section-match'
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
