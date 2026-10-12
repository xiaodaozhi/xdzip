"""Canonical 24-unit icon definitions and immutable source-pixel crop coordinates.

Control glyphs retain the calibrated toolbar geometry. File glyphs share an
optical center and height; their natural widths are intentionally different.
These definitions describe reconstructed assets, never recovered alpha layers.
"""

import math

# These bounds include the outer stroke. Canvas alignment belongs to the asset,
# so consumers can use one 24-unit viewBox and never stretch individual types.
FILE_OPTICAL_SPECS = {
    'folder': {'bounds': [1, 2.571429, 23, 21.428571], 'center': [12, 12]},
    'file-pdf': {'bounds': [2.5, 1, 21.5, 23], 'center': [12, 12]},
    'file-text': {'bounds': [3.75, 1, 20.25, 23], 'center': [12, 12]},
    'file-code': {'bounds': [2, 1, 22, 23], 'center': [12, 12]},
    'file-markdown': {'bounds': [2, 1, 22, 23], 'center': [12, 12]},
    'image-file': {'bounds': [3.2, 1, 20.8, 23], 'center': [12, 12]},
    'image-amber': {'bounds': [3.2, 1, 20.8, 23], 'center': [12, 12]},
}

EXTRA_CONTROL_PATHS = {
    'plus': 'M12 5v14 M5 12h14',
    'shield': 'm12 3 8 3v6c0 5-8 9-8 9s-8-4-8-9V6z m-4 9 3 3 5-6',
    'panel': 'M3 4h18v16H3z M15 4v16',
    'activity': 'M3 12h4l3-7 4 14 3-7h4',
    'sun': 'M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8 M12 2v2 M12 20v2 M2 12h2 M20 12h2 M5 5l1 1 M18 18l1 1 M5 19l1-1 M18 6l1-1',
    'moon': 'M20 14A8 8 0 0 1 10 4a8 8 0 1 0 10 10',
    'warning': 'm12 3 10 18H2z M12 9v5 M12 17h.01',
}


def folder_shape():
    """Balance the folder silhouette with documents while preserving its aspect ratio.

    The 22-unit width gives the horizontal folder comparable visual area to
    the 19-by-22 PDF. Scale inside the shared asset, keeping every consumer's
    24-unit canvas, center and display slot unchanged.
    """
    scale = 22 / 17.5
    return (
        f'<g transform="translate(12 12) scale({scale:.9f}) translate(-12 -12)">'
        '<defs><linearGradient id="f" x1="0" y1="0" x2="0" y2="1"><stop stop-color="#6bbcff"/><stop offset="1" stop-color="#4099fa"/></linearGradient></defs>'
        '<path fill="#2473ff" stroke="#226bfb" stroke-width="1" d="M5.05 5h4.1l2.65 2.55h7.15a1.3 1.3 0 0 1 1.3 1.3v8.85a1.3 1.3 0 0 1-1.3 1.3H5.05a1.3 1.3 0 0 1-1.3-1.3V6.3a1.3 1.3 0 0 1 1.3-1.3Z"/>'
        '<rect x="3.75" y="8.1" width="16.5" height="10.9" rx="1.3" fill="url(#f)" stroke="#287cf3" stroke-width="1"/>'
        '<path d="M4.75 9.05h14.5" stroke="#d4efff" stroke-opacity=".72" stroke-width=".65"/>'
        '</g>'
    )


