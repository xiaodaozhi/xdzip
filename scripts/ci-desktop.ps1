# Native M3 product build, lifecycle tests and developer-review artifacts.
$ErrorActionPreference = 'Stop'
function Invoke-Checked {
    param([string]$Program, [string[]]$Arguments)
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed with exit code $LASTEXITCODE" }
}
$qtRoot = Join-Path $env:RUNNER_TEMP 'Qt'
$qtHost = 'linux'
$qtArchitecture = 'linux_gcc_64'
$qtFolder = 'gcc_64'
if ($IsMacOS) {
    $qtHost = 'mac'
    $qtArchitecture = 'clang_64'
    $qtFolder = 'macos'
    $env:MACOSX_DEPLOYMENT_TARGET = '14.0'
    if ((& uname -m) -ne 'arm64') { throw 'Apple Silicon gate requires a native arm64 runner' }
}
if ($IsWindows) {
    $qtHost = 'windows'
    $qtArchitecture = 'win64_msvc2022_64'
    $qtFolder = 'msvc2022_64'
    $vswhere = "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (!$installation) { throw 'MSVC installation not found' }
    Import-Module "$installation/Common7/Tools/Microsoft.VisualStudio.DevShell.dll"
    Enter-VsDevShell -VsInstallPath $installation -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
}
$qtEnvironment = Join-Path $env:RUNNER_TEMP 'xdzip-aqt'
Invoke-Checked python @('-m', 'venv', $qtEnvironment)
$aqtPython = if ($IsWindows) { Join-Path $qtEnvironment 'Scripts/python.exe' } else { Join-Path $qtEnvironment 'bin/python' }
Invoke-Checked $aqtPython @('-m', 'pip', 'install', 'aqtinstall==3.3.0')
Invoke-Checked $aqtPython @('-m', 'aqt', 'install-qt', $qtHost, 'desktop', '6.8.2', $qtArchitecture, '-O', $qtRoot)
$qtPrefix = Join-Path $qtRoot "6.8.2/$qtFolder"
$qtBin = Join-Path $qtPrefix 'bin'
$env:PATH = "$qtBin$([IO.Path]::PathSeparator)$env:PATH"
if ($IsLinux) { $env:LD_LIBRARY_PATH = Join-Path $qtPrefix 'lib' }
$env:QT_QPA_PLATFORM = 'offscreen'
$env:QT_QUICK_BACKEND = 'software'
$env:XDZIP_CI_POWERSHELL = (Get-Process -Id $PID).Path
Invoke-Checked rustup @('toolchain', 'install', '1.99.0', '--profile', 'minimal', '--component', 'rustfmt', '--component', 'clippy')
Invoke-Checked rustc @('-vV')
$cmakeArgs = @('-S', '.', '-B', 'build/desktop', '-G', 'Ninja', '-DCMAKE_BUILD_TYPE=Release', '-DXDZIP_BUILD_DESKTOP_PREVIEW=ON', "-DCMAKE_PREFIX_PATH=$qtPrefix")
if ($IsMacOS) { $cmakeArgs += @('-DCMAKE_OSX_ARCHITECTURES=arm64', '-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0') }
Invoke-Checked cmake $cmakeArgs
Invoke-Checked cmake @('--build', 'build/desktop', '--parallel', '3')
Invoke-Checked qmllint (@('-I', 'build/desktop/qml') + (Get-ChildItem 'apps/desktop/ui/*.qml').FullName)
Invoke-Checked ctest @('--test-dir', 'build/desktop', '--output-on-failure')
# Repeat only the concurrent lifecycle suite to expose nondeterministic races.
Invoke-Checked ctest @('--test-dir', 'build/desktop', '-R', 'qt_zip_browser', '--repeat', 'until-fail:10', '--output-on-failure')
New-Item -ItemType Directory -Force 'dist/review' | Out-Null
# Gate the repaired geometry and palette contracts on each native host.
# Unresolved font evidence remains explicitly unverified; this is not golden-image approval.
$env:XDZIP_VISUAL_AUDIT = Join-Path (Get-Location) 'build/desktop/visual-audit'
$browserTests = if ($IsWindows) { 'build/desktop/apps/desktop/xdzip-browser-tests.exe' } else { 'build/desktop/apps/desktop/xdzip-browser-tests' }
Invoke-Checked $browserTests @('treeExpansionAndColumnInteraction', 'realContextMenusAndGrid', 'mainWindowFidelityAndSessionClose', 'reviewedMainControlsAndMenus', 'rootListingAndReviewedInteractions', 'm2LargeArchiveLatestQueryAndSourceChange', 'renderRealArchive')
Remove-Item Env:XDZIP_VISUAL_AUDIT
$env:XDZIP_M3_VISUAL_AUDIT = Join-Path (Get-Location) 'build/desktop/m3-operations'
Invoke-Checked $browserTests @('m3ExtractionWindowsAndRealOutput', 'm3IntegrityFailurePreservesDestinationAndCancelEndsWaiting')
Remove-Item Env:XDZIP_M3_VISUAL_AUDIT
Copy-Item -Recurse 'build/desktop/m3-operations' 'dist/review/m3-operations'
Invoke-Checked cargo @('build', '--release', '--locked', '-p', 'xdzip-m3-probe')
$m3Probe = if ($IsWindows) { 'target/release/xdzip-m3-probe.exe' } else { 'target/release/xdzip-m3-probe' }
Invoke-Checked python @('scripts/measure-m3-core.py', '--binary', $m3Probe, '--output', 'dist/review/m3-core-performance.json')
Invoke-Checked python @('scripts/measure-m2-qt.py', '--binary', $browserTests, '--output', 'dist/review/m2-qt-performance.json')
Invoke-Checked python @('scripts/generate-m2-fixture.py', '--output', 'build/desktop/m2-shuffled.zip', '--shuffle')
Invoke-Checked cargo @('build', '--release', '-p', 'xdzip-m2-probe')
$m2Probe = if ($IsWindows) { 'target/release/xdzip-m2-probe.exe' } else { 'target/release/xdzip-m2-probe' }
$m2Report = & $m2Probe 'build/desktop/m2-shuffled.zip'
if ($LASTEXITCODE -ne 0) { throw 'M2 core benchmark failed' }
$m2Report | Set-Content 'dist/review/m2-core-performance.json'
Invoke-Checked python @('scripts/build-visual-audit.py', '--input', 'build/desktop/visual-audit', '--output', 'dist/review/visual-audit', '--print-checks', '--fail-on-differences', '--require-navigation')
Invoke-Checked python @('scripts/check-qt-fidelity.py', '--input', 'build/desktop/visual-audit', '--output', 'dist/review/visual-audit/fidelity.json')
# Fractional and Retina DPR change capture pixels, never the 1024-wide logical UI.
foreach ($density in @('1.25', '2')) {
    $env:QT_SCALE_FACTOR = $density
    $capturePath = "build/desktop/visual-audit-dpr$density"
    $env:XDZIP_VISUAL_AUDIT = Join-Path (Get-Location) $capturePath
    Invoke-Checked $browserTests @('mainWindowFidelityAndSessionClose', 'rootListingAndReviewedInteractions', 'reviewedMainControlsAndMenus')
    Invoke-Checked python @('scripts/check-qt-fidelity.py', '--input', $capturePath, '--output', "dist/review/fidelity-dpr$density.json")
    Copy-Item -Recurse $capturePath "dist/review/visual-audit-dpr$density"
}
Remove-Item Env:QT_SCALE_FACTOR
Remove-Item Env:XDZIP_VISUAL_AUDIT
Copy-Item 'build/desktop/apps/desktop/*.png' 'dist/review'
Copy-Item 'build/desktop/Testing/Temporary/LastTest.log' 'dist/review'
Copy-Item 'docs/engineering/developer-build.md' 'dist/review/README.md'
Invoke-Checked python @('scripts/collect-notices.py', 'dist/review/notices')
Invoke-Checked git @('archive', '--format=tar', '-o', 'dist/review/xdzip-source.tar', 'HEAD')
if ($IsMacOS) {
    $bundle = 'build/desktop/apps/desktop/xdzip.app'
    # Qt Quick's LocalStorage may deploy QtSql. Only SQLite is needed; the downloaded
    # Qt SDK also contains unrelated server drivers with machine-specific dylib paths.
    Get-ChildItem "$qtPrefix/plugins/sqldrivers/*.dylib" |
        Where-Object { $_.Name -ne 'libqsqlite.dylib' } | Remove-Item
    $deploymentLog = & macdeployqt $bundle '-qmldir=apps/desktop/ui' '-qmlimport=build/desktop/qml' '-always-overwrite' 2>&1
    $deploymentExit = $LASTEXITCODE
    $deploymentLog | ForEach-Object { Write-Host $_ }
    # macdeployqt can print missing-library errors and still return zero.
    if ($deploymentExit -ne 0 -or ($deploymentLog -match '^\s*ERROR:')) {
        throw 'macdeployqt reported an incomplete deployment'
    }
    Invoke-Checked codesign @('--force', '--deep', '--sign', '-', $bundle)
    Invoke-Checked codesign @('--verify', '--deep', '--strict', $bundle)
    # Start the deployed bundle without the development Qt library search path.
    $env:QT_PLUGIN_PATH = ''
    $env:QML_IMPORT_PATH = ''
    $env:DYLD_FRAMEWORK_PATH = ''
    $env:DYLD_LIBRARY_PATH = ''
    $env:QT_QPA_PLATFORM = 'cocoa'
    Invoke-Checked "$bundle/Contents/MacOS/xdzip" @('--smoke-test')
    Invoke-Checked ditto @('-c', '-k', '--sequesterRsrc', '--keepParent', $bundle, 'dist/review/xdzip-macos-arm64.zip')
} elseif ($IsWindows) {
    New-Item -ItemType Directory -Force 'dist/review/windows' | Out-Null
    Copy-Item 'build/desktop/apps/desktop/xdzip-browser.exe' 'dist/review/windows/xdzip.exe'
    Invoke-Checked windeployqt @('--release', '--qmldir', 'apps/desktop/ui', '--qmlimport', 'build/desktop/qml', 'dist/review/windows/xdzip.exe')
    $env:QT_QPA_PLATFORM = 'windows'
    $env:QT_PLUGIN_PATH = ''
    $env:QML_IMPORT_PATH = ''
    $env:PATH = (($env:PATH -split ';') | Where-Object { $_ -ne $qtBin }) -join ';'
    Invoke-Checked 'dist/review/windows/xdzip.exe' @('--smoke-test')
} else {
    Copy-Item 'build/desktop/apps/desktop/xdzip-browser' 'dist/review/xdzip-linux-qt6.8.2'
}
$commit = & git rev-parse HEAD
@{ commit = $commit; platform = $qtHost; architecture = $env:RUNNER_ARCH; qt = '6.8.2'; rust = '1.99.0'; distribution = 'M3 ZIP Store/Deflate safe extraction developer review; independent security audit and signed public release pending' } |
    ConvertTo-Json | Set-Content 'dist/review/build.json'
