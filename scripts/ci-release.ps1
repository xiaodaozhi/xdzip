param([Parameter(Mandatory)][ValidateSet('Build', 'Verify', 'QtNetwork', 'QtImage', 'Dependencies')][string]$Stage,
    [ValidateSet('auto', 'cold')][string]$QtNetworkMode = 'cold',
    [ValidateSet('cold', 'hit')][string]$QtImageMode,
    [ValidateSet('auto', 'cold')][string]$DependencyMode = 'auto', [switch]$PublishDependencies)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
$packages = if ($IsMacOS) { 'build/release-package' } else { 'build/review' }
$networkToken = $env:XDZIP_RELEASE_QT_NETWORK_TOKEN
Remove-Item Env:XDZIP_RELEASE_QT_NETWORK_TOKEN -ErrorAction SilentlyContinue

if ($Stage -eq 'QtImage') {
    if (!$QtImageMode -or !($IsWindows -or ($IsMacOS -and $env:XDZIP_MACOS_INTEL_CI -eq '1'))) {
        throw 'Select cold or hit on Windows or macOS Intel'
    }
    & ./scripts/ci-desktop.ps1 -ReleaseBuild -QtNetworkMode $QtNetworkMode -QtNetworkToken $networkToken -QtNetworkOnly
    $cacheRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { Join-Path (Get-Location) 'build/toolchain' }
    $qtFolder = if ($IsWindows) { 'msvc2022_64' } else { 'macos' }
    $qtPrefix = Join-Path $cacheRoot "Qt/6.8.2/$qtFolder"
    python scripts/probe-qt-imageformats.py --mode $QtImageMode --qt-prefix $qtPrefix `
        --work-dir build/qt-image-probe/evidence --bundle build/qt-image-probe/qt-image.zip
    return
}

if ($Stage -eq 'QtNetwork') {
    & ./scripts/ci-desktop.ps1 -ReleaseBuild -QtNetworkMode $QtNetworkMode -QtNetworkToken $networkToken -QtNetworkOnly
    return
}

if ($Stage -eq 'Build' -or $Stage -eq 'Dependencies') {
    $env:XDZIP_CLI_RELEASE_BUILD = '1'
    & ./scripts/ci-desktop.ps1 -ReleaseBuild -ReleaseDependencies $DependencyMode -QtNetworkToken $networkToken `
        -DependenciesOnly:($Stage -eq 'Dependencies') -PublishDependencies:$PublishDependencies
    if ($Stage -eq 'Dependencies') { return }
    python scripts/cli.py package --build-dir build/desktop/cli --output build/release-cli
    $cli = Get-Content -Raw -LiteralPath 'build/release-cli/cli-package.json' | ConvertFrom-Json
    Copy-Item -LiteralPath "build/release-cli/$($cli.file)", 'build/release-cli/cli-package.json' -Destination $packages
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
} elseif ($IsLinux) {
    # The fresh runner executes one SDK-free image worker probe outside Docker.
    Set-CiPhase 'verification-prerequisites'
    $deployment = Get-Content -Raw -LiteralPath 'build/review/linux/share/deployment.json' | ConvertFrom-Json
    $systemPackages = @($deployment.system_packages)
    if (!$systemPackages.Count -or @($systemPackages | Where-Object { $_ -cnotmatch '\A[a-z0-9][a-z0-9+.-]*(?::amd64)?\z' }).Count) {
        throw 'Invalid Linux runtime package list'
    }
    Invoke-CiCommand -Program sudo -Arguments @('apt-get', 'update')
    Invoke-CiCommand -Program sudo -Arguments (@('apt-get', 'install', '-y', '--no-install-recommends', '--') + $systemPackages)
}
Set-CiPhase 'package-verification'
$arguments = @('scripts/desktop.py', 'verify', '--build-dir', 'build/desktop', '--output-dir', $packages)
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
