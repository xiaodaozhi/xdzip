#!/usr/bin/env python3
"""Check actual argv and Launch Services cold opens using an owned ZIP fixture."""
import argparse
import json
import os
from pathlib import Path
import subprocess
from deployment_audit import within

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    scratch = ROOT/'build/association-tests'/output.stem
    if not within(output, ROOT/'build') or output.exists() or scratch.exists():
        parser.error('Use a new report under build/')
    scratch.mkdir(parents=True)
    app, fixture = args.app.resolve(), args.fixture.resolve()
    env = dict(os.environ, QT_QPA_PLATFORM='cocoa', QSG_RHI_BACKEND='metal')
    env.pop('QT_QUICK_BACKEND', None)
    cases = {}
    for kind in ['argv', 'launch-services']:
        record = scratch/(kind+'.json')
        profile = scratch/kind/'settings.ini'
        environment = dict(env, XDZIP_SETTINGS_PATH=str(profile), XDZIP_ASSOCIATION_PROBE=str(record))
        if kind == 'argv':
            command = [str(app/'Contents/MacOS/xdzip'), '--', str(fixture)]
        else:
            command = ['open', '-n', '-W', '-a', str(app), '--stdout', str(scratch/(kind+'.stdout.log')),
                       '--stderr', str(scratch/(kind+'.stderr.log'))]
            for key in ['XDZIP_SETTINGS_PATH', 'XDZIP_ASSOCIATION_PROBE', 'QT_QPA_PLATFORM', 'QSG_RHI_BACKEND']:
                command += ['--env', key+'='+environment[key]]
            command += ['--env', 'QT_QUICK_BACKEND=', str(fixture)]
        result = subprocess.run(command, env=environment, capture_output=True, text=True, timeout=35)
        if result.returncode or not record.exists():
            raise SystemExit('macOS file-open launch failed; see isolated association logs')
        data = json.loads(record.read_text())
        if data.get('records') != 100000 or data.get('presented_frames', 0) < 1 or data.get('welcome_frames') != 0:
            raise SystemExit('macOS external launch exposed welcome content or did not open the fixture')
        cases[kind] = data
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps({'status': 'passed', 'platform': 'macos', 'cases': cases,
        'limits': 'Actual native launch and frame visibility, not Finder human interaction or a clean-machine installation.'}, indent=2)+'\n')
    print('macOS argv and Launch Services: real archive, presented frames, zero welcome frames')


if __name__ == '__main__':
    main()
