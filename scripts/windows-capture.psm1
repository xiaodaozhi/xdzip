# Reusable capture mechanics. Recipes own application lifecycle and scene semantics.
Set-StrictMode -Version Latest

function Initialize-WindowsCapture {
    if ($env:OS -ne 'Windows_NT' -or $PSVersionTable.PSEdition -ne 'Desktop') {
        throw 'Windows capture requires Windows PowerShell 5.1.'
    }
    if (!('Xdzip.WindowsCapture.Native' -as [type])) {
        Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, WindowsBase
        Add-Type -Path (Join-Path $PSScriptRoot 'windows-capture.cs') -ReferencedAssemblies @(
            'System.Drawing', [System.Windows.Automation.AutomationElement].Assembly.Location,
            [System.Windows.Automation.ControlType].Assembly.Location, [System.Windows.Rect].Assembly.Location)
    }
}

function Get-WindowsCaptureHash([string]$Path) {
    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        $stream = [IO.File]::OpenRead($Path)
        try { [BitConverter]::ToString($algorithm.ComputeHash($stream)).Replace('-', '').ToLowerInvariant() }
        finally { $stream.Dispose() }
    } finally { $algorithm.Dispose() }
}

function Assert-WindowsCaptureRunning([Diagnostics.Process]$Process) {
    if ($Process.HasExited) { throw 'The capture-owned process has exited.' }
    if ([Xdzip.WindowsCapture.Native]::PollKey() -eq 2) { throw 'Capture canceled with F10. Earlier PNGs are retained.' }
}

function Wait-WindowsCaptureState {
    param([Diagnostics.Process]$Process, [string]$Label, [scriptblock]$Predicate,
        [ValidateRange(1, 60)][int]$TimeoutSeconds = 20)
    $watch = [Diagnostics.Stopwatch]::StartNew()
    while ($watch.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
        Assert-WindowsCaptureRunning $Process
        try { if (& $Predicate) { return } } catch [System.Windows.Automation.ElementNotAvailableException] {}
        Start-Sleep -Milliseconds 100
    }
    throw "Timed out waiting for scene state: $Label"
}

function New-PrivateCaptureDirectory([string]$Directory) {
    if ([IO.Directory]::Exists($Directory)) { throw 'Private capture directory already exists.' }
    $security = New-Object Security.AccessControl.DirectorySecurity
    $security.SetAccessRuleProtection($true, $false)
    $owner = [Security.Principal.WindowsIdentity]::GetCurrent().User
    $security.SetOwner($owner)
    foreach ($sid in @($owner, [Security.Principal.SecurityIdentifier]::new('S-1-5-18'))) {
        $security.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($sid, 'FullControl', 'ContainerInherit, ObjectInherit', 'None', 'Allow'))
    }
    # The protected ACL applies at creation, before any fixture payload is written.
    $null = [IO.Directory]::CreateDirectory($Directory, $security)
}

