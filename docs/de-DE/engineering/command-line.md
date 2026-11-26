# Laufzeitvertrag der Befehlszeile

[English](../../engineering/command-line.md) | [简体中文](../../zh-CN/engineering/command-line.md) | [繁體中文](../../zh-TW/engineering/command-line.md) | [日本語](../../ja-JP/engineering/command-line.md) | [한국어](../../ko-KR/engineering/command-line.md) | [Français](../../fr-FR/engineering/command-line.md) | Deutsch | [Español](../../es-ES/engineering/command-line.md) | [Português (Brasil)](../../pt-BR/engineering/command-line.md) | [Русский](../../ru-RU/engineering/command-line.md)

Das CLI ist ein eigenständiger Rust-Host, der Format- und sichere E/A-Dienste über denselben `xdzip-runtime` aufruft. Es hängt weder von Qt noch QML oder einer grafischen Sitzung ab. 7z, RAR, CAB, ISO und UDF verwenden weiterhin mitgelieferte isolierte Worker und suchen nicht im PATH nach anderen Entpackprogrammen. Plattformen und Architekturen folgen der [Supportrichtlinie](support-policy.md). Das eigenständige Linux-CLI benötigt kein X11; zur Ausführung sind OpenSSL 3 und Zertifikate des Systems nötig, für den Build zusätzlich libssl-dev und pkg-config.

