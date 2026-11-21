<a name="xdzip-format-adapters-and-the-extension-trial"></a>

# Formatadapter und Erweiterungsversuch

[English](../../design/codec-extension-sdk.md) | [简体中文](../../zh-CN/design/codec-extension-sdk.md) | [繁體中文](../../zh-TW/design/codec-extension-sdk.md) | [日本語](../../ja-JP/design/codec-extension-sdk.md) | [한국어](../../ko-KR/design/codec-extension-sdk.md) | [Français](../../fr-FR/design/codec-extension-sdk.md) | Deutsch | [Español](../../es-ES/design/codec-extension-sdk.md) | [Português (Brasil)](../../pt-BR/design/codec-extension-sdk.md) | [Русский](../../ru-RU/design/codec-extension-sdk.md)

<a name="xdzip-three-execution-paths"></a>

## Drei Ausführungspfade

| Pfad | Implementierung und Verfügbarkeit |
| --- | --- |
| Integrierte Adapter im Host | Desktop und CLI verwenden dieselbe feste Formatregistry und Laufzeitweiterleitung. Rust-Implementierungen für ZIP, TAR, Stromfilter und unterstützte Container wie DEB und ASAR werden im Hostprozess aufgerufen. |
| Mitgelieferter Archiv-Worker | `xdzip-codec-native` im Host startet `xdzip-native-codec`. Seine C++-Protokollschicht ruft über `xdzip-codec-bridge` die Rust-Backends für 7z, RAR, CAB und optische Medien auf. Derselbe Weg behandelt zulässige 7z-Erstellung und -Umschreibung. ISO-Erstellung nutzt den optischen Writer im Prozess. |
| Externer SDK-Versuch | `xdzip-plugin-host` und `xdzip-plugin-protocol` bilden ein separates, ausdrücklich gestartetes XDP1-Experiment. Produktbuilds für Desktop und CLI entdecken, installieren oder starten keine Erweiterungen Dritter. |

Die ersten beiden Pfade sind integrierte Produktkomponenten. Der mitgelieferte Worker ist ein eigener Prozess mit privatem Protokoll und Ressourcenüberwachung. Der externe Versuch besitzt eigene Paketprüfung, Versionsaushandlung und plattformspezifische Berechtigungsgrenzen. [Systemarchitektur](system-architecture.md), [Archiv-Worker-Vertrag](../engineering/native-codecs.md), [SDK-Versuchsvertrag](../engineering/sdk-trial.md)

<a name="xdzip-product-routing-and-operation-capabilities"></a>

## Produktweiterleitung und Operationsfähigkeiten

