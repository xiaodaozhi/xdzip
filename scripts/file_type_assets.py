# scripts/file_type_assets.py

"""Original file-category glyphs sharing the existing document silhouette."""

from zipper_assets import archive_zipper

FILE_CATEGORY_GLYPHS = {
    'file-document': ('#3979d8', 'M7.5 10h9 M7.5 13h9 M7.5 16h6 M7.5 19h8'),
    'file-spreadsheet': ('#219568', 'M7.2 9.6h9.6v9.6H7.2z M7.2 12.8h9.6 M7.2 16h9.6 M10.4 9.6v9.6'),
    'file-presentation': ('#da7840', 'M7 9.3h10v7.2H7z M12 16.5v3 M9.5 19.5h5 M9.2 14.6l2-2 1.7 1 2.1-2.1'),
    'file-audio': ('#9262cc', 'M10 17V10l6-1.5V16 M10 10l6-1.5 M10 17a2 1.5 0 1 1-2-1.5c1.1 0 2 .7 2 1.5 M16 16a2 1.5 0 1 1-2-1.5c1.1 0 2 .7 2 1.5'),
    'file-video': ('#d85c7c', 'M6.8 9.5h10.4v9H6.8z M10.5 12l4 2-4 2z M6.8 12h1.5 M6.8 16h1.5 M15.7 12h1.5 M15.7 16h1.5'),
    'file-archive': ('#b8832e', None),
    'file-installer': ('#4b86af', 'M7 11.5l5-2.6 5 2.6v5.7L12 20l-5-2.8z M7 11.5l5 2.8 5-2.8 M12 14.3V20 M9.5 10.2l5 2.7'),
    'file-font': ('#9262b5', 'M6.5 18l3-8 3 8 M7.7 15h3.6 M15 14c3.6-1.4 3.7 4 0 3.8-1.8 0-1.7-2.1.2-2.1H18 M18 14v4'),
    'file-disk': ('#78899e', 'M17 14.5a5 5 0 1 1-10 0 5 5 0 0 1 10 0 M13.5 14.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0 M8.4 11l1 1 M14.6 17l1 1'),
}


def desktop_file_shapes(theme):
    paper, border = ('#fff', '#bac2d0') if theme == 'light' else ('#28313f', '#8190a6')
    sheet = (f'<path fill="{paper}" stroke="{border}" stroke-width="1" '
             'd="M5.8 1.5h8.55l5.4 5.4v14a1.6 1.6 0 0 1-1.6 1.6H5.8a1.55 1.55 0 0 1-1.55-1.55V3.05A1.55 1.55 0 0 1 5.8 1.5Z"/>'
             f'<path stroke="{border}" stroke-width="1" d="M14.35 1.75v4.1a1.05 1.05 0 0 0 1.05 1.05h4.1"/>')
    shapes = {name: sheet + f'<path d="{glyph}" stroke="{color}" stroke-width="1.2"/>'
              for name, (color, glyph) in FILE_CATEGORY_GLYPHS.items() if glyph is not None}
    shapes['file-archive'] = (sheet + '<g transform="translate(3.6 4.3) scale(.8)">'
                              + archive_zipper('ochre-' + theme) + '</g>')
    return shapes
