<a name="xdzip-adaptive-filename-encoding-contract"></a>

# Vertrag zur adaptiven Dateinamenkodierung

[English](../../engineering/filename-encoding.md) | [简体中文](../../zh-CN/engineering/filename-encoding.md) | [繁體中文](../../zh-TW/engineering/filename-encoding.md) | [日本語](../../ja-JP/engineering/filename-encoding.md) | [한국어](../../ko-KR/engineering/filename-encoding.md) | [Français](../../fr-FR/engineering/filename-encoding.md) | Deutsch | [Español](../../es-ES/engineering/filename-encoding.md) | [Português (Brasil)](../../pt-BR/engineering/filename-encoding.md) | [Русский](../../ru-RU/engineering/filename-encoding.md)

<a name="xdzip-resolution-and-identity"></a>

## Auflösung und Identität

Die Dekodierung erfolgt in der Qt-unabhängigen Rust-Formatschicht. Sie verwendet weder Systemcodepages des Hosts, UI-Sprache, Vermutungen zum Quellsystem, Dateiinhalte noch Netzwerkdienste. Bei ZIP bewahrt raw_name die ursprünglichen Bytes des Namensfeldes. TAR bewahrt den von PAX/GNU gewählten Rohnamen, nachdem führendes `./` nach bestehenden Regeln entfernt wurde. Die physische EntryId ist unabhängig von Namen. decoded_name speichert eine UTF-8-Schreibweise, die Transkodierung oder eine zusätzliche Kodierungsdeklaration benötigt; name_encoding bewahrt den Ursprung der Auflösung und name_bytes liefert den eingefrorenen wirksamen Namen. display_name ist nur maskierte Anzeige und kann kein Ausgabepfad sein.

ZIP löst Namen in dieser Reihenfolge auf:

1. Bei gesetztem UTF-8-Flag gültiges UTF-8 im Originalfeld verlangen; eine beschädigte Deklaration nicht mit alter Kodierung reparieren.
2. Andernfalls Info-ZIP Unicode Path 0x7075 lesen und Version 1 mit passender CRC32 des Originalnamens und vollständigem gültigem UTF-8 akzeptieren. Unbekannte Versionen und CRC-Abweichungen spezifikationsgemäß ignorieren. Passendes, aber ungültiges UTF-8, doppelte gültige Felder oder beschädigte Feldstruktur zur Prüfung markieren.
3. Ohne nutzbare Erweiterung unmarkiertes UTF-8 akzeptieren, wenn alle nicht deklarierten Namen der Gruppe gültiges UTF-8 sind.
4. Andernfalls archivweite statistische Erkennung für nicht deklarierte Nicht-ASCII-Namen anwenden, einschließlich GBK/GB18030, Big5, Shift-JIS, EUC-JP, Koreanisch und weiterer unterstützter Zeichensätze. ZIPs westliche Standardvermutung fällt auf den historischen Standard CP437 zurück. Bei erkanntem GBK den kompatiblen strikten GB18030-Decoder verwenden.

Gewöhnliche TAR-Header, lange GNU-Namen und ausdrücklich mit hdrcharset=BINARY markierte PAX-Pfade verwenden dieselbe Strategie für nicht deklarierte Namen. Gewöhnliche PAX-Pfade verlangen gültiges UTF-8; Erkennung darf beschädigte Deklarationen nicht verbergen. Andere ausdrückliche hdrcharset-Werte als UTF-8 und BINARY werden nicht unterstützt. TAR übernimmt nicht ZIPs CP437-Standard. TAR.GZ verwendet dieselbe Namensauflösung wie TAR.

