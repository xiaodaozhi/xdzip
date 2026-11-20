<a name="xdzip-bandizip-format-inventory-and-xdzip-feasibility"></a>

# Bandizip-Formatinventar und Machbarkeit für xdzip

[English](../../engineering/bandizip-format-assessment.md) | [简体中文](../../zh-CN/engineering/bandizip-format-assessment.md) | [繁體中文](../../zh-TW/engineering/bandizip-format-assessment.md) | [日本語](../../ja-JP/engineering/bandizip-format-assessment.md) | [한국어](../../ko-KR/engineering/bandizip-format-assessment.md) | [Français](../../fr-FR/engineering/bandizip-format-assessment.md) | Deutsch | [Español](../../es-ES/engineering/bandizip-format-assessment.md) | [Português (Brasil)](../../pt-BR/engineering/bandizip-format-assessment.md) | [Русский](../../ru-RU/engineering/bandizip-format-assessment.md)

Diese Bewertung vergleicht die von Bandizip aufgeführten Formate mit dem Implementierungsumfang von xdzip und benennt Fähigkeiten, die Quell-, Fixture- und automatisierte Prüfung erfordern. Das Inventar stammt von der Bandizip-Produktseite und den Formathilfen, geprüft am 2026-11-15:

- Die [Bandizip-Produktseite](https://www.bandisoft.com/bandizip/) nennt: `7Z`, `ACE`, `AES`, `ALZ`, `ARJ`, `BH`, `BIN`, `BR`, `BZ`, `BZ2`, `CAB`, `Compound(MSI)`, `DAA(1.0)`, `DEB`, `EGG`, `GZ`, `IMG`, `ISO`, `ISZ`, `LHA`, `LZ`, `LZH`, `LZMA`, `PMA`, `RAR`, `RAR5`, `SFX(EXE)`, `TAR`, `TBZ/TBZ2`, `TGZ`, `TLZ`, `TXZ`, `UDF`, `WIM`, `XPI`, `XZ`, `Z`, `ZIP`, `ZIPX`, `ZPAQ`, `PEA`, `UU`, `UUE`, `XXE`, `ASAR`, `ZSTD`, `NSIS` und `LZ4`.
- Dieselbe Seite nennt `ZIP`, `7Z`, `ZIPX`, `EXE (SFX)`, `TAR`, `TGZ`, `LZH`, `ISO`, `GZ` und `XZ` als erstellbar; die längere Liste betrifft überwiegend Extraktion.
- Die [Bandizip-Hilfe zu ALZ und EGG](https://www.bandisoft.com/bandizip/help/how-to-open-alz-and-egg-files/) beschreibt ALZ Deflate/verschleiertes Deflate/BZip2 und ZipCrypto sowie EGG Deflate/BZip2/LZMA/AZO mit AES/LEA-Verschlüsselung, soliden Blöcken und Teilarchiven.
- Die [Bandizip-Hilfe zu ZPAQ](https://www.bandisoft.com/bandizip/help/what-is-a-zpaq/) beschreibt ZPAQ als eigenes journalbasiertes Archivformat und nennt die von Bandizip verarbeitbaren Kompressionsmethoden.
- Der [Quellenleitfaden zu Disk-Image- und anderen Archivformaten](disk-image-archive-format-sources.md) erfasst Prüfungen öffentlicher Quellen, Lizenzen, Korpora und Clean-Room-Verfahren für DAA, ISZ, ZPAQ, PEA, BOND und J2J. BOND und J2J sind nicht als aktuelle offizielle Bandizip-Archivformate bestätigt und bleiben historische Hinweise.

<a name="xdzip-current-xdzip-disposition"></a>

## Aktuelle Einordnung in xdzip

Die Tabelle verwendet folgende Bedeutungen. **Zugelassen** bedeutet, dass aktuelles Register, Runtime, CLI und Desktop-Pfad automatisierte Nachweise für das angegebene Profil haben. **Alias/Profil** bedeutet Zulassung der Endung über einen vorhandenen Container erst nach Inhaltsprüfung. **Kandidat** bezeichnet eine mögliche spätere Endung ohne behaupteten Decoder. **Blockiert** bedeutet, dass das Format erst mit einer öffentlichen, unabhängig testbaren Datenformatspezifikation und einer kompatiblen Implementierung oder eigenem Algorithmus verantwortbar zugelassen werden kann. **Ausgeschlossen** bedeutet, dass das Projekt ausdrücklich gegen Unterstützung entschieden hat; bis zu einer erneuten Produktentscheidung erhält das Format weder Registereintrag noch Detektor.

| Bandizip-Name | xdzip-Status | Vertretbarer aktueller Umfang oder Grund |
| --- | --- | --- |
| 7Z | Zugelassen | Unabhängiges Rust-7z-Backend einschließlich der deklarierten Passwort-, Solid- und Teilarchivprofile. |
| ACE | Ausgeschlossen | Produktentscheidung: ACE dauerhaft aufgeben; kein Decoder, Detektor, keine Endung, kein MIME- oder Zuordnungseintrag. |
| AES | Zugelassen | AES Crypt v0–v3, Passwort- und HMAC-Profil; dies ist der `.aes`-Umschlag, keine beliebigen verschlüsselten Archive. |
| ALZ | Zugelassen | Schreibgeschütztes gewöhnliches einzelnes Volumen mit Store und rohem Deflate, exakten Größen und CRC-32; modifiziertes BZip2, ZipCrypto und Teilarchive bleiben außerhalb. |
| ARJ | Zugelassen | Schreibgeschützt Store und Methoden 1–4 mit vollständigen Headern, Größen und Mitglieds-CRC-32. |
| BH | Zugelassen | Schreibgeschützt gewöhnliche `BH 05 07`-Mitglieder, Methode 8 rohes Deflate, exakte Größen und CRC-32. Store, Verschlüsselung, Teilarchive und andere Methoden ausgeschlossen. |
| BIN | Kandidat | Endung ist für Disk-, ausführbare und Datendateien mehrdeutig. Nur ein validiertes inneres Format erkennen; generisches BIN nie als Archivdecoder registrieren. |
| BR | Zugelassen | Begrenzter Brotli-Strom und TAR.BR-Profile. |
| BZ / BZ2 | Zugelassen | BZip2-Einzeldateiströme und deklarierte TAR.BZ2-Aliase. |
| CAB | Zugelassen | Unabhängiges Rust-CAB-Backend mit Store, MSZip, Quantum, LZX und autorisierten Cabinet-Sätzen. |
| Compound(MSI) | Zugelassen | Schreibgeschützte CFB-Struktur, Storage-/Stream-Namen und begrenztes Kopieren roher Streams für validierten `.msi`-Inhalt. MSI-Tabellen, Interpretation eingebetteter CABs, Transformationen, benutzerdefinierte Aktionen und Installation bleiben außerhalb. |
| DAA(1.0) | Ausgeschlossen | Dauerhaft nicht unterstützt per Produktentscheidung; kein Detektor, Codec, Registereintrag, Leser oder Schreiber wird zugelassen. |
| DEB | Zugelassen | Gewöhnlicher `ar`-Umschlag, control/data-Ansichten und zugelassene komprimierte Mitglieder. |
| EGG | Zugelassen | Schreibgeschützt unverschlüsseltes, nicht solides gewöhnliches Einzelvolumen mit Store und rohem Deflate, exakten Blockgrößen und CRC-32; AZO, BZip2, LZMA, Verschlüsselung, Teilarchive und andere Profile bleiben außerhalb. |
| GZ | Zugelassen | GZIP-Mitglieder einschließlich verketteter Ströme und TAR.GZ. |
| IMG | Kandidat | Mehrdeutige Disk-Image-Endung. Inhaltsbasierte ISO-/UDF-Zulassung bedeutet keine generische IMG-Unterstützung. |
| ISO | Zugelassen | ISO9660/Joliet/Rock-Ridge-Lesen und gewöhnliche ISO9660 + Joliet-Datenabbilderstellung. |
| ISZ | Ausgeschlossen | Produktentscheidung: ISZ aufgeben; kein Detektor, Registereintrag oder Schlüsselbehandlung wird hinzugefügt. |
| LHA / LZH | Zugelassen | LHA-Level 0–3, Store, LH1–LH7, LZS/LZ5 und LH5-Erstellung. |
| LZ | Zugelassen | Lzip v1 und TAR.LZ. |
| LZMA | Zugelassen | LZMA Alone und TAR.LZMA. |
| PMA | Zugelassen | Schreibgeschützte PM0/PM1/PM2-Profile mit CP/M-Rahmung und CRC-Prüfungen. |
| RAR / RAR5 | Zugelassen | Unabhängiges Rust-RAR-Backend einschließlich älterer Formate, RAR5, RAR7-Erweiterungen, Passwörtern, Solid- und Teilarchivprofilen im deklarierten Umfang. |
| SFX(EXE) | Alias/Profil | ZIP-SFX-Präfixe werden vom ZIP-Leser validiert. Generische Extraktion oder Ausführung ausführbarer Dateien wird nie gestattet. Eigene Windows-x64-Store/Deflate-ZIP-Erstellung ist über das ausdrückliche `.sfx.exe`-Profil ohne automatisches Ausführen verfügbar. |
| TAR | Zugelassen | TAR und gefilterte TAR.GZ/TAR.BZ2/TAR.LZMA/TAR.LZ/TAR.XZ/TAR.Z/TAR.BR/TAR.LZ4/TAR.ZST-Profile gemäß Register. |
| TBZ / TBZ2 | Alias/Profil | BZip2-gefilterte TAR-Aliase. |
| TGZ | Alias/Profil | GZIP-gefilterter TAR-Alias. |
| TLZ | Alias/Profil | LZMA-gefilterter TAR-Alias. |
| TXZ | Alias/Profil | XZ-gefilterter TAR-Alias. |
| UDF | Zugelassen | Schreibgeschützte UDF-Abbilder einschließlich Namensraumregeln für Hybridabbilder. |
| WIM | Ausgeschlossen | Produktentscheidung: den für WIM benötigten Imaging-spezifischen Leser, das Ressourcenmodell und den Korpus nicht verfolgen. |
| XPI | Alias/Profil | ZIP-Inhalt unter XPI-Endung; Inhaltsprüfung bleibt maßgeblich. |
| XZ | Zugelassen | XZ und TAR.XZ mit LZMA2, deklarierten Prüfungen und zugelassenen Filtern. |
| Z | Zugelassen | Unix-compress-LZW und TAR.Z/TAZ-Aliase. |
| ZIP | Zugelassen | ZIP/ZIP64, AES-256 und unterstützte Teilarchiv-, Methoden- und Umschreibprofile. |
| ZIPX | Alias/Profil | ZIPX ist für validierte ZIP-Methoden wie XZ zugelassen; keine eigene Containerimplementierung. ZIPX-Erstellung wird derzeit nicht angeboten. |
| ZPAQ | Ausgeschlossen | Produktentscheidung: die für dieses solide, journalbasierte Format nötige virtuelle Maschine ZPAQL nicht implementieren. |
| PEA | Zugelassen | Schreibgeschütztes PEA-1.x-Einzelvolumenprofil mit UTF-8 und unverschlüsselten gespeicherten PCOMPRESS0-Dateien/-Verzeichnissen; Prüfungen, Verschlüsselung, Kompression, Teilarchive, Erstellung und Änderung bleiben außerhalb. |
| UU / UUE | Zugelassen | Begrenzte UU/UUE-Umschläge mit Validierung physischer Mitglieder und Zeilen. |
| XXE | Zugelassen | Begrenztes XXE-Umschlagprofil. |
| ASAR | Zugelassen | Geordnete Metadaten, gespeicherte Bereiche und deklarierte SHA-256-Prüfungen. |
| ZSTD | Zugelassen | Standard-Zstandard-Frames und TAR.ZST/TZST-Profile. |
| NSIS | Zugelassen | Nur statische `firstheader`-Metadaten. Inhaltsprüfung validiert ein begrenztes PE-Overlay; `.exe` ist kein Drag-and-drop- oder Shell-Zuordnungseintrag, und PE, Skript, Plugin oder Nutzdaten dürfen nicht ausgeführt werden. |
| LZ4 | Zugelassen | Standard-LZ4-Frames und TAR.LZ4. |

Das Inventar rechtfertigt daher keine generischen Handler für `BIN`, `IMG` oder `SFX(EXE)`. Diese Namen können ausführbare Dateien, Disk- oder Installerdaten bezeichnen; eine Endung allein belegt keine Archivfähigkeit. NSIS besitzt einen engeren, inhaltsvalidierten Leser statischer Metadaten; er gewährt niemals Ausführungs- oder Nutzdatenautorität und sein `.exe`-Eintrag ist für Drag-and-drop und Shell-Zuordnungen verborgen. Compound(MSI) hat inzwischen einen inhaltsvalidierten CFB-Leser; auch die Endung `.msi` gewährt keine Ausführungsautorität.

Siehe [Quellen zu Disk-Image- und Archivformaten](disk-image-archive-format-sources.md) für DAA, ISZ, ZPAQ, PEA, BOND und J2J. PEA ist im dokumentierten konservativen schreibgeschützten Profil verfügbar. ACE, DAA, ISZ und ZPAQ sind per Produktentscheidung ausgeschlossen. BOND und J2J sind ausgeschlossen, weil ihre Identität als Bandizip-Archivformate nicht bestätigt ist.

<a name="xdzip-delivery-plan"></a>

## Lieferplan

1. **Inventar und Registergrenze.** Einen maschinenlesbaren Eintrag je zugelassenem Profil in `specs/engineering/archive-formats.json` erhalten. Rust-, C++-, JavaScript- und Zuordnungskonsumenten generieren und beide Registerprüfungen ausführen. Jedes neue Format erhält einen Formatvertrag und zugehörigen Abnahmeabsatz.
2. **Datenformatvertrag vor Code.** Öffentliche Quellrevisionen und SHA-256 fixieren; Headerfelder, Namenskodierung, physisches Ende, Methoden, Prüfsummen, Duplikatidentität und Ressourcenlimits dokumentieren. Ein Referenzwerkzeug ist nur bei vollständigen eigenen Beispielen ein Nachweis; Absturz oder ein einzelnes Store-Beispiel sind als Einschränkung zu erfassen.
3. **Rohe Algorithmustests.** Begrenzten Leser unabhängig von Qt implementieren. Leere und Grenznutzdaten, jede zugelassene Methode, exakte Eingabe-/Ausgabe-/Arbeits-/Speicherbudgets, Abbruch, typisierte E/A, Quelländerung, Abschneidung, nachfolgende Bytes, CRC und terminale Sperre abdecken. Unbekannte Methoden bleiben ohne Nutzdatenautorität sichtbar.
4. **Container- und Produktzulassung.** Codec-Crate, schwache/starke Runtime-Erkennung, CLI list/test/extract, Desktop-Labels und Zuordnungsfixtures hinzufügen. Eingefrorene Metadaten und Auswahl physischer Duplikate prüfen, dann relevante automatische Qt- und Layoutprüfungen ausführen. Für diesen Forschungs- und Formatabschnitt ist keine manuelle visuelle Abnahme erforderlich.
5. **Prioritätenliste.** ALZs konservativer Store-/Roh-Deflate-Teil, EGGs unverschlüsselter, nicht solider Store-/Roh-Deflate-Teil, CFB-/Compound(MSI)-Metadaten, PEAs konservatives Speicherprofil und statische NSIS-`firstheader`-Metadaten sind zugelassen. ACE, DAA, WIM, ISZ, ZPAQ, BOND und J2J sind per Produktentscheidung dauerhaft ausgeschlossen. ZIPX-Erstellung bleibt aus aktuellen Schreibern ausgeschlossen. Der [eigene SFX-Schreiber](sfx-creation.md) ist separat implementiert und folgt nicht aus der Lesealias-Unterstützung.
6. **Release-Grenze.** Kandidatenformate aus der öffentlichen Fähigkeitstabelle heraushalten, bis Codec-, Runtime-, CLI- und Desktop-Nachweise vollständig sind. Nicht unterstützte Methoden und Einschränkungen von Referenzwerkzeugen im Issue erfassen, statt stillschweigend auf Store oder einen anderen Container zurückzufallen.

Der aktuelle BH-Teil folgt diesem Plan: schreibgeschützt, ohne Passwort- oder Teilarchivpfad, mit unabhängig begrenztem Deflate-Leser, verpflichtendem CRC-32 und exaktem physischem Ende. Der Compound(MSI)-Teil folgt derselben Grenze: CFB-FAT/DIFAT-, Verzeichnisbaum-, Mini-Stream- und Zyklusprüfung sind Qt-unabhängig; rohe Streams werden erst nach Übereinstimmung des unveränderlichen Metadaten-Snapshots kopiert. Der NSIS-Teil endet beim festen firstheader-Metadatensatz und meldet Nutzdatenzugriff als nicht unterstützt. Weder MSI- noch NSIS-Installerverhalten wird ausgeführt.