Ab `v0.2.0` bieten offizielle Releases eigenständige CLI-Pakete für drei Plattformen; Desktoppakete enthalten ein privates CLI. Frühere Release-Dateien ändern sich nicht rückwirkend. Downloads stehen bei den [Releases](https://github.com/xiaodaozhi/xdzip/releases); die folgenden Abschnitte und das [Release-Protokoll](release-protocol.md) erläutern die Grenzen von Quellbuild, Installation und Aktualisierung. Implementierung und Abnahme werden in [#254](https://github.com/xiaodaozhi/xdzip/issues/254) verfolgt.

## Verwendung und Ausgabe

```sh
xdzip --help
xdzip version
xdzip list archive.zip --json
xdzip extract archive.zip docs/ -C output
xdzip extract archive.zip --entry-id 3 -C output
xdzip create backup.7z docs/ --level normal
xdzip create data.iso docs/
xdzip create portable.sfx.exe docs/ --level normal
xdzip create private.zip docs/ --encrypt --password-stdin
xdzip add backup.zip notes.txt --into docs
xdzip test backup.zip
xdzip update --check
xdzip update --download --output downloads
xdzip update --yes
xdzip update --status
xdzip gui archive.zip
```

Ohne Argumente oder mit `--help` zeigt xdzip die Hilfe. `version`, `--version` und `--version-json` melden die Identität aus `VERSION.json` und dem Git-Zustand des Builds. `--version-json` gibt ein eigenständiges Objekt mit product, version, build, revision, dirty, component, channel, configuration, distribution, platform und architecture aus. Für Hilfe und Meldungen `--language zh-CN|en-US` verwenden. Standardmäßig werden `LC_ALL`, `LC_MESSAGES` und `LANG` in dieser Reihenfolge geprüft; beginnt der gewählte Wert mit `zh`, wird Chinesisch verwendet. Öffentliche Bezeichner und Fehlercodes sind Englisch. `l`, `x`, `c`, `a` und `t` sind Aliasnamen für `list`, `extract`, `create`, `add` und `test`. Mit `--` lassen sich wörtliche Dateiargumente übergeben.

Alle Geschäftsoperationen akzeptieren `--json`. stdout gibt genau ein JSON-Objekt mit den festen Feldern schema_version (1), command, status, error_code, data, summary und warnings aus. Unbekannte Größen sind null, auch bei Kindern eines gemeinsamen komprimierten TAR-Streams; rohe DEB-ar-Mitglieder behalten ihre exakten gepackten Größen. Listendaten enthalten Format und Einträge, jeweils mit id, path, display_name, size, packed_size, method, directory, link, payload_supported, encrypted und warning. path ist der festgehaltene effektive Name; ungültige oder nicht aufgelöste Namen sind null. display_name dient nur der maskierten Anzeige. Aufgabenzusammenfassungen enthalten tatsächliche Lese-, Dekodier-, Prüf-, Commit-, Übersprung-, Verzeichnis- und Volumezahlen sowie Bereinigungswarnungen und unsichere Ergebnisse.

Fortschritt, Fehler und interaktive Fragen gehen an stderr. JSON-Aufrufe, Aufrufe ohne Terminal oder mit `--non-interactive` fragen nicht nach. Fehlt ein Passwort oder eine Konfliktentscheidung, wird die wartende Aufgabe abgebrochen, die Bereinigung abgewartet und interaction_required zurückgegeben. Exitcodes sind 0 für Erfolg oder ausdrückliche Übergabe, 1 für Fehler, Teilerfolg oder Bereinigungswarnung, 2 für ungültige Argumente, 3 für fehlende Interaktion, Berechtigung oder Installationsbedingung und 130 für bestätigten Abbruch. Ausdrückliches Überspringen ist eine ausgeführte Richtlinie; übersprungene Einträge werden gesondert gezählt. Wird der Abbruch nicht angenommen und ist der Commit bereits abgeschlossen, bleibt das tatsächliche Ergebnis erhalten.

<a name="xdzip-archive-behavior"></a>

## Archivverhalten

`list` gibt physische Datensätze zurück und behält Duplikate. `extract` akzeptiert vollständige, exakte und groß-/kleinschreibungssensitive effektive Mitgliedspfade mit Schrägstrichen. Verzeichnisse werden rekursiv erweitert und behalten die Hierarchie unter der Wurzel. Ein doppelter Pfad ergibt `ambiguous_member`. Mit `--entry-id` die von `list` gelieferte numerische ID ab 1 übergeben; für mehrere IDs den Schalter wiederholen. Nicht auflösbare ausdrückliche Pfade oder IDs schlagen vor Erstellung des Zielverzeichnisses fehl. Platzhalter, Archivlesen von stdin und Ausgabe extrahierter Inhalte an stdout werden nicht unterstützt.

`--limit` oder `--offset` aktiviert Seitenaufteilung. Der Offset beginnt bei null und darf höchstens 1.000.000 sein. Bei nur `--limit` ist er null, bei nur `--offset` beträgt das Limit 128. Gültige Limits sind 1–4.096. JSON `data` enthält `offset`, `total` und `next_offset`, das null sein kann. `summary.entries` zählt tatsächlich zurückgegebene Zeilen. Ein gewöhnliches einteiliges ZIP/ZIP64 kann eine Million Einträge indexieren; eine Liste ohne Seiten lehnt mehr als 100.000 ab. Beide Modi haben ein Budget von 16 MiB für serialisierte JSON-Zeilen und gerenderten Text. Bei Überschreitung wird `resource_limit` ohne Teilzeilen zurückgegeben. Seitengrenzen ändern physische IDs nicht. Siehe [Indexierung großer Archive](large-index.md).

extract verwendet standardmäßig das aktuelle Verzeichnis. Bezeichnet `-C` ein fehlendes Verzeichnis, wird die Berechtigung komponentenweise vom nächsten vorhandenen sicheren Elternverzeichnis eingeholt, statt Verzeichnisse bedingungslos rekursiv anzulegen. Ziel, Quellidentität, Links, Unicode-/Großschreibungskollisionen, Vorbereitung, Prüfung und Commit folgen weiterhin [sicherer Extraktion](zip-safe-extraction.md) und [Dateinamenkodierung](filename-encoding.md). Beim Extrahieren werden Dateien bei Konflikten standardmäßig umbenannt. `--conflict ask|skip|rename|overwrite` wählt die Richtlinie ausdrücklich. `--allow-high-ratio` bestätigt nur das hohe Kompressionsverhältnis und hebt harte Ressourcengrenzen nicht auf.

`create` wählt das Profil anhand der Ausgabe-[Endung](format-registry.md) und verlangt ein vorhandenes Elternverzeichnis. `--level store|fast|normal|best` wird gegen das gewählte Profil geprüft. Für einfaches TAR und ISO die Stufe weglassen oder `store` wählen. LHA (`.lzh` oder `.lha`) verwendet LH5-Stufen 1, 6 und 9 für `fast`, `normal` und `best`. ZIP und 7z unterstützen `--encrypt` und `--volume-size`; 7z unterstützt profilabhängig auch `--encrypt-names` und `--solid`. Volumegrößen akzeptieren positive ganzzahlige Bytes oder die Endungen `KiB`, `MiB` und `GiB`. Vom Profil nicht unterstützte Argumente schlagen vor dem Schreiben fehl.

Passwörter werden nur über verdeckte Terminaleingabe oder eine UTF-8-Zeile von `--password-stdin` übergeben. Zeilenenden werden entfernt, das Passwort selbst nicht beschnitten. Interaktive verschlüsselte Erstellung erfordert zweimalige Bestätigung. Klartext-Passwortargumente werden abgelehnt; Passwörter werden weder aus Umgebungsvariablen gelesen noch in Einstellungen oder Logs geschrieben. Länge und Nullsetzen folgen dem [Passwortvertrag](password-extraction.md) und dem [Erstellungsvertrag](zip-creation.md).

`add` reicht eine Hostaktualisierung ein und verwendet standardmäßig die Archivwurzel. `--into` wählt ein vorhandenes sicheres Verzeichnis anhand seines archivrelativen Namens. Zulässige Eingaben können Einträge hinzufügen oder ersetzen. Interne Verschiebungen sind eine Desktopoperation. Das CLI teilt die Prüfungen der [Archivänderung](archive-transfers.md) für zulässige Formate und Datensätze, Eingabeschutz, temporären Neuaufbau und abschließenden Ersatz. `test` prüft das gesamte Archiv ohne Mitgliedsdateien zu erzeugen. JSON `data.check_kind` meldet die Stärke einer abgeschlossenen Prüfung gemäß [Archivprüfung](archive-checking.md).

Jede CLI-Operation besitzt eine eigene Laufzeit und tritt keinen Desktop-Browsersitzungen bei. Ctrl+C und Beendigungssignale fordern kooperativen Abbruch an und warten auf Endzustand, Bereinigung und Abschluss der Laufzeit; bereits bestätigte Dateien bleiben erhalten. `submit_wait` des gemeinsamen Dienstes nimmt seriell nur eine kurze Zustandssperre, wartet nicht auf Warteschlangenkapazität und verändert die nichtblockierende GUI-Aufnahme nicht. Katalogseiten enthalten höchstens 128 physische Einträge. Bestehende Grenzen für Anzahl und Namen von SelectMembers bleiben bestehen; alle Anfragen binden eine unveränderliche Sitzungsidentität.

<a name="xdzip-build-and-command-installation"></a>

## Build und Befehlsinstallation

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
python3 scripts/cli.py package --output build/package/cli
build/cli/bin/xdzip --help
```

Dieser Einstieg verwendet das im Repository festgelegte Rust, CMake, die C++-Plattformtoolchain und den vorhandenen isolierten Worker ohne Qt. Für Windows-Pakete ist außerdem das vorhandene `prepare-windows-runtime.py` nötig, um die festgelegte lokale VC++-Laufzeit mit ihrer ursprünglichen Lizenz vorzubereiten. Der einheitliche Qt-Buildeinstieg erstellt das CLI ebenfalls in einem privaten cli-Verzeichnis, ohne GUI-Datei, App-Symbol oder Zuordnungseinstiege umzubenennen.

Die mitgelieferte macOS-CLI-Datei liegt in `Contents/Helpers/cli/`. Ihr gewöhnlicher JSON-Kontext liegt in `Contents/Resources/xdzip-cli-context.json` und ist durch die Ressourcensignatur der App geschützt. Das CLI findet diese Ressource nur im festen Contents/Helpers/cli-Layout und prüft Schema, Desktoptyp und festen relativen GUI-Pfad. Fehlende Markierungen gelten als portable Identität; ungültige werden abgelehnt. Wiederholte Builds entfernen veraltete erzeugte Markierungen unter Helpers; die Signaturprüfung zur Behebung eines Layoutfehlers nicht abschwächen.

Neue Desktopbundles deklarieren im Kontext zusätzlich `worker_relative` und teilen den Desktop-Formatworker. Nur feste plattformrelative Pfade werden akzeptiert: der benachbarte Desktopworker unter Windows, `Contents/MacOS/xdzip-native-codec` unter macOS und der buildbenachbarte oder unter `bin/` bereitgestellte Worker unter Linux. Ein fehlender, verlinkter oder unerwarteter Worker lehnt den Kontext ohne Suche im PATH oder nach einer anderen Kopie ab. Alte Kontexte ohne dieses Feld sowie eigenständige oder verwaltete CLI-Pakete behalten die Suche nach einem benachbarten Worker. Ausdrückliche Entwicklerüberschreibungen haben weiterhin Vorrang. Die bereitgestellte Suche mit `scripts/check-cli.py --default-worker` prüfen; gewöhnliche Testüberschreibungen belegen sie nicht.

Nach Entpacken eines eigenständigen Pakets bin/xdzip ausführen, unter Windows mit .exe. `bin/xdzip setup` installiert ausdrücklich einen stabilen benutzerspezifischen Einstieg und ein versioniertes Laufzeitverzeichnis. Der Unix-Standardeinstieg liegt in ~/.local/bin, unter Windows in xdzip/cli/bin im LocalAppData des Benutzers. Verwaltungsverzeichnisse verwenden die Benutzerdatenorte der Plattform. `--bin-dir` und `--install-dir` überschreiben die Orte ausdrücklich. Benutzer fügen das Verzeichnis selbst zum PATH hinzu; das Werkzeug ändert keine Shell-Startdateien. Vorhandene Einstiege oder Registrierungsdateien werden niemals überschrieben.

Verwaltete Identität erfordert eine gültige installation.json. Ein bloß versions genanntes Verzeichnis ohne Registrierungsdatei bleibt portabel.

Beim Entfernen eines laufenden Windows-Launchers wird die eigene Kopie mit weiterhin passendem Hash in das private Unterverzeichnis .xdzip-retired desselben Befehlsverzeichnisses verschoben. Die Registrierung wird sofort entfernt und das Ergebnis hält retained_launcher fest. Ein späteres ausdrückliches setup räumt nur Kopien in begrenzten Belegen auf, deren Hashes noch stimmen und deren Prozesse beendet sind. Geänderte Dateien bleiben erhalten und werden nicht überschrieben. Das CLI im versionierten Verzeichnis kann setup erneut zur Registrierung ausführen.

Das Feld distribution unterscheidet Quell- und Release-Identität. Ein gewöhnlicher Quellbuild, selbst sauber und in Release-Konfiguration, unterstützt nur Prüfung und Download. Automatischer Ersatz erfordert gemeinsam eine vom dedizierten Release-Pipeline markierte distribution-Identität, eine gültige verwaltete Installation und eine authentifizierte Zielsignatur.

Die Windows-PATH-Option des Desktopinstallers ist standardmäßig aus und lehnt Konflikte mit anderen xdzip-Befehlen ab. Unter Mac/Linux kann setup aus dem privaten CLI des Desktoppakets ausgeführt werden; es registriert nur das CLI innerhalb der Desktopinstallation und kopiert keinen separat aktualisierbaren Kern heraus. `setup --remove` entfernt nur hashgeprüfte Einstiege und Registrierungsdateien, die diesem Werkzeug gehören. Versionsverzeichnisse und Wiederherstellungsmaterial bleiben; Benutzereingaben und möglicherweise noch verwendete Laufzeitverzeichnisse werden nicht gelöscht.

Der stabile Launcher verwendet keine Shell-Escapes. Unter Unix führt er das tatsächliche CLI mit exec aus; unter Windows wartet er auf den Konsolen-Kindprozess und reicht Abbruch und Exitstatus weiter. Ein atomarer current.json-Zeiger wählt das aktuelle Laufzeitverzeichnis. Fehler bei Erstinstallation, Selbstprüfung der neuen Version oder Datei-Commit dürfen kein erfolgreich installiertes Ergebnis erzeugen.

## Updates, Desktopübergabe und Wiederherstellung

update zielt auf die aktuelle Installation. Ein eigenständiges CLI aktualisiert nur das CLI; ein im Desktoppaket enthaltenes CLI aktualisiert die gesamte Desktopedition. `--check` fragt ab, `--download` lädt herunter und prüft, der Standardvorgang installiert nach Bestätigung und `--status` prüft Belege und tatsächliche Identität. `--yes` bestätigt nur dieses Update und ersetzt keine Systemberechtigung. Portable, veränderte, Quell- oder Builds unbekannter Identität erlauben nur Prüfung und Download. Für eigenständigen automatischen Ersatz ist eine verwaltete, saubere Release-Installation nötig.

Das CLI verwendet feste öffentliche GitHub Releases, Kanal und Regeln monotoner Buildnummern. CLI-Dateien der drei Plattformen verwenden separate cli-release.json und cli-release.json.sig; der eingebettete öffentliche Schlüssel prüft rohe Bytes vor Größen- und SHA-256-Prüfung. Fehlende Signaturen oder nicht vertrauenswürdige Schlüssel liefern nur Informationen zur manuellen Beschaffung; ungültige Signaturen schlagen ausdrücklich fehl. Das Desktop-release.json für sechs Pakete enthält keine CLI-Einträge; ältere Clients nutzen weiterhin das ursprüngliche Protokoll.

Das Netzwerk verwendet standardmäßig Systemzertifikate und liest übliche Proxy-Umgebungsvariablen. `--proxy` wählt einen HTTP/SOCKS5-Proxy ohne Zugangsdaten; `--no-proxy` verbindet direkt; `--mirror` wählt ein HTTPS-Präfix ohne Zugangsdaten, Abfrage oder Fragment. Ursprüngliche GitHub-URLs, Weiterleitungshosts, Antwortgrößen, Zeitlimits und Downloadbudgets bleiben begrenzt. Netzwerk-, TLS-, Offline-, Ratenlimit- und Kein-kompatibles-Update-Ergebnisse werden getrennt gemeldet; automatisches Polling oder Wiederholung gibt es nicht.

Eigenständige Updates werden in einem privaten benutzereigenen Verzeichnis vorbereitet, über dieselbe Archivlaufzeit sicher extrahiert und auf das vollständige feste Dateiinventar und jeden Dateidigest, Produkt, Komponente, lokale Plattform/Architektur und Kanal des signierten Manifests geprüft. Vor Umschalten des Versionszeigers wird die `--version-json`-Selbstprüfung der neuen Version ausgeführt. Transaktionen schließen sich gegenseitig aus und Belege binden die Zielidentität. Bei Fehler bleibt der alte Zeiger erhalten oder es wird ausdrücklich eine Wiederherstellung eingeleitet; ein automatischer Rollback wird nicht erfunden.

Desktopprüfung und Download fragen nur den nichtgrafischen Installationsidentitäts-Einstieg des Hauptprogramms ab; sie erstellen keine QApplication und verbrauchen keine Installationsbelege. Die Installationsabsicht läuft über den vorhandenen Einzelinstanzkanal desselben Benutzers, der nur ein begrenztes Anfrageverzeichnis und eine Kandidatenversion akzeptiert, keine beliebigen Befehle, Pakete oder Vertrauenswurzeln. Anfrage- und Ergebnisdateien verwenden Rusts native Verzeichnisberechtigung und sichere Commits. Der Desktopcontroller prüft und vergleicht den Kandidaten unabhängig erneut und verwendet Über, Installer, Sparkle und die Join-Barriere aller Aufgaben. Desktop-CLI-Aufgaben halten ebenfalls Installationsbelegungsschutz; die Installation kann sie nicht zerstören.

Ein Terminal meldet nach angenommener Übergabe nur handed_off. Tatsächliche Installation und Wiederherstellung hängen von Desktopbelegen und der tatsächlich laufenden Identität ab; Download oder Übergabe bedeutet keine Installation. Der nichtinteraktive Modus lehnt Desktopinstallation ab, die GUI oder Systemberechtigung verlangt. Siehe [Updatevertrag](software-update.md) zu Installationssicherheit und nativen Abnahmegrenzen. Automatische, native und manuelle Abnahme bleiben getrennt.

Das `.iso`-Profil erstellt unkomprimierte ISO9660/Joliet-Datenabbilder auf Stufe 0. `--level` weglassen oder `--level store` verwenden. Das Windows-x64-Profil `.sfx.exe` umhüllt Store/Deflate-ZIP mit dem mitgelieferten benachbarten `xdzip-sfx.exe`. Es lehnt Verschlüsselung, Solid-Modus und Volumes ab. Keines der beiden Profile unterstützt das Ändern einer vorhandenen Ausgabe. Siehe [ISO-Erstellung](iso-creation.md) und [eigene SFX-Erstellung](sfx-creation.md).
