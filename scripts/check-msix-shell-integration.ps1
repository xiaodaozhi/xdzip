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
$testUser = $null
$developerKey = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock'
$developerSnapshot = @{}
$developerKeyCreated = !(Test-Path -LiteralPath $developerKey)
$osInfo = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$result = [ordered]@{status='failed'; revision=(& git rev-parse HEAD).Trim(); os_name=(Get-CimInstance Win32_OperatingSystem).Caption; os_build=$osInfo.CurrentBuildNumber; os_architecture=[Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString(); executable_architecture='x64'; fixture_only=$true; checks=[ordered]@{}; limitations=@('Self-signed independent fixture, not a Store-signed product.', 'Native menu API and package activation checks do not establish actual Explorer display or human visual acceptance.')}
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
function Invoke-UserProbe([string]$package, [string]$channel) {
    $output = Join-Path $buildPath "$channel-user.json"
    $arguments = @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File',
        ((Join-Path (Get-Location) 'scripts/check-msix-shell-user.ps1')), '-Package', $package,
        '-Probe', (Join-Path $buildPath 'xdzip.exe'), '-Channel', $channel, '-Output', $output)
    $quoted = $arguments | ForEach-Object { '"' + $_ + '"' }
    $process = Start-Process -FilePath (Get-Process -Id $PID).Path -ArgumentList $quoted -Credential $credential -LoadUserProfile -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $buildPath "$channel-user.log") -RedirectStandardError (Join-Path $buildPath "$channel-user-error.log")
    try {
        $handle = $process.Handle
        if (!$process.WaitForExit(90000)) { $process.Kill(); $process.WaitForExit(); throw 'Ordinary-user probe timed out' }
        Get-Content -LiteralPath (Join-Path $buildPath "$channel-user.log")
        Get-Content -LiteralPath (Join-Path $buildPath "$channel-user-error.log")
        if ($process.ExitCode -ne 0 -or !(Test-Path -LiteralPath $output) -or (Get-Content -LiteralPath $output -Raw | ConvertFrom-Json).status -ne 'passed') { throw 'Ordinary-user package verification failed' }
    } finally { $process.Dispose() }
}

try {
    if ([int]$osInfo.CurrentBuildNumber -lt 22000) { throw 'Windows build 22000 or newer is required' }
    if ($developerKeyCreated) { $null = New-Item -Path $developerKey }
    foreach ($name in @('AllowDevelopmentWithoutDevLicense', 'AllowAllTrustedApps')) {
        $key = Get-Item -LiteralPath $developerKey
        $exists = $key.GetValueNames() -contains $name
        $developerSnapshot[$name] = @{ exists=$exists; value=if ($exists) { $key.GetValue($name) } else { $null } }
        if ($exists -and $key.GetValueKind($name).ToString() -ne 'DWord') { throw 'Unexpected development-policy value type' }
        $null = New-ItemProperty -LiteralPath $developerKey -Name $name -PropertyType DWord -Value 1 -Force
    }
    $certificate = New-SelfSignedCertificate -Type CodeSigningCert -Subject $subject -CertStoreLocation 'Cert:\CurrentUser\My' -KeyExportPolicy NonExportable -NotAfter (Get-Date).AddDays(2)
    $trusted = [Security.Cryptography.X509Certificates.X509Store]::new('TrustedPeople', 'LocalMachine')
    $trusted.Open([Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
    $trusted.Add([Security.Cryptography.X509Certificates.X509Certificate2]::new($certificate.RawData))
    $trusted.Close()
    $userName = 'xdzipCI' + [guid]::NewGuid().ToString('N').Substring(0,8)
    $password = ConvertTo-SecureString ([guid]::NewGuid().ToString('N') + '!aA1') -AsPlainText -Force
    $testUser = New-LocalUser -Name $userName -Password $password -AccountNeverExpires -PasswordNeverExpires
    if (!(Get-LocalGroupMember -SID 'S-1-5-32-545' | Where-Object SID -eq $testUser.SID)) {
        Add-LocalGroupMember -SID 'S-1-5-32-545' -Member $testUser.Name
    }
    $credential = [pscredential]::new("$env:COMPUTERNAME\$userName", $password)
    $acl = Get-Acl -LiteralPath $buildPath
    $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($testUser.SID, 'Modify', 'ContainerInherit,ObjectInherit', 'None', 'Allow'))
    Set-Acl -LiteralPath $buildPath -AclObject $acl
    foreach ($channel in @('sideload','store')) {
        $name = "xdzip.ShellCI.$channel"
        if (Get-AppxPackage -Name $name) { throw 'Fixture already installed' }
        $package = Join-Path $buildPath "$channel.msix"
        $report = Join-Path $buildPath "$channel.json"
        & python scripts/build-windows-msix.py --root $runtime --audit $audit --output $package --report $report --name $name --publisher $subject --publisher-display-name xdzip --channel $channel
        if ($LASTEXITCODE -ne 0) { throw 'Fixture packaging failed' }
        & $tools[0].FullName sign /fd SHA256 /sha1 $certificate.Thumbprint $package
        if ($LASTEXITCODE -ne 0) { throw 'Fixture signing failed' }
        Invoke-UserProbe $package $channel
        $result.checks[$channel] = 'passed'
        $result.checks["${channel}_uninstall"] = 'passed'
    }
    if ((Default-Snapshot) -ne $before) { throw 'Default application choices changed' }
    $result.checks.defaults_unchanged = 'passed'
    $result.status = 'passed'
} finally {
    if ($testUser) { Remove-LocalUser -SID $testUser.SID }
    if ($certificate) {
        foreach ($store in @('Cert:\LocalMachine\TrustedPeople', 'Cert:\CurrentUser\My')) {
            $path = Join-Path $store $certificate.Thumbprint
            if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path }
        }
    }
    foreach ($name in $developerSnapshot.Keys) {
        if ($developerSnapshot[$name].exists) {
            Set-ItemProperty -LiteralPath $developerKey -Name $name -Value $developerSnapshot[$name].value
        } else {
            Remove-ItemProperty -LiteralPath $developerKey -Name $name
        }
    }
    if ($developerKeyCreated -and (Test-Path -LiteralPath $developerKey)) {
        $key = Get-Item -LiteralPath $developerKey
        if (!$key.ValueCount -and !$key.SubKeyCount) { Remove-Item -LiteralPath $developerKey }
    }
    $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $outputPath -Encoding utf8
}
