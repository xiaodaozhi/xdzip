# 自动截取四个商店场景

简体中文 | [English](../../../market/windows/capture.md)

[capture-windows-store.ps1](../../../../scripts/capture-windows-store.ps1) 可在一次运行中自动准备并截取一种语言所需的四张截图。脚本使用商店候选程序和隔离配置打开两个演示压缩包，自动切换标签、选择文件、请求预览、切换网格视图，并打开填好文件的创建窗口，无需逐张按键或手动准备界面。按 **F10** 可取消，已保存图片会保留。

## 每种语言运行一次

先关闭已有的 xdzip 进程，在仓库根目录使用 Windows PowerShell 5.1 运行对应命令：

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language zh-CN
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language en-US
```

也可以双击 [capture-windows-store-zh-CN.cmd](../../../../scripts/capture-windows-store-zh-CN.cmd) 或 [capture-windows-store-en-US.cmd](../../../../scripts/capture-windows-store-en-US.cmd)。它们启动同一个脚本，成功后自动退出，失败时保留控制台显示错误。

`-ExecutionPolicy Bypass` 只对这次调用生效，不修改已保存的执行策略。脚本复用 `build/issue272-store-materials/runtime/xdzip.exe` 中的候选程序，以及相邻 `demo/` 目录中的自有演示文件。从 Git 工作树运行时，也可以复用主检出目录中的这些文件。脚本按 [windows-store.json](../../../../specs/marketing/windows-store.json) 中的 SHA-256 核对可执行文件，不重新构建或安装应用。

如文件位于其他位置，可指定 `-Executable`、`-DemoDirectory`，并按需指定新的 `-OutputDirectory`。中文演示目录需要包含 `设计交付.zip`、`图片精选.zip`，英文需要 `Design delivery.zip`、`Photo collection.zip`，另需 `coastline.png`、`dusk.png`、`forest.png`。脚本不会下载演示内容。输出目录已经存在时会拒绝运行，防止覆盖此前图片。

## 自动准备的场景

运行期间保持桌面可用。脚本只临时激活本次启动的 xdzip 窗口，把鼠标移到截图范围外，结束后恢复鼠标原位。窗口保持默认尺寸和当前系统缩放；如果超出显示器工作区，会仅移动窗口，不改变大小。如果默认尺寸无法放入工作区，脚本会报错停止。

| 截取顺序 | 自动准备与检查 | 场景名称 |
| --- | --- | --- |
| 1 | 激活设计交付标签，使用列表视图，仅选中 `README.md`，等待文件选择及预览、属性操作就绪。 | `01-browse` |
| 2 | 激活图片标签，仅选中 `coastline.png`，请求预览，等待图像预览对象和“重新预览”操作出现。 | `03-preview` |
| 3 | 保留两个压缩包标签，切换为网格视图，确认图片磁贴已选中。 | `04-workspace` |
| 4 | 将三张演示 PNG 复制到新的私有目录，通过 `create` 文件管理器请求打开创建窗口。核对三个输入和已启用的“创建压缩包”按钮，但不点击创建。 | `02-create` |

脚本从 Windows 可访问性树重新定位控件，并限定在本次进程和窗口中。选择文件时，先检查窗口归属、前台和鼠标位置对应的窗口，再点击实际文件行；其他控件使用可访问性操作。创建窗口复用已有文件管理器入口，临时请求仅允许当前用户和 System 访问。新增输入目录也使用独立的受保护权限，不改变已有目录或其权限。

每个场景通过状态检查后，还需要连续多次取得不变的画面，避免截到布局或图像加载中的状态。其他程序取得前台、控件不唯一、窗口最大化、主窗口尺寸变化、越出屏幕或受到遮挡时，脚本会停止，不转为手动截图。短暂运行期间不要切换程序或操作 xdzip。

## 输出与画质

默认输出到 `build/issue272-store-materials/native-captures/` 下的新目录，每次运行生成：

- 四张 `<语言>-<场景>.png`，保留客户区的原始像素。主窗口自绘标题区包含在其中，单独的原生标题栏、调整大小的边框和阴影不包含在其中。
- 四张 `<语言>-<场景>-window.png` 参考图，保留可见原生窗框，包括创建窗口的原生标题栏。外侧圆角可能包含桌面背景，这些参考图不是透明抠图。
- `capture-session.json`，记录语言、候选摘要、已核对的场景状态、尺寸、DPI、客户区裁切位置和 PNG 摘要。四个场景都保存后，`complete` 才会变为 true，未完成的运行保留 false 并记录失败原因。
- `create-demo/`，保存三张自有演示图片的副本。失败时还可能保留 `owned-ui.txt` 或 `creation-ui.txt`，供定位问题。

原生辅助代码等待 DWM 呈现后复制屏幕像素，在读取前后检查窗口归属、几何和遮挡，再编码为无损 PNG。客户区图片从同一帧按边界裁切，不缩放，不经过 JPEG，不重绘文字，不锐化字体，不添加装饰边框，也不绘制鼠标。图片物理尺寸跟随窗口实际 DPI，不会反写到 Qt 组件尺寸。实现见 [windows-store-capture.cs](../../../../scripts/windows-store-capture.cs)。

使用前请按 100% 比例查看原图。这种方法保留应用实际渲染结果，无法改善应用本身已经渲染不佳的文字。脚本不会上传图片，也不会自动替换已有商店宣传图。结束或取消时会释放取消快捷键，只关闭本次启动的 xdzip 进程，恢复鼠标原位，并移除尚未消费的私有请求。脚本不会开始压缩或解压任务。

## 不操作桌面的检查

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language zh-CN -ValidateOnly
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -SelfTest
```

`-ValidateOnly` 检查候选、演示文件路径和输出路径，不启动 xdzip，也不写入文件。`-SelfTest` 在 `build/` 中生成测试图片，检查 PNG 编码解码前后的 RGBA 像素完全一致、物理尺寸换算，以及拒绝覆盖文件。两种模式都不截取桌面，不能代替实际原生截图的画质检查或人工场景确认。
