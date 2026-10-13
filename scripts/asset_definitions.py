"""Canonical 24-unit icon definitions and immutable source-pixel crop coordinates.

Control glyphs retain the calibrated toolbar geometry. File glyphs share an
optical center and height; their natural widths are intentionally different.
These definitions describe reconstructed assets, never recovered alpha layers.
"""

import math
from directory_tree_assets import TREE_CUTS

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


def status_disc(name, highlight, body, shade, mark, radius=10):
    """A transparent result medallion with source-like light and a fine rim."""
    return (
        f'<defs><radialGradient id="{name}-disc" cx=".27" cy=".2" r=".86">'
        f'<stop stop-color="{highlight}"/><stop offset=".48" stop-color="{body}"/>'
        f'<stop offset="1" stop-color="{shade}"/></radialGradient></defs>'
        f'<circle cx="12" cy="12" r="{radius}" fill="url(#{name}-disc)" stroke="none"/>'
        f'<circle cx="12" cy="12" r="{radius - .27:.2f}" fill="none" '
        f'stroke="{shade}" stroke-opacity=".48" stroke-width=".38"/>'
        '<path d="M5.8 6.35a8.45 8.45 0 0 1 10.8-1.5" '
        'stroke="white" stroke-opacity=".17" stroke-width=".28"/>'
        + mark
    )


def warning_mark():
    """The source's compact exclamation occupies about half the disc height."""
    return ('<path d="M12 7.3v5.3" stroke="white" stroke-width="1.65"/>'
            '<circle cx="12" cy="16.7" r=".825" fill="white" stroke="none"/>')