Die [Quelle der Formatregistry](https://github.com/xiaodaozhi/xdzip/blob/main/specs/engineering/archive-formats.json) deklariert Endungen, Container- und Filtertypen, unterstützte Operationen, Methoden und Erstellungsoptionen. Der Generator hält Rust-Registry, Desktoplisten und Designartefakte synchron. Beim Lesen prüft die Laufzeit Quellbytes und validiert das Format vor der Backendwahl. Bei der Erstellung wählt der angeforderte Ausgabename ein Profil, gegen das die Laufzeit die Optionen prüft. Die Endungsliste unterstützt Entdeckung und UI-Hinweise. Erfolgreiche Erkennung und Scan bestimmen das geöffnete Format.

Lesen, Vorschau, Extraktion, Gesamtarchivprüfung, Erstellung und Umschreiben haben getrennte Grenzen. Ein gelisteter Eintrag kann sichtbar sein, ohne eine unterstützte Nutzdatenmethode zu besitzen. Passwörter, Volumes und spezielle Datensätze beeinflussen die Zulassung; die Laufzeit behandelt auch formatabhängige Unterschiede wie ISO/UDF-Strukturprüfungen und nicht unterstützte Umschreibprofile. [Laufzeitweiterleitung](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/formats.rs), [Formatregistry-Vertrag](../engineering/format-registry.md), [Archivprüfung](../engineering/archive-checking.md)

<a name="xdzip-internal-rust-adapter-interfaces"></a>

## Interne Rust-Adapter-Schnittstellen

| Schnittstelle | Verantwortung |
| --- | --- |
| `ReadSource` und `MetadataCodec` | Eine vom Host autorisierte Quelle einschließlich zutreffender Volume-Metadaten lesen oder positionieren und begrenzte Formatdatensätze erzeugen. |
| `PayloadCodec`, `PayloadReader` und `DecodedSink` | Inhalt auflösen und einen zulässigen physischen Datensatz in ein Host-Ausgabeziel dekodieren. Gewöhnliche Bytes, übersprungene Dekodierarbeit und Hilfsarbeit unterscheiden, wenn das Backend sie meldet. |
| `CreationCodec`, `EncodeInput` und `EncodeOutput` | Aus eingefrorenen Eingabemetadaten und indizierten Strömen kodieren. In ein dem Host gehörendes Ziel schreiben, bei Bedarf mit begrenzten Direktzugriffsschreibvorgängen. |
| `RewriteCodec` | Unterstützte Archive aus erhaltenen Originaleinträgen, Änderungen und autorisierten Ergänzungen in ein neues zwischengespeichertes Archiv umschreiben. |

Der Host besitzt Pfade, erneute Quellprüfung, Abbruch, Passwörter, Quoten und jede endgültige Veröffentlichung. Ein Codec erhält Fähigkeiten über die für die Operation nötigen Schnittstellen. Der Archiv-Worker fordert erlaubte Quellbereiche an und liefert begrenzte Datensätze oder Ausgabeblöcke. Der Host validiert Protokollzustand und Bytequoten vor jeder Aktion. Ein Workerfehler oder eine ungültige Antwort kann keine Veröffentlichung autorisieren. Die Workergrenze isoliert Fehler und Ressourcen. Kernel-Berechtigungsbeschränkungen für beliebigen externen Code sind eine separate Anforderung des Versuchs. [Metadatenverträge](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/lib.rs), [Nutzdatenverträge](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/payload.rs), [Kodier- und Umschreibverträge](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/encode.rs), [Dateiautorisierung](file-security.md)

Diese Rust-Traits sind interne Quellschnittstellen. Ein Produktadapter wird in den Workspace und die explizite Laufzeitweiterleitung integriert. Seine erlaubten Operationen benötigen getrennte Tests für gültige Dateien, nicht unterstützte Funktionen, Ressourcengrenzen, Beschädigung und Ausgabeverhalten. Nutzen Sie die [Verträge unabhängiger Rust-Codecs](../engineering/independent-rust-codecs.md) und den [Entwicklungsleitfaden](../contributing/README.md).

<a name="xdzip-implemented-external-sdk-trial"></a>

## Implementierter externer SDK-Versuch

Der Versuch akzeptiert ein ausdrücklich ausgewähltes Manifest eines nativen Pakets und eine vom Host gewählte Quelle. Er unterstützt das Scannen eines projekteigenen XDP1-Testdatensatzes und das Lesen eines regulären Eintrags. Die Paketzulassung validiert Manifestversion 1, ausführbares Ziel und Hash, paketinterne Pfade, festes Berechtigungsprofil und Grenzen. Der Host führt eine geprüfte private Kopie aus. Der deklarierte Hash belegt Paketkonsistenz; Herausgeberauthentifizierung benötigte eine eigene Vertrauensrichtlinie.

Das Versuchsprotokoll Version 1 verwendet längenpräfixiertes begrenztes CBOR und prüft Operationsidentität und Reihenfolge. Der Host gewährt Quelllesungen und ein Ausgabeziel über opake Token. Die Gegenstelle erhält Bytebereiche und Ausgabeguthaben für einen Block. Der Host misst und verifiziert Inhalt, prüft Protokollabschluss und Quellzustand und veröffentlicht ohne Überschreiben erst nach sauberem Ende der Gegenstelle. Das Beispiel legt der Gegenstelle weder Archivpasswörter, Split-Volumes, Erstellung, Updates, Links noch Hostpfade offen.

Der Versuch hat ausdrückliche native Berechtigungsschranken für macOS und Linux; Plattformanforderungen und Ablehnung bei Fehlern sind im Versuchsvertrag definiert. Windows verweigert die Ausführung. Native Interoperabilitäts- und Ablehnungstests prüfen diese Grenzen unabhängig vom Desktop. Schema, Budgets und Testeinstiege stehen im [Ausführungs- und Protokollvertrag des Versuchs](../engineering/sdk-trial.md). [Versuchshost](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-host/src/lib.rs), [Framing-Implementierung](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-protocol/src/lib.rs)

<a name="xdzip-design-options-outside-the-current-implementation"></a>

## Entwurfsoptionen außerhalb der aktuellen Implementierung

Ein produktseitiger Erweiterungsmanager, Herausgebervertrauen und Upgrade-Richtlinie, Kompatibilität über das Versuchsprotokoll hinaus sowie externe Schreib- oder Passwortfähigkeiten bleiben Entwurfsarbeit. Produktzulassung erfordert geprüfte Berechtigungsbeschränkungen, Interoperabilität und Fehlerverhalten für jede unterstützte Plattform und Fähigkeit. Der mitgelieferte Worker und das XDP1-Experiment aktivieren diese Funktion nicht. [Architekturentscheidungen](architecture-decisions.md)
