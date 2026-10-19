#!/usr/bin/env python3
"""Build a relocatable Linux/x86_64 Qt runtime directory from our trusted build.

Only explicit SDK files are copied; host graphics/glibc/desktop libraries remain OS
requirements. ldd inspects our own executable/SDK only, never user-supplied inputs.
"""
import argparse
import functools
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
from deployment_audit import digest, linux_imports, native, within

ROOT=Path(__file__).resolve().parents[1]
STATIC_MODULES={'Xdzip.Settings','Xdzip.Native'}
OPTIONAL_NATIVE=('QtQuick.Controls.Windows','QtQuick.Controls.macOS','QtQuick.Controls.iOS','QtQuick.NativeStyle')


def command(*args,env=None):
    result=subprocess.run(args,env=env,capture_output=True,text=True,timeout=60)
    if result.returncode:raise ValueError(f'{Path(args[0]).name} failed during deployment')
    return result.stdout


def qml_roots(entries,sdk):
    roots=set();omitted=[]
    for item in entries:
        if item.get('type')!='module':continue
        name=item['name']
        if name in STATIC_MODULES:continue
        if not item.get('path'):
            if name.startswith(OPTIONAL_NATIVE):omitted.append(name);continue
            raise ValueError('Unresolved QML module: '+name)
        path=Path(item['path']).resolve()
        if not within(path,sdk/'qml'):raise ValueError('QML dependency escapes the declared SDK')
        relative=path.relative_to((sdk/'qml').resolve())
        if not relative.parts:raise ValueError('QML module has no bounded directory')
        roots.add(relative.parts[0])
    if not {'QtQuick','QtQml'}<=roots:raise ValueError('Required Qt Quick/QML runtime modules missing')
    return sorted(roots),sorted(omitted)


def relative_rpath(binary,root):
    if not within(binary,root):raise ValueError('RPATH target outside output')
    return '$ORIGIN/'+os.path.relpath(root/'lib',binary.parent).replace('\\','/')


@functools.lru_cache(maxsize=None)
def system_package(path):
    candidates=[str(path)]
    if str(path).startswith('/usr/lib/'):candidates.append(str(path)[4:])
    for candidate in candidates:
        result=subprocess.run(['dpkg-query','-S',candidate],capture_output=True,text=True)
        if not result.returncode:
            name=result.stdout.splitlines()[0].split(': ',1)[0]
            if not re.fullmatch(r'[a-z0-9][a-z0-9+.-]*(?::amd64)?',name) or name.split(':')[0].endswith('-dev'):
                raise ValueError('Unbounded or development-only system dependency')
            return name
    raise ValueError('System runtime dependency has no Debian package owner: '+path.name)


def runtime_origin(path,sdk,output):
    if within(path,output):return 'artifact'
    if within(path,sdk):return 'sdk'
    if str(path).startswith(('/lib/','/lib64/','/usr/lib/','/usr/lib64/')) and not path.name.startswith('libQt'):
        return 'system'
    raise ValueError('Unexpected external runtime dependency: '+path.name)


