# This script runs only as the disposable ordinary CI user.
param([Parameter(Mandatory=$true)][string]$Package,
      [Parameter(Mandatory=$true)][string]$Probe,
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
function Default-Snapshot {
    $snapshot = [ordered]@{}
    foreach ($extension in @('zip','7z','rar','tar','gz','tgz','cab','iso','udf')) {
        foreach ($suffix in @('UserChoice','UserChoiceLatest\ProgId')) {
            $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.$extension\$suffix")
            try { if ($key) { foreach ($value in ($key.GetValueNames() | Sort-Object)) { $snapshot["$extension/$suffix/$value"] = $key.GetValue($value) } } }
            finally { if ($key) { $key.Close() } }
        }
    }
    return ($snapshot | ConvertTo-Json -Compress)
}
$defaults = Default-Snapshot
try {
    if (Get-AppxPackage -Name $name) { throw 'Fixture already registered' }
    Add-AppxPackage -Path $Package
    $installed = Get-AppxPackage -Name $name
    if (@($installed).Count -ne 1 -or $installed.SignatureKind.ToString() -eq 'Store') { throw 'Wrong fixture identity or signature' }
    # Ask Windows to activate the registered package. Direct CreateProcess of a
    # WindowsApps executable may be denied under alternate CI credentials.
    $log = Join-Path ([IO.Path]::GetDirectoryName($Output)) "$Channel-native.log"
    & $Probe --activate-verify $installed.InstallLocation $installed.PackageFamilyName $Channel $log | Out-Host
    $exitCode = $LASTEXITCODE
    if (Test-Path -LiteralPath $log) { $result.native_log = [string](Get-Content -LiteralPath $log -Raw); Write-Output $result.native_log }
    if ($exitCode -ne 0) { throw 'Native package menu probe failed' }
    Remove-AppxPackage -Package $installed.PackageFullName
    if (Get-AppxPackage -Name $name) { throw 'Fixture uninstall failed' }
    & $Probe --reject-unregistered $installed.InstallLocation | Out-Host
    if ($LASTEXITCODE -ne 0) { throw 'Uninstalled package remains a menu target' }
    if ((Default-Snapshot) -ne $defaults) { throw 'Fixture changed the ordinary user default applications' }
    $result.defaults_unchanged = $true
    $result.status = 'passed'
} finally {
    if ($installed -and (Get-AppxPackage -Name $name)) { Remove-AppxPackage -Package $installed.PackageFullName }
    $result | ConvertTo-Json | Set-Content -LiteralPath $Output -Encoding utf8
}
