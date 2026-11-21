<a name="xdzip-split-volume-reading-and-temporary-opening"></a>

# Lesen von Archivteilen und temporäres Öffnen

[English](../../engineering/archive-opening.md) | [简体中文](../../zh-CN/engineering/archive-opening.md) | [繁體中文](../../zh-TW/engineering/archive-opening.md) | [日本語](../../ja-JP/engineering/archive-opening.md) | [한국어](../../ko-KR/engineering/archive-opening.md) | [Français](../../fr-FR/engineering/archive-opening.md) | Deutsch | [Español](../../es-ES/engineering/archive-opening.md) | [Português (Brasil)](../../pt-BR/engineering/archive-opening.md) | [Русский](../../ru-RU/engineering/archive-opening.md)

Dateiauswahl auf dem Desktop, Anfragen der Dateiverwaltung und die CLI verwenden dieselben Rust-Regeln zum Öffnen von Quellen. Der Host löst zulässige Teilmengen auf, erfasst die Identität der Quelle und prüft die zugelassenen Bytes. Das gewählte Backend liefert [Metadaten zum Durchsuchen](readonly-browser.md), gegebenenfalls über den [isolierten Formatprozess](native-codecs.md). Beim ausdrücklichen Öffnen eines Eintrags entsteht eine geprüfte temporäre Kopie für die zugeordnete Anwendung. Zur Prüfung des gesamten Archivs dient eine gesonderte [Integritätsprüfung](archive-checking.md).

<a name="xdzip-split-zip"></a>

## Geteilte ZIP-Archive

Die Leseschicht unterstützt fortlaufende nummerierte Bytefolgen wie `name.zip.001`, `.002` sowie PKZIP-Teile wie `name.z01`, `.z02` und die abschließende `name.zip`. Jeder Teil kann als Einstieg zum Öffnen dienen. Die Teile müssen im selben Verzeichnis liegen und fortlaufend benannt sein; höchstens 1.024 Teile sind zulässig. Fehlende Teile, Links und Spezialdateien werden abgelehnt. Erweiterungen dienen nur zum Auffinden benachbarter Teile. Der tatsächliche Container wird anhand der Quellbytes erkannt und vom Backend vollständig geprüft.

Hat die klassische ZIP64-Datenträgernummer den Wert 0xFFFF, wird zuerst anhand der begrenzten Gesamtzahl im benachbarten Locator die tatsächliche Teilmenge ermittelt. Der Platzhalter bedeutet nicht, dass 65.535 Teile fehlen. Diese Ermittlung ersetzt nicht die vollständige Prüfung der Enddatensätze und des Verzeichnisses.

Der Rust-Host hält Identität, Länge und Änderungszeit jedes Teils fest. Die logische Eingabe liest bei Bedarf über Teilgrenzen hinweg, ohne das gesamte Archiv zusammenzufügen. PKZIP-Datenträger-relative Offsets werden in eine begrenzte Ansicht des zentralen Verzeichnisses auf einem Teil umgesetzt. Der vorhandene Codec prüft weiterhin Verzeichnis, lokale Header, CRC, Passwortauthentifizierung und Entpackkontingente. Alle Quellteile werden bei jeder Verwendung und vor dem Abschluss erneut geprüft; jede Änderung stoppt die Aufgabe. Die Identitäten aller Quellteile sind vor Überschreiben durch die Ausgabe geschützt.

Die PKZIP-Normalisierung verarbeitet nur zugelassene zentrale Einträge und unterstützt den Enddatensatz der digitalen Signatur des zentralen Verzeichnisses nicht. Diese Ablehnung bedeutet keinen Fehler bei einer Signaturprüfung. Ein gewöhnliches Archiv mit einem Teil kann den Datensatz innerhalb der Grenzen lesen, prüft seine kryptografische Signatur aber nicht und verweigert Änderungen, damit unbekannte Metadaten beim Verschieben weder verloren gehen noch beschädigt werden.

Die Abnahme umfasst nummerierte Teile, PKZIP, ZIP64, teilübergreifende Header und Inhalte, fehlende oder nicht aufeinanderfolgende Teile, Quellaustausch, ungültige Datenträgernummern, chinesische Namen und Regressionen bei gewöhnlichen Archiven mit einem Teil.

<a name="xdzip-7z-and-rar-volumes"></a>

## 7z- und RAR-Teile

