# The comparison installer and MSIX are installed only on a disposable runner.
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted' -or $env:RUNNER_OS -ne 'Windows') {
    throw 'Coexistence testing is restricted to a disposable GitHub-hosted Windows runner'
}
$folder = (Resolve-Path 'build/msix-ci-package').Path
$setup = Join-Path $folder 'owned-setup.exe'
$report = Get-Content (Join-Path $folder 'owned-setup.json') -Raw | ConvertFrom-Json
if ((Get-FileHash -LiteralPath $setup -Algorithm SHA256).Hash.ToLowerInvariant() -ne $report.sha256) { throw 'Comparison installer differs from its receipt' }
$installRoot = Join-Path $env:RUNNER_TEMP 'xdzip-msix-coexistence-installed'
if ((Test-Path -LiteralPath $installRoot) -or (Test-Path 'HKCU:\Software\xdzip\Install')) { throw 'Refusing to replace an existing installation' }
if (Get-AppxPackage -Name 'xdzip.MsixCI') { throw 'Refusing to replace an existing test package' }
$defaults = @{}
function Get-Default([string]$extension) {
    $key = [Microsoft.Win32.Registry]::ClassesRoot.OpenSubKey('.' + $extension)
    try { $value = if ($key) { $key.GetValue('') } else { $null } }
    finally { if ($key) { $key.Close() } }
    $choice = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.' + $extension + '\UserChoice')
    try {
        $progId = if ($choice) { $choice.GetValue('ProgId') } else { $null }
        $hash = if ($choice) { $choice.GetValue('Hash') } else { $null }
    } finally { if ($choice) { $choice.Close() } }
    return ([ordered]@{ default=$value; progId=$progId; hash=$hash } | ConvertTo-Json -Compress)
}
foreach ($extension in @('zip','tar','tgz','gz','7z','rar','cab','iso','udf')) { $defaults[$extension] = Get-Default $extension }
function Assert-Defaults {
    foreach ($extension in $defaults.Keys) {
        if ((Get-Default $extension) -ne $defaults[$extension]) { throw "Default file association changed: $extension" }
    }
}
$result = [ordered]@{ status='failed'; checks=[ordered]@{}; limitations=@('Hosted Windows Server with self-signed MSIX; Windows 11 and Store remain separate.') }
$failure = $null
$exe = Join-Path $installRoot 'xdzip.exe'
$uninstaller = Join-Path $installRoot 'unins000.exe'
try {
    $args = @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/CURRENTUSER',('/DIR="' + $installRoot + '"'))
    $process = Start-Process -FilePath $setup -ArgumentList $args -PassThru
    if (!$process.WaitForExit(120000)) { $process.Kill(); throw 'Comparison installer timed out' }
    if ($process.ExitCode -ne 0 -or !(Test-Path -LiteralPath $exe) -or !(Test-Path -LiteralPath $uninstaller)) { throw 'Comparison installer did not complete' }
    if ((Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant() -ne $report.executable_sha256) { throw 'Installed executable differs from the audited build' }
    Assert-Defaults
    $result.checks.installer_and_defaults = 'passed'
    & "$PSScriptRoot/check-windows-msix-install.ps1" -Package (Join-Path $folder 'xdzip-windows-x64.msix') -Output (Join-Path $folder 'lifecycle.json') -InstalledExecutable $exe
    $lifecycle = Get-Content (Join-Path $folder 'lifecycle.json') -Raw | ConvertFrom-Json
    if ($lifecycle.status -ne 'passed' -or $lifecycle.checks.installed_edition_coexistence -ne 'passed') { throw 'Installed and MSIX coexistence did not pass' }
    if (!(Test-Path -LiteralPath $exe) -or !(Test-Path -LiteralPath $uninstaller)) { throw 'MSIX uninstall changed the comparison installation' }
    $env:QT_QPA_PLATFORM = 'offscreen'
    $smoke = Start-Process -FilePath $exe -ArgumentList '--smoke-test' -PassThru
    if (!$smoke.WaitForExit(60000)) { $smoke.Kill(); throw 'Comparison installation smoke test timed out' }
    if ($smoke.ExitCode -ne 0) { throw 'Comparison installation no longer starts' }
    Assert-Defaults
    $result.checks.msix_uninstall_preserves_installer = 'passed'
} catch { $failure = $_ }
finally {
    try {
        if (Test-Path -LiteralPath $uninstaller) {
            $remove = Start-Process -FilePath $uninstaller -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART') -PassThru
            if (!$remove.WaitForExit(120000)) { $remove.Kill(); throw 'Comparison uninstaller timed out' }
            if ($remove.ExitCode -ne 0 -or (Test-Path -LiteralPath $exe)) { throw 'Comparison uninstall did not complete' }
            Assert-Defaults
            $result.checks.installer_uninstall_and_defaults = 'passed'
        } elseif (Test-Path -LiteralPath $exe) { throw 'Comparison installation has no uninstaller' }
    } catch { if (!$failure) { $failure = $_ } }
    if (!$failure) { $result.status = 'passed' }
    $result | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $folder 'coexistence.json') -Encoding utf8
}
if ($failure) { throw $failure }
