# Explicit local/CI preparation; fixed official tools, never bundled compiler.
param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
$config = (Get-Content 'scripts/installer-toolchain.json' -Raw | ConvertFrom-Json).inno_setup
New-Item -ItemType Directory -Force $OutputDirectory | Out-Null
function Get-PinnedFile($Url, $Hash, $Path) {
    Invoke-WebRequest -Uri $Url -OutFile $Path
    if ((Get-FileHash $Path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $Hash) { throw 'Installer tool/material hash mismatch' }
}
$setup = Join-Path $OutputDirectory 'compiler-setup.exe'
Get-PinnedFile $config.url $config.sha256 $setup
$compiler = Join-Path $OutputDirectory 'compiler'
$process = Start-Process $setup -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/CURRENTUSER',('/DIR="'+$compiler+'"')) -Wait -PassThru
if ($process.ExitCode -ne 0) { throw 'Pinned installer compiler setup failed' }
foreach ($material in $config.materials) {
    Get-PinnedFile $material.url $material.sha256 (Join-Path $OutputDirectory $material.file)
}
if (!(Test-Path (Join-Path $compiler 'ISCC.exe'))) { throw 'Installer compiler not found' }
