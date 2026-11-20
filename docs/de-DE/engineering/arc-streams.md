<a name="xdzip-arc-containers-and-compressed-stream-profiles"></a>

# ARC-Container und Profile komprimierter Streams

[English](../../engineering/arc-streams.md) | [简体中文](../../zh-CN/engineering/arc-streams.md) | [繁體中文（台灣）](../../zh-TW/engineering/arc-streams.md) | [日本語](../../ja-JP/engineering/arc-streams.md) | [한국어](../../ko-KR/engineering/arc-streams.md) | [Français](../../fr-FR/engineering/arc-streams.md) | Deutsch | [Español](../../es-ES/engineering/arc-streams.md) | [Português (Brasil)](../../pt-BR/engineering/arc-streams.md) | [Русский](../../ru-RU/engineering/arc-streams.md)

`ArcReader` implementiert begrenzte rohe Decodierung der ARC-Methoden 3–9. Methoden 1 und 2 sind Store-Profile des Containers. `xdzip-codec-arc` und `ArcCodec` verbinden vollständige Struktur und erforderlichen CRC-16 mit Laufzeit, CLI und Desktop-Navigation, Prüfung und Entpacken. Das [Formatregister](../../../specs/engineering/archive-formats.json) erklärt den schreibgeschützten Umfang; Erstellen, Ändern, Passwörter und Volumes sind nicht verfügbar.

<a name="xdzip-container-and-host-admission"></a>

## Container- und Hostzulassung

Das zugelassene Profil verwendet gewöhnliche ARC-5.x-Struktur; zusätzliche ARC-6.x-Metadaten und Spark-Erweiterungen werden nicht beansprucht. Jedes Mitglied beginnt mit `0x1a` und seinem ursprünglichen Methodenbyte. Methode 1 verwendet 25 Bytes, alle anderen Methoden ungleich null den gewöhnlichen 29-Byte-Header. Das dreizehn Byte lange Dateinamenfeld muss einen nicht leeren nullterminierten Namen enthalten. Alle Ganzzahlen sind Little-Endian: komprimierte Länge bei 15, DOS-Datum/-Zeit bei 19/21, erforderlicher CRC-16 bei 23 und gewöhnliche decodierte Länge bei 25. Altes Store übernimmt die komprimierte Länge. Beide Store-Methoden verlangen gleiche Längen. Komprimierte Bereiche müssen passen, die nächste Markierung ist verpflichtend und der exakte Zwei-Byte-Terminator `1a00` muss mit physischem EOF zusammenfallen. Resynchronisation, Präfixsuche, Füllung und benachbarte Dateien werden nicht zugelassen.

Der Container erhält alle physischen Duplikate und unbekannten Methoden. Metadatenlesungen haben eine harte Grenze von 64 MiB, höchstens 100000 Datensätze, durch den Aufrufer begrenzte Rohdatenamen und eine physische Eingabegrenze von 24 GiB. SHA-256 bindet jedes Headerbyte und den Terminator einschließlich ignorierter Namensfüllung beim erneuten Öffnen eines exakten `Index`. Header-Hashes authentifizieren keine Nutzlast. Jeder Leser besitzt kumulative Konten von je 24 GiB tatsächlicher Eingabe und decodierten Bytes sowie 96 GiB Arbeit, reserviert vor Decodierung die Bytearbeit für Tabellen-CRC und endet bei jedem Fehler ohne Budgeterstattung oder Veröffentlichung eines Teilindex.

Der Host bewahrt den Vorrang starker Container und ZIP/SFX und prüft schwache ARC-Vorschläge vollständig unter dem gemeinsamen Prüfkonto. Namen werden vor DOS-Backslash-Konvertierung als vollständige Felder nach der gemeinsamen begrenzten Archivkodierungsentscheidung decodiert. Ursprungsbytes und Methoden-IDs bleiben verfügbar. Verdächtige/ungelöste Namen und unbekannte Methoden bleiben ohne Nutzlastbefugnis sichtbar. ARC besitzt kein zugelassenes Profil für Verzeichnisse, Links, Moduswiederherstellung oder externe Objekte. Historisches XOR hat kein Verschlüsselungsflag; Stream-, Größen- und CRC-Prüfung bleiben verpflichtend, ohne Passwortraten oder Fallback. Für dieses Profil wird keine eigene Linux-MIME-Registrierung beansprucht.

Das Öffnen der Nutzlast projiziert und vergleicht eingefrorene Einträge erneut; ausgewählte IDs bezeichnen physische Zeilen und müssen exakt entsprechen. Laufzeitvorprüfung, eingefrorene Hostquellidentität, private Vorbereitung, Abbruch, Schutz alter Dateien und endgültige Veröffentlichung bleiben maßgeblich. Die archivweite `supported_content`-Prüfung decodiert jedes zugelassene gewöhnliche Mitglied tatsächlich, prüft exakte Größe und CRC-16 und scheitert bei nicht unterstützten Zeilen. Eine unterstützte physische Auswahl kann dennoch neben einem nicht unterstützten Nachbarn entpackt werden.

