$ErrorActionPreference = 'Stop'
if ($env:CI -ne 'true' -or $env:RUNNER_OS -ne 'Windows' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
    throw 'Disposable hosted Windows runner required'
}
. "$PSScriptRoot/../scripts/windows-settings-uia.ps1"
$formats = @('zip','tar','tgz','gz','7z','rar','cab','iso','udf')
$classes = 'HKCU:\Software\Classes'
$installRoot = Join-Path $env:RUNNER_TEMP 'xdzip language test'
$profileDir = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'xdzip\xdzip'
$profile = Join-Path $profileDir 'settings.ini'
if (Test-Path -LiteralPath $profile) { throw 'Existing application profile' }
if (Test-Path "$classes\xdzip.Archive") { throw 'Existing application registration' }
foreach ($format in $formats) {
    if (Test-Path "$classes\xdzip.Archive.$($format.ToUpperInvariant())") { throw 'Existing format registration' }
}
function DefaultSnapshot($format) {
    $base = [Microsoft.Win32.Registry]::ClassesRoot.OpenSubKey('.'+$format)
    $default = if ($base) { try { $base.GetValue('') } finally { $base.Close() } } else { $null }
    $choice = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.$format\UserChoice")
    $values = if ($choice) { try { @($choice.GetValueNames() | Sort-Object | ForEach-Object { "$_=$($choice.GetValue($_))" }) } finally { $choice.Close() } } else { @() }
    return (@{default=$default;choice=$values} | ConvertTo-Json -Compress)
}
function ContractSnapshot($format) {
    $id = 'xdzip.Archive.'+$format.ToUpperInvariant()
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Software\Classes\.$format\OpenWithProgids")
    if (!$key) { throw 'Missing candidates' }
    try { $candidates = @($key.GetValueNames() | Sort-Object | ForEach-Object { "$_/$($key.GetValueKind($_))" }) }
    finally { $key.Close() }
    if ($candidates -notcontains 'xdzip.QAForeign/None' -or $candidates -notcontains "$id/None") {
        throw "Missing test application candidate: $format"
    }
    return (@{icon=(Get-Item "$classes\$id\DefaultIcon").GetValue('');command=(Get-Item "$classes\$id\shell\open\command").GetValue('');candidates=$candidates} | ConvertTo-Json -Compress)
}
$defaults = @{}
foreach ($format in $formats) {
    $defaults[$format] = DefaultSnapshot $format
    $key = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey("Software\Classes\.$format\OpenWithProgids")
    try { $key.SetValue('xdzip.QAForeign', [byte[]]@(), [Microsoft.Win32.RegistryValueKind]::None) }
    finally { $key.Close() }
}
$process = $null
try {
    New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
    Set-Content -LiteralPath $profile -Encoding utf8 -Value "[General]`nschemaVersion=1`n[locale]`nlanguage=zh-CN`n[desktop]`ntrayEnabled=false"
    $installer = Join-Path $env:GITHUB_WORKSPACE 'build/issue187-input/xdzip-0.1.0-alpha.9-windows-x86_64-setup.exe'
    $hash = (Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash
    if ($hash -ne 'B58FDE9376B1D872021855C10A08DDFBFC19066E18AAFF223DDAB88F56985675') { throw 'Unexpected installer bytes' }
    $setup = Start-Process -FilePath $installer -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART',('/DIR="'+$installRoot+'"')) -WindowStyle Hidden -PassThru
    if (!$setup.WaitForExit(120000) -or $setup.ExitCode -ne 0) { throw 'Owned installation failed' }
    foreach ($format in $formats) {
        if ((DefaultSnapshot $format) -ne $defaults[$format]) { throw "Default changed on install: $format" }
    }
    $contracts = @{}
    foreach ($format in $formats) { $contracts[$format] = ContractSnapshot $format }
    Remove-Item Env:XDZIP_SETTINGS_PATH,Env:XDZIP_LANGUAGE -ErrorAction SilentlyContinue
    $env:QT_QPA_PLATFORM='windows'
    $env:QT_QUICK_BACKEND='software'
    $process = Start-Process -FilePath (Join-Path $installRoot 'xdzip.exe') -WindowStyle Hidden -PassThru
    Initialize-XdzipSettingsUia
    $zh = Get-Content 'resources/i18n/zh-CN.json' -Raw -Encoding utf8 | ConvertFrom-Json
    $en = Get-Content 'resources/i18n/en-US.json' -Raw -Encoding utf8 | ConvertFrom-Json
    Open-XdzipSettingsGeneral $process $zh
    foreach ($switch in @(
        @{from=$zh;to=$en;option=$zh.messages.'language.enUS';language='en-US';suffix=' archive'},
        @{from=$en;to=$zh;option=$en.messages.'language.zhCN';language='zh-CN';suffix=' 压缩包'}
    )) {
        Select-XdzipUiLanguage $process $switch.from $switch.option
        $null = Wait-XdzipOwnedUiElement $process $switch.to.messages.'settings.language' 'ControlType.ComboBox'
        $matched = $false
        for ($i=0; $i -lt 150; $i++) {
            $process.Refresh()
            if ($process.HasExited) { throw 'Installed product exited during switch' }
            $matched = (Get-Content -LiteralPath $profile -Raw -Encoding utf8) -match ('(?m)^language='+[regex]::Escape($switch.language)+'\r?$')
            foreach ($format in $formats) {
                $actual = (Get-Item "$classes\xdzip.Archive.$($format.ToUpperInvariant())").GetValue('')
                if ($actual -cne ($format.ToUpperInvariant()+$switch.suffix)) { $matched = $false }
            }
            if ($matched) { break }
            Start-Sleep -Milliseconds 100
        }
        if (!$matched) { throw "Descriptions did not synchronize: $($switch.language)" }
        foreach ($format in $formats) {
            if ((DefaultSnapshot $format) -ne $defaults[$format] -or (ContractSnapshot $format) -ne $contracts[$format]) {
                throw "Default, candidate, icon or command changed: $format"
            }
        }
        Write-Host "Installed language switch $($switch.language): nine descriptions and contracts passed"
    }
} finally {
    if ($process) { $process.Refresh(); if (!$process.HasExited) { $process.Kill(); $process.WaitForExit() } }
    $uninstaller = Join-Path $installRoot 'unins000.exe'
    if (Test-Path -LiteralPath $uninstaller -PathType Leaf) {
        $cleanup = Start-Process -FilePath $uninstaller -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART') -WindowStyle Hidden -PassThru
        if (!$cleanup.WaitForExit(120000) -or $cleanup.ExitCode -ne 0) { throw 'Owned uninstall failed' }
    }
    foreach ($format in $formats) {
        $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Software\Classes\.$format\OpenWithProgids",$true)
        if ($key) { try { $key.DeleteValue('xdzip.QAForeign',$false) } finally { $key.Close() } }
    }
    if (Test-Path -LiteralPath $profile) { Remove-Item -LiteralPath $profile -Force }
}
