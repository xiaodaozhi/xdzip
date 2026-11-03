# Never launch the QA .exe payload: it is a file fixture, not a product binary.
$ErrorActionPreference = 'Stop'
if ($env:CI -ne 'true' -or $env:RUNNER_OS -ne 'Windows') { throw 'Disposable Windows CI only' }
. "$PSScriptRoot/windows-installer-paths.ps1"
$root = Join-Path $env:RUNNER_TEMP 'xdzip 安装器 smoke, space'
if (Test-Path 'HKCU:\Software\Classes\xdzip.Archive') { throw 'Refusing to overwrite an existing installation' }
$profileDirectory = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'xdzip\xdzip'
$profile = Join-Path $profileDirectory 'settings.ini'
if (Test-Path -LiteralPath $profile) { throw 'Refusing to overwrite an existing application profile' }
$profileDirectoryExisted = Test-Path -LiteralPath $profileDirectory
New-Item -ItemType Directory -Path $profileDirectory -Force | Out-Null
try {
function Set-AssociationTestLanguage($Language) {
    Set-Content -LiteralPath $profile -Encoding ascii -Value "[General]`nschemaVersion=1`n[locale]`nlanguage=$Language"
}
$extensions = @('zip','tar','tgz','gz','7z','rar','cab','iso','udf')
foreach ($extension in $extensions) {
    $key = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey("Software\Classes\.$extension\OpenWithProgids")
    $key.SetValue('xdzip.QAForeign', [byte[]]@(), [Microsoft.Win32.RegistryValueKind]::None)
    $key.Close()
}
function Assert-ApplicationIdentity($Language) {
    foreach ($extension in $extensions) {
        $progid = 'xdzip.Archive.'+$extension.ToUpperInvariant()
        $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Classes\'+$progid)
        try {
            if (!$key -or $key.GetValueKind('AllowSilentDefaultTakeOver') -ne [Microsoft.Win32.RegistryValueKind]::None) { throw 'Silent-default marker was not written as REG_NONE' }
            $expected = $extension.ToUpperInvariant() + $(if ($Language -eq 'en-US') { ' archive' } else { ' 压缩包' })
            if ($key.GetValue('') -ne $expected) { throw "Wrong association description for $extension" }
        } finally { if ($key) { $key.Close() } }
        $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Software\Classes\.$extension\OpenWithProgids")
        try {
            $names=$key.GetValueNames()
            if ($names -notcontains $progid -or $names -notcontains 'xdzip.QAForeign' -or $names -contains 'xdzip.Archive') { throw 'Application candidate migration lost ownership boundaries' }
        } finally { $key.Close() }
    }
}
function Install-Smoke($Package, $SetupLanguage='') {
    $arguments = @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART',('/DIR="'+$root+'"'))
    if ($SetupLanguage) { $arguments += '/LANG='+$SetupLanguage }
    $process = Start-Process $Package -ArgumentList $arguments -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw 'Native installer smoke failed' }
}
try {
    Install-Smoke 'build/installer-smoke/legacy.exe'
    $legacy = Get-Content 'build/installer-smoke/legacy.json' -Raw | ConvertFrom-Json
    $legacyExe = Join-Path $root $legacy.executable_relative_path
    Assert-XdzipShortcut $legacyExe
    $legacyKeep = Join-Path (Split-Path $legacyExe) 'user-kept.txt'; Set-Content $legacyKeep 'legacy extra content'
    Set-AssociationTestLanguage 'zh-CN'
    Install-Smoke 'build/installer-smoke/qa1.exe' 'english'
    Assert-ApplicationIdentity 'zh-CN'
    Assert-XdzipShortcut (Join-Path $root 'xdzip.exe')
    if (!(Test-Path $legacyExe)) { throw 'Unmanifested legacy runtime was removed' }
    $keep = Join-Path $root 'retired/user-kept.txt'; Set-Content $keep 'keep user content'
    Set-AssociationTestLanguage 'en-US'
    Install-Smoke 'build/installer-smoke/qa2.exe' 'chinesesimplified'
    Assert-ApplicationIdentity 'en-US'
    Assert-XdzipShortcut (Join-Path $root 'xdzip.exe')
    if ((Test-Path (Join-Path $root 'retired/old-owned.txt')) -or !(Test-Path $keep)) { throw 'Obsolete file cleanup failed or removed user content' }
    $expected = (Get-Content 'build/installer-smoke/qa2.json' -Raw | ConvertFrom-Json).payload_manifest_sha256
    if ((Get-FileHash (Join-Path $root 'install-manifest.txt')).Hash.ToLowerInvariant() -ne $expected) { throw 'Installed manifest bytes differ' }
} finally {
    $uninstaller = Join-Path $root 'unins000.exe'
    if (Test-Path $uninstaller) { Install-Smoke $uninstaller }
}
for ($i=0; $i -lt 100 -and (Test-Path (Join-Path $root 'xdzip.exe')); $i++) { Start-Sleep -Milliseconds 100 }
if ((Test-Path (Join-Path $root 'xdzip.exe')) -or (Test-Path $legacyExe) -or !(Test-Path $keep) -or !(Test-Path $legacyKeep)) { throw 'Uninstall failed to preserve user content' }
$root = Join-Path $env:RUNNER_TEMP 'xdzip fresh installer smoke'
Remove-Item -LiteralPath $profile
try {
    Install-Smoke 'build/installer-smoke/qa2.exe' 'english'
    Assert-ApplicationIdentity 'zh-CN'
    Assert-XdzipShortcut (Join-Path $root 'xdzip.exe')
    if (Test-Path (Join-Path $root 'versions')) { throw 'Fresh install created a versions directory' }
} finally {
    if (Test-Path (Join-Path $root 'unins000.exe')) { Install-Smoke (Join-Path $root 'unins000.exe') }
}
for ($i=0; $i -lt 100 -and (Test-Path 'HKCU:\Software\Classes\xdzip.Archive'); $i++) { Start-Sleep -Milliseconds 100 }
if ((Test-Path (Join-Path $root 'xdzip.exe')) -or (Test-Path 'HKCU:\Software\Classes\xdzip.Archive')) { throw 'Fresh smoke uninstall did not finish' }
foreach ($extension in $extensions) {
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Software\Classes\.$extension\OpenWithProgids",$true)
    try {
        if ($key.GetValueNames() -notcontains 'xdzip.QAForeign') { throw 'Uninstall removed foreign application candidate' }
        $key.DeleteValue('xdzip.QAForeign',$false)
    } finally { $key.Close() }
}
@{status='passed'; legacy_migration='passed'; fresh_install='passed';scope='QA file fixtures only, no product payload execution';layout='flat';shortcut='canonical path checked';owned_cleanup='passed';uninstall_preserves_extra='passed'} | ConvertTo-Json | Set-Content 'build/review/windows-installer-smoke.json'
} finally {
    Remove-Item -LiteralPath $profile -ErrorAction SilentlyContinue
    if (!$profileDirectoryExisted) { Remove-Item -LiteralPath $profileDirectory -ErrorAction SilentlyContinue }
}
