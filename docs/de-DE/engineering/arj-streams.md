<a name="xdzip-arj-compressed-streams-and-container-admission"></a>

# Komprimierte ARJ-Streams und Containerzulassung

[English](../../engineering/arj-streams.md) | [简体中文](../../zh-CN/engineering/arj-streams.md) | [繁體中文（台灣）](../../zh-TW/engineering/arj-streams.md) | [日本語](../../ja-JP/engineering/arj-streams.md) | [한국어](../../ko-KR/engineering/arj-streams.md) | [Français](../../fr-FR/engineering/arj-streams.md) | Deutsch | [Español](../../es-ES/engineering/arj-streams.md) | [Português (Brasil)](../../pt-BR/engineering/arj-streams.md) | [Русский](../../ru-RU/engineering/arj-streams.md)

`ArjReader` bietet begrenzte rohe Decodierung der ARJ-Methoden 1–4. Store gehört zum Container, nicht zu diesem Leser. `xdzip-codec-arj` stellt vollständige physische Struktur und CRC-geprüfte Mitgliedsbereitstellung über eine vom Host gehaltene Quelle bereit. `ArjCodec` verbindet zugelassene Profile mit Laufzeit, CLI und Desktop-Navigation, Prüfung und Entpacken. Das [Formatregister](../../../specs/engineering/archive-formats.json) erklärt den schreibgeschützten Umfang; Erstellen, Umschreiben, Passwörter und Volumes sind nicht verfügbar.

<a name="xdzip-host-admission-and-names"></a>

## Hostzulassung und Namen

Ein Vorschlag mit zwölf Byte Hauptheader veröffentlicht niemals ein Format oder Zeilen. Die Laufzeit bewahrt zuerst den Vorrang geprüfter starker Container und von ZIP/SFX und scannt dann vorgeschlagene ARJ-Metadaten vollständig unter dem gemeinsamen Eingabekonto schwacher Prüfungen. Ein roher ungeprüfter TAR-Marker wird durch vollständige ARJ-Zulassung eindeutig zugeordnet. Ungültige Vermutungen veröffentlichen nichts; typisierte Hostfehler, Abbruch und Quoten bleiben maßgeblich.

Der Adapter erhält physische Mitgliedsordnungen, ursprüngliche Methodenbytes und rohe Namen. ANSI-markierte und OEM-Namen haben getrennte Kodierungsgruppen mit gemeinsamem Limit von 1 MiB für vollständige Feldstichproben, ohne Auswahl anhand der Umgebungslocale. Einen Mehrbytenamen vollständig decodieren, bevor Trenner interpretiert werden. Nur DOS-artige Hosts ohne Portable-Path-Flag wandeln Backslashes in Schrägstriche um. Wörtliche Backslashes in portablen oder Unix-Namen bleiben verdächtig. Ausdrückliche Verzeichnisse erhalten einen abschließenden Schrägstrich; verdächtige und ungelöste Namen, unbekannte Methoden und nicht unterstützte Objektprofile bleiben ohne Nutzlastbefugnis sichtbar. Decodierte Namen sind auf 4.096 Bytes und gehaltene Anzeige-/Decodierallokationen auf 16 MiB begrenzt, oder auf kleinere Aufrufergrenzen. Attribute werden nicht wiederhergestellt.

Das Öffnen der Nutzlast projiziert und vergleicht vor Bereitstellung die vollständige eingefrorene Eintragsmenge erneut. Jeder gewählte Datensatz muss genau seiner physischen Zeile entsprechen; nicht unterstützte Auswahl und jeder Decodierfehler vergiften die Nutzlastinstanz. Hostidentitätsprüfungen, Vorprüfung, private Vorbereitung, Abbruch und endgültige Veröffentlichung behalten gemeinsame Laufzeitsemantik. `supported_content`-Prüfungen decodieren alle zugelassenen gewöhnlichen Mitglieder vollständig und prüfen exakte Größen und erforderlichen CRC-32. Nicht unterstützte Zeilen verhindern archivweiten Erfolg. Historischer Text wird als vom CRC abgedeckte Archivbytes ohne hostabhängige Zeilenumwandlung geliefert.

