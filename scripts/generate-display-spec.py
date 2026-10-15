#!/usr/bin/env python3
"""Generate the reviewer conversion contract from the logical sizing standard."""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    layout = json.loads((ROOT / 'docs/visual/layout-scale.json').read_text())
    reference = json.loads((ROOT / 'docs/visual/qt-main-reference.json').read_text())
    sizes = {'welcome': ('welcome-titlebar-light.png', [1586, 992]),
             'welcome-narrow': ('welcome-titlebar-narrow.png', [1296, 875]),
             'main': ('main-light.png', [1586, 992]),
             'extract': ('extract-macos.png', [1092, 1440]),
             'create': ('create.png', [1073, 1466]),
             'progress': ('progress.png', [1446, 1088]),
             'archive-test': ('integrity-progress.png', [1445, 1088]),
             'result': ('partial-result.png', [1421, 1108]),
             'conflict': ('conflict.png', [1478, 1064]),
             'settings': ('settings.png', [1092, 1440]),
             'empty': (None, [1586, 992]), 'damaged': (None, [1586, 992])}
    settings = json.loads((ROOT / 'docs/visual/settings-reference.json').read_text())
    sizes['settings'] = (Path(settings['source_image']).name, settings['source_size'])
    main_ratio = layout['main']['source_to_logical']
    operation_ratio = layout['operation']['source_to_logical']
    spec = dict(version=2, logicalWindowWidth=main_ratio[0],
                sourceWindow=reference['source_window'],
                sourceToLogical=main_ratio[0] / main_ratio[1],
                operationSourceToLogical=operation_ratio[0] / operation_ratio[1],
                mainDensity=layout['main']['density'],
                views={key: dict(source='../hifi/screens/' + filename if filename else None,
                                 size=size) for key, (filename, size) in sizes.items()})
    for key, ratio in layout['operation'].get('source_to_logical_by_view', {}).items():
        spec['views'][key]['sourceToLogical'] = ratio[0] / ratio[1]
    spec['views']['welcome-narrow']['sourceToLogical'] = main_ratio[0] / main_ratio[1]
    spec['views']['welcome']['sourceToLogical'] = main_ratio[0] / main_ratio[1]
    spec['views']['welcome-narrow']['window'] = [20, 30, 832 / spec['sourceToLogical'], 544 / spec['sourceToLogical']]
    spec['views']['settings']['darkSource'] = '../hifi/screens/settings-current-dark.png'
    spec['views']['welcome']['darkSource'] = '../hifi/screens/welcome-titlebar-dark.png'
    output = ('"use strict";\n// Generated from docs/visual/layout-scale.json; never infer logical size from DPR.\n'
              'window.XDZIP_DISPLAY_SPEC = Object.freeze(' + json.dumps(spec, indent=4) + ');\n')
    path = ROOT / 'design/html/display-spec.js'
    if args.check:
        if path.read_text() != output:
            raise SystemExit('Display specification is stale; regenerate it')
    else:
        path.write_text(output)


if __name__ == '__main__':
    main()
