#!/usr/bin/env python3
"""Record shell phase duration and original exit status without command arguments."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def append(target, phase, label, started_ns, ended_ns, exit_code, revision):
    if not re.fullmatch('[a-z][a-z0-9-]{0,60}', phase) or not re.fullmatch('[A-Za-z0-9_.+-]{1,100}', label):
        raise ValueError('Invalid public timing label')
    if (not re.fullmatch('[a-f0-9]{40}', revision) or started_ns < 0 or ended_ns < started_ns
            or type(exit_code) is not int):
        raise ValueError('Invalid timing or source identity')
    data = {'schema_version': 1, 'scope': 'html', 'revision': revision,
            'run_id': os.environ.get('GITHUB_RUN_ID'), 'run_attempt': os.environ.get('GITHUB_RUN_ATTEMPT'), 'commands': []}
    if target.exists():
        previous = json.loads(target.read_text(encoding='utf-8'))
        if any(previous[k] != data[k] for k in ('schema_version', 'scope', 'revision', 'run_id', 'run_attempt')):
            raise ValueError('Timing records belong to another source or run')
        data = previous
    data['commands'].append({'phase': phase, 'command': label, 'elapsed_ms': (ended_ns - started_ns) / 1000000,
                             'exit_code': exit_code, 'status': 'passed' if exit_code == 0 else 'failed'})
    target.parent.mkdir(parents=True, exist_ok=True)
    temporary = target.with_suffix('.tmp')
    temporary.write_text(json.dumps(data, indent=2) + '\n', encoding='utf-8')
    temporary.replace(target)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--phase', required=True)
    parser.add_argument('--label', required=True)
    parser.add_argument('--started-ns', type=int, required=True)
    parser.add_argument('--ended-ns', type=int, required=True)
    parser.add_argument('--exit-code', type=int, required=True)
    args = parser.parse_args()
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    append(ROOT / 'build/review/html-ci-timings.json', args.phase, args.label,
           args.started_ns, args.ended_ns, args.exit_code, revision)


if __name__ == '__main__':main()