def supplemental_shapes(theme):
    """Rebuild the remaining source references as independent, named assets.

    Theme thumbnails describe the selected theme, so their illustrated colors
    remain stable in both host themes. Control-state assets document appearance;
    real controls still own their interaction, focus and checked state.
    """
    gray = '#8992a4' if theme == 'light' else '#adb7c9'

    def gradient(name, top, bottom):
        return (f'<defs><linearGradient id="{name}" x1="0" y1="0" x2="0" y2="1">'
                f'<stop stop-color="{top}"/><stop offset="1" stop-color="{bottom}"/>'
                '</linearGradient></defs>')

    def window_preview(name, dark, system=False):
        background, right, title, border, dot = (
            ('#2c323f', '#383f4c', '#282e3b', '#212735', '#8b929e') if dark else
            ('#ffffff', '#f3f4f6', '#eceef2', '#c5cbd5', '#c9ced7'))
        highlight_color, highlight_opacity = ('#8791a7', '.22') if dark else ('#ffffff', '.8')
        side_color = '#121a29' if dark else '#9aa4b6'
        return (
            f'<defs><clipPath id="{name}-clip"><rect x="2" y="4.75" width="20" height="14.5" rx="1.55"/></clipPath>'
            f'<linearGradient id="{name}-body" x2="0" y2="1"><stop stop-color="{background}"/>'
            f'<stop offset="1" stop-color="{right}"/></linearGradient>'
            f'<radialGradient id="{name}-shadow"><stop stop-color="#17243b" stop-opacity=".22"/>'
            '<stop offset=".56" stop-color="#17243b" stop-opacity=".09"/>'
            '<stop offset="1" stop-color="#17243b" stop-opacity="0"/></radialGradient></defs>'
            + (gradient('theme-system-split', '#3f9dff', '#165795') if system else '')
            + f'<ellipse cx="12" cy="19.65" rx="10.6" ry="2.1" fill="url(#{name}-shadow)" stroke="none"/>'
            + f'<g clip-path="url(#{name}-clip)" stroke="none">'
            f'<rect x="2" y="4.75" width="20" height="14.5" fill="url(#{name}-body)"/>'
            + (f'<path fill="url(#theme-system-split)" d="M12 4.75h10v14.5H12Z"/>' if system else
               f'<path fill="{right}" d="M12 9.3h10v10H12Z"/>')
            + f'<path fill="{title}" d="M2 4.75h{10 if system else 20}V9.3H2Z"/>'
            f'<path d="M2 9.3h20" stroke="{border}" stroke-width=".3"/>'
            f'<circle cx="4.7" cy="7" r=".9" fill="{dot}"/>'
            f'<circle cx="8" cy="7" r=".9" fill="{dot}"/></g>'
            f'<rect x="2" y="4.75" width="20" height="14.5" rx="1.55" stroke="{border}" stroke-width=".45"/>'
            f'<path d="M3.2 5.35H20.6" stroke="{highlight_color}" '
            f'stroke-opacity="{highlight_opacity}" stroke-width=".35"/>'
            f'<path d="M21.55 6.6v10.8q0 1.35-1.35 1.35H3.8" stroke="{side_color}" '
            'stroke-opacity=".25" stroke-width=".35"/>'
        )

    def paper_seam(name, tint, left=14.2, top=1, bottom=7.8, right=21):
        # The narrow fold seam is transparent and stays inside the paper. It
        # supplies depth without baking a white or dark surrounding rectangle.
        return (
            f'<defs><linearGradient id="{name}-seam" x1="0" y1="0" x2="1" y2="1">'
            f'<stop stop-color="{tint}" stop-opacity=".06"/>'
            f'<stop offset=".5" stop-color="{tint}" stop-opacity=".32"/>'
            f'<stop offset="1" stop-color="{tint}" stop-opacity=".08"/></linearGradient></defs>'
            f'<path d="M{left} {top + 1.2}v{bottom - top - 3.05:.2f}q0 1.85 1.85 1.85H{right - .45}" '
            f'stroke="url(#{name}-seam)" stroke-width=".65"/>'
        )

    def platform_document(name, dark):
        # The source Linux illustration sits on a dark panel; only the blue
        # paper is rebuilt, never the baked dark rectangle around the source.
        top, bottom, fold = ('#47a6ff', '#0758ee', '#164cd6') if dark else ('#4da8ff', '#0867ff', '#1257ef')
        return (
            gradient(f'{name}-paper', top, bottom)
            + f'<path fill="url(#{name}-paper)" stroke="none" d="M5.1 1h9.1L21 7.8v13.1a2.1 2.1 0 0 1-2.1 2.1H5.1A2.1 2.1 0 0 1 3 20.9V3.1A2.1 2.1 0 0 1 5.1 1Z"/>'
            + f'<path fill="{fold}" stroke="none" d="M14.2 1v5.05a1.75 1.75 0 0 0 1.75 1.75H21Z"/>'
            + paper_seam(name, '#092e8e')
            + '<path d="M4 4.3v16.2a1.4 1.4 0 0 0 1.4 1.4h12.9" stroke="#bde5ff" stroke-opacity=".16" stroke-width=".3"/>'
            + '<path d="M12 10.25v7.1m-3.05-2.9L12 17.35l3.05-2.9" stroke="white" stroke-width="2.05"/>'
        )

    existing_archive = (
        gradient('existing-archive-paper', '#49a6ff', '#1455fa')
        + '<path fill="url(#existing-archive-paper)" stroke="none" d="M5.2 1h9L21 7.8v12.9a2.3 2.3 0 0 1-2.3 2.3H5.3A2.3 2.3 0 0 1 3 20.7V3.2A2.2 2.2 0 0 1 5.2 1Z"/>'
        + '<path fill="#1450ed" stroke="none" d="M14.2 1v4.8a2 2 0 0 0 2 2H21Z"/>'
        + paper_seam('existing-archive', '#09287c')
        + '<path d="M11.3 2.15v11.8 M10.2 2.7h2.7 M11.3 4.7H14 M10.2 6.7h2.7 M11.3 8.7H14 M10.2 10.7h2.7" stroke="white" stroke-width=".88" stroke-linecap="square"/>'
        + '<path fill="#f8fcff" stroke="none" d="M10.7 12.9h1.7a1 1 0 0 1 1 1v1.2l1.7 1v4.65a1.1 1.1 0 0 1-1.1 1.1H8.4a1.1 1.1 0 0 1-1.1-1.1V16.1l2.4-1v-1.2a1 1 0 0 1 1-1Z"/>'
        + '<rect x="10.6" y="13.8" width="1.6" height="1.4" rx=".35" fill="#2475ff" stroke="none"/>'
        # ZIP is drawn as paths, preserving the source pull-tab lettering without
        # depending on a font being installed in Qt or the browser.
        + '<path d="M8.6 17.8h1.8l-1.8 2.6h1.8 M11.65 17.8v2.6 M13 20.4v-2.6h.85a.75.75 0 0 1 0 1.5H13" stroke="#1764f4" stroke-width=".48"/>'
    )
    conflict_pdf = (
        gradient('conflict-pdf-paper', '#ff5754', '#f22323')
        + '<path fill="url(#conflict-pdf-paper)" stroke="#ee292c" stroke-width=".45" d="M4.5 1.225h10L21.275 8v13.025a1.75 1.75 0 0 1-1.75 1.75H4.5a1.775 1.775 0 0 1-1.775-1.775V3A1.775 1.775 0 0 1 4.5 1.225Z"/>'
        + '<path fill="#e5272a" fill-opacity=".5" stroke="none" d="M14.2 1.9v5.65a1.6 1.6 0 0 0 1.6 1.6h5.2V8L14.2 1.9Z"/>'
        + '<path fill="#fffafa" stroke="#e93235" stroke-width=".42" d="M14.6 1.6v4.8a1.6 1.6 0 0 0 1.6 1.6H21Z"/>'
        + paper_seam('conflict-pdf', '#8f1421', left=14.6, top=1.6, bottom=8)
        + '<g fill="white" stroke="none" fill-rule="evenodd"><path d="M5.15 13.8h2.6a2 2 0 0 1 0 4H6.3v1.65H5.15Z M6.3 14.9v1.8h1.3a.9.9 0 0 0 0-1.8Z"/>'
        + '<path d="M10.4 13.8h1.75a2.65 2.65 0 0 1 2.65 2.65v.35a2.65 2.65 0 0 1-2.65 2.65H10.4Z M11.55 14.9v3.45h.55a1.5 1.5 0 0 0 1.5-1.5v-.45a1.5 1.5 0 0 0-1.5-1.5Z"/>'
        + '<path d="M15.9 13.8h3.2v1.1h-2.05v1.15h1.7v1.1h-1.7v2.3H15.9Z"/></g>'
    )
    progress_image = (
        gradient('progress-image-paper', '#b9e7ff', '#87d0fe')
        + '<path fill="url(#progress-image-paper)" stroke="none" d="M5 1h9.2L21 7.8V21a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V3a2 2 0 0 1 2-2Z"/>'
        + '<path fill="#74c3f4" stroke="none" d="M14.2 1v4.95a1.85 1.85 0 0 0 1.85 1.85H21Z"/>'
        + paper_seam('progress-image', '#347db6')
        + '<circle cx="8.2" cy="9.6" r="1.45" fill="#fff0b2" stroke="none"/>'
        + '<circle cx="12.5" cy="10.55" r="1.05" fill="#fff8d7" stroke="none"/>'
        + '<path fill="#259bec" stroke="none" d="m3 17.1 5.7-5.05 4.3 4.3 3.4-3.2 4.6 4.3V21a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"/>'
        + '<path fill="#0b8bea" stroke="none" d="m3 17.1 5.7-5.05 4.3 4.3L7.1 23H5a2 2 0 0 1-2-2Z"/>'
    )
    return {
        'theme-light': window_preview('theme-light', False),
        'theme-dark': window_preview('theme-dark', True),
        'theme-system': window_preview('theme-system', False, system=True),
        'existing-archive': existing_archive,
        'windows-document': platform_document('windows-document', False),
        'linux-document': platform_document('linux-document', True),
        'cancelled-large': status_disc('cancelled-large', '#a8b2c3', '#7c899e', '#637189',
                                      '<rect x="8.1" y="8.1" width="7.8" height="7.8" rx=".85" fill="#ffffff" stroke="none"/>', radius=10.2),
        'check-selected': gradient('check-selected-box', '#2b7aff', '#1751ff')
                          + '<rect x="2.5" y="2.5" width="19" height="19" rx="2.5" fill="url(#check-selected-box)" stroke="none"/>'
                          + '<path d="m6.8 12.2 3.35 3.35 7.1-7.55" stroke="white" stroke-width="1.85"/>',
        'success': status_disc('success', '#56ae63', '#329345', '#247737',
                               '<path d="m7.85 12 2.7 2.7 5.6-5.6" stroke="white" stroke-width="1.7"/>'),
        'radio-selected': '<circle cx="12" cy="12" r="9.2" stroke="#1d63ff" stroke-width="1.45"/>'
                          '<circle cx="12" cy="12" r="5.4" fill="#1d63ff" stroke="none"/>',
        'radio-unselected': f'<circle cx="12" cy="12" r="9.2" stroke="{gray}" stroke-width="1.45"/>',
        'conflict-pdf': conflict_pdf,
        'conflict-warning': gradient('conflict-warning-disc', '#ffc229', '#efa000')
                            + '<circle cx="12" cy="12" r="10" fill="url(#conflict-warning-disc)" stroke="none"/>'
                            + warning_mark(),
        'progress-image': progress_image,
    }


