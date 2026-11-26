# XDZip in Skripten verwenden

[English](../../integration/README.md) | [简体中文](../../zh-CN/integration/README.md) | [繁體中文](../../zh-TW/integration/README.md) | [日本語](../../ja-JP/integration/README.md) | [한국어](../../ko-KR/integration/README.md) | [Français](../../fr-FR/integration/README.md) | Deutsch | [Español](../../es-ES/integration/README.md) | [Português (Brasil)](../../pt-BR/integration/README.md) | [Русский](../../ru-RU/integration/README.md)

Die eigenständige CLI läuft ohne Qt und ohne Desktop-Sitzung. Wählen Sie bei den [GitHub-Veröffentlichungen](https://github.com/xiaodaozhi/xdzip/releases) ein Paket mit dem Präfix `xdzip-cli-` für Ihr System. Entpacken Sie das vollständige Paket und lassen Sie das Arbeitsprogramm neben der ausführbaren Datei. Führen Sie im entpackten Ordner unter macOS oder Linux `./bin/xdzip --help` aus, in PowerShell `.\bin\xdzip.exe --help`.

Die Beispiele setzen `xdzip` in PATH voraus. Führen Sie mit der entpackten CLI `setup` aus, um einen Starter für Ihren Benutzer zu registrieren. Fügen Sie sein Befehlsverzeichnis bei Bedarf zu PATH hinzu. Mit `xdzip --version-json` prüfen Sie, ob der gefundene Befehl `"component":"cli"` meldet. Desktop-Starter und CLI können denselben Befehlsnamen haben. Verwenden Sie bei einem Konflikt den vollständigen Pfad zur entpackten CLI. [Befehl installieren oder entfernen](../engineering/command-line.md#xdzip-build-and-command-installation)

<a name="xdzip-create-and-check"></a>

## Erstellen und prüfen

Legen Sie in einem neuen, privaten, beschreibbaren Arbeitsverzeichnis einen Ordner `docs` mit `hello.txt` an. Legen Sie die zweite Datei `notes.txt` neben diesem Ordner ab. Die folgenden Befehle erstellen ein ZIP, fügen die zweite Datei im vorhandenen Archivordner `docs/` hinzu und prüfen das Ergebnis.

```sh
xdzip create sample.zip docs --level normal --json
xdzip add sample.zip notes.txt --into docs --json
xdzip test sample.zip --json
```

Wählen Sie die Dateiendung und Optionen aus der [Formattabelle](../../../README.de-DE.md#xdzip-format-support). Legen Sie die Ausgabe außerhalb des Eingabebaums ab und verwenden Sie einen vorhandenen übergeordneten Ausgabeordner. `add` benötigt ein vorhandenes, geeignetes Archiv. Mit `--into` wählen Sie einen vorhandenen Archivordner. Ohne diese Option fügt der Befehl im Stammordner hinzu. `test` schreibt keine entpackten Dateien. `data.check_kind` gibt bei Erfolg die Stärke der Prüfung an. Siehe [Formatregister](../engineering/format-registry.md) und [Integritätsprüfungen](../engineering/archive-checking.md).

<a name="xdzip-list-select-and-extract"></a>

## Auflisten, auswählen und entpacken

Verwenden Sie das obige Archiv im selben Arbeitsverzeichnis. Verwenden Sie einen neuen Zielordner oder wählen Sie ausdrücklich, wie vorhandene Dateien behandelt werden.

```sh
xdzip list sample.zip --limit 2 --json
xdzip extract sample.zip docs/hello.txt -C output --conflict skip --json
```

Pfade müssen genau mit den Einträgen übereinstimmen, unterscheiden Groß- und Kleinschreibung und verwenden Schrägstriche. Gibt es mehrere Einträge mit demselben Pfad, wählen Sie sie anhand der von `list` gelieferten `id`. IDs beginnen bei 1. Ersetzen Sie zum Beispiel im zweiten Befehl `docs/hello.txt` durch `--entry-id 2`, wenn die Liste für diese Datei ID 2 angibt. Für mehrere Einträge wiederholen Sie `--entry-id`. Verwenden Sie `path` zum Abgleich und `display_name` zur Anzeige. Nach Änderungen am Archiv rufen Sie die Liste erneut ab.

Die erste Seite beginnt bei Offset 0. Übergeben Sie `data.next_offset` als `--offset` mit unverändertem `--limit`, um die nächste Seite zu lesen. Ist `next_offset` null, ist die Liste vollständig. Der Offset ist die nullbasierte Zeilenposition. Eine ID kennzeichnet einen physischen Datensatz. Jeder Befehl öffnet das Archiv erneut. Lassen Sie die Quelle zwischen den Seitenabrufen unverändert. [Auswahl und Grenzen beim Auflisten](../engineering/command-line.md#xdzip-archive-behavior)

<a name="xdzip-run-without-prompts"></a>

## Ohne Rückfragen ausführen

`--json` deaktiviert interaktive Rückfragen. Wenn ein Auftrag unbeaufsichtigt fertig werden soll, geben Sie die Konfliktregel ausdrücklich an. Bei Befehlen mit Passwortunterstützung wie `extract`, `test` und verschlüsseltem `create` verwenden Sie `--password-stdin` und senden eine UTF-8-Passwortzeile über eine private Standardeingabe-Pipe. Zum verschlüsselten Erstellen ist zusätzlich `--encrypt` erforderlich. Fehlt eine notwendige Entscheidung oder ein Passwort, meldet die CLI `interaction_required`. [Passwort- und Formatoptionen](../engineering/command-line.md#xdzip-archive-behavior)

<a name="xdzip-interpret-results"></a>

## Ergebnisse auswerten

Lesen Sie bei Archivvorgängen mit `--json` das UTF-8-Objekt auf stdout aus. Prüfen Sie `schema_version`, `status`, `error_code`, `data`, `summary` und `warnings`. Halten Sie stderr für Diagnosen verfügbar. `--version-json` gibt ein eigenes Objekt mit der Build-Identität zurück.

| Rückgabecode | Auswertung |
| --- | --- |
| 0 | Lesen Sie Ergebnis und Anzahl übersprungener Dateien. Der Status `handed_off` bestätigt bei einem entsprechenden Befehl die Übergabe an die Desktopanwendung. |
| 1 | Prüfen Sie Fehler, Warnungen und Zusammenfassung auf Fehlschlag, Teilerfolg oder Warnungen zur Bereinigung. |
| 2 | Korrigieren Sie den Befehl oder die Argumente. |
| 3 | Prüfen Sie die gemeldete Anforderung an Interaktion, Autorisierung, Plattform oder Installation. |
| 130 | Der Abbruch wurde bestätigt. Prüfen Sie, welche Dateien bereits endgültig gespeichert wurden. |

Unbekannte Werte können null sein. Bei Fehlern oder Abbruch können zuvor endgültig gespeicherte Dateien erhalten bleiben. Prüfen Sie vor einem erneuten Versuch `summary.committed_files`, wenn das Feld vorhanden ist. Die [CLI-Referenz](../engineering/command-line.md) beschreibt die Übergabe an die Desktopanwendung, Updatebelege und das vollständige Ausgabeformat.
