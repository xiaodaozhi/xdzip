# 辅助截取四个商店场景

简体中文 | [English](../../../market/windows/capture.md)

[capture-windows-store.ps1](../../../../scripts/capture-windows-store.ps1) 可在一次运行中辅助截取一种语言所需的四张截图。脚本使用商店候选程序和隔离配置打开两个演示压缩包，你手动准备每个界面，然后在 xdzip 激活时按 **F8**。脚本负责保存 PNG、命名并进入下一场景。按 **F10** 可取消，已保存图片会保留。

## 每种语言运行一次

先关闭已有的 xdzip 进程，在仓库根目录使用 Windows PowerShell 5.1 运行对应命令：

也可以双击 [capture-windows-store-zh-CN.cmd](../../../../scripts/capture-windows-store-zh-CN.cmd) 或 [capture-windows-store-en-US.cmd](../../../../scripts/capture-windows-store-en-US.cmd)。它们启动同一个辅助脚本，并保留控制台供查看结果。

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language zh-CN
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language en-US
```

`-ExecutionPolicy Bypass` 只对这次调用生效，不修改已保存的执行策略。脚本复用 `build/issue272-store-materials/runtime/xdzip.exe` 中的候选程序，以及相邻 `demo/` 目录中的自有演示文件。从 Git 工作树运行时，也可以复用主检出目录中的这些文件。脚本按 [windows-store.json](../../../../specs/marketing/windows-store.json) 中的 SHA-256 核对可执行文件，不重新构建或安装应用。

如文件位于其他位置，可指定 `-Executable`、`-DemoDirectory`，并按需指定新的 `-OutputDirectory`。中文演示目录需要包含 `设计交付.zip`、`图片精选.zip`，英文需要 `Design delivery.zip`、`Photo collection.zip`，另需 `coastline.png`、`dusk.png`、`forest.png`。脚本不会下载演示内容。输出目录已经存在时会拒绝运行，防止覆盖此前图片。

## 准备界面后按 F8

保持默认窗口大小和当前系统缩放，等待加载结束，关闭菜单和提示，把鼠标移到窗口外，激活当前要求的 xdzip 窗口后按 F8。控制台会显示当前步骤，保存成功后会发出提示音并显示文件名。

| 截取顺序 | 需要准备的界面 | 场景名称 |
| --- | --- | --- |
| 1 | 切到设计交付压缩包，使用列表视图，选中一个文件，让右侧显示属性。 | `01-browse` |
| 2 | 切到图片压缩包，选中 `coastline.png`，点击预览并等待完整图像显示。 | `03-preview` |
| 3 | 保留两个压缩包标签，将图片压缩包切换为网格视图并选中图片。 | `04-workspace` |
| 4 | 按 Ctrl+N 打开创建窗口，添加演示目录中的三张 PNG，目标目录设为演示目录，保留 ZIP 压缩选项。不要开始创建。 | `02-create` |

脚本不会代替你点击这些控件，也不会判断内容是否准备好，按 F8 即表示场景已就绪。它会拒绝其他程序的前台窗口、当前场景不匹配的 xdzip 窗口、最大化状态、主窗口尺寸变化、窗口越出屏幕或受到遮挡的情况。按照提示调整后重新按 F8 即可。完整窗口需要位于同一显示器的工作区内，脚本不会调整窗口尺寸或显示器设置来强行适配。

## 输出与画质

默认输出到 `build/issue272-store-materials/native-captures/` 下的新目录，每次运行生成：

- 四张 `<语言>-<场景>.png`，保留客户区的原始像素。主窗口自绘标题区包含在其中，单独的原生标题栏、调整大小的边框和阴影不包含在其中。
- 四张 `<语言>-<场景>-window.png` 参考图，保留可见原生窗框，包括创建窗口的原生标题栏。外侧圆角可能包含桌面背景，这些参考图不是透明抠图。
- `capture-session.json`，记录语言、候选摘要、尺寸、DPI、客户区裁切位置和 PNG 摘要。四个场景都保存后，`complete` 才会变为 true，未完成的运行保留 false。

原生辅助代码等待 DWM 呈现后复制屏幕像素，在读取前后检查窗口归属、几何和遮挡，再编码为无损 PNG。客户区图片从同一帧按边界裁切，不缩放，不经过 JPEG，不重绘文字，不锐化字体，不添加装饰边框，也不绘制鼠标。图片物理尺寸跟随窗口实际 DPI，不会反写到 Qt 组件尺寸。实现见 [windows-store-capture.cs](../../../../scripts/windows-store-capture.cs)。

使用前请按 100% 比例查看原图。这种方法保留应用实际渲染结果，无法改善应用本身已经渲染不佳的文字。脚本不会上传图片，也不会自动替换已有商店宣传图。结束或取消时会释放快捷键，并只关闭本次启动的 xdzip 进程。截图会话中不要执行真实压缩包任务。

## 不操作桌面的检查

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language zh-CN -ValidateOnly
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -SelfTest
```

`-ValidateOnly` 检查候选、演示文件路径和输出路径，不启动 xdzip，也不写入文件。`-SelfTest` 在 `build/` 中生成测试图片，检查 PNG 编码解码前后的 RGBA 像素完全一致、物理尺寸换算，以及拒绝覆盖文件。两种模式都不截取桌面，不能代替实际原生截图的画质检查或人工场景确认。
