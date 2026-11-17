# This script runs only as the disposable ordinary CI user.
param([Parameter(Mandatory=$true)][string]$Package, [Parameter(Mandatory=$true)][string]$Probe,
      [Parameter(Mandatory=$true)][string]$Channel, [Parameter(Mandatory=$true)][string]$Output)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') { throw 'Disposable hosted CI only' }
$principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if ($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'A real ordinary user is required' }
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$profile = Get-ItemProperty -LiteralPath ('HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList\' + $identity.User.Value)
$env:USERPROFILE = [Environment]::ExpandEnvironmentVariables($profile.ProfileImagePath)
$env:LOCALAPPDATA = Join-Path $env:USERPROFILE 'AppData\Local'
$env:APPDATA = Join-Path $env:USERPROFILE 'AppData\Roaming'
$env:TEMP = Join-Path $env:LOCALAPPDATA 'Temp'
$env:TMP = $env:TEMP
$null = New-Item -ItemType Directory -Path $env:TEMP -Force
if ($Channel -notin @('store','sideload')) { throw 'Invalid fixture channel' }
$name = "xdzip.ShellCI.$Channel"
$installed = $null
$result = [ordered]@{status='failed'; channel=$Channel; ordinary_user=$true}
try {
    if (Get-AppxPackage -Name $name) { throw 'Fixture already registered' }
    Add-AppxPackage -Path $Package
    $installed = Get-AppxPackage -Name $name
    if (@($installed).Count -ne 1 -or $installed.SignatureKind.ToString() -eq 'Store') { throw 'Wrong fixture identity or signature' }
    & $Probe --verify $installed.InstallLocation $installed.PackageFamilyName $Channel
    if ($LASTEXITCODE -ne 0) { throw 'Native package menu probe failed' }
    Remove-AppxPackage -Package $installed.PackageFullName
    if (Get-AppxPackage -Name $name) { throw 'Fixture uninstall failed' }
    $result.status = 'passed'
} finally {
    if ($installed -and (Get-AppxPackage -Name $name)) { Remove-AppxPackage -Package $installed.PackageFullName }
    $result | ConvertTo-Json | Set-Content -LiteralPath $Output -Encoding utf8
}
