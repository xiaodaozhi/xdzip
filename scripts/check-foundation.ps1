# Run inside the same shell as MSVC discovery so Cargo's cc build scripts inherit it.
$ErrorActionPreference = 'Stop'
if ($IsMacOS) { $env:MACOSX_DEPLOYMENT_TARGET = '14.0' }
if ($IsWindows) {
    $vswhere = "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (!$installation) { throw 'MSVC installation not found' }
    Import-Module "$installation/Common7/Tools/Microsoft.VisualStudio.DevShell.dll"
    Enter-VsDevShell -VsInstallPath $installation -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
}
function Invoke-Checked {
    param([string]$Program, [string[]]$Arguments)
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed with exit code $LASTEXITCODE" }
}
Invoke-Checked rustc @('-vV')
Invoke-Checked cargo @('fmt', '--all', '--', '--check')
Invoke-Checked cargo @('clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings')
$codecPython = if ($IsWindows) { 'python' } else { 'python3' }
Invoke-Checked $codecPython @('scripts/check-rust-codec-policy.py')
Invoke-Checked $codecPython @('scripts/prepare-native-codecs.py')
Invoke-Checked cmake @('-S', '.', '-B', 'build/foundation', '-G', 'Ninja', '-DCMAKE_BUILD_TYPE=Release', '-DXDZIP_BUILD_QT_PROBE=OFF')
Invoke-Checked cmake @('--build', 'build/foundation', '--target', 'xdzip-native-codec', '--parallel', '2')
$env:XDZIP_NATIVE_CODEC = Join-Path (Get-Location) $(if ($IsWindows) { 'build/foundation/tools/native-codec/xdzip-native-codec.exe' } else { 'build/foundation/tools/native-codec/xdzip-native-codec' })
$env:XDZIP_MOVE_INTEROP_OUTPUT = Join-Path (Get-Location) 'build/move-interop'
$env:XDZIP_UPDATE_INTEROP_OUTPUT = Join-Path (Get-Location) 'build/update-interop'
$env:XDZIP_METADATA_INTEROP_OUTPUT = Join-Path (Get-Location) 'build/macos-metadata-interop'
$env:XDZIP_LEGACY_INTEROP_OUTPUT = Join-Path (Get-Location) 'build/legacy-name-interop'
Invoke-Checked cargo @('test', '--workspace', '--locked')
Invoke-Checked cargo @('build', '-p', 'xdzip-m3-probe', '-p', 'xdzip-m4-probe', '--locked')
$rustCodecEvidence = 'build/review/independent-rust-codecs'
New-Item -ItemType Directory -Path $rustCodecEvidence -Force | Out-Null
$extractProbe = if ($IsWindows) { 'target/debug/xdzip-m3-probe.exe' } else { 'target/debug/xdzip-m3-probe' }
$createProbe = if ($IsWindows) { 'target/debug/xdzip-m4-probe.exe' } else { 'target/debug/xdzip-m4-probe' }
Invoke-Checked $codecPython @('scripts/check-tar-interop.py', '--create-binary', $createProbe, '--extract-binary', $extractProbe, '--output', "$rustCodecEvidence/tar-interop.json")
Invoke-Checked cargo @('run', '--release', '-p', 'xdzip-compression', '--example', 'gzip-probe', '--locked') > "$rustCodecEvidence/compression-benchmark.json"
Invoke-Checked cargo @('test', '-p', 'xdzip-runtime', '--locked', 'native_backend_tests', '--', '--ignored')
Invoke-Checked $codecPython @('scripts/prepare-native-interop.py')
$codecReference = if ($IsWindows) { 'build/native-codec/reference/7za.exe' } else { 'build/native-codec/reference/7zz' }
Invoke-Checked $codecPython @('scripts/check-native-codecs.py', '--worker', $env:XDZIP_NATIVE_CODEC, '--sevenzip', $codecReference, '--output', 'build/review/native-codec-interop')
Invoke-Checked $codecPython @('scripts/check-cab-iso-interop.py', '--worker', $env:XDZIP_NATIVE_CODEC, '--output', 'build/review/native-codec-interop/cab-iso')
$updatePython = if ($IsWindows) { 'python' } else { 'python3' }
Invoke-Checked $updatePython @('scripts/check-filename-encoding.py', '--probe', $extractProbe, '--input', 'build/legacy-name-interop', '--output', 'build/review/legacy-name-rewrite.json')
Invoke-Checked $updatePython @('scripts/check-update-interop.py', '--input', 'build/update-interop')
Invoke-Checked $updatePython @('scripts/check-macos-metadata-interop.py', '--input', 'build/macos-metadata-interop')
Invoke-Checked $updatePython @('scripts/check-move-interop.py', '--input', 'build/move-interop')
Invoke-Checked $updatePython @('scripts/test-release-updates.py')
Invoke-Checked cmake @('-S', '.', '-B', 'build/foundation', '-G', 'Ninja', '-DCMAKE_BUILD_TYPE=Release', '-DXDZIP_BUILD_QT_PROBE=OFF')
Invoke-Checked cmake @('--build', 'build/foundation')
Invoke-Checked ctest @('--test-dir', 'build/foundation', '--output-on-failure')
