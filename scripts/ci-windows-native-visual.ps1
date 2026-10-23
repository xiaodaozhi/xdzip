# scripts/ci-windows-native-visual.ps1

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
if (!$IsWindows -or $env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
    throw 'This entry is restricted to a disposable GitHub-hosted Windows runner'
}
function Invoke-Checked {
    param([string]$Program, [string[]]$Arguments)
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed with exit code $LASTEXITCODE" }
}
$revision = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $revision -ne $env:GITHUB_SHA -or (& git status --porcelain)) {
    throw 'A clean checkout of the workflow source is required'
}
$output = Join-Path (Get-Location) 'build/windows-native-visual'
New-Item -ItemType Directory -Path $output -ErrorAction Stop | Out-Null
$privateTemp = Join-Path $output 'private-temp'
New-Item -ItemType Directory -Path $privateTemp -ErrorAction Stop | Out-Null
# The safe-I/O tests require a non-inherited directory, including on hosted VMs.
$acl = [Security.AccessControl.DirectorySecurity]::new()
$acl.SetAccessRuleProtection($true, $false)
$inherit = [Security.AccessControl.InheritanceFlags]'ContainerInherit, ObjectInherit'
$propagation = [Security.AccessControl.PropagationFlags]::None
$ownerSid = [Security.Principal.WindowsIdentity]::GetCurrent().User
$acl.SetOwner($ownerSid)
foreach ($sid in @($ownerSid, [Security.Principal.SecurityIdentifier]::new('S-1-5-18'), [Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'))) {
    $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new(
        $sid, [Security.AccessControl.FileSystemRights]::FullControl, $inherit, $propagation,
        [Security.AccessControl.AccessControlType]::Allow))
}
Set-Acl -LiteralPath $privateTemp -AclObject $acl
$env:TEMP = $privateTemp
$env:TMP = $privateTemp
$env:PYTHONUTF8 = '1'
$vswhere = "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
$installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (!$installation) { throw 'MSVC installation not found' }
Import-Module "$installation/Common7/Tools/Microsoft.VisualStudio.DevShell.dll"
Enter-VsDevShell -VsInstallPath $installation -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
[ordered]@{
    schema_version = 1
    revision = $revision
    compiler_version = (Get-Item (Get-Command cl.exe -ErrorAction Stop).Source).VersionInfo.ProductVersion
    vc_tools_version = $env:VCToolsVersion
    windows_sdk_version = $env:WindowsSDKVersion.Trim('\/')
    runner_environment = $env:RUNNER_ENVIRONMENT
    runner_image = $env:ImageVersion
    hosted_license_reference = 'https://github.com/actions/runner-images/issues/12945#issuecomment-3348002951'
    limits = @('Tool identity is recorded; this is not automatic certification of license eligibility.')
} | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $output 'toolchain-identity.json') -Encoding utf8
$qtRoot = Join-Path $env:RUNNER_TEMP 'Qt'
$qtPrefix = Join-Path $qtRoot '6.8.2/msvc2022_64'
$tools = Join-Path $env:RUNNER_TEMP 'xdzip-native-visual-tools'
$shared = @('--qt-prefix', $qtPrefix, '--qt-root', $qtRoot,
    '--build-dir', 'build/native-visual-desktop', '--output-dir', 'build/windows-native-visual/build', '--tools-dir', $tools)
Invoke-Checked python (@('scripts/desktop.py', 'prepare', '--install-qt', '--install-rust') + $shared)
Invoke-Checked python (@('scripts/desktop.py', 'build') + $shared)
$env:PATH = "$qtPrefix/bin$([IO.Path]::PathSeparator)$env:PATH"
$tests = 'build/native-visual-desktop/apps/desktop/xdzip-browser-tests.exe'
$binaryHash = (Get-FileHash -LiteralPath $tests -Algorithm SHA256).Hash.ToLowerInvariant()
$env:QT_QPA_PLATFORM = 'windows'
$env:QSG_RHI_BACKEND = 'd3d11'
$env:QSG_INFO = '1'
foreach ($name in @('QT_QUICK_BACKEND', 'QT_SCALE_FACTOR', 'QT_SCREEN_SCALE_FACTORS', 'QT_FONT_DPI')) {
    Remove-Item -LiteralPath "Env:$name" -ErrorAction SilentlyContinue
}
$common = @('--tests', $tests, '--source', $revision, '--binary-sha256', $binaryHash)
$displayDirectory = 'build/windows-native-visual/display'
$displayReceipt = Join-Path (Get-Location) "$displayDirectory/receipt.json"
$prepareExit = $null
$restoreExit = $null
$menuExit = $null
$visualExit = $null
$layoutExit = $null
try {
    & python 'scripts/windows_native_ci_display.py' '--prepare' '--output' $displayDirectory '--source' $revision
    $prepareExit = $LASTEXITCODE
    if ($prepareExit -eq 0) {
        $env:XDZIP_NATIVE_CI_DISPLAY_RECEIPT = $displayReceipt
        # Both diagnostics retain their failures; one blocked route does not hide the other.
        & python 'scripts/check-windows-native-ci.py' @common '--output' 'build/windows-native-visual/menus'
        $menuExit = $LASTEXITCODE
        & python 'scripts/check-windows-native-visual-ci.py' @common '--output' 'build/windows-native-visual/scenes'
        $visualExit = $LASTEXITCODE
        & python 'scripts/check-windows-native-layout-ci.py' @common '--output' 'build/windows-native-visual/layout'
        $layoutExit = $LASTEXITCODE
    }
} finally {
    if (Test-Path -LiteralPath $displayReceipt) {
        & python 'scripts/windows_native_ci_display.py' '--restore' '--output' $displayDirectory '--source' $revision
        $restoreExit = $LASTEXITCODE
    }
    Remove-Item -LiteralPath 'Env:XDZIP_NATIVE_CI_DISPLAY_RECEIPT' -ErrorAction SilentlyContinue
}
$unchanged = $revision -eq (& git rev-parse HEAD).Trim() -and !(& git status --porcelain) -and
    $binaryHash -eq (Get-FileHash -LiteralPath $tests -Algorithm SHA256).Hash.ToLowerInvariant()
[ordered]@{
    schema_version = 1
    source_sha = $revision
    workflow_run_id = $env:GITHUB_RUN_ID
    workflow_attempt = $env:GITHUB_RUN_ATTEMPT
    runner_image = $env:ImageVersion
    test_binary_sha256 = $binaryHash
    virtual_display_prepare_exit_code = $prepareExit
    virtual_display_restore_exit_code = $restoreExit
    menu_exit_code = $menuExit
    visual_exit_code = $visualExit
    layout_exit_code = $layoutExit
    source_and_binary_unchanged = $unchanged
    real_user_click = $false
    fixed_reference_performance_acceptance = $false
    physical_dual_monitor_acceptance = $false
    windows_11_decoration_acceptance = $false
    release_authorized = $false
} | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $output 'workflow-result.json') -Encoding utf8
if ($prepareExit -ne 0 -or $restoreExit -ne 0 -or $menuExit -ne 0 -or $visualExit -ne 0 -or $layoutExit -ne 0 -or !$unchanged) {
    throw 'Native CI diagnostic did not pass; inspect the preserved reports and logs'
}