Die Regeln folgen [ZIP APPNOTE 6.3.10](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT), [POSIX pax](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/pax.html) und dem [WHATWG Encoding Standard](https://encoding.spec.whatwg.org/). Ein Unicode-Path-Feld darf im lokalen Header fehlen; ein gelieferter gültiger Wert muss aber dem eingefrorenen wirksamen Namen entsprechen. Die Nutzdatenprüfung kontrolliert weiterhin Rohname, lokale und zentrale Header, CRC, Größe, Passwortauthentifizierung und Stromende.

CAB bewahrt ursprüngliche Namensbytes und das UTF-8-Attribut. Ausdrückliches UTF-8 wird strikt geprüft; andere Namen verwenden begrenzte archivweite Erkennung ohne Systemcodepages. CAB-Backslashes sind Formatverzeichnistrenner und werden erst nach vollständiger Dekodierung in Schrägstriche umgewandelt, damit das Shift-JIS-Mehrbyte-Folgebyte 0x5C nicht getrennt wird. Rohbytes bleiben unverändert und wirksame Pfade durchlaufen dieselben Sicherheits- und Kollisionsprüfungen. ISO-Namen außerhalb von Joliet bewahren Rohbytes und nutzen dieselbe begrenzte Auflösung, ohne Rock-Ridge-Namen durch Host-OEM zu konvertieren. Joliet/UDF nutzen vom Prozessor gelieferte Unicode-Namen und prüfen sie strikt; verlustbehaftete Ersatzzeichen gelten nicht als Originalnamen.

<a name="xdzip-detection-boundaries-and-resources"></a>

## Erkennungsgrenzen und Ressourcen

Der Detektor liest höchstens 1 MiB vollständiger nicht deklarierter Nicht-UTF-8-Namen und trennt Datensätze mit Zeilenumbrüchen, damit abgeschnittene Mehrbytesequenzen nicht über Namen hinweg zusammenfinden. Ausdrückliche Unicode-Deklarationen und ungültige Erweiterungen sind keine Stichproben. Enthält eine Stichprobe Nicht-UTF-8-Namen, das Archivergebnis einheitlich auf nicht deklarierte Namen anwenden, damit zufällig gültige UTF-8-Fragmente in GBK nicht ein Verzeichnis in zwei Namen teilen. Nur wenn die strikte Dekodierung eines einzelnen Namens scheitert, dürfen gültiges UTF-8 oder ein lokales Ergebnis für diesen Namen versucht werden. ISO-2022-JP-Erkennung ist deaktiviert; Escape-Zustände in Namen werden nicht interpretiert. Strikte Dekodierung fügt keine Ersatzzeichen ein, entfernt keinen BOM, konvertiert keine Schrägstriche, normalisiert Unicode nicht und repariert keine Pfade.

Rohnamen umfassen insgesamt höchstens 16 MiB. Ein einzelner roher oder dekodierter Name umfasst höchstens 4.096 Bytes; dekodierte Namen und maskierte Anzeige zusammen höchstens 16 MiB. Einzel- und Gesamtallokationen der ZIP-Unicode-Erweiterung werden während des Scans statt nach Archivende geprüft. Auflösung, Erkennung und Einzelkonvertierung prüfen Abbruch. ZIP-Nutzdaten behalten zusätzlich höchstens 16 MiB für eine Prüftabelle wirksamer Namen; spätere Dekodierung kann eingefrorene Namen nicht ändern.

Statistische Erkennung kann beliebige kurze Namen oder gemischte alte Kodierungen nicht eindeutig rekonstruieren und erkennt kein Mojibake, das andere Software bereits als gültiges Unicode gespeichert hat. Ausdrücklich deklariertes Unicode und alte Kodierung können in einem Archiv koexistieren; mehrere nicht deklarierte Kodierungen werden nach bestem Bemühen erkannt. Einen manuellen Kodierungswähler gibt es derzeit nicht. Namen, die nicht strikt dekodiert werden können, beschädigte Deklarationen oder ungewöhnliche Pfade haben, bleiben maskiert sichtbar und zur Prüfung markiert, ohne Extraktion, temporäres Öffnen, Export oder Änderung zu erlauben. Korrekte Erkennung jedes historischen Archivs wird nicht versprochen. Bei ungeklärten Namen kann ihre tatsächliche Verzeichniszugehörigkeit unbekannt sein; deshalb wird Extraktion durch Verzeichniserweiterung abgelehnt, um nicht stillschweigend Einträge aus einem erfolgreichen Ergebnis auszulassen. Ausdrücklich ausgewählte aufgelöste reguläre Dateien können weiterhin einzeln extrahiert werden.

<a name="xdzip-end-to-end-use-and-saving"></a>

## Durchgängige Nutzung und Speichern

Verzeichnisbaum, Direktkindindex, rekursive Suche, Erweiterung der Verzeichnisauswahl, Vorprüfung, Extraktion, temporäres Öffnen, externer Export und Archivänderung verwenden denselben wirksamen Namen. Nach Dekodierung weiterhin PortablePath, Unicode-NFD-/Großkleinschreibungsalias, doppelte Namen und Ausgabeberechtigung prüfen. Das Shift-JIS-Mehrbyte-Folgebyte 0x5C ist kein Verzeichnistrenner. Abgesehen von CAB- und LHA-Formattrennern bleiben tatsächliche dekodierte Backslashes abgelehnt.

Nach Planung deklarieren Hinzufügen und Verschieben in gewöhnlichem ZIP, TAR und TAR.GZ Namen alter Kodierung, Unicode-Erweiterung und unmarkierten UTF-8 als UTF-8 neu und bewahren ursprüngliche Positionen und Überspringsemantik im logischen Verschiebeplan. ZIP bewahrt komprimierte Nutzdaten; TAR verwendet ausdrückliche UTF-8-PAX-Pfade. Nach dem Schreiben erneut scannen, jeden Namen und Inhalt prüfen und anschließend den bestehenden Ersatz auf demselben Volume ausführen. Entfernt das TAR-Umschreiben eine BINARY-Deklaration für einen PAX-Pfad und würde dadurch die Interpretation anderer Nicht-UTF-8-PAX-Texte ändern, ausdrücklich fehlschlagen und das Originalarchiv bewahren. Verschlüsselte und geteilte Archive nutzen beim Lesen und Extrahieren dieselben Namen; Beschränkungen für Hinzufügen und Verschieben bleiben gleich.

<a name="xdzip-reproducible-acceptance"></a>

## Reproduzierbare Abnahme

Projekteigene Rohbyte-Testdaten und unabhängig mit der Python-Standardbibliothek kodierte Archive verwenden; sie decken unmarkiertes UTF-8, GBK, vier Byte langes GB18030, Big5, Shift-JIS mit 0x5C, Koreanisch, Russisch, ZIP CP437 und Unicode Path ab. Vollständige Namen, Originalbytes, Verzeichnisse, Suche, Auswahl, Vorprüfung, tatsächliche Ausgabebytes, temporäres Öffnen, Export, Hinzufügen und Verschieben gewöhnlicher Archive, AES und geteilte Volumes prüfen. Zusätzlich beschädigte Deklarationen, Erweiterungspriorität, Pfadablehnung, Unicode-/Großkleinschreibungskollisionen, Abbruch und Budgets prüfen.

scripts/check-filename-encoding.py extrahiert Python-erzeugtes ZIP/TAR/TAR.GZ über eine tatsächliche Dienstsonde. Python liest auch Rust-umgeschriebene Archive unabhängig und prüft UTF-8-Flags/PAX, Namen und Inhalte. Testdaten stammen nur aus Quellkonstanten, nicht aus Benutzerarchiven. Tatsächliche Ergebnisse und verbleibende manuelle Abnahme werden in Issues und build/CI-Artefakten dokumentiert, nicht als bestanden in diesem Vertrag.

LHA bewahrt ursprüngliche dirname/filename-Feldgrenzen und rohe `0xff`-dirname-Trenner. Die gemeinsame begrenzte Erkennung beprobt jedes vollständige Feld getrennt. Jede Komponente strikt dekodieren, bevor dekodierte Backslashes interpretiert und `/` zusammengesetzt werden; ursprüngliche Bytegrenzen verhindern feldübergreifendes Übernehmen von Mehrbytefolgen. Verzeichniserweiterung und Ausgabe verwenden denselben wirksamen Namen. Siehe den [LHA-Vertrag](../../engineering/lha-containers.md).
