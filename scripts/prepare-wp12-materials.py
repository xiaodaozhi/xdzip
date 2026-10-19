#!/usr/bin/env python3
"""Prepare itemized external-review questions from verified automation evidence."""
import argparse
from collections import Counter
import json
from pathlib import Path
from deployment_audit import ROOT, digest, within


def platform_summary(root,platform,revision):
    def load(name):return json.loads((root/name).read_text())
    deploy=load('deployment-audit.json');sdk=load('qt-sdk-correspondence.json');attribution=load('runtime-attribution.json')
    if sdk['revision']!=revision or sdk['source_dirty'] or not revision.startswith(deploy['identity']['revision']) or deploy['identity']['dirty']:raise ValueError('Mixed platform identity')
    declared={d['repository']:d['sha256'] for d in attribution['upstream_documents']}
    if any(declared[m['module']]!=m['sdk_build_spdx_sha256'] for m in sdk['modules']):raise ValueError('SDK source/runtime document mismatch')
    if deploy['status']!='passed':raise ValueError('Unresolved native imports')
    life=load('macos-association.json' if platform=='macos' else platform+'-install-lifecycle.json')
    if life.get('status')!='passed':raise ValueError('Native association/lifecycle validation missing')
    for filename in ['source-build.cdx.json','artifact-files.cdx.json']:
        bom=load(filename)
        if bom.get('bomFormat')!='CycloneDX' or str(bom.get('specVersion'))!='1.6' or any(c.get('aggregate')=='complete' for c in bom.get('compositions',[])):raise ValueError('Incomplete component scope was misrepresented as complete')
    deployed_app=next(f['sha256'] for f in deploy['files'] if f['path']==deploy['app'])
    expected_build=deployed_app
    if platform=='linux':
        transform=load('linux-deployment.json')['application_transform']
        if transform['artifact_sha256']!=deployed_app:raise ValueError('Stale application RPATH transform')
        expected_build=transform['source_sha256']
    for filename in ['runtime-resources.json','linkage-evidence.json','desktop-performance.json']:
        if load(filename)['app_sha256']!=expected_build:raise ValueError('Resource/linkage/performance proof belongs to another executable')
    transform_report=load('qt-binary-differences.json')
    if transform_report['attribution_sha256']!=digest(root/'runtime-attribution.json'):raise ValueError('Stale SDK transform explanation')
    transform_by_file={item['sdk_path']:item['results'] for item in transform_report['files']}
    questions=[]
    for gap in attribution['gaps']:
        category=gap['kind']
        action={'upstream-sdk-checksum-difference':'Obtain publisher build/postprocessing/signing provenance; matching deployed SDK bytes does not reconstruct an unavailable prepackaging binary.',
                'upstream-license-unasserted':'Review the publisher declaration, source notices and actual file-bearing/system-wrapper scope; select/approve obligations without inferring a license.',
                'sdk-native-without-spdx-file':'Confirm exact SDK dependency provenance and required notices; use native version metadata as evidence, not proof of source equivalence.',
                'native-outside-qt-attribution':'Review product composite or vendor redistributable using actual link inputs, embedded-resource evidence and native file metadata.',
                'upstream-package-reference-missing':'Request the missing upstream component relationship declaration.',
                'unverified-rewrite':'Provide the exact transformation inputs/output hashes before accepting ownership.'}.get(category,'Resolve the precise upstream evidence gap before formal distribution.')
        questions.append({'gap':gap,'transformation_analysis':transform_by_file.get(gap.get('file'),[]),'external_action':action})
    return {'schema_version':1,'status':'passed','platform':platform,'revision':revision,'sdk_binding_verified':True,'deployment_passed':True,'lifecycle_passed':True,
            'deployment_mode':deploy['mode'],'gap_counts':dict(Counter(g['kind'] for g in attribution['gaps'])),'review_questions':questions,
            'release_approved':False,'limits':['Automated records and matching SDK declarations, not complete source/license attribution or manual desktop approval.']}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--ci',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    out=a.output.resolve()
    if out.exists() or not within(out,ROOT/'build'):p.error('Use a new build/ directory')
    ci=json.loads((a.ci/'ci-evidence.json').read_text());out.mkdir(parents=True);platforms=[]
    for platform in ['macos','windows','linux']:
        summary=platform_summary(a.ci/platform,platform,ci['revision']);platforms.append(summary)
        (out/(platform+'-platform.json')).write_text(json.dumps(summary,indent=2)+'\n')
    inventory=[]
    for file in sorted(a.ci.rglob('*.json')):
        inventory.append({'file':file.relative_to(a.ci).as_posix(),'sha256':digest(file)})
    report={'schema_version':1,'status':'automated-materials-prepared-review-required','revision':ci['revision'],'platforms':platforms,'inventory':inventory,
            'automatic_materials_complete':True,'legal_approval':False,'independent_review':False,
            'external_boundaries':['Unstated publisher source/build/signing provenance cannot be fabricated by the consumer.',
                                   'Per-source license selection, platform redistributable rights and exact optimized-code ownership require accountable review.',
                                   'Actual linker inputs, runtime resource bytes, SDK-source declarations and native supplier/version metadata are supplied for that review.']}
    (out/'review-materials.json').write_text(json.dumps(report,indent=2)+'\n');print('Verified platform bindings and prepared itemized review questions')

if __name__=='__main__':main()
