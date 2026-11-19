# scripts/menu_shadow_assets.py

"""Generate the menu and Linux window nine-slice shadows from visual parameters."""
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

REPO = Path(__file__).resolve().parents[1]
ROOT = REPO / 'design/assets'


def shadow_entry(target, kind, source, theme, color, corner, inset, blur, offset, check):
    red, green, blue, alpha = color
    edge = inset + corner
    # A wide center keeps the authored blur independent of both corners.
    size = edge * 2 + blur * 6
    mask = Image.new('L', (size, size))
    ImageDraw.Draw(mask).rounded_rectangle(
        (inset, inset + offset, size - inset - 1, size - inset - 1 + offset),
        radius=corner, fill=alpha)
    mask = mask.filter(ImageFilter.GaussianBlur(blur))
    pixels = Image.new('RGBA', mask.size, (red, green, blue, 0))
    pixels.putalpha(mask)
    if check:
        with Image.open(target) as recorded:
            if recorded.convert('RGBA').tobytes() != pixels.tobytes():
                raise ValueError(f'Stale shadow: {target.relative_to(ROOT)}')
    else:
        target.parent.mkdir(parents=True, exist_ok=True)
        pixels.save(target)
    return {
        'file': target.relative_to(ROOT).as_posix(), 'kind': kind,
        'theme': theme, 'size': list(pixels.size), 'border': edge, 'inset': inset,
        'usage': 'runtime-and-design', 'source': '../../' + source,
        'source_sha256': hashlib.sha256((REPO / source).read_bytes()).hexdigest(),
        'sha256': hashlib.sha256(target.read_bytes()).hexdigest(),
    }


def menu_shadow_entries(check=False):
    source = 'specs/visual/controls.json'
    controls = json.loads((REPO / source).read_text(encoding='utf-8'))
    spec = controls['menuShadowAsset']
    entries = []
    for theme in ('light', 'dark'):
        color = controls[theme]['menuShadow'].lstrip('#')
        rgba = tuple(int(color[index:index + 2], 16) for index in range(0, 8, 2))
        for scale in spec['fontScales']:
            corner = round(controls['size']['menuRadius'] * scale)
            target = ROOT / 'menu' / f'{theme}-{round(scale * 100)}.png'
            entry = shadow_entry(target, 'menu-shadow-rasterization', source, theme, rgba,
                                 corner, spec['inset'], spec['blurRadius'], spec['offsetY'], check)
            entries.append(dict(entry, id=f'menu-shadow/{theme}/{round(scale * 100)}',
                                font_scale=scale))
    return entries


def window_shadow_entries(check=False):
    source = 'specs/visual/layout-scale.json'
    spec = json.loads((REPO / source).read_text(encoding='utf-8'))['main']['linux_frame']
    entries = []
    for theme in ('light', 'dark'):
        alpha = round(spec['shadowOpacity' + theme.title()] * 255)
        target = ROOT / 'frame' / f'linux-{theme}.png'
        entry = shadow_entry(target, 'window-shadow-rasterization', source, theme, (0, 0, 0, alpha),
                             spec['cornerRadius'], spec['shadowInset'], spec['shadowBlur'],
                             spec['shadowOffsetY'], check)
        entries.append(dict(entry, id=f'linux-window-shadow/{theme}'))
    return entries
