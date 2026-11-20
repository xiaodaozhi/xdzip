<a name="xdzip-external-read-only-sdk-trial"></a>

# Externer schreibgeschützter SDK-Versuch

[English](../../engineering/sdk-trial.md) | [简体中文](../../zh-CN/engineering/sdk-trial.md) | [繁體中文（台灣）](../../zh-TW/engineering/sdk-trial.md) | [日本語](../../ja-JP/engineering/sdk-trial.md) | [한국어](../../ko-KR/engineering/sdk-trial.md) | [Français](../../fr-FR/engineering/sdk-trial.md) | Deutsch | [Español](../../es-ES/engineering/sdk-trial.md) | [Português (Brasil)](../../pt-BR/engineering/sdk-trial.md) | [Русский](../../ru-RU/engineering/sdk-trial.md)

`xdzip-plugin-protocol` und `xdzip-plugin-host` implementieren den unabhängigen schreibgeschützten Versuch aus dem [Format-Erweiterungsdesign](../design/codec-extension-sdk.md). Er ist von Formatentdeckung in Desktop und CLI, Installation, Paketierung und automatischem Start ausgeschlossen. Benutzer führen den Versuchsbefehl ausdrücklich aus und wählen Paket und Quelle; Drittanbietererweiterungen bleiben im Produkt deaktiviert. Es wird nur `xdp1` ausgehandelt, ohne Erstellung, Passwörter, Teilarchive, Links, GUI oder beliebige Hostpfadberechtigungen. Dies ist weder ein vollständiges Plugin-Ökosystem noch eine SDK-Veröffentlichung.

<a name="xdzip-reproducible-entry-points"></a>

## Reproduzierbare Einstiegspunkte

```sh
python3 scripts/sdk-trial.py test
python3 scripts/sdk-trial.py example --output-dir build/sdk-example
target/debug/xdzip-plugin-host scan build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp
mkdir build/sdk-example/output
target/debug/xdzip-plugin-host read build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp 0 build/sdk-example/output
```

Unter Windows `python` und `.exe` verwenden. `test` baut das native Referenzprogramm und führt Rust-Framingregressionen, unabhängige Python-Interoperabilität sowie Regressionen zu bösartigem Protokoll, Abbruch, Quelländerung, Konflikten und Kernelberechtigungen aus. Python wird nur vom vertrauenswürdigen, für Tests kompilierten Konformitätseinstieg gestartet; der ausgelieferte Host akzeptiert weder Interpreter, Argumente, Shell noch Schalter für Ausführung ohne Sandbox. Das Beispiel erzeugt ein eigenes Paket, Manifest und XDP1 nur unter `build/`; es löst weder Veröffentlichung noch Drittanbieterdownload aus. Logs lassen Arbeitsbereichs- und Benutzerverzeichnispfade weg; JSON bindet Commit, Änderungsstatus und Binärhashes. Beides sind lokale oder CI-Artefakte. Dieselbe native Foundation-Matrix ruft diesen Einstieg auf; der separate [SDK-Versuchsworkflow](../../../.github/workflows/sdk-trial.yml) klärt zuerst die Plattformkompatibilität der Sandbox, ohne andere Format- und Qt-Pipelines zu wiederholen. Der Foundation-Workflow kann den wiederverwendbaren Versuch auch mit `sdk_only=true` aufrufen; dies überspringt die vollständige Foundation-Prüfung und belegt kein vollständiges Foundation-Ergebnis. Windows prüft die Ausführungsverweigerung.

<a name="xdzip-package-admission"></a>

## Paketzulassung

Das strikte JSON-Manifest verwendet `deny_unknown_fields`. Felder sind `manifest_version=1`, `plugin_id`, `version`, `author`, `protocol_majors=[1]`, `targets`, ein paketinterner `entrypoint`, kleingeschriebenes hexadezimales `sha256`, `formats=["xdp1"]`, `permissions`, `resources`, `license_files` und eine HTTPS-`source_url`. Das oben durch `example` erzeugte Referenzmanifest ist ein ausführbarer Einstieg zum aktuellen Schema.

