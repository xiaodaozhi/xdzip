<a name="xdzip-read-only-zip-browsing-contract"></a>

# Vertrag für die schreibgeschützte ZIP-Ansicht

[English](../../engineering/zip-readonly-browsing.md) | [简体中文](../../zh-CN/engineering/zip-readonly-browsing.md) | [繁體中文](../../zh-TW/engineering/zip-readonly-browsing.md) | [日本語](../../ja-JP/engineering/zip-readonly-browsing.md) | [한국어](../../ko-KR/engineering/zip-readonly-browsing.md) | [Français](../../fr-FR/engineering/zip-readonly-browsing.md) | Deutsch | [Español](../../es-ES/engineering/zip-readonly-browsing.md) | [Português (Brasil)](../../pt-BR/engineering/zip-readonly-browsing.md) | [Русский](../../ru-RU/engineering/zip-readonly-browsing.md)

Der Vertrag definiert direkte Einträge im Stammverzeichnis, die Suche in Unterordnern, eine stabile physische Identität und die Sortierung mit Ordnern zuerst. Die ZIP-Ansicht liest nur Metadaten. Sie dekodiert keine Eintragsinhalte, prüft deren CRC nicht und entpackt keine Dateien. Große gewöhnliche ZIP-Archive können einen privaten [Metadatenindex auf dem Datenträger](large-index.md) verwenden.

Die [Indizierung großer Archive](large-index.md) beschreibt den aktuellen Index für große normale ZIP-Archive und das bedarfsgesteuerte Qt-Modell. Diese Seite hält den ursprünglichen Grenzwert von 100.000 Einträgen und die Anforderungen an die Abfrageidentität fest. Spätere Kapazitätserweiterungen lassen die Prüfungen für Fehler, Abbruch und Identität weiterhin gelten.

<a name="xdzip-browsing-scope-and-limits"></a>

## Funktionsumfang und Grenzen

- ZIP/ZIP64-Metadaten mit höchstens 100.000 physischen Datensätzen. Der [Host für geteilte Volumes](archive-opening.md) wandelt die Teile vor dem Lesen um. Der Codec weist rohe Metadaten mehrerer Datenträger weiterhin ab.
- Höchstens 4 KiB pro Name, insgesamt 16 MiB für Rohnamen und 16 MiB für dekodierte und maskierte Anzeigenamen, 64 MiB insgesamt gelesene Daten und 256 KiB Scanpuffer. Deklarierte 64-Bit-Größen bleiben exakte Ganzzahlen.
- Verzeichnisindex mit höchstens 32.768 Knoten, 64 Ebenen und insgesamt 4 MiB für Namen und Präfixe. Wird eine Grenze überschritten, wird der gesamte Navigationsindex verworfen und die eingeschränkte Funktion erläutert. Ein unvollständiger Baum wird nicht ausgegeben.
- Indizes direkter Verzeichniseinträge und Abfrageergebnisse werden wiederverwendet. Übertragungsblöcke derselben Abfrage filtern und sortieren nicht erneut. Jeder Block begrenzt die Namen separat auf 64 KiB. Die Oberfläche zeigt das aktuelle Verzeichnis vollständig ohne Seitenschaltflächen an.
- Seiten prüfen Sitzung, Abfrage-ID, UI-Generation, Revision des Abbilds und Offset. Eine geänderte Abfrage bricht alte Arbeit ab und übernimmt die letzte Anforderung. Veraltete Antworten können neue Ergebnisse nicht ersetzen.
- Vor und nach dem Scan sowie bei Abfragen und Vorabprüfungen werden Quelllänge, Änderungszeit, Unix-Gerät und -Inode sowie Windows-Volumenummer und -Dateiindex geprüft. Bei `source_changed` leeren Qt-Abfragen und Vorabprüfungen die ungültige Liste, Auswahl, den Verzeichnisbaum und die Details. Öffnen Sie das Archiv erneut für eine neue Sitzung. Die Erkennung erfolgt nach Möglichkeit und dient weder als atomarer Dateisystem-Snapshot noch als Berechtigung zum sicheren Schreiben.

<a name="xdzip-acceptance-exit-criteria"></a>

## Abnahmekriterien

Ein tatsächlich erzeugtes ZIP64 mit 100.000 Einträgen lässt sich vollständig scannen, sortieren und abrufen. Prüfen Sie Verzeichnisnavigation, vollständige Qt-Liste und Auswahl. Alle Blöcke haben dieselbe Abfrageidentität. Prüfen Sie beschädigte ZIP64-Dateien, geteilte Volumes, Namens-, Anzeige- und Leselimits, Abbruch, veraltete Antworten sowie geänderte und ersetzte Quelldateien. Halten Sie tatsächliche Dauer, Ressourcenbedarf, Ereigniswarteschlange und Anzahl der Delegates fest. Werden Builds für alle drei Plattformen oder Apple-Silicon-Entwicklungspakete auf Anfrage geprüft, erfassen Sie deren Ergebnisse separat.

Filtern Sie die Workflow-Läufe des Repositorys nach head_sha und dem tatsächlichen Ereignis. Routinemäßige Prüfungen laufen bei Push oder PR. Die vollständigen Foundation- und Qt-Prüfungen starten bei Bedarf manuell. Dokumentieren Sie Ergebnisse und nicht ausgeführte Prüfungen im zugehörigen Issue. Eine leere Liste für ein anderes Ereignis belegt keinen Prüfstatus.
