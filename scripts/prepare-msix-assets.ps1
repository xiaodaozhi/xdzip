param(
    [Parameter(Mandatory=$true)][string]$Source,
    [Parameter(Mandatory=$true)][string]$OutputDirectory
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$policy = Get-Content -LiteralPath (Join-Path $PSScriptRoot '../specs/visual/msix-icons.json') -Raw | ConvertFrom-Json
$image = [System.Drawing.Image]::FromFile($Source)
try {
    $assets = [ordered]@{}
    foreach ($logo in $policy.application.logos.PSObject.Properties) {
        $assets["$($logo.Name).png"] = [int]$logo.Value
        foreach ($scale in $policy.application.scales) {
            if ($scale -ne 100) {
                $assets["$($logo.Name).scale-$scale.png"] = [int][Math]::Floor(($logo.Value * $scale + 50) / 100)
            }
        }
    }
    foreach ($size in $policy.application.target_sizes) {
        foreach ($form in $policy.application.alternate_forms) {
            $suffix = if ($form) { "_altform-$form" } else { '' }
            $assets["logo44.targetsize-$size$suffix.png"] = [int]$size
        }
    }
    foreach ($asset in $assets.GetEnumerator()) {
        $size = $asset.Value
        $bitmap = [System.Drawing.Bitmap]::new($size, $size)
        try {
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            try {
                $graphics.Clear([System.Drawing.Color]::Transparent)
                $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
                $graphics.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceCopy
                $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
                $graphics.DrawImage($image, 0, 0, $size, $size)
            } finally { $graphics.Dispose() }
            $bitmap.Save((Join-Path $OutputDirectory $asset.Key), [System.Drawing.Imaging.ImageFormat]::Png)
        } finally { $bitmap.Dispose() }
    }
} finally { $image.Dispose() }
