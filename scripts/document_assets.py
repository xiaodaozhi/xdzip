"""Folded-paper illustrations derived from the shared archive artwork."""
from brand_assets import brand_shapes, continuous_square_path, document_paper, gradients


def status_badge(color: str, mark: str, ink: str = '#fff') -> str:
    return (
        f'<circle cx="18.4" cy="18.8" r="4.1" fill="{color}" stroke="#fff" stroke-width=".9"/>'
        f'<path d="{mark}" fill="none" stroke="{ink}" stroke-width="1.15"/>'
    )


def corner_badge(color: str, symbol: str) -> str:
    return (
        '<g transform="translate(18.65 18.7)">'
        f'<circle r="4.5" fill="{color}" stroke="#fff" stroke-width=".9"/>{symbol}</g>'
    )


def document_illustrations(theme: str) -> dict[str, str]:
    outline = '#c7d1e1' if theme == 'light' else '#98aac2'
    archive = brand_shapes(theme)['archive']
    corner_document = brand_shapes(theme, document_profile='corner-badge')['archive']
    warning = status_badge('#f6bb55', 'M18.4 16.8v2.25m0 1.55h.01', '#5c340c')
    conflict = corner_badge(
        '#f6bb55',
        '<path d="M0 -2.2V.35" stroke="#5c340c" stroke-width="1.75"/>'
        '<circle cy="2.1" r=".85" fill="#5c340c" stroke="none"/>',
    )
    windows_download = corner_badge(
        '#2d64da',
        '<path d="M-.88 -2.65H.88V.1H2.6L0 2.72 -2.6 .1H-.88Z" fill="#fff" stroke="none"/>',
    )
    download = 'M18.4 16.65v3.6m-1.55-1.5 1.55 1.5 1.55-1.5'
    tray = download + 'M16.7 21h3.4'
    pdf = (
        '<g transform="translate(-1.2 -1.2) scale(1.1)">'
        + gradients('pdf') + document_paper('pdf', outline)
        + f'<path d="{continuous_square_path(6.8, 9, 10.4)}" fill="#bd354b" stroke="none"/>'
        + '<path d="M8.25 16v-3.6h1.05a.95.95 0 0 1 0 1.9H8.25'
          ' M11.1 16v-3.6h.75a1.2 1.2 0 0 1 1.2 1.2v1.2a1.2 1.2 0 0 1-1.2 1.2H11.1Z'
          ' M14.05 16v-3.6h1.95m-1.95 1.8h1.5" stroke="#fff" stroke-width=".62"/></g>'
    )
    back = (
        '<path d="M5 4.6H14L18.2 8.8V21Q18.2 22.55 16.65 22.55H5'
        'Q3 22.55 3 20.55V6.6Q3 4.6 5 4.6Z" fill="#dfe8f7" '
        f'stroke="{outline}" stroke-width=".55"/>'
    )
    return {
        'archive-add': archive + status_badge('#2d64da', 'M18.4 16.7v4.2m-2.1-2.1h4.2'),
        'document-check': archive + status_badge('#267758', 'm16.5 18.85 1.2 1.3 2.7-2.8'),
        'document-download': archive + status_badge('#2d64da', download),
        'existing-archive': back + archive,
        'file-pdf': pdf,
        'windows-document': corner_document + windows_download,
        'linux-document': archive + status_badge('#177a85', tray),
        'conflict-pdf': pdf,
        'document-conflict': corner_document + conflict,
        'archive-warning': back + archive + warning,
        'document-warning': archive + warning,
    }
