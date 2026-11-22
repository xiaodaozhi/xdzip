<a name="xdzip-large-archive-indexing-and-on-demand-browsing"></a>

# Indizierung großer Archive und bedarfsgesteuertes Durchsuchen

[English](../../engineering/large-index.md) | [简体中文](../../zh-CN/engineering/large-index.md) | [繁體中文](../../zh-TW/engineering/large-index.md) | [日本語](../../ja-JP/engineering/large-index.md) | [한국어](../../ko-KR/engineering/large-index.md) | [Français](../../fr-FR/engineering/large-index.md) | Deutsch | [Español](../../es-ES/engineering/large-index.md) | [Português (Brasil)](../../pt-BR/engineering/large-index.md) | [Русский](../../ru-RU/engineering/large-index.md)

Die Grenze der Quellcodefähigkeit für gewöhnliche einteilige ZIP/ZIP64-Archive beträgt eine Million physische Einträge. Andere Formate und Mehrteilarchive behalten ihre bestehenden Zulassungsgrenzen von 100.000 Einträgen; der Formatname bedeutet nicht, dass alle Kombinationen erweiterte Grenzen haben. Verzeichnisse, doppelte Namen und prüfbedürftige Datensätze zählen als physische Einträge; implizite Navigationsverzeichnisse ergänzen nur Anzeigezeilen.

<a name="xdzip-scanning-and-storage"></a>

## Scannen und Speichern

ZIP scannt Datensätze einzeln mit dem gewöhnlichen ZIP-Parser. Zuerst wird eine begrenzte archivweite Stichprobe für Namen in alten Kodierungen abgeschlossen, danach folgt ein zweiter Durchlauf mit derselben Kodierungsentscheidung. Beide Durchläufe lesen zusammen höchstens 512 MiB; das Zentralverzeichnis umfasst höchstens 256 MiB, Rohname insgesamt höchstens 128 MiB, dekodierte und angezeigte Namen zusammen höchstens 128 MiB. Jeder rohe oder dekodierte Name bleibt auf 4 KiB begrenzt. Ein beschädigtes Ende, Abbruch oder eine Quellenänderung lässt das ganze Ergebnis fehlschlagen, ohne einen Teilindex zu veröffentlichen.

Bei 64 MiB aktiven Metadaten oder mehr als 100.000 Einträgen Datensätze einzeln in privaten Festplattenspeicher migrieren und vorhandene Namen und Datensätze freigeben, ohne zwei vollständige Indizes gleichzeitig zu halten. Die Tabelle physischer Positionen enthält höchstens eine Million u64-Offsets. Festplattenmetadaten umfassen höchstens 512 MiB; Dateinamen und Inhalte gelangen nicht in Diagnosemeldungen. Der Verzeichnisbaum bleibt auf 32.768 Knoten, 64 Ebenen und 4 MiB Namensbudget begrenzt. Bei Überschreitung wird der Baum nicht veröffentlicht; Wurzelansicht und physische Suche bleiben verfügbar.

Temporäre Dateien liegen in einem zufällig benannten Verzeichnis `xdzip-index-*` unter dem temporären Hostverzeichnis, das nur der aktuelle Benutzer lesen und beschreiben kann. Der Zugriff erfolgt über gehaltene Verzeichnis- und reguläre Dateihandles. Normales Schließen, Ersetzen einer Abfrage, Fehler und Abbruch bereinigen bekannte Dateien und identitätsgeprüfte leere Verzeichnisse. Nach Abstürzen verbliebene Dateien werden durch einen 16-Byte-Marker passend zur zufälligen Verzeichnisidentität und eine exklusive Lease erkannt. Spätere Erstellung scannt begrenzt und bereinigt nur bekannte ungenutzte Dateien, die älter als sieben Tage sind. Links, Identitätsabweichungen und unbekannte Objekte bleiben erhalten. Unbekannte Inhalte nicht rekursiv löschen und Indizes nicht über Programmstarts hinweg wiederherstellen oder wiederverwenden.

Unter Windows muss das neu erstellte Verzeichnis zuerst die gewöhnlichen Prüfungen von Eigentümer und Änderungsrechten bestehen, bevor seine DACL für vertrauliche Speicherung geschützt wird. Geerbte Änderungsrechte für unbeteiligte Prinzipale führen zur Ablehnung, bevor Lease oder Metadatendatei geschrieben werden; die Laufzeit verwendet nicht stillschweigend einen weniger privaten Ort. Für isolierte Prüfung nur `TMP` und `TEMP` des Testprozesses auf ein neues eigenes Verzeichnis richten, dessen ACL Änderungen nur für den aktuellen Benutzer, SYSTEM und Administrators erlaubt. ACLs echter Benutzerverzeichnisse nicht ändern und Berechtigungsprüfungen nicht lockern. Allokation, tatsächliche Festplattenabfragen und Zusammenführung, aktive Leases, Bereinigung bekannter Dateien und Ablehnung fremder Änderungsrechte getrennt prüfen.

<a name="xdzip-queries-and-selection"></a>

## Abfragen und Auswahl

