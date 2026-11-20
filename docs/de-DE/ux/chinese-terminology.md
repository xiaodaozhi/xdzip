<a name="xdzip-simplified-chinese-ui-terminology"></a>

# Terminologie der Oberfläche auf vereinfachtem Chinesisch

[English](../../ux/chinese-terminology.md) | [简体中文](../../zh-CN/ux/chinese-terminology.md) | [繁體中文](../../zh-TW/ux/chinese-terminology.md) | [日本語](../../ja-JP/ux/chinese-terminology.md) | [한국어](../../ko-KR/ux/chinese-terminology.md) | [Français](../../fr-FR/ux/chinese-terminology.md) | Deutsch | [Español](../../es-ES/ux/chinese-terminology.md) | [Português (Brasil)](../../pt-BR/ux/chinese-terminology.md) | [Русский](../../ru-RU/ux/chinese-terminology.md)

Texte für Benutzer auf vereinfachtem Chinesisch folgen der [Terminologiekonfiguration](../../../specs/ux/chinese-terminology.json). Statische Qt-Texte behalten ihre stabilen IDs in resources/i18n/zh-CN.json. HTML, aktuelle Detailentwürfe und Benutzeranleitungen übernehmen die Tabellenbegriffe. Technik- und Formatdokumente dürfen Fachbegriffe verwenden. Komponentenmaße, Dichte 0.8 im Hauptfenster und 1 im Menü, DPR und Skalierungsparameter gelten wie bisher.

| Bedeutung | Standardtext auf vereinfachtem Chinesisch | Verwendung |
| --- | --- | --- |
| Containerdatei wie ZIP oder TAR | 压缩包 | 压缩包名称, 空压缩包, 压缩包信息 |
| Ausgeschriebene Aktionen | 打开压缩包, 创建压缩包, 解压 | Bei eindeutigem Kontext in der Symbolleiste sind „打开“ und „新建“ möglich |
| Objekte im Archiv | 文件, 文件夹, 内容 | 所选文件和文件夹, 全部内容 |
| Dateieigenschaften | 原始大小, 压缩大小 | Gleiche Namen in Spaltenüberschriften, Vorschau und Sortiermenüs |
| Physische Eintragsreihenfolge wiederherstellen | 按原始顺序排列 | Von der Sortierung nach Name oder Größe unterscheiden |

„档案文件“ gehört nicht zur Produktterminologie. „归档“, „归档文件“ und „归档格式“ sind technischen Verträgen, Formatbeschreibungen und öffentlichen APIs vorbehalten. Die Oberfläche verwendet „压缩包“ auch für TAR, wenn es Dateien nur zusammenfasst. Format und Kompressionsstufe geben die tatsächliche Kompression an.

„压缩文件“ darf eine Aktion bezeichnen, etwa in „正在压缩文件“. Der Eigenschaftsname lautet „压缩大小“. „压缩后“ ist als Eigenschaftsname abgelöst. Das Erstellungsfenster schlägt „新建压缩包.zip“ vor und belässt bestehende Dateinamen.

Von Benutzern vorgegebene Namen und Pfade, Namen innerhalb von Archiven, Protokollfelder, Codebezeichner und historische Entwurfsbilder bleiben im Original. Englisch verwendet das übliche „archive“ nach eigenen Sprachregeln und „All contents“ für „全部内容“. In schmalen Spaltenüberschriften kürzt „Compressed“ den Ausdruck „compressed size“ ab. Englische Codebezeichner bleiben erhalten.

Siehe [Entwurf der Terminologiezustände](../../../design/hifi/terminology.html). Originalbilder, Prüfsummen und Messungen der sichtbaren Textfläche bleiben historische Nachweise. Das [Skript zur Inhaltsmessung](../../../scripts/check-html-body.py) prüft neue Texte über explizite Zuordnungen alter Bezeichnungen. Geometrievorgaben und Toleranzen gelten weiter. Prüfen Sie, ob längere Texte im Hauptfenster, in Tabellenköpfen, in der Vorschau und in Menüs vollständig angezeigt werden.

`python3 scripts/check-ui-i18n.py` prüft alle Sprachressourcen und die Terminologie für vereinfachtes Chinesisch. Die Terminologieprüfung gilt für statische Produkttexte. Sie erlaubt die ausdrücklich zugelassene Verwendung von „压缩文件“ als Verb und erfasst weder Benutzerinhalte noch Fachbegriffe in technischen Dokumenten.

Dateimanager behalten die in [#270](https://github.com/xiaodaozhi/xdzip/issues/270) bestätigten Einträge „添加到压缩文件...“ und „解压每个压缩文件到单独的文件夹“. Die Ausnahme gilt für den vollständigen Text dieser beiden stabilen IDs. An anderen Stellen gelten die vorgesehenen Namen für Container.
