<a name="xdzip-xdzip-design-guide"></a>

# Entwurfsleitfaden für XDZip

[English](../../design/index.md) | [简体中文](../../zh-CN/design/index.md) | [繁體中文（台灣）](../../zh-TW/design/index.md) | [日本語](../../ja-JP/design/index.md) | [한국어](../../ko-KR/design/index.md) | [Français](../../fr-FR/design/index.md) | Deutsch | [Español](../../es-ES/design/index.md) | [Português (Brasil)](../../pt-BR/design/index.md) | [Русский](../../ru-RU/design/index.md)

Die [Systemarchitektur](system-architecture.md) zeigt, wie der Qt-Quick-Desktop und die eigenständige Rust-CLI auf die Archivlaufzeit zugreifen. Die Entwurfsseiten erklären aktuelle Verträge, Entscheidungen und den Umfang vorgeschlagener Arbeiten. Installation und alltägliche Vorgänge stehen im [Benutzerhandbuch](../user-guide/README.md). Zum Bauen oder Ändern siehe [Bauen und beitragen](../contributing/README.md).

<a name="xdzip-find-a-design-topic"></a>

## Entwurfsthemen finden

| Thema | Dokumente |
| --- | --- |
| Produktverhalten | Die [Produktanforderungen](product-requirements.md) beschreiben erwartete Vorgänge und ihren Umfang. Die [Nachverfolgbarkeit](requirements-traceability.md) nennt die zugehörigen Prüfungen. |
| Archivdaten | Das [Domänenmodell](domain-data.md) beschreibt Sitzungen, Quellversionen, physische Einträge, Abfragen und Vorgangseingaben. |
| Desktop | Die [Qt-Quick-Oberfläche](qt-quick-ui.md) beschreibt Ansichten, Besitz der Controller und Interaktion. Die [Qt/Rust-Brücke](qt-rust-bridge.md) nennt tatsächliche Befehle, IDs, Ereignisse und Lebenszyklen. |
| Formate und Erweiterungen | [Formatadapter und Erweiterungsversuch](codec-extension-sdk.md) beschreiben das integrierte Register, den mitgelieferten Prozess und den eigenständigen XDP1-Versuch. |
| Ausgabe und Hintergrundarbeit | [Dateiberechtigung und sichere Veröffentlichung](file-security.md) behandelt Zwischenspeicherung, Prüfung und endgültige Übernahme. Die [Aufgabenplanung](task-scheduling.md) beschreibt Annahme, Rückfragen, Fortschritt und Abbruch. |
| Leistung und Qualität | Der [Leistungsentwurf](performance.md) erklärt Budgets, Messungen und Ziele. Die [Qualitätssicherung](quality-engineering.md) ordnet Änderungen Prüfungen zu. |
| Auslieferung und Entscheidungen | [Build, Bereitstellung und Veröffentlichung](build-release.md) behandelt Plattformen, Pakete und Updates. [Architekturentscheidungen](architecture-decisions.md) begründen die Wahl. Die [Architekturentwicklung](architecture-evolution.md) beschreibt den Weg vom Prototyp zum Produkt. |
| Begriffe | [Terminologie und Implementierungsgrenzen](glossary.md) erklärt Begriffe und verweist auf die aktuellen Grenzen jedes Teilsystems. |

<a name="xdzip-interpreting-the-references"></a>

## Referenzen einordnen

Aktuelles Verhalten ist mit Verweisen auf die Implementierung dokumentiert. Entwurfsoptionen und experimentelle Werkzeuge geben ihren Umfang an. Die [technische Referenz](../reference/README.md) führt zu den einzelnen Verträgen. Die [Projekt-README](../../../README.de-DE.md) und die zugehörigen Versionshinweise beschreiben Distributionen und ausgelieferte Funktionen. Zeit-, Ressourcen- und native Prüfangaben gelten für den dokumentierten Build, die Plattform und die Arbeitslast.
