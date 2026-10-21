# CI extension: shared local stages plus historical, visual and installer gates.
param([string]$QtRoot, [string]$ToolsDirectory)
$ErrorActionPreference = 'Stop'
function Invoke-Checked {
    param([string]$Program, [string[]]$Arguments)
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed with exit code $LASTEXITCODE" }
    Assert-CleanSources
}
function Assert-CleanSources {
    $changes = & git status --porcelain --untracked-files=normal
    if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect build source identity' }
    if ($changes) {
        $changes | Write-Host
        throw 'Installer build requires clean source; relative changed paths shown above'
    }
}
# RUNNER_TEMP is an optional CI cache location, never a local prerequisite.
$cacheRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { Join-Path (Get-Location) '.cache' }
$qtRoot = if ($QtRoot) { $QtRoot } else { Join-Path $cacheRoot 'Qt' }
$qtEnvironment = if ($ToolsDirectory) { $ToolsDirectory } else { Join-Path $cacheRoot 'xdzip-desktop-tools' }
$qtHost = if ($IsMacOS) { 'mac' } elseif ($IsWindows) { 'windows' } else { 'linux' }
$qtFolder = if ($IsMacOS) { 'macos' } elseif ($IsWindows) { 'msvc2022_64' } else { 'gcc_64' }
if ($IsWindows) {
    $vswhere = "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (!$installation) { throw 'MSVC installation not found' }
    Import-Module "$installation/Common7/Tools/Microsoft.VisualStudio.DevShell.dll"
    Enter-VsDevShell -VsInstallPath $installation -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
}
$qtPrefix = Join-Path $qtRoot "6.8.2/$qtFolder"
$shared = @('--qt-prefix', $qtPrefix, '--qt-root', $qtRoot,
    '--build-dir', 'build/desktop', '--output-dir', 'build/review', '--tools-dir', $qtEnvironment)
# Fail on PATH/compiler/SDK issues before downloads, compilation or license collection.
Invoke-Checked python (@('scripts/desktop.py', 'prepare', '--install-qt', '--install-rust', '--packaging') + $shared)
$aqtPython = if ($IsWindows) { Join-Path $qtEnvironment 'Scripts/python.exe' } else { Join-Path $qtEnvironment 'bin/python' }
Invoke-Checked python @('scripts/sync-version.py', '--check')
Invoke-Checked python @('scripts/check-ui-i18n.py')
Invoke-Checked python @('scripts/check-window-contracts.py','--self-test','--output','build/review/window-contract-audit.json')
Invoke-Checked python @('scripts/check-operation-components.py')
Assert-CleanSources
Invoke-Checked python @('scripts/test-installers.py')
Invoke-Checked python @('scripts/test-source-delivery.py')
Invoke-Checked python @('scripts/test-qt-sdk-correspondence.py')
Invoke-Checked python @('scripts/test-deployment-audit.py')
Invoke-Checked python @('scripts/test-linux-deployment.py')
Invoke-Checked python @('scripts/test-windows-deployment.py')
Invoke-Checked python @('scripts/test-runtime-attribution.py')
if ($IsWindows) {
    Invoke-Checked python @('scripts/test-windows-installer-native.py')
}
# Source bytes and installed configuration are evidence, not a Qt rebuild claim.
Invoke-Checked python @('scripts/qt_sdk_correspondence.py', '--qt-prefix', $qtPrefix, '--fetch', '--output', 'build/review/qt-sdk-correspondence.json')
$qtBin = Join-Path $qtPrefix 'bin'
$env:PATH = "$qtBin$([IO.Path]::PathSeparator)$env:PATH"
if ($IsLinux) { $env:LD_LIBRARY_PATH = Join-Path $qtPrefix 'lib' }
$env:QT_QPA_PLATFORM = 'offscreen'
$env:QT_QUICK_BACKEND = 'software'
$env:XDZIP_CI_POWERSHELL = (Get-Process -Id $PID).Path
Invoke-Checked python @('scripts/check-settings-compat.py', '--qt-prefix', $qtPrefix, '--output', 'build/settings-compat')
New-Item -ItemType Directory -Force 'build/review' | Out-Null
Copy-Item 'build/settings-compat/settings-compat.json' 'build/review/settings-compat.json'
Invoke-Checked python (@('scripts/desktop.py', 'build') + $shared)
$productExe = if ($IsWindows) { 'build/desktop/apps/desktop/xdzip.exe' } elseif ($IsMacOS) { 'build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip' } else { 'build/desktop/apps/desktop/xdzip' }
if ($IsWindows) {
    Invoke-Checked python @('scripts/check-windows-subsystem.py', '--gui', $productExe, '--console', 'build/desktop/apps/desktop/xdzip-browser-tests.exe')
}
if ($IsMacOS) {
    Invoke-Checked python @('scripts/check-macos-localization.py', 'build/desktop/apps/desktop/xdzip.app/Contents/Info.plist')
    Invoke-Checked python @('scripts/check-macos-registration.py', '--app', 'build/desktop/apps/desktop/xdzip.app', '--fixture', 'tests/fixtures/zip/normal.zip', '--output', 'build/review/macos-registration.json')
}
# Foundation independently executes locked workspace Rust tests.
$browserTests = if ($IsWindows) { 'build/desktop/apps/desktop/xdzip-browser-tests.exe' } else { 'build/desktop/apps/desktop/xdzip-browser-tests' }
Invoke-Checked python @('scripts/check-native-sessions.py', '--tests', $browserTests, '--output', 'build/review/native-sessions.json')
Invoke-Checked python (@('scripts/desktop.py', 'test', '--skip-rust') + $shared)
# Repeat only the concurrent lifecycle suite to expose nondeterministic races.
Invoke-Checked ctest @('--test-dir', 'build/desktop', '-R', 'qt_zip_browser', '--repeat', 'until-fail:10', '--output-on-failure')
New-Item -ItemType Directory -Force 'build/review' | Out-Null
Invoke-Checked python @('scripts/check-build-version.py', '--app', $productExe, '--output', 'build/review/build-version.json')
$metricsExe = if ($IsWindows) { 'build/desktop/tools/process-metrics/xdzip-process-metrics.exe' } else { 'build/desktop/tools/process-metrics/xdzip-process-metrics' }
Invoke-Checked python @('scripts/test-performance.py','--metrics',$metricsExe)
$env:XDZIP_PROBE_RESOURCE_MANIFEST = '1'
# One offscreen run checks the measurement protocol; this is not a desktop performance budget.
Invoke-Checked python @('scripts/measure-desktop.py', '--app', $productExe, '--fixture', 'build/desktop/apps/desktop/m2-flat.zip', '--output', 'build/review/desktop-performance.json', '--metrics', $(if ($IsWindows) { 'build/desktop/tools/process-metrics/xdzip-process-metrics.exe' } else { 'build/desktop/tools/process-metrics/xdzip-process-metrics' }), '--runs', '1')

