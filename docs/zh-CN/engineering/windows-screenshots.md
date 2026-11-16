# 可复用的 Windows 截图工具

简体中文 | [English](../../engineering/windows-screenshots.md)

仓库中的 Windows 截图工具用于保存应用实际呈现的无损 PNG，保留原生窗口尺寸，并检查每个场景是否就绪。[商店截图流程](../market/windows/capture.md)是完整示例：每种语言运行一次，自动准备并截取四个界面。其他 Windows 场景可以保留自己的界面编排，复用相同捕获模块。

## 实现边界

| 文件 | 职责 |
| --- | --- |
| [windows-capture.cs](../../../scripts/windows-capture.cs) | 进程与窗口归属、限定范围的可访问性操作、输入检查、激活、DPI 几何、DWM 呈现、原生屏幕读取、PNG 编码和客户区像素比对。 |
| [windows-capture.psm1](../../../scripts/windows-capture.psm1) | 模块加载、SHA-256、有限时长的状态等待、私有演示目录、稳定画面采样、PNG 成对导出和回执校验，不包含商店场景选择器。 |
| [capture-windows-store.ps1](../../../scripts/capture-windows-store.ps1) | xdzip 生命周期、隔离设置、本地化选择器、真实文件选择、预览、网格和已填入文件的创建窗口。 |
| [windows-store-demo.psm1](../../../scripts/windows-store-demo.psm1) | 从源码生成自有插图与压缩包，不下载素材，也不依赖已有本地演示文件。 |
| [商店截图配置](../../../specs/capture/windows-store.json) | 固定候选身份、场景集合、语言和演示素材修订号。产品逻辑尺寸仍来自 [layout-scale.json](../../../specs/visual/layout-scale.json)。 |
| [check-windows-captures.ps1](../../../scripts/check-windows-captures.ps1) | 不运行应用，直接检查已有完整截图目录。 |
| [test-windows-capture.ps1](../../../scripts/test-windows-capture.ps1) | 不截取桌面的回归检查，覆盖编码、精确裁切、有损输入、回执、超时和双语素材可复现性。 |

工具需要 Windows PowerShell 5.1 和 Windows 自带的框架程序集，商店流程还需要完整的 xdzip 运行目录。工具不构建或安装产品。原生捕获已在 Windows 执行，没有提供 macOS 或 Linux 截图实现。

## 为其他 xdzip 构建截图

两个双击入口见[商店截图说明](../market/windows/capture.md)。更新迭代时，选择新构建的完整运行目录，并传入预期可执行文件的摘要：

```powershell
$captureExecutable = 'build/package/windows/xdzip.exe'
$captureHash = (Get-FileHash -LiteralPath $captureExecutable -Algorithm SHA256).Hash
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language en-US -Executable $captureExecutable -ExpectedSha256 $captureHash
```

请使用实际构建路径。摘要用于标识所选可执行文件，不证明签名或源码来源。如果构建记录明确其源码提交，还可以通过 `-SourceRevision` 传入对应的 40 位提交号。其他可执行文件不会被标为固定商店候选，不知道源码提交时就保持未知。修改默认候选配置属于另一项明确的候选更新。

每次运行创建新输出目录，重新生成自有演示输入，并记录可执行文件、场景脚本和捕获辅助代码的摘要。也可以用 `-DemoDirectory` 明确复用另一套自有演示文件。需要复现时，应将运行目录、源码提交和本地回执一起保留；当前检出提交不能证明旧二进制的来源。

## 在其他场景复用

导入并初始化模块，由场景脚本启动可丢弃的自有测试进程，等待该进程的确定窗口。完成实际界面准备和状态检查后，调用 `Save-WindowsCapture`。应用生命周期和业务状态仍由调用方负责：

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
    # 在这里准备实际控件，并核对业务状态。
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

`$myTestExecutable` 和 `$myWindowTitle` 由新场景提供。必须使用隔离、可以安全终止的测试进程，不要替换成按名称搜索到的进程，也不要关闭其他应用。存在明确逻辑尺寸契约时，可以传 `-LogicalSize @(宽度, 高度)`；默认的 `@(0, 0)` 保留窗口，不宣称它符合某个产品尺寸。

`Xdzip.WindowsCapture.Driver` 在指定进程和窗口内重新定位控件，提供 `Ready`、`ReadyObject`、`Invoke`、`ClickNamed`、`Selected` 等方法，`Snapshot` 可保存本地诊断。应读取真实名称、角色和标识，不猜测坐标。复选框已勾选不等于模型中只选中一个文件，商店流程会点击文件行并核对结果。Qt 活动标签标识的兼容处理留在场景脚本中，没有写入通用驱动。

## 捕获与证据规则

场景脚本必须先证明界面状态就绪。通用模块随后等待画面稳定，检查确定窗口和前台，拒绝可见遮挡，等待 DWM 呈现，并在读取前后重新核对几何与归属。持续动画可能始终无法稳定，应单独定义对应场景和证据要求，不要悄悄放宽静态截图流程。有限时长的等待期间可以按 F10 取消。

客户区图片是同一张原生画面中按整数坐标裁切的区域，与 `-window.png` 参考图直接对应。不经过 JPEG，不缩放、锐化、虚构控件或添加装饰边框。客户区不包含独立原生标题栏、边框和阴影。窗框参考图保留可见系统窗框，外侧圆角可能包含桌面像素，不是透明抠图。DPI 只影响输出物理尺寸，不反写组件逻辑尺寸，也不修改系统缩放。

生成的 PNG、JSON、诊断和演示输入放在 `build/` 或 `dist/`。场景回执应记录 `complete`、预期场景集合、实际图片尺寸、裁切位置、摘要和已检查状态，失败或部分完成时保持未完成。商店脚本可作为回执示例。完整性检查只证明文件和像素一致，不能代替视觉审阅、人工操作验收或发行许可。

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check-windows-captures.ps1 -Directory build/my-capture
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/test-windows-capture.ps1
```

校验器拒绝摘要不符、非法路径、缺失场景或被修改的裁切像素。自检还会拒绝扩展名伪装为 `.png` 的 JPEG，并检查演示输入重新生成后完全一致；自检不截取桌面。既有的按需 [Windows 原生视觉工作流](../../../.github/workflows/windows-native-visual.yml)会在产品诊断前执行自检，日常 Linux CI 保持原样。
