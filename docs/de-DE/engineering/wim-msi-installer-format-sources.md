<a name="xdzip-wim-msi-and-installer-format-sources-and-licenses"></a>

# Quellen und Lizenzen für WIM, MSI und Installationsformate

[English](../../engineering/wim-msi-installer-format-sources.md) | [简体中文](../../zh-CN/engineering/wim-msi-installer-format-sources.md) | [繁體中文（台灣）](../../zh-TW/engineering/wim-msi-installer-format-sources.md) | [日本語](../../ja-JP/engineering/wim-msi-installer-format-sources.md) | [한국어](../../ko-KR/engineering/wim-msi-installer-format-sources.md) | [Français](../../fr-FR/engineering/wim-msi-installer-format-sources.md) | Deutsch | [Español](../../es-ES/engineering/wim-msi-installer-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/wim-msi-installer-format-sources.md) | [Русский](../../ru-RU/engineering/wim-msi-installer-format-sources.md)

Diese Seite beschreibt technische Referenzen, Lizenzbeschränkungen und Voraussetzungen für die Unterstützung von WIM, MSI und Installationsformaten. Referenzen bleiben in `build/toolchain/references` oder verweisen auf öffentliche Spezifikationen; keine davon ist eine Produktabhängigkeit.

<a name="xdzip-wim"></a>

## WIM

- Referenzimplementierung: `ebiggers/wimlib`, Commit `cd5e231c348c255ae5088873b5a66ee0eb96fa07`.
- Das Repository bietet GPLv3+ für das Projekt und eine bedingte LGPL-2.1+-Option für den Bibliotheksteil. Die LGPL-Option setzt unter anderem voraus, dass Abhängigkeiten Dritter wie `libntfs-3g` keine GPL-Bedingungen erzwingen.
- WIM ist ein Abbildcontainer und kein gewöhnliches Archiv. Ein begrenzter Leser muss Ressourcentabellen, Metadatenströme, blockweise gespeicherte Ressourcen, Xpress/LZX/LZMS-Kompression, Integritätstabellen, Metadaten harter Verknüpfungen und die Abbildauswahl verarbeiten, ohne ein Abbild einzuhängen oder Sicherheitsberechtigungen wiederherzustellen.

Die Referenzimplementierung ist für die Erforschung des Formats und der Testdaten nützlich, darf aber nach der aktuellen Richtlinie gegen Laufzeitabhängigkeiten Dritter nicht mit xdzip verknüpft werden. WIM ist **durch Produktentscheidung ausgeschlossen**: Das Projekt übernimmt weder den spezifischen Abbildleser noch das Modell der Dekompressionsverstärkung, Integritätsressourcen, die Semantik harter Verknüpfungen und den vollständigen Testkorpus, die für die Unterstützung erforderlich wären. Ohne ausdrückliche Neubewertung dieser Entscheidung sind keine Einträge für Registry, MIME, Laufzeit oder CLI zulässig.

<a name="xdzip-compound-file-and-msi"></a>

## Compound File und MSI

- Microsoft [MS-CFB] ist die wichtigste öffentliche Spezifikation: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b . Die Seite veröffentlicht PDF/DOCX-Fassungen und einen Hinweis zu geistigen Eigentumsrechten, der die Implementierung der beschriebenen Technologie erlaubt. Patentlizenzen unterliegen weiterhin den anwendbaren Microsoft-Programmen.
- Vor jeder MSI-spezifischen Interpretation muss ein CFB-Leser die Sektor- und FAT/DIFAT-Zuordnung, das Durchlaufen des Rot-Schwarz-Baums der Verzeichnisse, die Mini-Stream-Zuordnung, Grenzen für Stromgrößen und die Erkennung von Zyklen behandeln.

