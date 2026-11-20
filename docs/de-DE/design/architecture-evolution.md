<a name="xdzip-architecture-evolution-and-current-boundaries"></a>

# Entwicklung der Architektur und aktueller Umfang

[English](../../design/architecture-evolution.md) | [简体中文](../../zh-CN/design/architecture-evolution.md) | [繁體中文](../../zh-TW/design/architecture-evolution.md) | [日本語](../../ja-JP/design/architecture-evolution.md) | [한국어](../../ko-KR/design/architecture-evolution.md) | [Français](../../fr-FR/design/architecture-evolution.md) | Deutsch | [Español](../../es-ES/design/architecture-evolution.md) | [Português (Brasil)](../../pt-BR/design/architecture-evolution.md) | [Русский](../../ru-RU/design/architecture-evolution.md)

<a name="xdzip-from-framework-experiments-to-the-product"></a>

## Von Framework-Experimenten zum Produkt

Frühe Framework-Experimente verwendeten eine synthetische Archivliste und simulierte Aufgaben. Das Produkt übernahm Qt Quick und eine typisierte CXX-Brücke. Die tatsächlichen Archiv-, Aufgaben- und Dateioperationen wurden danach in Rust implementiert. JSON, C-ABI und Zeitmessungen des Prototyps gehören zu diesen Experimenten. Die [Architekturentscheidungen](architecture-decisions.md) halten Gründe und Bedingungen für eine Neubewertung fest.

Das Produkt begrenzt die Zeilenmodelle und prüft die Identität von Abfragen, damit veraltete Ergebnisse die neue Ansicht nicht überschreiben. Es ergänzt physische Eintrags-IDs, ausdrückliche Formatfähigkeiten, Dateiberechtigungen, Zwischenspeicherung und geprüfte Veröffentlichung. Die eigenständige CLI nutzt denselben Kern ohne Qt. Siehe [Systemarchitektur](../../design/system-architecture.md) und [Domänenmodell](domain-data.md).

<a name="xdzip-current-experimental-and-proposed-work"></a>

## Aktueller Stand, Experimente und Vorschläge

| Bereich | Aktueller Stand |
| --- | --- |
| Laufzeit und Indizierung | Im Host umgesetzt. Gewöhnliches ZIP/ZIP64 in einer Datei kann zu einem privaten Festplattenindex wechseln. Aufgaben- und Metadatengrenzen bleiben getrennt. [Leistungsentwurf](performance.md) |
| Formatisolierung | Die integrierte Formatauswahl verwendet Rust im Prozess oder den mitgelieferten Formatprozess. Registrierung, Profil und Laufzeitprüfung bestimmen, ob ein Vorgang unterstützt wird. [Formatadapter](codec-extension-sdk.md) |
| Externe Erweiterungen | Der eigenständige, schreibgeschützte XDP1-Versuch nutzt ein begrenztes Protokoll und plattformspezifische Voraussetzungen. Er läuft in einem getrennten Host. Suche, Installation und Aktivierung von Erweiterungen im Produkt bleiben Vorschläge. [SDK-Versuch](../engineering/sdk-trial.md) |
| Dauerhafter Zustand und Nebenläufigkeit | Die Planung lässt jeweils einen aufwendigen Desktopvorgang zu. Startübergreifende Indizes, Aufgabenwiederherstellung und eine übergreifende gemeinsame Planung bleiben Entwurfsoptionen. [Aufgabenplanung](task-scheduling.md), [Dateiveröffentlichung](file-security.md) |

Die [Formattabelle](../../../README.de-DE.md#xdzip-format-support) beschreibt den Umfang des aktuellen Quellcodes. Für einen veröffentlichten Build gelten seine Versionshinweise und Artefakte. Native Implementierung, experimentelle Werkzeuge und Messwerte haben jeweils ihren eigenen Geltungsbereich.