<a name="xdzip-sources-and-reference-boundary"></a>

## Quellen und Referenzgrenzen

Das [Open-Source-ARJ-Projekt](https://arj.sourceforge.net/) stellt [Quelle 3.10.22](https://arj.sourceforge.net/files/arj-3.10.22.tar.gz), SHA-256 `589e4c9bccc8669e7b6d8d6fcd64e01f6a2c21fe10aad56a83304ecc3b96a7db`, und sein [Win32-3.10-Paket](https://arj.sourceforge.net/files/arjw_310.exe), SHA-256 `e5cc5d8f82c5f08246e1c0a8c5be77e5f796c2dd30f987e6bbb27a2c1df1e371`, bereit. Die Quelle enthält den Text der GNU GPL Version 2. Begrenzte Beobachtungen des Drahtformats nutzen `defines.h`, `decode.c` und `arj_arcv.c`; die native Referenz bleibt unverändert unter ihren ursprünglichen Bedingungen ausschließlich in build/toolchain.

Der unabhängige Vergleich nutzt offizielles [7-Zip](https://www.7-zip.org/) mit begrenzten Quellbeobachtungen seines [ARJ-Handlers](https://github.com/ip7z/7zip/blob/main/CPP/7zip/Archive/ArjHandler.cpp), Git-Blob `da890f279d35c00bc61d332ce55f4d96ecd6d373`. Ein strenger Reinraum wird nicht behauptet. Die Rust-Implementierung verwendet projekteigene Bit-, Präfix- und LZ-Primitiven und drückt die schnellen unären Felder direkt aus. Referenzquellen, Binärdateien und Kompressionsengines gelangen nicht in die Produktabhängigkeiten. Genaue Komponenten- und eigene Eingabeidentitäten stehen bei den [Testdaten](../../../tests/fixtures/arj/README.md).

<a name="xdzip-static-methods-13"></a>

## Statische Methoden 1–3

Methoden 1–3 teilen das statische Huffman/LZ-Drahtmodell, während native Encoder verschiedene Suchstrategien wählen können. Ihre 16-Bit-Blockbefehlsanzahl, das Längenalphabet mit 19 Symbolen, Literal-/Längenalphabet mit 510 Symbolen, Positionsalphabet mit 17 Symbolen und der 5-Bit-Positionszähler verwenden die gemeinsame begrenzte LZH-Präfiximplementierung über ein ausdrückliches ARJ-Profil. Sie erhalten weder alte LH3-Baumfelder noch eine öffentliche LHA-Methodenkennung. Blockbefehlszahlen null sind nicht zugelassen.

Literale sind Symbole 0–255; andere Symbole ergeben Längen `symbol - 253` von 3–256. Positionssymbol null ergibt Offset null; sonst ist der Offset `2^(symbol - 1)` plus entsprechend viele niedrige Bits. Die tatsächliche Distanz ist Offset plus eins, höchstens 26624, und muss bereits ausgegebene Bytes referenzieren. Ein Ring von 32 KiB implementiert die begrenzte logische Historie; die größere Allokation erweitert die Zulassung nicht. Initiale LHA-Leerzeichen bleiben getrennt und können ARJ-Ausgabe nicht initialisieren. Wörterbuchumlauf und überlappende Kopien behalten sequenzielle Bytesemantik. Vollständige kanonische Bäume, Singleton-Felder, Nullläufe und 16-Bit-Codetiefe behalten die gemeinsame Präfixvalidierung und Arbeitsabrechnung.

<a name="xdzip-fast-method-4"></a>

## Schnelle Methode 4

Der Stream ist höchstwertiges Bit zuerst. Ein Längenfeld zählt 0–7 führende Einsbits mit Nullstopp außer beim Maximum. Danach genauso viele niedrige Bits lesen und `2^count - 1` addieren. Null bezeichnet ein Acht-Bit-Literal, ungleich null eine Kopie der Wertzahl plus zwei von 3–256 Bytes. Der Zeiger zählt ähnlich 0–4 führende Einsbits, mit niedriger Breite `9 + count` und Basis `512 * (2^count - 1)`. Offset plus eins ergibt höchstens 15872 Distanz. Jede Referenz muss auf ausgegebene Historie zeigen; ein Ring von 16 KiB unterstützt Überlappung und Umlauf. Eine Reparatur initialer Historie ist nicht erlaubt.

<a name="xdzip-limits-and-failures"></a>

## Grenzen und Fehler

Statische Profile lassen 32 KiB Historie plus 64 KiB für Eingabe/Tabellen/Arbeitsbereich zu. Schnelle Decodierung lässt 16 KiB Historie plus 32 KiB für Eingabe/Zustand/Arbeitsbereich zu. Komprimierte und decodierte Größe haben jeweils eine harte Grenze von 24 GiB, Arbeit von 96 GiB; kleinere Aufruferbudgets bleiben maßgeblich. Ausgabelesungen umfassen höchstens 8192 Bytes. Präfixoperationen, Historienarbeit und quellenunabhängige Kopien behalten kooperative Abfragepunkte. `work_remaining()` unterstützt das kumulierte tatsächliche Arbeitskonto des Containers.

Angegebene Längen dürfen nicht überschritten werden. Logische komprimierte Bytes müssen exakt enden; letzte Füllbits unterhalb eines Bytes werden ignoriert, zusätzliche vollständige Bytes oder fehlende Eingabe scheitern, und es werden keine Null-EOF-Bytes erfunden. Nur bloßes `Interrupted` wird erneut versucht. Typisierte Fehler behalten ihre erste Nutzlast und beenden; jeder Quell-, Daten-, Quoten- oder Abbruchfehler vergiftet den Leser. Rohe Leser haben keine Befugnis über Pfad, Zugangsdaten, Ausführung, Dateisystemveröffentlichung oder CRC.

<a name="xdzip-bounded-physical-containers"></a>

## Begrenzte physische Container

`inspect` prüft ein Archiv ab Offset null, eine erste Hauptheaderbreite von 30/34, Dateiheaderbreiten 30/34/46 und vollständige Basisblöcke bis 2600 Bytes. Namen und Kommentare enden exakt, das Dateinamenpositionsfeld erlaubt keine Abschneidung und alle Basis-/erweiterten CRC-32 müssen bestehen. Jeder Datensatz lässt höchstens 256 Erweiterungsblöcke mit 16-Bit-Längen zu. Undurchsichtige Dateiattribute werden geprüft und in die Metadatenidentität aufgenommen, ohne Anwendung oder Ausführung. Der exakte Terminator `60 ea 00 00` muss die physische Quelle beenden; SFX-Präfix, Füllung oder Wiederherstellungsrest werden nicht angenommen.

Metadatenlesungen sind auf 64 MiB, physische Einträge auf 100000, rohe Namen auf jeweils 4096 Bytes und insgesamt 16 MiB begrenzt; kleinere Aufrufergrenzen bleiben erhalten. Ein späterer Fehler veröffentlicht keinen Teilindex. SHA-256 deckt vollständige physische Metadaten und Terminator einschließlich ignorierter Kommentare/Modifikatoren/Attribute ab; `Reader::open` scannt und vergleicht dies sowie jeden Datensatz und die physische Länge vor Ausgabe erneut. Quellfehler bleiben terminal und werden nicht zu Vermutungen fehlerhafter Daten. Unix-Hostzeitstempel verwenden nur den optionalen Hostkalender; fehlende Projektionen werden null, Rückruffehler beenden.

Das Mitgliedsprofil lässt erforderliche Versionen bis 11, Hosts DOS/Unix/OS2/Win95/WinNT, unverschlüsselte Binär-/Alttextmethoden 0–4 und leere Store-Verzeichnisse zu. Nicht unterstützte Methoden, künftige Versionen, Spezialtypen, Unix-Spezialmodi und Verschlüsselungs-/Fortsetzungs-/Kapitelfelder bleiben ausdrücklich und können nicht decodieren. Hauptprofile für Volume/Schutz/Sicherheit/Doppelnamen/Verschlüsselung/Kapitel und Dateilayouts mit Doppelnamen sind nicht zugelassen. Zugriffsmodi und Attribute berechtigen nie zu Rechten oder Spezialdateierstellung. Historischer Text wird als von Archivgröße und CRC abgedeckte Bytes wie bei 7-Zip geliefert; natives Win32 ARJ erweitert für das Textprofil dagegen LF zu CRLF. Eine hostabhängige Zeilenumwandlung erfolgt nicht.

Ein Leser behält kumuliert 24 GiB tatsächliche Eingabe einschließlich des neuen Metadatenscans, 24 GiB decodierte Ausgabe und 96 GiB Arbeit. CRC-Bitschritt-Arbeit für die angegebene Ausgabe wird reserviert, bevor verbleibende Arbeit an den zustandsbehafteten Decoder verliehen wird. Erfolgreiche Mitglieder rechnen genau diese Reserve plus tatsächliche Konstruktions-/Modell-/Store-Arbeit ab; Fehler beendet ohne Erstattung. Senkenabrechnung des Aufrufers enthält Konstruktion und beginnt nicht erst nach Initialisierung. Alle Lesungen bleiben auf 8192 Bytes begrenzt, Snapshots und physische Indizes werden geprüft und jeder Nutzlast-/CRC-/Senken-/Abbruch-/Quotenfehler vergiftet die Instanz.

<a name="xdzip-required-container-and-host-admission"></a>

## Erforderliche Container- und Hostzulassung

ARJ-Haupt- und Dateiheader, ihre begrenzten ersten Headerlayouts, CRC-32, Erweiterungsheaderketten, exakte komprimierte Bereiche und Endmarkierung der Länge null müssen vor Veröffentlichung eines vollständigen Index gültig sein. Physische IDs erhalten doppelte Datensätze. Namen, Codepages, Trenner, Verzeichnisse, historische Text-/Binärtypen, Spezialobjekte und jede unbekannte Methode benötigen ausdrückliche Projektion und Nutzlastzulassung; ein erkannter Marker autorisiert keinen Inhalt.

Jeder angegebene Datei-CRC muss nach vollständiger Decodierung bestehen. Header und optionale Archivsicherheitsfelder bedeuten keine kryptografische Authentizität. Passwörter, verschlüsselte Methoden, Volumes, fortgesetzte Einträge, Kapitel, Wiederherstellungs-/Sicherheitsanhänge und selbstextrahierende Hüllen benötigen eigene Profile und Nachweise. Kein Skript, automatischer Befehl, externes Attribut, Nachbarpfad oder Ausführungsflag erteilt Befugnisse. Einfrieren der Quelle, selektives Entpacken, kumulierte Eingabe-/Arbeits-/Ausgabebudgets, private Vorbereitung, Abbruch, Schutz alter Dateien und endgültige Veröffentlichung folgen dem [Entpackvertrag](zip-safe-extraction.md).

Der Testdatengenerator erstellt nur eigene Binäreingaben, erfasst angeforderte und tatsächliche Methoden und liest die vollständige Ausgabe unabhängig mit nativem ARJ und 7-Zip. Ein natives Store-Fallback bleibt Store. Nur unverschlüsselte Zeitstempel, ungenutzter zeitabhängiger Modifikator und zugehörige Header-CRCs werden normalisiert; komprimierte Bytes und Inhalts-CRCs bleiben unverändert. Leere native Eingaben erzeugen Store-Einträge und belegen keinen codierten leeren komprimierten Stream. Produktzulassung erfordert außerdem tatsächliche Runtime-/CLI-/Qt- und anwendbare Kandidaten-/Layoutautomatisierung gemäß [QA-Umfang](../../../developers/qa/README.md).
