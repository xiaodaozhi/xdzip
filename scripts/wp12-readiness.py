#!/usr/bin/env python3
"""Validate bound WP-12 automation evidence while keeping external gates explicit."""
import argparse
import json
from pathlib import Path
import re
from deployment_audit import ROOT, digest, within

GATES=json.loads((ROOT/'scripts/wp12-gates.json').read_text())


def revision(record):
    identity=record.get('identity',{})
    value=record.get('revision') or record.get('commit') or (identity.get('revision') if isinstance(identity,dict) else None)
    if not value and isinstance(identity,str):
        match=re.search(r', ([0-9a-f]{12,40})\)',identity)
        value=match[1] if match else None
    return value


def validate_record(kind, record, expected):
    actual=revision(record)
    if actual and not expected.startswith(actual):raise ValueError('Evidence source identity mismatch')
    identity=record.get('identity',{})
    if record.get('source_dirty') or (isinstance(identity,dict) and identity.get('dirty')):raise ValueError('Dirty evidence rejected')
    status=record.get('status')
    allowed={
        'ci':{'passed'}, 'platform':{'passed'}, 'source-delivery':{'collected-not-compliance-verified'},
        'source-rebuild':{'passed'}, 'qt-source-product':{'passed'}, 'qt-replacement':{'passed'},
        'performance':{'automated-observations-complete-reference-approval-pending'},
        'resources':{'source-runtime-resource-bytes-matched'}, 'linkage':{'link-input-evidence-not-complete-attribution'},
        'materials':{'automated-materials-prepared-review-required'}, 'package':{'passed'}}
    # source-delivery's established field is deliberately not a release pass.
    if kind=='source-delivery':status=record.get('source_delivery')
    if kind not in allowed or status not in allowed[kind]:raise ValueError('Evidence kind/status does not meet automation contract')
    if kind=='ci' and (not record.get('all_jobs_succeeded') or len(record.get('product_platforms',[]))!=3):raise ValueError('Missing native CI platforms')
    if kind=='platform' and (not record.get('sdk_binding_verified') or not record.get('deployment_passed') or not record.get('lifecycle_passed')):raise ValueError('Incomplete platform evidence')
    if kind=='performance':
        if record.get('interactive_samples',0)<30 or record.get('throughput_samples_per_driver',0)<5 or (record.get('idle_observed_ms') or 0)<59000:raise ValueError('Performance sample/dwell contract incomplete')
        if record.get('release_reference_qualified',False):raise ValueError('This automation cannot approve a reference host')
    if kind=='qt-source-product':
        events={e['event']:e for e in record.get('events',[])}
        if events.get('qt_source_proof',{}).get('value')!='xdzip-qt-6.8.2-source-proof-v1' or events.get('open_ready',{}).get('records')!=100000:raise ValueError('Actual source Qt marker/archive proof missing')
    if kind=='qt-replacement' and (not record.get('source_marker_verified') or not record.get('sdk_independent') or not record.get('independent_extraction_verified')):raise ValueError('Source replacement not actually demonstrated')
    if kind=='resources' and not record.get('files'):raise ValueError('Empty runtime resource set')
    if kind=='linkage' and not record.get('staticlib_members'):raise ValueError('Empty linkage input inventory')