def file_shapes(theme):
    """Return file glyphs with shared height and deliberately natural widths."""
    paper = '#fff' if theme == 'light' else '#28313f'
    border = '#bac2d0' if theme == 'light' else '#8190a6'
    rule = '#929eb2' if theme == 'light' else '#a4b1c6'
    symbol = '#343b4c' if theme == 'light' else '#e4ebf5'
    return {
        'folder': folder_shape(),
        'file-pdf': '''<defs><linearGradient id="p" x1="0" y1="0" x2="0" y2="1"><stop stop-color="#ff4c50"/><stop offset=".55" stop-color="#fb2d34"/><stop offset="1" stop-color="#e91623"/></linearGradient></defs><path fill="url(#p)" stroke="#ed2430" stroke-width=".7" d="M4.55 1.35h9.4l7.2 7.1v12.2a2 2 0 0 1-2 2H4.55a1.7 1.7 0 0 1-1.7-1.7V3.05a1.7 1.7 0 0 1 1.7-1.7Z"/><path fill="#fff9fa" stroke="#e82c38" stroke-width=".6" d="M14 1.65v5.2a1.65 1.65 0 0 0 1.65 1.65h5.15Z"/><path d="M4.1 3.3v17a.9.9 0 0 0 .9.9h14.2" stroke="#ffb6b9" stroke-opacity=".5" stroke-width=".55"/><g stroke="#fff" stroke-width=".92"><path d="M5.8 18.8v-5.3h1.55a1.45 1.45 0 0 1 0 2.9H5.8 M10.7 13.5h1.2a1.8 1.8 0 0 1 1.8 1.8V17a1.8 1.8 0 0 1-1.8 1.8h-1.2Z M16.1 18.8v-5.3h2.55 M16.1 16h2.05"/></g>''',
        'file-text': f'''<path fill="{paper}" stroke="{border}" stroke-width="1" d="M5.8 1.5h8.55l5.4 5.4v14a1.6 1.6 0 0 1-1.6 1.6H5.8a1.55 1.55 0 0 1-1.55-1.55V3.05A1.55 1.55 0 0 1 5.8 1.5Z"/><path stroke="{border}" stroke-width="1" d="M14.35 1.75v4.1a1.05 1.05 0 0 0 1.05 1.05h4.1"/><path stroke="{rule}" stroke-width="1" d="M7.25 7.9h4.7 M7.25 10.85h9 M7.25 13.8h9 M7.25 16.75h9"/>''',
        'file-code': f'''<rect x="2.5" y="1.5" width="19" height="21" rx="2.15" fill="{paper}" stroke="{border}" stroke-width="1"/><path d="m8.45 9.1-2.9 2.9 2.9 2.9m7.1-5.8 2.9 2.9-2.9 2.9m-2.35-6.5-2.4 7.2" stroke="{symbol}" stroke-width="1.15"/>''',
        'file-markdown': f'''<rect x="2.5" y="1.5" width="19" height="21" rx="2.15" fill="{paper}" stroke="{border}" stroke-width="1"/><path d="M6.582 15.5V10l2.666 3.7 2.666-3.7v5.5 M16.386 10.35v5.1m-1.849-2.15 1.849 2.15 1.849-2.15" stroke="{symbol}" stroke-width="1.2"/>''',
    }


def rounded_gear_path():
    """An eight-tooth illustration gear with small rounded tooth corners.

    Each tooth follows the same angular profile. Keeping the radial definition
    here makes the source's eight-tooth silhouette explicit and easy to inspect.
    This illustration is separate from the calibrated toolbar settings glyph.
    """
    points = []
    for tooth in range(8):
        for offset, radius in ((-22.5, 6.2), (-13, 6.2), (-10, 8.15), (10, 8.15), (13, 6.2)):
            angle = math.radians(-90 + tooth * 45 + offset)
            points.append((12 + radius * math.cos(angle), 12 + radius * math.sin(angle)))

    def toward(point, neighbor):
        dx, dy = neighbor[0] - point[0], neighbor[1] - point[1]
        fraction = min(.26 / math.hypot(dx, dy), .3)
        return point[0] + dx * fraction, point[1] + dy * fraction

    commands = []
    for index, point in enumerate(points):
        entry = toward(point, points[index - 1])
        exit_point = toward(point, points[(index + 1) % len(points)])
        commands.append(f'{"M" if not commands else "L"}{entry[0]:.3f} {entry[1]:.3f}')
        commands.append(f'Q{point[0]:.3f} {point[1]:.3f} {exit_point[0]:.3f} {exit_point[1]:.3f}')
    return ' '.join(commands) + 'Z'


