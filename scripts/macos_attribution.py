# scripts/macos_attribution.py

"""Bind deployed Qt images to SDK inputs without treating signatures as code identity."""
import hashlib
import os
from pathlib import Path
import re
import struct

from deployment_audit import digest, native
from runtime_attribution import bounded


def arm64_image(data):
    if data[:4] == b'\xca\xfe\xba\xbe':
        if len(data) < 8:
            raise ValueError('Truncated universal Mach-O')
        count = struct.unpack_from('>I', data, 4)[0]
        if count > 64 or 8 + count * 20 > len(data):
            raise ValueError('Invalid universal Mach-O table')
        slices = []
        for i in range(count):
            cpu, _, offset, size, _ = struct.unpack_from('>IIIII', data, 8 + i * 20)
            if offset < 8 + count * 20 or offset + size > len(data):
                raise ValueError('Unbounded universal Mach-O slice')
            if cpu == 0x100000c:
                slices.append(data[offset:offset + size])
        if len(slices) != 1:
            raise ValueError('Expected one arm64 SDK slice')
        data = slices[0]
    if len(data) < 32 or data[:4] != b'\xcf\xfa\xed\xfe':
        raise ValueError('Expected little-endian 64-bit Mach-O')
    cpu = struct.unpack_from('<I', data, 4)[0]
    if cpu != 0x100000c:
        raise ValueError('Expected arm64 Mach-O')
    return data


def image_identity(data):
    """Compare all sections, UUID and import versions; exclude relocation/signature tables.

    lipo, install_name_tool and codesign change container/load-command/LINKEDIT
    bytes. This is deliberately narrower than a whole-image or build-equivalence
    claim. Unexpected formats, bounds, imports or changed section bytes fail.
    """
    data = arm64_image(data)
    count, size = struct.unpack_from('<II', data, 16)
    end = 32 + size
    if count > 4096 or end > len(data):
        raise ValueError('Invalid Mach-O command table')
    commands, sections, imports, uuids = 32, [], [], []
    for _ in range(count):
        if commands + 8 > end:
            raise ValueError('Truncated Mach-O command')
        kind, length = struct.unpack_from('<II', data, commands)
        if length < 8 or length % 8 or commands + length > end:
            raise ValueError('Unbounded Mach-O command')
        if kind == 0x19:  # LC_SEGMENT_64
            if length < 72:
                raise ValueError('Truncated Mach-O segment')
            nsects = struct.unpack_from('<I', data, commands + 64)[0]
            if 72 + nsects * 80 > length:
                raise ValueError('Truncated Mach-O sections')
            for i in range(nsects):
                at = commands + 72 + i * 80
                name, segment, address, amount, offset = struct.unpack_from('<16s16sQQI', data, at)
                flags = struct.unpack_from('<I', data, at + 64)[0]
                zero = (flags & 0xff) in {1, 0xc, 0x12}
                if not zero and (offset < end or offset + amount > len(data)):
                    raise ValueError('Unbounded Mach-O section')
                content = b'' if zero else data[offset:offset + amount]
                sections.append((segment.hex(), name.hex(), address, amount, flags,
                                 hashlib.sha256(content).hexdigest()))
        elif kind in {0xc, 0x18 | 0x80000000, 0x1f | 0x80000000, 0x20, 0x23 | 0x80000000}:
            if length < 24:
                raise ValueError('Truncated Mach-O import')
            name_at, _, current, compatibility = struct.unpack_from('<IIII', data, commands + 8)
            if name_at < 24 or name_at >= length:
                raise ValueError('Unbounded Mach-O import name')
            raw = data[commands + name_at:commands + length]
            if b'\0' not in raw:
                raise ValueError('Unterminated Mach-O import')
            name = raw.split(b'\0', 1)[0].decode('utf-8')
            # Only Qt relocation is allowed. System library names retain paths.
            qt_import = re.search(r'/(Qt[A-Za-z0-9]+)\.framework/(Versions/[^/]+/)?\1$', name)
            normalized = name[qt_import.start()+1:] if qt_import else name
            imports.append((kind, normalized, current, compatibility))
        elif kind == 0x1b:
            if length != 24:
                raise ValueError('Invalid Mach-O UUID')
            uuids.append(data[commands + 8:commands + 24].hex())
        elif kind in {0x21, 0x2c}:
            raise ValueError('Encrypted Mach-O is outside attribution policy')
        commands += length
    if commands != end or not sections or len(uuids) != 1:
        raise ValueError('Incomplete Mach-O identity')
    return {'header': struct.unpack_from('<7I', data, 4), 'sections': sorted(sections),
            'imports': sorted(imports), 'uuid': uuids[0]}


