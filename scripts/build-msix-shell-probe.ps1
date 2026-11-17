# Build the small owned fixture with the installed compiler; no product install.
param([string]$BuildDirectory = 'build/msix-shell')
$ErrorActionPreference = 'Stop'
$compilerTemp = Join-Path ([IO.Path]::GetFullPath($BuildDirectory)) 'compiler-temp'
$null = New-Item -ItemType Directory -Path $compilerTemp -Force
$env:TEMP = $compilerTemp
$env:TMP = $compilerTemp
$vswhere = "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
$vs = & $vswhere -latest -products '*' -property installationPath
if (!$vs) { throw 'Installed Visual Studio toolchain not found' }
Import-Module "$vs/Common7/Tools/Microsoft.VisualStudio.DevShell.dll"
$hostArchitecture = if ([Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString() -eq 'Arm64') { 'arm64' } else { 'x64' }
Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments "-arch=x64 -host_arch=$hostArchitecture" | Out-Null
$env:VSLANG = '1033'
cmake -S tools/msix-shell-probe -B $BuildDirectory -G Ninja -DCMAKE_BUILD_TYPE=Release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cmake --build $BuildDirectory --parallel 2
exit $LASTEXITCODE