`permissions` sind genau `broker_source_read=true`, `broker_output_write=true`, `network=false`; `resources` genau `memory_bytes=536870912`, `threads=1`. Das Manifest ist höchstens 32 KiB, das native Programm höchstens 64 MiB groß; höchstens vier Lizenzdateien mit je höchstens 128 KiB sind erlaubt. Einstiegspunkte und Lizenzdateien dürfen weder absolut sein noch Elternverzeichnisdurchläufe enthalten oder über Links beziehungsweise Reparse-Punkte ausbrechen. Jedes Paketmitglied wird über ein gehaltenes cap-std-Verzeichnis komponentenweise ohne Linkverfolgung geöffnet und nach Prüfung nicht über seine ursprüngliche Zeichenfolge erneut geöffnet. Der Host kopiert aus der gehaltenen regulären Datei, prüft tatsächliche Bytezahl, SHA-256 und Dateiversion vor und nach dem Kopieren und führt nur die private temporäre Kopie aus, die bis zum Ende von Prozess und IPC erhalten bleibt. Der deklarierte Hash belegt die Konsistenz des gewählten Paketinhalts, authentifiziert aber weder Herausgeber noch Codesicherheit.

<a name="xdzip-wire-protocol-and-state"></a>

## Datenprotokoll und Zustand

Eine eigene begrenzte CBOR-Teilmenge folgt [RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html) ohne CBOR-Implementierung Dritter. Der Rust-Referenzpartner verwendet das Framing-Crate erneut; der [Python-Partner](../../../examples/sdk/reference.py) implementiert Kodierung, Dekodierung und XDP1-Parsing unabhängig, ohne Rust oder generierten Protokollcode zu importieren. Interoperabilität umfasst mehrere Blöcke tatsächlicher Inhalte und den Host-Digest.

Frames sind `u32 LE length + CBOR`; die Länge wird vor der Allokation geprüft. Frames sind höchstens 1 MiB, einzelne bytes/text höchstens 256 KiB, Tiefe höchstens 16, Arrays höchstens 128, Maps höchstens 32 und dekodierte Knoten höchstens 8192. Nur UInt, bytes, strenger UTF-8-Text, Arrays, Maps mit UInt-Schlüsseln und bool sind zulässig. Nicht kürzeste Ganzzahlen, doppelte Schlüssel, unbestimmte Längen, negative Ganzzahlen, Tags, Gleitkommazahlen und nachfolgende Daten werden abgelehnt. Die Envelope-Map-Schlüssel `1..5` sind kind, request, operation, sequence und payload. Sequence beginnt bei 1 und steigt je Richtung fortlaufend; request steigt je Aufruf und operation ist eine zufällige, von null verschiedene Hostidentität. Jede Antwort muss exakt zur aktuellen Identität passen. Unbekannte Payload-Schlüssel `<128` werden abgelehnt; `>=128` sind ignorierbare optionale Felder.

| kind | Nachricht | Numerische Payload-Schlüssel |
| --- | --- | --- |
| 1 | Hello | 1 minimales Major, 2 maximales Major, 3 Paket-ID, 4 `xdp1` |
| 2 | Welcome | 1 Major, 2 Blocklimit, 3 Einzelblock-Kredit, 4 Gesamtlesebudget, 5 Ausgabebudget |
| 10 | Probe | 1 Quelltoken, 2 Quelllänge |
| 11 | Open | 1 Quelltoken |
| 12 | Scan | 1 Sitzungstoken |
| 13 | ReadEntry | 1 Sitzung, 2 Eintrag, 3 Senke, 4 Kreditzahl 1, 5 Ausgabebudget, 6 Kredittoken des ersten Blocks |
| 14 | Close | 1 Sitzung |
| 20, 21 | ReadAt, ReadData | 1 Quelltoken, 2 Offset, 3 Länge oder Bytes |
| 30 | EntryBatch | 1 Sitzung, 2 Einträge; je Datensatz: 1 Token, 2 Rohpfad, 3 Typ 0 Datei / 1 Verzeichnis, 4 bekannte Größe |
| 31 | ScanFinished | 1 tatsächliche Datensatzzahl |
| 40 | WriteData | 1 Senke, 2 Blocksequenz ab 0, 3 Bytes, 4 aktuelles Kredittoken |
| 41 | FinishOutput | 1 Senke, 2 deklarierte Ausgabeanzahl, 3 SHA-256-Bytes |
| 42 | WriteAck | 1 Senke, 2 nächste Blocksequenz, 3 neues zufälliges Kredittoken |
| 50, 51, 52 | Cancel, CancelAck, Shutdown | Leere Map |
| 61 | OperationFinished | 1 Ergebnis, 0 Erfolg / 1 abgebrochen; Probe zusätzlich 2 Format, Open zusätzlich 2 Sitzung |

