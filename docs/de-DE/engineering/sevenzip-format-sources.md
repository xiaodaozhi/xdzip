<a name="xdzip-7z-format-sources-and-independent-implementation"></a>

# Quellen des 7z-Formats und unabhängige Implementierung

[English](../../engineering/sevenzip-format-sources.md) | [简体中文](../../zh-CN/engineering/sevenzip-format-sources.md) | [繁體中文](../../zh-TW/engineering/sevenzip-format-sources.md) | [日本語](../../ja-JP/engineering/sevenzip-format-sources.md) | [한국어](../../ko-KR/engineering/sevenzip-format-sources.md) | [Français](../../fr-FR/engineering/sevenzip-format-sources.md) | Deutsch | [Español](../../es-ES/engineering/sevenzip-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/sevenzip-format-sources.md) | [Русский](../../ru-RU/engineering/sevenzip-format-sources.md)

Dieses Dokument hält wiederverwendbare Quellen und Implementierungsgrenzen fest. Migrationsabnahme und verbleibende Fähigkeiten werden in [Issue #50](https://github.com/xiaodaozhi/xdzip/issues/50) verfolgt; feste Rust-Routen und die Entfernung der offiziellen Produktabhängigkeit belegen nicht automatisch plattformübergreifende, manuelle oder Release-Abnahme. Allgemeine Ersatzregeln stehen im [Vertrag für unabhängige Implementierungen](../../engineering/independent-rust-codecs.md).

<a name="xdzip-specifications-and-reference-identity"></a>

## Spezifikationen und Referenzidentität

Containerfelder, Ganzzahlen variabler Länge, Methodengraphen, Eigenschaften und Dateizuordnungen folgen den offiziellen `DOC/7zFormat.txt` und `DOC/Methods.txt`. Festgelegte Referenz ist das [offizielle 7-Zip-26.05-Quellarchiv](https://github.com/ip7z/7zip/releases/download/26.05/7z2605-src.tar.xz), SHA-256 `0b6080b8c3e7a579161d2d495224c0e51c467576cb4c7048b0ab485ef44cc22e`. Die beiden Dokumente haben jeweils SHA-256 `8ec976b73ca88710286be1e991e49dbab485d9c0bf2410a80fa87e657850f82b` und `b16ea5fdc50345fa2cb031f06fe410cea0b089f74a4138ac8141b9e2d816ffc7`.

Der LZMA-Bitstrom bezieht sich auch auf Igor Pavlovs [offizielles Spezifikationspaket](https://www.7-zip.org/a/lzma-specification.7z) vom 2015-06-14, Archiv-SHA-256 `4529aca392d6b9598f72086e7d136b1e05d2863bf7a757836d3fd4f49e623b58`; der Spezifikationstext hat SHA-256 `6a4d441c2fa1d8d37b0189c7147a4b5424d5cdb122a7fc1fbebd022864ed54fb`. Gelesen werden nur Formattext, Spezifikationsmathematik und Pseudocode; der Referenzdecoder im Paket dient nicht als Eingabe für eine Rust-Portierung. Das Herunterladen von Dokumenten oder Beispielen gewährt keine neue Weiterverbreitungslizenz.

Generisches Deflate folgt RFC 1951 und teilt eine Implementierung mit ZIP, GZIP und MSZIP. Deflate64 wählt darin ausdrücklich erweiterte Regeln: 64 KiB Verlauf, Distanzcodes 30/31 und Längencode 285; gewöhnliches Deflate/MSZIP bleibt unverändert. Der Abschnitt zu erweitertem Deflate in [PKWARE APPNOTE](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT) ist nur ein Überblick. Konkrete Bitstromunterschiede ergänzen faktische Analyse der festgelegten Referenz und unabhängige Leseprüfung; der Überblick darf nicht als vollständige Spezifikation gelten. AES-CBC-Mathematik und unabhängige Vektoren folgen [NIST SP 800-38A](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf). 7z-Eigenschaften, UTF-16LE-Passwortableitung und Nullauffüllung sind zusätzliche Formatregeln und lassen sich nicht aus ZIP AES oder PKCS#7 ableiten. BZip2-Handbuch und PPMd-Formatname spezifizieren historische Bitströme, Modellneustart und Terminierung nicht vollständig; diese Implementierungen dürfen nicht als allein durch Handbücher erreichbar beschrieben werden.

<a name="xdzip-reference-analysis-and-original-coding"></a>

## Referenzanalyse und eigenständige Codierung

Eine analysierende Person liest gezielt folgende Gruppen im festgelegten Originalarchiv, um ausgelassene Felder, mathematische Zustände, Kapazitätsneustarts, Terminierung und Fehlerbeziehungen zu ergänzen. Übergaben enthalten nur Fakten und Identifikationsanforderungen, keinen Referenzcode, zeilenweise Umschreibung, Klassen- oder Arraylayouts, Funktionszerlegung, kopierten Kontrollfluss oder übersetzte Kommentare. Rust-Algorithmen werden von Personen unabhängig organisiert, die die entsprechenden Referenzimplementierungen nicht gelesen haben; eine analysierende Person implementiert keinen von ihr analysierten Algorithmus. Auch die Person für den Container hat die offizielle Containerimplementierung nicht gelesen und codiert unabhängig anhand von Dokumentation und sachlicher Übergabe.

| Analysethema | Referenzumfang im festgelegten Archiv | Lizenzunterscheidung |
| --- | --- | --- |
| LZMA1/LZMA2 | `C/LzmaDec.c/.h`, `C/Lzma2Dec.c/.h`, Range-, Chunk/Reset- und Eigenschaftsteile der Encoder sowie C++-Wrapper Lzma/Lzma2 | Diese C-Dateien erklären Public Domain; C++-Wrapper unterliegen den Standardbedingungen des Archivs |
| Containerdetails | Externe Attribute, Prüfsummenvererbung von Teilströmen und Dateieigenschaften in `CPP/7zip/Archive/7z/7zIn.cpp` | Archivstandard LGPL-2.1-or-later |
| 7z AES | `CPP/7zip/Crypto/7zAes.cpp/.h`, Blocklänge, Passwort, Auffüllung und Abschluss in MyAes, FilterCoder und 7z-Codec-Wrappern | Archivstandard LGPL-2.1-or-later; zugrunde liegende AES-Implementierung wird weder gelesen noch portiert |
| BZip2 | BZip2Decoder, Const, Crc; Fehlerbehandlung gewöhnlicher 7z- und eigenständiger bz2-Wrapper unterscheiden | Archivstandard LGPL-2.1-or-later; keine Lizenz einer anderen Implementierung aus dem Formatnamen ableiten |
| PPMd7 | Mathematische Zustände, virtuelle Kapazität und Neustartfakten in `C/Ppmd7.c/.h`, `Ppmd7Dec.c`, `Ppmd7Enc.c`; Eigenschafts- und Abschlussgrenzen im C++-Ppmd-Wrapper | Diese C-Dateien erklären Public Domain; C++-Wrapper nutzen Archivstandard. Kontextlayouts, Allokatorimplementierung und Kontrollfluss werden nicht kopiert |
| Deflate64 | 64-Bit-Modus in `CPP/7zip/Compress/DeflateConst.h` und DeflateDecoder, Deflate64Register sowie 7z-Eigenschaftszulassung | Archivstandard LGPL-2.1-or-later; Übergabe nur zu Formatunterschieden gegenüber gewöhnlichem Deflate |
| ISA, Delta, Swap | Skalare Semantik in `C/Bra.c`, `Bra86.c`, `Bra.h`, `Delta.c`, `SwapBytes.c` und C++-Wrapper für Methodenregistrierung, Eigenschaften und Ende | Diese C-Dateien erklären Public Domain; C++-Wrapper unterliegen gesondert dem Archivstandard. SIMD-Strategie ist keine Formatregel |
| BCJ2 | Vier Ströme und mathematischer Zustand in `C/Bcj2.c/.h`; C++-Bcj2Coder-Wrapper für Decoderabschluss und Größe | C-Dateien erklären Public Domain; C++-Wrapper nutzen Archivstandard. Der Encoderkörper wird weder analysiert noch kopiert |

Identitäten des festgelegten Archivs und der Dateien, tatsächlichen Leseumfang und sachliche Übergabezusammenfassungen mit Entwicklungsnachweisen aufbewahren. Bei Aktualisierung der Referenz erneut prüfen; alte Zeilennummern belegen nicht das Verhalten einer neuen Version. Spezifikationskonstanten, Kennungen und mathematische Beziehungen können zur Formatimplementierung genutzt werden; interne Optimierungen und historisch undefiniertes Verhalten gewähren dem Produkt keine Befugnisse. Public-Domain-Erklärungen gelten nicht für das gesamte Archiv, Dokumentation, C++-Wrapper oder andere Beispiele.

Dies behauptet weder einen strengen Reinraum noch null Rechtsrisiko. Fremdquellen, frühere Lieferungen mit offiziellem Backend und unabhängige Testwerkzeuge unterliegen ihren jeweiligen Bedingungen. Das aktuelle Produkt baut das offizielle Backend nicht; siehe [Lizenzen](licensing.md) und [korrespondierenden Quellcode](../../engineering/source-delivery.md).

<a name="xdzip-interoperability-boundaries"></a>

## Interoperabilitätsgrenzen

Erfolg zwischen eigenem Writer und Reader ersetzt keine unabhängige Referenz. Tests prüfen getrennt bekannte Ausgaben unabhängiger Kompressoren, offizielles Lesen eigener komprimierter Ströme, vollständigen Verbrauch der Eingabe, Terminierung, CRC, Beschädigung, Quoten, Abbruch und Quellfehler. Komponentenprüfung und Host-, Prozess- sowie Plattformabnahme werden getrennt erfasst.

Historische Kompatibilitätstoleranzen müssen einzeln bestätigt werden. Beispielsweise kann ein BZip2-7z-Knoten ungenutzte Nichtnullbits im letzten Byte und genau vier wiederholte Bytes am Blockende ohne zusätzliches Zählbyte akzeptieren, verlangt aber weiterhin korrektes CRC, deklarierte Größe und vollständige Stromterminierung. Zusätzliche Bytes oder verkettete Ströme werden dadurch nicht zugelassen. Bits im letzten Byte und zusätzliche Bytes sind keine gleichartige Auffüllung.

PPMd7 unterscheidet Abschlussprofile bei bekannter und unbekannter äußerer Länge. Nach exakter Ausgabe prüft ein 7z-Knoten bekannter Länge Range-Endzustand und komprimierten Bereich, ohne ein zusätzliches END-Symbol zu verbrauchen; der gemeinsame Leser für unbekannte Längen verlangt END. Formatbedingte Modellkapazitätsneustarts und Host-Speicherzulassung sind getrennt: Speichermangel darf keinen erfundenen Formatneustart auslösen. Unabhängige Vergleichseingaben enthalten Kapazitätsneustartströme, die vom festgelegten offiziellen C-Encoder erzeugt wurden. Generator und ursprünglicher Referenzquellcode sind nur lokale Testwerkzeuge und keine Produktionsabhängigkeiten.
