# Runs only on the disposable native Windows CI host, never on the developer Mac.
param([string]$Packages, [string]$Fixture, [string]$Output)
$ErrorActionPreference = 'Stop'
$classes = 'HKCU:\Software\Classes'
if (Test-Path "$classes\xdzip.Archive") { throw 'Refusing to replace an existing xdzip registration in this verification host' }
$testRoot = Join-Path $env:RUNNER_TEMP 'xdzip 安装, space'
$evidenceRoot = Join-Path $env:RUNNER_TEMP 'xdzip-install-evidence'
New-Item -ItemType Directory -Force $evidenceRoot | Out-Null
$settings = Join-Path $evidenceRoot 'settings.ini'
Set-Content $settings "[General]`nschemaVersion=1`n[appearance]`ntheme=dark`n[privacy]`nrecentEnabled=false" -Encoding utf8
$env:XDZIP_SETTINGS_PATH = $settings
$archive = Join-Path $evidenceRoot '中文 space & --smoke-test.zip'
Copy-Item $Fixture $archive
function Get-DefaultSnapshot {
    $key = [Microsoft.Win32.Registry]::ClassesRoot.OpenSubKey('.zip')
    $default = if ($key) { $key.GetValue(''); $key.Close() } else { $null }
    $choice = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.zip\UserChoice')
    $progId = if ($choice) { $choice.GetValue('ProgId') } else { $null }
    $hash = if ($choice) { $choice.GetValue('Hash'); $choice.Close() } else { $null }
    return ([ordered]@{ default=$default; progId=$progId; hash=$hash } | ConvertTo-Json -Compress)
}
$defaultBefore = Get-DefaultSnapshot
$foreignKey = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Software\Classes\.zip\OpenWithProgids')
$foreignKey.SetValue('xdzip.QAForeign', [byte[]]@(), [Microsoft.Win32.RegistryValueKind]::None)
$foreignKey.Close()
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class XdzipShell {
 [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)]
 public struct ExecuteInfo {
  public uint cbSize, fMask; public IntPtr hwnd;
  public string lpVerb, lpFile, lpParameters, lpDirectory; public int nShow;
  public IntPtr hInstApp, lpIDList; public string lpClass; public IntPtr hkeyClass;
  public uint dwHotKey; public IntPtr hIcon, hProcess;
 }
 [DllImport("shell32.dll",CharSet=CharSet.Unicode,SetLastError=true)]
 public static extern bool ShellExecuteExW(ref ExecuteInfo info);
 [DllImport("kernel32.dll")] public static extern uint GetProcessId(IntPtr process);
 [DllImport("kernel32.dll")] public static extern uint WaitForSingleObject(IntPtr process,uint timeout);
 [DllImport("kernel32.dll")] public static extern bool CloseHandle(IntPtr handle);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr OpenMutexW(uint access,bool inherit,string name);
}
'@
function Invoke-Installer($File, $Label, $ExpectFailure=$false) {
    $arguments = @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART',('/DIR="'+$testRoot+'"'),('/LOG="'+(Join-Path $evidenceRoot ($Label+'.log'))+'"'))
    $process = Start-Process $File -ArgumentList $arguments -PassThru
    if (!$process.WaitForExit(120000)) { $process.Kill(); throw 'QA installer timed out' }
    if ($ExpectFailure) {
        if ($process.ExitCode -eq 0) { throw 'Installer replaced an active application' }
    } elseif ($process.ExitCode -ne 0) { throw "Installer phase failed: $Label" }
}
function Assert-Default {
    $current = Get-DefaultSnapshot
    if ($current -ne $defaultBefore) { throw 'Foreign default association changed' }
}
$phases = [Collections.Generic.List[string]]::new()
function Test-AssociatedOpen($Label) {
    foreach ($extension in @('zip','tar','tgz','gz')) {
        $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Software\Classes\.$extension\OpenWithProgids")
        if (!$key -or $key.GetValueNames() -notcontains 'xdzip.Archive') { throw 'OpenWith candidate value missing' }
        $key.Close()
    }
    $probe = Join-Path $evidenceRoot ($Label+'.json')
    $env:XDZIP_ASSOCIATION_PROBE = $probe
    $info = [XdzipShell+ExecuteInfo]::new()
    $info.cbSize = [Runtime.InteropServices.Marshal]::SizeOf($info)
    $info.fMask = 0x541 # CLASSNAME, process handle, synchronous shell dispatch, no error UI
    $info.lpVerb = 'open'; $info.lpFile = $archive; $info.lpClass = 'xdzip.Archive'; $info.nShow = 1
    if (![XdzipShell]::ShellExecuteExW([ref]$info)) { throw 'Native ShellExecuteEx association launch failed' }
    try {
        if ([XdzipShell]::WaitForSingleObject($info.hProcess,25000) -ne 0) { throw 'Associated product did not shut down' }
    } finally { [void][XdzipShell]::CloseHandle($info.hProcess) }
    $result = Get-Content $probe -Raw | ConvertFrom-Json
    if ($result.event -ne 'association_open' -or $result.records -ne 100000) { throw 'Associated product did not read actual archive' }
    Assert-Default
    $phases.Add($Label)
    Remove-Item Env:XDZIP_ASSOCIATION_PROBE
}
try {
    Invoke-Installer (Join-Path $Packages 'qa1.exe') 'install'; Test-AssociatedOpen 'install'
    $sentinel = Join-Path $testRoot 'user-kept.txt'; Set-Content $sentinel 'keep user data'
    Invoke-Installer (Join-Path $Packages 'qa2.exe') 'upgrade'; Test-AssociatedOpen 'upgrade'
    $command = (Get-Item "$classes\xdzip.Archive\shell\open\command").GetValue('')
    $executable = [regex]::Match($command, '^"([^"]+)" -- "%1"$').Groups[1].Value
    if (!(Test-Path $executable)) { throw 'Association command quoting invalid' }
    $active = Start-Process $executable -PassThru
    try {
        $seen = $false
        for ($i=0; $i -lt 100; $i++) {
            $handle = [XdzipShell]::OpenMutexW(0x100000,$false,'Local\xdzip.desktop.running')
            if ($handle -ne [IntPtr]::Zero) {
                [void][XdzipShell]::CloseHandle($handle); $active.Refresh()
                if ($active.MainWindowHandle -ne [IntPtr]::Zero) { $seen=$true; break }
            }
            Start-Sleep -Milliseconds 50
        }
        if (!$seen) { throw 'Product lifetime mutex missing' }
        Invoke-Installer (Join-Path $Packages 'qa1.exe') 'busy-update' $true
        $active.Refresh()
        if ($active.HasExited -or (Get-Item "$classes\xdzip.Archive\shell\open\command").GetValue('') -ne $command) { throw 'Active product changed' }
        $phases.Add('busy-update-rejected')
    } finally {
        $active.Refresh(); [void]$active.CloseMainWindow()
        if (!$active.WaitForExit(15000)) { $active.Kill(); throw 'Owned QA product close timed out' }
    }
    Invoke-Installer (Join-Path $Packages 'qa1.exe') 'rollback'; Test-AssociatedOpen 'rollback'
    Invoke-Installer (Join-Path $Packages 'product.exe') 'production-package'; Test-AssociatedOpen 'production-package'
    Invoke-Installer (Join-Path $testRoot 'unins000.exe') 'uninstall'
    for ($i=0; $i -lt 200 -and (Test-Path "$classes\xdzip.Archive"); $i++) { Start-Sleep -Milliseconds 50 }
    if (Test-Path "$classes\xdzip.Archive") { throw 'Owned ProgID remains' }
    if ((Get-ChildItem (Join-Path $testRoot 'versions') -Filter 'xdzip.exe' -Recurse -ErrorAction SilentlyContinue).Count) { throw 'Owned runtime remains' }
    if (!(Test-Path $sentinel) -or !(Select-String -Path $settings -SimpleMatch 'theme=dark' -Quiet)) { throw 'User data removed' }
    $remaining = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Classes\.zip\OpenWithProgids')
    if ($remaining.GetValueNames() -notcontains 'xdzip.QAForeign' -or $remaining.GetValueNames() -contains 'xdzip.Archive') { throw 'OpenWith uninstall ownership incorrect' }
    $remaining.Close(); Assert-Default
    $phases.Add('uninstall-preserves-user-data-and-default')
    @{ status='passed'; platform='windows'; phases=$phases.ToArray(); records=100000; scope='current user'; limits='Native CI shell dispatch, not an Explorer human click, signed release or physical-device acceptance.' } | ConvertTo-Json -Depth 4 | Set-Content $Output
    Write-Host 'Windows install/upgrade/rollback/busy protection/native association/uninstall passed'
} finally {
    Remove-Item Env:XDZIP_ASSOCIATION_PROBE -ErrorAction SilentlyContinue
    Remove-Item Env:XDZIP_SETTINGS_PATH -ErrorAction SilentlyContinue
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Classes\.zip\OpenWithProgids',$true)
    if ($key) { $key.DeleteValue('xdzip.QAForeign',$false); $key.Close() }
}
