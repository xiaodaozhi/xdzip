#requires -Version 5.1
<#
.SYNOPSIS
Capture all four Store scenes for one language, guided by F8.
.DESCRIPTION
Run in Windows PowerShell 5.1. The assistant starts the candidate with isolated
settings and two demo archives. Prepare each requested scene, then press F8 in
xdzip. F10 cancels. It saves native-resolution, lossless client and window PNGs.
It does not click controls, change system scaling, install packages or upload.
.EXAMPLE
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language zh-CN
.EXAMPLE
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language en-US
#>
[CmdletBinding(DefaultParameterSetName = 'Capture')]
param(
    [Parameter(Mandatory = $true, ParameterSetName = 'Capture')]
    [ValidateSet('zh-CN', 'en-US')][string]$Language,
    [Parameter(ParameterSetName = 'Capture')][string]$Executable,
    [Parameter(ParameterSetName = 'Capture')][string]$DemoDirectory,
    [string]$OutputDirectory,
    [Parameter(ParameterSetName = 'Capture')][switch]$ValidateOnly,
    [Parameter(Mandatory = $true, ParameterSetName = 'SelfTest')][switch]$SelfTest
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT' -or $PSVersionTable.PSEdition -ne 'Desktop') {
    throw 'Run this assistant with Windows PowerShell 5.1: powershell.exe -NoProfile -File ...'
}
$repoRoot = Split-Path -Parent $PSScriptRoot
function Get-CaptureSha256([string]$Path) {
    # Use the framework directly so cmd.exe launchers do not depend on an
    # inherited PSModulePath from another PowerShell installation.
    $algorithm = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::OpenRead($Path)
    try { [BitConverter]::ToString($algorithm.ComputeHash($stream)).Replace('-', '').ToLowerInvariant() }
    finally { $stream.Dispose(); $algorithm.Dispose() }
}
$materialRoot = Join-Path $repoRoot 'build/issue272-store-materials'
# A worktree reuses the existing candidate, rather than copying its runtime.
if ((!$Executable -or !$DemoDirectory) -and !(Test-Path -LiteralPath (Join-Path $materialRoot 'runtime/xdzip.exe'))) {
    $common = (& git -C $repoRoot rev-parse --path-format=absolute --git-common-dir 2>$null)
    if ($LASTEXITCODE -eq 0 -and $common) {
        $sharedMaterials = Join-Path (Split-Path -Parent $common) 'build/issue272-store-materials'
        if (Test-Path -LiteralPath $sharedMaterials) { $materialRoot = $sharedMaterials }
    }
}
if (!$OutputDirectory) {
    $label = if ($SelfTest) { 'self-test' } else { $Language }
    $OutputDirectory = Join-Path $materialRoot ('native-captures/{0}-{1}-{2}' -f $label, (Get-Date -Format 'yyyyMMdd-HHmmss'), [guid]::NewGuid().ToString('N').Substring(0, 6))
}
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $OutputDirectory) { throw 'OutputDirectory already exists. Choose a new directory to preserve earlier captures.' }
if (!('Xdzip.StoreCapture.Native' -as [type])) {
    Add-Type -Path (Join-Path $PSScriptRoot 'windows-store-capture.cs') -ReferencedAssemblies System.Drawing
}
if ($SelfTest) {
    $null = New-Item -ItemType Directory -Path $OutputDirectory
    [Xdzip.StoreCapture.Native]::CodecSelfTest((Join-Path $OutputDirectory 'rgba-roundtrip.png'))
    Write-Host 'PASS: exact PNG RGBA roundtrip, native DPI size guard, exclusive output writes. No desktop capture or app launch.'
    return
}
if (!$Executable) { $Executable = Join-Path $materialRoot 'runtime/xdzip.exe' }
if (!$DemoDirectory) { $DemoDirectory = Join-Path $materialRoot 'demo' }
$Executable = (Resolve-Path -LiteralPath $Executable).ProviderPath
$DemoDirectory = (Resolve-Path -LiteralPath $DemoDirectory).ProviderPath
$spec = Get-Content -LiteralPath (Join-Path $repoRoot 'specs/marketing/windows-store.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$layout = Get-Content -LiteralPath (Join-Path $repoRoot 'specs/visual/layout-scale.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$catalog = Get-Content -LiteralPath (Join-Path $repoRoot "resources/i18n/$Language.json") -Raw -Encoding UTF8 | ConvertFrom-Json
$exeHash = Get-CaptureSha256 $Executable
if ($exeHash -ne $spec.baseline.runtime_exe_sha256) { throw 'Executable differs from the Store candidate recorded in windows-store.json.' }
$browseName = if ($Language -eq 'zh-CN') { '设计交付.zip' } else { 'Design delivery.zip' }
$photoName = if ($Language -eq 'zh-CN') { '图片精选.zip' } else { 'Photo collection.zip' }
$demoInputs = @($browseName, $photoName, 'coastline.png', 'dusk.png', 'forest.png')
foreach ($name in $demoInputs) {
    if (!(Test-Path -LiteralPath (Join-Path $DemoDirectory $name) -PathType Leaf)) { throw "Missing demonstration file: $name" }
}
$currentSession = (Get-Process -Id $PID).SessionId
$existing = @(Get-Process -Name xdzip -ErrorAction SilentlyContinue | Where-Object { $_.SessionId -eq $currentSession })
if ($existing.Count) { throw 'Close existing xdzip processes before starting, so single-instance forwarding cannot target your normal session.' }
if ($ValidateOnly) {
    Write-Host "PASS: $Language, candidate SHA-256, five demonstration files, fresh output path. No files written or app launched."
    return
}

$null = New-Item -ItemType Directory -Path $OutputDirectory
$settings = Join-Path $OutputDirectory 'settings.ini'
$utf8 = New-Object Text.UTF8Encoding($false)
[IO.File]::WriteAllText($settings, @"
[General]
schemaVersion=1
[appearance]
theme=light
textScale=1
density=standard
[locale]
language=$Language
[desktop]
shellEnabled=false
trayEnabled=false
openMode=tabs
[privacy]
recentEnabled=false
"@, $utf8)

if ($Language -eq 'zh-CN') {
    $scenes = @(
        @{ id = '01-browse'; hint = '切到“设计交付.zip”，在列表视图中选中一个文件，让右侧显示属性。' },
        @{ id = '03-preview'; hint = '切到“图片精选.zip”，选中 coastline.png，点击预览并等待图片完整显示。' },
        @{ id = '04-workspace'; hint = '保留两个标签，在“图片精选.zip”切换为网格视图，选中图片。' },
        @{ id = '02-create'; hint = '按 Ctrl+N 打开创建窗口，添加演示目录中的三张 PNG，目标目录设为演示目录，保留 ZIP 和压缩选项。只截图，不点击创建。' }
    )
} else {
    $scenes = @(
        @{ id = '01-browse'; hint = 'Switch to Design delivery.zip, use list view, select a file and show its properties.' },
        @{ id = '03-preview'; hint = 'Switch to Photo collection.zip, select coastline.png, request Preview and wait for the complete image.' },
        @{ id = '04-workspace'; hint = 'Keep both tabs open, switch Photo collection.zip to grid view and select an image.' },
        @{ id = '02-create'; hint = 'Press Ctrl+N, add the three demo PNGs, set the destination to the demo directory and keep ZIP options visible. Do not start creation.' }
    )
}
$receipt = [ordered]@{
    schema_version = 1; complete = $false; language = $Language
    baseline_revision = $spec.baseline.revision; runtime_exe_sha256 = $exeHash
    method = 'Native screen pixels, DWM visible frame and exact client crop, lossless PNG, no resampling'
    scene_preparation = 'User operated, F8 confirms each scene'; images = @()
}
function Save-Receipt {
    [IO.File]::WriteAllText((Join-Path $OutputDirectory 'capture-session.json'), ($receipt | ConvertTo-Json -Depth 8), $utf8)
}
$process = $null
$keysRegistered = $false
try {
    [Xdzip.StoreCapture.Native]::RegisterKeys()
    $keysRegistered = $true
    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = $Executable
    $start.WorkingDirectory = $DemoDirectory
    $start.UseShellExecute = $false
    # Paths are literal argv data, never a shell command. Windows filenames
    # cannot contain quotes and these archive paths have no trailing backslash.
    $start.Arguments = '-- "' + (Join-Path $DemoDirectory $browseName) + '" "' + (Join-Path $DemoDirectory $photoName) + '"'
    $start.EnvironmentVariables['XDZIP_SETTINGS_PATH'] = $settings
    $start.EnvironmentVariables['XDZIP_LANGUAGE'] = $Language
    foreach ($name in @('QT_QPA_PLATFORM', 'QT_QUICK_BACKEND', 'QSG_RHI_BACKEND', 'QT_SCALE_FACTOR', 'QT_SCREEN_SCALE_FACTORS', 'QT_FONT_DPI', 'QT_DEVICE_PIXEL_RATIO', 'QT_ENABLE_HIGHDPI_SCALING')) {
        $start.EnvironmentVariables.Remove($name)
    }
    $process = [Diagnostics.Process]::Start($start)
    $mainWindow = 0L
    $timer = [Diagnostics.Stopwatch]::StartNew()
    while (!$mainWindow -and $timer.Elapsed.TotalSeconds -lt 25) {
        if ($process.HasExited) { throw 'Candidate exited before exposing its own window. Close existing xdzip instances and retry.' }
        $mainWindow = [Xdzip.StoreCapture.Native]::FindMain($process.Id, $layout.main.logical_window[0], $layout.main.logical_window[1])
        Start-Sleep -Milliseconds 100
    }
    if (!$mainWindow) { throw 'Default-size candidate window not found. No screenshots captured.' }
    Write-Host "`nDemo files: $DemoDirectory"
    Write-Host 'Prepare each scene, move the pointer away from the window and press F8. F10 cancels. Keep the default size and system scaling.'
    foreach ($scene in $scenes) {
        Write-Host "`n[$($scene.id)] $($scene.hint)" -ForegroundColor Cyan
        $captured = $false
        while (!$captured) {
            if ($process.HasExited) { throw 'The capture-owned xdzip process has exited.' }
            $key = [Xdzip.StoreCapture.Native]::PollKey()
            if ($key -eq 2) { throw 'Capture canceled with F10. Earlier PNGs are retained.' }
            if ($key -eq 1) {
                # The native helper treats null/empty titles as main-window mode.
                $title = if ($scene.id -eq '02-create') { $catalog.messages.'ui.fe189b616c1d' } else { $null }
                try {
                    $image = [Xdzip.StoreCapture.Native]::Capture($process.Id, $mainWindow, $title,
                        $layout.main.logical_window[0], $layout.main.logical_window[1],
                        (Join-Path $OutputDirectory "$Language-$($scene.id).png"),
                        (Join-Path $OutputDirectory "$Language-$($scene.id)-window.png"))
                } catch {
                    $cause = $_.Exception.GetBaseException()
                    if ($cause -isnot [InvalidOperationException]) { throw }
                    Write-Warning $cause.Message
                    Write-Host 'Fix the window state and press F8 again. F10 cancels.'
                    continue
                }
                $receipt.images += [ordered]@{
                    scene = $scene.id; file = $image.ClientFile
                    sha256 = Get-CaptureSha256 (Join-Path $OutputDirectory $image.ClientFile)
                    width = $image.ClientWidth; height = $image.ClientHeight; dpi = $image.Dpi
                    window_file = $image.WindowFile
                    window_sha256 = Get-CaptureSha256 (Join-Path $OutputDirectory $image.WindowFile)
                    window_dimensions = @($image.WindowWidth, $image.WindowHeight)
                    client_crop = @($image.ClientLeft, $image.ClientTop, $image.ClientWidth, $image.ClientHeight)
                }
                Save-Receipt
                Write-Host "Saved $($image.ClientFile), $($image.ClientWidth) x $($image.ClientHeight), DPI $($image.Dpi)." -ForegroundColor Green
                try { [Console]::Beep(880, 100) } catch { Write-Verbose 'Capture saved; sound unavailable.' }
                $captured = $true
            }
            Start-Sleep -Milliseconds 80
        }
    }
    $receipt.complete = $true
    Save-Receipt
    Write-Host "`nComplete: four client PNGs, four native-window references and capture-session.json.`n$OutputDirectory" -ForegroundColor Green
} finally {
    if ($keysRegistered) { [Xdzip.StoreCapture.Native]::ReleaseKeys() }
    try { Save-Receipt } finally {
        # Only the process created by this invocation may be closed. There are no
        # submitted archive operations in this workflow; never stop a process by name.
        if ($process -and !$process.HasExited) { $process.Kill(); $process.WaitForExit() }
        if ($process) { $process.Dispose() }
    }
}
