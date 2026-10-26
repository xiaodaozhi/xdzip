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
# ci-desktop already completed the shared native session/activation regression.
if ($IsMacOS) {
    $shared = @('--qt-prefix', $qt, '--tools-dir', $tools, '--build-dir', 'build/release-desktop',
        '--output-dir', 'build/release-package', '--macos-handler-rank', 'Alternate', '--release-distribution')
    Checked python (@('scripts/desktop.py', 'build') + $shared)
    # The complete ignored Rust batch needs the same pinned data as Foundation.
    Checked python @('scripts/prepare-rar-rust-fixtures.py', '--output', 'build/release-rar-fixtures')
    $fixtures = Get-Content -LiteralPath 'build/release-rar-fixtures/sources.json' -Raw -Encoding utf8 | ConvertFrom-Json
    $fixtureEnvironment = @{ XDZIP_RAR_FIXTURE_PASSWORD = 'password' }
    foreach ($item in $fixtures.environment.PSObject.Properties) {
        $fixtureEnvironment[$item.Name] = Join-Path (Get-Location) $item.Value
    }
    $previousEnvironment = @{}
    try {
        foreach ($name in $fixtureEnvironment.Keys) {
            $previousEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
            [Environment]::SetEnvironmentVariable($name, $fixtureEnvironment[$name], 'Process')
        }
        Checked python (@('scripts/desktop.py', 'test') + $shared)
    } finally {
        foreach ($name in $previousEnvironment.Keys) {
            if ($null -eq $previousEnvironment[$name]) {
                Remove-Item "Env:$name" -ErrorAction SilentlyContinue
            } else {
                [Environment]::SetEnvironmentVariable($name, $previousEnvironment[$name], 'Process')
            }
        }
    }
    Checked python (@('scripts/desktop.py', 'package') + $shared)
    Checked python (@('scripts/desktop.py', 'verify') + $shared)
    $packages = 'build/release-package'
} else {
    Checked python (@('scripts/desktop.py', 'verify', '--kind', 'all') + $shared)
    $packages = 'build/review'
}
$platform = if ($IsMacOS) { 'macos' } elseif ($IsWindows) { 'windows' } else { 'linux' }
Checked python @('scripts/prepare-release.py', '--packages', $packages, '--output', 'build/release-assets', '--platform', $platform)
