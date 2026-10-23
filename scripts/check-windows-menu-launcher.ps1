# scripts/check-windows-menu-launcher.ps1

[CmdletBinding()]
param(
    [ValidateSet('SelfTest', 'Launch')][string]$Mode = 'SelfTest',
    [Parameter(Mandatory = $true)][string]$Tests,
    [Parameter(Mandatory = $true)][string]$OutputDirectory,
    [string]$SourceRevision,
    [string]$ExpectedSha256,
    [switch]$CoreOnly
)
$ErrorActionPreference = 'Stop'
if (![Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([Runtime.InteropServices.OSPlatform]::Windows)) {
    throw 'The menu launcher is a Windows-only QA tool.'
}
if ($Mode -eq 'Launch' -and $CoreOnly) { throw 'CoreOnly is a self-test option.' }
$taskRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskTests = [IO.Path]::GetFullPath($Tests)
$taskOutput = [IO.Path]::GetFullPath($OutputDirectory).TrimEnd([IO.Path]::DirectorySeparatorChar)
$taskBuildPrefix = [IO.Path]::GetFullPath((Join-Path $taskRoot 'build')) + [IO.Path]::DirectorySeparatorChar
if (!$taskOutput.StartsWith($taskBuildPrefix, [StringComparison]::OrdinalIgnoreCase) -or (Test-Path -LiteralPath $taskOutput)) {
    throw 'Choose a fresh evidence directory under build/.'
}
if (!$taskTests.StartsWith($taskBuildPrefix, [StringComparison]::OrdinalIgnoreCase) -or !(Test-Path -LiteralPath $taskTests -PathType Leaf)) {
    throw 'Use the owned browser-test executable under build/.'
}
if (!$SourceRevision) {
    $SourceRevision = (& git -C $taskRoot rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read the source revision.' }
}
if (!$ExpectedSha256) { $ExpectedSha256 = (Get-FileHash -LiteralPath $taskTests -Algorithm SHA256).Hash.ToLowerInvariant() }
if ($SourceRevision -notmatch '^[a-f0-9]{40}$' -or $ExpectedSha256 -notmatch '^[a-f0-9]{64}$') {
    throw 'Use a full source SHA and executable SHA-256.'
}
$taskCompiler = Join-Path $env:WINDIR 'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
if (!(Test-Path -LiteralPath $taskCompiler -PathType Leaf)) { throw 'The installed .NET Framework C# compiler is unavailable.' }
$taskSources = @(
    (Join-Path $taskRoot 'tests/fixtures/native/windows-menu-qa-launcher.cs'),
    (Join-Path $taskRoot 'tests/fixtures/native/windows-menu-qa-early-exit.cs')
)
$taskTools = Join-Path $taskRoot ('build/native-menu-launcher-tools/' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $taskTools | Out-Null
$taskLauncher = Join-Path $taskTools 'xdzip-menu-qa.exe'
$taskFixture = Join-Path $taskTools 'xdzip-menu-qa-selftest.exe'
$taskReferences = @('/r:System.dll', '/r:System.Core.dll', '/r:System.Windows.Forms.dll', '/r:System.Drawing.dll', '/r:System.Web.Extensions.dll')
& $taskCompiler /nologo /target:winexe /main:XdzipOwnedMenuQa.MenuQaEntry "/out:$taskLauncher" @taskReferences $taskSources[0]
if ($LASTEXITCODE -ne 0) { throw 'Menu launcher compilation failed.' }
& $taskCompiler /nologo /target:exe /main:XdzipOwnedMenuQa.HelperEarlyExitFixture "/out:$taskFixture" @taskReferences @taskSources
if ($LASTEXITCODE -ne 0) { throw 'Menu launcher regression compilation failed.' }
$taskArguments = @('--root', $taskRoot, '--tests', $taskTests, '--source', $SourceRevision, '--hash', $ExpectedSha256, '--output', $taskOutput)
$taskProcessId = $null
if ($Mode -eq 'SelfTest') {
    if ($CoreOnly) { $taskArguments += '--core-only' }
    & $taskFixture @taskArguments
    if ($LASTEXITCODE -ne 0) { throw 'Owned menu-launcher regression failed; inspect its evidence.' }
} else {
    # This visible window is the explicitly requested user-click entry. It does
    # not start a test child until the real foreground mouse click arrives.
    $taskQuotedArguments = ($taskArguments | ForEach-Object { '"' + $_ + '"' }) -join ' '
    $taskProcess = Start-Process -FilePath $taskLauncher -ArgumentList $taskQuotedArguments -WindowStyle Normal -PassThru
    $taskProcessId = $taskProcess.Id
}
New-Item -ItemType Directory -Path $taskOutput -Force | Out-Null
$taskSourceHashes = [ordered]@{}
foreach ($taskSource in $taskSources) {
    $taskSourceHashes[[IO.Path]::GetRelativePath($taskRoot, $taskSource).Replace('\', '/')] = (Get-FileHash -LiteralPath $taskSource -Algorithm SHA256).Hash.ToLowerInvariant()
}
$taskRecord = [ordered]@{
    schema_version = 1
    mode = $Mode
    core_only = [bool]$CoreOnly
    source_revision = $SourceRevision
    tests_sha256 = $ExpectedSha256
    launcher_sha256 = (Get-FileHash -LiteralPath $taskLauncher -Algorithm SHA256).Hash.ToLowerInvariant()
    fixture_sha256 = (Get-FileHash -LiteralPath $taskFixture -Algorithm SHA256).Hash.ToLowerInvariant()
    source_files = $taskSourceHashes
    launcher_process_id = $taskProcessId
    automatic_menu_sampling = $false
}
$taskRecord | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $taskOutput 'launcher-build.json') -Encoding utf8
if ($Mode -eq 'Launch') {
    [ordered]@{ status = 'launched_user_click_entry'; process_id = $taskProcessId; source_revision = $SourceRevision; automatic_sampling = $false } | ConvertTo-Json -Compress
}
