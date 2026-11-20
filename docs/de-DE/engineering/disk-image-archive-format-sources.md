<a name="xdzip-sources-and-licenses-for-disk-images-and-other-archive-formats"></a>

# Quellen und Lizenzen für Disk-Images und andere Archivformate

[English](../../engineering/disk-image-archive-format-sources.md) | [简体中文](../../zh-CN/engineering/disk-image-archive-format-sources.md) | [繁體中文（台灣）](../../zh-TW/engineering/disk-image-archive-format-sources.md) | [日本語](../../ja-JP/engineering/disk-image-archive-format-sources.md) | [한국어](../../ko-KR/engineering/disk-image-archive-format-sources.md) | [Français](../../fr-FR/engineering/disk-image-archive-format-sources.md) | Deutsch | [Español](../../es-ES/engineering/disk-image-archive-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/disk-image-archive-format-sources.md) | [Русский](../../ru-RU/engineering/disk-image-archive-format-sources.md)

Diese Seite beschreibt öffentliche technische Referenzen, Lizenzbeschränkungen und Unterstützungsanforderungen für DAA, ISZ, ZPAQ, PEA, BOND, J2J und verwandte Formate. Eine öffentliche Formatbeschreibung erlaubt nicht automatisch die Auslieferung eines Decoders; eine Referenzimplementierung ist nicht zwingend eine weiterverteilbare Produktabhängigkeit. Keine der folgenden Referenzen wird in xdzip gelinkt. Bevor ein Format ins Produktregister gelangt, müssen Quellrevision, Lizenztext, vollständige Fixtures, Ressourcenlimits und Fehlerverhalten feststehen.

<a name="xdzip-daa-10"></a>

## DAA 1.0

- **Öffentliche Spezifikation:** Keine vom Hersteller veröffentlichte DAA-1.0-Spezifikation gefunden. Öffentliche Beschreibungen bezeichnen DAA als proprietäres Disk-/Archivabbild von PowerISO und beschreiben Header, Chunklängentabelle und unabhängig komprimierte ISO-Chunks. Öffentliche Angaben umfassen zlib- und LZMA-Varianten und spätere interne Versionen wie 110, definieren aber keinen stabilen 1.0-Vertrag.
- **Referenzimplementierung:** `daa2iso` 0.1.7e, das Debian-/Fedora-Paket unter GPL-2.0-or-later. Die Quelle dient nur historischer Forschung. Ein Upstream-Git-Commit oder eine Herstellerspezifikation lässt sich derzeit nicht aus einem maßgeblichen öffentlichen Repository fixieren.
- **Lizenz und Weiterverteilung:** Die Implementierung steht unter GPL-2.0-or-later. Diese Lizenz gewährt keine Rechte an proprietärer DAA-Technologie; das Projekt hat keinen unabhängigen Berechtigungsnachweis zur Auslieferung eines Decoders in xdzip.
- **Vollständige Beispiele:** Öffentliche Beispiele sind häufig Disk-Images oder mehrteilige Archive ohne fixierte Herkunft und Version. Kein weiterverteilbarer Korpus zu alten Headern, Version 110, Mehrteilreihenfolge, zlib, LZMA, Verschleierung, Prüfsummen und fehlerhaften Chunks ist derzeit zugelassen.
- **Clean-Room-Machbarkeit:** Eine begrenzte ISO-Rekonstruktion könnte nach stabilem Datenformat und rechtlicher Entscheidung technisch möglich sein. Reverse Engineering allein anhand eines GPL-Decoders genügt nicht als Nachweis für eine Produktimplementierung.
- **Fähigkeitsgrenze:** Per Produktentscheidung dauerhaft nicht unterstützt. Keinen DAA-Detektor, Codec, Registereintrag, MIME-Typ, keine Zuordnung, keinen Leser oder Schreiber hinzufügen. Historische Forschung autorisiert keine spätere Zulassung.
- **Fazit:** **Per Produktentscheidung dauerhaft ausgeschlossen.** xdzip verfolgt DAA nicht weiter. Die Erkenntnisse zu Quellen, Lizenzen und Beispielen bleiben historische Forschung und sind keine ausstehenden Lieferbedingungen.

