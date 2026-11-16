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
    Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, WindowsBase
    Add-Type -Path (Join-Path $PSScriptRoot 'windows-store-capture.cs') -ReferencedAssemblies @(
        'System.Drawing', [System.Windows.Automation.AutomationElement].Assembly.Location,
        [System.Windows.Automation.ControlType].Assembly.Location, [System.Windows.Rect].Assembly.Location)
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

$receipt = [ordered]@{
    schema_version = 2; complete = $false; language = $Language
    baseline_revision = $spec.baseline.revision; runtime_exe_sha256 = $exeHash
    method = 'Native screen pixels, DWM visible frame and exact client crop, lossless PNG, no resampling'
    scene_preparation = 'Automatic owned UI Automation and create shell request'; images = @()
}
function Save-Receipt {
    [IO.File]::WriteAllText((Join-Path $OutputDirectory 'capture-session.json'), ($receipt | ConvertTo-Json -Depth 8), $utf8)
}
function Assert-Running {
    if ($process.HasExited) { throw 'The capture-owned xdzip process has exited.' }
    if ([Xdzip.StoreCapture.Native]::PollKey() -eq 2) { throw 'Capture canceled with F10. Earlier PNGs are retained.' }
}
function Wait-State([string]$Label, [scriptblock]$Predicate) {
    $watch = [Diagnostics.Stopwatch]::StartNew()
    while ($watch.Elapsed.TotalSeconds -lt 20) {
        Assert-Running
        try { if (& $Predicate) { return } } catch [System.Windows.Automation.ElementNotAvailableException] {}
        Start-Sleep -Milliseconds 100
    }
    throw "Timed out waiting for scene state: $Label"
}
function Wait-Control([long]$Window, [string]$Name, [string]$Role) {
    Wait-State "$Name ($Role)" { [Xdzip.StoreCapture.Driver]::Ready($process.Id, $Window, $Name, $Role) }
}
function Invoke-Control([long]$Window, [string]$Name, [string]$Role) {
    Wait-Control $Window $Name $Role
    [Xdzip.StoreCapture.Native]::Activate($process.Id, $Window)
    [Xdzip.StoreCapture.Driver]::Invoke($process.Id, $Window, $Name, $Role)
}
function Select-Tab([string]$Name) {
    Invoke-Control $mainWindow $Name 'TabItem'
    Wait-State "selected tab $Name" { [Xdzip.StoreCapture.Driver]::Selected($process.Id, $mainWindow, $Name, 'TabItem') }
}
function Select-File([string]$Name) {
    Wait-Control $mainWindow $Name 'CheckBox'
    [Xdzip.StoreCapture.Driver]::SelectFile($process.Id, $mainWindow, $Name)
    Wait-State "selected file $Name" { [Xdzip.StoreCapture.Driver]::Selected($process.Id, $mainWindow, $Name, 'CheckBox') }
}
function Save-Scene([string]$Scene, [long]$Window, [string]$Title, [hashtable]$State) {
    [Xdzip.StoreCapture.Native]::Activate($process.Id, $Window)
    [Xdzip.StoreCapture.Native]::FitPosition($process.Id, $Window)
    [Xdzip.StoreCapture.Native]::ParkPointer($process.Id, $Window)
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $previous = ''; $unchanged = 0
    while ($watch.Elapsed.TotalSeconds -lt 10) {
        Assert-Running
        $current = [Xdzip.StoreCapture.Native]::Fingerprint($process.Id, $Window)
        if ($current -eq $previous) { $unchanged++ } else { $unchanged = 0 }
        if ($unchanged -ge 3 -and $watch.Elapsed.TotalMilliseconds -ge 600) { break }
        $previous = $current
        Start-Sleep -Milliseconds 180
    }
    if ($unchanged -lt 3) { throw "The scene did not finish rendering: $Scene" }
    $image = [Xdzip.StoreCapture.Native]::Capture($process.Id, $mainWindow, $Title,
        $layout.main.logical_window[0], $layout.main.logical_window[1],
        (Join-Path $OutputDirectory "$Language-$Scene.png"),
        (Join-Path $OutputDirectory "$Language-$Scene-window.png"))
    $receipt.images += [ordered]@{
        scene = $Scene; file = $image.ClientFile; state = $State
        sha256 = Get-CaptureSha256 (Join-Path $OutputDirectory $image.ClientFile)
        width = $image.ClientWidth; height = $image.ClientHeight; dpi = $image.Dpi
        window_file = $image.WindowFile
        window_sha256 = Get-CaptureSha256 (Join-Path $OutputDirectory $image.WindowFile)
        window_dimensions = @($image.WindowWidth, $image.WindowHeight)
        client_crop = @($image.ClientLeft, $image.ClientTop, $image.ClientWidth, $image.ClientHeight)
    }
    Save-Receipt
    Write-Host "Saved $($image.ClientFile), $($image.ClientWidth) x $($image.ClientHeight)." -ForegroundColor Green
}
function New-PrivateDirectory([string]$Directory) {
    if ([IO.Directory]::Exists($Directory)) { throw 'Private capture directory already exists.' }
    $security = New-Object Security.AccessControl.DirectorySecurity
    $security.SetAccessRuleProtection($true, $false)
    $owner = [Security.Principal.WindowsIdentity]::GetCurrent().User
    $security.SetOwner($owner)
    foreach ($sid in @($owner, [Security.Principal.SecurityIdentifier]::new('S-1-5-18'))) {
        $security.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($sid, 'FullControl', 'ContainerInherit, ObjectInherit', 'None', 'Allow'))
    }
    # Creation applies the protected owner/System ACL before writing paths.
    $null = [IO.Directory]::CreateDirectory($Directory, $security)
}
function New-CreateRequest {
    $directory = Join-Path ([IO.Path]::GetTempPath()) ('xdzip-shell-' + [guid]::NewGuid().ToString('N'))
    New-PrivateDirectory $directory
    $script:requestFile = Join-Path $directory 'request.json'
    $creationDemo = Join-Path $OutputDirectory 'create-demo'
    New-PrivateDirectory $creationDemo
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
    $pointer = [Xdzip.StoreCapture.Native]::Pointer()
    Write-Host "Automatic capture: $Language. F10 cancels."
    [Xdzip.StoreCapture.Native]::Activate($process.Id, $mainWindow)
    Select-Tab $browseName
    Invoke-Control $mainWindow $catalog.messages.'ui.d56220435194' 'Button'
    Select-File 'README.md'
    Wait-Control $mainWindow $catalog.messages.'preview.action' 'Button'
    Save-Scene '01-browse' $mainWindow '' @{ tab = $browseName; selected = 'README.md'; view = 'list' }

    Select-Tab $photoName
    Select-File 'coastline.png'
    Invoke-Control $mainWindow $catalog.messages.'preview.action' 'Button'
    Wait-Control $mainWindow $catalog.messages.'preview.again' 'Button'
    Wait-State 'loaded image preview' { [Xdzip.StoreCapture.Driver]::ReadyObject($process.Id, $mainWindow, 'coastline.png', 'contentPreviewImage') }
    Save-Scene '03-preview' $mainWindow '' @{ tab = $photoName; selected = 'coastline.png'; preview = 'image' }

    Invoke-Control $mainWindow $catalog.messages.'ui.a35441db5cd1' 'Button'
    $gridSelection = $catalog.messages.'ui.98fb7e184544'.Replace('%1', 'coastline.png')
    Wait-State 'selected grid tile' { [Xdzip.StoreCapture.Driver]::Selected($process.Id, $mainWindow, $gridSelection, 'CheckBox') }
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
    Wait-State 'creation window' { $script:creationWindow = [Xdzip.StoreCapture.Native]::TitledWindow($process.Id, $creationTitle); $script:creationWindow -ne 0 }
    [Xdzip.StoreCapture.Native]::Activate($process.Id, $creationWindow)
    foreach ($name in @('coastline.png', 'dusk.png', 'forest.png')) {
        $removeName = $catalog.messages.'ui.2ec7e443b1c7'.Replace('%1', $name)
        Wait-Control $creationWindow $removeName 'Button'
    }
    Wait-Control $creationWindow $creationTitle 'Button'
    Save-Scene '02-create' $creationWindow $creationTitle @{ inputs = @('coastline.png', 'dusk.png', 'forest.png'); submitted = $false }
    $receipt.complete = $true
    Save-Receipt
    Write-Host "`nComplete: four client PNGs, four native-window references and capture-session.json.`n$OutputDirectory" -ForegroundColor Green
} catch {
    $receipt['failure'] = $_.Exception.GetBaseException().Message
    if ($process -and !$process.HasExited -and $mainWindow) {
        [IO.File]::WriteAllText((Join-Path $OutputDirectory 'owned-ui.txt'), [Xdzip.StoreCapture.Driver]::Snapshot($process.Id, $mainWindow), $utf8)
        if ($creationWindow) { [IO.File]::WriteAllText((Join-Path $OutputDirectory 'creation-ui.txt'), [Xdzip.StoreCapture.Driver]::Snapshot($process.Id, $creationWindow), $utf8) }
    }
    throw
} finally {
    if ($keysRegistered) { [Xdzip.StoreCapture.Native]::ReleaseKeys() }
    try { Save-Receipt } finally {
        # Only the process created by this invocation may be closed. There are no
        # submitted archive operations in this workflow; never stop a process by name.
        if ($process -and !$process.HasExited) { $process.Kill(); $process.WaitForExit() }
        if ($process) { $process.Dispose() }
        if ($pointer) { [Xdzip.StoreCapture.Native]::RestorePointer($pointer[0], $pointer[1]) }
        # The product normally consumes the request. On failure remove only our
        # exact private file and then its empty directory, never a recursive tree.
        if ($requestFile) {
            if ([IO.File]::Exists($requestFile)) { [IO.File]::Delete($requestFile) }
            $requestDirectory = [IO.Path]::GetDirectoryName($requestFile)
            if ([IO.Directory]::Exists($requestDirectory)) { [IO.Directory]::Delete($requestDirectory, $false) }
        }
    }
}
