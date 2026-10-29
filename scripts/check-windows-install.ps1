# Runs only on the disposable native Windows CI host, never on the developer Mac.
param([string]$Packages, [string]$Fixture, [string]$Formats, [string]$Output)
$ErrorActionPreference = 'Stop'
if ($env:CI -ne 'true' -or $env:RUNNER_OS -ne 'Windows') { throw 'Disposable Windows CI only' }
. "$PSScriptRoot/windows-installer-paths.ps1"
. "$PSScriptRoot/windows-association-handlers.ps1"
$classes = 'HKCU:\Software\Classes'
$extensions = @('zip','tar','tgz','gz','7z','rar','cab','iso','udf')
$formatFiles = @{zip='sample.zip';tar='sample.tar';tgz='sample.tgz';gz='sample.tar.gz';'7z'='sample.7z';rar='sample.rar';cab='sample.cab';iso='sample.iso';udf='sample.udf'}
function Get-ProgId($Extension) {
    return 'xdzip.Archive.' + $Extension.ToUpperInvariant()
}
if (Test-Path "$classes\xdzip.Archive") { throw 'Refusing to replace an existing xdzip registration in this verification host' }
foreach ($extension in $extensions) {
    if (Test-Path "$classes\$(Get-ProgId $extension)") { throw 'Refusing to replace an existing format registration' }
}
$testRoot = Join-Path $env:RUNNER_TEMP 'xdzip 安装, space'
$evidenceRoot = Join-Path $env:RUNNER_TEMP 'xdzip-install-evidence'
New-Item -ItemType Directory -Force $evidenceRoot | Out-Null
$settings = Join-Path $evidenceRoot 'settings.ini'
Set-Content $settings "[General]`nschemaVersion=1`n[appearance]`ntheme=dark`n[privacy]`nrecentEnabled=false" -Encoding utf8
$env:XDZIP_SETTINGS_PATH = $settings
$archive = Join-Path $evidenceRoot '中文 space & --smoke-test.zip'
Copy-Item $Fixture $archive
function Get-DefaultSnapshot($Extension) {
    $key = [Microsoft.Win32.Registry]::ClassesRoot.OpenSubKey('.'+$Extension)
    $default = if ($key) { $key.GetValue(''); $key.Close() } else { $null }
    $choice = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.'+$Extension+'\UserChoice')
    $progId = if ($choice) { $choice.GetValue('ProgId') } else { $null }
    $hash = if ($choice) { $choice.GetValue('Hash'); $choice.Close() } else { $null }
    $latest = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.'+$Extension+'\UserChoiceLatest\ProgId')
    $latestValues = [ordered]@{}
    if ($latest) {
        try { foreach ($name in ($latest.GetValueNames() | Sort-Object)) { $latestValues[$name]=$latest.GetValue($name) } }
        finally { $latest.Close() }
    }
    return ([ordered]@{ default=$default; progId=$progId; hash=$hash; latest=$latestValues } | ConvertTo-Json -Compress)
}
$defaultBefore = @{}
foreach ($extension in $extensions) {
    $defaultBefore[$extension] = Get-DefaultSnapshot $extension
    $foreignKey = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey("Software\Classes\.$extension\OpenWithProgids")
    $foreignKey.SetValue('xdzip.QAForeign', [byte[]]@(), [Microsoft.Win32.RegistryValueKind]::None)
    $foreignKey.Close()
}
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
function Invoke-Installer($File, $Label, $ExpectFailure=$false, $ExpectedCode=-1) {
    $arguments = @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART',('/DIR="'+$testRoot+'"'),('/LOG="'+(Join-Path $evidenceRoot ($Label+'.log'))+'"'))
    $process = Start-Process $File -ArgumentList $arguments -PassThru
    if (!$process.WaitForExit(120000)) { $process.Kill(); throw 'QA installer timed out' }
    if ($ExpectFailure) {
        if ($process.ExitCode -eq 0) { throw "Expected installer rejection or incomplete cleanup: $Label" }
        if ($ExpectedCode -ge 0 -and $process.ExitCode -ne $ExpectedCode) { throw "Unexpected installer failure code: $Label" }
    } elseif ($process.ExitCode -ne 0) { throw "Installer phase failed: $Label" }
}
function Assert-Default {
    foreach ($extension in $extensions) {
        if ((Get-DefaultSnapshot $extension) -ne $defaultBefore[$extension]) { throw "Foreign default association changed: $extension" }
    }
}
$phases = [Collections.Generic.List[string]]::new()
$applicationChecks = [Collections.Generic.List[object]]::new()
function Test-SingleApplication($Label) {
    $rows = @(foreach ($extension in $extensions) {
        $progid = Get-ProgId $extension
        $our = @([XdzipAssociationHandlers]::Read('.'+$extension) | Where-Object { $_.UIName -eq 'xdzip' })
        if ($our.Count -ne 1 -or $our[0].Name -ne (Join-Path $testRoot 'xdzip-association.exe')) { throw "Duplicate or incorrect application candidate: $extension" }
        if ($our[0].Icon -ne (Join-Path $testRoot 'xdzip-association.exe')) { throw "Wrong application icon source: $extension" }
        foreach ($id in @('xdzip.Archive',$progid)) {
            $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Classes\'+$id)
            try {
                if ($key.GetValueKind('AllowSilentDefaultTakeOver') -ne [Microsoft.Win32.RegistryValueKind]::None) { throw 'Missing silent-default opt-out marker' }
            } finally { if ($key) { $key.Close() } }
        }
        # This is a fallback-route guard, not a substitute for the Settings UI.
        if ([XdzipAssociationHandlers]::FallbackProgId('.'+$extension) -like 'xdzip.*') { throw "Unexpected xdzip automatic default: $extension" }
        $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Classes\.'+$extension+'\OpenWithProgids')
        try {
            if ($key.GetValueNames() -notcontains 'xdzip.QAForeign') { throw 'Foreign application candidate removed' }
            if ($Label -ne 'actual-product-rollback' -and $key.GetValueNames() -contains 'xdzip.Archive') { throw 'Legacy recommendation was not retired' }
        } finally { $key.Close() }
        [ordered]@{extension=$extension; application_count=$our.Count; application=[IO.Path]::GetFileName($our[0].Name); icon=[IO.Path]::GetFileName($our[0].Icon)}
    })
    $applicationChecks.Add([ordered]@{phase=$Label; formats=$rows})
}
function Test-AssociatedOpen($Label) {
    foreach ($extension in $extensions) {
        $progid = if ($Label -in @('historical-product-install','actual-product-rollback')) { 'xdzip.Archive' } else { Get-ProgId $extension }
        $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Software\Classes\.$extension\OpenWithProgids")
        if (!$key -or $key.GetValueNames() -notcontains $progid) { throw "OpenWith candidate value missing: $extension" }
        $key.Close()
        if ($Label -notin @('historical-product-install','actual-product-rollback')) {
            $helperName = (Get-Item "$classes\Applications\xdzip-association.exe").GetValue('FriendlyAppName')
            if ($helperName -ne 'xdzip') { throw 'Association launcher is missing its application identity' }
            $icon = (Get-Item "$classes\$progid\DefaultIcon").GetValue('')
            if ($icon -ne ('"'+(Join-Path $testRoot "xdzip-$extension.ico")+'",0')) { throw "Wrong format icon: $extension" }
            $command = (Get-Item "$classes\$progid\shell\open\command").GetValue('')
            $target = 'xdzip-association.exe'
            if ($command -ne ('"'+(Join-Path $testRoot $target)+'" -- "%1"')) { throw "Wrong format command: $extension" }
        }
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
    if ($Label -notin @('historical-product-install','actual-product-rollback') -and ($result.presented_frames -lt 1 -or $result.welcome_frames -ne 0)) { throw 'Associated current product presented welcome content during startup' }
    $command = (Get-Item "$classes\xdzip.Archive\shell\open\command").GetValue('')
    $installedExe = [regex]::Match($command, '^"([^"]+)" -- "%1"$').Groups[1].Value
    if ($Label -notin @('historical-product-install','actual-product-rollback')) {
        if ($installedExe -ne (Join-Path $testRoot 'xdzip.exe')) { throw 'Current executable is not at the app root' }
        if (!(Test-Path (Join-Path $testRoot 'install-manifest.txt'))) { throw 'Installed ownership manifest missing' }
        $producer = Get-Content (Join-Path $Packages 'qa1.json') -Raw | ConvertFrom-Json
        if ((Get-FileHash (Join-Path $testRoot 'install-manifest.txt') -Algorithm SHA256).Hash.ToLowerInvariant() -ne $producer.payload_manifest_sha256) { throw 'Installed manifest differs from the audited producer bytes' }
        Assert-XdzipShortcut $installedExe
        $icon = (Get-Item "$classes\xdzip.Archive\DefaultIcon").GetValue('')
        if ($icon -ne ('"'+(Join-Path $testRoot 'xdzip-zip.ico')+'",0')) { throw 'Archive icon did not migrate to the app root' }
    } elseif ($installedExe -notlike (Join-Path $testRoot 'versions\*\xdzip.exe')) {
        throw 'Historical product fixture did not use the original versioned layout'
    }
    if ($Label -ne 'historical-product-install') { Test-SingleApplication $Label }
    Assert-Default
    $phases.Add($Label)
    Remove-Item Env:XDZIP_ASSOCIATION_PROBE
}
function Test-FormatDispatch($Label, $Which, $Revision) {
    foreach ($extension in $Which) {
        $progid = Get-ProgId $extension
        $probe = Join-Path $evidenceRoot ($Label+'-'+$extension+'.json')
        $env:XDZIP_ASSOCIATION_PROBE = $probe
        $info = [XdzipShell+ExecuteInfo]::new()
        $info.cbSize = [Runtime.InteropServices.Marshal]::SizeOf($info)
        $info.fMask = 0x541
        $info.lpVerb = 'open'; $info.lpFile = (Resolve-Path (Join-Path $Formats $formatFiles[$extension])).Path
        $info.lpClass = $progid; $info.nShow = 1
        if (![XdzipShell]::ShellExecuteExW([ref]$info)) { throw "Format shell dispatch failed: $extension" }
        try {
            if ([XdzipShell]::WaitForSingleObject($info.hProcess,25000) -ne 0) { throw "Format launcher timed out: $extension" }
        } finally { [void][XdzipShell]::CloseHandle($info.hProcess) }
        for ($i=0; $i -lt 300 -and !(Test-Path $probe); $i++) { Start-Sleep -Milliseconds 50 }
        if (!(Test-Path $probe)) { throw "Format product probe timed out: $extension" }
        $result = Get-Content $probe -Raw | ConvertFrom-Json
        if ($result.event -ne 'association_open' -or $result.records -lt 1 -or $result.revision -ne $Revision) { throw "Wrong format open result: $extension" }
        for ($i=0; $i -lt 300; $i++) {
            $handle = [XdzipShell]::OpenMutexW(0x100000,$false,'Local\xdzip.desktop.running')
            if ($handle -eq [IntPtr]::Zero) { break }
            [void][XdzipShell]::CloseHandle($handle); Start-Sleep -Milliseconds 50
        }
        if ($i -eq 300) { throw 'Associated product did not release its lifetime mutex' }
        Remove-Item Env:XDZIP_ASSOCIATION_PROBE
    }
    Assert-Default
    $phases.Add($Label+'-format-dispatch')
}
try {
    Invoke-Installer (Join-Path $Packages 'previous.exe') 'historical-product-install'; Test-AssociatedOpen 'historical-product-install'
    $sentinel = Join-Path $testRoot 'user-kept.txt'; Set-Content $sentinel 'keep user data'
    $settingsBefore = [Convert]::ToBase64String([IO.File]::ReadAllBytes($settings))
    $legacyCommand = (Get-Item "$classes\xdzip.Archive\shell\open\command").GetValue('')
    $legacyExe = [regex]::Match($legacyCommand, '^"([^"]+)" -- "%1"$').Groups[1].Value
    $nestedSentinel = Join-Path (Split-Path $legacyExe) 'user-kept-nested.txt'
    Set-Content $nestedSentinel 'keep nested user data'
    $historical = Get-Content (Join-Path $evidenceRoot 'historical-product-install.json') -Raw | ConvertFrom-Json
    Invoke-Installer (Join-Path $Packages 'product.exe') 'actual-product-upgrade'; Test-AssociatedOpen 'actual-product-upgrade'
    if (!(Test-Path $legacyExe) -or !(Test-Path $nestedSentinel)) { throw 'Legacy migration deleted files without an ownership manifest' }
    $phases.Add('legacy-to-flat-entrypoints-and-untracked-files-preserved')
    $current = Get-Content (Join-Path $evidenceRoot 'actual-product-upgrade.json') -Raw | ConvertFrom-Json
    Test-FormatDispatch 'current' $extensions $current.revision
    if ($historical.revision -eq $current.revision -or $historical.version -eq $current.version -or [Convert]::ToBase64String([IO.File]::ReadAllBytes($settings)) -ne $settingsBefore) { throw 'Full product upgrade did not change version/revision or preserve settings' }
    Invoke-Installer (Join-Path $Packages 'previous.exe') 'actual-product-rollback'; Test-AssociatedOpen 'actual-product-rollback'
    Test-FormatDispatch 'historical-rollback' @('zip','tgz') $historical.revision
    if ([Convert]::ToBase64String([IO.File]::ReadAllBytes($settings)) -ne $settingsBefore -or !(Select-String -Path $sentinel -SimpleMatch 'keep user data' -Quiet)) { throw 'Full product rollback lost user data' }
    Invoke-Installer (Join-Path $Packages 'qa1.exe') 'install'; Test-AssociatedOpen 'install'
    $manifest = Join-Path $testRoot 'install-manifest.txt'
    $manifestBefore = [IO.File]::ReadAllBytes($manifest)
    $exeHashBefore = (Get-FileHash (Join-Path $testRoot 'xdzip.exe') -Algorithm SHA256).Hash
    try {
        # Forged traversal and reparse-point records must fail before any writes.
        [IO.File]::AppendAllText($manifest, (('0'*64)+"`t..\keep.txt`n"))
        Invoke-Installer (Join-Path $Packages 'qa2.exe') 'invalid-manifest-rejected' $true
        if ((Get-FileHash (Join-Path $testRoot 'xdzip.exe') -Algorithm SHA256).Hash -ne $exeHashBefore) { throw 'Invalid manifest changed the product' }
    } finally { [IO.File]::WriteAllBytes($manifest,$manifestBefore) }
    $outside = Join-Path $evidenceRoot 'outside'; New-Item -ItemType Directory -Force $outside | Out-Null
    $outsideFile = Join-Path $outside 'keep.txt'; Set-Content $outsideFile 'outside content must survive'
    $outsideHash = (Get-FileHash $outsideFile -Algorithm SHA256).Hash.ToLowerInvariant()
    $junction = Join-Path $testRoot 'redirect'
    New-Item -ItemType Junction -Path $junction -Target $outside | Out-Null
    try {
        [IO.File]::AppendAllText($manifest, ($outsideHash+"`tredirect\keep.txt`n"))
        Invoke-Installer (Join-Path $Packages 'qa2.exe') 'reparse-cleanup-rejected' $true
        if ((Get-FileHash $outsideFile -Algorithm SHA256).Hash.ToLowerInvariant() -ne $outsideHash) { throw 'Cleanup escaped through a junction' }
    } finally { [IO.File]::WriteAllBytes($manifest,$manifestBefore); [IO.Directory]::Delete($junction) }
    $retired = Join-Path $testRoot 'retired\old-owned.txt'
    New-Item -ItemType Directory -Force (Split-Path $retired) | Out-Null
    Set-Content $retired 'owned obsolete runtime fixture'
    $retiredHash = (Get-FileHash $retired -Algorithm SHA256).Hash.ToLowerInvariant()
    [IO.File]::AppendAllText($manifest, ($retiredHash+"`tretired\old-owned.txt`n"))
    $changedManifest = [IO.File]::ReadAllBytes($manifest)
    try {
        Set-Content $retired 'modified content must survive'
        Invoke-Installer (Join-Path $Packages 'qa2.exe') 'modified-retired-file-rejected' $true
        if (!(Select-String -Path $retired -SimpleMatch 'modified content must survive' -Quiet)) { throw 'Modified obsolete file removed' }
    } finally { [IO.File]::WriteAllBytes($manifest,$changedManifest); Set-Content $retired 'owned obsolete runtime fixture' }
    $extra = Join-Path $testRoot 'retired\user-kept.txt'; Set-Content $extra 'unlisted user content'
    Invoke-Installer (Join-Path $Packages 'qa2.exe') 'upgrade'; Test-AssociatedOpen 'upgrade'
    if ((Test-Path $retired) -or !(Test-Path $extra)) { throw 'Manifest cleanup did not preserve unrelated content' }
    $phases.Add('manifest-traversal-reparse-modification-rejected-and-stale-owned-files-pruned')
    Set-Content $retired 'owned obsolete runtime fixture'
    [IO.File]::AppendAllText($manifest, ($retiredHash+"`tretired\old-owned.txt`n"))
    $locked = [IO.File]::Open($retired,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
    try {
        Invoke-Installer (Join-Path $Packages 'qa1.exe') 'locked-cleanup-incomplete' $true 20
        if (!(Test-Path $retired) -or !(Select-String -Path $manifest -SimpleMatch 'retired\old-owned.txt' -Quiet)) { throw 'Incomplete cleanup lost the locked file or retry record' }
    } finally { $locked.Dispose() }
    Invoke-Installer (Join-Path $Packages 'qa2.exe') 'cleanup-retry'; Test-AssociatedOpen 'cleanup-retry'
    if ((Test-Path $retired) -or !(Test-Path $extra)) { throw 'Cleanup retry did not preserve user content' }
    $phases.Add('locked-cleanup-exit20-record-preserved-and-retry-succeeded')
    $command = (Get-Item "$classes\xdzip.Archive\shell\open\command").GetValue('')
    $executable = [regex]::Match($command, '^"([^"]+)" -- "%1"$').Groups[1].Value
    if (!(Test-Path $executable)) { throw 'Association command quoting invalid' }
    # This probe checks installer lifetime protection, not close-to-tray. Keep
    # association probes on the original default-tray profile; only this owned,
    # task-free process opts out so CloseMainWindow is an explicit exit policy.
    $busySettings = Join-Path $evidenceRoot 'busy-settings.ini'
    Set-Content $busySettings ((Get-Content $settings -Raw) + "`n[desktop]`ntrayEnabled=false`n") -Encoding utf8
    $env:XDZIP_SETTINGS_PATH = $busySettings
    try { $active = Start-Process $executable -PassThru }
    finally { $env:XDZIP_SETTINGS_PATH = $settings }
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
        Invoke-Installer (Join-Path $testRoot 'unins000.exe') 'busy-uninstall' $true
        if ($active.HasExited -or !(Test-Path $executable)) { throw 'Uninstaller changed the active product' }
        $phases.Add('busy-update-and-uninstall-rejected')
    } finally {
        $active.Refresh(); [void]$active.CloseMainWindow()
        if (!$active.WaitForExit(15000)) { $active.Kill(); throw 'Owned QA product close timed out' }
    }
    Invoke-Installer (Join-Path $Packages 'qa1.exe') 'rollback'; Test-AssociatedOpen 'rollback'
    Invoke-Installer (Join-Path $Packages 'product.exe') 'production-package'; Test-AssociatedOpen 'production-package'
    Invoke-Installer (Join-Path $testRoot 'unins000.exe') 'uninstall'
    for ($i=0; $i -lt 200 -and (Test-Path "$classes\xdzip.Archive"); $i++) { Start-Sleep -Milliseconds 50 }
    if (Test-Path "$classes\xdzip.Archive") { throw 'Owned ProgID remains' }
    if (Test-Path "$classes\Applications\xdzip-association.exe") { throw 'Owned association launcher registration remains' }
    if ((Get-ChildItem $testRoot -Filter 'xdzip.exe' -Recurse -ErrorAction SilentlyContinue).Count) { throw 'Owned runtime remains' }
    if (!(Test-Path $sentinel) -or !(Select-String -Path $settings -SimpleMatch 'theme=dark' -Quiet)) { throw 'User data removed' }
    if (!(Test-Path $nestedSentinel) -or !(Test-Path $extra) -or (Test-Path $manifest)) { throw 'Uninstall lost nested user files or retained the ownership manifest' }
    foreach ($extension in $extensions) {
        $remaining = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Software\Classes\.$extension\OpenWithProgids")
        if (!$remaining -or $remaining.GetValueNames() -notcontains 'xdzip.QAForeign' -or $remaining.GetValueNames() -contains (Get-ProgId $extension)) { throw "OpenWith uninstall ownership incorrect: $extension" }
        $remaining.Close()
        if (Test-Path "$classes\$(Get-ProgId $extension)") { throw "Owned format ProgID remains: $extension" }
    }
    Assert-Default
    $phases.Add('uninstall-preserves-user-data-and-default')
    $testRoot = Join-Path $env:RUNNER_TEMP 'xdzip fresh flat'
    Invoke-Installer (Join-Path $Packages 'product.exe') 'fresh-flat-install'; Test-AssociatedOpen 'fresh-flat-install'
    if (Test-Path (Join-Path $testRoot 'versions')) { throw 'A fresh Windows installation created a versions directory' }
    Invoke-Installer (Join-Path $testRoot 'unins000.exe') 'fresh-flat-uninstall'
    for ($i=0; $i -lt 200 -and (Test-Path "$classes\xdzip.Archive"); $i++) { Start-Sleep -Milliseconds 50 }
    if ((Test-Path (Join-Path $testRoot 'xdzip.exe')) -or (Test-Path "$classes\xdzip.Archive")) { throw 'Fresh flat uninstall left executable or registration' }
    Assert-Default
    $phases.Add('fresh-flat-layout-install-and-uninstall')
    @{ status='passed'; layout='flat'; legacy_directory_policy='Preserve legacy files without a manifest until cumulative uninstall'; platform='windows'; phases=$phases.ToArray(); application_checks=$applicationChecks.ToArray(); records=100000; scope='current user'; busy_profile_tray_enabled=$false; product_revisions=@{ previous=$historical.revision; current=$current.revision }; limits='Native CI shell dispatch, not an Explorer human click, signed release or physical-device acceptance.' } | ConvertTo-Json -Depth 7 | Set-Content $Output
    Write-Host 'Windows install/upgrade/rollback/busy protection/native association/uninstall passed'
} finally {
    New-Item -ItemType Directory -Force 'build/review/installer-diagnostics' | Out-Null
    Get-ChildItem $evidenceRoot -File | Where-Object { $_.Extension -in @('.log','.json') } | Copy-Item -Destination 'build/review/installer-diagnostics/' -ErrorAction SilentlyContinue
    Remove-Item Env:XDZIP_ASSOCIATION_PROBE -ErrorAction SilentlyContinue
    Remove-Item Env:XDZIP_SETTINGS_PATH -ErrorAction SilentlyContinue
    foreach ($extension in $extensions) {
        $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Software\Classes\.$extension\OpenWithProgids",$true)
        if ($key) { $key.DeleteValue('xdzip.QAForeign',$false); $key.Close() }
    }
}
