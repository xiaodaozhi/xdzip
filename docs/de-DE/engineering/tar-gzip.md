<a name="xdzip-targzip-implementation-contract"></a>

# Implementierungsvertrag für TAR/GZIP

[English](../../engineering/tar-gzip.md) | [简体中文](../../zh-CN/engineering/tar-gzip.md) | [繁體中文](../../zh-TW/engineering/tar-gzip.md) | [日本語](../../ja-JP/engineering/tar-gzip.md) | [한국어](../../ko-KR/engineering/tar-gzip.md) | [Français](../../fr-FR/engineering/tar-gzip.md) | Deutsch | [Español](../../es-ES/engineering/tar-gzip.md) | [Português (Brasil)](../../pt-BR/engineering/tar-gzip.md) | [Русский](../../ru-RU/engineering/tar-gzip.md)

Dieser Vertrag definiert Grenzen für Browsen, Erstellen und sicheres Extrahieren von TAR/TAR.GZ. Plattformresultate und manuelle Abnahme werden in Issues oder CI-Artefakten festgehalten. Eingabeautorisierung, Staging, Konflikte, Abbruch und Gesamtarchiv-Commit aus [sicherer Extraktion](zip-safe-extraction.md) und [ZIP-Erstellung](zip-creation.md) bleiben erhalten.

<a name="xdzip-scope-and-capabilities"></a>

## Umfang und Fähigkeiten

Browsen, Erstellen und sicheres Extrahieren von TAR und TAR.GZ werden unterstützt. TAR ist ein Container und GZIP ein Kompressionsfilter; beim Lesen wird Inhalt erkannt statt ein Backend anhand der Endung gewählt. ZIP-Verhalten bleibt kompatibel. Automatisches verschachteltes Entfalten, Verschlüsselung und geteiltes TAR sind nicht aktiviert. Gewöhnliches GZIP für eine Datei verwendet den separaten [Stromvertrag](single-file-streams.md).

TAR-Header, Zahlen, PAX, Streaming-Kodierung und Umschreiben verwenden eine unabhängige Rust-Datensatzschicht. GZIP-Rahmung, CRC-32 und Deflate LZ77/Huffman verwenden das gemeinsame `xdzip-compression`. `tar` und `flate2` dienen nur als Entwicklungstest-Referenzen und gelangen nicht in Produktionsabhängigkeitsschließungen dieser Crates. Spezifikationen, Quellen und Grenzen stehen im [Vertrag eigener Formate](independent-rust-codecs.md).

TAR akzeptiert reguläre Dateien und Verzeichnisse und analysiert PAX-Pfade und lange GNU-Namen. Symbolische und harte Links, Geräte, FIFOs, Sparse-Dateien und unbekannte Typen können nicht extrahiert werden. Alle Ausgabenamen durchlaufen weiterhin PortablePath und die Host-Konfliktvorprüfung; Formatbibliotheken erhalten weder Dateipfade noch Entpackberechtigung. Verzeichnisrechte, Eigentümer und Spezialmodi werden nicht wiederhergestellt. Datumsangaben sind Anzeigemetadaten ohne Wiederherstellungsgarantie.

<a name="xdzip-bounded-streaming"></a>

## Begrenztes Streaming

Alte Dateinamencodierungen, lange GNU-Namen und PAX-UTF-8/BINARY-Deklarationen werden einheitlich nach dem [Kodierungsvertrag](filename-encoding.md) aufgelöst. Indizes werden erst nach vollständigem Scan veröffentlicht; Eintragszahlen und Gesamtsummen sind währenddessen unbekannt. Für Metadaten gelten weiterhin 100.000 Einträge, 4.096 Bytes je Name und kumulative Namensbudgets. Kodierte Eingabe der Streamingquelle und Dekodierarbeit sind jeweils auf 24 GiB begrenzt; ein GNU/PAX-Erweiterungsheader auf 64 KiB, höchstens vier Erweiterungsheader je Eintrag. Globales PAX akzeptiert nur Kommentartext ohne Dateisemantik für git-archive-Kompatibilität. Er wird vollständig gegen die deklarierte Datensatzlänge geprüft und verworfen statt an Einträge vererbt. Globale Felder path, size, mtime, linkpath, sparse und unbekannte Felder bleiben ausdrücklich abgelehnt; lange GNU-Links und Sparse-Varianten werden nicht unterstützt. Globale Kommentare zählen ebenfalls zum Limit von vier Headern je Eintrag und zum Strombytebudget und machen unvollendete lokale Erweiterungen nicht gültig. Häufiges führendes `./` in TAR-Namen wird normalisiert; Wurzelmarker sind keine Ausgabeeinträge. Sequenzielles Lesen und Überspringen prüfen ebenfalls Abbruch und Bytebudgets; Positionieren umgeht keine Prüfungen beschädigter Ströme. GZIP muss EOF erreichen und CRC und ISIZE prüfen. Mehrere Mitglieder bilden nur einen zusammenhängenden TAR-Bytestrom. Nichtnull-Restdaten, Abschneiden und Beschädigung führen zum Fehler. Ungeprüfte Teilindizes werden abgelehnt.