<a name="xdzip-sources-and-references"></a>

## Quellen und Referenzen

Die ursprüngliche ARC-Unix-Portierung von Howard Chu und Anibal Monsalve Salazar ist auf [Commit `70f0b93516d0ee222db5cf1241aad0db1b721c0d`](https://github.com/ani6al/arc/tree/70f0b93516d0ee222db5cf1241aad0db1b721c0d) festgelegt. Ihr `Arcinfo` dokumentiert Robert Freeds ARC-5.12-Strukturbeschreibung. Begrenzte Drahtbeobachtungen nutzen `arcunp.c`, `arcusq.c` und `arclzw.c`. `LICENSE` und `COPYING` der Autoren dokumentieren GPL-Lizenzierung. Referenzalgorithmen werden unverändert nur unter build/toolchain mit einem projekteigenen Testadapter kompiliert. Sie werden weder in das Produkt kompiliert noch mit ihm ausgeliefert.

Der unabhängige Vergleich nutzt Russell Marks’ nomarch 1.4 unter GPL Version 2 oder später. Das ursprüngliche Quellpaket ist im [Debian-Quellpool](https://deb.debian.org/debian/pool/main/n/nomarch/) verfügbar, SHA-256 `fe20da34e0d3ba0cf6388701f44ac22224cf65130ddbc5fcbc27bc4949a6e1ad`, 22411 Bytes, passend zum Quelldeskriptor des Betreuers. Ein strenger Reinraum wird nicht behauptet. Der Produktcode drückt Drahtfelder mit projekteigenen Bit-, Eingabe-, Arbeits- und Allokationsprimitiven aus. Der [eigene Testdatensatz](../../../tests/fixtures/arc/README.md) unterscheidet native Encoderausgabe, selbst erstellte Drahteingaben, unabhängige Vergleiche vollständiger Bytes und Referenzgrenzen.

<a name="xdzip-method-scope"></a>

## Methodenbereich

| Methode | Drahtprofil | Nachfolgende Stufe |
| --- | --- | --- |
| 1 | Veraltete 25-Byte-Struktur mit impliziter Ursprungslänge | Store |
| 2 | Gewöhnliche 29-Byte-Struktur | Store |
| 3 | Bytes mit DLE-`0x90`-Läufen und maskiertem DLE | Keine |
| 4 | Expliziter Squeeze-Baum mit vorzeichenbehafteten Kindern, LSB-zuerst-Codes und EOF-Symbol 256 | RLE von Methode 3 |
| 5 | Feste 12-Bit-Codes MSB zuerst, ursprüngliches Hash-Wörterbuch | Keine |
| 6 | Gleiches ursprüngliches Hash-Wörterbuch | RLE von Methode 3 |
| 7 | Feste 12-Bit-Codes, verbesserter Hash | RLE von Methode 3 |
| 8 | Anfangsbyte für maximale Breite 12, gruppiertes LSB-LZW mit 9–12 Bit | RLE von Methode 3 |
| 9 | Kein Breitenheader, gruppiertes LSB-LZW mit 9–13 Bit | Keine |

Unbekannte Methoden, Spark-Erweiterungen, Passwörter und SFX-Struktur erfordern gesonderte Containerzulassung. Ein gemeinsamer Algorithmusname erlaubt nicht Bitreihenfolge, Wörterbuch, Ausrichtung, Reset oder RLE-Stufe einer anderen Methode.

<a name="xdzip-rle-and-squeeze-trees"></a>

## RLE und Squeeze-Bäume

Ein normales Byte wird ausgegeben und zur Wiederholungshistorie. DLE gefolgt von null gibt ein wörtliches DLE aus, ohne diese Historie zu ändern. Eine Anzahl ungleich null gibt `count - 1` Kopien des vorherigen normalen Bytes aus; eine Anzahl ohne Historie ist ungültig. Die Maskierung muss vollständig sein und die exakte decodierte Größe begrenzt jede Wiederholung. Der RLE-Zustand bleibt über Wörterbuchzeichenfolgen und Ausgabeblöcke des Aufrufers erhalten.

Squeeze beginnt mit einer Little-Endian-Knotenzahl von höchstens 256, gefolgt von zwei vorzeichenbehafteten 16-Bit-Kindern pro Knoten. Nicht negative Kinder bezeichnen interne Knoten; negative Werte codieren Symbol `-child - 1` von 0 bis 256. Die Wurzel ist null; die historische Form ohne Knoten hat implizite EOF-Kinder. Der vollständige Graph muss ein verwurzelter Baum ohne Zyklus, gemeinsam genutztes internes Kind, unerreichbaren internen Knoten oder Kante außerhalb des Bereichs sein; EOF muss erreichbar sein. Bits bis EOF vom niedrigsten an durchlaufen. Komprimierte Eingabe muss im selben Byte wie dieser Terminator enden; ungenutzte letzte Bits werden ignoriert. Decodierung bis zur angegebenen Größe ersetzt die EOF-Prüfung nicht.

<a name="xdzip-fixed-dictionaries"></a>

## Feste Wörterbücher

Das Wörterbuch mit 4096 Plätzen fügt zunächst Bytefolgen 0–255 mit Vorgänger `0xffff` ein. Codes bezeichnen physische Hashplätze. Der alte Hash nutzt die mittleren zwölf Bits des Quadrats von `((prefix + suffix) mod 65536) | 0x0800`. Methode 7 multipliziert stattdessen die 16-Bit-Summe mit 15073 und nimmt die zwölf niedrigsten Bits. Diese Konstanten definieren historische Drahtkennungen.

Kollisionen folgen der bestehenden Platzkette, beginnen die Suche 101 Plätze hinter deren letztem Element und suchen dann linear mit Umlauf. Jede Suche und Präfixexpansion ist begrenzt und wird berechnet. Jede neue Zeichenfolge ist die vorherige plus das erste Byte der aktuellen. Ein noch nicht vorhandener Code wird nur für den exakt neu eingefügten KwKwK-Platz zugelassen. Volle Wörterbücher wachsen nicht weiter. Zwölf-Bit-Codes stehen byteübergreifend mit hohem Halbbyte zuerst, anders als spätere LSB-Profile. Das letzte ungenutzte Halbbyte wird ignoriert; fehlende Codebytes und zusätzliche vollständige Bytes sind ungültig.

<a name="xdzip-dynamic-dictionaries"></a>

## Dynamische Wörterbücher

Der erste Code muss ein Byte sein. Code 256 ist CLEAR und 257 beginnt sequenzielle Wörterbuchergänzungen. Die Breite steigt an Wörterbuchschwellen bis zum Methodenmaximum. Codes werden in Achtergruppen gepackt; Breitenwechsel oder CLEAR beendet die alte Gruppe, verbraucht ungenutzte Ausrichtungsbits und startet eine neue Gruppe. Ausrichtungsbits sind nicht spezifiziert. CLEAR setzt nächsten Wörterbuchplatz und Breite zurück, verlangt ein folgendes Literal und kann keine Wörterbuchzeichenfolge vor dem Reset ausleihen. Synthetische EOF-Codes oder Bytes werden nicht geliefert.

Wörterbuchwachstum, KwKwK, Betrieb bei Maximalbreite und adaptive Resets sind von der RLE-Stufe getrennt. Native Referenzeingaben mit erzwungenem LZW belegen jede Breite und tatsächliche Resets. Selbst erstellte CLEAR-Eingaben decken unabhängig alle acht Positionen der alten Gruppe ab.

<a name="xdzip-budgets-and-failures"></a>

## Budgets und Fehler

Komprimierte und decodierte Länge haben jeweils eine harte Grenze von 24 GiB, Arbeit eine harte Grenze von 96 GiB. Kleinere Aufrufergrenzen gelten vorrangig. Vollständige private Speicherzulassung beträgt 16 KiB für Methode 3, 32 KiB für Methode 4, 64 KiB für Methoden 5–7 und 96 KiB für Methoden 8–9. Diese Grenzen umfassen den Eingabepuffer von 8192 Bytes, Wörterbücher, umgekehrte Expansion, Graphvalidierung und Zustand.

Wörterbuchinitialisierung, verbrauchte Bits, Baumkanten, Hashsonden, Präfixexpansion, Stapelausgabe und Ausgabe-/Wiederholungsarbeit werden berechnet. `work_remaining()` enthält Konstruktorarbeit für ein kumuliertes Konto des Containers. Kooperative Abfragen decken eingabeabhängige und -unabhängige Arbeit ab. Ausgabelesungen betragen höchstens 8192 Bytes. Kurze Eingabe, typisierte Hostfehler, Quotenerschöpfung, Abbruch, ungültige Codes/Bäume, nachlaufende Daten und abweichende decodierte Größe beenden die Instanz; der erste Fehler behält seine Nutzlast und spätere Lesevorgänge können nicht fortgesetzt werden. Abgeschlossene Leser liefern stabiles EOF ohne weitere Arbeit.

ARCs erforderlicher Nutzlast-CRC-16 und vollständige physische Struktur gehören zum Container. Roher Erfolg allein authentifiziert keinen Inhalt und gewährt keine Dateisystem-, Pfad-, Prozess-, Nachbardatei- oder Veröffentlichungsbefugnis.