def hero_shapes():
    """Paper illustrations use layered light, a fold and an inset hairline."""
    body = 'M6 2h8l6 6v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2Z'

    def paper(color, fold):
        return (f'<path fill="url(#{color})" stroke="none" d="{body}"/>'
                f'<path fill="{fold}" stroke="none" d="M14 2v4.3A1.7 1.7 0 0 0 15.7 8H20Z"/>'
                '<path d="M5 5v14.8A1.2 1.2 0 0 0 6.2 21h11.6" stroke="white" stroke-opacity=".16" stroke-width=".3"/>'
                '<path d="M6 3h7.1" stroke="white" stroke-opacity=".28" stroke-width=".35"/>')

    # Colored paper folds are darker than the body in the approved source cuts;
    # the white PDF fold intentionally follows a different document convention.
    blue = paper('blue', '#1655e4')
    amber = paper('amber-warning', '#eea015')
    pale_amber = paper('amber-conflict', '#eea015')
    amber_archive = paper('amber-archive', '#eea015')
    amber_image = paper('amber-image', '#ffbe39')
    sky = paper('sky', '#69c1f2')
    zipper = '<path stroke="white" stroke-width=".7" d="M10 3.5v11 M9 4h2 M10 6h2 M9 8h2 M10 10h2 M9 12h2"/><rect x="9" y="14.5" width="3" height="4" rx=".75" stroke="white" stroke-width=".8"/>'
    mountain_blue = '<circle cx="8.1" cy="10.2" r="1.4" fill="#fff9d8" stroke="none"/><path fill="#3099e6" stroke="none" d="m4.5 17.2 4.8-4.8 4 3.7 3.1-3.2 3.1 3.6v3.3a1.7 1.7 0 0 1-1.7 1.7H6.2a1.7 1.7 0 0 1-1.7-1.7Z"/><path d="m7.4 14.3 1.9-1.9 2.3 2.1" stroke="#e7faff" stroke-width=".7"/>'
    mountain_amber = mountain_blue.replace('#3099e6', '#ee971e').replace('#e7faff', '#fff2c1')
    return {
        'archive-add': blue + zipper + '<circle cx="18.4" cy="18.4" r="4.45" fill="#2874ff" stroke="white" stroke-width="1.1"/><path stroke="white" stroke-width="1.3" d="M18.4 16.1v4.6 M16.1 18.4h4.6"/>',
        'archive-warning': amber_archive + zipper,
        'document-download': blue + '<path stroke="white" stroke-width="1.8" d="M12 10.6v6.4m-2.8-2.7 2.8 2.7 2.8-2.7"/>',
        'document-warning': amber + '<circle cx="12" cy="15.2" r="4.6" fill="#fff7dc" stroke="none"/><path stroke="#db860e" stroke-width="1.4" d="M12 12.6v2.9m0 2h.01"/>',
        'document-conflict': pale_amber + '<path fill="#f39318" stroke="#ffe5a4" stroke-width=".4" d="m10.9 10.8-4 7a1.3 1.3 0 0 0 1.1 2h8a1.3 1.3 0 0 0 1.1-2l-4-7a1.25 1.25 0 0 0-2.2 0Z"/><path stroke="white" stroke-width="1.2" d="M12 13.2v2.65m0 1.85h.01"/>',
        # Image documents appear in the same file slots as PDF/TXT/CODE/MD.
        # This authored group expands around the common center, never the DOM.
        'image-file': '<g transform="translate(-1.2 -1.2) scale(1.1)">' + sky + mountain_blue + '</g>',
        'image-amber': '<g transform="translate(-1.2 -1.2) scale(1.1)">' + amber_image + mountain_amber + '</g>',
        'settings-tile': '<rect x="1" y="1" width="22" height="22" rx="5" fill="url(#blue)" stroke="none"/><rect x="1.7" y="1.7" width="20.6" height="20.6" rx="4.3" stroke="#c4e5ff" stroke-opacity=".22" stroke-width=".35"/>'
                         f'<path fill="#e9f4ff" stroke="#cde4ff" stroke-width=".3" d="{rounded_gear_path()}"/>'
                         '<circle cx="12" cy="12" r="3" fill="#2975f5" stroke="#b6d9ff" stroke-width=".45"/>',
    }

