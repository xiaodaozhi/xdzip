# 目录树 B 设计与切图交付检查

交付日期：2026-10-12。范围为正式设计稿、原像素切片、共享资源和[接入规格](../ux/directory-tree.md)；10 月 12 日的原稿与切图交付证据保留；10 月 13 日已将核心目录交互接入 HTML 和 Qt，实际行为检查另见[组件交互复核](main-window-components.md)。

## 交付清单

- 三张 1024×1536 已确认原稿：[多级展开](../../design/hifi/screens/directory-tree.png)、[节点状态](../../design/hifi/screens/directory-tree-states.png)、[导航边界](../../design/hifi/screens/directory-tree-boundaries.png)，在既有 `screens/` 中按功能平铺。
- 26 张带来源坐标的原像素切片：文件夹、展开 / 折叠箭头、连接线区域、当前标记、各种行状态、全文提示、深层级、大字号、骨架、空态、超限、错误与隐藏当前分支。
- 新增 `tree-active-branch`、`tree-info`、`tree-warning`、`tree-spinner` 四种语义，共八张 24×24 浅深色 SVG、24 张 24/48/72 透明 PNG；圆点可见直径 6 单位。信息圆、警告三角和分段转轮按 B 稿造型重建。
- 文件夹与箭头复用已有共享图标，避免复制同一形状。动态背景、文字与连接线有明确绘制规格。
- [逻辑尺寸、16 种状态与完整样例](../visual/directory-tree.json)以及每个切片的 `component_reference_coverage` 接入说明。

## 实际执行的检查

[原始检查结果](directory-tree/validation.json)记录本次通过项；[保存基线](directory-tree/baseline.json)来自提交 `5e989fb557d0cee80217cba7f4d82f23c46ffbb8`。

[离线交付复核数据](directory-tree/delivery-review.json)记录 1280、390、320 宽度的浏览结果：各 43 张图片完成解码，三张图板与 26 项切图完整，无外部图片请求及页面横向溢出。ZIP 共 93 个成员，92 个负载文件按交付台账核对哈希，压缩容器完整性检查通过。

1. 新图板按 SHA-256 验证原始字节；26 张裁剪全部位于源图范围内，尺寸与像素逐项等于原图对应矩形。
2. 原有十张完整设计、45 张图标参考、三张演示照片共 58 个文件的 SHA-256 保持不变。
3. 新圆点 SVG 的中心、半径和主题色独立检查；信息圆、警告三角的实心填充和转轮十二段结构单独检查。24 张 PNG 检查透明通道、统一画布和三个倍率。
4. 多级样例包括折叠节点的真实后代；30 条根计数、冰川湖 6 条以及非空叶子的箭头条件一致。空归档、根文件、未知计数、超限和隐藏当前路径分别验证规格数据。
5. 统一素材清单与原稿 / 素材画廊同步，文档本地链接及现有 Qt / HTML 资源引用检查通过。离线 HTML 复核图片能解码，桌面和手机宽度下布局不溢出。

生成与复核命令：

```sh
python3 scripts/build-hifi-gallery.py
python3 scripts/build-design-assets.py
python3 scripts/render-design-assets.py
python3 scripts/check-design-assets.py
python3 scripts/check-directory-tree-assets.py
python3 scripts/check-asset-integration.py
python3 scripts/check-docs.py
python3 scripts/build-directory-tree-delivery.py --standalone dist/xdzip-directory-tree.html --package dist/xdzip-directory-tree.zip
python3 scripts/check-directory-tree-delivery.py
```

`check-design-assets.py` 同时运行组件交付检查，现有素材 CI 因而覆盖新增文件。交付 HTML 为原稿和素材的静态画廊，内嵌全部相关图片；ZIP 保留仓库相对目录、规格、切图、被引用的矢量 / PNG 与 Apache-2.0 许可，不包含产品代码或其它设计版本。

逐项视觉查看确认了完整文件夹、箭头、行状态和边界切片；连接线区域重新裁剪，排除被截断的文件夹。[透明图标对照](directory-tree/glyphs.png)展示原稿区域与两主题导出：信息与警告使用实心造型，读取使用分段转轮。原稿切片在对照图中按审阅框适应显示，不代表与统一运行画布使用同一缩放系数；圆点运行尺寸明确为 6 逻辑单位。

## 使用与边界

手机可打开离线 `xdzip-directory-tree.html` 查看三幅完整图板与原像素切片；素材画廊同样从 `design/assets/index.html#directory-tree` 查看。切片保留原背景，只用于对照；产品图标来自共享 SVG 和透明 PNG，不能把不同裁剪尺寸直接设成控件尺寸。

核心消费者现已接入：连接线、固定数字、20 逻辑图标槽与隐藏当前圆点用于实际 HTML / Qt 行。实时点击与方向键证据单独记录；专用边界提示图标、骨架、完整深色图板、人工读屏和目标系统 HiDPI 仍待后续验证。此处的素材完整性检查不能替代交互检查。
