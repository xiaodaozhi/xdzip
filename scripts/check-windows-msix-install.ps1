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
    limitations = @('Self-signed sideload only; not Store or Windows 11 desktop acceptance.', 'File activation, coexistence and UI visuals are not exercised by this check.')
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
