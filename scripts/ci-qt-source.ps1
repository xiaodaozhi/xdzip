$ErrorActionPreference = 'Stop'
function Checked {
    param([string]$Program, [string[]]$Arguments)
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed; inspect source Qt diagnostic records" }
}
if ($IsWindows) {
    $vswhere = "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    Import-Module "$installation/Common7/Tools/Microsoft.VisualStudio.DevShell.dll"
    Enter-VsDevShell -VsInstallPath $installation -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
}
Checked rustup @('toolchain','install','1.99.0','--profile','minimal','--component','rust-src','--component','rustfmt','--component','clippy')
Checked python @('scripts/build-qt-source.py','--output','build/qt-source','--resume','--parallel','3')
Checked python @('scripts/verify-qt-source-product.py','--qt-build','build/qt-source','--output','build/qt-source-product','--parallel','3')
if ($IsWindows -or $IsLinux) {
    Checked python @('scripts/verify-alpha8-qt-replacement.py','--qt-build','build/qt-source','--output','build/alpha8-qt-replacement')
}
