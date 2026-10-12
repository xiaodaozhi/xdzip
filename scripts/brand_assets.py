"""One xdzip signature for UI glyphs and platform icon masters.

The four staggered ribbons and open bowl are redrawn from the supplied mark.
Platform shells and document folds are separate from the signature geometry;
small glyphs retain the signature while omitting large-icon lighting and text.
"""
from __future__ import annotations

MARK_VIEWBOX = (346, 325, 562, 674)
MARK_PATHS = (
    'M370 325H534Q548 325 558 338L575 354H639Q663 354 663 377V397Q663 420 639 420H442Q428 420 415 410L355 365Q346 358 346 348V346Q346 325 370 325Z',
    'M606 432H674Q687 432 695 420L704 411Q712 404 728 404H887Q907 404 907 421Q907 428 898 435L836 478Q822 490 803 490H606Q582 490 582 466V455Q582 432 606 432Z',
    'M446 499H639Q663 499 663 523V539Q663 562 639 562H583Q571 562 563 572L553 580Q547 584 535 584H370Q348 584 348 567Q348 557 361 548L419 508Q431 499 446 499Z',
    'M606 569H796Q810 569 821 577L893 620Q907 629 903 638Q901 647 887 647H716Q704 647 695 638L672 624H606Q582 624 582 600V593Q582 569 606 569Z',
    'M390 659H864Q907 659 907 701V792C907 906 826 999 715 999H535C426 999 347 914 347 803V701Q347 659 390 659Z M440 749H814V796C814 869 765 919 699 919H554C492 919 440 868 440 804Z',
)
EXPORT_SIZES = (16, 24, 32, 48, 64, 128, 256, 512, 1024)
ICO_SIZES = (16, 24, 32, 48, 64, 128, 256)
ICNS_CHUNKS = {
    'icp4': 16, 'icp5': 32, 'icp6': 64, 'ic07': 128,
    'ic08': 256, 'ic09': 512, 'ic10': 1024,
    'ic11': 32, 'ic12': 64, 'ic13': 256, 'ic14': 512,
}


def signature(x: float, y: float, width: float, fill: str = '#fff') -> str:
    """Place the five-piece signature without changing its aspect ratio."""
    scale = width / MARK_VIEWBOX[2]
    paths = ''.join(f'<path d="{path}" fill-rule="evenodd"/>' for path in MARK_PATHS)
    return (f'<g fill="{fill}" stroke="none" transform="translate({x} {y}) '
            f'scale({scale:.9f}) translate(-346 -325)">{paths}</g>')


def gradients(prefix: str, grey: bool = False) -> str:
    blue = ('#c1c6cf', '#979eaa', '#7b8493') if grey else ('#00beef', '#0068ff', '#153de2')
    return (f'<defs><linearGradient id="{prefix}-blue" x1=".15" y1="0" x2=".78" y2="1">'
            f'<stop stop-color="{blue[0]}"/><stop offset=".42" stop-color="{blue[1]}"/>'
            f'<stop offset="1" stop-color="{blue[2]}"/></linearGradient>'
            f'<linearGradient id="{prefix}-paper" x1="0" y1="0" x2=".85" y2="1">'
            '<stop stop-color="#fff"/><stop offset=".62" stop-color="#fbfcfe"/>'
            '<stop offset="1" stop-color="#e6edf7"/></linearGradient>'
            f'<linearGradient id="{prefix}-fold" x1="0" y1="0" x2="1" y2="1">'
            '<stop stop-color="#e3ebf5"/><stop offset=".35" stop-color="#fff"/>'
            '<stop offset="1" stop-color="#f7faff"/></linearGradient></defs>')


def brand_shapes(theme: str, document_profile: str = 'ui') -> dict[str, str]:
    """Return the fixed 24-unit UI family; colors do not alter its geometry."""
    if document_profile not in ('ui', 'tiny', 'small'):
        raise ValueError(f'Unknown document optical profile: {document_profile}')
    outline = '#c7d1e1' if theme == 'light' else '#98aac2'

    def document(grey: bool = False) -> str:
        prefix = 'archive-grey-brand' if grey else 'archive-brand'
        badge = {'ui': (6.8, 9, 10.4), 'tiny': (5.5, 7.9, 13), 'small': (6, 8.4, 12)}[document_profile]
        geometry = {'ui': (9.05, 10.65, 5.9), 'tiny': (7.1, 8.525, 9.8),
                    'small': (7.875, 9.453, 8.25)}[document_profile]
        mark = signature(*geometry)
        return (gradients(prefix, grey)
                + f'<path fill="url(#{prefix}-paper)" stroke="{outline}" stroke-width=".55" '
                  'd="M6.4 2.275H14.4L19.325 7.2V20.4Q19.325 21.725 18 21.725H6.4Q4.675 21.725 4.675 20V4Q4.675 2.275 6.4 2.275Z"/>'
                + f'<path fill="url(#{prefix}-fold)" stroke="{outline}" stroke-width=".45" '
                  'd="M14.4 2.45V5.9Q14.4 7.2 15.7 7.2H19.15Z"/>'
                + f'<rect x="{badge[0]}" y="{badge[1]}" width="{badge[2]}" height="{badge[2]}" rx="2.6" '
                  f'fill="url(#{prefix}-blue)" stroke="none"/>'
                + mark)

    return {
        'app-mark': gradients('app-brand')
        + '<rect x="2" y="2" width="20" height="20" rx="4.75" fill="url(#app-brand-blue)" stroke="none"/>'
        + '<path d="M6.75 2.55H17.25Q21.45 2.55 21.45 6.75" stroke="#7de4ff" stroke-opacity=".42" stroke-width=".45"/>'
        + signature(6.4, 5.285, 11.2),
        'archive': document(),
        'archive-grey': document(True),
    }


