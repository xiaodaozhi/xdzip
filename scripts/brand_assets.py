"""One xdzip signature for UI glyphs and platform icon masters.

The four staggered ribbons and open bowl are redrawn from the supplied mark.
Platform shells and document folds are separate from the signature geometry;
small glyphs retain the signature while omitting large-icon lighting and text.
"""
from __future__ import annotations

from math import sqrt

MARK_VIEWBOX = (407, 325, 440, 660)
MARK_PATHS = (
    'M431 325H550Q561 325 570 337L584 353H634Q657 353 657 376V393Q657 416 634 416H486Q475 416 466 408L418 365Q409 357 409 348V346Q409 325 431 325Z',
    'M615 432H665Q676 432 684 418L692 408Q699 402 710 402H823Q843 402 843 420Q843 428 835 435L781 480Q771 489 755 489H615Q592 489 592 466V455Q592 432 615 432Z',
    'M485 497H634Q657 497 657 521V536Q657 559 634 559H590Q580 559 572 571L565 579Q559 583 546 583H431Q409 583 409 565Q409 557 420 547L465 507Q474 497 485 497Z',
    'M615 568H752Q765 568 776 577L834 619Q845 627 842 637Q839 646 825 646H699Q689 646 681 638L665 625H615Q592 625 592 602V591Q592 568 615 568Z',
    'M449 658H805Q847 658 847 700V785C847 896 758 985 638 985H615C495 985 407 896 407 785V700Q407 658 449 658Z M481 747H773V783C773 858 715 915 639 915H615C539 915 481 858 481 783Z',
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
    origin_x, origin_y = MARK_VIEWBOX[:2]
    paths = ''.join(f'<path d="{path}" fill-rule="evenodd"/>' for path in MARK_PATHS)
    return (f'<g fill="{fill}" stroke="none" transform="translate({x} {y}) '
            f'scale({scale:.9f}) translate(-{origin_x} -{origin_y})">{paths}</g>')


def continuous_square_path(x: float, y: float, side: float) -> str:
    """Use the same G2-continuous contour for every tile and document badge.

    Each corner uses two cubic segments. Its midpoint has matching tangent and
    curvature; curvature reaches zero where it meets a straight side. Rotated
    control points keep all four corners identical. Lighting uses this path too,
    never a separately approximated rounded rectangle or open highlight stroke.
    """
    def point(px: float, py: float) -> str:
        return f'{x + px * side:.12g} {y + py * side:.12g}'

    # Make curvature stationary at the diagonal midpoint. A low midpoint
    # curvature would create a flat bevel between two rounded shoulders.
    control = (4.95 - sqrt(.1785)) / 6
    opposite = 1 - control
    return (f'M{point(.3, 0)}H{x + .7 * side:.12g}'
            f'C{point(control, 0)} {point(.86, 0)} {point(.93, .07)}'
            f'C{point(1, .14)} {point(1, opposite)} {point(1, .3)}V{y + .7 * side:.12g}'
            f'C{point(1, control)} {point(1, .86)} {point(.93, .93)}'
            f'C{point(.86, 1)} {point(control, 1)} {point(.7, 1)}H{x + .3 * side:.12g}'
            f'C{point(opposite, 1)} {point(.14, 1)} {point(.07, .93)}'
            f'C{point(0, .86)} {point(0, control)} {point(0, .7)}V{y + .3 * side:.12g}'
            f'C{point(0, opposite)} {point(0, .14)} {point(.07, .07)}'
            f'C{point(.14, 0)} {point(opposite, 0)} {point(.3, 0)}Z')


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
        # Preserve the optical height, not the old wider signature's width.
        height = {'ui': 7.08, 'tiny': 11.75, 'small': 9.894}[document_profile]
        width = height * MARK_VIEWBOX[2] / MARK_VIEWBOX[3]
        mark = signature(badge[0] + (badge[2] - width) / 2,
                         badge[1] + (badge[2] - height) / 2, width)
        return (gradients(prefix, grey)
                + f'<path fill="url(#{prefix}-paper)" stroke="{outline}" stroke-width=".55" '
                  'd="M6.4 2.275H14.4L19.325 7.2V20.4Q19.325 21.725 18 21.725H6.4Q4.675 21.725 4.675 20V4Q4.675 2.275 6.4 2.275Z"/>'
                + f'<path fill="url(#{prefix}-fold)" stroke="{outline}" stroke-width=".45" '
                  'd="M14.4 2.45V5.9Q14.4 7.2 15.7 7.2H19.15Z"/>'
                + f'<path d="{continuous_square_path(*badge)}" '
                  f'fill="url(#{prefix}-blue)" stroke="none"/>'
                + mark)

    return {
        'app-mark': gradients('app-brand')
        + f'<path d="{continuous_square_path(2, 2, 20)}" fill="url(#app-brand-blue)" stroke="none"/>'
        + signature(7.5, 5.25, 9),
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
        '<stop stop-color="#fff"/><stop offset="1" stop-color="#f3f9ff"/></linearGradient>'
        '<linearGradient id="tile-surface" x1="0" y1="0" x2="0" y2="1">'
        '<stop stop-color="#fff" stop-opacity=".08"/>'
        '<stop offset=".45" stop-color="#fff" stop-opacity="0"/>'
        '<stop offset="1" stop-color="#09255b" stop-opacity=".05"/></linearGradient></defs>'
    )
    if kind == 'application':
        content = (
            f'<path id="app-tile" d="{continuous_square_path(80, 80, 864)}" '
            f'fill="url(#{prefix}-blue)" filter="url(#tile-shadow)"/>'
            f'<path d="{continuous_square_path(80, 80, 864)}" fill="url(#tile-surface)"/>'
            + signature(322, 227, 380, 'url(#porcelain)')
        )
    elif kind == 'archive':
        content = (
            f'<path fill="url(#{prefix}-paper)" stroke="#bdcce0" stroke-width="1.7" filter="url(#tile-shadow)" '
            'd="M248 80H620L832 292V888Q832 944 776 944H248Q192 944 192 888V136Q192 80 248 80Z"/>'
            f'<path fill="url(#{prefix}-fold)" filter="url(#fold-shadow)" '
            'd="M620 80V232Q620 292 680 292H832Z"/>'
            '<path d="M212 162V878Q212 922 256 922H771" fill="none" stroke="#fff" stroke-width="5" stroke-opacity=".6"/>'
            f'<path d="{continuous_square_path(294, 334, 436)}" fill="url(#{prefix}-blue)"/>'
            + signature(416, 408, 192, 'url(#porcelain)')
            + '<path d="M417 817H473L417 878H473 M507 817V878 M545 878V817H578C618 817 618 852 578 852H545" '
              'fill="none" stroke="#39465d" stroke-width="14" stroke-linejoin="round" stroke-linecap="square"/>'
        )
    else:
        raise ValueError(f'Unknown brand master: {kind}')
    return ('<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" '
            f'viewBox="0 0 1024 1024" fill="none">{defs}{effects}{content}</svg>\n')


def mark_svg() -> str:
    paths = ''.join(f'<path d="{path}" fill-rule="evenodd"/>' for path in MARK_PATHS)
    x, y, width, height = MARK_VIEWBOX
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
            f'viewBox="{x} {y} {width} {height}" fill="#20232a">{paths}</svg>\n')


def png_name(stem: str, size: int) -> str:
    return f'{stem}.png' if size == 512 else f'{stem}-{size}.png'


def brand_export_specs() -> list[dict]:
    """Metadata consumed by the single canonical asset-manifest builder."""
    entries = [
        {'file': 'brand/mark.svg', 'kind': 'brand-master', 'purpose': 'signature-source',
         'size': list(MARK_VIEWBOX[2:]), 'provenance': 'vector redraw of the revised narrow signature'},
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