Andere kinds lehnt die Zustandsmaschine dieses Versuchs ab, auch wenn das Framing sie erkennt. Hello muss zuerst eintreffen und beide Seiten wählen Major 1; die Handshake-Identität ist null, spätere Aufrufe sind ungleich null. Auf ScanFinished muss weiterhin ein eindeutiger erfolgreicher Endzustand folgen. Nur eine Operation läuft gleichzeitig und Erweiterungen werden nicht im Hintergrund erneut versucht. Der aktuelle Einzelblock-Kredit ist strenger als das Designmaximum von vier; nach tatsächlicher Annahme jedes Ausgabeblocks wird ein unvorhersehbares Kredittoken erneuert und kann weder vorab bezahlt noch wiederholt werden.

<a name="xdzip-broker-budgets-and-publication"></a>

## Broker, Budgets und Veröffentlichung

Quelltoken, Senken und Kredite pro Block sind zufällige 128-Bit-Werte und dürfen zwischen Operationen nicht wiederverwendet werden. Der Partner erhält weder Archiv- noch Zielpfade. Der Host gibt pro Lesevorgang höchstens 256 KiB innerhalb des autorisierten Quellbereichs zurück; Offsetüberlauf, Lesen hinter EOF, Länge null und kumulierte Lesevorgänge über 64 MiB schlagen fehl. Er prüft `FileVersion` des gehaltenen Objekts und des zugehörigen Pfads vor und nach jedem Lesen und vor endgültiger Veröffentlichung. Dies verwendet den vorhandenen Identitäts-/Zeit-/Längenfingerabdruck und behauptet nicht, beliebige gleichzeitige Änderungen durch einen Angreifer desselben Kontos zu erkennen, der alle Metadaten wiederherstellt.

Metadaten sind auf 1000 physische Datensätze und insgesamt höchstens 256 KiB für Pfade und Token begrenzt; Datensatztoken sind eindeutig. Der Host prüft Rohpfade streng gegen PortablePath; doppelte Namen behalten getrennte Identitäten. Jeweils ein regulärer Eintrag wird ausdrücklich ausgewählt, mit deklarierter Größe höchstens 4 MiB; Verzeichnisse können keine Dateiausgabe sein. Der Host hält Ziel-Grant, privaten Workspace und Stage, misst tatsächliche Bytes und berechnet SHA-256, ohne dem Partner einen Dateihandle oder Veröffentlichungsautorität zu geben.

Bestehende sichere E/A darf nur ohne Überschreiben veröffentlichen, nachdem alle Ausgabeblöcke, FinishOutput-Anzahl und Digest, eindeutiger Endzustand, Close, normaler Exit nach Shutdown und sauberes EOF, Quellidentität und abschließende Abbruchprüfung erfolgreich sind. Bei Anomalie, doppeltem Endzustand, Absturz oder Timeout wird zuerst die Ausgabe widerrufen; die Zerstörung entfernt Stage vor dem Aufräumen des Workspace. Vorhandene Ziele bleiben unverändert. Parsing und erwartete Digests stammen weiterhin von der Erweiterung; die Hostberechnung kann nicht beweisen, dass die Ausgabe einer bösartigen Erweiterung dem Archiv entspricht.

Handshake ist auf 2 Sekunden und jeder Versuchsaufruf insgesamt auf 5 Sekunden begrenzt, ohne dass Fortschritt die Frist zurücksetzt. CPU je Prozess ist auf 3 Sekunden, Dateideskriptoren auf 32 begrenzt, direkte Dateiausgabe ist verboten und Core-Dumps sind deaktiviert. IPC-Empfangswarteschlangen halten höchstens vier Frames, Sendewarteschlangen einen; Pipe-Schreibvorgänge haben 2 Sekunden Limit. stderr wird bei Budget null direkt verworfen; unvertrauenswürdige Inhalte werden nicht weitergeleitet. Abbruch widerruft die Senke sofort, gibt 2 Sekunden zur Kooperation und beendet und räumt dann Prozess und Pipes auf; Ack ist keine Veröffentlichungserlaubnis. Diese Budgets begrenzen einen kleinen Versuch und versprechen keine Leistung für gewöhnliche große Archive.

