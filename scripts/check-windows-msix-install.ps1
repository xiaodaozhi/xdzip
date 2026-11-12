# Install only the owned, self-signed CI fixture on a disposable hosted runner.
param(
    [Parameter(Mandatory=$true)][string]$Package,
    [Parameter(Mandatory=$true)][string]$Output,
    [string]$InstalledExecutable
)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted' -or $env:RUNNER_OS -ne 'Windows') {
    throw 'MSIX installation is restricted to a disposable GitHub-hosted Windows runner'
}
$identityName = 'xdzip.MsixCI'
$subject = 'CN=xdzip MSIX CI'
$packagePath = (Resolve-Path -LiteralPath $Package).Path
$outputPath = [IO.Path]::GetFullPath($Output)
if (Get-AppxPackage -Name $identityName) { throw 'Test package already exists' }
if (Test-Path -LiteralPath $outputPath) { throw 'Evidence output already exists' }
$tools = Get-ChildItem "${env:ProgramFiles(x86)}/Windows Kits/10/bin/10.*/x64/signtool.exe" |
    Sort-Object { [version]$_.Directory.Parent.Name } -Descending
if (!$tools) { throw 'Windows SDK SignTool is required' }
$signedPath = Join-Path (Split-Path -Parent $outputPath) 'signed-ci-fixture.msix'
if (Test-Path -LiteralPath $signedPath) { throw 'Signed fixture already exists' }
Copy-Item -LiteralPath $packagePath -Destination $signedPath
$certificate = $null
$trusted = $null
$installed = $null
$extensionsToCheck = @('zip','tar','tgz','gz','7z','rar','cab','iso','udf')
function Get-DefaultSnapshot([string]$extension) {
    $rootKey = [Microsoft.Win32.Registry]::ClassesRoot.OpenSubKey('.' + $extension)
    try { $default = if ($rootKey) { $rootKey.GetValue('') } else { $null } }
    finally { if ($rootKey) { $rootKey.Close() } }
    $choiceKey = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.' + $extension + '\UserChoice')
    try {
        $progId = if ($choiceKey) { $choiceKey.GetValue('ProgId') } else { $null }
        $hash = if ($choiceKey) { $choiceKey.GetValue('Hash') } else { $null }
    } finally { if ($choiceKey) { $choiceKey.Close() } }
    $latestKey = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.' + $extension + '\UserChoiceLatest\ProgId')
    $latest = [ordered]@{}
    try { if ($latestKey) { foreach ($name in ($latestKey.GetValueNames() | Sort-Object)) { $latest[$name] = $latestKey.GetValue($name) } } }
    finally { if ($latestKey) { $latestKey.Close() } }
    return ([ordered]@{ default=$default; progId=$progId; hash=$hash; latest=$latest } | ConvertTo-Json -Compress)
}
$defaultsBefore = @{}
foreach ($extension in $extensionsToCheck) { $defaultsBefore[$extension] = Get-DefaultSnapshot $extension }
function Assert-DefaultsUnchanged {
    foreach ($extension in $extensionsToCheck) {
        if ((Get-DefaultSnapshot $extension) -ne $defaultsBefore[$extension]) { throw "Default file association changed: $extension" }
    }
}
$result = [ordered]@{
    status = 'failed'
    test_revision = (& git rev-parse HEAD).Trim()
    package_revision = (Get-Content (Join-Path (Split-Path -Parent $packagePath) 'package-msix.json') -Raw | ConvertFrom-Json).source.revision
    platform = 'Windows Server hosted runner'
    checks = [ordered]@{}
    limitations = @('Self-signed sideload only; not Store or Windows 11 desktop acceptance.', 'Full installer coexistence and visual appearance are not exercised by this check.')
}
try {
    $certificate = New-SelfSignedCertificate -Type CodeSigningCert -Subject $subject -CertStoreLocation 'Cert:\CurrentUser\My' -KeyExportPolicy NonExportable -NotAfter (Get-Date).AddDays(2)
    $trusted = [System.Security.Cryptography.X509Certificates.X509Store]::new('TrustedPeople', 'LocalMachine')
    $trusted.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
    $trusted.Add([System.Security.Cryptography.X509Certificates.X509Certificate2]::new($certificate.RawData))
    $trusted.Close()
    & $tools[0].FullName sign /fd SHA256 /sha1 $certificate.Thumbprint $signedPath
    if ($LASTEXITCODE -ne 0) { throw 'Test package signing failed' }
    $signature = Get-AuthenticodeSignature -LiteralPath $signedPath
    if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Thumbprint -ne $certificate.Thumbprint) { throw 'Test package signature is not trusted' }
    $result.checks.test_signature = 'passed'
    Add-AppxPackage -Path $signedPath
    Assert-DefaultsUnchanged
    $result.checks.install_preserves_defaults = 'passed'
    $installed = Get-AppxPackage -Name $identityName
    if (@($installed).Count -ne 1 -or $installed.Publisher -ne $subject) { throw 'Installed package identity differs from the fixture' }
    $result.checks.install_identity = 'passed'
    $manifest = Get-AppxPackageManifest -Package $installed.PackageFullName
    $extensions = @($manifest.SelectNodes('//*[local-name()="Extension"]'))
    if (@($extensions | Where-Object { $_.Category -eq 'windows.fileExplorerContextMenus' }).Count) { throw 'Sideload fixture unexpectedly registers a modern context menu' }
    $types = @($manifest.SelectNodes('//*[local-name()="FileType"]') | ForEach-Object { $_.InnerText } | Sort-Object)
    if (($types -join ',') -ne ((@('.zip','.tar','.tgz','.gz','.7z','.rar','.cab','.iso','.udf') | Sort-Object) -join ',')) { throw 'Installed file-type declarations differ' }
    $result.checks.manifest = 'passed'
    Add-Type @'