<a name="xdzip-isz"></a>

## ISZ

- **Öffentliche Spezifikation:** EZB Systems, *ISZ File Format Specification*, Version 1.00, überarbeitet 2006-07-03, dokumentiert die `IsZ!`-Signatur, festen Header, optionale Segmentdefinitionstabelle, Chunkdefinitionstabelle und Datenblöcke. Headerfelder umfassen Sektorgröße, Gesamtsektoren, Passwortmodus, Segmentgröße, Blockanzahl, Blockgröße, Zeigerlänge, Segmentnummer sowie Tabellen- und Datenoffsets. Chunkflags unterscheiden Null-, Store-, zlib- und BZip2-Blöcke. Die Spezifikation ist in [der Kopie im ImgBurn-Forum](https://forum.imgburn.com/topic/14260-isz-images-supporting/#comment-115553) wiedergegeben.
- **Referenzimplementierungen:** ISZ-Filter von CDEmu `libmirage`, Commit `39fcffe09739e1dc40bef4781646f9ffb8b7dc4e`, und Play- `IszImageStream`, Commit `5bb010f27d8e2309f37b1f8eba0f74ecd546b29b`. Beide dienen nur der Referenzanalyse. `Helias/isz-tool` ist ein Python-Leser, aber ein maßgeblicher unveränderlicher Commit und Lizenznachweis waren in dieser Umgebung nicht abrufbar.
- **Lizenz und Weiterverteilung:** Abschnitt 8 der EZB-Spezifikation besagt, dass die Aufnahme von ISZ-Technologie in ein Produkt die Kontaktaufnahme mit EZB für eine kostenlose Lizenz unter Bedingungen erfordert. Ein öffentlicher Text oder ein unabhängig permissiv lizenzierter Leser beseitigt diese Produktlizenzbedingung nicht. CDEmu und Play- sind keine xdzip-Abhängigkeiten.
- **Vollständige Beispiele:** Gültige Store-, zlib- und BZip2-Blöcke können mit öffentlichen Werkzeugen erzeugt werden; eine vollständige weiterverteilbare Matrix benötigt zusätzlich Einzel- und Segmentabbilder, Nullblöcke, alle Zeigerlängen, exakte Endblöcke, Passwortmodi, Tabellenverschleierung, Abschneidung, Offsets und Quelländerungsfälle. Ein solcher eigener Korpus ist im Repository nicht erfasst.
- **Clean-Room-Machbarkeit:** Der unverschlüsselte, einvolumige, schreibgeschützte Teil ist technisch von ISO-Ansicht trennbar. Eine erste Implementierung könnte die vier dokumentierten Blockarten in eine virtuelle `image.iso`-Ausgabe dekodieren, ohne Mounten oder Bootausführung. Rechtliche Freigabe, unveränderlich fixierte Referenz und vollständige Fixtures müssen der Implementierung jedoch vorausgehen.
- **Fähigkeitsgrenze:** Vor Erfüllung dieser Bedingungen `.isz` nicht registrieren, Unterstützung nicht aus `IsZ!` ableiten und weder Passwort, AES, Segmentierung, Schreiben noch direkte optische Ansicht bereitstellen. Bei späterer Zulassung muss der Decoder Header/Version, Zeiger- und Segmenttabellen, XOR-/Entschleierungsregeln, Block-/Datengrenzen, logische Größenarithmetik, vollständiges Blockende, Abbruch sowie Eingabe-/Arbeits-/Ausgabelimits prüfen.
- **Fazit:** **Per Produktentscheidung ausgeschlossen.** xdzip verfolgt ISZ nicht weiter und kontaktiert EZB in diesem Arbeitspaket nicht für eine Produktlizenz. Ein `.isz`-Detektor oder Registereintrag darf nur nach erneuter Entscheidung hinzugefügt werden.

<a name="xdzip-zpaq"></a>

## ZPAQ

- **Öffentliche Spezifikation:** Matt Mahoney, *The ZPAQ Open Standard Format for Highly Compressed Data*, Versionen 2.03 und 2.06. Das Format speichert ein Dekompressionsprogramm als ZPAQL-Bytecode und unterstützt solide Blöcke, Journalführung, Deduplizierung, SHA-1-Datensätze, Verschlüsselung in späteren Stufen und mehrteilige Archive.
- **Referenzimplementierung:** `zpaq/zpaq`, Tag `6.22`, und zugehörige `libzpaq`-Quellen. Der Tag ist eine fixierte Forschungsreferenz für diese Bewertung; vor Implementierungsarbeit müssen Commit-Hash und Hash des Lizenztexts erfasst werden.
- **Lizenz und Weiterverteilung:** Referenzimplementierung und Konfigurationsdateien stehen unter GPLv3. Sie dürfen gemäß Richtlinie ohne Drittanbieter-Archivruntime nicht in das aktuelle Produkt gelinkt werden. Der öffentliche Standard erlaubt Clean-Room-Analyse, doch Decoder-VM und Konfigurationskorpus benötigen eigene Implementierung und Prüfung.
- **Vollständige Beispiele:** Ein brauchbarer Korpus muss Level-1- und -2-Blöcke, beliebige ZPAQL-Programme, Journalupdates, Deduplizierung, verschlüsselte und mehrteilige Datensätze, SHA-1-Prüfung sowie fehlerhafte VM- oder Blockgrenzen umfassen. Das Repository enthält keinen solchen eigenen Korpus.
- **Clean-Room-Machbarkeit:** Theoretisch möglich, aber ein Projekt für virtuelle Maschine und Ressourcenabrechnung statt eines kleinen Containerparsers. Sichere Limits müssen Bytecodeinterpretation, Modellspeicher, dekomprimierte Ausgabe, Historie und Deduplizierungszustand berechnen.
- **Fähigkeitsgrenze:** Ein erster zukünftiger Abschnitt wäre schreibgeschützt, unverschlüsselt, nicht mehrteilig und ohne Journalupdates mit deklariertem VM-Subset. Erstellung, Update, Passwortbehandlung oder unbegrenzte Solid-Extraktion folgen daraus nicht.
- **Fazit:** **Per Produktentscheidung ausgeschlossen.** xdzip implementiert die ZPAQL-VM nicht und akzeptiert kein davon abhängiges ZPAQ-Profil. Ein `.zpaq`-Detektor oder Registereintrag darf nur nach erneuter Entscheidung hinzugefügt werden.

<a name="xdzip-pea"></a>

## PEA

- **Öffentliche Spezifikation:** PeaZip, *Pack, Encrypt, Authenticate*-Spezifikation für PEA 1.x und das weiterhin vorgeschlagene PEA 2.0, Ströme, Volume- und Objektsteuerung, Endmarker und PCOMPRESS-Stufen. Die Spezifikation steht als [`pea_help.pdf`](https://peazip.github.io/pea_help.pdf) und zusammengefasst unter [pea-file-format.html](https://peazip.github.io/pea-file-format.html). Die aktuelle offizielle Seite stellt PEA- und PCOMPRESS-Spezifikationen ausdrücklich in die Public Domain.
- **Referenzimplementierung:** `peazip/PeaZip`, Commit `81b31f7e487b462980a52ac0f2e0b3cf700fe799`, einschließlich `peazip-sources/dev/pea_utils.pas` und `unit_pea.pas`. Die Implementierung steht unter LGPLv3; mitgelieferte kryptografische Einheiten von Wolfgang Ehrhardt verwenden die zlib/libpng-Lizenz. Die Quellveröffentlichung ist [peazip-11.4.0.src.zip](https://github.com/peazip/PeaZip/releases/download/11.4.0/peazip-11.4.0.src.zip). Die fixierte Quelle dient nur der Referenzanalyse.
- **Lizenz und Weiterverteilung:** Die Formatspezifikation selbst blockiert keine Produktlizenz, aber LGPLv3-Engine und kryptografische Abhängigkeiten dürfen nicht standardmäßig ins aktuelle Produkt kopiert oder gelinkt werden. Eine Rust-Clean-Room-Implementierung muss die Public-Domain-Formatbeschreibung und unabhängig gewählte Algorithmen verwenden; das Referenzprogramm bleibt nur für Tests.
- **Vollständige Beispiele:** Das zugelassene Profil hat selbst erstellte vollständige Store-Fixtures für Dateien, Verzeichnisse, gültige UTF-8-Namen, leere und nichtleere Daten, Abschneidung, nachfolgende Bytes, Quelländerung und Limits. Ein breiterer Korpus benötigt weiterhin PEA 1.x und 2.0, mehrere Ströme, Volume-Grenzen, PCOMP-Stufen, Authentifizierungsfehler, verschlüsselte Header, fehlerhafte Endmarker und große deklarierte Größen; diese Profile bleiben außerhalb des Produktumfangs.
- **Clean-Room-Machbarkeit:** Ein unverschlüsselter Einzelstrom-Store/Deflate-Leser ist technisch plausibel, Authentifizierung und Verschlüsselung erfordern aber eigene Sicherheitsarbeit. Die Integritätsgestaltung des Formats erlaubt nicht, reine Strukturanalyse als Inhaltsprüfung zu bewerben.
- **Fähigkeitsgrenze:** Eine künftige Zulassung wäre zunächst schreibgeschützt, ohne Passwort, Verschlüsselung, mehrere Volumes, Erstellung oder Änderung. Jeder extrahierte Strom müsste deklarierte Digest- und abschließende Authentifizierungsprüfungen vor Veröffentlichung bestehen.
- **Fazit:** **Als konservatives schreibgeschütztes Profil zugelassen.** xdzip lässt jetzt PEA 1.x, ein physisches Volume, UTF-8-Namen, gespeicherte PCOMPRESS0-Dateien/-Verzeichnisse und keine Objekt-, Strom- oder Volume-Prüfungen zu. Passwörter, authentifizierte/verschlüsselte Ströme, Prüfsummenalgorithmen, PCOMPRESS1–3, Teilarchive, PEA 2.0, Erstellung und Änderung bleiben nicht unterstützt. Die LGPL-Referenz bleibt nur für Tests und wird nicht ins Produkt gelinkt.

<a name="xdzip-bond"></a>

## BOND

- **Öffentliche Spezifikation:** Keine Bandizip-BOND-Archivspezifikation, Signatur, Version oder Kompressionsvertrag gefunden. Neuere Bandizip-Release-Notes erwähnen Schwachstellen in „einigen BOND-Dateien“, definieren aber kein Datenformat. Öffentliche `.bond`-Endungsdatenbanken beschreiben vorwiegend Microsoft-Bond-Schemadaten, ein anderes Serialisierungsformat.
- **Referenzimplementierung:** Keine Implementierung mit maßgeblicher Archivformatidentität, fixer Revision und weiterverteilbarer Lizenz gefunden.
- **Lizenz und Weiterverteilung:** Unbekannt. Solange Formateigentümer, Technologierechte und Referenzbedingungen unbekannt sind, darf kein Produkt oder Decoder zugelassen werden.
- **Vollständige Beispiele:** Keine aus vertrauenswürdiger versionierter Quelle. Eine Dateiendung allein ist keine akzeptable Fixture-Identität.
- **Clean-Room-Machbarkeit:** Ohne Signatur und ausreichend vollständige Beispiele zur Unterscheidung eines Archivs von Microsoft-Bond-Daten oder anderen Binärdaten nicht beurteilbar.
- **Fähigkeitsgrenze:** Keine Erkennung, Endungsregistrierung, Ansicht, Extraktion, Prüfung, Erstellung, Passwort- oder Volume-Unterstützung.
- **Fazit:** **Per Produktentscheidung ausgeschlossen und nicht als Bandizip-Archivformat bestätigt.** In diesem Arbeitspaket wird kein `.bond`-Detektor oder Registereintrag hinzugefügt.

<a name="xdzip-j2j"></a>

## J2J

- **Öffentliche Spezifikation:** Keine J2J-Archivspezifikation, Signatur, Version oder Entwicklerangabe gefunden. Historische Bandizip-Funktionslisten nennen J2J nur zur Extraktion, während aktuelle offizielle Formatseiten keine technische Definition geben. Andere J2J-Suchergebnisse betreffen nicht verwandte job-to-job-Datensätze oder JPEG-to-JPEG-Werkzeuge.
- **Referenzimplementierung:** Keine Implementierung mit fixer Revision und Lizenz gefunden, die einem Bandizip-Archivleser zugeordnet werden kann.
- **Lizenz und Weiterverteilung:** Unbekannt. Drittanbieter-Endungsseiten und alte Softwarelisten sind keine Lizenzerteilung.
- **Vollständige Beispiele:** Im Repository gibt es keine vertrauenswürdige weiterverteilbare Beispielmatrix. Ein `.j2j`-Name belegt keine Containeridentität.
- **Clean-Room-Machbarkeit:** Nicht beurteilbar, bevor Signatur, Datensatzmodell und Kompressions-/Verschlüsselungsverhalten dokumentiert und ein vollständiger Korpus verfügbar sind.
- **Fähigkeitsgrenze:** Keine Erkennung, Endungsregistrierung, Ansicht, Extraktion, Prüfung, Erstellung, Passwort- oder Volume-Unterstützung.
- **Fazit:** **Per Produktentscheidung ausgeschlossen und nicht als Bandizip-Archivformat bestätigt.** In diesem Arbeitspaket wird kein `.j2j`-Detektor oder Registereintrag hinzugefügt.

<a name="xdzip-requirements-for-format-support"></a>

## Anforderungen an Formatunterstützung

1. Maßgebliche Spezifikation oder rechtlich geprüften Clean-Room-Vertrag erfassen. Blog, Endungsdatenbank oder Decoderquelle allein genügen nicht.
2. Jede Forschungsquelle auf unveränderlichen Commit oder Inhalts-Hash fixieren und anwendbaren Lizenztext erfassen.
3. Vollständige weiterverteilbare Fixture-Matrix für jede beanspruchte Methode, Version, jeden Volumemodus, Verschlüsselungszustand und jede Endbedingung erstellen. Nur Header- oder Store-Beispiele belegen keine Interoperabilität.
4. Begrenztes Ressourcenmodell für Eingabezugriffe, Metadaten, Dekompressionsarbeit, Speicher, logische Ausgabe, Historie, kryptografischen Zustand und Abbruch definieren. Fehlerhafte Offsets, Zyklen, Überlauf und Quelländerungen vor Veröffentlichung ablehnen.
5. Jeden Decoder unabhängig von APIs für Qt, Mounten, Booten, Installer, Skripte, Registry und beliebige Prozessausführung halten.
6. Runtime, CLI, Zuordnungen und Benutzerdokumentation erst anbinden, nachdem Codec, Snapshot-Identität, exaktes Nutzdatenverhalten und Fehlerklassifikation gezielte Automatisierung bestehen.
7. Ein blockierter Kandidat bleibt in der Bewertung als blockiert sichtbar. Er darf nicht allein deshalb Endung, MIME-Eintrag oder Produktfähigkeit erhalten, weil Bandizip oder eine Drittanbieterliste ihn nennt.

Die Repository-Prüfung `python scripts/test-m10-source-gates.py` prüft, dass alle sechs zweisprachigen Abschnitte die erforderlichen Nachweisfelder behalten, jedes Fazit ausdrücklich blockiert, ausgeschlossen oder zugelassen bleibt und kein blockierter oder ausgeschlossener Kandidat in `specs/engineering/archive-formats.json` aufgenommen wurde.
