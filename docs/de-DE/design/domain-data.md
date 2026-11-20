<a name="xdzip-domain-model-and-archive-data"></a>

# Domänenmodell und Archivdaten

[English](../../design/domain-data.md) | [简体中文](../../zh-CN/design/domain-data.md) | [繁體中文](../../zh-TW/design/domain-data.md) | [日本語](../../ja-JP/design/domain-data.md) | [한국어](../../ko-KR/design/domain-data.md) | [Français](../../fr-FR/design/domain-data.md) | Deutsch | [Español](../../es-ES/design/domain-data.md) | [Português (Brasil)](../../pt-BR/design/domain-data.md) | [Русский](../../ru-RU/design/domain-data.md)

Die Laufzeit besitzt die Archivsitzung, den abgeschlossenen Index und die letzte Abfrage dieser Sitzung. Qt hält eine Ansicht des Ergebnisses; die CLI kann physische Datensätze über ihren eigenen Katalogbefehl lesen. Beide übergeben Operationen an denselben Rust-Dienst. [Systemarchitektur](system-architecture.md), [Qt/Rust-Brücke](qt-rust-bridge.md)

<a name="xdzip-identity-and-source-version"></a>

## Identität und Quellversion

| Identität oder Daten | Aktuelle Darstellung und Geltungsbereich |
| --- | --- |
| `SessionId` | Laufzeitidentität ungleich null für ein geöffnetes Archiv. Jedes erfolgreiche neue Öffnen erzeugt eine neue Sitzung. Dateipfade dienen Eingabe und UI-Routing. |
| `SourceVersion` | Die Laufzeit erfasst Identität, Länge und verfügbare Zeitstempel der regulären Datei jedes zugelassenen Teils. An relevanten Lese- und Schreibgrenzen prüft sie geöffnetes Objekt und zugehörigen Pfad erneut. Änderungen werden nach bestem Bemühen erkannt. |
| `EntryId` | Identifiziert einen physischen Archivdatensatz innerhalb einer Sitzung. Gleichnamige Datensätze behalten unterschiedliche IDs. |
| Navigationsidentität | Repräsentiert ein indiziertes Verzeichnis. Ein aus Nachfahren erzeugtes virtuelles Verzeichnis kann ohne physische Eintrags-ID im Baum oder der Liste erscheinen. |
| `QueryKey` | Kombiniert Sitzung, Abfrage-ID, Generation und Snapshot-Revision. Offset bezeichnet eine Position in diesem Ergebnis. |
| Anfrage-, Aufgaben- und Abfrageaufforderungsidentität | Der Dienst vergibt Anfrage- und Aufgabennummern innerhalb einer Verbindung. Eine Aufforderung gehört zu einer bestimmten Aufgabe. Der gemeinsame Desktop-Aufgabeneintrag nutzt die Sitzungsidentität des Controllers und die Aufgabennummer. |

Domänentypen erzwingen Archiv- und Abfrageidentitäten ungleich null. An der nativen Brücke werden typisierte Rust-Werte in generierten CXX-Daten dargestellt. QML erhält undurchsichtige Zeichenfolgen für IDs, die alle 64 Bits benötigen. Eine Zeilennummer ist nur eine Position in einer bestimmten Abfrage. [Domänenidentitäten](../../../crates/xdzip-domain/src/identity.rs), [Quellimplementierung](../../../crates/xdzip-runtime/src/source.rs), [Anwendungssitzungen](../../engineering/application-sessions.md)

<a name="xdzip-physical-entries-and-output-names"></a>

## Physische Einträge und Ausgabenamen

`xdzip-codec-api::Entry` enthält Format, physische Identität, Namensdarstellungen, Größeninformationen, Dateitypkennzeichen, Änderungsinformationen sowie formatabhängige Methoden- und Verschlüsselungsfelder. Rohe Namensbytes, eine optionale dekodierte UTF-8-Schreibweise, die Kodierungsentscheidung und maskierter Anzeigetext werden getrennt gespeichert. Für Ausgabe validiert die Laufzeit den wirksamen Namen nach der Richtlinie für portable Pfade und dem autorisierten Ziel. Ein ungeklärter Name kann sichtbar bleiben, während die Extraktion dieses Datensatzes oder Verzeichnisses abgelehnt wird.

Das Originalgrößenfeld besitzt ein eigenes `size_known`-Kennzeichen; so unterscheiden sich Dateien bekannter Größe null von unbekannten Größen. Manche komprimierte Container haben keine komprimierte Größe pro Eintrag. Methodenwerte werden formatspezifisch interpretiert. Zeitdaten behalten ebenfalls Formatkonventionen, etwa lokale DOS-Zeit bei ZIP; fehlende Zeit bleibt unbekannt. `payload_supported()` bestimmt, ob der aktuelle Datensatz dekodiert werden kann. Ausgabe hängt zusätzlich von gewählter Operation, Quellzustand, Name und Dateisystemregeln ab. [Eintragsfelder](../../../crates/xdzip-codec-api/src/lib.rs), [Dateinamen](../engineering/filename-encoding.md), [Dateiberechtigung](file-security.md)

<a name="xdzip-index-query-and-page-lifecycle"></a>

## Lebenszyklus von Index, Abfrage und Seite

Die Laufzeit öffnet die Quelle und schließt ihren begrenzten Scan ab, bevor Archiv und erste Seite veröffentlicht werden. Der Scan kann abgebrochen werden. Beschädigtes Ende, fehlendes Passwort oder Quelländerung führt zu keinem teilweise veröffentlichten Index. Navigation besitzt ein separates Budget. Überschreitet der Verzeichnisbaum dieses, können Wurzeldurchsicht und physische Suche weiterhin unterstützt werden. Gewöhnliches einteiliges ZIP/ZIP64 kann über den privaten Festplattenpfad bis zu eine Million Datensätze halten. Andere Formate und geteilte Pfade behalten eigene Grenzen.

