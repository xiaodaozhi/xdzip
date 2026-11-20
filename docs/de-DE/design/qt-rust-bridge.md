<a name="xdzip-qt-rust-bridge-and-application-service"></a>

# Qt/Rust-Brücke und Anwendungsdienst

[English](../../design/qt-rust-bridge.md) | [简体中文](../../zh-CN/design/qt-rust-bridge.md) | [繁體中文](../../zh-TW/design/qt-rust-bridge.md) | [日本語](../../ja-JP/design/qt-rust-bridge.md) | [한국어](../../ko-KR/design/qt-rust-bridge.md) | [Français](../../fr-FR/design/qt-rust-bridge.md) | Deutsch | [Español](../../es-ES/design/qt-rust-bridge.md) | [Português (Brasil)](../../pt-BR/design/qt-rust-bridge.md) | [Русский](../../ru-RU/design/qt-rust-bridge.md)

Der Qt-Desktop ruft die Rust-Laufzeit über die eingebundene `cxx`-Brücke auf. Jeder `BrowserController` besitzt eine `RuntimeConnection` mit eigener Dienstinstanz. Die Brücke überträgt typisierte Befehle und Ergebnisse mit übertragener Eigentümerschaft innerhalb des Desktopprozesses. Die CLI verwendet denselben Rust-Dienst direkt. [Systemarchitektur](../../design/system-architecture.md), [Desktop-Sitzungen](../../engineering/application-sessions.md)

<a name="xdzip-handles-threads-and-ownership"></a>

## Handles, Threads und Eigentümerschaft

`new_runtime()` startet die Rust-Befehls- und Aufgabenthreads. Die temporäre `Runtime` gibt je einen `CommandClient` und einen `EventReceiver` aus. Der GUI-Thread verwendet das Befehlshandle. Ein eigener Qt-Empfangsthread besitzt das Ereignishandle und blockiert in `next_event()`. Der Rust-Befehlsthread besitzt das aktuelle Archiv und seinen Index. Der separate Aufgabenthread besitzt die vorbereitete Arbeit während ihrer Ausführung.

CXX überträgt Werttypen und besitzende Rust-Container. Der empfangende Qt-Code hält oder konvertiert diese Werte, solange er sie besitzt. Kein Archiv- oder natives Dateihandle gelangt nach Qt. Für IDs oder genaue Bytezahlen, deren volle 64 Bit erhalten bleiben müssen, verwendet der Controller gegenüber QML Zeichenfolgen. Generierte Header und statische Bibliothek werden zusammen mit dem Host gebaut. Diese Schnittstelle gehört zum Produktbuild. Der mitgelieferte Worker verwendet ein getrenntes [Prozessprotokoll](../engineering/native-codecs.md).

[Brückendeklarationen und Konvertierungen](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-qt-bridge/src/lib.rs), [Qt-Verbindung](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/RuntimeConnection.cpp), [Rust-Dienst](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/service.rs)

<a name="xdzip-request-query-and-task-identities"></a>

## Identitäten von Anfragen, Abfragen und Aufgaben

| Identität | Zweck |
| --- | --- |
| Anfrage-ID | Der Rust-Dienst vergibt bei Zulassung eine verbindungsbezogene ID. `Submission` gibt `Accepted`, `Busy`, `Closed`, `Invalid` oder `Exhausted` zurück. Ablehnungen liefern ID null. |
| Sitzungs-ID | Kennzeichnet den Snapshot des geöffneten Archivs. Befehle für dieses Archiv müssen dazu passen. Neue Öffnungen erhalten eine neue Sitzungs-ID. |
| Abfrageidentität und Offset | Eine Seite ist an Sitzung, Abfrage-ID, GUI-Generation, Revision und Offset gebunden. Der Controller prüft sie vor Annahme der Zeilen. |
| Physische Eintrags- oder Navigations-ID | Ein physischer Eintrag behält seine Identität beim Sortieren und Blättern. Synthetisierte Verzeichnisse haben eigene Identitäten. Zeilenindex, Anzeigename oder Abfragebereich berechtigen nie zum Zugriff auf einen Dateisystempfad. |
| Aufgaben- und Prompt-ID | Eine Aufgaben-ID gehört zu ihrer Verbindung. Ein Prompt kennzeichnet eine ausstehende Konflikt- oder Passwortantwort in dieser Aufgabe. Der Anwendungseintrag der Aufgabe kennt auch den zuständigen Controller. |
| Ereignissequenz | Der Empfänger vergibt beim Entnehmen monoton steigende Sequenznummern pro Verbindung. Qt verwirft Ereignisse, deren Sequenz nicht neuer als die zuletzt verarbeitete ist. Aufgabenaktualisierungen teilen diese Sequenz und verwenden zur Zuordnung die ursprüngliche Anfrage-ID. |