function Save-WindowsCapture {
    param([Parameter(Mandatory = $true)][Diagnostics.Process]$Process,
        [Parameter(Mandatory = $true)][long]$Window,
        [Parameter(Mandatory = $true)][string]$OutputDirectory,
        [Parameter(Mandatory = $true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_-]*$')][string]$Name,
        [string]$ExpectedTitle = '', [int[]]$LogicalSize = @(0, 0))
    if ($LogicalSize.Count -ne 2 -or $LogicalSize[0] -lt 0 -or $LogicalSize[1] -lt 0 -or
        (($LogicalSize[0] -eq 0) -ne ($LogicalSize[1] -eq 0))) { throw 'LogicalSize requires two positive dimensions or @(0, 0).' }
    [Xdzip.WindowsCapture.Native]::Activate($Process.Id, $Window)
    [Xdzip.WindowsCapture.Native]::FitPosition($Process.Id, $Window)
    [Xdzip.WindowsCapture.Native]::ParkPointer($Process.Id, $Window)
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $previous = ''; $unchanged = 0
    while ($watch.Elapsed.TotalSeconds -lt 10) {
        Assert-WindowsCaptureRunning $Process
        $current = [Xdzip.WindowsCapture.Native]::Fingerprint($Process.Id, $Window)
        if ($current -eq $previous) { $unchanged++ } else { $unchanged = 0 }
        if ($unchanged -ge 3 -and $watch.Elapsed.TotalMilliseconds -ge 600) { break }
        $previous = $current
        Start-Sleep -Milliseconds 180
    }
    if ($unchanged -lt 3) { throw "The scene did not finish rendering: $Name" }
    $image = [Xdzip.WindowsCapture.Native]::Capture($Process.Id, $Window, $ExpectedTitle,
        $LogicalSize[0], $LogicalSize[1], (Join-Path $OutputDirectory "$Name.png"),
        (Join-Path $OutputDirectory "$Name-window.png"))
    return [ordered]@{
        file = $image.ClientFile; sha256 = Get-WindowsCaptureHash (Join-Path $OutputDirectory $image.ClientFile)
        width = $image.ClientWidth; height = $image.ClientHeight; dpi = $image.Dpi
        window_file = $image.WindowFile
        window_sha256 = Get-WindowsCaptureHash (Join-Path $OutputDirectory $image.WindowFile)
        window_dimensions = @($image.WindowWidth, $image.WindowHeight)
        client_crop = @($image.ClientLeft, $image.ClientTop, $image.ClientWidth, $image.ClientHeight)
    }
}

function Test-WindowsCaptureImages {
    param([string]$Directory, [object]$Session)
    # Accept both an in-memory ordered receipt and a parsed JSON receipt.
    $session = ConvertTo-Json -InputObject $Session -Depth 12 | ConvertFrom-Json
    if ($session.images.Count -eq 0) { throw 'Capture session has no images.' }
    $files = @{}
    foreach ($image in $session.images) {
        foreach ($name in @($image.file, $image.window_file)) {
            if ([IO.Path]::GetFileName($name) -ne $name -or $name -notmatch '^[A-Za-z0-9][A-Za-z0-9_-]*\.png$' -or $files.ContainsKey($name)) {
                throw 'Invalid or repeated capture filename.'
            }
            $files[$name] = $true
        }
        $client = Join-Path $Directory $image.file
        $window = Join-Path $Directory $image.window_file
        if ((Get-WindowsCaptureHash $client) -ne $image.sha256 -or (Get-WindowsCaptureHash $window) -ne $image.window_sha256) {
            throw 'Capture file hash differs from the session receipt.'
        }
        $dimensions = [Xdzip.WindowsCapture.Native]::VerifyPair($client, $window, [int[]]$image.client_crop)
        if ($dimensions[0] -ne $image.width -or $dimensions[1] -ne $image.height -or
            $dimensions[2] -ne $image.window_dimensions[0] -or $dimensions[3] -ne $image.window_dimensions[1]) {
            throw 'Capture dimensions differ from the session receipt.'
        }
    }
    if ($session.PSObject.Properties.Name -contains 'expected_scenes') {
        $actual = @($session.images | ForEach-Object { $_.scene } | Sort-Object)
        $expected = @($session.expected_scenes | Sort-Object)
        if (($actual -join ',') -ne ($expected -join ',')) { throw 'Capture scene set differs from the recipe.' }
    }
    return $session.images.Count
}

function Test-WindowsCaptureSession([string]$Directory) {
    $session = Get-Content -LiteralPath (Join-Path $Directory 'capture-session.json') -Raw -Encoding UTF8 | ConvertFrom-Json
    if (!$session.complete) { throw 'Capture session is incomplete.' }
    return Test-WindowsCaptureImages -Directory $Directory -Session $session
}

Export-ModuleMember -Function Initialize-WindowsCapture, Get-WindowsCaptureHash,
    Assert-WindowsCaptureRunning, Wait-WindowsCaptureState, New-PrivateCaptureDirectory,
    Save-WindowsCapture, Test-WindowsCaptureImages, Test-WindowsCaptureSession