Ein eigenständiger CFB-Metadatenleser ist technisch möglich und als schreibgeschützter Teil `xdzip-codec-cfb` implementiert. Bevor er Speicher- und Stromzeilen bereitstellt, prüft er CFB-3.0/4.0-Header einschließlich der Sektorgrundlagen von 512 und 4096 Byte, FAT/DIFAT-Ketten, Verzeichnisbaum, Mini-Stream-Zuordnung, Stromgrößengrenzen und Zyklen. Das Kopieren roher Ströme ist nur nach Abgleich mit dem unveränderlichen Metadaten-Snapshot möglich. MSI-Tabellen, eingebettete Cabinets, benutzerdefinierte Aktionen und Transformationen bleiben außerhalb des Vertrags: Es werden weder Installationsprogramme ausgeführt noch benutzerdefinierte Aktionen interpretiert, Pakete installiert oder Änderungen zurückgeschrieben. Die Endung `.msi` verleiht keine Ausführungsberechtigung. Testdaten und Laufzeitnachweise sind in den Codec- und Laufzeittests automatisiert.

<a name="xdzip-nsis-and-other-sfx"></a>

## NSIS und andere SFX

- NSIS-Referenzquelle: `kichik/nsis`, Commit `a96303eae3115221471bb127fa916a13d8bf9ecd`.
- Die Datei `COPYRIGHT` nennt zlib/libpng für den größten Teil des Quellcodes und den zlib-Kompressor, bzip2 für das bzip2-Modul und Common Public License 1.0 für das LZMA-Modul.
- NSIS-Dateien sind ausführbare Installationsprogramme mit Skripten und eingebetteten Nutzdaten. Ein Parser muss unterhalb der Ausführungsgrenze bleiben, Header- und Nutzdatenbereiche begrenzen und Kompressionsmethoden vom Verhalten des Installationsprogramms trennen.
- Die öffentliche Quelle definiert den festen Datensatz `firstheader`: `flags`, `0xDEADBEEF`, die Signaturwörter `NullsoftInstallSystem`, `length_of_header` und `length_of_all_following_data`. Ein statischer Leser kann diesen Datensatz ohne Öffnen des PE-Startprogramms finden und prüfen; für komprimierte Headerblöcke und vollständige eigene Installationsprogramme ist weiterhin ein unabhängiges Modell nötig.

NSIS ist nun **ausschließlich für statische Metadaten zugelassen**. Der Codec durchsucht nur an 512-Byte-Grenzen ausgerichtete Datensätze in einem begrenzten PE-Overlay, akzeptiert die zwei dokumentierten magischen `firstheader`-Werte, begrenzt Flags und Längen und verlangt, dass der angegebene anschließende Bereich in die physische Eingabe passt. Danach gibt er eine Metadatenzeile `NSIS/firstheader` aus. Er dekomprimiert weder den Installationsheader noch stellt er Nutzdatenbytes bereit. Der Registry-Eintrag `.exe` ist bewusst weder ein Drag-and-drop-Kandidat noch eine Shell-Zuordnung; maßgeblich ist die Inhaltserkennung. Der Parser ist unabhängig von Qt und Ausführungs-APIs für Installationsprogramme. Er führt niemals das PE-Startprogramm, Skripte, benutzerdefinierte Seiten, Plug-ins oder extrahierte Nutzdaten aus. Andere SFX-Formate bleiben nur dann Aliasformate, wenn ihr inneres Archiv die Inhaltsprüfung besteht.

<a name="xdzip-admission-gates"></a>

## Zulassungsvoraussetzungen

1. Quellrevisionen und Hashwerte der Lizenztexte vor Verwendung eines Referenzwerkzeugs festlegen.
2. Vollständige, weiterverteilbare Testdaten für jeden zugelassenen Container und jede Kompressionsmethode erstellen; ein Beispiel nur mit Header reicht nicht aus.
3. Die CFB/WIM/NSIS-Analyse unabhängig von Qt sowie von APIs für Installation, Einhängen, Registry oder Skriptausführung halten.
4. Laufzeit-, CLI- und Desktop-Einträge erst nach bestandenen Tests für genaue physische Identität, doppelte Einträge, Zyklusgrenzen, kumulative Eingabe-, Arbeits- und Ausgabegrenzen, Abbruch, Quelländerung und endgültiges Fehlschlagen hinzufügen.
5. GPL/LGPL/CPL-Referenzkomponenten in build/toolchain belassen, sofern nicht eine gesonderte Abhängigkeits- und Lizenzentscheidung die Produktrichtlinie ausdrücklich ändert.
