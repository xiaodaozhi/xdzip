#requires -Version 5.1
# Codec, provenance and fixture regression checks; no desktop capture or UI input.
[CmdletBinding()]
param([string]$OutputDirectory)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'windows-capture.psm1')
Import-Module (Join-Path $PSScriptRoot 'windows-store-demo.psm1')
Initialize-WindowsCapture
if (!$OutputDirectory) { $OutputDirectory = Join-Path (Split-Path -Parent $PSScriptRoot) ('build/capture-tools-self-test/' + [guid]::NewGuid().ToString('N')) }
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Self-test output directory already exists.' }
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
$null = New-Item -ItemType Directory -Path $OutputDirectory
[Xdzip.WindowsCapture.Native]::CodecSelfTest((Join-Path $OutputDirectory 'rgba-roundtrip.png'))
[Xdzip.WindowsCapture.Native]::PairSelfTest($OutputDirectory)
function Expect-Rejection([scriptblock]$Action, [string]$Label) {
    $rejected = $false
    try { & $Action } catch { $rejected = $true }
    if (!$rejected) { throw "Expected rejection: $Label" }
}
function Check-PredicateScope {
    $localToken = 'ready'
    Wait-WindowsCaptureState -Process ([Diagnostics.Process]::GetCurrentProcess()) -Label 'predicate scope' -TimeoutSeconds 1 -Predicate { $localToken -eq 'ready' }
}
Check-PredicateScope
Expect-Rejection { Wait-WindowsCaptureState -Process ([Diagnostics.Process]::GetCurrentProcess()) -Label 'timeout' -TimeoutSeconds 1 -Predicate { $false } } 'bounded scene timeout'

$receipt = [ordered]@{
    schema_version = 3; complete = $true; expected_scenes = @('test')
    images = @([ordered]@{
        scene = 'test'; file = 'pair-client.png'; window_file = 'pair-window.png'
        sha256 = Get-WindowsCaptureHash (Join-Path $OutputDirectory 'pair-client.png')
        window_sha256 = Get-WindowsCaptureHash (Join-Path $OutputDirectory 'pair-window.png')
        width = 11; height = 9; window_dimensions = @(23, 19); client_crop = @(3, 4, 11, 9)
    })
}
$receiptFile = Join-Path $OutputDirectory 'capture-session.json'
function Write-Receipt { [IO.File]::WriteAllText($receiptFile, ($receipt | ConvertTo-Json -Depth 6)) }
Write-Receipt
if ((Test-WindowsCaptureSession $OutputDirectory) -ne 1) { throw 'Valid capture session rejected.' }
$receipt.complete = $false; Write-Receipt
if ((Test-WindowsCaptureImages -Directory $OutputDirectory -Session $receipt) -ne 1) { throw 'Pending image verification failed.' }
Expect-Rejection { Test-WindowsCaptureSession $OutputDirectory } 'partial session'
$receipt.complete = $true; $receipt.images[0].sha256 = '0' * 64; Write-Receipt
Expect-Rejection { Test-WindowsCaptureSession $OutputDirectory } 'file hash mismatch'
$receipt.images[0].file = '../pair-client.png'; Write-Receipt
Expect-Rejection { Test-WindowsCaptureSession $OutputDirectory } 'receipt path traversal'
$receipt.images[0].file = 'pair-client.png'
$receipt.images[0].sha256 = Get-WindowsCaptureHash (Join-Path $OutputDirectory 'pair-client.png')
$receipt.expected_scenes = @('missing'); Write-Receipt
Expect-Rejection { Test-WindowsCaptureSession $OutputDirectory } 'missing scene'
$receipt.expected_scenes = @('test'); Write-Receipt

Add-Type -AssemblyName System.IO.Compression
foreach ($language in @('en-US', 'zh-CN')) {
    $directory = Join-Path $OutputDirectory $language
    $null = New-WindowsStoreDemo -Directory $directory -Language $language
    $archiveNames = @(Get-ChildItem -LiteralPath $directory -Filter '*.zip')
    if ($archiveNames.Count -ne 2) { throw 'Demo archive count differs.' }
    foreach ($file in $archiveNames) {
        $stream = [IO.File]::OpenRead($file.FullName)
        $archive = [IO.Compression.ZipArchive]::new($stream, [IO.Compression.ZipArchiveMode]::Read)
        try {
            $names = @($archive.Entries | ForEach-Object { $_.FullName })
            if ($names.Count -notin @(3, 8) -or @($names | Where-Object { $_ -match '(^/|\.\.)' }).Count) { throw 'Invalid demo entry set.' }
            if ($names.Count -eq 8 -and $names -notcontains 'README.md') { throw 'Browse scene fixture is missing.' }
            if ($names.Count -eq 3 -and $names -notcontains 'coastline.png') { throw 'Preview fixture is missing.' }
        } finally { $archive.Dispose(); $stream.Dispose() }
    }
    Expect-Rejection { New-WindowsStoreDemo -Directory $directory -Language $language } 'fixture overwrite'
    # Compare another independently generated tree, including exact ZIP bytes.
    $repeat = Join-Path $OutputDirectory ($language + '-repeat')
    $null = New-WindowsStoreDemo -Directory $repeat -Language $language
    foreach ($file in (Get-ChildItem -LiteralPath $directory -File)) {
        if ((Get-WindowsCaptureHash $file.FullName) -ne (Get-WindowsCaptureHash (Join-Path $repeat $file.Name))) { throw 'Demo inputs are not reproducible.' }
    }
}
Write-Host 'PASS: PNG/RGBA, exact crop pixels, JPEG rejection, dimensions, exclusive writes, receipt hashes/scenes/paths, bounded waits, bilingual reproducible fixtures. No desktop capture.'
