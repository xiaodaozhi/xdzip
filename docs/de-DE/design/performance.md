<a name="xdzip-performance-design-and-measurement"></a>

# Leistungsentwurf und Messung

[English](../../design/performance.md) | [简体中文](../../zh-CN/design/performance.md) | [繁體中文（台灣）](../../zh-TW/design/performance.md) | [日本語](../../ja-JP/design/performance.md) | [한국어](../../ko-KR/design/performance.md) | [Français](../../fr-FR/design/performance.md) | Deutsch | [Español](../../es-ES/design/performance.md) | [Português (Brasil)](../../pt-BR/design/performance.md) | [Русский](../../ru-RU/design/performance.md)

XDZip begrenzt Speicher und Arbeit für Indizierung, Transport, Aufgabenausführung und Bildvorschau getrennt. Die folgenden Codegrenzen sind Implementierungsregeln. Zeiten, Speichermessungen und Durchsatz hängen von Build und Arbeitslast ab. Die später genannten Referenzziele sind technische Ziele.

<a name="xdzip-limits-and-mechanisms-in-the-current-code"></a>

## Grenzen und Mechanismen im aktuellen Code

| Bereich | Aktuelles Verhalten und Kosten |
| --- | --- |
| Gewöhnliches einteiliges ZIP/ZIP64 | Der Streaming-Scanner lässt bis zu eine Million physische Datensätze zu, begrenzt durch Namens-, Verzeichnis- und Lesebudgets. Wenn aktive Metadaten 64 MiB überschreiten oder ein neuer Datensatz über 100.000 residente Einträge führen würde, wandern Datensätze in privaten Festplattenspeicher. Die Metadatendatei ist auf 512 MiB begrenzt, mit separater Positionstabelle. Beim Durchsuchen werden Metadaten gescannt statt alle Nutzdaten zu dekodieren. |
| Abfragen in Speicher und auf Festplatte | Die Laufzeit erstellt einmal die aktuelle unveränderliche Ergebnisreihenfolge. Der Festplattenpfad lagert Sortierläufe bis 8 MiB aus und führt sie zu einer Tokenreihenfolge zusammen. Namens- und Sortierspeicher sind begrenzt. Der Cache enthält die aktuelle Abfrage statt einer vollständig gerenderten Ansicht oder unbegrenzter Abfragehistorie. |
| Navigation und Qt-Übertragung | Der Verzeichnisbaum hat Budgets von 32.768 Knoten, 64 Ebenen und 4 MiB Namen. Bei Überschreitung kann die Laufzeit Wurzeldurchsicht und Suche erhalten. Die Brücke sendet pro Seite bis zu 128 Zeilen und 64 KiB Namen. Große Ansichten halten bis zu 1.024 residente Zeilen bei logischer Zeilenzahl für das gesamte Ergebnis. |
| Befehle, Aufgaben und Schreiben | Metadaten- und Aufgabenthread verwenden getrennte begrenzte Warteschlangen. Schwere Desktopaufgaben werden einzeln zugelassen. Dekodierarbeit, Ausgabebytes, Datei-Commits und Workerressourcen werden getrennt abgerechnet. Solide und Streaming-Formate können vor einem ausgewählten Eintrag vorherige Daten lesen oder dekodieren. |
| Vorschau und native Worker | Text und kodierte Bilder haben getrennte Bytegrenzen. Bildmaße und Pixelzahl, Workerausführung und Miniaturgröße sind unabhängig begrenzt. Formatworker haben eigene Protokoll- und Prozessgrenzen. |

[Festplattenindex](../../../crates/xdzip-runtime/src/disk_index.rs), [Index großer Archive](../engineering/large-index.md), [Qt/Rust-Brückengrenzen](qt-rust-bridge.md), [Aufgabenzulassung](task-scheduling.md), [Ausgabebudgets](file-security.md), [Vorschaubudgets](../engineering/content-preview.md)

Die Speicherkosten umfassen Qt, Schriften, Rendering und Modelle, Archivmetadaten, Positionstabelle und Abfragepuffer, Decoderzustand, Kindprozesse und E/A-Puffer. Der Migrationsauslöser bei 64 MiB ist eine Metadatenschwelle. Prozessbaumspeicher enthält auch alles außerhalb dieses Index. Größere Metadatenzulassung erweitert weder Extraktionsquoten noch aktiviert sie Gesamtarchivprüfung oder -bearbeitung für festplattengestützte Sitzungen. Privates Indexschreiben und Extraktionsvorbereitung verursachen getrennte Festplattenkosten.

<a name="xdzip-measuring-the-current-implementation"></a>

## Aktuelle Implementierung messen

| Einstieg | Messung oder Prüfung |
| --- | --- |
| `scripts/probe-large-index.py` und `xdzip-m2-probe` | Erzeugen eigenes ZIP64, nutzen echten Scanner, Sortierung und paginierte Abfragen und berichten Phasen und Speicher getrennt. Duplikate, lange Namen und Verzeichnisstruktur einbeziehen und angeben, ob der Festplattenpfad erreicht wurde. |
| `scripts/measure-desktop.py` und `xdzip-process-metrics` | Messen Desktop-Prozessbaum, Start und Durchsuchen. Mit Extraktionsoption eine echte Extraktion ausführen und Ausgabebytes unabhängig prüfen. Renderer, Buildidentität und genaue Arbeitslast erfassen. |
| `scripts/run-performance-suite.py` | Erfasst Release-Beobachtungen für Start, stabilen Leerlauf und Durchsuchen und vergleicht Extraktion mit einer festgelegten Kernsonde auf eigenen Testdaten. Der Bericht erfasst Build- und Testdatenidentitäten. Der Vergleich ist beobachtend; Freigabe der Referenzmaschine bleibt eine separate Bedingung. |
| `scripts/check-runtime-resources.py` | Prüft Laufzeitressourcen- und Schriftbeleg der Sonde. Er belegt Ressourcenidentität und -abdeckung für diese Sonde, ohne interaktive Latenz zu messen. |

