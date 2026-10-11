# Run inside the same shell as MSVC discovery so Cargo's cc build scripts inherit it.
$ErrorActionPreference = 'Stop'
if ($IsWindows) {
    $vswhere = "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (!$installation) { throw 'MSVC installation not found' }
    Import-Module "$installation/Common7/Tools/Microsoft.VisualStudio.DevShell.dll"
    Enter-VsDevShell -VsInstallPath $installation -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
}
function Invoke-Checked {
    param([string]$Program, [string[]]$Arguments)
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed with exit code $LASTEXITCODE" }
}
Invoke-Checked rustc @('-vV')
Invoke-Checked cargo @('fmt', '--all', '--', '--check')
Invoke-Checked cargo @('clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings')
Invoke-Checked cargo @('test', '--workspace', '--locked')
Invoke-Checked cmake @('-S', '.', '-B', 'build/foundation', '-G', 'Ninja', '-DCMAKE_BUILD_TYPE=Release', '-DXDZIP_BUILD_QT_PROBE=OFF')
Invoke-Checked cmake @('--build', 'build/foundation')
Invoke-Checked ctest @('--test-dir', 'build/foundation', '--output-on-failure')