<a name="xdzip-platform-gates"></a>

## Plattformbedingungen

| Plattform | Versuchspolitik und Fehlerverhalten |
| --- | --- |
| macOS | Die feste `/usr/bin/sandbox-exec`-Deny-default-Policy erlaubt die genaue private ausführbare Kopie, Systembibliotheken/dyld-Cache, Root-Loader-Lesezugriffe und `/var`-Metadaten. Benutzerdateien, Schreiben, Netzwerk und fork sind verboten; die Umgebung wird geleert und nur stdio geerbt. Native rlimits und RSS-Überwachung alle 20 ms beenden bei über 512 MiB oder Messfehler. RSS wird stichprobenartig erfasst, kurze Spitzen zwischen Messungen können den Grenzwert überschreiten; diese Schnittstelle wird für zukünftige offizielle Distribution nicht versprochen. |
| Linux x86_64 / aarch64 | Landlock ABI ≥3 behandelt alle v3-Dateirechte ohne Schreibregeln und mit Lesezugriff nur auf Programm und System-Loaderbibliotheken. `no_new_privs` und eine architekturgeprüfte seccomp-Systemruf-Allowlist enthalten readv/writev für Rust-Standardpipes und verbieten socket, fork/clone, ptrace, io_uring und andere. Es gelten 3 Sekunden CPU-Limit, 512 MiB Adressraumlimit und weitere rlimits. Fehlende Regeln, ABI, seccomp oder benötigte Systemrufe führen vor exec zum Fehler. Nur native CI belegt tatsächliche Ergebnisse dieser Plattform. |
| Windows und andere Plattformen | Ausführung wird bedingungslos verweigert. AppContainer, Job und Handle-Allowlisting sind nicht implementiert und geprüft; ein gewöhnlicher Unterprozess ersetzt keine Sandbox. |

Linux-API-Entscheidungen folgen der [Kernel-Landlock-Dokumentation](https://docs.kernel.org/userspace-api/landlock.html) und UAPI; künftige Windows-Arbeit folgt [AppContainer-Isolation](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation). Nativer unsafe-Code des Hosts ist auf das Sandboxmodul beschränkt und dokumentiert Layouts, Aufrufvoraussetzungen und Fehlerverhalten; pre-exec allokiert und protokolliert nicht. Der Referenzpartner ergänzt kontrollierte fork/_exit/waitpid-Sonden nur im ausdrücklichen Auditmodus; das Kind führt keine Rust-Bereinigung oder Formatlogik aus. Plattformregressionen verwenden zuerst eigene lesbare/schreibbare Marker, einen eigenen localhost-Listener und self-exec als Positivkontrollen und prüfen danach die Kernelverweigerung von Lesen, Schreiben, Netzwerk, direktem fork und self-exec anhand tatsächlicher Marker-, Datei- und Verbindungszustände. Ungetestete Plattformen werden nicht als bestanden markiert und Drittanbieterausführung auf dem Desktop bleibt deaktiviert.

<a name="xdzip-owned-xdp1-example-format"></a>

## Eigenes XDP1-Beispielformat

XDP1 ist nur ein Interoperabilitätsfixture und ersetzt kein reales Archivformat. Auf vier Bytes `XDP1` folgt eine `u32 LE`-Datensatzanzahl. Jeder Datensatz enthält eine `u16 LE`-Bytezahl des UTF-8-Namens, eine `u32 LE`-Bytezahl des Inhalts, Namen und unkomprimierten Inhalt. Höchstens 1000 Datensätze sind ohne nachfolgende Daten erlaubt; Namen und Inhalte werden innerhalb der Hostbudgets gelesen. Eintragstoken sind vier Byte lange physische LE-Ordinalzahlen ohne pfadbasierte Deduplizierung. Python-Fehlermodi werden nur in Konformitätstests ausdrücklich gewählt, um unbefugten Zugriff, Reihenfolge, Replay, doppelte Identität, Pfade, Kredite, Digests, Abstürze, Abbruch und doppelte Endzustände zu prüfen.
