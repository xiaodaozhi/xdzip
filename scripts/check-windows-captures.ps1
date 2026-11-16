#requires -Version 5.1
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$Directory)
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'windows-capture.psm1')
Initialize-WindowsCapture
$count = Test-WindowsCaptureSession -Directory $Directory
Write-Host "Validated $count captures: PNG format, hashes, dimensions, exact window/client pixels and complete scene set."
