# Reusable Windows screenshots

English | [简体中文](../zh-CN/engineering/windows-screenshots.md)

Use the repository's Windows capture tools to save real application pixels as lossless PNG, preserve native window dimensions and check what each scene actually shows. The [Store recipe](../market/windows/capture.md) is a complete example: one run prepares and captures four views in one language. Other Windows workflows can reuse the same capture module with their own scene preparation.

## Implementation boundaries

| File | Responsibility |
| --- | --- |
| [windows-capture.cs](../../scripts/windows-capture.cs) | Process/window ownership, scoped UI Automation, guarded input, activation, DPI-aware geometry, DWM presentation, native screen readback, PNG encoding and exact client/window pixel comparison. |
| [windows-capture.psm1](../../scripts/windows-capture.psm1) | Module loading, SHA-256, bounded state waits, private fixture directories, stable-frame sampling, PNG pair export and receipt verification. It has no Store scene selectors. |
| [capture-windows-store.ps1](../../scripts/capture-windows-store.ps1) | xdzip lifecycle, isolated settings, localized selectors, real file selection, preview, grid view and a populated creation window. |
| [refresh-windows-store-screenshots.py](../../scripts/refresh-windows-store-screenshots.py) | Local bilingual capture from a verified MSIX, native-pixel cleanup, Store and README export, and optional material updates after preview. |
| [windows-store-demo.psm1](../../scripts/windows-store-demo.psm1) | Owned illustration and archive generation, without downloads or existing local demo files. |
| [Store capture configuration](../../specs/capture/windows-store.json) | Pinned candidate identity, scene set, languages and fixture revision. Product logical dimensions remain in [layout-scale.json](../../specs/visual/layout-scale.json). |
| [check-windows-captures.ps1](../../scripts/check-windows-captures.ps1) | Verify a completed capture directory without running the application. |
| [test-windows-capture.ps1](../../scripts/test-windows-capture.ps1) | Desktop-free regression checks for encoding, exact crops, lossy inputs, receipts, timeouts and reproducible bilingual fixtures. |

These tools require Windows PowerShell 5.1 and framework assemblies already included with Windows. The Store recipe additionally needs a complete xdzip runtime. They do not build or install the product. Native captures have been exercised on Windows; they provide no macOS or Linux screenshot implementation.

## Capture another xdzip build

The [Store guide](../market/windows/capture.md) covers the two double-click entry points. For a newer build, select its complete runtime and supply the intended executable's hash:

```powershell
$captureExecutable = 'build/package/windows/xdzip.exe'
$captureHash = (Get-FileHash -LiteralPath $captureExecutable -Algorithm SHA256).Hash
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language en-US -Executable $captureExecutable -ExpectedSha256 $captureHash
```

Use the actual path of your build. The hash identifies the chosen executable; it does not establish a signature or source provenance. If its source commit is known from the build record, also pass `-SourceRevision` with that 40-character commit. A different executable is never labeled as the pinned Store candidate, and an unknown source revision remains unknown. Updating the pinned configuration is a separate, deliberate candidate change.

Each run creates a new output directory, regenerates its own demonstration inputs and records the executable, recipe and capture-helper hashes. `-DemoDirectory` can explicitly reuse another owned demo set. Retain the runtime, source revision and local receipt together when reproducibility matters; a current checkout alone does not prove an old binary's origin.

## Reuse capture mechanics

Import the module, initialize it, launch a disposable process owned by your recipe and wait for that process's exact window. After preparing and checking a scene, call `Save-WindowsCapture`. The caller owns process cleanup and scene semantics:

```powershell
Import-Module ./scripts/windows-capture.psm1
Initialize-WindowsCapture
$captureOutput = 'build/my-capture'
$null = New-Item -ItemType Directory -Path $captureOutput -ErrorAction Stop
$start = [Diagnostics.ProcessStartInfo]::new()
$start.FileName = $myTestExecutable
$start.UseShellExecute = $false
$owned = [Diagnostics.Process]::Start($start)
$pointer = [Xdzip.WindowsCapture.Native]::Pointer()
[Xdzip.WindowsCapture.Native]::RegisterKeys()
try {
    $script:ownedWindow = 0L
    Wait-WindowsCaptureState -Process $owned -Label 'owned window' -Predicate {
        $script:ownedWindow = [Xdzip.WindowsCapture.Native]::TitledWindow($owned.Id, $myWindowTitle)
        $script:ownedWindow -ne 0
    }
    # Prepare real controls and verify the domain state here.
    $shot = Save-WindowsCapture -Process $owned -Window $ownedWindow -OutputDirectory $captureOutput -Name 'main' -ExpectedTitle $myWindowTitle
    $shot['scene'] = 'main'
    @{ schema_version = 3; complete = $true; expected_scenes = @('main'); images = @($shot) } |
        ConvertTo-Json -Depth 6 | Set-Content (Join-Path $captureOutput 'capture-session.json') -Encoding UTF8
} finally {
    [Xdzip.WindowsCapture.Native]::ReleaseKeys()
    if (!$owned.HasExited) { $owned.Kill(); $owned.WaitForExit() }
    $owned.Dispose()
    [Xdzip.WindowsCapture.Native]::RestorePointer($pointer[0], $pointer[1])
}
```

`$myTestExecutable` and `$myWindowTitle` belong to the new workflow. Use an isolated test process that can safely be terminated. Never substitute a process found by name or close unrelated applications. Use `-LogicalSize @(width, height)` when a known logical-size contract applies; the default `@(0, 0)` preserves the window without claiming a particular product size.

`Xdzip.WindowsCapture.Driver` resolves fresh controls inside the supplied process and window. `Ready`, `ReadyObject`, `Invoke`, `ClickNamed` and `Selected` can support new recipes. `Snapshot` saves local diagnostics. Read actual names, roles and IDs instead of guessing coordinates. A checked box is not proof of a single selected model entry; the Store recipe clicks the file row and checks the resulting state. Its Qt active-tab ID fallback stays in the recipe, not in the generic driver.

## Capture and evidence rules

The recipe must establish real scene readiness. The generic layer then waits for stable frames, checks the exact window and foreground, rejects visible overlap, waits for DWM and rechecks geometry and ownership around readback. Animated scenes may never stabilize; use a dedicated recipe and an explicit evidence contract rather than weakening a static recipe silently. F10 cancels during bounded waits.

The client image is an integer crop of the same native frame as its `-window.png` reference. No JPEG, resizing, sharpening, invented controls or decorative frames are used. Native title bars, borders and shadows are excluded from the client crop. The frame reference retains the visible OS frame; rounded outer corners may include desktop pixels and are not transparent cutouts. DPI affects physical output dimensions, not component logical sizes or system scaling.

Keep generated PNGs, JSON, diagnostics and demo inputs under `build/` or `dist/`. A recipe should record `complete`, an expected scene set, actual image dimensions, client crops, hashes and checked scene state. Failed or partial runs remain incomplete. The Store script's receipt is a working example. A successful integrity check proves file and pixel consistency; it does not replace visual review, manual interaction acceptance or release approval.

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check-windows-captures.ps1 -Directory build/my-capture
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/test-windows-capture.ps1
```

The validator rejects mismatched hashes, invalid paths, missing scenes and changed crop pixels. The self-test also rejects a JPEG carrying a `.png` filename and checks identical regenerated demo inputs. It does not capture the desktop. The existing manually dispatched [Windows native visual workflow](../../.github/workflows/windows-native-visual.yml) runs this self-test before its product diagnostics; ordinary Linux CI remains unchanged.
