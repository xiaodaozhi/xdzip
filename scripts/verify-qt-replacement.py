#!/usr/bin/env python3
"""Explicit local Mac source-built QtCore replacement in an isolated developer app."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from macos_launch_services import temporary_bundle_path, transient_registrations
from deployment_audit import ROOT, audit, digest, within, mac_rpaths


def module(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/'scripts'/(name+'.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['app','qt-build','metrics','output']:p.add_argument('--'+name,type=Path,required=True)
    a=p.parse_args();out=a.output.resolve()
    if sys.platform!='darwin' or out.exists() or not within(out,ROOT/'build'):p.error('Use a new local Mac build/ output')
    source_record=json.loads((a.qt_build/'qt-source-build.json').read_text());new_core=a.qt_build/'qt-install/lib/QtCore.framework/Versions/A/QtCore'
    if digest(new_core)!=source_record['qtcore_sha256']:raise ValueError('Source-built QtCore changed')
    mac=module('build-macos-dmg');out.mkdir(parents=True)
    app=out/'中文 relocated/xdzip.app'
    with temporary_bundle_path(app), transient_registrations(out):
        shutil.copytree(a.app,app,symlinks=True);binary=mac.validate_bundle(app)
        identity=mac.checked([binary,'--version']).strip()
        if 'dirty' in identity or 'unknown' in identity or source_record['revision'][:12] not in identity:raise ValueError('Replacement source/product identity mismatch')
        core=app/'Contents/Frameworks/QtCore.framework/Versions/A/QtCore';before=digest(core)
        shutil.copy2(new_core,core)
        for value in mac_rpaths(mac.checked(['otool','-l',core])):
            if value.startswith('/') and not value.startswith(('/usr/lib/','/System/Library/')):mac.checked(['install_name_tool','-delete_rpath',value,core])
        mac.sign_local(app)
        closure=audit(app,binary,None,'macos','deployed')
        replaced_core=digest(core)
        if closure['status']!='passed' or before==replaced_core:raise ValueError('Replacement not SDK-independent or unchanged')
        env=dict(os.environ,PATH='/usr/bin:/bin:/usr/sbin:/sbin',QT_QPA_PLATFORM='cocoa',QSG_RHI_BACKEND='metal',XDZIP_EXPECT_QT_SOURCE_PROOF='xdzip-qt-6.8.2-source-proof-v1')
        for key in ['QT_PLUGIN_PATH','QML_IMPORT_PATH','QML2_IMPORT_PATH','DYLD_LIBRARY_PATH','DYLD_FRAMEWORK_PATH','QT_QUICK_BACKEND','QT_QPA_PLATFORM_PLUGIN_PATH']:env.pop(key,None)
        metadata=out/'metadata.zip';payload=out/'payload.zip'
        for script,target in [('generate-m2-fixture.py',metadata),('generate-m3-fixture.py',payload)]:subprocess.run([sys.executable,str(ROOT/'scripts'/script),'--output',str(target)],check=True,capture_output=True)
        with (out/'replacement.log').open('w') as log:
            for fixture,extra,label in [(metadata,[],'metadata'),(payload,['--operation-extract','--expected-records','64'],'extract')]:
                cmd=[sys.executable,str(ROOT/'scripts/measure-desktop.py'),'--app',str(binary),'--metrics',str(a.metrics.resolve()),'--fixture',str(fixture),'--runs','1','--require-backend','metal','--output',str(out/(label+'.json'))]+extra
                subprocess.run(cmd,env=env,stdout=log,stderr=log,check=True)
        runs=[json.loads((out/(k+'.json')).read_text())['runs'][0] for k in ['metadata','extract']]
        for run in runs:
            proof=next(e for e in run['events'] if e['event']=='qt_source_proof')
            if proof['value']!=env['XDZIP_EXPECT_QT_SOURCE_PROOF']:raise ValueError('Actually loaded QtCore marker missing')
        app_sha256=digest(binary)
    result={'schema_version':1,'status':'passed','platform':'macos','revision':source_record['revision'],'identity':identity,
            'source_build_sha256':digest(a.qt_build/'qt-source-build.json'),'source_qtcore_sha256':source_record['qtcore_sha256'],
            'original_qtcore_sha256':before,'replaced_signed_qtcore_sha256':replaced_core,'app_sha256':app_sha256,
            'source_marker_verified':True,'sdk_independent':True,'independent_extraction_verified':runs[1]['extraction']['independent_byte_crc_check'],
            'developer_id_signed':False,'notarized':False,
            'reports':[{'file':k+'.json','sha256':digest(out/(k+'.json'))} for k in ['metadata','extract']],
            'limits':['Source-built compatible QtCore actually used with other original Qt modules in isolated app; not arbitrary ABI or original SDK binary equivalence.',
                      'Local ad-hoc integrity only. Original installed app and SDK unchanged; no Developer ID/notarization or legal approval.']}
    (out/'qt-source-replacement.json').write_text(json.dumps(result,indent=2)+'\n');print('Actual source-built QtCore loaded, 100k browsing/query/cancel and 32MiB byte-verified extraction passed')

if __name__=='__main__':main()
