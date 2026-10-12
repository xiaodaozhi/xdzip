# HTML 视觉一致性复核

高保真图片是视觉依据，HTML 是可编辑实现。顶部复核从 [已确认主窗口](../../design/hifi/v1/screens/main-light.png) 独立量取尺寸，不从当前 CSS 反推“设计应当如此”。原稿存在柔化边缘和生成式字形，不能凭测试数量声明整页逐像素一致。

## 固定可重复条件

- 画布 1588×1000、窗口 1548×940、窗口原点 `(20,30)`；比较时裁切同一 `(20,30)-(1568,167)` 区域，不拉伸原稿。
- 原稿 SHA-256 固定在 [header-reference.json](../../design/html/header-reference.json)。原稿变化后必须重新测量，不能默默替换。
- 字体为离线随包的 [Xdzip Design Sans SC](../../design/assets/fonts/README.md)，测量等待 `document.fonts.ready`。固定真实 400–750 字重，减少平台回退字体改变宽度的问题。
- 参考字体的原始家族无法从生成图片证明。字形轮廓与系统抗锯齿仍可能不同；字形差异单独记录，不能伪称已经还原原始字体。

## 本次顶部修正

| 部位 | 修正前 | 按原稿修正 |
| --- | --- | --- |
| 活动 Tab | 起点 x299、宽244 | x293、宽246 |
| 非活动 Tab | x557、宽244 | x542、宽262 |
| Tab 间距 | 14 px | 3 px；按柔化边缘允许约1 px不确定性 |
| 加号与关闭按钮 | 分属两套垂直布局，中线差5 px | 使用同一标签行，按原稿的约1 px光学偏移定位 |
| 关闭控件 | 一个按钮、一个裸 span | 同一图标／名称／关闭控件结构与明确槽位 |
| 设置图标 | 可见图形小、偏左偏上 | 中心 `(1532,61)`；检查实际图形而非仅检查SVG容器 |
| 活动 Tab 下边框 | 与标题栏横线叠加 | 与工具栏白面接通，左肩过渡独立定义 |
| 搜索与解压 | 左右、垂直边界有偏移 | 分别对照 `(906,101,449,51)`、`(1368,101,183,51)` |
| 工具栏左侧图标 | 通用按钮尺寸和间距决定布局 | 按原稿的图形锚点定位与检查 |

顶部组件的槽位和排版集中在 `hifi.css` 的标题栏、标签和工具栏段落，不依赖通用 `.icon-button` 的尺寸或继承行高。浅深主题共用几何；Windows/Linux 的标题装饰仍是浏览器演示，正式产品原生窗口单独验收。

## 检查与交付

执行：

```sh
python scripts/build-html-design.py
python scripts/check-html-header.py
python scripts/check-html-preview.py
python scripts/check-html-references.py
python scripts/build-html-header-review.py
```

顶部检查使用原稿独立锚点：DOM 边界／中心最大偏差不超过 1 px，隔离区域中的实际图标墨迹边界偏差不超过 2 px。下拉箭头白色墨迹边界容差 1 px；Tab 右肩按原稿的白面和弧线取样，单通道颜色容差10/255，以容纳原稿柔化。字形轮廓只记录实测值。叠加图和差异图包含抗锯齿、光影和字形差异，不能将总像素差转换成虚构的相似度。

查看 [同尺寸对照](../qa/html-header/index.html)、[测量结果](../qa/html-header/measurements.json)、[顶部复核说明](../qa/html-header-review.md)。离线文件是 `dist/xdzip-header-review.html`，可切换原稿、修复前、修复后、透明叠加和像素差异；界面单文件是 `dist/xdzip-hifi-design.html`。

[HTML design review 工作流](../../.github/workflows/html-design.yml) 在相关设计变更时运行检查并上传实际截图和报告。锚点失败使检查失败；几何通过后仍要看叠加证据。当前检查范围是顶部，不替代其他页面各自的视觉校准。