<a name="xdzip-commands-in-the-current-bridge"></a>

## Befehle der aktuellen Brücke

| Befehl oder Methode | Zweck und gewöhnliches Ergebnis |
| --- | --- |
| `Open`, `OpenWithPassword` | Gewählte Quelle öffnen und scannen. Eine erfolgreiche Antwort enthält die erste `Page` und begrenzte Navigationsdaten. Ein verschlüsseltes Verzeichnis kann vor dem Öffnen einer Sitzung ein Passwort erfordern. |
| `Browse` | Abfrage starten oder mit passender Identität und passendem Offset fortsetzen. Jeder Befehl liefert höchstens eine `Page` mit höchstens 128 Zeilen und 64 KiB Namen. |
| `FreezeSelection`, `Preflight` | Aktuelle Selektoren in besitzende physische oder Navigations-IDs in `Selected` auflösen oder Metadatenbefunde in `Preflight` zusammenfassen. Snapshot und gewählter Umfang werden für den tatsächlichen Vorgang erneut geprüft. |
| `Preview` | Einen physischen Eintrag auflösen. Bei Erfolg besitzende, begrenzte `Preview`-Bytes samt Eintrags- und Sitzungsidentität zurückgeben. |
| `CheckTarget`, `InspectInputs` | Nach Öffnen und Bereinigen eines Prüfarbeitsbereichs durch die Laufzeit die Zahl verbleibender Elemente in `TargetChecked` zurückgeben. Die Erstellungsprüfung verwendet Sitzung null. `Inputs` meldet oberflächliche Metadaten in Eingabereihenfolge ohne rekursives Durchsuchen von Verzeichnissen. Keine Antwort berechtigt zur Übernahme. |
| `Extract`, `Create`, `Check`, `OpenEntry` | Absicht prüfen und Aufgabe einreihen. `TaskAccepted` enthält die vergebene Aufgaben-ID. Zielautorisierung und Übernahme erfolgen durch Laufzeit und E/A-Schicht. |
| `Update`, `Move`, `Export`, `PublishExport`, `ExportManifest` | Archivänderungen und Datenveröffentlichung gelangen in die Warteschlange schwerer Aufgaben. Nach Zulassung von `Update` oder `Move` verwirft der Dienst den aktuellen Archiv-Snapshot, damit spätere Anfragen alte Eintrags-IDs nicht nutzen. `ExportManifest` liefert relative Übertragungsmetadaten als gewöhnliche Antwort. |
| `Snapshot`, `CloseSession` | Aktuelle Archivsitzungs-ID oder null zurückgeben. Das Schließen der angegebenen Sitzung verwirft ihren Navigations-Snapshot; unabhängig gehaltene Aufgabeneingaben können bestehen bleiben. |
| `cancel`, `cancel_task`, `decide_task`, `respond_password`, `release_export`, `shutdown` | Steuermethoden außerhalb der gewöhnlichen Befehlswarteschlange. Der Abbruch einer Anfrage hat eine eigene Anfrage-ID und `Finished`; Aufgabensteuerungen wirken auf die bestehende Aufgabe. |

Der Rust-Dienst bietet weitere CLI-spezifische Vorgänge wie `Catalog` und `SelectMembers`. Die Brücke prüft Eingabegröße und Enum-Werte vor der Einreichung. Erstellungspfade sind beispielsweise jeweils auf 4 KiB und insgesamt auf 4 MiB begrenzt. Die Laufzeit prüft Sitzung und Vorgangssemantik; die E/A-Schicht öffnet Dateien und beschafft Ausgabeberechtigungen. [Vertrag des Laufzeitdienstes](../engineering/product-runtime.md), [CLI-Vertrag](../../engineering/command-line.md), [Dateiautorisierung](file-security.md)

<a name="xdzip-event-delivery-and-backpressure"></a>

## Ereigniszustellung und Rückstau

Eine zugelassene gewöhnliche Anfrage erzeugt bei normaler Zustellung höchstens ein Datenergebnis und danach ein `Finished`. Ein Fehler wird durch ein `Finished` mit Fehlercode dargestellt. Der Controller wartet auf erfolgreichen Abschluss, bevor er die zugehörige Seite oder Vorschau veröffentlicht. Ein überholtes Ergebnis wird verworfen, während seine Anfrage dennoch endet. Der Pfad für große Indizes hält bis zu 1.024 Zeilen im Qt-Cache und fordert fehlende Seiten an. Eine seitenübergreifende Auswahl wird vor der Entpackbestätigung eingefroren. [Indizierung großer Archive](../engineering/large-index.md), [Inhaltsvorschau](../engineering/content-preview.md)

