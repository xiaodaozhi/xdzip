"""Shared zipper geometry for every current illustration and file-category icon."""

ZIPPER_ICONS = frozenset({'archive-add', 'existing-archive', 'archive-warning', 'file-archive'})

# Teeth, metal, tape, stitches, top stop, rim and highlight. The silver profile
# preserves the approved blue illustrations; other surfaces only change color.
PALETTES = {
    'silver': (('#e2f0ff', '#fff', '#d0e5ff'), ('#fff', '#edf6ff', '#b6d5ff'),
               '#1454c9', '#d9edff', '#eef8ff', '#f5fbff', 'white'),
    'warm': (('#fff1d5', '#fff', '#ffe3ac'), ('#fff', '#fff5df', '#e8c586'),
             '#a8660e', '#fff1d1', '#fff9e9', '#fffaf0', 'white'),
    'ochre-light': (('#c49749', '#b27a27', '#a66e20'), ('#d0a255', '#b58130', '#986619'),
                    '#a97424', '#c99b4c', '#be8c3b', '#d7b474', '#f3d79f'),
    'ochre-dark': (('#f4d38c', '#e3b65f', '#ce9c45'), ('#f4d38c', '#dfb15d', '#c38e34'),
                   '#b8832e', '#ebc77c', '#efc977', '#f0cf8c', '#fff0c9'),
}


def archive_zipper(palette='silver'):
    """Interlocking teeth and a separate slider with an open pull-tab aperture."""
    teeth_colors, metal, tape, stitch, stop, rim, highlight = PALETTES[palette]
    prefix = 'zip' if palette == 'silver' else 'zip-' + palette
    teeth = []
    for row in range(7):
        y = 4.05 + row * 1.25
        tooth = (f'<path d="M9.32 {y:.2f}h.66q.16 0 .16.16v.01h.3q.16 0 .16.16v.04'
                 'q0 .16-.16.16h-.3v.01q0 .16-.16.16h-.66q-.16 0-.16-.16v-.38q0-.16.16-.16Z"/>')
        teeth.append(tooth)
        teeth.append(f'<g transform="translate(21 .625) scale(-1 1)">{tooth}</g>')
    return (
        f'<defs><linearGradient id="{prefix}-teeth" x1="0" y1="0" x2="1" y2="0">'
        f'<stop stop-color="{teeth_colors[0]}"/><stop offset=".45" stop-color="{teeth_colors[1]}"/>'
        f'<stop offset="1" stop-color="{teeth_colors[2]}"/></linearGradient>'
        f'<linearGradient id="{prefix}-metal" x1="0" y1="0" x2="1" y2="1">'
        f'<stop stop-color="{metal[0]}"/><stop offset=".48" stop-color="{metal[1]}"/>'
        f'<stop offset="1" stop-color="{metal[2]}"/></linearGradient></defs>'
        f'<rect x="8.9" y="3.45" width="3.2" height="10.6" rx=".65" fill="{tape}" fill-opacity=".2" stroke="none"/>'
        f'<path d="M9.02 3.85v9.2m2.96-9.2v9.2" stroke="{stitch}" stroke-opacity=".3" stroke-width=".18"/>'
        f'<rect x="9.15" y="3.45" width="2.7" height=".4" rx=".2" fill="{stop}" stroke="none"/>'
        f'<g fill="url(#{prefix}-teeth)" stroke="none">' + ''.join(teeth) + '</g>'
        '<path d="M9.5 12.95h2l.42 1.12a1.15 1.15 0 0 1-1.08 1.55h-.68a1.15 1.15 0 0 1-1.08-1.55Z" '
        f'fill="{tape}" fill-opacity=".25" stroke="none" transform="translate(0 .2)"/>'
        '<path d="M9.5 12.95h2l.42 1.12a1.15 1.15 0 0 1-1.08 1.55h-.68a1.15 1.15 0 0 1-1.08-1.55Z" '
        f'fill="url(#{prefix}-metal)" stroke="{rim}" stroke-width=".16"/>'
        f'<path d="M9.83 13.48h1.34" stroke="{highlight}" stroke-width=".25"/>'
        f'<path fill="url(#{prefix}-metal)" stroke="none" fill-rule="evenodd" '
        'd="M10.08 14.65h.84a.86.86 0 0 1 .86.86v2.4a.9.9 0 0 1-.9.9h-.76a.9.9 0 0 1-.9-.9v-2.4a.86.86 0 0 1 .86-.86Z '
        'M10.1 15.75a.38.38 0 0 0-.38.38v1.58a.45.45 0 0 0 .45.45h.66a.45.45 0 0 0 .45-.45v-1.58a.38.38 0 0 0-.38-.38Z"/>'
        f'<path d="M9.52 15.5v2.28a.67.67 0 0 0 .67.67" stroke="{highlight}" stroke-opacity=".85" stroke-width=".16"/>'
    )