Suche und Sortierung erzeugen eine unveränderliche vollständige Zeilenreihenfolge. Sortierschlüssel werden in private Merge-Läufe von jeweils höchstens 8 MiB geschrieben, mit höchstens 128 Läufen und 512 MiB temporärem Sortierspeicher. Endgültige Abfragen speichern nur Zeilentoken ohne eine weitere vollständige Namenskopie. Vergleiche bewahren ursprüngliche Ganzzahlen, bestehende Namens- und Typregeln sowie stabile physische Identität. Jede Sortierung stellt explizite und implizite Verzeichnisse vor Dateien; die Richtung gilt innerhalb jeder Gruppe. Unbekannte Größen stehen innerhalb ihrer Verzeichnis- oder Dateigruppe nach bekannten Werten; absteigende Sortierung ändert nicht die Identitätsreihenfolge von Duplikaten. Abbruch wird beim Scannen und rund um Sortierläufe und Zusammenführung geprüft, nicht durch Änderung der Ordnungsrelation im Vergleich.

Die Brücke überträgt pro Paket höchstens 128 Zeilen und 64 KiB Namen. Abfragen mit höchstens 1.024 Zeilen behalten das bisherige vollständige Commit-Verfahren. Größere Qt-Modelle behalten die volle logische Zeilenzahl und laden nur sichtbare Positionen mit höchstens 1.024 residenten Cachezeilen. Nicht geladene Zeilen können keine Eintragsaktionen auslösen. Jede verzögerte Seite bindet Sitzung, Abfrage, Generation, Revision und angeforderten Offset; veraltete Seiten dürfen nicht in neue Ergebnisse gelangen.

Seitenübergreifende Auswahl nutzt höchstens 4.096 begrenzte Bereiche in der aktuellen unveränderlichen Abfrage. Alles auswählen umfasst die gesamte aktuelle Abfrage; das Verlassen des Caches ändert die Auswahl nicht. Vor der Extraktionsbestätigung friert Rust Bereiche als physische oder Navigationsidentitäten ein, die auch nach einer Abfrageänderung gelten. Auswahlen über der Aufgabengrenze von 100.000 Einträgen werden ausdrücklich abgelehnt und nicht still verkleinert. Die Gesamtgröße nicht residenter Auswahlen wird derzeit nicht angezeigt. Massenkopieren in die Zwischenablage über den residenten Inhalt hinaus meldet eine Ressourcengrenze statt still eine unvollständige Liste zu kopieren.

<a name="xdzip-task-boundaries"></a>

Datenträgerabfragen verwenden für den Namensvergleich einen gemeinsamen Datensatzpuffer und erzeugen Eintragsdatensätze bei der Zeilenausgabe. Eine kleine Suche oder Sortierung gilt als laufend, bis alle Batches gesammelt und das Ergebnis übernommen wurde. Seitenanforderungen für den sichtbaren Bereich füllen den Cache einer bereits übernommenen Abfrage.

## Aufgabengrenzen

Aus einem Festplattenindex ausgewählte reguläre Dateien können nach vollständiger erneuter Quellprüfung, sicherer Planung, Prüfung des gesamten Eintrags und bestehendem autorisiertem Commit extrahiert werden. Ein eingefrorener Plan bleibt auf 100.000 Einträge und 64 MiB Metadaten begrenzt; Datei- und tatsächliche dekodierte Bytegrenzen folgen dem [Extraktionsvertrag](zip-safe-extraction.md). Verzeichniserweiterung darf ungeklärte Namen nicht überspringen; ausgewählte Links werden in dieser Indexkonfiguration abgelehnt. Der ZIP-Leser prüft physische Offsets des gesamten Zentralverzeichnisses erneut, behält nur Positionen ausgewählter Nutzdaten und erhält Prüfungen auf Überlappung, lokalen Header, Größe, CRC, Authentifizierung und Abbruch.

Archive mit Festplattenindex unterstützen derzeit weder vollständige Änderung noch Prüfung des gesamten Archivs; eine leere Speicherprojektion darf nicht als vollständiges Archiv gelten. Bestehende Grenzen für Erstellung, andere Formate, geteilte Volumes und Archivumschreiben bleiben unverändert. Siehe gesondert die [Inhaltsvorschau](content-preview.md). Physische CLI-Auflistung und Auswahl nutzen denselben Index; [explizite Seiteneinteilung](command-line.md#xdzip-archive-behavior) liefert den vollständigen physischen Bereich großer Archive. Antworten ohne Seiteneinteilung behalten Budgets von 100.000 Einträgen und 16 MiB serialisierten Daten oder Text und schlagen bei Überschreitung vollständig fehl.

<a name="xdzip-verification"></a>

## Prüfung

`scripts/probe-large-index.py` erzeugt unter `build/` ein echtes ZIP64 mit chinesischen Namen, Duplikaten und verschachtelten Verzeichnissen und ruft anschließend den tatsächlichen `xdzip-m2-probe` auf. Die Sonde liest jedes Abfragepaket und erfasst getrennt Scannen, Sortieren, Abrufen, Festplattenbytes und plattformverfügbare Speichermesswerte. Der Beispielgenerator erstellt kein Array von einer Million Dateinamen im Speicher. Umfang, Fehler, Abbruch, Bereinigung, veraltete Seiten und tatsächliche Ausgabe ausgewählter Dateien folgen den [Abnahmeanforderungen](../../../developers/qa/acceptance.md); Ergebnisse bleiben in Issues, lokal oder in CI-Artefakten. Messungen belegen nur die tatsächlich verwendete Maschine und Arbeitslast und garantieren keine allgemeine Leistung.
