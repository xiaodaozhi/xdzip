# scripts/menu_shadow_assets.py

"""Generate the shared menu's nine-slice blur from visual control parameters."""
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

REPO = Path(__file__).resolve().parents[1]
ROOT = REPO / 'design/assets'


def menu_shadow_entries(check=False):
    controls = json.loads((REPO / 'docs/visual/controls.json').read_text(encoding='utf-8'))
    spec = controls['menuShadowAsset']
    entries = []
    for theme in ('light', 'dark'):
        color = controls[theme]['menuShadow'].lstrip('#')
        red, green, blue, alpha = (int(color[index:index + 2], 16) for index in range(0, 8, 2))
        for scale in spec['fontScales']:
            corner = round(controls['size']['menuRadius'] * scale)
            edge = spec['inset'] + corner
            # A wide center keeps the authored blur independent of both corners.
            size = edge * 2 + spec['blurRadius'] * 6
            mask = Image.new('L', (size, size))
            ImageDraw.Draw(mask).rounded_rectangle(
                (spec['inset'], spec['inset'] + spec['offsetY'],
                 size - spec['inset'] - 1, size - spec['inset'] - 1 + spec['offsetY']),
                radius=corner, fill=alpha)
            mask = mask.filter(ImageFilter.GaussianBlur(spec['blurRadius']))
            pixels = Image.new('RGBA', mask.size, (red, green, blue, 0))
            pixels.putalpha(mask)
            target = ROOT / 'menu' / f'{theme}-{round(scale * 100)}.png'
            if check:
                with Image.open(target) as recorded:
                    if recorded.convert('RGBA').tobytes() != pixels.tobytes():
                        raise ValueError(f'Stale menu shadow: {target.relative_to(ROOT)}')
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                pixels.save(target)
            entries.append({
                'id': f'menu-shadow/{theme}/{round(scale * 100)}',
                'file': target.relative_to(ROOT).as_posix(), 'kind': 'menu-shadow-rasterization',
                'theme': theme, 'font_scale': scale, 'size': list(pixels.size),
                'border': edge, 'inset': spec['inset'], 'usage': 'runtime-and-design',
                'source': '../../docs/visual/controls.json',
                'source_sha256': hashlib.sha256((REPO / 'docs/visual/controls.json').read_bytes()).hexdigest(),
                'sha256': hashlib.sha256(target.read_bytes()).hexdigest(),
            })
    return entries