SUPPLEMENTAL_ROLES = {
    'theme-light': 'theme-preview', 'theme-dark': 'theme-preview', 'theme-system': 'theme-preview',
    'existing-archive': 'operation-illustration',
    'windows-document': 'platform-operation-illustration',
    'linux-document': 'platform-operation-illustration',
    'cancelled-large': 'task-result-illustration',
    'check-selected': 'control-state-reference',
    'success': 'validation-state-illustration',
    'radio-selected': 'control-state-reference', 'radio-unselected': 'control-state-reference',
    'conflict-pdf': 'conflict-file-illustration',
    'conflict-warning': 'conflict-state-illustration',
    'progress-image': 'progress-file-illustration',
}


def revised_status_shapes(theme):
    """Source-specific result discs and the source's open, blue-gray spinner."""
    white_check = '<path stroke="white" stroke-width="1.7" d="m7.85 12 2.7 2.7 5.6-5.6"/>'
    skipped = ('#bac2d1', '#949daf', '#7f8a9f') if theme == 'light' else ('#bac6db', '#9eacc2', '#8192ac')
    stopped = ('#a8b2c3', '#7c899e', '#637189') if theme == 'light' else ('#bcc8dd', '#93a6c2', '#7185a3')
    track = '#e2e6ef' if theme == 'light' else '#475269'
    return {
        'status-success': status_disc('status-success', '#55c342', '#31ad27', '#238e1c', white_check),
        'status-partial-success': status_disc('status-partial-success', '#4695ff', '#1367ff', '#034cdb', white_check),
        'status-skipped': status_disc('status-skipped', *skipped, '<path stroke="white" stroke-width="1.7" d="M7.65 12h8.7"/>'),
        'status-stopped': status_disc('status-stopped', *stopped, '<rect x="8.1" y="8.1" width="7.8" height="7.8" rx=".85" fill="white" stroke="none"/>'),
        'status-warning': status_disc('status-warning', '#ffd36b', '#f3aa0a', '#df8e00',
                                      warning_mark()),
        'spinner': '<defs><linearGradient id="spinner-active" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#607bb5"/><stop offset="1" stop-color="#63709e"/></linearGradient></defs>'
                   f'<path d="M12 3a9 9 0 1 0 0 18" stroke="{track}" stroke-width="2.5"/>'
                   '<path d="M12 3a9 9 0 0 1 9 9" stroke="url(#spinner-active)" stroke-width="2.5"/>',
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

CROPS.update({name: (source, rect) for name, (source, rect, _, _, _) in TREE_CUTS.items()})

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

MAIN_SHAPES = {'task-list': '<g fill="{ink}" stroke="none"><circle cx="4" cy="6" r=".8"/><circle cx="4" cy="12" '
              'r=".8"/><circle cx="4" cy="18" r=".8"/></g><path d="M9 6h10 M9 12h10 M9 18h10"/>',
 'more': '<circle cx="5" cy="12" r="1.5" fill="{ink}" stroke="none"/><circle cx="12" cy="12" r="1.5" '
         'fill="{ink}" stroke="none"/><circle cx="19" cy="12" r="1.5" fill="{ink}" stroke="none"/>'}

# Selected view glyphs reuse the exact outlines; only the semantic ink changes.
EXTRA_CONTROL_PATHS.update({name + '-selected': CONTROL_PATHS[name] for name in ('view-list', 'view-grid')})

STATE_SHAPES = {'info': '<g stroke="{muted}"><circle cx="12" cy="12" r="9"/><path d="M12 11v6m0-10h.01"/></g>',
 'remove': '<circle cx="12" cy="12" r="9" stroke="{track}"/><path d="M8 12h8"/>',
 'clock': '<circle cx="12" cy="12" r="9"/><path d="M12 6v7l4 2"/>'}

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

# Every previously unmapped source has its own named reconstruction.
REFERENCES.update({name: name for name in SUPPLEMENTAL_ROLES})
REFERENCES.update({'view-list-selected': 'view-list', 'view-grid-selected': 'view-grid'})
