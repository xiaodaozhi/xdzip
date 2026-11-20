<a name="xdzip-host-calendar-and-encoding-services-for-format-workers"></a>
# Kalender- und Kodierungsdienste des Hosts für Formatprozesse

[English](../../engineering/host-platform-services.md) | [简体中文](../../zh-CN/engineering/host-platform-services.md) | [繁體中文](../../zh-TW/engineering/host-platform-services.md) | [日本語](../../ja-JP/engineering/host-platform-services.md) | [한국어](../../ko-KR/engineering/host-platform-services.md) | [Français](../../fr-FR/engineering/host-platform-services.md) | Deutsch | [Español](../../es-ES/engineering/host-platform-services.md) | [Português (Brasil)](../../pt-BR/engineering/host-platform-services.md) | [Русский](../../ru-RU/engineering/host-platform-services.md)

Rust verarbeitet Formatfelder, Passwort-Kompatibilitätsregeln und die Validierung. Historische Zeitzonenregeln des Betriebssystems, OEM- und ANSI-Codepages sowie lokale C-Kodierungen stellt der Host als optionale `ReadSource`-Dienste bereit. Die Formatschicht darf sie nicht aus Dateiname, HostOS oder aktuellem UTC-Versatz ableiten. Die Dienste sind von Qt unabhängig und geben weder Pfadzugriff noch die Berechtigung zur endgültigen Veröffentlichung von Dateien.

<a name="xdzip-calendar-projection"></a>

## Umrechnung in Kalenderwerte

`Timestamp` unterscheidet FILETIME-Werte in 100-ns-Einheiten seit 1601 von vorzeichenlosen ganzen Unix-Sekunden seit 1970. `ReadSource::local_calendar` gibt optional einen `Calendar` zurück. Reguläre Dateien und Speicherquellen verwenden standardmäßig deterministisch UTC. Produktprozesse verwenden lokale Betriebssystem-Zeitzonenregeln über synchrone CXX-Rückrufe. Windows verwendet System-APIs für Zeitzonen, Unix nutzt `localtime_r`. Beide prüfen das Ergebnis. Ist die Umrechnung nicht verfügbar, wird kein Kalender zurückgegeben. Nicht initialisierte Felder dürfen nicht gelesen werden.

Rust validiert Jahr, Monat, Tag, Stunde, Minute und Sekunde, bevor es einen Anzeigewert für DOS kodiert. 7z behält UTC und den Anzeigebereich 1980..2107 bei. RAR wendet seine [festgelegten Kompatibilitätsregeln](rar-format-sources.md) für interne modulare Arithmetik, überschriebene Zeitwerte und Plattformunterschiede an. Anschließend nutzt es den Host-Kalender und den historischen Überlauf des Jahres. Ursprüngliche Genauigkeit, Markierungen für nicht gesetzte Werte und vollständige physische Header verbleiben im privaten Formatindex. Die Anzeige überschreibt sie nicht. Kann ein Zeitwert nicht umgerechnet werden, zeigt RAR nach seiner sicheren Regel 0. Die Referenzimplementierung kann nach einem ungeprüften Betriebssystemfehler undefinierte Daten kopieren.

<a name="xdzip-controlled-encoding-mappings"></a>

## Kontrollierte Zeichenkodierung

| Dienst | Ein- und Ausgabe | Bereich des Hosts |
| --- | --- | --- |
| `oem_to_ansi_wide` | 512 UTF-16-Einheiten zu 512 UTF-16-Einheiten einschließlich NUL | Vollständige Windows-Konvertierung über OEM und ANSI mit Systemzuordnung und tatsächlichen Ersetzungen |
| `wide_to_ansi` | Höchstens 512 UTF-16-Einheiten einschließlich NUL zu 512 Byte | Aktuelle Windows-ANSI-Codepage. Ein universelles Ersatzzeichen wird nicht angenommen |
| `wide_to_local_prefix` | Höchstens 128 breite Skalarwerte einschließlich NUL zu 512 Byte | Aktuelle Unix-C-Kodierung. Bei einem Fehler wird gemäß der festen Kompatibilitätsregel gelöscht und erneut versucht. Der konvertierte Präfix bleibt erhalten |

Die Methoden geben die Länge ohne NUL zurück. Standardimplementierungen ohne Host-Dienste liefern `None`. Die Formatschicht darf nur definierte portable Ausweichverfahren nutzen oder eine Umrechnung ausdrücklich ablehnen. Sie darf keine Codepage erraten. Die Produktrückrufe prüfen Größen, NUL und Rückgabebereiche. Ist eine Zuordnung nicht verfügbar, geben sie die stabile Fehlerkategorie Unsupported zurück.

Der Längenparameter von Windows-[CharToOemBuffW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-chartooembuffw) zählt breite Eingabezeichen und verarbeitet auch Fülleinheiten nach NUL. Er gibt keine Zielkapazität in Byte an. Der Host prüft mit `GetCPInfo(CP_OEMCP)`, ob die Codepage höchstens vier Byte je Einheit benötigt. Für alle 512 Einheiten verwendet er einen temporären Zielpuffer mit 2.048 Byte. Erst dann setzt er Byte 511 auf NUL und konvertiert nach ANSI. Ist die benötigte Kapazität nicht sichergestellt, wird die Konvertierung abgelehnt. 512 breite Zeichen dürfen nicht in einen 512-Byte-Puffer geschrieben oder zur Umgehung der Kapazität vorab gekürzt werden. Systemprüfungen umfassen die gesamte DBCS-Grenze von 511 Einheiten und einen unabhängigen direkten WinAPI-Vergleich.

Die Grenze von 512 Einheiten, das erste NUL, die OEM-Ausgleichsbedingungen, Kürzungen auf 127 Einheiten oder nach Bytes, das Protokoll älterer Markierungen und explizites UTF8 unter macOS sind Rust-Kompatibilitätsregeln. Ein Passwort darf nicht vor einer Betriebssystemumrechnung gekürzt werden, die zuerst stattfinden muss. Eine Kürzung innerhalb einer DBCS-Folge darf nicht willkürlich an eine Zeichengrenze verschoben werden.

<a name="xdzip-ownership-and-failure"></a>

## Eigentum und Fehler

Alle Slices sind nur für synchrone Aufrufe ausgeliehen. C++ speichert keine Zeiger. Rust-Aufrufer bestimmen, wann geheime Ein- und Ausgaben zu löschen sind. Begrenzte breite und schmale C++-Zwischenpuffer werden vor jeder Rückkehr gelöscht. Das garantiert keine Löschung innerhalb des Betriebssystems oder aller älteren Speicherkopien. Die Primitive protokollieren keine Passwörter und übergeben sie weder als Prozessargumente noch als Umgebungsvariablen.

Leser prüfen einen Abbruch vor dem Dienstaufruf. Dienstergebnisse zählen nicht als Extraktionsausgabe und ändern keine Zähler für gelesene, dekodierte oder geschriebene Bytes. Fehler behalten ihre bestehende Kategorie für Quelle, Abbruch oder Kontingent und dürfen nicht als Passwortfehler eingestuft werden. Nur die Formatschicht bestimmt anhand des tatsächlichen Prüferzustands, ob Passwortfehler und Integritätsfehler nicht unterschieden werden können.
