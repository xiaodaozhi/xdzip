# Install only the owned, self-signed CI fixture on a disposable hosted runner.
param(
    [Parameter(Mandatory=$true)][string]$Package,
    [Parameter(Mandatory=$true)][string]$Output
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
$result = [ordered]@{
    status = 'failed'
    revision = (& git rev-parse HEAD).Trim()
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
    if ($errorCode -ne 0 -or $origin -eq 3) { throw 'Cannot establish that the test package is not Store-origin' }
    $result.checks.staged_origin = $origin
    $exe = Join-Path $installed.InstallLocation 'xdzip.exe'
    if (!(Test-Path -LiteralPath $exe -PathType Leaf)) { throw 'Installed product executable is missing' }
    $env:QT_QPA_PLATFORM = 'offscreen'
    $env:QT_QUICK_BACKEND = 'software'
    foreach ($check in @(@('--smoke-test', 0), @('--register-shell-integration', 13))) {
        $process = Start-Process -FilePath $exe -ArgumentList $check[0] -PassThru
        if (!$process.WaitForExit(60000)) {
            $process.Kill()
            throw 'Installed product check timed out'
        }
        if ($process.ExitCode -ne $check[1]) { throw "Installed product check $($check[0]) failed with exit code $($process.ExitCode)" }
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
    foreach ($case in @(@{ name='portable'; executable=$portable; updateVisible=$true }, @{ name='msix'; executable=$exe; updateVisible=$false })) {
      foreach ($language in @('en-US', 'zh-CN')) {
        $env:XDZIP_LANGUAGE = $language
        $catalog = Get-Content "resources/i18n/$language.json" -Raw -Encoding utf8 | ConvertFrom-Json
        $active = $null
        try {
            $active = Start-Process -FilePath $case.executable -PassThru
            $settingsButton = Wait-XdzipOwnedUiElement $active $catalog.messages.'settings.title' 'ControlType.Button'
            $settingsButton.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
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
                    if ($item.Current.ProcessId -eq $active.Id -and !$item.Current.IsOffscreen) {
                        $updateVisible = $true
                    }
                }
            }
            if ($updateVisible -ne $case.updateVisible) { throw "About update visibility differs for $($case.name), $language" }
        } finally {
            if ($active) {
                $active.Refresh()
                if (!$active.HasExited) { $active.Kill(); $active.WaitForExit() }
            }
        }
      }
    }
    $result.checks.about_update_channels_in_both_languages = 'passed'
    # Use the OS file contract and inspect an actual entry in the owned ZIP.
    Add-Type -Path 'scripts/windows-msix-activation.cs'
    $archiveName = 'xdzip msix fixture.zip'
    $archivePath = Join-Path $env:RUNNER_TEMP $archiveName
    if (Test-Path -LiteralPath $archivePath) { throw 'File activation fixture already exists' }
    Copy-Item -LiteralPath 'tests/fixtures/zip/normal.zip' -Destination $archivePath
    $activationProcess = $null
    try {
        $processId = [XdzipMsixActivation]::Open(($installed.PackageFamilyName + '!App'), $archivePath)
        $activationProcess = Get-Process -Id $processId -ErrorAction Stop
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
    $installed = $null
    $result.checks.uninstall = 'passed'
    $result.status = 'passed'
} finally {
    if ($installed) { Remove-AppxPackage -Package $installed.PackageFullName -ErrorAction Continue }
    if ($trusted -and $certificate) {
        $trusted.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
        $trusted.Remove([System.Security.Cryptography.X509Certificates.X509Certificate2]::new($certificate.RawData))
        $trusted.Close()
    }
    if ($certificate) { Remove-Item -LiteralPath "Cert:\CurrentUser\My\$($certificate.Thumbprint)" -ErrorAction Continue }
    $result | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $outputPath -Encoding utf8
}
