#!/usr/bin/env python3
"""Prepare reviewed license texts and SDK declarations before artifact hashing."""
import argparse
import json
from pathlib import Path
from runtime_attribution import ROOT, license_materials, load_qt
from qt_psl_source import collect_runtime_source

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--qt-prefix',type=Path,required=True)
parser.add_argument('--output',type=Path,required=True)
parser.add_argument('--license-cache',type=Path,default=ROOT/'build/review/qt-license-cache')
args=parser.parse_args()
packages,dependencies,index,documents,extracted=load_qt(args.qt_prefix.resolve())
materials=license_materials(args.output,args.license_cache)
psl_source=collect_runtime_source(args.output)
config=json.loads((ROOT/'scripts/qt-license-sources.json').read_text())
report={'scope':'Reference declarations for installed Qt SDK, including unused modules and build tools; runtime selection is a separate report.',
        'qt_version':'6.8.2','source_commits':config['source_commits'],
        'source_commit_scope':config['source_commit_scope'],
        'corresponding_source_url':'https://download.qt.io/archive/qt/6.8/6.8.2/single/qt-everywhere-src-6.8.2.tar.xz',
        'source_delivery':'URL only; corresponding-source archive delivery and relinking obligations require release review.',
        'upstream_documents':documents,'packages':list(packages.values()),'extracted_license_texts':extracted,
        'license_materials':materials,'psl_source_material':psl_source,
        'licensing_decision':'No automatic license selection or compliance approval.'}
(args.output/'qt-sdk-declarations.json').write_text(json.dumps(report,indent=2)+'\n')
print(f'Prepared {len(materials)} pinned license texts and {len(packages)} SDK reference declarations')