def deploy(app,sdk,output,notices):
    if sys.platform!='linux' or platform.machine()!='x86_64':raise ValueError('Deployment requires native Linux x86_64')
    app,sdk,output=app.resolve(),sdk.resolve(),output.resolve()
    if not within(output,ROOT/'build') and not within(output,ROOT/'dist'):
        raise ValueError('Generated application belongs under build/ or dist/')
    if sdk in {Path('/'),Path('/usr'),Path('/usr/local'),Path('/opt')}:raise ValueError('Explicit standalone Qt SDK prefix required')
    if output.exists():raise ValueError('Refusing to overwrite an existing application directory')
    if not app.is_file() or not native(app,'linux'):raise ValueError('Expected trusted native application')
    env=dict(os.environ,LD_LIBRARY_PATH=str(sdk/'lib'))
    if command(str(sdk/'bin/qmake'),'-query','QT_VERSION',env=env).strip()!='6.8.2':
        raise ValueError('Linux deployment requires the pinned Qt 6.8.2 SDK')
    scanner=sdk/'libexec/qmlimportscanner'
    entries=json.loads(command(str(scanner),'-rootPath',str(ROOT/'apps/desktop/ui'),'-importPath',str(sdk/'qml'),env=env))
    roots,omitted=qml_roots(entries,sdk)
    output.mkdir(parents=True)
    copied=[]
    def copy(source,dest):
        dest.parent.mkdir(parents=True,exist_ok=True)
        shutil.copy2(source,dest)
        copied.append({'source':'qt-sdk/'+source.relative_to(sdk).as_posix() if within(source,sdk) else 'product',
                       'path':dest.relative_to(output).as_posix()})
    copy(app,output/'bin/xdzip')
    for name in roots:
        source=sdk/'qml'/name
        for item in source.rglob('*'):
            if not within(item,sdk):raise ValueError('QML SDK symlink escapes its root')
        shutil.copytree(source,output/'qml'/name,ignore=shutil.ignore_patterns('*.a','*.debug','*.prl'))
    # The initial tested desktop contract is X11. Do not ship untested EGLFS,
    # direct-framebuffer or Wayland platform plugins as if they were validated.
    for name in ['libqxcb.so','libqoffscreen.so']:
        source=sdk/'plugins/platforms'/name
        if not source.is_file():raise ValueError('Required platform plugin missing: '+name)
        copy(source,output/'plugins/platforms'/name)
    for group in ['imageformats','iconengines','platforminputcontexts','xcbglintegrations']:
        source=sdk/'plugins'/group
        if source.is_dir():shutil.copytree(source,output/'plugins'/group)
    translations=output/'translations';translations.mkdir()
    for path in (sdk/'translations').glob('qtbase_*.qm'):copy(path,translations/path.name)
    if not (translations/'qtbase_zh_CN.qm').is_file():raise ValueError('System Chinese Qt translations missing')
    pending=[p for p in output.rglob('*') if p.is_file() and native(p,'linux')]
    inspected=set();packages=set();system_libraries=set()
    while pending:
        binary=pending.pop()
        if binary in inspected:continue
        inspected.add(binary)
        for name,source in linux_imports(command('ldd',str(binary),env=env)):
            if source is None or not source.is_file():raise ValueError('Unresolved runtime import: '+name)
            source=source.resolve()
            origin=runtime_origin(source,sdk,output)
            if origin=='artifact':
                # ICU and some SDK binaries use DT_RPATH=$ORIGIN, which can prefer
                # an already copied sibling over the staging SDK search path.
                if source not in inspected:pending.append(source)
                continue
            if origin=='sdk':
                if Path(name).name!=name:raise ValueError('Unsafe native import name')
                target=output/'lib'/name
                if target.exists():
                    # Never silently merge two different libraries with one loader name.
                    previous=next((c for c in copied if c['path']==target.relative_to(output).as_posix()),None)
                    if previous and previous['source']!='qt-sdk/'+source.relative_to(sdk).as_posix():
                        raise ValueError('Conflicting runtime library names')
                else:copy(source,target);pending.append(target)
            else:
                packages.add(system_package(source));system_libraries.add(name)
        command('patchelf','--set-rpath',relative_rpath(binary,output),str(binary))
        if command('patchelf','--print-rpath',str(binary)).strip()!=relative_rpath(binary,output):
            raise ValueError('Relocatable RPATH verification failed')
    (output/'bin/qt.conf').write_text('[Paths]\nPrefix=..\nLibraries=lib\nPlugins=plugins\nQmlImports=qml\nTranslations=translations\n')
    launcher=output/'xdzip'
    launcher.write_text('''#!/bin/sh
set -eu
xdzip_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# Only the deployed runtime participates; users may replace these shared libraries.
export LD_LIBRARY_PATH="$xdzip_dir/lib"
export QT_PLUGIN_PATH="$xdzip_dir/plugins"
export QML_IMPORT_PATH="$xdzip_dir/qml"
export QML2_IMPORT_PATH="$xdzip_dir/qml"
exec "$xdzip_dir/bin/xdzip" "$@"
''')
    launcher.chmod(0o755)
    shutil.copytree(notices,output/'share/notices')
    shutil.copy2(ROOT/'design/assets/brand/xdzip.png',output/'share/xdzip.png')
    shutil.copy2(ROOT/'apps/desktop/packaging/linux/README.md',output/'README.md')
    report={'schema_version':1,'baseline':'Ubuntu 24.04 x86_64, X11','qt':'6.8.2',
            'entry':'xdzip','executable':'bin/xdzip','qml_roots':roots,'optional_native_modules_omitted':omitted,
            'patched_elf_files':len(inspected),'copied_libraries':[x for x in copied if x['path'].startswith('lib/')],
            'system_packages':sorted(packages),'system_libraries':sorted(system_libraries),
            'limits':['Relocatable runtime directory, not an installer or universal Linux compatibility claim.',
                      'Desktop IME/readers, physical GPU, Wayland, clean install/upgrade and full license attribution remain separate gates.']}
    # Bind original SDK bytes to final bytes after patchelf; filenames alone do
    # not establish provenance for libraries rewritten during deployment.
    origins = {c['path']:c['source'].removeprefix('qt-sdk/') for c in copied if c['source'].startswith('qt-sdk/')}
    for group in ['qml','plugins','translations']:
        for path in (output/group).rglob('*'):
            if path.is_file():origins[path.relative_to(output).as_posix()] = path.relative_to(output).as_posix()
    report['sdk_files'] = [{'path':path,'sdk_path':source,'sdk_sha256':digest(sdk/source),
                            'artifact_sha256':digest(output/path)} for path,source in sorted(origins.items())]
    (output/'share/deployment.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app',type=Path,required=True);parser.add_argument('--qt-prefix',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True);parser.add_argument('--notices',type=Path,required=True)
    args=parser.parse_args()
    try:report=deploy(args.app,args.qt_prefix,args.output,args.notices)
    except (ValueError,OSError,subprocess.SubprocessError) as error:raise SystemExit(str(error))
    print(f'Linux runtime directory: {report["patched_elf_files"]} relocated ELF files, {len(report["system_packages"])} OS packages')

if __name__=='__main__':main()