using System.Runtime.InteropServices;
public static class XdzipPackageOrigin {
    [DllImport("kernelbase.dll", CharSet=CharSet.Unicode)]
    public static extern int GetStagedPackageOrigin(string fullName, out int origin);
}
'@
    $origin = -1
    $errorCode = [XdzipPackageOrigin]::GetStagedPackageOrigin($installed.PackageFullName, [ref]$origin)
    # Unknown or other origin values do not establish the side-load channel.
    if ($errorCode -ne 0 -or $origin -notin @(5, 6)) { throw 'Cannot establish a developer-signed or line-of-business test package origin' }
    $result.checks.staged_origin = $origin
    $exe = Join-Path $installed.InstallLocation 'xdzip.exe'
    if (!(Test-Path -LiteralPath $exe -PathType Leaf)) { throw 'Installed product executable is missing' }
    Add-Type -Path 'scripts/windows-msix-activation.cs'
    $appId = $installed.PackageFamilyName + '!App'
    foreach ($check in @(@('--smoke-test', 0), @('--register-shell-integration', 13))) {
        $process = $null
        try {
            $processId = [XdzipMsixActivation]::Start($appId, $check[0])
            $process = Get-Process -Id $processId -ErrorAction Stop
            $processHandle = $process.Handle
            if ([XdzipMsixActivation]::PackageName($processHandle) -ne $installed.PackageFullName) { throw 'Activated check has the wrong package identity' }
            if (!$process.WaitForExit(60000)) { throw 'Installed product check timed out' }
            if ($process.ExitCode -ne $check[1]) { throw "Installed product check $($check[0]) failed with exit code $($process.ExitCode)" }
        } finally {
            if ($process) {
                $process.Refresh()
                if (!$process.HasExited) { $process.Kill(); $process.WaitForExit() }
            }
            [XdzipMsixActivation]::Close()
        }
    }
    $result.checks.installed_smoke = 'passed'
    $result.checks.packaged_shell_registration_rejected = 'passed'
    # Observe the installed product's About page, not a synthetic QML fixture.
    # Only windows owned by this test process are inspected or terminated.
    . "$PSScriptRoot/windows-settings-uia.ps1"
    Initialize-XdzipSettingsUia
    $env:QT_QPA_PLATFORM = 'windows'
    $env:XDZIP_SETTINGS_PATH = Join-Path $env:RUNNER_TEMP 'xdzip-msix-ci-settings.ini'
    if (Test-Path -LiteralPath $env:XDZIP_SETTINGS_PATH) { throw 'Test settings already exist' }
    $portable = Join-Path (Split-Path -Parent $packagePath) 'windows/xdzip.exe'
    if (!(Test-Path -LiteralPath $portable -PathType Leaf)) { throw 'Audited portable control is missing' }
    function Assert-AboutUpdate($active, $catalog, [bool]$expected) {
        $about = Wait-XdzipOwnedUiElement $active $catalog.messages.'settings.tabAbout' 'ControlType.TabItem'
        $about.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Select()
        $null = Wait-XdzipOwnedUiElement $active $catalog.messages.'about.copy' 'ControlType.Button'
        $condition = [System.Windows.Automation.PropertyCondition]::new(
            [System.Windows.Automation.AutomationElement]::NameProperty, $catalog.messages.'update.check')
        $updateVisible = $false
        foreach ($handle in [XdzipOwnedSettingsUia]::Windows([uint32]$active.Id)) {
            $window = [System.Windows.Automation.AutomationElement]::FromHandle($handle)
            if (!$window) { continue }
            foreach ($item in $window.FindAll([System.Windows.Automation.TreeScope]::Descendants, $condition)) {
                if ($item.Current.ProcessId -eq $active.Id -and !$item.Current.IsOffscreen) { $updateVisible = $true }
            }
        }
        if ($updateVisible -ne $expected) { throw 'About update visibility differs from the expected channel' }
    }
    foreach ($language in @('en-US', 'zh-CN')) {
        $env:XDZIP_LANGUAGE = $language
        $catalog = Get-Content "resources/i18n/$language.json" -Raw -Encoding utf8 | ConvertFrom-Json
        $active = $null
        try {
            $active = Start-Process -FilePath $portable -PassThru
            if ([XdzipMsixActivation]::PackageName($active.Handle) -ne '') { throw 'Portable control unexpectedly has package identity' }
            $settingsButton = Wait-XdzipOwnedUiElement $active $catalog.messages.'settings.title' 'ControlType.Button'
            $settingsButton.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
            Assert-AboutUpdate $active $catalog $true
        } finally {
            if ($active) {
                $active.Refresh()
                if (!$active.HasExited) { $active.Kill(); $active.WaitForExit() }
            }
        }
    }
    Remove-Item Env:XDZIP_LANGUAGE -ErrorAction SilentlyContinue
    Remove-Item Env:XDZIP_SETTINGS_PATH -ErrorAction SilentlyContinue
    $active = $null
    try {
        $processId = [XdzipMsixActivation]::Start($appId, '')
        $active = Get-Process -Id $processId -ErrorAction Stop
        if ([XdzipMsixActivation]::PackageName($active.Handle) -ne $installed.PackageFullName) { throw 'Activated product has the wrong package identity' }
        $zh = Get-Content 'resources/i18n/zh-CN.json' -Raw -Encoding utf8 | ConvertFrom-Json
        $en = Get-Content 'resources/i18n/en-US.json' -Raw -Encoding utf8 | ConvertFrom-Json
        Open-XdzipSettingsGeneral $active $zh
        Assert-AboutUpdate $active $zh $false
        $general = Wait-XdzipOwnedUiElement $active $zh.messages.'settings.tabGeneral' 'ControlType.TabItem'
        $general.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Select()
        Select-XdzipUiLanguage $active $zh $zh.messages.'language.enUS'
        $null = Wait-XdzipOwnedUiElement $active $en.messages.'settings.tabGeneral' 'ControlType.TabItem'
        Assert-AboutUpdate $active $en $false
    } finally {
        if ($active) {
            $active.Refresh()
            if (!$active.HasExited) { $active.Kill(); $active.WaitForExit() }
        }
        [XdzipMsixActivation]::Close()
    }
    $result.checks.about_update_channels_in_both_languages = 'passed'
    # A fresh packaged process must retain its saved language and run at the
    # same time as the independent portable channel.
    $portableActive = $null
    $packagedActive = $null
    try {
        $env:XDZIP_LANGUAGE = 'en-US'
        $env:XDZIP_SETTINGS_PATH = Join-Path $env:RUNNER_TEMP 'xdzip-msix-ci-settings.ini'
        $portableActive = Start-Process -FilePath $portable -PassThru
        Remove-Item Env:XDZIP_LANGUAGE -ErrorAction SilentlyContinue
        Remove-Item Env:XDZIP_SETTINGS_PATH -ErrorAction SilentlyContinue
        $processId = [XdzipMsixActivation]::Start($appId, '')
        $packagedActive = Get-Process -Id $processId -ErrorAction Stop
        if ($portableActive.Id -eq $packagedActive.Id -or
            [XdzipMsixActivation]::PackageName($portableActive.Handle) -ne '' -or
            [XdzipMsixActivation]::PackageName($packagedActive.Handle) -ne $installed.PackageFullName) {
            throw 'Portable and packaged processes do not have distinct channel identities'
        }
        $null = Wait-XdzipOwnedUiElement $portableActive $en.messages.'settings.title' 'ControlType.Button'
        $null = Wait-XdzipOwnedUiElement $packagedActive $en.messages.'settings.title' 'ControlType.Button'
        $portableActive.Refresh()
        if ($portableActive.HasExited) { throw 'Portable process did not coexist with the package' }
    } finally {
        foreach ($owned in @($portableActive, $packagedActive)) {
            if ($owned) {
                $owned.Refresh()
                if (!$owned.HasExited) { $owned.Kill(); $owned.WaitForExit() }
            }
        }
        [XdzipMsixActivation]::Close()
    }
    $result.checks.saved_language_and_portable_coexistence = 'passed'
    if ($InstalledExecutable) {
        $installedControl = $null
        $packagedControl = $null
        try {
            $installedControl = Start-Process -FilePath $InstalledExecutable -PassThru
            $processId = [XdzipMsixActivation]::Start($appId, '')
            $packagedControl = Get-Process -Id $processId -ErrorAction Stop
            if ($installedControl.Id -eq $packagedControl.Id -or
                [XdzipMsixActivation]::PackageName($installedControl.Handle) -ne '' -or
                [XdzipMsixActivation]::PackageName($packagedControl.Handle) -ne $installed.PackageFullName) {
                throw 'Installed and packaged processes do not have distinct channel identities'
            }
            $null = Wait-XdzipOwnedUiElement $packagedControl $en.messages.'settings.title' 'ControlType.Button'
            for ($attempt = 0; $attempt -lt 150; $attempt++) {
                $installedControl.Refresh()
                if ($installedControl.HasExited) { throw 'Installed process exited while the package was running' }
                if (@([XdzipOwnedSettingsUia]::Windows([uint32]$installedControl.Id)).Count -gt 0) { break }
                Start-Sleep -Milliseconds 100
            }
            if ($attempt -eq 150) { throw 'Installed process did not create a window' }
        } finally {
            foreach ($owned in @($installedControl, $packagedControl)) {
                if ($owned) {
                    $owned.Refresh()
                    if (!$owned.HasExited) { $owned.Kill(); $owned.WaitForExit() }
                }
            }
            [XdzipMsixActivation]::Close()
        }
        $result.checks.installed_edition_coexistence = 'passed'
        $result.limitations = @('Self-signed sideload only; not Store or Windows 11 desktop acceptance.', 'Visual appearance is not exercised by this check.')
    }
    # Invoke the package's actual Open With handler and inspect an owned ZIP entry.
    $archiveName = 'xdzip msix fixture.zip'
    $archivePath = Join-Path $env:RUNNER_TEMP $archiveName
    if (Test-Path -LiteralPath $archivePath) { throw 'File activation fixture already exists' }
    Copy-Item -LiteralPath 'tests/fixtures/zip/normal.zip' -Destination $archivePath
    $directProcess = $null
    try {
        $processId = [XdzipMsixActivation]::Start($appId, ('"' + $archivePath + '"'))
        $directProcess = Get-Process -Id $processId -ErrorAction Stop
        if ([XdzipMsixActivation]::PackageName($directProcess.Handle) -ne $installed.PackageFullName) { throw 'Archive check has the wrong package identity' }
        $null = Wait-XdzipOwnedUiElement $directProcess $archiveName 'ControlType.TabItem'
        $null = Wait-XdzipOwnedUiElement $directProcess 'empty.txt' 'ControlType.CheckBox'
    } finally {
        if ($directProcess) {
            $directProcess.Refresh()
            if (!$directProcess.HasExited) { $directProcess.Kill(); $directProcess.WaitForExit() }
        }
        [XdzipMsixActivation]::Close()
    }
    $result.checks.packaged_archive_read = 'passed'
    $activationProcess = $null
    try {
        $progId = [XdzipMsixActivation]::OpenAssociated($appId, $exe, $archivePath)
        $result.checks.shell_handler_invoke = 'returned_success'
        $result.checks.shell_handler_progid = if ($progId) { 'present' } else { 'unavailable' }
        for ($attempt = 0; $attempt -lt 300 -and !$activationProcess; $attempt++) {
            if ($attempt -eq 150 -and $progId) {
                [XdzipMsixActivation]::OpenWithProgId($progId, $archivePath)
                $result.checks.shell_progid_invoke = 'returned_success'
            }
            foreach ($candidate in @(Get-Process -Name xdzip -ErrorAction SilentlyContinue)) {
                try {
                    if ([XdzipMsixActivation]::PackageName($candidate.Handle) -eq $installed.PackageFullName) { $activationProcess = $candidate; break }
                } catch [System.InvalidOperationException] {}
                  catch [System.ComponentModel.Win32Exception] {}
            }
            if (!$activationProcess) { Start-Sleep -Milliseconds 100 }
        }
        if (!$activationProcess) {
            $result.checks.shell_handler_process = 'not_observed'
            $dialogElements = [System.Collections.Generic.List[object]]::new()
            foreach ($dialog in @(Get-Process -Name OpenWith -ErrorAction SilentlyContinue)) {
                foreach ($handle in [XdzipOwnedSettingsUia]::Windows([uint32]$dialog.Id)) {
                    try {
                        $window = [System.Windows.Automation.AutomationElement]::FromHandle($handle)
                        if (!$window) { continue }
                        $all = $window.FindAll([System.Windows.Automation.TreeScope]::Descendants,
                            [System.Windows.Automation.Condition]::TrueCondition)
                        foreach ($entry in $all) {
                            $name = $entry.Current.Name
                            if ($name -and $name.Length -le 80 -and $name -notmatch '[\\/]') {
                                $dialogElements.Add(@{ name=$name; type=$entry.Current.ControlType.ProgrammaticName; offscreen=$entry.Current.IsOffscreen })
                                if ($dialogElements.Count -ge 40) { break }
                            }
                        }
                    } catch [System.Windows.Automation.ElementNotAvailableException] {}
                }
            }
            $result.checks.open_with_dialog_elements = @($dialogElements.ToArray())
            throw 'File handler did not start the installed package'
        }
        $null = Wait-XdzipOwnedUiElement $activationProcess $archiveName 'ControlType.TabItem'
        $null = Wait-XdzipOwnedUiElement $activationProcess 'empty.txt' 'ControlType.CheckBox'
    } finally {
        if ($activationProcess) {
            $activationProcess.Refresh()
            if (!$activationProcess.HasExited) { $activationProcess.Kill(); $activationProcess.WaitForExit() }
        }
    }
    $result.checks.file_activation_and_archive_contents = 'passed'
    Remove-AppxPackage -Package $installed.PackageFullName
    if (Get-AppxPackage -Name $identityName) { throw 'Test package remains installed' }
    Assert-DefaultsUnchanged
    $result.checks.uninstall_preserves_defaults = 'passed'
    $installed = $null
    $result.checks.uninstall = 'passed'
    $result.status = 'passed'
} finally {
    if ($installed) {
        try {
            Remove-AppxPackage -Package $installed.PackageFullName -ErrorAction Stop
            if (Get-AppxPackage -Name $identityName) { throw 'Test package remains after cleanup' }
            $result.checks.cleanup_uninstall = 'passed'
        } catch {
            $result.checks.cleanup_uninstall = 'failed'
            $result.status = 'failed'
        }
    }
    if ($trusted -and $certificate) {
        $trusted.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
        $trusted.Remove([System.Security.Cryptography.X509Certificates.X509Certificate2]::new($certificate.RawData))
        $trusted.Close()
    }
    if ($certificate) { Remove-Item -LiteralPath "Cert:\CurrentUser\My\$($certificate.Thumbprint)" -ErrorAction Continue }
    $result | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $outputPath -Encoding utf8
}
