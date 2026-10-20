# scripts/ci-release.ps1

# Dedicated publishing pipeline. Ordinary CI keeps its SDK-only macOS policy.
$ErrorActionPreference = 'Stop'
& ./scripts/ci-desktop.ps1
if ($LASTEXITCODE -ne 0) { throw 'Native desktop validation failed' }
function Checked([string]$Program, [string[]]$Arguments) {
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed: $LASTEXITCODE" }
}
$cacheRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { Join-Path (Get-Location) '.cache' }
$qtFolder = if ($IsMacOS) { 'macos' } elseif ($IsWindows) { 'msvc2022_64' } else { 'gcc_64' }
$qt = Join-Path $cacheRoot "Qt/6.8.2/$qtFolder"
$tools = Join-Path $cacheRoot 'xdzip-desktop-tools'
$shared = @('--qt-prefix', $qt, '--tools-dir', $tools, '--build-dir', 'build/desktop', '--output-dir', 'build/review')
$tests = if ($IsWindows) { 'build/desktop/apps/desktop/xdzip-browser-tests.exe' } else { 'build/desktop/apps/desktop/xdzip-browser-tests' }
# Validate the real platform state transitions, not the offscreen emulation.
$nativeCases = @('desktopActivationPreservesWindowState', 'independentSessionsAndOpenModes', 'singleInstanceStructuredForwarding', 'singleInstanceConcurrentStartupAndCrashRecovery', 'backgroundTasksKeepSessionIdentityAndSerialAdmission')
if ($IsLinux) {
    $env:QT_QPA_PLATFORM = 'xcb'
    Checked xvfb-run (@('-a', $tests) + $nativeCases)
} else {
    $env:QT_QPA_PLATFORM = if ($IsMacOS) { 'cocoa' } else { 'windows' }
    Checked $tests $nativeCases
}
if ($IsMacOS) {
    $shared = @('--qt-prefix', $qt, '--tools-dir', $tools, '--build-dir', 'build/release-desktop',
        '--output-dir', 'build/release-package', '--macos-handler-rank', 'Alternate', '--release-distribution')
    Checked python (@('scripts/desktop.py', 'build') + $shared)
    Checked python (@('scripts/desktop.py', 'test') + $shared)
    Checked python (@('scripts/desktop.py', 'package') + $shared)
    Checked python (@('scripts/desktop.py', 'verify') + $shared)
    $packages = 'build/release-package'
} else {
    Checked python (@('scripts/desktop.py', 'verify', '--kind', 'all') + $shared)
    $packages = 'build/review'
}
$platform = if ($IsMacOS) { 'macos' } elseif ($IsWindows) { 'windows' } else { 'linux' }
Checked python @('scripts/prepare-release.py', '--packages', $packages, '--output', 'build/release-assets', '--platform', $platform)
