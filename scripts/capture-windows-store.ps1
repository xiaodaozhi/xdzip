#requires -Version 5.1
<#
.SYNOPSIS
Automatically prepare and capture all four Store scenes for one language.
.DESCRIPTION
Run in Windows PowerShell 5.1. The assistant starts the candidate with isolated
settings and two demo archives. It selects files, requests image preview,
switches to grid view and opens a populated creation window automatically.
F10 cancels. It saves native-resolution, lossless client and window PNGs.
It does not change system scaling, install packages or upload.
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
    [Parameter(ParameterSetName = 'Capture')][ValidatePattern('^[A-Fa-f0-9]{64}$')][string]$ExpectedSha256,
    [Parameter(ParameterSetName = 'Capture')][ValidatePattern('^[A-Fa-f0-9]{40}$')][string]$SourceRevision,
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
Import-Module (Join-Path $PSScriptRoot 'windows-capture.psm1')
Import-Module (Join-Path $PSScriptRoot 'windows-store-demo.psm1')
$spec = Get-Content -LiteralPath (Join-Path $repoRoot 'specs/capture/windows-store.json') -Raw -Encoding UTF8 | ConvertFrom-Json
if (!$ExpectedSha256) { $ExpectedSha256 = $spec.baseline.runtime_exe_sha256 }
if (!$Executable -and !$SelfTest) {
    $roots = @($repoRoot)
    $common = (& git -C $repoRoot rev-parse --path-format=absolute --git-common-dir 2>$null)
    if ($LASTEXITCODE -eq 0 -and $common) {
        $roots += Split-Path -Parent $common
    }
    foreach ($workspace in @($roots | Select-Object -Unique)) {
        foreach ($relative in $spec.runtime_candidates) {
            $candidate = Join-Path $workspace $relative
            if ((Test-Path -LiteralPath $candidate -PathType Leaf) -and (Get-WindowsCaptureHash $candidate) -eq $ExpectedSha256.ToLowerInvariant()) {
                $Executable = $candidate
                break
            }
        }
        if ($Executable) { break }
    }
    if (!$Executable) { throw 'No runtime matches the expected SHA-256. Supply -Executable and, for another build, -ExpectedSha256.' }
}
if (!$OutputDirectory) {
    $label = if ($SelfTest) { 'self-test' } else { $Language }
    $OutputDirectory = Join-Path $repoRoot ('build/screenshots/windows-store/{0}-{1}-{2}' -f $label, (Get-Date -Format 'yyyyMMdd-HHmmss'), [guid]::NewGuid().ToString('N').Substring(0, 6))
}
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $OutputDirectory) { throw 'OutputDirectory already exists. Choose a new directory to preserve earlier captures.' }
Initialize-WindowsCapture
if ($SelfTest) {
    $null = New-Item -ItemType Directory -Path $OutputDirectory
    [Xdzip.WindowsCapture.Native]::CodecSelfTest((Join-Path $OutputDirectory 'rgba-roundtrip.png'))
    [Xdzip.WindowsCapture.Native]::PairSelfTest($OutputDirectory)
    Write-Host 'PASS: exact PNG RGBA roundtrip, native DPI size guard, exclusive output writes. No desktop capture or app launch.'
    return
}
$Executable = (Resolve-Path -LiteralPath $Executable).ProviderPath
$generateDemo = !$DemoDirectory
if ($generateDemo) { $DemoDirectory = Join-Path $OutputDirectory 'demo' }
else { $DemoDirectory = (Resolve-Path -LiteralPath $DemoDirectory).ProviderPath }
$layout = Get-Content -LiteralPath (Join-Path $repoRoot 'specs/visual/layout-scale.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$catalog = Get-Content -LiteralPath (Join-Path $repoRoot "resources/i18n/$Language.json") -Raw -Encoding UTF8 | ConvertFrom-Json
$exeHash = Get-WindowsCaptureHash $Executable
if ($exeHash -ne $ExpectedSha256.ToLowerInvariant()) { throw 'Executable differs from the expected capture build SHA-256.' }
$pinnedCandidate = $exeHash -eq $spec.baseline.runtime_exe_sha256
if (!$SourceRevision -and $pinnedCandidate) { $SourceRevision = $spec.baseline.revision }
$browseName = if ($Language -eq 'zh-CN') { '设计交付.zip' } else { 'Design delivery.zip' }
$photoName = if ($Language -eq 'zh-CN') { '图片精选.zip' } else { 'Photo collection.zip' }
$demoInputs = @($browseName, $photoName, 'coastline.png', 'dusk.png', 'forest.png')
if (!$generateDemo) {
    foreach ($name in $demoInputs) {
        if (!(Test-Path -LiteralPath (Join-Path $DemoDirectory $name) -PathType Leaf)) { throw "Missing demonstration file: $name" }
    }
}
$currentSession = (Get-Process -Id $PID).SessionId
$existing = @(Get-Process -Name xdzip -ErrorAction SilentlyContinue | Where-Object { $_.SessionId -eq $currentSession })
if ($existing.Count) { throw 'Close existing xdzip processes before starting, so single-instance forwarding cannot target your normal session.' }
if ($ValidateOnly) {
    Write-Host "PASS: $Language, expected executable SHA-256 and fresh output path. Demo generation: $generateDemo. No files written or app launched."
    return
}

$null = New-Item -ItemType Directory -Path $OutputDirectory
if ($generateDemo) { $null = New-WindowsStoreDemo -Directory $DemoDirectory -Language $Language }
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

$receipt = [ordered]@{
    schema_version = 3; complete = $false; language = $Language
    baseline_revision = $SourceRevision; runtime_exe_sha256 = $exeHash
    pinned_store_candidate = $pinnedCandidate; expected_scenes = @($spec.scenes)
    recipe_sha256 = Get-WindowsCaptureHash $PSCommandPath
    capture_helper_sha256 = Get-WindowsCaptureHash (Join-Path $PSScriptRoot 'windows-capture.cs')
    capture_module_sha256 = Get-WindowsCaptureHash (Join-Path $PSScriptRoot 'windows-capture.psm1')
    capture_config_sha256 = Get-WindowsCaptureHash (Join-Path $repoRoot 'specs/capture/windows-store.json')
    fixture_generator_sha256 = Get-WindowsCaptureHash (Join-Path $PSScriptRoot 'windows-store-demo.psm1')
    demo_source = if ($generateDemo) { 'generated' } else { 'provided' }
    demo_inputs = @($demoInputs | ForEach-Object { @{ file = $_; sha256 = Get-WindowsCaptureHash (Join-Path $DemoDirectory $_) } })
    fixture_revision = $spec.fixture_revision
    method = 'Native screen pixels, DWM visible frame and exact client crop, lossless PNG, no resampling'
    scene_preparation = 'Automatic owned UI Automation and create shell request'; images = @()
}
function Save-Receipt {
    [IO.File]::WriteAllText((Join-Path $OutputDirectory 'capture-session.json'), ($receipt | ConvertTo-Json -Depth 8), $utf8)
}
function Wait-State([string]$Label, [scriptblock]$Predicate) {
    Wait-WindowsCaptureState -Process $process -Label $Label -Predicate $Predicate
}
function Wait-Control([long]$Window, [string]$Name, [string]$Role) {
    Wait-State "$Name ($Role)" { [Xdzip.WindowsCapture.Driver]::Ready($process.Id, $Window, $Name, $Role) }
}
function Invoke-Control([long]$Window, [string]$Name, [string]$Role) {
    Wait-Control $Window $Name $Role
    [Xdzip.WindowsCapture.Native]::Activate($process.Id, $Window)
    [Xdzip.WindowsCapture.Driver]::Invoke($process.Id, $Window, $Name, $Role)
}
function Select-Tab([string]$Name) {
    Invoke-Control $mainWindow $Name 'TabItem'
    # This Qt capsule ID follows the real selected-tab model; the pinned Qt
    # provider does not reliably expose SelectionItem.IsSelected.
    Wait-State "selected tab $Name" { [Xdzip.WindowsCapture.Driver]::ReadyObject($process.Id, $mainWindow, $Name, 'activeArchiveTab') }
}
function Select-File([string]$Name) {
    Wait-Control $mainWindow $Name 'CheckBox'
    [Xdzip.WindowsCapture.Driver]::ClickNamed($process.Id, $mainWindow, $Name, 'Group')
    Wait-State "selected file $Name" { [Xdzip.WindowsCapture.Driver]::Selected($process.Id, $mainWindow, $Name, 'CheckBox') }
}
function Save-Scene([string]$Scene, [long]$Window, [string]$Title, [hashtable]$State) {
    $logical = if ($Window -eq $mainWindow) { [int[]]$layout.main.logical_window } else { @(0, 0) }
    $image = Save-WindowsCapture -Process $process -Window $Window -OutputDirectory $OutputDirectory -Name "$Language-$Scene" -ExpectedTitle $Title -LogicalSize $logical
    $image['scene'] = $Scene
    $image['state'] = $State
    $receipt.images += $image
    Save-Receipt
    Write-Host "Saved $($image.file), $($image.width) x $($image.height)." -ForegroundColor Green
}
function New-CreateRequest {
    $directory = Join-Path ([IO.Path]::GetTempPath()) ('xdzip-shell-' + [guid]::NewGuid().ToString('N'))
    New-PrivateCaptureDirectory $directory
    $script:requestFile = Join-Path $directory 'request.json'
    $creationDemo = Join-Path $OutputDirectory 'create-demo'
    New-PrivateCaptureDirectory $creationDemo
    foreach ($name in @('coastline.png', 'dusk.png', 'forest.png')) {
        [IO.File]::Copy((Join-Path $DemoDirectory $name), (Join-Path $creationDemo $name), $false)
    }
    $request = [ordered]@{ schema_version = 1; id = [guid]::NewGuid().ToString(); action = 'create'; archive_candidates = $false
        paths = @('coastline.png', 'dusk.png', 'forest.png') | ForEach-Object { Join-Path $creationDemo $_ } }
    $file = Join-Path $directory 'request.json'
    [IO.File]::WriteAllText($file, ($request | ConvertTo-Json -Depth 3), $utf8)
    return $file
}
$process = $null
$mainWindow = 0L
$creationWindow = 0L
$requestFile = $null
$pointer = $null
$keysRegistered = $false
try {
    [Xdzip.WindowsCapture.Native]::RegisterKeys()
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
        $mainWindow = [Xdzip.WindowsCapture.Native]::FindDefaultWindow($process.Id, $layout.main.logical_window[0], $layout.main.logical_window[1])
        Start-Sleep -Milliseconds 100
    }
    if (!$mainWindow) { throw 'Default-size candidate window not found. No screenshots captured.' }
    $pointer = [Xdzip.WindowsCapture.Native]::Pointer()
    Write-Host "Automatic capture: $Language. F10 cancels."
    [Xdzip.WindowsCapture.Native]::Activate($process.Id, $mainWindow)
    Select-Tab $browseName
    Invoke-Control $mainWindow $catalog.messages.'ui.d56220435194' 'Button'
    Select-File 'README.md'
    Wait-Control $mainWindow $catalog.messages.'preview.action' 'Button'
    Save-Scene '01-browse' $mainWindow '' @{ tab = $browseName; selected = 'README.md'; view = 'list' }

    Select-Tab $photoName
    Select-File 'coastline.png'
    Invoke-Control $mainWindow $catalog.messages.'preview.action' 'Button'
    Wait-Control $mainWindow $catalog.messages.'preview.again' 'Button'
    Wait-State 'loaded image preview' { [Xdzip.WindowsCapture.Driver]::ReadyObject($process.Id, $mainWindow, 'coastline.png', 'contentPreviewImage') }
    Save-Scene '03-preview' $mainWindow '' @{ tab = $photoName; selected = 'coastline.png'; preview = 'image' }

    Invoke-Control $mainWindow $catalog.messages.'ui.a35441db5cd1' 'Button'
    $gridSelection = $catalog.messages.'ui.98fb7e184544'.Replace('%1', 'coastline.png')
    Wait-State 'selected grid tile' { [Xdzip.WindowsCapture.Driver]::Selected($process.Id, $mainWindow, $gridSelection, 'CheckBox') }
    Wait-Control $mainWindow $browseName 'TabItem'
    Wait-Control $mainWindow $photoName 'TabItem'
    Save-Scene '04-workspace' $mainWindow '' @{ tabs = @($browseName, $photoName); view = 'grid'; selected = 'coastline.png' }

    $requestFile = New-CreateRequest
    $start.Arguments = '--shell-request "' + $requestFile + '"'
    $relay = [Diagnostics.Process]::Start($start)
    try {
        if (!$relay.WaitForExit(10000)) { $relay.Kill(); throw 'Creation request relay timed out.' }
        if ($relay.ExitCode -ne 0) { throw 'Candidate rejected the creation request.' }
    } finally { $relay.Dispose() }
    $creationTitle = $catalog.messages.'ui.fe189b616c1d'
    Wait-State 'creation window' { $script:creationWindow = [Xdzip.WindowsCapture.Native]::TitledWindow($process.Id, $creationTitle); $script:creationWindow -ne 0 }
    [Xdzip.WindowsCapture.Native]::Activate($process.Id, $creationWindow)
    foreach ($name in @('coastline.png', 'dusk.png', 'forest.png')) {
        $removeName = $catalog.messages.'ui.2ec7e443b1c7'.Replace('%1', $name)
        Wait-Control $creationWindow $removeName 'Button'
    }
    Wait-Control $creationWindow $creationTitle 'Button'
    Save-Scene '02-create' $creationWindow $creationTitle @{ inputs = @('coastline.png', 'dusk.png', 'forest.png'); submitted = $false }
    $null = Test-WindowsCaptureImages -Directory $OutputDirectory -Session $receipt
    $receipt.complete = $true
    Save-Receipt
    Write-Host "`nComplete: four client PNGs, four native-window references and capture-session.json.`n$OutputDirectory" -ForegroundColor Green
} catch {
    $receipt['complete'] = $false
    $receipt['failure'] = $_.Exception.GetBaseException().Message
    if ($process -and !$process.HasExited -and $mainWindow) {
        [IO.File]::WriteAllText((Join-Path $OutputDirectory 'owned-ui.txt'), [Xdzip.WindowsCapture.Driver]::Snapshot($process.Id, $mainWindow), $utf8)
        if ($creationWindow) { [IO.File]::WriteAllText((Join-Path $OutputDirectory 'creation-ui.txt'), [Xdzip.WindowsCapture.Driver]::Snapshot($process.Id, $creationWindow), $utf8) }
    }
    throw
} finally {
    if ($keysRegistered) { [Xdzip.WindowsCapture.Native]::ReleaseKeys() }
    try { Save-Receipt } finally {
        # Only the process created by this invocation may be closed. There are no
        # submitted archive operations in this workflow; never stop a process by name.
        if ($process -and !$process.HasExited) { $process.Kill(); $process.WaitForExit() }
        if ($process) { $process.Dispose() }
        if ($pointer) { [Xdzip.WindowsCapture.Native]::RestorePointer($pointer[0], $pointer[1]) }
        # The product normally consumes the request. On failure remove only our
        # exact private file and then its empty directory, never a recursive tree.
        if ($requestFile) {
            if ([IO.File]::Exists($requestFile)) { [IO.File]::Delete($requestFile) }
            $requestDirectory = [IO.Path]::GetDirectoryName($requestFile)
            if ([IO.Directory]::Exists($requestDirectory)) { [IO.Directory]::Delete($requestDirectory, $false) }
        }
    }
}
