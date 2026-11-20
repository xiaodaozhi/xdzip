<a name="xdzip-interface-contracts-and-checks"></a>

# Schnittstellenverträge und Prüfungen

[English](../../engineering/contracts.md) | [简体中文](../../zh-CN/engineering/contracts.md) | [繁體中文](../../zh-TW/engineering/contracts.md) | [日本語](../../ja-JP/engineering/contracts.md) | [한국어](../../ko-KR/engineering/contracts.md) | [Français](../../fr-FR/engineering/contracts.md) | Deutsch | [Español](../../es-ES/engineering/contracts.md) | [Português (Brasil)](../../pt-BR/engineering/contracts.md) | [Русский](../../ru-RU/engineering/contracts.md)

Die maßgeblichen Schnittstellen stehen in [Domäne und Daten](../design/domain-data.md), der [Qt/Rust-Brücke](../design/qt-rust-bridge.md), dem [Entwurf der Formaterweiterung](../design/codec-extension-sdk.md) und der [Aufgabenplanung](../design/task-scheduling.md). Diese Seite beschreibt die Prüfungen vor dem Festschreiben der Schnittstellen. Die interne Brücke, die beiden Handles und der Lebenszyklus sind implementiert. Ihre Grenzen stehen im [Laufzeitvertrag](product-runtime.md). Abfragegenerationen, Quellversionsprüfungen sowie Erstellungs- und Entpackaufgaben sind ebenfalls implementiert. Siehe [Archivansicht](../../engineering/readonly-browser.md), [sicheres Entpacken](zip-safe-extraction.md) und [Erstellung](../../engineering/zip-creation.md). Das Protokoll für externe Prozesse hat einen eigenständigen [schreibgeschützten SDK-Versuch](sdk-trial.md). Es ist weder als stabiles SDK veröffentlicht noch ins Produkt integriert. Die Tabelle nennt die Integrationsbedingungen.

| Schnittstelle | Vor der Implementierung bestätigen | Zu prüfende Fehlerfälle |
| --- | --- | --- |
| UI → Brücke | Handle-Besitz, RequestId, nichtblockierendes Einreichen, Ablehnung beim Beenden | Volle Warteschlange, wiederholtes Beenden, zerstörtes Objekt |
| Brücke → UI | session/query/generation/revision, Seitenblöcke, Ereignisfolge | Veraltete Antworten verwerfen, Fehler bei engerer Ganzzahlkonvertierung, Endzustand erhalten |
| Laufzeit → Codec | Funktionen statt Endungen, Eingabeleser, Ausgabesenke, Abbruchtoken | Nicht unterstützte Funktionen, CRC-Fehler, unvollständiges Lesen oder Schreiben |
| Laufzeit → sichere E/A | Autorisierter Stamm, Zuordnung, Vorbereitung, Freigabe und Beleg für die Übernahme | Rennen um Links, voller Datenträger, geändertes Ziel beim Überschreiben |
| Erweiterung → Host | Handshake-Version, Frame-Grenzen, Fähigkeiten, Sitzungslebenszyklus | Fehlerhafte oder zu große Frames, Zeitüberschreitung, Reihenfolgeänderung, verweigerter Abbruch |

Jede Schnittstelle definiert gültige Eingaben, Besitz, Threads, Ressourcenbudgets, Fehlerkategorien, Idempotenz und Abbruchgrenzen. Unbekannte Werte unterscheiden sich von null. TaskId und EntryId haben unterschiedliche Aufgaben. Die Identität einer Auswahl bezieht sich auf die Einträge und ist unabhängig von den aktuellen Zeilennummern.

CLI und GUI nutzen dieselben Rust-Dienste. Catalog liefert pro Seite höchstens 128 physische Metadateneinträge. SelectMembers löst vollständige effektive Pfade und EntryId-Werte in derselben unveränderlichen Sitzung auf und reicht dann vorhandene Aufgaben ein. Listen-IDs gehören zu ihrem Scan und sind nach Änderungen am Archiv nicht dauerhaft gültig. submit_wait wartet nur auf die kurz gehaltene Aufnahmesperre. Regeln für Kapazität und Ablehnung beim Beenden bleiben unverändert. Die GUI gibt die QML-Kodierung dieser internen Befehle nicht aus. Die CLI schreibt keine Dateien anhand maskierter Anzeigetexte.

Feste Host-Metadaten verwenden open_regular_nofollow zusammen mit Verzeichnisberechtigungen sowie Größen- und Identitätsprüfungen. Symbolische Links und Reparse-Punkte als letztes Pfadelement werden abgelehnt. Für bereits vom Benutzer ausgewählte Archive gelten die vorhandenen Regeln für Pfad-Aliasse. Desktop-CLI-Anfragen schicken ein begrenztes Anfrageverzeichnis über die optionale Einzelinstanz-Erweiterung `cli_protocol=1` für denselben Benutzer. Das alte Öffnungsprotokoll bleibt Version 1. Eine Instanz ohne Erweiterung kann die CLI-Übergabe nicht bestätigen. Siehe [CLI-Vertrag](command-line.md) für Ergebnisse, Exitcodes und Installationsgrenzen.

Interne Verträge können sich mit der Implementierung ändern und werden im gleichen Commit aktualisiert. Das Protokoll für Drittanbieter ist erst v1-stabil, wenn zwei unabhängige Implementierungen interoperabel sind und die Kompatibilitätstests bestehen. Unbekannte Versionen werden ausdrücklich abgelehnt oder nach Aushandlungsregeln auf eine kompatible Version herabgestuft.