Nummerierte 7z-Teile bilden fortlaufende Bytefolgen wie name.7z.001 und .002. RAR verwendet name.partN.rar oder ältere Namen wie name.rar und name.r00. Bei der Übergabe an den isolierten Rust-RAR-Leser bleiben die physischen Grenzen jedes Teils erhalten. Beide Formate finden die Teilmenge im selben Verzeichnis ausgehend von jedem vorhandenen Teil. Die Grenzen liegen bei 1.024 Teilen und 100.000 Einträgen bei der Verzeichnissuche. Fehlende Teile, widersprüchliche Namensnummern, Links und Reparse Points werden ausdrücklich abgelehnt.

Rust hält Identität, Länge und Zeitstempel des Handles jedes Teils fest und meldet die zugelassenen Grenzen über ReadSource::volume_lengths() an den nativen Adapter. Der native Prozess kann den Host nur zum Lesen zugelassener Bereiche auffordern; beim Teilwechsel werden keine beliebigen, vom Archiv gelieferten Hostpfade akzeptiert. Alle Teile werden erneut geprüft und vor Überschreiben durch die Ausgabe geschützt. Eine Änderung am ersten oder einem späteren Teil stoppt die Aufgabe.

Verschlüsselte Dateinamen in 7z und RAR können bereits beim Öffnen ein Passwort erfordern. Liefert Open password_required, wird kein Index veröffentlicht. Qt verwendet das vorhandene Passwortfenster erneut und sendet die ursprünglich gewählte Datei und das begrenzte Geheimnis mit OpenWithPassword; ein falscher Prüfwert ergibt wrong_password. Nach Erfolg halten nur die Sitzung und ihre zugelassenen Aufgaben das Passwort. Beim Schließen gibt die Sitzung ihre Kopie frei; das Passwort wird weder in Einstellungen noch in Wiederherstellungsdaten geschrieben.

<a name="xdzip-cab-volumes"></a>

## CAB-Teile

CAB-Teilnamen stammen aus Headern und können beliebige gültige Dateinamen sein; die Regeln zum Zusammenfügen nummerierter Bytefolgen gelten nicht. Die Referenz auf den nächsten Teil geht jeweils einen Teil weiter. Eine logische Referenz auf den vorherigen Teil kann statt auf den unmittelbar vorigen auf einen früheren Teil zeigen, in dem eine fortgesetzte Datei begonnen hat; vorhandene Beispiele können auch benachbarte Vorgänger referenzieren. Vom gewählten Teil aus folgt Rust den logischen Vorgängern rückwärts und ermittelt anschließend die vollständige Menge im selben Verzeichnis vorwärts. Nach Prüfung von Mengenidentität, lückenloser Reihenfolge und Identitäten werden Länge und ursprünglicher Name jedes Teils festgehalten. Der Decoder prüft alle Dateifragmente; ein nicht benachbarter Vorgänger muss der tatsächliche Anfangsteil einer fortgesetzten Datei sein. Die Grenzen liegen bei 1.024 Teilen und 100.000 Suchtreffern im selben Verzeichnis. Fehlende oder falsche Teile, Zyklen, doppelte Identitäten, absolute Pfade, Verzeichnistrenner, Ausbrüche in übergeordnete Verzeichnisse, Links und Reparse Points werden abgelehnt. Eine geöffnete Teilmenge wird nicht als vollständiges Archiv veröffentlicht.

Rückrufe zum Teilwechsel des Workers dürfen nur Namen aus dieser zugelassenen Menge wählen; jeder Teil liefert seinen eigenen begrenzten Stream. Alle Teile werden vor Aufgaben und Dateiabschlüssen erneut geprüft und vor Überschreiben geschützt. Warnungen des nativen Handlers oder Ergebnisse über fehlende Teile dürfen nicht zu einem erfolgreichen Index werden. CAB-Archive mit einem oder mehreren Teilen sind schreibgeschützt.

<a name="xdzip-double-clicking-regular-files"></a>

## Doppelklick auf gewöhnliche Dateien

Ein Doppelklick auf ein Verzeichnis öffnet es weiterhin. Bei einer gewöhnlichen Datei startet er eine Hintergrundextraktion nur dieses physischen Eintrags in ein zufällig benanntes Verzeichnis im privaten Cache des Benutzers. Links und Spezialobjekte lassen sich auf diesem Weg nicht öffnen. Passwort-, CRC- und Authentifizierungsprüfungen, Abbruch sowie die anwendungsweite serielle Zulassung schwerer Aufgaben gelten weiterhin; ein Ziel muss nicht gewählt werden.