[Indexsonde](../../../scripts/probe-large-index.py), [Desktopmessung](../../../scripts/measure-desktop.py), [Leistungssuite](../../../scripts/run-performance-suite.py), [Ressourcensonde](../../../scripts/check-runtime-resources.py), [Nachweissammlung](../../../developers/engineering/candidate-verification.md#performance-and-resource-evidence)

Für Vergleiche Commit und Konfiguration, Betriebssystem und Architektur, Renderer, Hardware, Anzahl der Durchläufe, Testdaten, Geräte- und Cachebedingungen erfassen. Kalt- und Wiederholungsstarts trennen. Soweit messbar erste Interaktion, Archivscan, Abfragesortierung, Seitenübertragung und sichtbares Rendering als getrennte Phasen berichten. Abbruchrückmeldung von tatsächlichem Stopp und Bereinigung unterscheiden. Eingabe-, dekodierte und festgeschriebene Bytes beim Durchsatz getrennt ausweisen.

Das Prozessbaumwerkzeug kennzeichnet Linux PSS, macOS Physical Footprint und Windows Private Commit getrennt und berichtet bei Verfügbarkeit auch RSS oder Working Set. Dieselbe Metrik unter vergleichbaren Bedingungen vergleichen. Download- und installierte Größen tatsächlicher Pakete getrennt angeben. Offscreen-Zeiten und Prototypläufe beschreiben nur ihre jeweiligen Bedingungen.

<a name="xdzip-workloads-for-comparative-tests"></a>

## Arbeitslasten für Vergleichstests

| Arbeitslast | Messungen |
| --- | --- |
| B01/B08 Leeres Fenster, Leerlauf und Themenwechsel | Start- und Framezeiten, Leerlauf-CPU, gehaltener Speicher und Renderer. |
| B02 100.000 und eine Million ZIP-Datensätze | Metadatenscan, Migration, Sortierung, Seitenabruf und getrennte Speicher- und Festplattenkosten. |
| B03/B04 Eine große reguläre Datei oder viele kleine Dateien | Durchsatz, Abbruch, Zielarbeit, Synchronisierung und Bereinigung. Bei kleinen Dateien können Dateioperationen dominieren. |
| B05/B06 Stark komprimierte oder tief im Strom liegende Einträge | Quotenbehandlung und zusätzliche Dekodierarbeit vor dem ausgewählten Eintrag. |
| B07 Ausstehende schwere Arbeit beim Durchsuchen | Such- und Seitenlatenz, während eine schwere Operation läuft und eine andere wartet. |
| B09 Mitgelieferte Worker und vollständiges Paket | Prozessbaumnutzung, Protokollkosten, Download- und installierte Größe. |

Dies sind Vergleichsszenarien. Die zur Änderung passende Arbeitslast und projekteigene Testdaten mit erfassten Größen und Hashes wählen.

<a name="xdzip-reference-targets-and-planned-research"></a>

## Referenzziele und geplante Forschung

Der bestehende Entwurf nutzt eine Apple-Silicon-Referenz mit 16 GiB Speicher, lokaler SSD, 60 Hz Bildschirm, Systemschrift, echter GPU und Release-Build. Die Ziele sind Kandidaten für vergleichbare Messungen. Sie behaupten nichts über den neuesten Release oder andere Plattformen.

| Szenario | Entwurfsziel |
| --- | --- |
| Wiederholter Start eines leeren Fensters | P95 höchstens 500 ms bis zum interaktiven Frame bei mindestens 30 Durchläufen |
| 100.000 gecachte ZIP-Datensätze öffnen und abfragen | P95 höchstens 1 s für Metadatenvorbereitung und höchstens 80 ms von indizierter Abfrage bis Sichtbarkeit |
| Warmer Seitensprung und GUI-Planung | P95 höchstens 50 ms für sichtbaren Seitensprung und 8 ms für kontinuierliche GUI-Threadblockierung |
| Speicher | Prozessbaumziel höchstens 150 MiB leer und 256 MiB beim Durchsuchen von 100.000 Einträgen ohne Vorschau oder Decoderwörterbuch |
| Leerlauf-CPU | Nach Stabilisierung durchschnittlich höchstens 0,5% eines logischen Kerns |
| Abbruch | P95 höchstens 50 ms für Rückmeldung und 250 ms für lokalen kooperativen Streaming-Stopp, getrennt gemessen |
| Extraktion und Distribution | GUI und Kern bei identischer Arbeitslast und E/A vergleichen, mit Entwurfsziel höchstens 10% Durchsatzverlust. Mac-Ziele für eine Architektur: höchstens 100 MiB Download und 250 MiB installiert. |

Blockweise parallele Extraktion, dauerhafte Indexwiederverwendung und gemeinsamer CPU-Token-Scheduler bleiben Entwurfsoptionen. Vor Änderung von Parallelität oder Speicher den aktuellen seriellen Aufgabenpfad und Speicher-/Festplattenpfade profilieren. Bei Vergleichen gleichwertige Byteprüfung, Abbruch und sichere Veröffentlichung erhalten. [Architekturentwicklung](architecture-evolution.md), [Dateiveröffentlichung](file-security.md)