TAR.GZ hat keine komprimierten Größen je Eintrag. Die Hochverhältnis-Zulassung vergleicht die ausgewählte deklarierte Gesamtsumme mit der vollständigen festgehaltenen Quellgröße. Über 1000:1 und 1 MiB ist weiterhin ausdrückliche Erlaubnis nötig; sie umgeht weder Scanarbeits- noch Ausgabequoten. Metadatenvorprüfung erfindet keine Kompressionsverhältnisse einzelner Einträge.

Die Extraktion verarbeitet ausgewählte Einträge in physischer Reihenfolge. Das Überspringen nicht ausgewählter Inhalte wird gemessen und ist abbrechbar; Direktzugriff wird nicht zugesagt. PayloadReader erhält nur Quelle und Host-Ausgabeziel. Der Host prüft Quellversionen weiterhin an Scan-, Dekodier- und Commit-Grenzen. Commit je Eintrag und Fehlersemantik der Gesamterstellung folgen dem bestehenden sicheren E/A-Vertrag.

<a name="xdzip-creation-and-interface"></a>

## Erstellung und Oberfläche

Das Erstellungsfenster aktiviert vorhandene TAR- und TAR.GZ-Segmente ohne Dichteänderung. Formatwechsel aktualisiert Endungen. TAR deaktiviert die Kompressionsstufe; beide TAR-Formate deaktivieren und löschen Passwort und Split-Volume-Werte. TAR.GZ verwendet GZIP-Stufen 0/1/6/9. Die komprimierte Größe je Eintrag ist unbekannt und wird als Strich statt 0 Bytes angezeigt. Rust leitet den nach dem Archiv benannten Zielordner einheitlich vom Quellnamen ab und stellt ihn über die begrenzte Brücke zur Vorschau bereit. TAR.GZ entfernt die vollständige zusammengesetzte Endung ohne Beachtung der Großschreibung; ein leerer Basisname fällt auf archive zurück. Die Oberfläche leitet kein anderes Ziel ab; PortablePath und die autorisierte Wurzel prüfen es erneut. Formatmarkierungen des Hauptfensters passen sich tatsächlichem Format und Textbreite an. TAR-Daten werden in UTC angezeigt und bleiben außerhalb des bestehenden Anzeigebereichs unbekannt. Die Erstellung schreibt für Unicode oder lange Namen ausdrücklich PAX-UTF-8-Pfade und für große Dateien PAX-Größe, um lokale Kodierungsvermutungen von Referenzlesern zu vermeiden. Eingabebaum einfrieren, Ausgabe streamen, TAR/GZIP beenden, erneut scannen und jeden Eintrag dekodieren und prüfen; erst nach Erfolg insgesamt veröffentlichen. Umbenennung bei Konflikten erhält die vollständige zusammengesetzte Endung.

<a name="xdzip-verification-and-limits"></a>

## Prüfung und Grenzen

Bidirektionale Interoperabilität mit System-tar/Python tarfile, lange Pfade, Unicode, leere Verzeichnisse, Duplikate, PAX/GNU, Spezialeinträge, Pfadausbruch, Abschneiden, CRC, Ressourcengrenzen, Abbruch und Schutz alter Dateien abdecken. Tatsächliche Qt-Erstellung, erneutes Öffnen und Extraktion verwenden vorhandene Aufgaben- und Konfliktfenster. Tatsächliche Nachweise entstehen nur unter build/ oder in CI-Artefakten.

Betroffene Desktop- und Barrierefreiheitsnachprüfungen nach der [allgemeinen Verifikation](../../../developers/qa/README.md) auswählen; native Automatisierung und Offscreen-Prüfungen ersetzen keine manuellen Prüfungen. Tatsächliche Ergebnisse und Grenzen gehören in das betreffende Issue. Bestehende Desktop-Abnahme steht in [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5); einmalige Berechtigungs- oder Umgebungsblockaden bleiben nicht in diesem Vertrag. Historische Release-Lizenzprüfung zu `v0.1.0` steht in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) und [#53](https://github.com/xiaodaozhi/xdzip/issues/53); zukünftige Kandidaten benötigen neue Release-Issues.

Der CLI-Erstellungseinstieg akzeptiert .tgz als Alias desselben Formats wie .tar.gz mit gleicher GZIP-Kodierung, Prüfung und Commit. Gewöhnliches GZIP ohne TAR nutzt den separaten Einzeldatei-Codec.
