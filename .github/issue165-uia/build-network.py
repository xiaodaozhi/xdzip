import os, subprocess, sys
from pathlib import Path
root=Path.cwd()
sys.path.insert(0,str(root/'scripts'))
from windows_ninja_safety import check_dependencies, utf8_console
out=root/'build/issue165'
qt=Path(os.environ['RUNNER_TEMP'])/'Qt/6.8.2/msvc2022_64'
os.environ['PATH']=str(qt/'bin')+os.pathsep+os.environ['PATH']
os.environ['RUSTUP_AUTO_INSTALL']='0'
with utf8_console():
    check_dependencies(out)
    subprocess.run(['cmake','-S',str(root),'-B',str(out),'-G','Ninja','-DCMAKE_BUILD_TYPE=Release',f'-DCMAKE_PREFIX_PATH={qt}','-DXDZIP_BUILD_DESKTOP_PREVIEW=ON','-DXDZIP_BUILD_QT_PROBE=ON','-DBUILD_TESTING=ON'],check=True)
    subprocess.run(['cmake','--build',str(out),'--target','xdzip-update-tests','--parallel','3'],check=True)
    check_dependencies(out,require_compiled=True)