`TaskAccepted` und `Finished` gehören zur gewöhnlichen Anfragezustellung. `TaskUpdate` verwendet getrennte ausstehende Plätze pro Aufgabe und enthält ursprüngliche Anfrage-ID, Aufgaben-ID und Quellsitzungs-ID. Eine Aktualisierung kann vor `TaskAccepted` oder nach `Finished` eintreffen. Der Controller ordnet eine frühe Aufgabe der ursprünglichen Anfrage zu und leitet spätere Aktualisierungen an diese Aufgabe. Eine spätere Aufgabenaktualisierung schließt die gewöhnliche Anfrage nie erneut ab. Die Plätze behalten den neuesten Snapshot einer Aufgabe. Fortschritt kann zusammengefasst werden; der End-Snapshot bleibt bestehen, bis der Empfänger ihn entnimmt. Prompts erscheinen in Snapshots, solange die Aufgabe auf ihre Antwort wartet. [Aufgabenstatus und Abbruch](task-scheduling.md)

- **Rust.** Pro Verbindung erlaubt die gewöhnliche Zulassung 128 ausstehende Anfragen und ein Bytebudget von 8 MiB für eingereihte Befehle. Das Nutzlastbudget gewöhnlicher Ereignisse beträgt 32 MiB. Zugelassene Anfragen reservieren Platz für Ergebnis und Abschluss. Bestätigungen von Anfrageabbrüchen haben eine eigene begrenzte Warteschlange; das Herunterfahren benötigt darin keinen Platz.
- **Qt.** Die Mailbox hält höchstens 256 Ereignisse, davon höchstens 128 Nutzlastereignisse, und 32 MiB berechnete Nutzlast. Bei voller Kapazität wartet der Empfänger darauf, dass die GUI sie leert. Eine einzige eingereihte Benachrichtigungskette verarbeitet bis zu 32 Ereignisse oder 4 ms pro Durchlauf. Die Benachrichtigung erreicht den GUI-Thread; eingereihte Ereignisse behalten ihre eigenen Daten.

<a name="xdzip-prompts-failures-and-shutdown"></a>

## Prompts, Fehler und Herunterfahren

- Konfliktentscheidungen verwenden `decide_task`. Aufgabenpasswörter verwenden `respond_password` und eine eigene Passwort-Prompt-ID. Beide prüfen den aktuellen unbeantworteten Prompt. Die Passwortantwort gelangt in beim Freigeben genullten Rust-Speicher und erscheint nie in einem Ergebnis oder Fortschrittsereignis. `OpenWithPassword`, `Preview` und verschlüsselte Erstellung tragen vorgangsspezifische Eingabegeheimnisse, die die Brücke in die jeweiligen Rust-Passworttypen verschiebt. [Passwortbehandlung](../engineering/password-extraction.md)
- Stabile Fehlercodes beschreiben Zulassung sowie Anfrage- und Aufgabenergebnisse. Beispiele sind `stale_session`, `stale_query`, `source_changed`, `unsafe_target`, `target_changed`, `disk_full` und `password_required`. Der Controller übersetzt Codes für die Anzeige. Eine Quelländerung macht den betroffenen Navigations-Snapshot ungültig. `RuntimeFault` und unerwartete Verbindungsabbrüche machen ausstehende GUI-Arbeit mit `runtime_unavailable` ungültig; ein Verbindungsverlust kann die Zustellung ursprünglicher Endereignisse verhindern.
- `shutdown` schließt die Zulassung und bricht angenommene Arbeit einschließlich auf Eingabe wartender Aufgaben ab. Der Qt-Empfänger leert weiter Ereignisse, während die Rust-Threads enden. Der Rust-Empfänger wartet auf sie, bevor er `ShutdownReady` ausgibt; danach gibt die Brücke `End` aus. Qt meldet nach Leeren der Mailbox eine saubere Trennung. Wird die Verbindung früh zerstört, schließt sie die Mailbox und weckt den Empfänger, ohne im GUI-Thread auf ihn zu warten. [Aufgabenende](task-scheduling.md), [Lebenszyklus der Laufzeit](../engineering/product-runtime.md)

Bei Änderungen an dieser Grenze prüfen die [Brückensonde](https://github.com/xiaodaozhi/xdzip/blob/main/tools/bridge-probe/README.md), Rust-Diensttests und [Desktop-Controllertests](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/tests/browser_test.cpp) verschiedene Ebenen. Die [Build-Anleitung](../../engineering/developer-build.md) nennt die relevanten Ziele.
