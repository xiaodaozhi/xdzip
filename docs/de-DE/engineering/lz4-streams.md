<a name="xdzip-lz4-stream-contract"></a>

# Vertrag für LZ4-Datenströme

[English](../../engineering/lz4-streams.md) | [简体中文](../../zh-CN/engineering/lz4-streams.md) | [繁體中文](../../zh-TW/engineering/lz4-streams.md) | [日本語](../../ja-JP/engineering/lz4-streams.md) | [한국어](../../ko-KR/engineering/lz4-streams.md) | [Français](../../fr-FR/engineering/lz4-streams.md) | Deutsch | [Español](../../es-ES/engineering/lz4-streams.md) | [Português (Brasil)](../../pt-BR/engineering/lz4-streams.md) | [Русский](../../ru-RU/engineering/lz4-streams.md)

Das Standard-Frame-Profil folgt den Spezifikationen des Autors für das [Frame-Format 1.6.5](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Frame_format.md), das [Blockformat vom 2026-10-26](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Block_format.md) und [XXH32](https://github.com/Cyan4973/xxHash/blob/v0.8.3/doc/xxhash_spec.md). Die Produktalgorithmen sind eigenständige Rust-Implementierungen. Referenzwerkzeuge und erzeugte Prüfnachweise bleiben lokal unter `build/`. Versioniert werden nur eigene reproduzierbare Eingabemuster. Siehe [Einzeldateiströme](single-file-streams.md) für Berechtigungen des Hosts und Veröffentlichung.

<a name="xdzip-admission-and-framing"></a>

## Zulassung und Frames

Bei Standard-Frames der Version 1 werden reservierte Bits, BD-Werte von 4 bis 7, der XXH32-Wert des Headers, die optionale Inhaltsgröße, alle deklarierten XXH32-Werte von Blöcken und Inhalt sowie die Endmarkierung geprüft. Blockdaten dürfen das deklarierte Maximum zwischen 64 KiB und 4 MiB nicht überschreiten. Unabhängige Blöcke setzen den Verlauf zurück. Verbundene Blöcke behalten nur die vorherigen 64 KiB. Rohblöcke, auch solche der Länge null, folgen denselben Prüfungen und Limits. Profile mit externem Wörterbuch und alte Frames werden nicht unterstützt. Eine Wörterbuchdeklaration erlaubt niemals das Lesen einer anderen Datei.

Aneinandergereihte Standard-Frames und überspringbare Frames dazwischen sind durch die physische Eingabe, Aufgabenbudgets, Abbruch und eine Obergrenze von 100.000 Frames oder Blöcken begrenzt. Eine gültige eigenständige Datei enthält mindestens einen Standard-Datenframe. Beliebige Anhänge oder ausschließlich überspringbare Frames führen nicht zum Erfolg. Die Erkennung kann höchstens 8 MiB komprimierte Eingabe prüfen, bevor sie ein Inhaltspräfix von 512 Byte erzeugt. Bei eindeutiger Erkennung oder erkanntem beschädigtem TAR ist bei einem Fehler kein Rückfall auf ein anderes Format möglich. Der vollständige Scan prüft alle Frames, bevor er die ermittelte Größe und den virtuellen Eintrag ausgibt.

<a name="xdzip-block-processing-and-creation"></a>

## Blockverarbeitung und Erstellung

Längen werden mit geprüfter Arithmetik berechnet. Übereinstimmungen benötigen einen von null verschiedenen Rückverweis innerhalb des Verlaufs. Überlappende Bytes werden in aufsteigender Reihenfolge kopiert und bleiben innerhalb der deklarierten Ausgabegrenze. Die letzte Folge enthält nur Literale. Ihr ungenutztes Match-Halbbyte wird ignoriert. Ein Datenstrom mit Verweisen endet mit mindestens fünf Literalen. Der letzte Verweis beginnt mindestens zwölf dekodierte Bytes vor dem Blockende. Die Leseoperation liefert begrenzte Abschnitte und bleibt nach einem Fehler beendet. Der Verlauf wird erst nach Prüfung des exakten Blockendes übernommen. Host-E/A, Abbruch, überschrittene Budgets und nicht unterstützte Profile haben eigene Fehlerkategorien.

Ein zugelassener Encoder muss echte LZ4-Übereinstimmungen ausgeben, Rohblöcke nur ohne Größenersparnis verwenden und Block- sowie Inhaltsprüfsummen schreiben. Jede Stufe begrenzt die Suche nach Übereinstimmungen und muss unabhängig dekodierbaren Inhalt mit mehreren Blöcken erzeugen. Frame- und Suchpuffer werden vor der Zuweisung geprüft. Die Erstellung verwendet exakte eingefrorene Eingabegrößen, temporäre Ausgabe im Host, vollständige Verifikation und abschließende Quellen- und Ausgabeprüfungen. Gewöhnliches LZ4 akzeptiert eine reguläre Eingabedatei. Eine TAR-Kombination darf erst nach Abdeckung durch den gemeinsamen TAR-Parser und -Encoder sowie passende automatische Prüfungen deklariert werden. Eine registrierte Endung ermöglicht bei keinem der Profile Änderungen, Verschlüsselung oder Volumes.

Die Prüfung verwendet `format_declared_checks`, da Frames ohne Inhaltsprüfsummen zulässig sind. Ein Frame ohne Prüfsumme wird nicht als inhaltlich verifiziert ausgewiesen. Ressourcenlimits, tatsächliche Plattformabnahme, Formatgültigkeit und Freigabe zur Veröffentlichung sind getrennte Kriterien.