`Archive::query_page` akzeptiert Verzeichnis, Text, Sortiermodus, Offset und erwartete Abfrageidentität. Eine neue Abfrage beginnt bei Offset null mit neuer Generation. Die Laufzeit erstellt eine vollständige Ergebnisreihenfolge und hält eine aktuelle unveränderliche Abfrage pro Archiv. Seiten lösen Zeilenidentitäten aus dieser Reihenfolge auf. Suche und Sortierung erhalten die ursprünglichen Identitäten der Archivdatensätze. Jeder Sortiermodus gruppiert Verzeichnisse vor Dateien, löst Gleichstände deterministisch nach Identität und ordnet unbekannte Größen innerhalb einer Gruppe nach bekannten ein.

Das generische Domänenobjekt `PageRequest` validiert bis zu 256 Zeilen. Der Desktop-Laufzeittransport fordert bis zu 128 Zeilen mit 64 KiB Namensbudget an. Qt übernimmt kleine Ergebnisse vollständig und hält für große Ergebnisse höchstens 1.024 residente Zeilen. Antworten mit abweichender Sitzung, Abfrage, Generation, Revision oder Offset können aktuelle Daten nicht ersetzen. [Abfrageimplementierung](../../../crates/xdzip-runtime/src/query.rs), [Auflistung und Sortierung](../../../crates/xdzip-runtime/src/listing.rs), [Index großer Archive](../engineering/large-index.md)

<a name="xdzip-selections-and-operation-input"></a>

## Auswahl und Operationseingaben

Das Qt-Modell speichert Auswahl geladener Ergebnisse nach ID und Auswahl großer Ergebnisse als begrenzte Bereiche im aktuellen Ergebnis. Alles auswählen umfasst dieses Ergebnis. Eine Aufgabe kann das gesamte Archiv von einer ausdrücklich gewählten Menge an Suchergebnissen unterscheiden. Rust löst seitenübergreifende Selektoren zur Extraktionsbestätigung über den vollständigen Abfrageschlüssel mit `FreezeSelection` auf. Verzeichniserweiterung löst tatsächliche Nachfahren auf und entfernt doppelte physische IDs. Überschreitung der Operationsgrenze gibt einen Fehler zurück. Die CLI löst Mitgliedspfade und physische IDs mit `SelectMembers` auf.

Extraktion, Prüfung und Bearbeitung bereiten operationsspezifische Jobs mit benötigter Quelle und gespeicherten Metadaten vor. Erstellung erhält Optionen und scannt und validiert danach einen separaten `InputTree`. Metadatenvorprüfung kann mögliche Probleme beschreiben, ohne einer Aufgabe Ausgabebefugnis zu geben. Die Laufzeit prüft Eingaben erneut und erhält vor dem Schreiben die entsprechende Berechtigung. Sobald ein Job seine Auswahl besitzt, ändern UI-Änderungen dessen Umfang nicht mehr. [Desktopmodell](../../../apps/desktop/BrowserController.cpp), [Laufzeitbefehle](../../../crates/xdzip-runtime/src/service.rs), [Aufgabenplanung](task-scheduling.md)

<a name="xdzip-results-and-data-lifetime"></a>

## Ergebnisse und Datenlebensdauer

| Daten | Lebensdauer |
| --- | --- |
| Aktuelles Archiv, Abfrage und Festplattenindex | Gehören zur Dienstsitzung. Abfrageersetzung oder Sitzungsschluss gibt zugehörige private Daten frei, vorbehaltlich von Jobs unabhängig gehaltener Ressourcen. Der Festplattenmetadatenindex wird beim erneuten Öffnen des Archivs neu erstellt. |
| Aufgaben-Snapshots | Die Laufzeit veröffentlicht begrenzte Snapshots mit tatsächlichem Status, Byte- und Elementanzahl, Bereinigungszustand und begrenzter Detailliste. Der Desktop hält Aufgabenergebnisse über den zuständigen Controller während der Anwendungssitzung verfügbar. |
| Benutzereinstellungen | Getrennt von Archivindizes und Aufgabenergebnissen verwaltet. Datenaufbewahrung und Datenschutz sind in Einstellungs- und Datenschutzverträgen dokumentiert. |
| Arbeitsbereiche und Journale | Private Ausgabevorbereitung und relative Veröffentlichungsdatensätze unterstützen Bereinigung und Prüfung. Ein Neustart wiederholt keinen unvollständigen Archivschreibvorgang. |

[Snapshot-Felder](../../../crates/xdzip-runtime/src/extraction/mod.rs), [Dateiveröffentlichung](file-security.md), [Einstellungen und Lokalisierung](../ux/settings-localization.md), [Datenlebenszyklus und Datenschutz](../engineering/privacy.md)

<a name="xdzip-design-options-outside-the-current-implementation"></a>

## Entwurfsoptionen außerhalb der aktuellen Implementierung

Inkrementelle Veröffentlichung durchsuchbarer Indizes, ein wiederverwendbarer Plan mit eigener Revision und eigenem Starttoken sowie dauerhafte seitenweise Aufgabenhistorie sind Entwurfsoptionen. Aktuelle Befehle bereiten eigene begrenzte Jobs vor und reihen sie ein, halten eine aktuelle Abfrage und liefern begrenzte Aufgaben-Snapshots. Dauerhafte Wiederherstellung erforderte zusätzlich einen separaten Entwurf zur Quell- und Ausgabevalidierung.