CROPS = {'brand-titlebar': ('main-light', [136, 44, 31, 37]),
 'brand-extract': ('extract-macos', [125, 136, 104, 122]),
 'folder': ('main-light', [397, 302, 42, 41]),
 'file-pdf': ('main-light', [397, 626, 44, 50]),
 'file-text': ('main-light', [398, 694, 43, 49]),
 'file-code': ('main-light', [397, 760, 47, 48]),
 'file-markdown': ('main-light', [397, 829, 47, 51]),
 'photo-lake-preview': ('main-light', [1198, 241, 351, 277]),
 'photo-lake-thumbnail': ('main-light', [397, 494, 45, 45]),
 'photo-sunrise-thumbnail': ('main-light', [397, 560, 45, 46]),
 'open': ('main-light', [48, 111, 32, 32]),
 'new': ('main-light', [174, 111, 28, 32]),
 'archive-test': ('main-light', [317, 110, 34, 34]),
 'more': ('main-light', [448, 111, 31, 31]),
 'search': ('main-light', [920, 111, 32, 32]),
 'settings': ('main-light', [1515, 43, 36, 34]),
 'view-list': ('main-light', [1000, 182, 46, 40]),
 'view-grid': ('main-light', [1065, 185, 28, 30]),
 'sort': ('main-light', [1114, 185, 36, 30]),
 'close': ('main-light', [1524, 192, 28, 27]),
 'check-selected': ('main-light', [337, 501, 30, 31]),
 'success': ('extract-macos', [141, 501, 43, 43]),
 'radio-selected': ('extract-macos', [142, 654, 42, 42]),
 'radio-unselected': ('extract-macos', [142, 934, 42, 43]),
 'create-archive': ('create', [110, 130, 115, 125]),
 'image-blue': ('create', [144, 415, 54, 59]),
 'image-amber': ('create', [144, 485, 54, 60]),
 'remove-row': ('create', [886, 422, 49, 49]),
 'conflict-document': ('conflict', [130, 193, 116, 142]),
 'conflict-pdf': ('conflict', [156, 458, 109, 119]),
 'conflict-warning': ('conflict', [140, 681, 53, 53]),
 'progress-document': ('progress', [133, 140, 104, 119]),
 'progress-image': ('progress', [169, 794, 51, 61]),
 'partial-document': ('partial-result', [115, 143, 113, 130]),
 'partial-success': ('partial-result', [242, 306, 57, 57]),
 'partial-skipped': ('partial-result', [645, 307, 56, 56]),
 'partial-warning': ('partial-result', [1035, 306, 59, 58]),
 'settings-gear': ('settings', [115, 139, 119, 120]),
 'theme-system': ('settings', [337, 378, 109, 86]),
 'theme-light': ('settings', [568, 378, 109, 86]),
 'theme-dark': ('settings', [798, 378, 108, 86]),
 'success-large': ('task-states', [122, 107, 101, 99]),
 'stopping-spinner': ('task-states', [122, 634, 101, 100]),
 'cancelled-large': ('task-states', [122, 1077, 101, 98]),
 'overwrite-archive': ('create-overwrite', [122, 214, 138, 163]),
 'existing-archive': ('create-overwrite', [165, 504, 117, 138]),
 'windows-document': ('platforms', [139, 123, 77, 88]),
 'linux-document': ('platforms', [136, 852, 80, 89])}

