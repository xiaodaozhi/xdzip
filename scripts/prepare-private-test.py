#!/usr/bin/env python3
"""Prepare pending desktop records, or check manually entered evidence without inventing results."""
import argparse
import json
from pathlib import Path
import platform
import re
import runpy
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
MATRIX=json.loads((ROOT/'scripts/private-test-matrix.json').read_text(encoding='utf-8'))


def host_platform():
    return 'windows' if sys.platform=='win32' else 'macos' if sys.platform=='darwin' else 'linux'


def prepare(target):
    version=runpy.run_path(str(ROOT/'scripts/sync-version.py'))['version_info']()
    revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    return {'schema_version':1,'platform':target,'version':version['display_version'],'commit':revision,
        'environment':{'os':platform.platform() if target==host_platform() else None,
                       'architecture':platform.machine() if target==host_platform() else None,
                       'device':None,'renderer':None,'driver':None,'system_scale':None,
                       'ime':None,'screen_reader':None,'artifact_sha256':None},
        'operator_role':None,'executed_at':None,
        'cases':[dict(case,status='pending',actual='',evidence=[],issue_refs=[]) for case in MATRIX['cases']],
        'limits':['Generated preparation record only. No desktop test has been performed.',
                  'Prior accepted macOS UI checks remain valid; these are uncovered specialty gates.',
                  'Local developer DMG and ad-hoc integrity are authorized; Developer ID, notarization and public upload require release identity.']}


def validate(record,require_pass=False):
    if record.get('schema_version')!=1 or record.get('platform') not in MATRIX['platforms']:
        raise ValueError('Invalid record schema or platform')
    if not re.fullmatch('[0-9a-f]{40}',record.get('commit','')):
        raise ValueError('A full source commit is required')
    expected={c['id'] for c in MATRIX['cases']}
    cases=record.get('cases',[])
    if len(cases)!=len(expected) or {c.get('id') for c in cases}!=expected:
        raise ValueError('Missing, duplicate or unknown manual case')
    counts={status:0 for status in ['pending','passed','failed','blocked','not_applicable']}
    for case in cases:
        status=case.get('status')
        if status not in counts:raise ValueError('Unknown manual result status')
        counts[status]+=1
        if status!='pending' and not case.get('actual','').strip():
            raise ValueError('Executed or scoped case needs actual result/reason')
        if status=='passed':
            if not record.get('operator_role') or not record.get('executed_at') or not case.get('evidence'):
                raise ValueError('Passed cases require operator role, execution date and evidence')
            environment=record.get('environment',{})
            if any(not environment.get(k) for k in ['os','architecture','device','artifact_sha256']):
                raise ValueError('Passed cases require actual device and artifact identity')
            if not re.fullmatch('[0-9a-f]{64}',environment['artifact_sha256']):
                raise ValueError('Invalid tested artifact hash')
            required = {'IME-01':['ime'], 'A11Y-01':['screen_reader'], 'GPU-01':['renderer','driver','system_scale']}.get(case['id'],[])
            if any(not environment.get(key) for key in required):
                raise ValueError('Passed specialty case requires its actual input/reader/rendering environment')
            for name in case['evidence']:
                path=Path(name)
                if path.is_absolute() or not path.parts or path.parts[0] not in ['build','dist'] or '..' in path.parts:
                    raise ValueError('Evidence must be a relative build/ or dist/ file')
                resolved=(ROOT/path).resolve()
                if not resolved.is_relative_to(ROOT/path.parts[0]) or not resolved.is_file():
                    raise ValueError('Evidence file is missing or escapes the artifact directory')
        if status=='not_applicable' and not case.get('scope_decision'):
            raise ValueError('Scope exclusion requires a release-owner decision reference')
    if require_pass and counts['passed']!=len(expected):
        raise ValueError('Desktop matrix is incomplete; scope decisions need separate readiness review')
    return counts


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--platform',choices=MATRIX['platforms'],default=host_platform())
    parser.add_argument('--output',type=Path)
    parser.add_argument('--validate',type=Path)
    parser.add_argument('--require-pass',action='store_true')
    args=parser.parse_args()
    try:
        if args.validate:
            print('Manual record structure:',json.dumps(validate(json.loads(args.validate.read_text(encoding='utf-8')),args.require_pass)))
        else:
            if not args.output:parser.error('--output is required when preparing a record')
            if not args.output.resolve().is_relative_to(ROOT/'build') and not args.output.resolve().is_relative_to(ROOT/'dist'):
                parser.error('Generated records belong under build/ or dist/')
            if args.output.exists():parser.error('Refusing to overwrite an existing manual record')
            args.output.parent.mkdir(parents=True,exist_ok=True)
            args.output.write_text(json.dumps(prepare(args.platform),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
            print('Prepared manual record: all 8 cases pending, no readiness claim')
    except ValueError as error:
        raise SystemExit(str(error))

if __name__=='__main__':main()
