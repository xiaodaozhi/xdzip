# Run inside the same shell as MSVC discovery so Cargo's cc build scripts inherit it.
$ErrorActionPreference = 'Stop'
if ($IsMacOS) { $env:MACOSX_DEPLOYMENT_TARGET = '14.0' }
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
$env:XDZIP_MOVE_INTEROP_OUTPUT = Join-Path (Get-Location) 'build/move-interop'
$env:XDZIP_UPDATE_INTEROP_OUTPUT = Join-Path (Get-Location) 'build/update-interop'
$env:XDZIP_METADATA_INTEROP_OUTPUT = Join-Path (Get-Location) 'build/macos-metadata-interop'
Invoke-Checked cargo @('test', '--workspace', '--locked')
$updatePython = if ($IsWindows) { 'python' } else { 'python3' }
Invoke-Checked $updatePython @('scripts/check-update-interop.py', '--input', 'build/update-interop')
Invoke-Checked $updatePython @('scripts/check-macos-metadata-interop.py', '--input', 'build/macos-metadata-interop')
Invoke-Checked $updatePython @('scripts/check-move-interop.py', '--input', 'build/move-interop')
Invoke-Checked $updatePython @('scripts/test-release-updates.py')
Invoke-Checked cmake @('-S', '.', '-B', 'build/foundation', '-G', 'Ninja', '-DCMAKE_BUILD_TYPE=Release', '-DXDZIP_BUILD_QT_PROBE=OFF')
Invoke-Checked cmake @('--build', 'build/foundation')
Invoke-Checked ctest @('--test-dir', 'build/foundation', '--output-on-failure')