Nach erfolgreicher Ausgabe wird die Systemzuordnung über eine lokale Datei-URL ohne Befehlsinterpreter aufgerufen. Das zugehörige Fortschrittsfenster wird nur automatisch ausgeblendet, wenn das System die Öffnungsanfrage annimmt. Bei Extraktionsfehler, Abbruch oder Ablehnung durch das System bleibt es mit einer lesbaren Fehlermeldung sichtbar. Die Annahme beweist nicht, dass die externe Anwendung das Lesen abgeschlossen hat; die temporäre Datei darf beim Absenden der Anfrage nicht gelöscht werden.

Temporäre Verzeichnisse verwenden unvorhersehbare 128-Bit-Zufallsnamen; Inhalt und Identitätsmarker sind getrennt. Das Ausgabeverzeichnis erlaubt nur dem aktuellen Benutzer und vertrauenswürdigen Systemkonten Zugriff. Unter Windows wird eine geschützte DACL über das gehaltene Handle nur für ein leeres, ausschließlich für diesen Vorgang erstelltes Verzeichnis gesetzt und vor dem Schreiben von Klartext erneut geprüft; gemeinsame Berechtigungen eines vom Benutzer gewählten Verzeichnisses werden nicht geändert. POSIX verwendet 0700 und prüft geerbte macOS-ACLs. Kann die Berechtigung nicht gewährleistet werden, wird die Ausgabe abgelehnt; siehe [Win32-DACL-API](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setsecurityinfo).

Solange eine Sperre gehalten wird, erfolgt keine Bereinigung. Abgeschlossene Ausgaben bleiben mindestens sieben Tage erhalten; abgelaufene, nicht gesperrte Verzeichnisse werden bei einem späteren temporären Öffnen bereinigt. Der Cache hält höchstens 256 Einträge und lehnt am Limit ausdrücklich ab; jede Suche ist auf 4.096 Einträge begrenzt. Die Bereinigung prüft private Berechtigungen, Marker und Namen und folgt keinen Links. Änderungen an der temporären Kopie werden nicht ins Archiv zurückgeschrieben.

Die Bereinigung beschränkt sich auf den von der Anwendung erstellten privaten Cache; sie durchsucht oder löscht keine Entpackverzeichnisse des Benutzers. Die Abnahme umfasst automatisches Ausblenden bei Erfolg, sichtbare Fenster bei Fehlern, Abbruch, erneute Passworteingabe, doppelte Dateinamen, Unicode und Sonderzeichen, Aufgabenzuordnung nach Schließen der Seite, Sicherheit temporärer Verzeichnisse und die Bereinigungsregel beim Beenden.

Die schreibgeschützte Zulassung von [Brotli/TAR.BR](brotli-streams.md) prüft Kandidaten ohne Header vor der Veröffentlichung von Zeilen vollständig, teilt das Lesekonto für Brotli/LZMA-Kandidaten, erhält den Vorrang starker Quellen und gültiger TAR/ZIP-Erkennung und stuft beschädigtes decodiertes TAR nie zu einem gewöhnlichen Stream herab.

[LHA/LZH](lha-containers.md)-Vorschläge erfordern vor Veröffentlichung die vollständige begrenzte Prüfung von Headern und Namen, auch bei einem leeren Archiv aus einem einzelnen Nullbyte. Schwache LHA-, Brotli- und LZMA-Vorschläge teilen ein Konto tatsächlich gelesener Daten. Starke gültige Container behalten Vorrang; eine vollständige LHA-Zulassung kann einen ungültigen rohen TAR-Marker eindeutig zuordnen. Passwort-, Teilarchiv- und Reparaturprofile sind nicht zugelassen.

[PMA](pma-streams.md) nutzt dasselbe gemeinsame Konto schwacher Prüfungen und die vollständige Metadatenzulassung mit exakt einem Nullterminator und optionaler begrenzter CP/M-Füllung. PM0/PM1/PM2-Inhalte der Stufe 0 werden unterstützt; unbekannte Methoden bleiben sichtbar, aber nicht unterstützt. Ein gefüllter leerer CP/M-Container erkennt PMA; ein nicht unterscheidbarer Ein-Byte-Nullterminator behält die frühere LZH-Auswahl bei. Erstellen, Passwort, Teile, Ändern und Reparieren sind nicht zugelassen.
