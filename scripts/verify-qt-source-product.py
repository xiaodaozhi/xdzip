#!/usr/bin/env python3
"""Compile/test the real product against the source-built Qt prefix, no deployment/signing."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from macos_launch_services import transient_registrations
from deployment_audit import ROOT, digest, within


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--qt-build',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    p.add_argument('--parallel',type=int,default=3);a=p.parse_args();out=a.output.resolve();qt=a.qt_build.resolve()/'qt-install'
    if out.exists() or not within(out,ROOT/'build'):p.error('Use a new build/ directory')
    with transient_registrations(out):
        out.mkdir(parents=True);report=json.loads((a.qt_build/'qt-source-build.json').read_text())
        configure=['cmake','-S',str(ROOT),'-B',str(out/'compiled'),'-G','Ninja','-DCMAKE_BUILD_TYPE=Release','-DXDZIP_BUILD_DESKTOP_PREVIEW=ON','-DCMAKE_PREFIX_PATH='+str(qt)]
        if sys.platform=='darwin':configure+=['-DCMAKE_OSX_ARCHITECTURES=arm64','-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0']
        env=dict(os.environ,QT_PLUGIN_PATH=str(qt/'plugins'),QML_IMPORT_PATH=str(qt/'qml'),QT_QPA_PLATFORM='offscreen',QT_QUICK_BACKEND='software')
        if os.name=='nt':env['PATH']=str(qt/'bin')+os.pathsep+env['PATH']
        elif sys.platform!='darwin':env['LD_LIBRARY_PATH']=str(qt/'lib')
        with (out/'validation.log').open('w') as log:
            for command in [configure,['cmake','--build',str(out/'compiled'),'--parallel',str(a.parallel)],['ctest','--test-dir',str(out/'compiled'),'--output-on-failure']]:
                subprocess.run(command,stdout=log,stderr=log,env=env,check=True)
            app=out/'compiled/apps/desktop'/('xdzip.app/Contents/MacOS/xdzip' if sys.platform=='darwin' else 'xdzip-browser.exe' if os.name=='nt' else 'xdzip-browser')
            identity=subprocess.check_output([str(app),'--version'],env=env,text=True).strip()
            if 'dirty' in identity or 'unknown' in identity or report['revision'][:12] not in identity:raise ValueError('Source Qt/product identity mismatch')
            if sys.platform=='darwin':env.update(QT_QPA_PLATFORM='cocoa',QSG_RHI_BACKEND='metal');env.pop('QT_QUICK_BACKEND',None)
            env.update(XDZIP_SETTINGS_PATH=str(out/'isolated/settings.ini'),XDZIP_EXPECT_QT_SOURCE_PROOF='xdzip-qt-6.8.2-source-proof-v1')
            fixture=out/'compiled/apps/desktop/m2-flat.zip'
            probe=subprocess.run([str(app),'--desktop-probe',str(fixture)],env=env,capture_output=True,text=True,timeout=40,check=True)
            events=[json.loads(line.removeprefix('XDZIP_PROBE ')) for line in probe.stdout.splitlines() if line.startswith('XDZIP_PROBE ')]
            by={e['event']:e for e in events}
            if by.get('qt_source_proof',{}).get('value')!=env['XDZIP_EXPECT_QT_SOURCE_PROOF'] or by.get('open_ready',{}).get('records')!=100000:
                raise ValueError('Actual loaded Qt source marker/real archive proof missing')
            if sys.platform=='darwin' and by['first_frame']['backend']!='metal':raise ValueError('Native Metal required')
    result={'schema_version':1,'platform':'macos' if sys.platform=='darwin' else 'windows' if os.name=='nt' else 'linux','status':'passed','revision':report['revision'],'identity':identity,'source_build_sha256':digest(a.qt_build/'qt-source-build.json'),
            'app_sha256':digest(app),'cases':['real-product-native-build','six-product-ctest-suites','actual-loaded-QtCore-source-marker','100k-real-open-query-sort-cancel'],
            'events':events,'deployed':False,'explicit_signing_performed':False,'developer_id_signed':False,
            'limits':['Native source Qt validation, no original SDK binary equivalence or formal platform/desktop approval.']}
    (out/'qt-source-product.json').write_text(json.dumps(result,indent=2)+'\n');print('Real product tests and actual source Qt load marker passed')

if __name__=='__main__':main()
