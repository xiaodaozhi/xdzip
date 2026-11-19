param([Parameter(Mandatory)][ValidateSet('Build', 'Verify')][string]$Stage)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

if ($Stage -eq 'Build') {
    $env:XDZIP_CLI_RELEASE_BUILD = '1'
    & ./scripts/ci-desktop.ps1 -ReleaseBuild
    python scripts/cli.py package --build-dir build/desktop/cli --output build/release-cli
    $cli = Get-Content -Raw -LiteralPath 'build/release-cli/cli-package.json' | ConvertFrom-Json
    Copy-Item -LiteralPath "build/release-cli/$($cli.file)", 'build/release-cli/cli-package.json' -Destination build/review
    python scripts/release_handoff.py pack
    return
}

. "$PSScriptRoot/ci-observability.ps1"
Initialize-CiTiming -Scope release-verify -OutputPath 'build/review/release-verify-timings.json'
if ($IsWindows) {
    $vswhere = "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (!$installation) { throw 'MSVC verification tools not found' }
    Import-Module "$installation/Common7/Tools/Microsoft.VisualStudio.DevShell.dll"
    Enter-VsDevShell -VsInstallPath $installation -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
}
Set-CiPhase 'package-verification'
$arguments = @('scripts/desktop.py', 'verify', '--build-dir', 'build/desktop', '--output-dir', 'build/review')
if ($IsMacOS) {
    $arguments += '--release-distribution'
} elseif ($IsWindows) {
    $arguments += @('--qt-prefix', 'build/release-verify-qt')
}
Invoke-CiCommand -Program python -Arguments $arguments
if (!$IsMacOS) {
    Set-CiPhase 'installer-lifecycle'
    Invoke-CiCommand -Program pwsh -Arguments @('-NoProfile', '-File', 'scripts/ci-installer-lifecycle.ps1')
}
Set-CiPhase 'export-verified-packages'
Invoke-CiCommand -Program python -Arguments @('scripts/release_handoff.py', 'stage')
