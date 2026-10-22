#!/usr/bin/env python3
"""CI-only installer filesystem smoke. QA payload stubs are never executed."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from deployment_audit import digest, ROOT


def main():
    if sys.platform != 'win32' or os.environ.get('CI') != 'true':
        raise SystemExit('Native installer smoke requires disposable Windows CI')
    spec=importlib.util.spec_from_file_location('windows_builder',ROOT/'scripts/build-windows-installer.py')
    builder=importlib.util.module_from_spec(spec);spec.loader.exec_module(builder)
    output=ROOT/'build/installer-smoke';output.mkdir()
    for label,variant,legacy in [('legacy','1',True),('qa1','1',False),('qa2','2',False)]:
        payload=output/('payload-'+label);payload.mkdir()
        (payload/'xdzip.exe').write_bytes(b'MZ QA file fixture, never executed\n')
        # Compile the real agreement and bilingual notice even for the owned
        # file stubs. Missing supplier text must not be hidden by QA-only pages.
        license_dir=payload/'share/notices/Microsoft-VC-runtime';license_dir.mkdir(parents=True)
        shutil.copyfile(ROOT/'build/windows-runtime/runtime/Microsoft-VC-runtime-license.rtf',license_dir/'license.rtf')
        notice=ROOT/'apps/desktop/packaging/windows/Windows-runtime-NOTICE.txt'
        (license_dir.parent/notice.name).write_bytes(b'\xef\xbb\xbf'+notice.read_bytes())
        builder.runtime_agreement(payload)
        if variant=='1':
            (payload/'retired').mkdir();(payload/'retired/old-owned.txt').write_bytes(b'owned fixture\n')
        audit={'status':'passed','platform':'windows','mode':'deployed',
               'identity':{'version':'0.1.0-dev.5','build':5,'revision':'a'*12,'dirty':False},
               'files':[{'path':p.relative_to(payload).as_posix(),'sha256':digest(p)} for p in payload.rglob('*') if p.is_file()]}
        package=output/(label+'.exe')
        text,record=builder.render(payload,audit,package,ROOT/'build/installer-toolchain/ChineseSimplified.isl',qa=variant,legacy=legacy)
        package.with_suffix('.files').write_bytes(builder.payload_manifest(audit).encode('utf-8'))
        package.with_suffix('.iss').write_text(text,encoding='utf-8-sig')
        subprocess.run([str(ROOT/'build/installer-toolchain/compiler/ISCC.exe'),str(package.with_suffix('.iss'))],check=True)
        package.with_suffix('.json').write_text(json.dumps(record),encoding='utf-8')
    subprocess.run(['pwsh','-NoProfile','-File',str(ROOT/'scripts/test-windows-installer-native.ps1')],check=True)


if __name__=='__main__':main()
