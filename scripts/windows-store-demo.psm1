# Owned, reproducible fixtures for the Store capture recipe. No downloads.
Set-StrictMode -Version Latest

function New-WindowsStoreDemo {
    param([string]$Directory, [ValidateSet('zh-CN', 'en-US', 'zh-TW', 'ja-JP', 'ko-KR', 'fr-FR', 'de-DE', 'es-ES', 'pt-BR', 'ru-RU')][string]$Language)
    if (Test-Path -LiteralPath $Directory) { throw 'Demo directory already exists.' }
    if (!('XdzipStoreDemoImages' -as [type])) {
        Add-Type -ReferencedAssemblies System.Drawing @'
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.IO;
public static class XdzipStoreDemoImages {
    public static void Write(string directory) {
        string[] names = { "coastline", "dusk", "forest" };
        Color[] colors = { Color.FromArgb(39, 111, 244), Color.FromArgb(117, 72, 190), Color.FromArgb(31, 137, 116) };
        for (int i = 0; i < names.Length; i++) {
            using (var image = new Bitmap(960, 640, PixelFormat.Format24bppRgb))
            using (var graphics = Graphics.FromImage(image))
            using (var sky = new LinearGradientBrush(new Rectangle(0, 0, 960, 640), colors[i], Color.FromArgb(115, 213, 220), 25f)) {
                graphics.SmoothingMode = SmoothingMode.AntiAlias;
                graphics.FillRectangle(sky, 0, 0, 960, 640);
                using (var sun = new SolidBrush(Color.FromArgb(195, 225, 250))) graphics.FillEllipse(sun, 700, 90, 140, 140);
                using (var hill = new GraphicsPath()) {
                    hill.AddBezier(0, 480, 260, 260, 390, 720, 960, 340);
                    hill.AddLine(960, 340, 960, 640); hill.AddLine(960, 640, 0, 640); hill.CloseFigure();
                    using (var fill = new SolidBrush(Color.FromArgb(226, 241, 235))) graphics.FillPath(fill, hill);
                }
                using (var sand = new GraphicsPath()) {
                    sand.AddBezier(0, 580, 340, 410, 620, 620, 960, 470);
                    sand.AddLine(960, 470, 960, 640); sand.AddLine(960, 640, 0, 640); sand.CloseFigure();
                    using (var fill = new SolidBrush(Color.FromArgb(253, 247, 231))) graphics.FillPath(fill, sand);
                }
                image.Save(Path.Combine(directory, names[i] + ".png"), ImageFormat.Png);
            }
        }
    }
}
'@
    }
    Add-Type -AssemblyName System.IO.Compression
    $null = New-Item -ItemType Directory -Path $Directory
    [XdzipStoreDemoImages]::Write($Directory)
    $chinese = $Language -eq 'zh-CN'
    $browseName = if ($chinese) { '设计交付.zip' } else { 'Design delivery.zip' }
    $photoName = if ($chinese) { '图片精选.zip' } else { 'Photo collection.zip' }
    $utf8 = New-Object Text.UTF8Encoding($false)
    function Write-Entry($Archive, [string]$Name, [byte[]]$Bytes) {
        $entry = $Archive.CreateEntry($Name, [IO.Compression.CompressionLevel]::Optimal)
        $entry.LastWriteTime = [DateTimeOffset]::new(2026, 1, 1, 0, 0, 0, [TimeSpan]::Zero)
        $stream = $entry.Open()
        try { $stream.Write($Bytes, 0, $Bytes.Length) } finally { $stream.Dispose() }
    }
    foreach ($archiveName in @($browseName, $photoName)) {
        $file = [IO.File]::Open((Join-Path $Directory $archiveName), [IO.FileMode]::CreateNew)
        try {
            $archive = [IO.Compression.ZipArchive]::new($file, [IO.Compression.ZipArchiveMode]::Create, $true, $utf8)
            try {
                if ($archiveName -eq $browseName) {
                    $readme = if ($chinese) { "# 设计交付`n`n这是用于展示压缩包浏览与内容预览的自有演示文件。`n" } else { "# Design delivery`n`nOwned demonstration files for archive browsing and content preview.`n" }
                    Write-Entry $archive 'README.md' ($utf8.GetBytes($readme))
                    $documents = if ($chinese) { '文档' } else { 'Documents' }
                    $source = if ($chinese) { '示例代码' } else { 'Source' }
                    Write-Entry $archive "$documents/Project brief.txt" ($utf8.GetBytes('Project notes and delivery checklist.'))
                    Write-Entry $archive "$documents/Notes.txt" ($utf8.GetBytes('Review, package and share.'))
                    Write-Entry $archive "$source/example.py" ($utf8.GetBytes("print('Hello, archives!')`n"))
                    Write-Entry $archive "$source/palette.csv" ($utf8.GetBytes("name,color`nblue,#276ff4`n"))
                }
                foreach ($image in @('coastline.png', 'dusk.png', 'forest.png')) {
                    $name = $image
                    if ($archiveName -eq $browseName) {
                        $prefix = if ($chinese) { '图像/' } else { 'Images/' }
                        $name = $prefix + $image
                    }
                    Write-Entry $archive $name ([IO.File]::ReadAllBytes((Join-Path $Directory $image)))
                }
            } finally { $archive.Dispose() }
        } finally { $file.Dispose() }
    }
    return $Directory
}

Export-ModuleMember -Function New-WindowsStoreDemo
