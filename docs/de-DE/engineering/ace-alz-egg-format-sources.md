<a name="xdzip-ace-alz-and-egg-source-and-licensing-gates"></a>

# Quellen und Lizenzvoraussetzungen für ACE, ALZ und EGG

[English](../../engineering/ace-alz-egg-format-sources.md) | [简体中文](../../zh-CN/engineering/ace-alz-egg-format-sources.md) | [繁體中文](../../zh-TW/engineering/ace-alz-egg-format-sources.md) | [日本語](../../ja-JP/engineering/ace-alz-egg-format-sources.md) | [한국어](../../ko-KR/engineering/ace-alz-egg-format-sources.md) | [Français](../../fr-FR/engineering/ace-alz-egg-format-sources.md) | Deutsch | [Español](../../es-ES/engineering/ace-alz-egg-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/ace-alz-egg-format-sources.md) | [Русский](../../ru-RU/engineering/ace-alz-egg-format-sources.md)

Diese Seite beschreibt die Unterstützung für ACE, ALZ und EGG in xdzip und die öffentlichen technischen Quellen und Lizenzbeschränkungen dahinter. Referenzdateien bleiben unter `build/toolchain/references` und gehören nicht zu den Produktabhängigkeiten.

<a name="xdzip-ace"></a>

## ACE

- Technische Notiz von Marcel Lemke, *Technical information of the archiver ACE v1.2*, aus `wdlkmpx/unace`, SHA-256 `F645C91348D5DCD560A19CA21DEF849D25C304EFFAACF65B03EFBB0A893E51F9`.
- Unabhängige Implementierung `droe/acefile`, Commit `11ed076ae19e8228148731cfbc05f019a395b3b9`. SHA-256 der BSD-2-Clause-Lizenz `AD376BD9695F0E355F5C523443ABF822BB896999A3252AAA614D313CC4F1F5B5`. SHA-256 der Implementierung `BC701569AD8521964C62256472AB44976AC57FB46E52B147A21018BC96348F1F`.
- Laut Implementierung unterstützt sie das Lesen von ACE 1.0 und 2.0 einschließlich LZ77, DELTA, EXE, PIC, SOUND, Passwörtern und Volumes. Die Lizenz gilt nur für die Python-Implementierung und beansprucht ausdrücklich keine Rechte an der ACE-Technologie.

Die technischen Angaben reichen zur Beschreibung von Blockköpfen, CRCs, unkomprimierten Einträgen und der alten LZ77-Familie. Sie erteilen keine kommerzielle Freigabe für ACE. Verweise der Implementierung auf WinAce und unace liefern keinen unabhängigen Interoperabilitätskorpus. Das Produkt schließt ACE dauerhaft aus. xdzip implementiert und registriert ACE nicht. Suffixe, Erkennung, MIME-Typ und Zuordnungen für ACE bleiben ausgeschlossen.

<a name="xdzip-alz"></a>

## ALZ

- Formatspezifikation `alkegi/docs`, Commit `36230c34d3056604cc63519e9855a79e894496b7`. SHA-256 der CC0-Lizenz `F4E7F373B9B996950337E8D41A4A2939C2D90B7725E9BAF3D5084A22717AD328`.
- SHA-256 von `alz.md` `CBD35E77AB53CFD08C0FF192FA2793438150E4935522608936B1A0942560E237`.
- Das Dokument beschreibt ALZ- und lokale Dateisignaturen, Größen variabler Breite, CRC-32, UTF-8/CP949-Namen, Store, rohes Deflate nach RFC 1951 und den ALZ-spezifisch geänderten BZip2-Datenstrom. Es beschreibt außerdem klassische PKWARE-Verschlüsselung und die Struktur geteilter Archive.

