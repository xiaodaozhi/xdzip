#!/usr/bin/env python3
"""Keep bounded sanitized CI diagnostics, no SDK or application uploads."""
from pathlib import Path
from deployment_audit import ROOT

out=ROOT/'build/qt-source-records';out.mkdir(parents=True,exist_ok=True)
for base in [ROOT/'build/qt-source',ROOT/'build/qt-source-product',ROOT/'build/alpha8-qt-replacement']:
    if not base.exists():continue
    for file in list(base.glob('*.log'))+list(base.glob('*.json')):
        text=file.read_text(errors='replace').replace(str(ROOT),'<workspace>').replace(ROOT.as_posix(),'<workspace>').replace(str(Path.home()),'<home>').replace(Path.home().as_posix(),'<home>')
        if file.suffix=='.log':text=text[-160000:]
        (out/file.name).write_text(text,encoding='utf-8')
print('Collected bounded source-build validation records only')
