#!/usr/bin/env python3
"""Check the shared marketing capture scale against the committed README reference."""
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[1]
spec = json.loads((ROOT / 'specs/capture/marketing-screenshot-scale.json').read_text(encoding='utf-8'))
layout = json.loads((ROOT / 'specs/visual/layout-scale.json').read_text(encoding='utf-8'))
assert spec['schema_version'] == 1
assert spec['physical_pixels_per_qt_logical_pixel'] == 1.5
ref = spec['reference']
assert ref['logical_main_client'] == layout['main']['logical_window']
export_scale = spec['physical_pixels_per_qt_logical_pixel']
assert ref['native_scale'] >= export_scale
assert ref['native_main_client'] == [round(value * ref['native_scale']) for value in ref['logical_main_client']]
assert ref['published_image'] == [round(value * export_scale) for value in ref['logical_main_client']]
image = (ROOT / ref['file']).read_bytes()
assert image[:8] == b'\x89PNG\r\n\x1a\n' and image[12:16] == b'IHDR'
assert list(struct.unpack('>II', image[16:24])) == ref['published_image']
print('Marketing capture target: 150%; README reference dimensions match.')