CONTROL_PATHS = {'open': 'M3 8V5.5A2 2 0 0 1 5 3.5h4.2L12 6h7a2 2 0 0 1 2 2v10.5a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8Z M3 9h18',
 'new': 'M6 3h12a3 3 0 0 1 3 3v12a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3Z M12 8v8 M8 12h8',
 'archive-test': 'M6 3h12a3 3 0 0 1 3 3v12a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3Z M7.5 12l3 3 6-6',
 'search': 'M17 10a7 7 0 1 1-14 0 7 7 0 0 1 14 0 M15 15l6 6',
 'close': 'M6 6l12 12 M18 6L6 18',
 'chevron-down': 'M6 9l6 6 6-6',
 'chevron-right': 'M9 6l6 6-6 6',
 'chevron-up': 'M6 15l6-6 6 6',
 'check': 'M5 12l4 4L19 6',
 'arrow-right': 'M4 12h16 M14 6l6 6-6 6',
 'extract': 'M12 3v12 M7 10l5 5 5-5 M4 15v6h16v-6',
 'view-list': 'M5 6h14 M5 12h14 M5 18h14',
 'view-grid': 'M4 4h6v6H4Z M14 4h6v6h-6Z M4 14h6v6H4Z M14 14h6v6h-6Z',
 'sort': 'M5 5v14 M2 16l3 3 3-3 M12 6h9 M12 12h6 M12 18h3',
 'sort-ascending': 'M12 20V4 M6 10l6-6 6 6',
 'completed': 'M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0 M7 12l3 3 7-7',
 'external-open': 'M14 3h7v7 M21 3L10 14 M10 5H4v16h16v-6',
 'minimize': 'M5 12h14',
 'maximize': 'M5 5h14v14H5z',
 'settings': 'M10 3.2c0-.7.5-1.2 1.2-1.2h1.6c.7 0 1.2.5 1.2 1.2v1.3l1.9 1.1 1.1-.6c.6-.4 1.3-.2 1.7.4l.8 '
             '1.4c.4.6.2 1.3-.4 1.7l-1.1.6v2.2l1.1.6c.6.4.8 1.1.4 1.7l-.8 1.4c-.4.6-1.1.8-1.7.4l-1.1-.6-1.9 '
             '1.1v1.3c0 .7-.5 1.2-1.2 1.2h-1.6c-.7 '
             '0-1.2-.5-1.2-1.2v-1.3l-1.9-1.1-1.1.6c-.6.4-1.3.2-1.7-.4l-.8-1.4c-.4-.6-.2-1.3.4-1.7l1.1-.6V9.1L4.9 '
             '8.5c-.6-.4-.8-1.1-.4-1.7l.8-1.4c.4-.6 1.1-.8 1.7-.4l1.1.6L10 4.5Z M15 10.2a3 3 0 1 1-6 0 3 3 0 '
             '0 1 6 0'}

MAIN_SHAPES = {'archive': '<defs><linearGradient id="a" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#409cff"/><stop '
            'offset="1" stop-color="#164fff"/></linearGradient></defs><path fill="url(#a)" stroke="none" '
            'd="M6 2h8l6 6v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2Z"/><path fill="#1650da" '
            'stroke="none" d="M14 2v4a2 2 0 0 0 2 2h4Z"/><path stroke="white" stroke-width=".8" d="M11 '
            '3.5v11 M10 4h2 M11 6h2 M10 8h2 M11 10h2 M10 12h2"/><rect x="10.1" y="14.5" width="3.2" '
            'height="4.5" rx=".8" fill="none" stroke="white" stroke-width="1"/>',
 'task-list': '<g fill="{ink}" stroke="none"><circle cx="4" cy="6" r=".8"/><circle cx="4" cy="12" '
              'r=".8"/><circle cx="4" cy="18" r=".8"/></g><path d="M9 6h10 M9 12h10 M9 18h10"/>',
 'more': '<circle cx="5" cy="12" r="1.5" fill="{ink}" stroke="none"/><circle cx="12" cy="12" r="1.5" '
         'fill="{ink}" stroke="none"/><circle cx="19" cy="12" r="1.5" fill="{ink}" stroke="none"/>',
 'app-mark': '<defs><linearGradient id="a" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#409cff"/><stop '
             'offset="1" stop-color="#164fff"/></linearGradient></defs><rect x="3" y="2" width="18" '
             'height="20" rx="3" fill="url(#a)" stroke="none"/><path stroke="white" stroke-width=".8" d="M11 '
             '3.5v11 M10 4h2 M11 6h2 M10 8h2 M11 10h2 M10 12h2"/><rect x="10.1" y="14.5" width="3.2" '
             'height="4.5" rx=".8" fill="none" stroke="white" stroke-width="1"/>'}

