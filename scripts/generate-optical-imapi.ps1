<#
Authors independent, self-owned optical test images using the Windows IMAPI API.
No image is mounted and no recorder is opened. Windows only.
Official API: https://learn.microsoft.com/en-us/windows/win32/api/imapi2fs/nn-imapi2fs-ifilesystemimage
Revision API: https://learn.microsoft.com/en-us/windows/win32/api/imapi2fs/nf-imapi2fs-ifilesystemimage-put_udfrevision
Generated images/temporary files must remain under repository build/.
Example: pwsh -File scripts/generate-optical-imapi.ps1 -Output build/optical-imapi
#>
param([Parameter(Mandatory = $true)][string]$Output)
$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$buildRoot = [IO.Path]::GetFullPath((Join-Path $repositoryRoot 'build'))
$outputRoot = [IO.Path]::GetFullPath($Output, $repositoryRoot)
if (-not $outputRoot.StartsWith($buildRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
  throw 'Generated image artifacts belong below repository build/.'
}
if (Test-Path -LiteralPath $outputRoot) {
  if (-not (Get-Item -LiteralPath $outputRoot).PSIsContainer -or @(Get-ChildItem -LiteralPath $outputRoot -Force).Count -ne 0) {
    throw 'Output must not exist or must be an empty directory.'
  }
}
$inputRoot = Join-Path $outputRoot 'input'
New-Item -ItemType Directory -Force (Join-Path $inputRoot 'nested'), (Join-Path $outputRoot 'imapi-stage') | Out-Null
[IO.File]::WriteAllBytes((Join-Path $inputRoot 'hello.txt'), [Text.Encoding]::UTF8.GetBytes("independent Microsoft IMAPI fixture`n"))
[IO.File]::WriteAllBytes((Join-Path $inputRoot 'nested/中文.bin'), [byte[]](0..255) * 32)
[IO.File]::WriteAllBytes((Join-Path $inputRoot 'empty.txt'), [byte[]]@())
Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;
public static class ImapiArtifact {
  public static long Write(object source, string path) {
    IStream stream = (IStream)source;
    IntPtr count = Marshal.AllocHGlobal(4);
    long total = 0;
    try {
      using (FileStream output = new FileStream(path, FileMode.Create, FileAccess.Write)) {
        byte[] buffer = new byte[65536];
        while (true) {
          stream.Read(buffer, buffer.Length, count);
          int read = Marshal.ReadInt32(count);
          if (read == 0) break;
          output.Write(buffer, 0, read);
          total += read;
          if (total > 8 * 1024 * 1024) throw new InvalidDataException("fixture_size_limit");
        }
      }
    } finally { Marshal.FreeHGlobal(count); }
    return total;
  }
}
'@
$records = @()
foreach ($revision in @(0x102, 0x150, 0x200, 0x201, 0x250)) {
  $image = New-Object -ComObject IMAPI2FS.MsftFileSystemImage
  $image.FileSystemsToCreate = 4
  $image.UDFRevision = $revision
  $image.FreeMediaBlocks = 2048
  $image.VolumeName = 'XDZIP_INDEPENDENT'
  $image.WorkingDirectory = Join-Path $outputRoot 'imapi-stage'
  $image.Root.AddTree($inputRoot, $false)
  $result = $image.CreateResultImage()
  $path = Join-Path $outputRoot ('imapi-{0:x}.iso' -f $revision)
  $size = [ImapiArtifact]::Write($result.ImageStream, $path)
  $records += [ordered]@{file = [IO.Path]::GetFileName($path); revision = ('{0:x}' -f $revision); bytes=$size; sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLower()}
}
$payloads = @()
foreach ($name in @('hello.txt', 'empty.txt', 'nested/中文.bin')) {
  $path = Join-Path $inputRoot $name
  $payloads += [ordered]@{name=$name; bytes=(Get-Item -LiteralPath $path).Length; sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLower()}
}
$manifest = [ordered]@{
  author='Microsoft Windows IMAPI2FS'
  os=[System.Runtime.InteropServices.RuntimeInformation]::OSDescription
  imapi_version=(Get-Item (Join-Path $env:SystemRoot 'System32/imapi2fs.dll')).VersionInfo.FileVersion
  payloads=$payloads
  images=$records
}
$manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $outputRoot 'imapi-manifest.json') -Encoding utf8
$records | Format-Table
