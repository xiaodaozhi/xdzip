#!/usr/bin/env python3
"""Bind actual developer package bytes to producer and lifecycle receipts."""
import argparse
import json
from pathlib import Path
from deployment_audit import ROOT, digest, within


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['package','producer','lifecycle','audit','output']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--platform',choices=['macos','windows','linux'],required=True);p.add_argument('--revision',required=True);a=p.parse_args()
    if a.output.exists() or not within(a.output.resolve(),ROOT/'build'):p.error('Use a new build/ file')
    producer=json.loads(a.producer.read_text());life=json.loads(a.lifecycle.read_text());actual=digest(a.package)
    # The producer's established package field differs from the deployed app hash.
    candidates=[producer.get(k) for k in ['package_sha256','installer_sha256','dmg_sha256','sha256']]
    if actual not in candidates:raise ValueError('Package bytes do not match producer receipt')
    if life.get('status')!='passed':raise ValueError('Package lifecycle validation missing')
    audit=json.loads(a.audit.read_text())
    if audit['status']!='passed':raise ValueError('Package native audit missing')
    identity=producer.get('identity',audit.get('identity',{}))
    if a.platform=='windows' and (producer.get('layout')!='flat' or producer.get('executable_relative_path')!='xdzip.exe'):
        raise ValueError('Windows product must install its executable at the app root')
    if producer.get('runtime_id') and not producer['runtime_id'].startswith(identity['revision']+'-'):raise ValueError('Installer runtime id mismatch')
    if producer.get('executable_sha256') and producer['executable_sha256'] not in {f['sha256'] for f in audit['files']}:raise ValueError('Installer executable not in audited payload')
    declared=identity.get('revision','')
    if not declared or not a.revision.startswith(declared) or identity.get('dirty'):raise ValueError('Package identity mismatch')
    if a.platform!='macos' and producer.get('qa_fixture'):raise ValueError('QA-only installer is not product evidence')
    result={'schema_version':1,'status':'passed','platform':a.platform,'revision':a.revision,'identity':identity,
            'package_sha256':actual,'package_bytes':a.package.stat().st_size,'producer_sha256':digest(a.producer),'lifecycle_sha256':digest(a.lifecycle),'audit_sha256':digest(a.audit),
            'formal_signature_approved':False,'limits':['Actual experimental developer package bound to producer/lifecycle records, not formal signature, clean human desktop or release approval.']}
    a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(result,indent=2)+'\n');print('Bound actual developer package bytes, identity and lifecycle')

if __name__=='__main__':main()
