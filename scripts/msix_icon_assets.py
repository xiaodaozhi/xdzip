"""MSIX image resources, shared by package construction and verification."""
import json
from pathlib import Path
import struct

from association_types import WINDOWS_ICONS

ROOT = Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / 'specs/visual/msix-icons.json'
POLICY = json.loads(POLICY_PATH.read_text(encoding='utf-8'))
UAP = '{http://schemas.microsoft.com/appx/manifest/uap/windows10}'
NS = '{http://schemas.microsoft.com/appx/manifest/foundation/windows10}'
PNG = b'\x89PNG\r\n\x1a\n'


def png_size(data):
    if len(data) < 33 or data[:8] != PNG or data[12:16] != b'IHDR':
        raise ValueError('Invalid MSIX icon PNG')
    return struct.unpack_from('>II', data, 16)


def association_logo(icon):
    return f'Assets/file-{icon}.png'


def application_assets():
    app = POLICY['application']
    for name, size in app['logos'].items():
        yield name + '.png', size
        for scale in app['scales']:
            if scale != 100:
                yield f'{name}.scale-{scale}.png', (size * scale + 50) // 100
    for size in app['target_sizes']:
        for form in app['alternate_forms']:
            suffix = '_altform-' + form if form else ''
            yield f'logo44.targetsize-{size}{suffix}.png', size


def ico_frames(path):
    """Read the already-rendered canonical PNG frames without resampling them."""
    data = path.read_bytes()
    if len(data) < 6 or data[:4] != b'\x00\x00\x01\x00':
        raise ValueError('Invalid canonical association ICO')
    count = struct.unpack_from('<H', data, 4)[0]
    if count != len(POLICY['associations']['target_sizes']) or len(data) < 6 + 16 * count:
        raise ValueError('Canonical association ICO frame set differs')
    frames = {}
    for index in range(count):
        width, height, _, _, _, _, length, offset = struct.unpack_from('<BBBBHHII', data, 6 + 16 * index)
        size = width or 256
        if size != (height or 256) or offset < 6 + 16 * count or offset + length > len(data):
            raise ValueError('Invalid canonical association ICO frame')
        frame = data[offset:offset + length]
        if size in frames or png_size(frame) != (size, size):
            raise ValueError('Invalid canonical association PNG frame')
        frames[size] = frame
    if set(frames) != set(POLICY['associations']['target_sizes']):
        raise ValueError('Canonical association icon sizes differ')
    return frames


def association_assets():
    for icon in sorted(set(WINDOWS_ICONS.values())):
        frames = ico_frames(ROOT / POLICY['associations']['source_directory'] / f'xdzip-{icon}.ico')
        yield f'file-{icon}.png', frames[POLICY['associations']['base_size']]
        for size, data in frames.items():
            yield f'file-{icon}.targetsize-{size}.png', data


def prepare_associations(directory):
    for name, data in association_assets():
        (directory / name).write_bytes(data)


def verify_manifest_icons(root):
    visual = root.find(NS + 'Applications/' + NS + 'Application/' + UAP + 'VisualElements')
    if (visual is None or visual.get('BackgroundColor') != 'transparent' or
            visual.get('Square44x44Logo') != 'Assets\\logo44.png' or
            visual.get('Square150x150Logo') != 'Assets\\logo150.png'):
        raise ValueError('MSIX application icon declaration differs')
    associations = list(root.iter(UAP + 'FileTypeAssociation'))
    expected = {'xdzip-' + ext: icon for ext, icon in WINDOWS_ICONS.items()}
    if len(associations) != len(expected) or {item.get('Name') for item in associations} != set(expected):
        raise ValueError('MSIX association icon mapping differs')
    for item in associations:
        logo = item.find(UAP + 'Logo')
        if logo is None or logo.text != association_logo(expected[item.get('Name')]).replace('/', '\\'):
            raise ValueError('MSIX file association lacks its canonical format icon')


def verify_package_icons(package, root):
    verify_manifest_icons(root)
    names = set(package.namelist())
    pri = POLICY['resource_index']
    if pri not in names or package.getinfo(pri).file_size == 0:
        raise ValueError('MSIX icon resource index is missing')
    expected = dict(application_assets())
    association_data = dict(association_assets())
    expected.update({name: png_size(data)[0] for name, data in association_data.items()})
    actual = {name[7:] for name in names if name.startswith('Assets/')}
    if actual != set(expected):
        raise ValueError('MSIX icon resource set differs')
    for name, size in expected.items():
        data = package.read('Assets/' + name)
        if png_size(data) != (size, size):
            raise ValueError('MSIX icon dimensions differ: ' + name)
        if name in association_data and data != association_data[name]:
            raise ValueError('MSIX association icon pixels differ: ' + name)
    return len(expected)
