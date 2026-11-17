# Install only independent, owned fixtures on a disposable GitHub-hosted machine.
param([string]$BuildDirectory = 'build/msix-shell', [string]$Output = 'build/msix-shell/results.json')
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted' -or $env:RUNNER_OS -ne 'Windows') {
    throw 'Package installation is restricted to a disposable GitHub-hosted Windows runner'
}
$buildPath = (Resolve-Path -LiteralPath $BuildDirectory).Path
$outputPath = [IO.Path]::GetFullPath($Output)
if (Test-Path -LiteralPath $outputPath) { throw 'Evidence output already exists' }
$tools = Get-ChildItem "${env:ProgramFiles(x86)}/Windows Kits/10/bin/10.*/x64/signtool.exe" | Sort-Object { [version]$_.Directory.Parent.Name } -Descending
if (!$tools) { throw 'Windows SDK signing tool missing' }
$runtime = Join-Path $buildPath 'runtime'
$null = New-Item -ItemType Directory -Path $runtime
foreach ($file in @('xdzip.exe', 'xdzip-shell.dll')) { Copy-Item -LiteralPath (Join-Path $buildPath $file) -Destination $runtime }
$identity = (& (Join-Path $runtime 'xdzip.exe') --version-json | ConvertFrom-Json)
if ($LASTEXITCODE -ne 0) { throw 'Fixture identity failed' }
$files = @('xdzip.exe', 'xdzip-shell.dll') | ForEach-Object { @{path=$_; sha256=(Get-FileHash -LiteralPath (Join-Path $runtime $_) -Algorithm SHA256).Hash.ToLowerInvariant()} }
$audit = Join-Path $buildPath 'fixture-audit.json'
@{status='passed'; platform='windows'; mode='deployed'; identity=$identity; files=@($files)} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $audit -Encoding utf8
$subject = 'CN=xdzip Shell CI'
$certificate = $null
$installed = @()
$osInfo = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$result = [ordered]@{status='failed'; revision=(& git rev-parse HEAD).Trim(); os_build=$osInfo.CurrentBuildNumber; os_architecture=[Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString(); executable_architecture='x64'; fixture_only=$true; checks=[ordered]@{}; limitations=@('Self-signed independent fixture, not a Store-signed product.', 'Native menu API and package activation checks do not establish actual Explorer display or human visual acceptance.')}
$choices = 'Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts'
function Default-Snapshot {
    $snapshot = @{}
    foreach ($ext in @('zip','7z','rar','tar','gz','tgz','cab','iso','udf')) {
        foreach ($suffix in @('UserChoice','UserChoiceLatest\ProgId')) {
            $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("$choices\.$ext\$suffix")
            try { if ($key) { foreach ($value in $key.GetValueNames()) { $snapshot["$ext/$suffix/$value"] = $key.GetValue($value) } } }
            finally { if ($key) { $key.Close() } }
        }
    }
    return ($snapshot | ConvertTo-Json -Compress)
}
$before = Default-Snapshot
function Invoke-UserProbe($package, [string]$channel) {
    # Hosted runners are elevated. Explorer uses the interactive user's normal
    # token; elevated COM clients cannot validate per-user package activation.
    $taskName = "xdzip-shell-ci-$env:GITHUB_RUN_ID-$channel"
    if (Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue) { throw 'Owned fixture task already exists' }
    $log = Join-Path $buildPath "$channel-probe.log"
    $arguments = '--verify "{0}" "{1}" {2} "{3}"' -f $package.InstallLocation, $package.PackageFamilyName, $channel, $log
    $action = New-ScheduledTaskAction -Execute (Join-Path $buildPath 'xdzip.exe') -Argument $arguments -WorkingDirectory (Get-Location).Path
    $principal = New-ScheduledTaskPrincipal -UserId ([Security.Principal.WindowsIdentity]::GetCurrent().Name) -LogonType Interactive -RunLevel Limited
    $registered = $false
    try {
        $null = Register-ScheduledTask -TaskName $taskName -Action $action -Principal $principal
        $registered = $true
        $started = Get-Date
        Start-ScheduledTask -TaskName $taskName
        $finished = $false
        do {
            Start-Sleep -Milliseconds 250
            $info = Get-ScheduledTaskInfo -TaskName $taskName
            $state = (Get-ScheduledTask -TaskName $taskName).State
            $finished = $info.LastRunTime -ge $started.AddSeconds(-2) -and $state -notin @('Running', 'Queued')
        } while (!$finished -and (Get-Date) -lt $started.AddSeconds(60))
        if (Test-Path -LiteralPath $log) { Get-Content -LiteralPath $log }
        if (!$finished -or $info.LastTaskResult -ne 0) { throw "Packaged $channel user-token probe failed" }
    } finally {
        if ($registered) {
            Stop-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
            Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
        }
    }
}
try {
    if ([int]$osInfo.CurrentBuildNumber -lt 22000) { throw 'Windows 11 is required' }
    $certificate = New-SelfSignedCertificate -Type CodeSigningCert -Subject $subject -CertStoreLocation 'Cert:\CurrentUser\My' -KeyExportPolicy NonExportable -NotAfter (Get-Date).AddDays(2)
    $trusted = [Security.Cryptography.X509Certificates.X509Store]::new('TrustedPeople', 'LocalMachine')
    $trusted.Open([Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
    $trusted.Add([Security.Cryptography.X509Certificates.X509Certificate2]::new($certificate.RawData))
    $trusted.Close()
    foreach ($channel in @('sideload','store')) {
        $name = "xdzip.ShellCI.$channel"
        if (Get-AppxPackage -Name $name) { throw 'Fixture already installed' }
        $package = Join-Path $buildPath "$channel.msix"
        $report = Join-Path $buildPath "$channel.json"
        & python scripts/build-windows-msix.py --root $runtime --audit $audit --output $package --report $report --name $name --publisher $subject --publisher-display-name xdzip --channel $channel
        if ($LASTEXITCODE -ne 0) { throw 'Fixture packaging failed' }
        & $tools[0].FullName sign /fd SHA256 /sha1 $certificate.Thumbprint $package
        if ($LASTEXITCODE -ne 0) { throw 'Fixture signing failed' }
        Add-AppxPackage -Path $package
        $current = Get-AppxPackage -Name $name
        if (@($current).Count -ne 1 -or $current.SignatureKind.ToString() -eq 'Store') { throw 'Unexpected fixture identity or signature' }
        $installed += $current.PackageFullName
        Invoke-UserProbe $current $channel
        $result.checks[$channel] = 'passed'
        Remove-AppxPackage -Package $current.PackageFullName
        if (Get-AppxPackage -Name $name) { throw 'Fixture uninstall failed' }
        $result.checks["${channel}_uninstall"] = 'passed'
    }
    if ((Default-Snapshot) -ne $before) { throw 'Default application choices changed' }
    $result.checks.defaults_unchanged = 'passed'
    $result.status = 'passed'
} finally {
    foreach ($fullName in $installed) { if (Get-AppxPackage | Where-Object PackageFullName -eq $fullName) { Remove-AppxPackage -Package $fullName } }
    if ($certificate) {
        foreach ($store in @('Cert:\LocalMachine\TrustedPeople', 'Cert:\CurrentUser\My')) {
            $path = Join-Path $store $certificate.Thumbprint
            if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path }
        }
    }
    $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $outputPath -Encoding utf8
}
