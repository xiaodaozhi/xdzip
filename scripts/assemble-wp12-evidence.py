#!/usr/bin/env python3
"""Assemble complete automation records and run strict WP-12 evidence gates."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
from deployment_audit import ROOT, digest, within


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['ci','materials','source-delivery','source-rebuild','qt-replacement','performance','mac-package','mac-producer','mac-verification','output']:
        p.add_argument('--'+name,type=Path,required=True)
    a=p.parse_args();out=a.output.resolve()
    if out.exists() or not within(out,ROOT/'build'):p.error('Use a new build/ directory')
    out.mkdir(parents=True);ci=json.loads((a.ci/'ci-evidence.json').read_text());revision=ci['revision']
    manifest={'schema_version':1,'revision':revision,'automatic':{}}
    def ref(file,kind,platform=None):
        file=file.resolve()
        if not within(file,ROOT/'build') or not file.is_file():raise ValueError('Required actual evidence missing')
        result={'path':file.relative_to(ROOT).as_posix(),'kind':kind,'sha256':digest(file)}
        if platform:result['platform']=platform
        return result
    def add(gate,*records):manifest['automatic'][gate]=list(records)
    add('CI',ref(a.ci/'ci-evidence.json','ci'))
    add('DEPLOY',*[ref(a.materials/(platform+'-platform.json'),'platform',platform) for platform in ['macos','windows','linux']])
    add('SOURCE',ref(a.source_delivery/'source-delivery.json','source-delivery'),ref(a.source_rebuild/'source-rebuild.json','source-rebuild'))
    add('QT_SOURCE',*[ref(a.ci/'source'/platform/'qt-source-product.json','qt-source-product',platform) for platform in ['macos','windows','linux']])
    add('QT_REPLACE',ref(a.qt_replacement/'qt-source-replacement.json','qt-replacement'))
    add('PERFORMANCE',ref(a.performance/'performance-suite.json','performance'))
    add('RESOURCES',*[ref(a.ci/platform/'runtime-resources.json','resources',platform) for platform in ['macos','windows','linux']])
    add('LINKAGE',*[ref(a.ci/platform/'linkage-evidence.json','linkage',platform) for platform in ['macos','windows','linux']])
    add('MATERIALS',ref(a.materials/'review-materials.json','materials'))
    packages=[]
    for platform in ['macos','windows','linux']:
        producer=a.mac_producer if platform=='macos' else a.ci/platform/(platform+'-installer.json')
        package=a.mac_package if platform=='macos' else a.ci/platform/('xdzip-windows-x64-setup.exe' if platform=='windows' else 'xdzip-ubuntu24.04-amd64.deb')
        life=a.mac_verification/'verification.json' if platform=='macos' else a.ci/platform/(platform+'-install-lifecycle.json')
        audit=a.mac_producer if platform=='macos' else a.ci/platform/'deployment-audit.json'
        record=out/(platform+'-package.json')
        subprocess.run([sys.executable,str(ROOT/'scripts/check-package-evidence.py'),'--package',str(package),'--producer',str(producer),'--lifecycle',str(life),'--audit',str(audit),'--platform',platform,'--revision',revision,'--output',str(record)],check=True)
        packages.append(ref(record,'package',platform))
    add('PACKAGES',*packages)
    path=out/'evidence-manifest.json';path.write_text(json.dumps(manifest,indent=2)+'\n')
    subprocess.run([sys.executable,str(ROOT/'scripts/wp12-readiness.py'),'--manifest',str(path),'--output',str(out/'readiness.json'),'--require-auto-pass'],check=True)
    print('All automatic WP-12 gates validated; external responsibilities retained')

if __name__=='__main__':main()