def master_svg(kind: str) -> str:
    """Produce a transparent 1024-pixel master, including native-safe margins."""
    prefix = 'xdzip-master'
    defs = gradients(prefix)
    effects = (
        '<defs><filter id="tile-shadow" x="-15%" y="-15%" width="130%" height="140%">'
        '<feDropShadow dx="0" dy="12" stdDeviation="12" flood-color="#183887" flood-opacity=".20"/></filter>'
        '<filter id="fold-shadow" x="-40%" y="-40%" width="180%" height="180%">'
        '<feDropShadow dx="-4" dy="7" stdDeviation="9" flood-color="#637897" flood-opacity=".20"/></filter>'
        '<linearGradient id="porcelain" x1="0" y1="0" x2=".85" y2="1">'
        '<stop stop-color="#fff"/><stop offset="1" stop-color="#f3f9ff"/></linearGradient></defs>'
    )
    if kind == 'application':
        content = (
            '<rect x="80" y="80" width="864" height="864" rx="198" '
            f'fill="url(#{prefix}-blue)" filter="url(#tile-shadow)"/>'
            '<rect x="84" y="84" width="856" height="856" rx="194" fill="none" stroke="#06baff" stroke-width="4" stroke-opacity=".55"/>'
            '<path d="M278 88H746Q936 88 936 278" fill="none" stroke="#8eeeff" stroke-width="5" stroke-opacity=".42"/>'
            + signature(274, 226.565, 476, 'url(#porcelain)')
        )
    elif kind == 'archive':
        content = (
            f'<path fill="url(#{prefix}-paper)" stroke="#bdcce0" stroke-width="1.7" filter="url(#tile-shadow)" '
            'd="M248 80H620L832 292V888Q832 944 776 944H248Q192 944 192 888V136Q192 80 248 80Z"/>'
            f'<path fill="url(#{prefix}-fold)" filter="url(#fold-shadow)" '
            'd="M620 80V232Q620 292 680 292H832Z"/>'
            '<path d="M212 162V878Q212 922 256 922H771" fill="none" stroke="#fff" stroke-width="5" stroke-opacity=".6"/>'
            f'<rect x="294" y="334" width="436" height="436" rx="110" fill="url(#{prefix}-blue)"/>'
            + signature(392, 409.82, 240, 'url(#porcelain)')
            + '<path d="M417 817H473L417 878H473 M507 817V878 M545 878V817H578C618 817 618 852 578 852H545" '
              'fill="none" stroke="#39465d" stroke-width="14" stroke-linejoin="round" stroke-linecap="square"/>'
        )
    else:
        raise ValueError(f'Unknown brand master: {kind}')
    return ('<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" '
            f'viewBox="0 0 1024 1024" fill="none">{defs}{effects}{content}</svg>\n')


def mark_svg() -> str:
    paths = ''.join(f'<path d="{path}" fill-rule="evenodd"/>' for path in MARK_PATHS)
    return ('<svg xmlns="http://www.w3.org/2000/svg" width="562" height="674" '
            f'viewBox="346 325 562 674" fill="#20232a">{paths}</svg>\n')


def png_name(stem: str, size: int) -> str:
    return f'{stem}.png' if size == 512 else f'{stem}-{size}.png'


def brand_export_specs() -> list[dict]:
    """Metadata consumed by the single canonical asset-manifest builder."""
    entries = [
        {'file': 'brand/mark.svg', 'kind': 'brand-master', 'purpose': 'signature-source',
         'size': [562, 674], 'provenance': 'vector redraw of the supplied signature'},
        {'file': 'brand/identity-reference.png', 'kind': 'brand-reference',
         'purpose': 'generated-visual-direction', 'usage': 'visual-reference-only',
         'provenance': 'generated high-fidelity application and document design sheet'},
    ]
    for stem, purpose in [('xdzip', 'application-icon'), ('xdzip-zip', 'archive-document-icon')]:
        source = f'brand/{stem}.svg'
        entries.append({'file': source, 'kind': 'brand-master', 'purpose': purpose,
                        'source': 'brand/mark.svg', 'size': [1024, 1024]})
        for size in EXPORT_SIZES:
            entries.append({'file': f'brand/{png_name(stem, size)}', 'kind': 'platform-icon',
                            'purpose': purpose, 'source': source, 'size': [size, size], 'format': 'png',
                            'raster_profile': ('Compact document at 16/24; balanced document at 32; master lighting at 48–1024'
                                               if stem == 'xdzip-zip' else 'UI detail at 16/24/32; master lighting at 48–1024')})
        entries.append({'file': f'brand/{stem}.ico', 'kind': 'platform-icon', 'purpose': purpose,
                        'source': source, 'format': 'ico', 'sizes': list(ICO_SIZES)})
        entries.append({'file': f'brand/{stem}.icns', 'kind': 'platform-icon', 'purpose': purpose,
                        'source': source, 'format': 'icns', 'chunks': ICNS_CHUNKS.copy()})
    return entries