STATE_SHAPES = {'status-warning': '<circle cx="12" cy="12" r="10" fill="#eda100" stroke="none"/><path stroke="white" '
                   'stroke-width="2" d="M12 6v7m0 4h.01"/>',
 'status-success': '<circle cx="12" cy="12" r="10" fill="#348c44" stroke="none"/><path stroke="white" '
                   'stroke-width="2" d="m6 12 4 4 8-8"/>',
 'status-skipped': '<circle cx="12" cy="12" r="10" fill="{muted}" stroke="none"/><path stroke="white" '
                   'stroke-width="2" d="M7 12h10"/>',
 'status-stopped': '<circle cx="12" cy="12" r="10" fill="{muted}" stroke="none"/><rect x="8" y="8" width="8" '
                   'height="8" rx="1.5" fill="white" stroke="none"/>',
 'spinner': '<circle cx="12" cy="12" r="9" stroke="{track}" stroke-width="2.5"/><path d="M12 3a9 9 0 0 1 9 '
            '9" stroke="{muted}" stroke-width="2.5"/>',
 'info': '<g stroke="{muted}"><circle cx="12" cy="12" r="9"/><path d="M12 11v6m0-10h.01"/></g>',
 'remove': '<circle cx="12" cy="12" r="9" stroke="{track}"/><path d="M8 12h8"/>',
 'clock': '<circle cx="12" cy="12" r="9"/><path d="M12 6v7l4 2"/>',
 'theme-system': '<defs><clipPath id="window"><rect x="2" y="4" width="20" height="16" '
                 'rx="2"/></clipPath></defs><g clip-path="url(#window)"><rect x="2" y="4" width="20" '
                 'height="16" fill="{paper}" stroke="none"/><path d="M12 4h10v16H12Z" fill="#185eb2" '
                 'stroke="none"/><path d="M2 4h20v5H2Z" fill="#cfd9e6" fill-opacity=".45" '
                 'stroke="none"/></g><rect x="2" y="4" width="20" height="16" rx="2" stroke="{muted}" '
                 'stroke-width=".7"/><g fill="{muted}" stroke="none"><circle cx="5" cy="6.5" r=".7"/><circle '
                 'cx="8" cy="6.5" r=".7"/></g>',
 'status-partial-success': '<circle cx="12" cy="12" r="10" fill="#1764ff" stroke="none"/><path '
                           'stroke="white" stroke-width="2" d="m6 12 4 4 8-8"/>'}

REFERENCES = {'app-mark': 'brand-titlebar',
 'archive': 'brand-extract',
 'folder': 'folder',
 'file-pdf': 'file-pdf',
 'file-text': 'file-text',
 'file-code': 'file-code',
 'file-markdown': 'file-markdown',
 'open': 'open',
 'new': 'new',
 'archive-test': 'archive-test',
 'more': 'more',
 'search': 'search',
 'settings': 'settings',
 'view-list': 'view-list',
 'view-grid': 'view-grid',
 'sort': 'sort',
 'close': 'close',
 'document-conflict': 'conflict-document',
 'image-amber': 'image-amber',
 'status-partial-success': 'partial-success',
 'archive-warning': 'overwrite-archive',
 'archive-add': 'create-archive',
 'document-download': 'progress-document',
 'document-warning': 'partial-document',
 'image-file': 'image-blue',
 'status-warning': 'partial-warning',
 'status-success': 'success-large',
 'status-skipped': 'partial-skipped',
 'status-stopped': 'cancelled-large',
 'spinner': 'stopping-spinner',
 'remove': 'remove-row',
 'theme-system': 'theme-system',
 'settings-tile': 'settings-gear'}
