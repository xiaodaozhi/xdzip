$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted' -or -not $env:RUNNER_TEMP) {
    throw 'This probe runs only on a disposable GitHub-hosted Windows runner.'
}
$records = Join-Path (Get-Location) 'build/public-update-records'
New-Item -ItemType Directory -Force -Path $records | Out-Null
$work = Join-Path $env:RUNNER_TEMP 'xdzip-public-update-probe'
if (Test-Path -LiteralPath $work) { throw 'Refusing an existing test directory.' }
New-Item -ItemType Directory -Path $work | Out-Null
$installRoot = Join-Path $work 'installed'
$settings = Join-Path $work 'settings.ini'
$setup = Join-Path (Get-Location) 'build/alpha8-candidate/xdzip-0.1.0-alpha.8-windows-x86_64-setup.exe'
$oldSetupHash = '246e0107c4d1e0c2c64f296042bfcafc4e8f92768c76b50cc9fc80c065a10600'
$newSetupHash = 'b58fde9376b1d872021855c10a08ddfbfc19066e18aaff223ddab88f56985675'
$oldAppHash = '7391003d93f9ea358c7ca530ac3244497dc0a31028dea6f3535313329747ba1f'
$newAppHash = 'c58f7c65249c35b687e9b7333bba632194de3f2c45892eba0c36149b9f045204'
$script:stage = 'prepare'
$script:events = [System.Collections.Generic.List[string]]::new()
$script:licenseSeen = $false
$script:lastSnapshot = @()
$script:oldProcessId = 0
$script:outcome = 'failed'
$script:observations = [System.Collections.Generic.List[object]]::new()
$script:downloadControl = $null
$script:screenshotError = ''
$script:installAcl = [System.Collections.Generic.List[object]]::new()

