# Product interface catalogs

Add each static desktop message to every catalog defined by [the shared language registry](../ui-languages.json). The keys in `messages` are stable IDs referenced by `qsTrId` and `qtTrId`. Use semantic IDs for new messages and retain existing IDs, including `ui.<hash>`, when editing their text.

Use strings or plural objects with the locale's integer plural categories. Keep placeholders consistent across translations. Qt uses `%n` for the quantity, and `%1` and subsequent placeholders for runtime values. Keep language endonyms as shown in the registry, translate `language.auto`, and use Qt patterns in `format.*` for displayed dates and times. Preserve protocol fields, user-entered names, fixture filenames, and code identifiers. Simplified Chinese follows the [terminology guide](../../docs/ux/chinese-terminology.md).

Run `python3 scripts/check-ui-i18n.py` after editing. Run `python3 scripts/generate-ui-messages.py` to regenerate native shell strings and the HTML settings resources. Project-owned package metadata and installer messages are in [packaging-i18n.json](../packaging-i18n.json). Normal builds subset and check the [bundled fonts](../../design/assets/fonts/README.md). Keep generated fonts and test evidence under `build/` or `dist/`.

Auto-detect (`auto`) resolves the first system UI preference at launch and uses English (US) when it cannot match a supported language. Language changes take effect immediately after a successful save. See [settings and localization](../../docs/ux/settings-localization.md) for script and region matching, save rollback, and old-version fallback.

## 产品界面语言资源

每条静态桌面文案都须填写[共享语言清单](../ui-languages.json)中的全部目录。界面通过 `qsTrId` 和 `qtTrId` 引用 `messages` 中的稳定 ID。新增文案使用语义 ID，修改既有文案时保留 ID，包括 `ui.<hash>`。

值可以是字符串，也可以是符合该语言整数复数类别的对象。各语言保留相同的占位符。Qt 用 `%n` 传递数量，用 `%1` 等传递运行数据。语言自称沿用清单，`language.auto` 随界面语言翻译，`format.*` 以 Qt 格式表示界面日期和时间。保留协议字段、用户输入的名称、测试夹具文件名和代码标识。简体中文遵循[界面术语规范](../../docs/zh-CN/ux/chinese-terminology.md)。

编辑后运行 `python3 scripts/check-ui-i18n.py`。运行 `python3 scripts/generate-ui-messages.py` 更新原生菜单字符串和 HTML 设置资源。项目自有打包元数据和安装器文案维护在 [packaging-i18n.json](../packaging-i18n.json)。普通构建裁剪并检查[随附字体](../../design/assets/fonts/README.md)。生成的字体和测试证据保存在 `build/` 或 `dist/`。

启动时，“自动识别”（`auto`）解析系统的首选界面语言。无法匹配支持的语言时使用美式英语。语言设置保存成功后即时生效。文字变体和地区匹配、保存失败回滚以及旧版回退规则见[设置和界面语言](../../docs/zh-CN/ux/settings-localization.md)。