def evaluate(manifest):
    expected=manifest.get('revision','')
    if not re.fullmatch('[a-f0-9]{40}',expected):raise ValueError('Full source commit required')
    entries=manifest.get('automatic',{})
    if set(entries)!={g['id'] for g in GATES['automatic']}:raise ValueError('Missing/extra automatic gate')
    results=[]
    for gate in GATES['automatic']:
        refs=entries[gate['id']]
        if len(refs)<gate['minimum_reports']:raise ValueError('Missing required gate evidence: '+gate['id'])
        required={'CI':{'ci'},'DEPLOY':{'platform'},'SOURCE':{'source-delivery','source-rebuild'},'QT_SOURCE':{'qt-source-product'},
                  'QT_REPLACE':{'qt-replacement'},'PERFORMANCE':{'performance'},'RESOURCES':{'resources'},'LINKAGE':{'linkage'},'MATERIALS':{'materials'},'PACKAGES':{'package'}}[gate['id']]
        if {r['kind'] for r in refs}!=required:raise ValueError('Wrong evidence kinds for '+gate['id'])
        seen=set();bound=[]
        for ref in refs:
            path=ROOT/ref['path']
            if Path(ref['path']).is_absolute() or not within(path,ROOT/'build') or not path.is_file():raise ValueError('Evidence path outside existing build artifact')
            if path.resolve() in seen:raise ValueError('Duplicate evidence cannot satisfy platform count')
            seen.add(path.resolve())
            if digest(path)!=ref['sha256']:raise ValueError('Evidence hash mismatch')
            record=json.loads(path.read_text());validate_record(ref['kind'],record,expected)
            if gate['id'] in {'DEPLOY','QT_SOURCE','RESOURCES','LINKAGE','PACKAGES'} and record.get('platform')!=ref.get('platform'):
                raise ValueError('Evidence platform label mismatch')
            # Re-check nested producer files, not just the top-level summary.
            nested=record.get('files',[]) if ref['kind']=='source-delivery' else record.get('reports',[]) if ref['kind'] in {'performance','qt-replacement'} else []
            for item in nested:
                child=path.parent/item['file']
                if not within(child,path.parent) or not child.is_file() or digest(child)!=item['sha256']:raise ValueError('Nested material hash mismatch')
            if ref['kind']=='qt-source-product':
                source=path.parent/'qt-source-build.json'
                if not source.is_file() or digest(source)!=record['source_build_sha256']:raise ValueError('Source Qt build binding mismatch')
                upstream=json.loads(source.read_text())
                if upstream.get('source_dirty') or upstream['revision']!=expected:raise ValueError('Source Qt build identity mismatch')
            bound.append(ref)
        # Three reports must actually cover distinct native platforms.
        if gate['id'] in {'DEPLOY','QT_SOURCE','RESOURCES','LINKAGE','PACKAGES'}:
            platforms={r.get('platform') for r in refs}
            if platforms!={'macos','windows','linux'}:raise ValueError('Platform coverage missing or duplicated')
        results.append({'id':gate['id'],'title':gate['title'],'status':'passed','evidence':bound})
    source_refs=entries['SOURCE'] if 'SOURCE' in entries else []
    if source_refs:
        delivered=next(r for r in source_refs if r['kind']=='source-delivery');rebuilt=next(r for r in source_refs if r['kind']=='source-rebuild')
        if json.loads((ROOT/rebuilt['path']).read_text())['source_manifest_sha256']!=delivered['sha256']:raise ValueError('Offline rebuild bound to another source delivery')
    return {'schema_version':1,'revision':expected,'status':'automatic-scope-complete-external-gates-pending','automatic_complete':True,
            'release_ready':False,'automatic':results,'external':[dict(g,status='pending') for g in GATES['external']],
            'limits':['Verified automation evidence is not formal release approval or license choice.',
                      'Previously accepted macOS routine checks remain valid; only uncovered specialties need additional records.']}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--manifest',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    p.add_argument('--require-auto-pass',action='store_true');p.add_argument('--require-release-pass',action='store_true');a=p.parse_args()
    if a.output.exists() or not within(a.output.resolve(),ROOT/'build'):p.error('Use a new output under build/')
    result=evaluate(json.loads(a.manifest.read_text()));result['input_manifest_sha256']=digest(a.manifest)
    a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print('WP-12 automation evidence passed; external gates remain pending')
    if a.require_release_pass:raise SystemExit('Release gate remains blocked by license, device and support-policy requirements')

if __name__=='__main__':main()