function Record([string]$name) { $script:events.Add($name); Write-Host $name }
function File-Hash([string]$path) { (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() }
function Process-Elements([int]$processId) {
    $condition = [System.Windows.Automation.PropertyCondition]::new(
        [System.Windows.Automation.AutomationElement]::ProcessIdProperty, $processId)
    $windows = [System.Windows.Automation.AutomationElement]::RootElement.FindAll(
        [System.Windows.Automation.TreeScope]::Children, $condition)
    for ($i = 0; $i -lt $windows.Count; $i++) {
        $window = $windows.Item($i)
        Write-Output -NoEnumerate $window
        try {
            $items = $window.FindAll([System.Windows.Automation.TreeScope]::Descendants,
                [System.Windows.Automation.Condition]::TrueCondition)
            for ($j = 0; $j -lt $items.Count; $j++) { Write-Output -NoEnumerate $items.Item($j) }
        } catch [System.Windows.Automation.ElementNotAvailableException] { }
    }
}
function Snapshot-Process([int]$processId) {
    $list = [System.Collections.Generic.List[object]]::new()
    foreach ($element in @(Process-Elements $processId)) {
        try {
            $current = $element.Current
            $name = [string]$current.Name
            if ($name.Length -gt 160) { $name = $name.Substring(0, 160) }
            $name = $name.Replace($env:RUNNER_TEMP, '<runner-temp>').Replace($env:USERPROFILE, '<profile>')
            if ($name -or $current.ControlType -eq [System.Windows.Automation.ControlType]::Button) {
                $list.Add([pscustomobject]@{ name=$name; type=$current.ControlType.ProgrammaticName; enabled=$current.IsEnabled })
            }
        } catch [System.Windows.Automation.ElementNotAvailableException] { }
        if ($list.Count -ge 180) { break }
    }
    $script:lastSnapshot = @($list.ToArray())
}
function Observe-Update([int]$processId, [string]$label) {
    Snapshot-Process $processId
    $messages = @($script:lastSnapshot | Where-Object {
        $_.name -match '(?i)(update|download|install|unable|failed|error|prepar|verif)'
    } | Select-Object -First 14)
    $cacheRoot = Join-Path $env:LOCALAPPDATA 'xdzip'
    $files = @()
    if (Test-Path -LiteralPath $cacheRoot) {
        $files = @(Get-ChildItem -LiteralPath $cacheRoot -File -Recurse -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -match '[\\/]updates[\\/]' } | Select-Object -First 30 | ForEach-Object {
                $hash = $null
                $helperError = $null
                if ($_.Name -eq 'xdzip-0.1.0-alpha.9-windows-x86_64-setup.exe') {
                    try { $hash = File-Hash $_.FullName } catch { }
                }
                if ($_.Name -eq 'result.json' -and $_.Length -le 4096) {
                    try { $helperError = (Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json).error } catch { }
                }
                [pscustomobject]@{ name=$_.Name; bytes=$_.Length; sha256=$hash; helper_error=$helperError }
            })
    }
    $processes = @(Get-Process -ErrorAction SilentlyContinue | Where-Object {
        $_.ProcessName -match '(?i)xdzip|setup'
    } | Select-Object -First 20 | ForEach-Object {
        [pscustomobject]@{ name=$_.ProcessName; id=$_.Id; title=$_.MainWindowTitle }
    })
    $script:observations.Add([pscustomobject]@{
        label=$label; time_utc=[DateTime]::UtcNow.ToString('o');
        original_app_alive=($null -ne (Get-Process -Id $processId -ErrorAction SilentlyContinue));
        update_state_present=(Test-Path -LiteralPath (Join-Path $work 'update-state.ini'));
        messages=$messages; cache_files=$files; related_processes=$processes
    })
}
function Measure-PublicDownload {
    $url = 'https://github.com/xiaodaozhi/xdzip/releases/download/v0.1.0-alpha.9/xdzip-0.1.0-alpha.9-windows-x86_64-setup.exe'
    $target = Join-Path $work 'public-download-control.exe'
    $output = & curl.exe --silent --show-error --location --fail --max-time 120 --user-agent 'xdzip/0.1.0-alpha.8' --output $target --write-out '%{http_code} %{size_download}' $url 2>&1
    $exit = $LASTEXITCODE
    $script:downloadControl = [pscustomobject]@{
        exit_code=$exit; result=(@($output) -join ' ');
        bytes=if (Test-Path -LiteralPath $target) { (Get-Item -LiteralPath $target).Length } else { 0 };
        expected_sha256=if ($exit -eq 0 -and (Test-Path -LiteralPath $target)) { (File-Hash $target) -eq $newSetupHash } else { $false }
    }
}
function Observe-InstallAcl([string]$label) {
    $userSid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $acl = Get-Acl -LiteralPath $installRoot
    $entries = @($acl.Access | ForEach-Object {
        $sid = ''
        try { $sid = $_.IdentityReference.Translate([System.Security.Principal.SecurityIdentifier]).Value } catch { }
        $principal = switch ($sid) {
            $userSid { 'current-user' }
            'S-1-1-0' { 'everyone' }
            'S-1-5-11' { 'authenticated-users' }
            'S-1-5-32-545' { 'builtin-users' }
            'S-1-5-32-544' { 'administrators' }
            'S-1-5-18' { 'system' }
            default { 'other' }
        }
        [pscustomobject]@{ principal=$principal; type=$_.AccessControlType.ToString();
            rights=$_.FileSystemRights.ToString(); inherited=$_.IsInherited }
    })
    $script:installAcl.Add([pscustomobject]@{ label=$label; protected=$acl.AreAccessRulesProtected; entries=$entries })
}
function Protect-OwnedInstallDirectory {
    $ownedPrefix = [IO.Path]::GetFullPath($work).TrimEnd('\') + '\'
    $resolved = [IO.Path]::GetFullPath($installRoot)
    if (-not $resolved.StartsWith($ownedPrefix, [StringComparison]::OrdinalIgnoreCase) -or
        -not (Test-Path -LiteralPath (Join-Path $resolved 'install-manifest.txt'))) {
        throw 'Refusing to change an unowned installation directory.'
    }
    Observe-InstallAcl 'inherited-fixture'
    $sid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    & icacls.exe $resolved '/inheritance:r' '/grant:r' ('*' + $sid + ':(OI)(CI)F') | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Cannot set private fixture directory access.' }
    Observe-InstallAcl 'private-fixture'
    if (-not $script:installAcl[-1].protected) { throw 'Fixture directory access was not protected.' }
}
function Find-AppButton([int]$processId, [string]$pattern) {
    foreach ($element in @(Process-Elements $processId)) {
        try {
            $current = $element.Current
            if ($current.ControlType -eq [System.Windows.Automation.ControlType]::Button -and
                $current.IsEnabled -and $current.Name -match $pattern) { return $element }
        } catch [System.Windows.Automation.ElementNotAvailableException] { }
    }
    return $null
}
function Wait-AppButton([int]$processId, [string]$pattern, [int]$seconds) {
    $deadline = [DateTime]::UtcNow.AddSeconds($seconds)
    while ([DateTime]::UtcNow -lt $deadline) {
        $element = Find-AppButton $processId $pattern
        if ($null -ne $element) { return $element }
        Start-Sleep -Milliseconds 700
    }
    Snapshot-Process $processId
    throw 'Expected application button did not appear.'
}
function Wait-AppTab([int]$processId, [string]$pattern, [int]$seconds) {
    $deadline = [DateTime]::UtcNow.AddSeconds($seconds)
    while ([DateTime]::UtcNow -lt $deadline) {
        foreach ($element in @(Process-Elements $processId)) {
            try {
                $current = $element.Current
                if ($current.ControlType -eq [System.Windows.Automation.ControlType]::TabItem -and
                    $current.IsEnabled -and $current.Name -match $pattern) { return $element }
            } catch [System.Windows.Automation.ElementNotAvailableException] { }
        }
        Start-Sleep -Milliseconds 700
    }
    Snapshot-Process $processId
    throw 'Expected application tab did not appear.'
}
function Save-SettingsWindow([int]$processId) {
    Add-Type -AssemblyName System.Drawing
    foreach ($window in @(Process-Elements $processId)) {
        if ($window.Current.ControlType -ne [System.Windows.Automation.ControlType]::Window -or
            $window.Current.Name -ne 'Settings') { continue }
        try { $window.SetFocus() } catch { }
        Start-Sleep -Seconds 1
        $rectangle = $window.Current.BoundingRectangle
        $width = [int][Math]::Ceiling($rectangle.Width)
        $height = [int][Math]::Ceiling($rectangle.Height)
        if ($rectangle.IsEmpty -or $width -lt 1 -or $height -lt 1 -or $width -gt 4096 -or $height -gt 4096) {
            throw 'Settings screenshot has unexpected dimensions.'
        }
        $bitmap = [System.Drawing.Bitmap]::new($width, $height)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        try {
            $graphics.CopyFromScreen([int]$rectangle.X, [int]$rectangle.Y, 0, 0, $bitmap.Size)
            $bitmap.Save((Join-Path $records 'updated-about.png'), [System.Drawing.Imaging.ImageFormat]::Png)
        } finally {
            $graphics.Dispose()
            $bitmap.Dispose()
        }
        return
    }
    throw 'Settings window was not found for screenshot.'
}
function Activate-Element($element, [bool]$select = $false) {
    $pattern = $null
    if ($select -and $element.TryGetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern, [ref]$pattern)) {
        $pattern.Select(); return
    }
    $pattern = $null
    if ($element.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern, [ref]$pattern)) {
        $pattern.Invoke(); return
    }
    $rectangle = $element.Current.BoundingRectangle
    if ($rectangle.IsEmpty -or $element.Current.IsOffscreen) { throw 'Element has no visible activation point.' }
    [ProbeMouse]::Click([int]($rectangle.X + $rectangle.Width / 2), [int]($rectangle.Y + $rectangle.Height / 2))
}
function Wizard-Window {
    $root = [System.Windows.Automation.AutomationElement]::RootElement
    $windows = $root.FindAll([System.Windows.Automation.TreeScope]::Children,
        [System.Windows.Automation.Condition]::TrueCondition)
    for ($i = 0; $i -lt $windows.Count; $i++) {
        try {
            $item = $windows.Item($i)
            if ($item.Current.ClassName -eq 'TWizardForm' -and $item.Current.Name -match '(?i)xdzip') { return $item }
        } catch [System.Windows.Automation.ElementNotAvailableException] { }
    }
    return $null
}
function Find-WizardControl($window, $type, [string]$pattern) {
    $items = $window.FindAll([System.Windows.Automation.TreeScope]::Descendants,
        [System.Windows.Automation.Condition]::TrueCondition)
    for ($i = 0; $i -lt $items.Count; $i++) {
        $item = $items.Item($i)
        try {
            if ($item.Current.ControlType -eq $type -and $item.Current.IsEnabled -and
                -not $item.Current.IsOffscreen -and $item.Current.Name -match $pattern) { return $item }
        } catch [System.Windows.Automation.ElementNotAvailableException] { }
    }
    return $null
}
function Complete-SetupLanguage {
    $root = [System.Windows.Automation.AutomationElement]::RootElement
    $windows = $root.FindAll([System.Windows.Automation.TreeScope]::Children,
        [System.Windows.Automation.Condition]::TrueCondition)
    for ($i = 0; $i -lt $windows.Count; $i++) {
        try {
            $item = $windows.Item($i)
            if ($item.Current.Name -ne 'Select Setup Language') { continue }
            $process = Get-Process -Id $item.Current.ProcessId -ErrorAction SilentlyContinue
            if ($null -eq $process -or $process.ProcessName -notmatch '(?i)^xdzip-0\.1\.0-alpha\.9-windows-x86_64-setup') { continue }
            $button = Find-WizardControl $item ([System.Windows.Automation.ControlType]::Button) '(?i)^\s*&?OK\s*$'
            if ($null -eq $button) { return $false }
            Activate-Element $button
            Record 'setup-language-confirmed'
            return $true
        } catch [System.Windows.Automation.ElementNotAvailableException] { }
    }
    return $false
}
try {
    Add-Type -AssemblyName UIAutomationClient
    Add-Type -AssemblyName UIAutomationTypes
    Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class ProbeMouse {
    [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] static extern void mouse_event(uint flags, uint x, uint y, uint data, UIntPtr extra);
    public static void Click(int x, int y) { SetCursorPos(x,y); mouse_event(2,0,0,0,UIntPtr.Zero); mouse_event(4,0,0,0,UIntPtr.Zero); }
}
'@
    $os = Get-CimInstance Win32_OperatingSystem
    if ([int]$os.BuildNumber -lt 22000 -or $env:PROCESSOR_ARCHITECTURE -ne 'AMD64') { throw 'Runner does not meet the released updater minimum build.' }
    if ((File-Hash $setup) -ne $oldSetupHash) { throw 'The alpha.8 setup hash differs from the approved candidate.' }
    if (Get-Process -Name xdzip -ErrorAction SilentlyContinue) { throw 'Unexpected pre-existing xdzip process on runner.' }
    if ((Test-Path -LiteralPath 'HKCU:\Software\xdzip\Install') -or (Test-Path -LiteralPath 'HKCU:\Software\Classes\xdzip.Archive')) { throw 'Unexpected pre-existing xdzip install or association.' }
    @('[General]', 'schemaVersion=1', '[locale]', 'language=en-US', '[issue53]', 'sentinel=owned-update-check') |
        Set-Content -LiteralPath $settings -Encoding utf8
    $env:XDZIP_SETTINGS_PATH = $settings
    $env:XDZIP_LANGUAGE = 'en-US'
    $script:stage = 'install-alpha8'
    $setupArguments = @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', ('/DIR=' + $installRoot))
    $installer = Start-Process -FilePath $setup -ArgumentList $setupArguments -Wait -PassThru -WindowStyle Hidden
    if ($installer.ExitCode -ne 0) { throw 'The reviewed alpha.8 candidate did not install.' }
    $app = Join-Path $installRoot 'xdzip.exe'
    if ((File-Hash $app) -ne $oldAppHash) { throw 'Installed alpha.8 executable differs.' }
    $identity = (& $app --version-json | Out-String | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0 -or $identity.version -ne '0.1.0-alpha.8' -or $identity.build -ne 21 -or
        -not ('e1cd6a3e125d5fcabc85a9f4edb106d013709c89'.StartsWith($identity.revision))) { throw 'Unexpected installed alpha.8 identity.' }
    'owned-user-file' | Set-Content -LiteralPath (Join-Path $installRoot 'owned-sentinel.txt') -Encoding ascii
    Record 'verified-alpha8-install'
    $script:stage = 'open-about'
    $process = Start-Process -FilePath $app -PassThru -WindowStyle Normal
    $script:oldProcessId = $process.Id
    Activate-Element (Wait-AppButton $process.Id '^Settings$' 60)
    Activate-Element (Wait-AppButton $process.Id '^About xdzip$' 30)
    Activate-Element (Wait-AppButton $process.Id '^Check for Updates$' 30)
    Record 'clicked-real-about-check'
    $script:stage = 'verify-and-download'
    $download = Wait-AppButton $process.Id '^Download and Install$' 150
    Record 'trusted-installed-candidate-available'
    Activate-Element $download
    $deadline = [DateTime]::UtcNow.AddSeconds(210)
    $window = $null
    $nextObservation = [DateTime]::UtcNow
    $observationCount = 0
    while ([DateTime]::UtcNow -lt $deadline) {
        $window = Wizard-Window
        if ($null -ne $window) { break }
        if (Complete-SetupLanguage) { Start-Sleep -Seconds 1; continue }
        if ([DateTime]::UtcNow -ge $nextObservation) {
            Observe-Update $process.Id ('after-click-' + $observationCount)
            $observationCount++
            $nextObservation = [DateTime]::UtcNow.AddMilliseconds(400)
            if ($script:lastSnapshot.name -match 'Unable to complete this update operation') { break }
        }
        Start-Sleep -Milliseconds 800
    }
    if ($null -eq $window) {
        Observe-Update $process.Id 'final'
        $helperErrors = @($script:observations | ForEach-Object { $_.cache_files } |
            Where-Object { $_.helper_error } | ForEach-Object { $_.helper_error })
        if ($helperErrors -contains 'unsafe_installation') {
            Record 'inherited-fixture-installation-rejected'
            Protect-OwnedInstallDirectory
            Record 'owned-fixture-directory-made-private'
            Activate-Element (Wait-AppButton $process.Id '^Check for Updates$' 15)
            $download = Wait-AppButton $process.Id '^Download and Install$' 150
            Activate-Element $download
            Record 'private-fixture-update-attempted'
            $deadline = [DateTime]::UtcNow.AddSeconds(210)
            $nextObservation = [DateTime]::UtcNow
            while ([DateTime]::UtcNow -lt $deadline) {
                $window = Wizard-Window
                if ($null -ne $window) { break }
                if (Complete-SetupLanguage) { Start-Sleep -Seconds 1; continue }
                if ([DateTime]::UtcNow -ge $nextObservation) {
                    Observe-Update $process.Id 'private-fixture'
                    $nextObservation = [DateTime]::UtcNow.AddMilliseconds(400)
                    if ($script:lastSnapshot.name -match 'Unable to complete this update operation') { break }
                }
                Start-Sleep -Milliseconds 500
            }
        }
        if ($null -eq $window) {
            Observe-Update $process.Id 'last-before-failure'
            Measure-PublicDownload
            throw 'Published update installer window did not appear.'
        }
    }
    $cacheRoot = Join-Path $env:LOCALAPPDATA 'xdzip'
    $cached = @(Get-ChildItem -LiteralPath $cacheRoot -Filter 'xdzip-0.1.0-alpha.9-windows-x86_64-setup.exe' -File -Recurse -ErrorAction SilentlyContinue)
    if ($cached.Count -lt 1 -or @($cached | Where-Object { (File-Hash $_.FullName) -ne $newSetupHash }).Count -ne 0) {
        throw 'Downloaded public installer identity differs.'
    }
    Record 'verified-public-alpha9-installer'
    $script:stage = 'interactive-installer'
    $deadline = [DateTime]::UtcNow.AddSeconds(200)
    while ([DateTime]::UtcNow -lt $deadline) {
        $window = Wizard-Window
        if ($null -ne $window) {
            $accept = Find-WizardControl $window ([System.Windows.Automation.ControlType]::RadioButton) '(?i)I accept'
            if ($null -ne $accept) { Activate-Element $accept $true; $script:licenseSeen = $true }
            $button = Find-WizardControl $window ([System.Windows.Automation.ControlType]::Button) '(?i)^\s*&?(Next|Install|Finish)'
            if ($null -ne $button) { Activate-Element $button; Start-Sleep -Seconds 2; continue }
        }
        if ((Test-Path -LiteralPath $app) -and (File-Hash $app) -eq $newAppHash -and $null -eq $window) { break }
        Start-Sleep -Milliseconds 800
    }
    if (-not $script:licenseSeen -or -not (Test-Path -LiteralPath $app) -or (File-Hash $app) -ne $newAppHash) {
        throw 'The original interactive installer did not complete with an observed license page.'
    }
    Record 'installer-license-page-and-upgrade-completed'
    $script:stage = 'verify-restart'
    $deadline = [DateTime]::UtcNow.AddSeconds(90)
    $newProcess = $null
    while ([DateTime]::UtcNow -lt $deadline) {
        $newProcess = @(Get-Process -Name xdzip -ErrorAction SilentlyContinue | Where-Object { $_.Id -ne $script:oldProcessId -and $_.Path -eq $app } | Select-Object -First 1)
        if ($newProcess.Count -eq 1) { break }
        Start-Sleep -Seconds 1
    }
    if ($newProcess.Count -ne 1) { throw 'Updated application did not restart.' }
    $newIdentity = (& $app --version-json | Out-String | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0 -or $newIdentity.version -ne '0.1.0-alpha.9' -or $newIdentity.build -ne 22 -or
        -not ('8b277ac418561b754cf16bd41adefe908285cca6'.StartsWith($newIdentity.revision))) { throw 'Unexpected updated application identity.' }
    Record 'updated-application-restarted'
    if ((Get-Content -LiteralPath $settings -Raw) -notmatch 'sentinel=owned-update-check' -or
        (Get-Content -LiteralPath (Join-Path $installRoot 'owned-sentinel.txt') -Raw).Trim() -ne 'owned-user-file') {
        throw 'Settings or unrelated installed-directory content was not preserved.'
    }
    Activate-Element (Wait-AppButton $newProcess[0].Id '^Settings$' 45)
    Activate-Element (Wait-AppTab $newProcess[0].Id '^About$' 30) $true
    Start-Sleep -Seconds 2
    Snapshot-Process $newProcess[0].Id
    $receiptAccessible = [bool]($script:lastSnapshot.name -match 'Update complete\. You are running the new version')
    if ($receiptAccessible) {
        Record 'restart-and-update-receipt-verified'
    } else {
        Record 'update-receipt-not-exposed-through-accessibility'
        try {
            Save-SettingsWindow $newProcess[0].Id
            Record 'captured-updated-about-window'
        } catch {
            $script:screenshotError = $_.Exception.Message.Replace($env:RUNNER_TEMP, '<runner-temp>').Replace($env:USERPROFILE, '<profile>').Replace((Get-Location).Path, '<workspace>')
            Record 'updated-about-window-capture-failed'
        }
    }
    $script:stage = 'payload'
    & python .github/issue216-public-update/verify-payload.py $installRoot $records
    if ($LASTEXITCODE -ne 0) { throw 'Updated payload does not match the published portable package.' }
    Record 'all-published-payload-files-verified'
    if (-not $receiptAccessible) {
        throw 'The restarted application did not expose its update receipt through accessibility.'
    }
    $script:outcome = 'passed'
} catch {
    $script:failure = $_.Exception.Message.Replace($env:RUNNER_TEMP, '<runner-temp>').Replace($env:USERPROFILE, '<profile>').Replace((Get-Location).Path, '<workspace>')
    Write-Error $script:failure -ErrorAction Continue
} finally {
    $result = [ordered]@{ status=$script:outcome; stage=$script:stage; events=@($script:events.ToArray());
        license_page_observed=$script:licenseSeen; platform='windows-2025 hosted runner'; failure=$script:failure;
        last_accessibility_snapshot=$script:lastSnapshot;
        observations=@($script:observations.ToArray()); independent_public_download=$script:downloadControl;
        screenshot_error=$script:screenshotError;
        install_directory_acl=@($script:installAcl.ToArray());
        limits=@('Disposable Windows Server runner, not Windows 11 human desktop acceptance.',
                 'The published update was not modified or re-signed; no production private key was used.',
                 'A controlled settings path was used; the default-profile experience remains a separate observation.') }
    $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $records 'public-update.json') -Encoding utf8
}
if ($script:outcome -ne 'passed') { exit 1 }