xdzip erlaubt einen eingeschränkten, schreibgeschützten ALZ-Umfang. Gewöhnliche einteilige Archive dürfen unverschlüsseltes Store und rohes Deflate mit genauer Prüfung von CRC, Größe und Abschluss nutzen. Codec, Laufzeit, CLI und Dateizuordnungen werden automatisiert geprüft. Geändertes BZip2, ZipCrypto und geteilte Volumes benötigen unabhängige vollständige Testdateien und gesonderte Ressourcen- und Sicherheitsprüfungen. Die CC0-Dokumentlizenz belegt keine generelle Freiheit von Patenten oder Markenrechten Dritter an Implementierungsdetails.

<a name="xdzip-egg"></a>

## EGG

- Öffentliches Formatdokument `alkegi/docs`, derselbe Commit `36230c34d3056604cc63519e9855a79e894496b7` unter der oben genannten CC0-Lizenz.
- SHA-256 von `egg.md` `4CA2CB6E8FA9A4F40C30EDE05A5E977BE113A96E1527E28A7B1ED0836D8A43A3`.
- Referenzmodul `dterracino/UnEgg`, Commit `f21e653b02f7ab467bbc16af879e68a48dabcb2b`. SHA-256 des EGG-Spezifikations-PDF `0BB7E6F174CAFB9F6726D70B7DEE8DBF33C034717058B1B0EB2B372D9A8FBB37`.
- SHA-256 der UnEGG-Quelllizenz `9B012928756D9243CC549FA0A7D7B5742ED8F69B00A2BE1C964F42EF21F1D8D6` für das Quellpaket und `30F2E45148D43D7A77574403807C13AA359EE9ACE841A00B6A5348AD961423F0` für das Modulpaket.

Die UnEGG-Lizenz verlangt für kommerzielle Nutzung die Zustimmung von ESTsoft, beschränkt die Weitergabe auf nichtkommerzielle Zwecke und verbietet die Entwicklung eines EGG-Kompressors mit dem Modul. ESTsoft behält laut Lizenz alle Rechte. Das Modul dient daher nur als Prüfreferenz. Es darf nicht kopiert, eingebunden oder als Produktabhängigkeit genutzt werden.

Das Dateiformat lässt sich unabhängig vom eingeschränkten Modul implementieren. xdzip unterstützt eine eigene Implementierung von Store und rohem Deflate in einteiligen Archiven ohne Verschlüsselung und Solid-Verarbeitung. Tests für Codec, Laufzeit, CLI und Zuordnungen prüfen exakte Blockgrößen und CRC-32. Standard-BZip2 und die dokumentierten LZMA-Rahmen benötigen eigene vollständige Testdateien und Prüfung. AZO, AES/LEA, ZipCrypto, mehrteilige und Solid-Archive bleiben außerhalb des aktuellen Umfangs.

ALZ und EGG prüfen ihre Container-Header unabhängig und gleichen physische Einträge mit dem jeweiligen Metadaten-Snapshot ab. Nach der Zulassung nutzen beide denselben begrenzten Leser für Store und rohes Deflate. Er prüft deklarierte Länge und CRC-32 und erfasst Eingabe, dekodierte Bytes und Aufwand über alle Einträge hinweg.

<a name="xdzip-required-implementation-gates"></a>

## Voraussetzungen für die Implementierung

1. Referenzen außerhalb des Produktbaums halten und die festgelegten Hashes vor der Verwendung prüfen.
2. Für jede zugelassene Methode unabhängige vollständige Testdateien erstellen. Nur Header oder nur Store belegen keine Methoden-Interoperabilität.
3. Container, Laufzeit, CLI und Desktop gemeinsam ergänzen. Unbekannte Methoden bleiben sichtbar, ohne Befugnis zur Ausgabe von Nutzdaten.
4. Passwörter, Volumes, Solid-Zustand und Spezialobjekte erst nach eigener Quellen-, Sicherheits- und Abbruchprüfung freigeben.
5. Rechtliche Entscheidung und Testdateiidentitäten vor Änderung des öffentlichen Formatregisters im Issue dokumentieren.
