<a name="xdzip-xdzip-terminology-and-implementation-limits"></a>

# XDZip-Begriffe und Implementierungsgrenzen

[English](../../design/glossary.md) | [简体中文](../../zh-CN/design/glossary.md) | [繁體中文（台灣）](../../zh-TW/design/glossary.md) | [日本語](../../ja-JP/design/glossary.md) | [한국어](../../ko-KR/design/glossary.md) | [Français](../../fr-FR/design/glossary.md) | Deutsch | [Español](../../es-ES/design/glossary.md) | [Português (Brasil)](../../pt-BR/design/glossary.md) | [Русский](../../ru-RU/design/glossary.md)

<a name="xdzip-1-terminology"></a>

## 1. Begriffe

| Begriff | Definition |
| --- | --- |
| Archiv | Container oder unterstützter komprimierter Datenstrom samt autorisierter Quelle. |
| Codec | Implementierung zum Erkennen, Scannen, Lesen oder Erstellen eines Formats. |
| Sitzung | Geöffnete Quelle, Backend, Index und Ansichtskontext. Aufgaben haben einen eigenen Lebenszyklus. |
| Eintrag | Physischer Archivdatensatz. Ein Ordner kann zusätzlich einen synthetischen Anzeigeknoten haben. |
| SourceVersion | Nach Möglichkeit ermittelter Fingerabdruck einer zugelassenen Quelldatei oder eines Volumes. |
| QuerySnapshot | Aktuelle unveränderliche Ergebnisreihenfolge, adressiert durch Sitzung, Abfrage, Generation und Revision. |
| Generation | Ablösung von UI-Anfragen, damit veraltete Ergebnisse neuere nicht überschreiben. |
| Vorbereiteter Auftrag | Begrenzte Laufzeiteingabe mit Quelle, Auswahl und Optionen für den Vorgang. |
| Aufgabe | Lebenszyklus und Ergebnis eines in die Aufgabenschlange aufgenommenen Auftrags. |
| Berechtigung | Zugriffsrecht auf Datei oder Verzeichnis im Besitz des Hosts, unabhängig vom Pfadtext. |
| Ausgabesenke | Schreibschnittstelle mit Messung, Quoten- und Transaktionskontrolle. |
| Worker | Befehle und Aufgaben verwenden Threads im Host. Archiv- und Bild-Worker laufen in eigenen Prozessen. |
| Übernahme | Geprüfte temporäre Ausgabe über den autorisierten Datei- oder Archivvorgang veröffentlichen. |
| Aufgabenmomentaufnahme | Neueste Meldung zu Zustand, Fortschritt, Rückfrage oder Ergebnis. Zwischenwerte können zusammengefasst werden. |
| Fähigkeiten | Schnittmenge aus Implementierung, Formatfunktionen, Plattform und Richtlinien. |
| Solid | Dateien teilen komprimierte Blöcke. Selektives Lesen kann das Dekodieren vorausgehender Inhalte benötigen. |
| Erster angezeigter Frame | Tatsächlich gerenderter und angezeigter Frame. Eine ui_ready-Meldung ersetzt ihn nicht. |

<a name="xdzip-2-units-and-values"></a>

## 2. Einheiten und Werte

Speicher- und Dateigrößen verwenden binäre KiB/MiB/GiB. Es gilt 1 MiB = 1024² Bytes. Transport und Abrechnung behalten ganzzahlige Bytewerte. Zeiten werden in ms oder s angegeben. Laufzeiten werden mit einer monotonen Uhr gemessen. Dateizeitstempel behalten Kalenderdarstellung und Quellgenauigkeit, soweit verfügbar. Durchsatz wird in MiB/s mit Angabe der Bezugsdaten gemessen, etwa komprimierte Eingabe, dekodierte Daten oder Ausgabe.

CPU-Auslastung von 100% eines Kerns bedeutet einen voll belegten logischen Kern. Ein Prozessbaum kann 100% überschreiten. PSS, RSS, USS und physischer Speicherbedarf messen verschiedene Größen. Komprimierte Downloadgröße, installierte Größe und EXE-Größe werden getrennt erfasst.

<a name="xdzip-3-where-to-find-current-limits"></a>

## 3. Aktuelle Grenzen finden

| Thema | Implementierungsreferenz |
| --- | --- |
| Anfragen, Seiten und Postfachgrenzen | [Qt/Rust-Brücke](qt-rust-bridge.md) |
| Aufgabenannahme, Fortschritt und Statuscodes | [Aufgabenplanung](task-scheduling.md) |
| Ausgabelimits, Dateiberechtigungen und Übernahme | [Dateisicherheit](file-security.md) |
| Metadaten- und Festplattenindexgrenzen für ZIP | [Große Archivindizes](../engineering/large-index.md) |
| Quell- und Volumegrenzen | [Archive öffnen](../engineering/archive-opening.md) |
| Grenzen der Text- und Bildvorschau | [Inhaltsvorschau](../engineering/content-preview.md) |
| Frames, Überwachung und Ressourcenbegrenzung | [Archiv-Worker](../engineering/native-codecs.md) |
| Experimentelles Erweiterungsprotokoll | [SDK-Versuch](../engineering/sdk-trial.md) |
| Leistungsziele | [Leistungsentwurf](performance.md) |

<a name="xdzip-4-versioning"></a>

## 4. Versionierung

Anwendungsversion, interne CXX-Buildschnittstelle, externes Übertragungsprotokoll und persistentes Datenschema haben getrennte Versionen. Die Produktversion allein bestimmt die SDK-Protokollversion nicht. Siehe [Entwurf des Format-SDK](codec-extension-sdk.md).
