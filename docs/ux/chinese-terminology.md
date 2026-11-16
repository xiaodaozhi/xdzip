# Simplified Chinese UI terminology

User-facing Chinese copy follows the [terminology configuration](../../specs/ux/chinese-terminology.json). Qt static text retains stable IDs in resources/i18n/zh-CN.json. HTML, current high-fidelity designs and user documentation follow the table below; engineering and format documents may use technical terminology. This change does not alter component dimensions, main-window 0.8/menu 1 density, DPR or scaling parameters.

| Meaning | Standard Chinese wording | Example use |
| --- | --- | --- |
| Container file such as ZIP or TAR | 压缩包 | 压缩包名称, 空压缩包, 压缩包信息 |
| Full action names | 打开压缩包, 创建压缩包, 解压 | “打开” and “新建” may be shortened where the toolbar context is already clear |
| Objects within an archive | 文件, 文件夹, 内容 | 所选文件和文件夹, 全部内容 |
| File properties | 原始大小, 压缩大小 | Use the same names in column headers, preview properties and sort menus |
| Restore physical entry order | 按原始顺序排列 | Distinguish from sorting by name or size |

“档案文件” is not product terminology. “归档”, “归档文件” and “归档格式” are restricted to engineering specifications, format descriptions or public API contexts and do not appear in ordinary Chinese UI. Even when TAR only packs without compression, the UI uses “压缩包”; actual compression capability is expressed by format and compression level.

“压缩文件” may only describe an action, such as “正在压缩文件”, not a container. “压缩后” is no longer a property name; use “压缩大小”. The creation window's default is “新建压缩包.zip”; this does not rename an existing file.

Do not replace terminology in user-provided names or paths, filenames inside archives, protocol fields, code identifiers or historical high-fidelity images. English retains the standard word “archive” according to English grammar; “全部内容” is “All contents”. English size properties retain “Compressed” as an abbreviation for compressed size suitable for narrow headers. Do not replace English code identifiers according to Chinese vocabulary.

See the current [terminology state design](../../design/hifi/terminology.html). Original images, hashes and text-ink measurements remain historical evidence. The [body measurement script](../../scripts/check-html-body.py) verifies new copy only through explicit historical label mappings, without changing geometry targets or tolerances. Longer labels still require verification that the main window, headers, preview and menus display completely.

Run `python3 scripts/check-ui-i18n.py` to check both language resources and Chinese terminology. The check applies only to static product copy, permits the explicitly allowed verb use of “压缩文件”, and does not restrict user content or engineering terms.

As confirmed by the owner in [#270](https://github.com/xiaodaozhi/xdzip/issues/270), system file-manager entries retain “添加到压缩文件...” and “解压每个压缩文件到单独的文件夹”. The configuration registers exceptions only for the complete text of these two stable IDs; it does not broaden permitted Chinese container names elsewhere.