def compare_images(deployed, sdk):
    left, right = image_identity(deployed), image_identity(sdk)
    # ncmds/sizeofcmds change during relocation and signing, CPU/subtype/type,
    # flags/reserved must remain the same.
    for image in (left, right):
        image['header'] = tuple(v for i, v in enumerate(image['header']) if i not in {3, 4})
    if left != right:
        raise ValueError('Deployed Mach-O sections or imports differ from SDK')
    return hashlib.sha256(repr(left).encode('utf-8')).hexdigest()


def sdk_relative(relative):
    for prefix, replacement in [('Contents/Frameworks/', 'lib/'),
                                ('Contents/PlugIns/', 'plugins/'),
                                ('Contents/Resources/qml/', 'qml/'),
                                ('Contents/Resources/translations/', 'translations/')]:
        if relative.startswith(prefix):
            return replacement + relative[len(prefix):]
    return None


def audited_qml_aliases(root, audit):
    """Bind audited QML file links to their actual in-bundle quick-plugin targets."""
    aliases, seen = {}, set()
    for item in audit['files']:
        relative = item['path']
        if not relative.startswith('Contents/Resources/qml/') or 'symlink_target' not in item:
            continue
        if relative in seen:
            raise ValueError('Duplicate audited QML link')
        seen.add(relative)
        link = bounded(root, relative)
        if not link.is_symlink():
            raise ValueError('Audited QML link is no longer a symlink')
        actual_target = os.path.relpath(link.resolve(), link.parent.resolve()).replace('\\', '/')
        if item['symlink_target'] != actual_target or digest(link) != item['sha256']:
            raise ValueError('Audited QML link changed after deployment audit')
        target = link.resolve().relative_to(root.resolve()).as_posix()
        if target.startswith('Contents/PlugIns/quick/'):
            aliases.setdefault(target, set()).add(sdk_relative(relative))
    return aliases


def sdk_source_for_path(sdk, relative, aliases):
    """Return one bounded SDK path; never guess when QML aliases are ambiguous."""
    direct = sdk_relative(relative)
    if direct and bounded(sdk, direct).is_file():
        return bounded(sdk, direct).resolve().relative_to(sdk.resolve()).as_posix()
    if not relative.startswith('Contents/PlugIns/quick/'):
        return None
    candidates = set()
    for source in aliases.get(relative, set()):
        if not source or not bounded(sdk, source).is_file():
            return None
        candidates.add(bounded(sdk, source).resolve().relative_to(sdk.resolve()).as_posix())
    if len(candidates) > 1:
        raise ValueError('Ambiguous audited QML links for SDK attribution')
    return next(iter(candidates), None)


def deployment_record(root, sdk, audit):
    records = []
    aliases = audited_qml_aliases(root, audit)
    for item in audit['files']:
        relative = item['path']
        path = bounded(root, relative)
        if digest(path) != item['sha256']:
            raise ValueError('Artifact changed before macOS attribution')
        source = sdk_source_for_path(sdk, relative, aliases)
        if not source:
            continue
        original = bounded(sdk, source)
        if digest(original) == item['sha256']:
            method, fingerprint = 'sdk-byte-identity', None
        elif native(path, 'macos'):
            fingerprint = compare_images(path.read_bytes(), original.read_bytes())
            method = 'mach-o-sections-uuid-and-imports'
        else:
            continue  # Changed metadata is not silently granted byte identity.
        records.append({'path': relative, 'sdk_path': source,
                        'sdk_sha256': digest(original), 'artifact_sha256': item['sha256'],
                        'comparison': method, 'fingerprint': fingerprint})
    return {'schema_version': 1, 'platform': 'macos', 'sdk_files': records,
            'limits': ['Mach-O section/UUID/import comparison excludes load-command layout, LINKEDIT and signatures; it is component attribution, not whole-image or upstream build equivalence.']}