Remove-Item Env:XDZIP_PROBE_RESOURCE_MANIFEST
Invoke-Checked python @('scripts/check-runtime-resources.py','--probe','build/review/desktop-performance.json','--output','build/review/runtime-resources.json')
$rustStatic = (Get-ChildItem 'build/desktop/apps/desktop/rust' -Recurse -File | Where-Object { $_.Name -in @('libxdzip_qt_bridge.a','xdzip_qt_bridge.lib') } | Select-Object -First 1).FullName
Invoke-Checked python @('scripts/linkage-evidence.py','--app',$productExe,'--map','build/desktop/apps/desktop/xdzip-link.map','--rust-library',$rustStatic,'--output','build/review/linkage-evidence.json')
# Record synthetic IME/accessibility coverage; offscreen does not imply native desktop approval.
$env:XDZIP_PRIVATE_TEST_AUDIT = Join-Path (Get-Location) 'build/review'
$desktopProbe = if ($IsWindows) { 'build/desktop/apps/desktop/xdzip-browser-tests.exe' } else { 'build/desktop/apps/desktop/xdzip-browser-tests' }
Invoke-Checked $desktopProbe @('privateTestDesktopInputAndAccessibility')
Remove-Item Env:XDZIP_PRIVATE_TEST_AUDIT
$tabLanguage = $env:XDZIP_UI_TEST_LANGUAGE
$env:XDZIP_TABS_VISUAL_AUDIT = Join-Path (Get-Location) 'build/review/tab-strip-native'
foreach ($language in @('zh-CN','en-US')) {
    $env:XDZIP_UI_TEST_LANGUAGE = $language
    Invoke-Checked $browserTests @('sessionTabReorderingPreservesControllers')
}
if ($tabLanguage) { $env:XDZIP_UI_TEST_LANGUAGE = $tabLanguage } else { Remove-Item Env:XDZIP_UI_TEST_LANGUAGE }
Remove-Item Env:XDZIP_TABS_VISUAL_AUDIT
# Gate the repaired geometry and palette contracts on each native host.
# Unresolved font evidence remains explicitly unverified; this is not golden-image approval.
$env:XDZIP_VISUAL_AUDIT = Join-Path (Get-Location) 'build/desktop/visual-audit'
Invoke-Checked $browserTests @('treeExpansionAndColumnInteraction', 'realContextMenusAndGrid', 'mainWindowFidelityAndSessionClose', 'reviewedMainControlsAndMenus', 'rootListingAndReviewedInteractions', 'm2LargeArchiveLatestQueryAndSourceChange', 'renderRealArchive')
Remove-Item Env:XDZIP_VISUAL_AUDIT
$env:XDZIP_WELCOME_VISUAL_AUDIT = Join-Path (Get-Location) 'build/desktop/welcome'
Invoke-Checked $browserTests @('welcomeOpensRealArchivesAndReturns')
Invoke-Checked python @('scripts/check-qt-welcome.py', '--input', 'build/desktop/welcome', '--output', 'build/review/welcome.json')
Remove-Item Env:XDZIP_WELCOME_VISUAL_AUDIT
if (!$IsMacOS) { Copy-Item -Recurse 'build/desktop/welcome' 'build/review/welcome' }
$env:XDZIP_M3_VISUAL_AUDIT = Join-Path (Get-Location) 'build/desktop/m3-operations'
Invoke-Checked $browserTests @('m3ExtractionWindowsAndRealOutput', 'm3IntegrityFailurePreservesDestinationAndCancelEndsWaiting')
Remove-Item Env:XDZIP_M3_VISUAL_AUDIT
Copy-Item -Recurse 'build/desktop/m3-operations' 'build/review/m3-operations'
Invoke-Checked python @('scripts/check-m3-operation-windows.py', '--input', 'build/desktop/m3-operations', '--output', 'build/review/m3-operation-ui.json')
$env:XDZIP_RESULT_AUDIT = Join-Path (Get-Location) 'build/desktop/result-ui'
Invoke-Checked $browserTests @('partialResultUsesActualOutcomes')
Remove-Item Env:XDZIP_RESULT_AUDIT
Invoke-Checked python @('scripts/check-result-ui.py', '--input', 'build/desktop/result-ui', '--output', 'build/review/result-ui.json')
if (!$IsMacOS) { Copy-Item -Recurse 'build/desktop/result-ui' 'build/review/result-ui' }
Invoke-Checked cargo @('build', '--release', '--locked', '-p', 'xdzip-m3-probe')
$m3Probe = if ($IsWindows) { 'target/release/xdzip-m3-probe.exe' } else { 'target/release/xdzip-m3-probe' }
Invoke-Checked python @('scripts/measure-m3-core.py', '--binary', $m3Probe, '--output', 'build/review/m3-core-performance.json')
$env:XDZIP_M4_VISUAL_AUDIT = Join-Path (Get-Location) 'build/desktop/m4-operations'
Invoke-Checked $browserTests @('m4CreationWindowAndRoundTrip', 'tarCreationBrowseAndSafeExtraction')
Remove-Item Env:XDZIP_M4_VISUAL_AUDIT
Invoke-Checked python @('scripts/check-operation-components.py')
$env:XDZIP_CREATE_CONTROLS_AUDIT = Join-Path (Get-Location) 'build/desktop/create-controls'
Invoke-Checked $browserTests @('creationControlsMatchReviewedSource')
Invoke-Checked python @('scripts/check-create-controls.py', '--input', 'build/desktop/create-controls', '--output', 'build/review/create-controls.json')
Copy-Item -Recurse 'build/desktop/create-controls' 'build/review/create-controls'
Remove-Item Env:XDZIP_CREATE_CONTROLS_AUDIT
$env:XDZIP_OPENING_VISUAL_AUDIT = Join-Path (Get-Location) 'build/desktop/temporary-opening'
Invoke-Checked $browserTests @('temporaryFileOpeningUsesAssociationAndHidesOnlyAfterAcceptance', 'terminalTemporaryOpenBeforeAcceptanceDoesNotReopenProgress')
Copy-Item -Recurse 'build/desktop/temporary-opening' 'build/review/temporary-opening'
Remove-Item Env:XDZIP_OPENING_VISUAL_AUDIT
Copy-Item -Recurse 'build/desktop/m4-operations' 'build/review/m4-operations'
Invoke-Checked python @('scripts/check-m4-operation-windows.py', '--input', 'build/desktop/m4-operations', '--output', 'build/review/m4-operation-ui.json')
Invoke-Checked cargo @('build', '--release', '--locked', '-p', 'xdzip-m4-probe')
$m4Probe = if ($IsWindows) { 'target/release/xdzip-m4-probe.exe' } else { 'target/release/xdzip-m4-probe' }
if ($IsLinux) {
    Invoke-Checked python @('scripts/check-advanced-creation.py', '--binary', $m4Probe, '--extract-binary', $m3Probe, '--output', 'build/review/advanced-creation.json')
}
Invoke-Checked python @('scripts/check-tar-interop.py', '--create-binary', $m4Probe, '--extract-binary', $m3Probe, '--output', 'build/review/tar-interop.json')
Invoke-Checked python @('scripts/measure-m4-core.py', '--binary', $m4Probe, '--output', 'build/review/m4-core-performance.json')
Invoke-Checked python @('scripts/measure-m2-qt.py', '--binary', $browserTests, '--output', 'build/review/m2-qt-performance.json')
Invoke-Checked python @('scripts/generate-m2-fixture.py', '--output', 'build/desktop/m2-shuffled.zip', '--shuffle')
Invoke-Checked cargo @('build', '--release', '-p', 'xdzip-m2-probe')
$m2Probe = if ($IsWindows) { 'target/release/xdzip-m2-probe.exe' } else { 'target/release/xdzip-m2-probe' }
$m2Report = & $m2Probe 'build/desktop/m2-shuffled.zip'
if ($LASTEXITCODE -ne 0) { throw 'M2 core benchmark failed' }
$m2Report | Set-Content 'build/review/m2-core-performance.json'
Invoke-Checked python @('scripts/build-visual-audit.py', '--input', 'build/desktop/visual-audit', '--output', 'build/review/visual-audit', '--print-checks', '--fail-on-differences', '--require-navigation')
Invoke-Checked python @('scripts/check-qt-fidelity.py', '--input', 'build/desktop/visual-audit', '--output', 'build/review/visual-audit/fidelity.json')
# Fractional and Retina DPR change capture pixels, never the 1024-wide logical UI.
foreach ($density in @('1.25', '2')) {
    $env:QT_SCALE_FACTOR = $density
    $welcomePath = "build/desktop/welcome-dpr$density"
    $env:XDZIP_WELCOME_VISUAL_AUDIT = Join-Path (Get-Location) $welcomePath
    Invoke-Checked $browserTests @('welcomeOpensRealArchivesAndReturns')
    Invoke-Checked python @('scripts/check-qt-welcome.py', '--input', $welcomePath, '--output', "build/review/welcome-dpr$density.json")
    Remove-Item Env:XDZIP_WELCOME_VISUAL_AUDIT
    if (!$IsMacOS) { Copy-Item -Recurse $welcomePath "build/review/welcome-dpr$density" }
    Invoke-Checked ctest @('--test-dir', 'build/desktop', '-R', 'qt_task_components', '--output-on-failure')
    $capturePath = "build/desktop/visual-audit-dpr$density"
    $env:XDZIP_VISUAL_AUDIT = Join-Path (Get-Location) $capturePath
    Invoke-Checked $browserTests @('mainWindowFidelityAndSessionClose', 'rootListingAndReviewedInteractions', 'reviewedMainControlsAndMenus')
    Invoke-Checked python @('scripts/check-qt-fidelity.py', '--input', $capturePath, '--output', "build/review/fidelity-dpr$density.json")
    Copy-Item -Recurse $capturePath "build/review/visual-audit-dpr$density"
    $operationPath = "build/desktop/m3-operations-dpr$density"
    $env:XDZIP_M3_VISUAL_AUDIT = Join-Path (Get-Location) $operationPath
    Invoke-Checked $browserTests @('m3ExtractionWindowsAndRealOutput', 'm3IntegrityFailurePreservesDestinationAndCancelEndsWaiting')
    Invoke-Checked python @('scripts/check-m3-operation-windows.py', '--input', $operationPath, '--output', "build/review/m3-operation-ui-dpr$density.json")
    Remove-Item Env:XDZIP_M3_VISUAL_AUDIT
    $creationPath = "build/desktop/m4-operations-dpr$density"
    $env:XDZIP_M4_VISUAL_AUDIT = Join-Path (Get-Location) $creationPath
    Invoke-Checked $browserTests @('m4CreationWindowAndRoundTrip', 'tarCreationBrowseAndSafeExtraction')
    Invoke-Checked python @('scripts/check-m4-operation-windows.py', '--input', $creationPath, '--output', "build/review/m4-operation-ui-dpr$density.json")
    Remove-Item Env:XDZIP_M4_VISUAL_AUDIT
    $stylePath = "build/desktop/create-controls-dpr$density"
    $env:XDZIP_CREATE_CONTROLS_AUDIT = Join-Path (Get-Location) $stylePath
    Invoke-Checked $browserTests @('creationControlsMatchReviewedSource')
    Invoke-Checked python @('scripts/check-create-controls.py', '--input', $stylePath, '--output', "build/review/create-controls-dpr$density.json")
    Remove-Item Env:XDZIP_CREATE_CONTROLS_AUDIT
}
Remove-Item Env:QT_SCALE_FACTOR
Remove-Item Env:XDZIP_VISUAL_AUDIT
Copy-Item 'build/desktop/apps/desktop/*.png' 'build/review'
Copy-Item 'build/desktop/Testing/Temporary/LastTest.log' 'build/review'
if ($IsMacOS) {
    Invoke-Checked python @('scripts/collect-notices.py', 'build/review/notices')
    Invoke-Checked python @('scripts/prepare-runtime-notices.py', '--qt-prefix', $qtPrefix, '--output', 'build/review/notices/Qt-runtime')
    Copy-Item 'build/review/notices/source-build.cdx.json' 'build/review/source-build.cdx.json'
} else {
    Invoke-Checked python (@('scripts/desktop.py', 'package', '--kind', 'runtime') + $shared)
    Copy-Item 'docs/engineering/developer-build.md' 'build/review/README.md'
    Invoke-Checked git @('archive', '--format=tar', '-o', 'build/review/xdzip-source.tar', 'HEAD')
    Invoke-Checked python @('scripts/prepare-previous-product.py', '--qt-prefix', $qtPrefix, '--output', 'build/previous-product', '--notices', 'build/review/notices')
    Copy-Item 'build/previous-product/previous-product.json' 'build/review/previous-product.json'
}
if ($IsMacOS) {
    # Validate the native executable using the installed SDK. Do not deploy Qt,
    # sign, archive or upload an Apple Silicon developer application package.
    $env:QT_PLUGIN_PATH = Join-Path $qtPrefix 'plugins'
    $env:QML_IMPORT_PATH = Join-Path $qtPrefix 'qml'
    $env:QT_QPA_PLATFORM = 'cocoa'
    $nativeLanguage = $env:XDZIP_LANGUAGE
    try {
        foreach ($language in @('zh-CN', 'en-US')) {
            $env:XDZIP_LANGUAGE = $language
            Invoke-Checked 'build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip' @('--smoke-test')
        }
    } finally { $env:XDZIP_LANGUAGE = $nativeLanguage }
    Invoke-Checked python @('scripts/check-macos-open.py', '--app', 'build/desktop/apps/desktop/xdzip.app', '--fixture', 'build/desktop/apps/desktop/m2-flat.zip', '--output', 'build/review/macos-association.json')
} elseif ($IsWindows) {
    $env:QT_QPA_PLATFORM = 'windows'
    Invoke-Checked $browserTests @('windowsCaptionControlsAndWindowState')
    Invoke-Checked python (@('scripts/desktop.py', 'verify', '--kind', 'runtime') + $shared)
} else {
    Invoke-Checked python (@('scripts/desktop.py', 'verify', '--kind', 'runtime') + $shared)
}
$auditRoot = if ($IsMacOS) { 'build/desktop/apps/desktop/xdzip.app' } elseif ($IsWindows) { 'build/review/windows' } else { 'build/review/linux' }
$auditExe = if ($IsMacOS) { $productExe } elseif ($IsWindows) { 'build/review/windows/xdzip.exe' } else { 'build/review/linux/bin/xdzip' }
$auditMode = if ($IsMacOS) { 'sdk' } else { 'deployed' }
if ($IsMacOS) {
    Invoke-Checked python @('scripts/deployment_audit.py', '--root', $auditRoot, '--app', $auditExe, '--qt-prefix', $qtPrefix, '--mode', $auditMode, '--output', 'build/review/deployment-audit.json')
    Invoke-Checked python @('scripts/native-redistribution.py','--root',$auditRoot,'--audit','build/review/deployment-audit.json','--output','build/review/native-redistribution.json')
    $attributionArgs = @('scripts/runtime_attribution.py', '--audit', 'build/review/deployment-audit.json', '--root', $auditRoot, '--qt-prefix', $qtPrefix, '--output', 'build/review/runtime-attribution.json')
    if ($IsLinux) { $attributionArgs += @('--deployment', 'build/review/linux-deployment.json') }
    Invoke-Checked python $attributionArgs
    Invoke-Checked python @('scripts/classify-qt-binary-differences.py','--attribution','build/review/runtime-attribution.json','--qt-prefix',$qtPrefix,'--output','build/review/qt-binary-differences.json')
    Invoke-Checked python @('scripts/sbom.py', '--audit', 'build/review/deployment-audit.json', '--source', 'build/review/source-build.cdx.json', '--output', 'build/review/artifact-files.cdx.json', '--attribution', 'build/review/runtime-attribution.json')
    Invoke-Checked $aqtPython @('scripts/validate-sbom.py', 'build/review/source-build.cdx.json', 'build/review/artifact-files.cdx.json')
} else {
    Invoke-Checked python (@('scripts/desktop.py', 'package', '--kind', 'installer') + $shared)
}
if ($IsLinux) {
    New-Item -ItemType Directory -Force 'build/installer-tests/linux-packages' | Out-Null
    Copy-Item 'build/review/xdzip-ubuntu24.04-amd64.deb' 'build/installer-tests/linux-packages/product.deb'
    Copy-Item 'build/previous-product/previous.deb' 'build/installer-tests/linux-packages/previous.deb'
    foreach ($variant in @('1','2')) {
        Invoke-Checked python @('scripts/build-linux-installer.py', '--root', $auditRoot, '--audit', 'build/review/deployment-audit.json', '--output', "build/installer-tests/linux-packages/qa$variant.deb", '--report', "build/installer-tests/linux-packages/qa$variant.json", '--qa-variant', $variant)
    }
    Invoke-Checked python @('scripts/build-linux-installer.py', '--root', $auditRoot, '--audit', 'build/review/deployment-audit.json', '--output', 'build/installer-tests/linux-packages/qa-failure.deb', '--report', 'build/installer-tests/linux-packages/qa-failure.json', '--qa-variant', '2', '--qa-failure', 'postinst-once')
    Invoke-Checked python @('scripts/verify-linux-installer.py', '--packages', 'build/installer-tests/linux-packages', '--fixture', 'build/desktop/apps/desktop/m2-flat.zip', '--output', 'build/review/linux-install-lifecycle.json')
}
if ($IsWindows) {
    New-Item -ItemType Directory -Force 'build/installer-tests/windows-packages' | Out-Null
    $installerBase = @('scripts/build-windows-installer.py','--root',$auditRoot,'--audit','build/review/deployment-audit.json','--iscc','build/installer-toolchain/compiler/ISCC.exe','--chinese','build/installer-toolchain/ChineseSimplified.isl')
    Copy-Item 'build/review/xdzip-windows-x64-setup.exe' 'build/installer-tests/windows-packages/product.exe'
    Copy-Item 'build/previous-product/previous.exe' 'build/installer-tests/windows-packages/previous.exe'
    foreach ($variant in @('1','2')) {
        Invoke-Checked python ($installerBase + @('--output',"build/installer-tests/windows-packages/qa$variant.exe",'--report',"build/installer-tests/windows-packages/qa$variant.json",'--qa-variant',$variant))
    }
    Invoke-Checked pwsh @('-File','scripts/check-windows-install.ps1','-Packages','build/installer-tests/windows-packages','-Fixture','build/desktop/apps/desktop/m2-flat.zip','-Output','build/review/windows-install-lifecycle.json')
}
Invoke-Checked python @('scripts/prepare-private-test.py', '--output', 'build/review/manual-desktop.json')
$commit = & git rev-parse HEAD
$distribution = if ($IsMacOS) {
    'Native macOS SDK validation only; no Apple Silicon developer package'
} else {
    'M4 ZIP Store/Deflate creation and safe extraction developer review; independent security audit and signed public release pending'
}
@{ commit = $commit; platform = $qtHost; dependencyAuditMode = $auditMode; architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString(); qt = '6.8.2'; rust = '1.99.0'; packaged = !$IsMacOS; distribution = $distribution } |
    ConvertTo-Json | Set-Content 'build/review/build.json'
