# Entwicklungsbuilds und Plattformprüfung

[English](../../engineering/developer-build.md) | [简体中文](../../zh-CN/engineering/developer-build.md) | [繁體中文](../../zh-TW/engineering/developer-build.md) | [日本語](../../ja-JP/engineering/developer-build.md) | [한국어](../../ko-KR/engineering/developer-build.md) | [Français](../../fr-FR/engineering/developer-build.md) | Deutsch | [Español](../../es-ES/engineering/developer-build.md) | [Português (Brasil)](../../pt-BR/engineering/developer-build.md)

Dies ist der gemeinsame Einstieg für Vorbereitung, Build, Tests, Paketierung und Prüfung von Umgebungen auf drei Plattformen. Die gemeinsame Orchestrierung implementiert [scripts/desktop.py](../../../scripts/desktop.py). Plattformdokumente ergänzen Bereitstellungslayouts, Werkzeugverhalten und Systemgrenzen, statt getrennte Routinebefehle zu pflegen.

Die eigenständige CLI verwendet `python3 scripts/cli.py build`, `test` und `package` ohne Qt, X11 oder grafische Sitzung. Linux-Kompilierung benötigt zusätzlich `libssl-dev pkg-config`; zur Laufzeit werden System-OpenSSL und Zertifikate verwendet. Desktop-Builds erzeugen automatisch ein privates CLI-Verzeichnis. Befehle, Installation und Grenzen vertrauenswürdiger Updates stehen im [CLI-Vertrag](command-line.md).

Entwicklungsbuilds bleiben experimentell und sind keine offiziellen Release-Installer. Fähigkeiten und Grenzen stehen im [README](../../../README.de-DE.md) der Wurzel, unterstützter Umfang in der [Supportpolitik](support-policy.md). Die historische Release-Prüfung von `v0.1.0` steht in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) und [#53](https://github.com/xiaodaozhi/xdzip/issues/53); spätere Versionen benötigen neue Prüfungen ihrer konkreten Kandidaten.

<a id="environment-and-entry-points"></a>

## Umgebung und Einstiegspunkte

Zunächst Python 3.10 oder neuer, Git, CMake 3.24 oder neuer, Ninja, rustup und Plattformcompiler/SDK installieren. Die Rust-Version stammt aus rust-toolchain.toml in der Wurzel; Qt ist auf 6.8.2 festgelegt. Abhängigkeits- und Updateregeln stehen in der [Toolchain-Politik](toolchain.md). Befehle im Repository-Wurzelverzeichnis ausführen. Unter Windows in Beispielen `python3` durch ein verfügbares `python` ersetzen und eine x64-MSVC-Entwicklungsumgebung nutzen.

## Lokale Toolchain-Verzeichnisse

Beim Laden der PowerShell-Umgebung von Visual Studio die Unicode-Werte von `USERPROFILE`, `LOCALAPPDATA` und `APPDATA` bewahren und wiederherstellen, falls der Lader ihre Kodierung ändert. Qt verwendet diese Orte für Standard-Cache und Konfiguration; beschädigte Werte können Dateisystemfehler verursachen, selbst wenn der Compiler funktioniert.

`prepare` baut und prüft im vorhandenen SDK ein sicherheitskorrigiertes QtNetwork für CVE-2026-116337 und CVE-2026-105946; andere Laufzeitmodule werden durch die neu gebaute Bibliothek nicht ersetzt. Perl, make (nmake unter Windows) und das OpenSSL-Kommandozeilenwerkzeug sind ebenfalls erforderlich. Ubuntu benötigt `zlib1g-dev libpcre2-dev libicu-dev libglib2.0-dev libzstd-dev libkrb5-dev`. Quelle, erzeugte Deklarationen und Build-Caches bleiben unter `build/`, standardmäßig mit Wiederverwendung von `build/toolchain/qt-network-security/<platform>`. Stimmen Eingabe- und tatsächliche Bibliotheksdigests nicht überein, schlagen Produktbuild und Paketierung sofort fehl; `prepare` erneut ausführen. Herkunft, ABI, TLS-Funktion und Fehlerbehebung stehen in den [Patch-Wartungsnotizen](../../../patches/qt/README.md).

Lokale Abhängigkeiten des gemeinsamen Einstiegs nutzen ausschließlich `build/toolchain/`, das Git ignoriert und nicht committet:

| Inhalt | Festes Verzeichnis | Hinweise |
| --- | --- | --- |
| Qt SDK 6.8.2 | `build/toolchain/Qt/6.8.2/<platform-subdirectory>` | `<platform-subdirectory>` ist `macos`, `msvc2022_64` oder `gcc_64`; qmllint, qmake, CMake-Konfiguration und Qt-Laufzeiten stammen von hier |
| Python-Werkzeugumgebung | `build/toolchain/desktop-tools` | Feste virtuelle Umgebung für aqtinstall und Paketierungs-/Prüfskripte |
| Qt-Download-Staging | `build/toolchain/qt-install-tool` | Nur Downloads und temporäre Eingaben des Qt-Installers |

`scripts/desktop.py prepare --install-qt`, `doctor`, `build` und `test` teilen diese Orte standardmäßig. Keine separaten Kopien `.cache/Qt`, `.cache/desktop-tools` oder Ähnliches für jede Entwicklungspipeline anlegen. `--qt-prefix`, `--qt-root` und `--tools-dir` nur zur Trennung verschiedener Qt-Versionen, Plattformen oder experimenteller Builds ausdrücklich angeben. CI kann explizit temporäre Runner-Verzeichnisse verwenden; CI-Fallbackpfade ändern lokale Standardwerte nicht. CMake, Ninja, rustup, Rust und der Plattform-C++-Compiler gehören zur Host-Toolchain und werden gemäß Umgebungsvorprüfung installiert bzw. aufgerufen, nicht durch das Qt-Werkzeugverzeichnis kopiert.

Nach einem Entwicklungszyklus und der Bereinigung von Branches und Worktrees temporäre, in diesem Zyklus unter `build/` erstellte Verzeichnisse und Dateien entfernen. Wiederverwendbare gemeinsame Toolchains wie `build/toolchain/` behalten; zyklusspezifische Builds, Diagnosen und Download-Staging entfernen. Regeln und manuelle Ersatzmethode stehen unter [Repository-Inhalt und Historie](../../../developers/engineering/repository-hygiene.md#研发周期工件收尾).

| Plattform | Zuerst vorzubereitende Systemumgebung | Qt-SDK-Unterverzeichnis |
| --- | --- | --- |
| macOS | Natives Apple-Silicon-Terminal, Xcode Command Line Tools und gültiges macOS SDK | macos |
| Windows | MSVC 2022 x64 C++-Werkzeuge und Windows SDK, x64 Developer PowerShell/Kommandozeile | msvc2022_64 |
| Ubuntu 24.04 | x86_64-C++-Compiler, OpenGL-/EGL-/xkbcommon-Entwicklungspakete und X11-Laufzeitbibliotheken | gcc_64 |

Unter macOS bereitet `xcode-select --install` die Kommandozeilenwerkzeuge vor. Ubuntu-Basisentwicklungspakete umfassen `libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev libx11-dev libxext-dev libnautilus-extension-dev pkg-config`; Paketierung benötigt zusätzlich patchelf, binutils (readelf und strip) und Debian-Paketwerkzeuge, isolierte Laufzeitprüfung erreichbares Docker. Visual Studio unter Windows nur zu installieren, ohne eine Entwicklungsumgebung zu öffnen, kann cl, link, rc oder dumpbin unzugänglich lassen; die Vorprüfung identifiziert tatsächlich fehlende Werkzeuge. Der gemeinsame Einstieg installiert keine Systemcompiler, SDKs oder administrativ zu installierenden Systempakete automatisch.

Windows-Entwickler können ihre Werkzeuge nach dem [kostenlosen Community-Lizenzweg](windows-licensing.md) vorbereiten und die Bedingungen dieser Version lesen und akzeptieren. Build Tools oder VC_redist allein belegen keine Berechtigung. Unternehmensnutzung und zugehörige Weiterverteilung auf standardmäßig von GitHub gehosteten Runnern besitzen separate öffentliche Befugnis; Maintainer müssen für gehostete Builds weder Enterprise kaufen noch lokal eine Community-IDE installieren, und diese Befugnis gilt nicht für selbst gehostete Rechner. Vorhandene 2019-Werkzeuge müssen zur Lizenzprüfung nicht deinstalliert werden; die Kompatibilität lokaler Alternativen mit dem aktuellen 2022-Release-SDK ist separat zu prüfen.

Alle Befehle teilen diese Optionen:

| Option | Standard und Zweck |
| --- | --- |
| `--qt-prefix` | Zuerst `XDZIP_QT_PREFIX`, sonst `build/toolchain/Qt/6.8.2/<platform-subdirectory>`; verweist auf SDK-Präfix, nicht bin |
| `--build-dir` | `build/desktop`, CMake-Build- und Testverzeichnis |
| `--output-dir` | `build/package`, Verzeichnis für Lieferartefakte und Prüfausgaben |
| `--tools-dir` | `build/toolchain/desktop-tools`, isolierte Python-Werkzeuge und Vorbereitungsmaterialien |
| `--configuration` | `Release`; eine andere Entwicklungskonfiguration kann explizit gewählt werden |

build-dir und output-dir müssen unabhängige Unterverzeichnisse innerhalb von build/ des Repositorys sein, entsprechend den Ausgabegrenzen von Paketierung und Prüfung. Für alle Stufen eines Laufs dieselben Optionen verwenden. Explizites `--qt-prefix` hat Vorrang vor der Umgebungsvariable; globale Änderungen von CMAKE_PREFIX_PATH sind unnötig. Der Einstieg bereitet Qt- und Werkzeugsuchumgebungen für Unterprozesse vor. Normale Desktopstarts sollen keine Offscreen-Testvariablen erben. Erzeugte Ausgaben liegen nur in ignoriertem build/, target/, dist/ oder Werkzeugcaches und werden nicht committet.

## Stufenweise Ausführung

Stufen können unabhängig laufen und wiederholen frühere Vorbereitung, Builds oder Tests nicht implizit. Fehlende Voraussetzungen erzeugen Fehler; entsprechende Stufe ausführen und erneut versuchen. Jede Stufe prüft vor der Arbeit ihre Umgebung; `doctor` zeigt Probleme ohne Build oder Paketierung.

Geschützte Windows-Builds koordinieren Prozesse innerhalb ihrer vorhandenen Konsole. Getrennte Konsolen dürfen parallel laufen; Builds in derselben vorhandenen Konsole laufen sequenziell. Während eines Builds darf anderer Code geschützte Prozesse nicht von dieser Konsole trennen oder erneut verbinden. Eine von einem Prozess ohne Konsole erzeugte temporäre Konsole dient diesem Prozess und seinen Kindern; externe Prozesse dürfen sich nicht verbinden. Ältere Skripte nutzen für Konsolen ohne Fensterhandle eine gemeinsame Fallback-Sperre und können nicht mit neueren Skripten koordinieren; keinen neuen Build in derselben fensterlosen Konsole starten, bevor ein älterer endet. Können Prozesslisten nicht bestimmt werden, ohne Änderung der Codepage stoppen.

Für nicht interaktive Builds kann ausdrücklich ein unabhängiger Konsolen-Wrapper mit vollständig umgeleitetem stdin, stdout und stderr genutzt werden. Beispiel PowerShell:

```powershell
New-Item -ItemType Directory -Force build\isolated-logs | Out-Null
cmd /c "python scripts\windows_detached_console.py --timeout 7200 -- python scripts\desktop.py build <NUL >build\isolated-logs\desktop.log 2>&1"
$LASTEXITCODE
```

Der Wrapper erzeugt pro Befehl eine unabhängige versteckte Konsole und beendet Kinder bei Timeout oder Supervisor-Ende; fehlende Abschlussmeldungen führen zum Fehler. Parallele Builds benötigen weiterhin getrennte `--build-dir`-Werte; der Wrapper sperrt Build-Verzeichnisse nicht. Vorhandene Abhängigkeitsprüfungen von desktop-build und `windows_ninja_safety.py` laufen weiterhin, werden durch Wrapping anderer Befehle aber nicht automatisch ergänzt. Dieser Einstieg dient nur nicht interaktiven Befehlen; Eingabeaufforderungen, Passworteingabe und Pseudokonsoleninteraktion werden nicht unterstützt.

`prepare` prüft den aktuellen Produktformat-Quellbestand. ZIP, TAR/GZIP, 7z, RAR, CAB und optische Container verwenden unabhängiges Rust; die offizielle Produktquellliste ist leer, offizielle UnRAR-/7-Zip-Engines werden nicht heruntergeladen oder kompiliert. Unabhängige Testreferenzen werden aus eigenen festgelegten Quellen vorbereitet und von Produktlaufzeitabhängigkeiten getrennt. Ursprüngliche Quellhinweise für begrenzte RAR-Zuordnungen bleiben in Paketierung und entsprechendem Quellmaterial; siehe [Lizenzvertrag](licensing.md). `build` nutzt nur vorbereitete Eingaben und lädt keine variablen Versionen herunter. Für reine Rust-Tests zuerst `xdzip-native-codec` bauen, `XDZIP_NATIVE_CODEC` auf diesen Worker richten und Workspace-Tests ausführen. Der native Backend-Ablauf führt zusätzlich `cargo test -p xdzip-runtime --locked native_backend_tests -- --ignored` aus; gemeinsames `test` und Foundation CI verlangen beides. Ein ignored-Status ist keine bestandene Formatprüfung. Quellen, Stream-Berechtigung und Lizenzgrenzen stehen im [nativen Backend-Vertrag](native-codecs.md).

| Befehl | Zweck und Voraussetzungen |
| --- | --- |
| `doctor --for-stage build` | Prüft PATH, festgelegtes Rust, Qt, Compiler, SDK und Versionen; prüft standardmäßig die Build-Umgebung |
| `prepare` | Bereitet gemeinsame Werkzeuge und Abhängigkeiten vor; Download/Installation von Qt, Rust-Toolchains und Paketmaterial benötigt explizite Schalter unten |
| `build` | Konfiguriert und baut tatsächliches Produkt, Tests und Diagnosen; vorbereitete Umgebung erforderlich, uncommittete Änderungen erlaubt |
| `test` | Führt standardmäßig Rust-Workspace-Tests, qmllint und CTest aus; abgeschlossener Build gleicher Konfiguration nötig, uncommittete Änderungen erlaubt |
| `package` | Stellt Laufzeit bereit, sammelt Lizenz-/Herkunftsmaterial, auditiert Bereitstellung und erzeugt ausgewählte Artefakte; saubere Quelle, Build-/Testbelege desselben Commits und Paketwerkzeuge nötig |
| `verify` | Prüft bestehende Laufzeit oder Paket ohne Neubau, Produktinstallation oder Änderung von Standardzuordnungen; Artefakt und Plattformprüfumgebung nötig |

Die Vorprüfung unterscheidet fehlende notwendige Werkzeuge, Versionsabweichungen und für die aktuelle Stufe unnötige Werkzeuge; fehlendes Docker oder Inno Setup blockiert gewöhnliche Entwicklungsbuilds nicht. Die Paketvorprüfung prüft tatsächlichen Rust-/Cargo-Aufruf und gesperrte Abhängigkeitsmetadaten, damit PATH- oder Cacheprobleme nicht erst bei der Lizenzsammlung auffallen. Fehlgeschlagene Befehle und Prüfgrenzen bleiben in lokaler/CI-Ausgabe; Teilabschluss wird nicht als bestanden gemeldet.

Zunächst festgelegte Qt- und Rust-Toolchains vorbereiten:

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt --qt-root build/toolchain/Qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

`--install-rust` verwendet installiertes rustup zur Vorbereitung der im Repository festgelegten Toolchain; rustup selbst wird nicht installiert. `--install-qt` lädt das festgelegte Qt-SDK standardmäßig ausdrücklich nach `build/toolchain/Qt`. Ein vorhandenes SDK muss nicht erneut geladen werden; seinen Pfad jeder Stufe übergeben:

```sh
python3 scripts/desktop.py doctor --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
python3 scripts/desktop.py build --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
```

Nach dem Build öffentliche Testeingaben für den vollständigen `native_backend_tests`-Satz ausdrücklich vorbereiten. CAB-, ISO/UDF- und grundlegende RAR-Fixtures bleiben in Projektquellen unter `tests/fixtures/cab-iso/` und `tests/fixtures/native-archives/`. 7z-Tests erstellen eigene Archive mit dem frisch gebauten Rust-Worker. Dieser Laufzeitsatz benötigt keine offiziellen 7-Zip-/UnRAR-Werkzeuge oder eingebundenen Images; andere Foundation-Interoperabilitätsreferenzen folgen dem [nativen Backend-Vertrag](native-codecs.md).

Zur erstmaligen Beschaffung festgelegter öffentlicher RAR-PPM-, Altpasswort- und weiterer Eingaben ausdrücklich ausführen:

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs
```

Mit vorhandenem, nach Größe und SHA-256 geprüftem festgelegtem Cache ausdrücklich Offline-Modus verwenden, Cachepfad ersetzen und die `junrar/`-Unterverzeichnisstruktur des Manifests beibehalten:

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs --archives /path/to/fixed-rar-cache --offline
```

Festgelegte Quellen und Hinweise stehen in `scripts/rar-rust-fixtures.json` und dessen öffentlichem Kompatibilitätsbestand. Ausgabe `sources.json.environment` verwendet Pfade relativ zur Quellwurzel. Die aktuelle Lieferung entsprechender Quellen enthält diese sieben öffentlichen Cachedateien nicht; Cargo `--offline` liefert sie nicht. Der Vorbereiter exportiert keine Shell-Umgebungsvariablen. Vor vollständigem `test` einen Kindprozess mit folgender expliziter Übergabe starten:

```sh
python3 -c "
import json, os, pathlib, subprocess, sys
root = pathlib.Path.cwd()
worker = pathlib.Path(sys.argv[1]).resolve(strict=True)
prepared = json.loads((root/'build/native-test-inputs/sources.json').read_text(encoding='utf-8'))
environment = os.environ.copy()
environment.update({name: str(root/value) for name, value in prepared['environment'].items()})
environment.update(XDZIP_NATIVE_CODEC=str(worker), XDZIP_NATIVE_CODEC_WORKER=str(worker), XDZIP_CAB_BACKEND='rust')
environment['XDZIP_RAR_FIXTURE_PASSWORD'] = 'password'
subprocess.run([sys.executable, 'scripts/desktop.py', 'test', *sys.argv[2:]], env=environment, check=True)
" build/desktop/tools/native-codec/xdzip-native-codec
```

Unter Windows `.exe` an den Workerpfad anhängen. Bei eigenem SDK, build-dir oder tools-dir dieselben Optionen nach dem Workerpfad ergänzen, etwa `--qt-prefix /path/to/Qt/6.8.2/gcc_64`, und den Pfad an das tatsächliche Build-Verzeichnis anpassen. `XDZIP_CAB_BACKEND=rust` wählt die festgelegte Rust-CAB-Prüfung; `password` ist ein vorhandener öffentlicher Fixture-Testwert, der nur an das Testkind übergeben wird. Diese Übergabe behält Standard-Workspace-Tests und den vollständigen ignored-Satz bei; sie lädt keine Eingaben und baut den Worker nicht implizit.

`test` aktiviert Offscreen/Software nur für Kinder und verunreinigt spätere Desktopbefehle nicht. Bei Routineänderungen nach Auswirkung neu bauen und prüfen. Reine Rust-Kernprüfung kann für den betroffenen Crate ohne Qt `cargo test --locked` ausführen. `test --skip-rust` führt qmllint und Qt CTest aus, überspringt aber Rust-Workspace und vollständige native Backend-Tests; Belege markieren Rust als `not-run`. Wenn eine Rust-Aussage nötig ist, separat ausführen und tatsächliche Ergebnisse dokumentieren.

Browser-Tests können direkt oder in Gruppen über den vorhandenen Suite-Einstieg laufen. Jeder Suite-Prozess erstellt unter Projekt-`build/test-tmp/` eine eindeutige kurze temporäre Wurzel. Beim Beenden nach Zerstörung der Testobjekte und vor Zerstörung der Qt-Anwendung TEMP, TMP und TMPDIR wiederherstellen und bereinigen. Standard-Einstellungsdateien sind ebenfalls isoliert. Unix prüft die Bytelänge der temporären Wurzel samt vorhandenem IPC-Namen gegen die tatsächliche sun_path-Kapazität. Ein zu langer Pfad lässt die Fixture-Vorbedingung ausdrücklich scheitern; Checkoutpfad verkürzen, statt das Projekt zu verlassen oder Produkt-IPC zu ändern. Hilfsprozesse einer Gruppe erben die temporäre Wurzel des Elternprozesses, um die Verzeichnisidentität der Einzelinstanz-IPC zu bewahren, verwenden private Vorprüfungssonden und enden vor dessen Bereinigung. Gruppen teilen keine Wurzeln. Kindern für Sprachneustart und Öffnungsmodus ausdrücklich übergebene Testkonfigurationen werden weiterhin geteilt, statt eine neue leere Konfiguration zu nutzen. Vor Ausführung Kinderdateierstellung, Sperren, Staging, atomare Commits und Einstellungsrücklesen tatsächlich prüfen. Fehler meldet `Fixture precondition` und Exit ungleich null; Tests nicht überspringen oder Fensterverbergungs-Assertions abschwächen. Verzeichnisexistenz und QTemporaryDir::isValid allein belegen keine Schreibbarkeit; Systemrechte werden nicht geändert, um Fixtures bestehen zu lassen.

Testkonfigurationspfade nutzen begrenztes Speichern/Wiederherstellen durch `tests/test_environment`. Windows behält Wide-Character-Werte und unterscheidet im Systemumgebungsblock nicht gesetzte von explizit leeren Einträgen; Unix bewahrt ursprüngliche Umgebungsbytes. Der MSVC-CRT-Setter löscht leere Werte, daher wird der native Umgebungsblock separat wiederhergestellt. Vorhandensein leerer Werte über processValue des Helfers oder QProcessEnvironment prüfen, statt Qt-CRT-Unterscheidung leer/null zu versprechen. Dieser Helfer ist auf den seriellen Test-Hauptthread beschränkt: vor Erstellung relevanter Engines/Worker setzen, nach Zerstörung/Join wiederherstellen. Er dient weder Produktcode noch parallelen Umgebungsänderungen. ASCII, chinesische und in ANSI nicht darstellbare Zeichen erfordern Prüfung tatsächlicher Dateien am ursprünglichen Pfad, sprachwiederherstellender Kinder und Erhaltung von Fenster/Entwurf bei Fehlern. Automatische Ergebnisse bleiben von realen Einstellungsoperationen durch Menschen getrennt.

Das tatsächliche Produkt heißt auf allen Plattformen `xdzip`. Standard-Build-Einstiege:

| Plattform | Einstieg |
| --- | --- |
| macOS | `build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip` |
| Windows | `build/desktop/apps/desktop/xdzip.exe` |
| Linux | `build/desktop/apps/desktop/xdzip` |

Das Komponentenbeispiel `xdzip-ui-preview` dient nicht zur Abnahme mit echten Dateien. Nach Änderung von build-dir Pfade anpassen. Tatsächliche Build-Identität dokumentiert Version, Kanal, Commit und Änderungszustand. Module und niedrigstufige CMake-Diagnose stehen unter [Qt-Entwicklungsanwendung](../../../apps/desktop/README.md).

## UI-Schriftgenerierung

`prepare` installiert festgelegtes FontTools/Brotli in der gemeinsamen Werkzeugumgebung und hält hashgeprüfte Upstream-UI-Schriften im Cache. Jeder Desktop-Build sammelt automatisch unterstützte Sprachkataloge und festen QML-/C++-Text und erzeugt Schrift-Subsets unter `<build-directory>/generated/fonts/`. Neue Zeichen erfordern keinen Commit von Schriftbinaries. Nach Vorbereitung funktionieren Builds offline; fehlende oder beschädigte Quellcaches scheitern ausdrücklich. Direkte CMake-Builds müssen diese Umgebung über `Python3_EXECUTABLE` wählen.

Für reine HTML-Vorschau, Offline-Export, manuellen Quellimport und Schriftprüfungen siehe [gemeinsame Schriftanleitung](../../../design/assets/fonts/README.en.md). Paketierung prüft das Schriftmanifest des passenden Produktbuilds ohne Regenerierung. Fester UI-Text und Systemschriftwahl für Nutzerdaten behalten ihre bestehenden Rollen.

## Paketierung und Prüfung

`test` und `package` prüfen Build-Belege, Quellfingerabdrücke, Konfiguration, SDK-Identität und tatsächliche Produktdateien. Nach Quell- oder Konfigurationsänderungen zuerst neu bauen. Lieferbare Quelle vor Paketierung committen, dann aus diesem sauberen Commit bauen und testen. package darf Prüfungen nicht durch Ignorieren des Änderungszustands, Ändern der Artefaktidentität oder Wiederverwenden alter Berichte umgehen. Lizenzmaterial, Rust-Metadaten und tatsächliche CLI-Version müssen dieser Quelle entsprechen. Plattformwerkzeuge prüfen Nutzungs- und Überschreibgrenzen vorhandener Paketverzeichnisse. Für neu gebaute Kandidaten ein neues output-dir bevorzugen, um alte Dateien nicht zu vermischen.

Unter Windows/Linux Paketwerkzeuge vorbereiten und die gemeinsamen Stufen ausführen:

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py build
python3 scripts/desktop.py test
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

`prepare --packaging` bereitet isolierte Python-Prüfabhängigkeiten, festgelegte Lizenztexte und Schemacaches sowie Plattform-Paketmaterial vor; Windows umfasst festgelegtes CRT und Inno Setup. Entwickler installieren Systemwerkzeuge weiterhin zuerst. `package --kind runtime` bereitet nur bereitgestelltes Laufzeitverzeichnis und Auditmaterial vor. Mit demselben output-dir kann anschließend `package --kind installer` aus der geprüften Laufzeit einen Installer erzeugen; Linux erstellt zusätzlich tar.gz mit erhaltenen Ausführungsrechten. `--kind all` ist Standard und erledigt beides in einem Befehl. Windows-Paketierung nutzt Release oder RelWithDebInfo ohne Debug-Qt-Bibliotheken. Windows liefert ein bereitgestelltes Verzeichnis und nutzerbezogenen Installer; Linux liefert Laufzeitverzeichnis, tar.gz und .deb. Windows-Prüfung benötigt den Browser-Test-Executable derselben Konfiguration und Qt-Test-/Offscreen-Eingaben. Linux-Prüfung benötigt durch Buildtests erzeugtes m2-flat.zip. Testdateien gelangen nicht ins Produktverzeichnis. verify `--kind runtime` prüft die Laufzeit, Standard `--kind all` die ausgewählten Lieferungen. Gewöhnliche Prüfung installiert oder aktualisiert/rollt nicht zurück und ändert keine Nutzerdateizuordnungen.

Ein macOS-DMG benötigt einen separat gewählten Hauptbuild, der sich als Dokumenthandler-Kandidat registrieren darf, damit gewöhnliche Entwicklungs-/Testkopien nicht in „Öffnen mit“ gelangen:

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py build --build-dir build/desktop-dmg --macos-handler-rank Alternate
python3 scripts/desktop.py test --build-dir build/desktop-dmg
python3 scripts/desktop.py doctor --for-stage package --build-dir build/desktop-dmg
python3 scripts/desktop.py package --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
python3 scripts/desktop.py verify --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
```

Der Handler-Rang gewöhnlicher macOS-Builds ist standardmäßig `None` und ändert Nutzerstandards nicht; `Alternate` nur für eine langlebige Hauptanwendung oder DMG-Eingabe wählen. Lokale macOS-Paketierung unterstützt nur `all` und erstellt ein arm64-Entwicklungs-DMG mit erforderlicher Ad-hoc-Integritätssignatur. verify prüft verschobenes Cocoa/Metal und echte Extraktion. Gewöhnliche CI darf kein Mac package ausführen oder ein Anwendungspaket bereitstellen, signieren, komprimieren oder hochladen; die Ausnahme für dediziertes Release steht im [Release-Protokoll](release-protocol.md). Vollständige Grenzen und Pflege der Kopieregistrierung stehen in der [Mac-Bereitstellungsanleitung](macos-deployment.md).

Layouts und zugrunde liegende Werkzeuge stehen unter [Linux-Bereitstellung](linux-deployment.md), [Mac-DMG](macos-deployment.md) und [Installation und Dateizuordnungen](installers-associations.md). Windows-/Linux-Installer sind nicht formal signiert; macOS hat weder Developer-ID-Signatur noch Notarisierung. Installations-/Laufzeitprobleme am tatsächlichen Produkt und mit passender Plattformprüfung diagnostizieren; Betrieb unter dem SDK belegt keine Prüfung des eigenständigen Pakets.

## Beziehung zur CI

Vor Commit oder Push lokale Builds oder Prüfungen passend zum Änderungsvorschlag ausführen. Dokumentationsänderungen führen `python3 scripts/check-docs.py` aus und prüfen betroffene erzeugte Ausgaben. Rust-Änderungen prüfen Formatierung und bauen oder testen mindestens betroffene Crates. Tests isolierter Format-Backends bereiten wie oben tatsächlichen Worker und nötige Fixtures vor. C++-, Brücken- und QML-Änderungen bauen das betroffene Produkt mit `python3 scripts/desktop.py build` und führen relevante Tests aus. Nicht jede Änderung benötigt sämtliche Formatinteroperabilität oder zehn Wiederholungen der vollständigen Browser-Suite.

Beim Hinzufügen oder Ändern eines QML-Fensters `python3 scripts/check-window-contracts.py --self-test`, `python3 scripts/check-operation-components.py` und betroffene zweisprachige Layoutregressionen mit mehreren Schriftgrößen ausführen. Nach einem Build führt `python3 scripts/desktop.py test --skip-rust` vollständige Qt-Komponenten- und CTest-Prüfungen aus. Alternativ nach Einrichtung der Qt-SDK-Pfade und Offscreen-Umgebung relevante Fälle über bestehende Einstiege wie `ctest --test-dir build/desktop -R qt_task_components --output-on-failure` ausführen. Änderungen an Radio-/Checkboxen führen auch passende QAccessibleActionInterface-Regressionen wie `accessibleRadioActionsCommitSelection` und `accessibleCheckboxActionsCommitIntent` aus.

[Routine-CI](../../../.github/workflows/docs.yml) führt nur leichte statische Prüfungen aus und ersetzt lokale Produktbuilds nicht. [Foundation](../../../.github/workflows/foundation.yml) und der [Qt-Workflow](../../../.github/workflows/qt-components.yml) laufen manuell bei Bedarf. Qt nutzt dasselbe desktop.py für Builds, Qt-Tests und Windows-/Linux-Paketierung und Laufzeitprüfung. [ci-desktop.ps1](../../../scripts/ci-desktop.ps1) behält Wiederholungs-, Sicht-, Interoperabilitäts- und Installationskompatibilitätsprüfungen bei Bedarf und ist für routinemäßige lokale Prüfung nicht verpflichtend.

Historische Vollprodukt-, tatsächliche Installationsupgrade-/Rollback- und Zuordnungslebenszyklen laufen nur auf kurzlebigen Windows-/Linux-CI-Hosts oder isolierten Containern; der gemeinsame Einstieg macht sie nicht zum Standard gewöhnlichen lokalen verify. Der macOS-Job im Qt-Workflow bei Bedarf nutzt SDK-Bereitstellungsaudit und Cocoa-Start-/Dateiöffnungsprüfungen und behält nur Logs und JSON. Vollständige Prüfungen und Beleggrenzen stehen in der [CI-Politik](../../../developers/engineering/ci-policy.md).

## Datei- und Aufgabengrenzen

Öffnen dient nur der Ansicht und extrahiert nicht automatisch. Gleichnamige reguläre Dateien unterstützen Fragen, Überspringen, Umbenennen und ausdrückliches Ersetzen. Eine ganze Datei erst nach Inhaltsprüfung und Synchronisierung veröffentlichen. Abbruch vor Commit oder Schreibfehler bewahrt die alte Datei; bereits committete extrahierte Einträge bleiben erhalten. Schließen eines Aufgabenfensters verbirgt nur Details. Beenden nutzt die Entscheidung über aktive Aufgaben und die asynchrone Shutdown-Barriere.

ZIP kann relative symbolische Links und Ausführungsbits unter eingeschränkten POSIX-Bedingungen wiederherstellen, aber keine Privilegienbits, erweiterten Attribute oder ursprünglichen Änderungszeiten. Dies belegt weder App-Bundle-Installation noch Signaturprüfung. Quelle und Ziel müssen den [sicheren Extraktionsvertrag](zip-safe-extraction.md) erfüllen. Staging nach Absturz wird nur gemeldet und zur Untersuchung bewahrt; unbekannter Inhalt wird nicht automatisch wiederhergestellt oder gelöscht.

## Microsoft Store MSIX

Für eine Store-Einreichung dem [Maintainer-Verfahren](../../../developers/market/windows/build.md) folgen.

<a id="windows-msix-实验构建"></a>

## Experimenteller Windows-MSIX-Build

MSIX verwendet Windows-Release-Desktopbuild, Tests und auditiertes Laufzeitverzeichnis desselben Commits erneut. `prepare --packaging`, `build` und `test` wie oben abschließen und mit denselben Optionen ausführen:

```powershell
python scripts/desktop.py package --kind runtime
python scripts/desktop.py package --kind msix --msix-name xdzip.Development --msix-publisher 'CN=xdzip development' --msix-publisher-display-name xdzip
python scripts/desktop.py verify --kind msix
```

Die Beispielidentität dient nur lokaler Entwicklung. Paketname, Herausgeber und angezeigter Herausgebername einer formalen Store-Identität müssen Partner Center entsprechen. Ausgabe `build/package/xdzip-windows-x64.msix` ist ein unsigniertes experimentelles Paket, nicht automatisch ein verteilbares Sideload-Paket oder Store-zertifiziert. Normales Sideloading erfordert eine gerätevertrauenswürdige Signatur passend zum Herausgeber. Der eingeschränkte unsignierte Windows-11-Entwicklungsmodus verlangt zusätzlich eine bestimmte Herausgeber-OID und bei ausführbarem Inhalt typischerweise Administratorrechte; die Beispielidentität erfüllt diesen Modus nicht. Skripte erstellen keine Zertifikate, signieren, installieren oder übermitteln nicht an den Store. Das technische Minimum ist Windows 10 22H2 x64 und erweitert keine Plattformunterstützungszusagen. Version `1.0.<product build>.0` ist von der Anzeigeversion getrennt; die Store-Versionspolitik ist vor Erschöpfung der Buildnummern erneut zu prüfen.

Der Build erfordert saubere Quelle, passende Build-/Testbelege, ein vollständig auditiertes Laufzeitverzeichnis sowie MakeAppx und MakePRI des Windows SDK. Skripte prüfen Laufzeit und deren Update-, Zuordnungs- und Menüfähigkeitsmarker erneut, erzeugen Manifest und Icon-Ressourcen in separatem Staging, vergleichen jede Paketdatei und dokumentieren Hashes. Vorhandene Pakete oder Staging werden nicht überschrieben. Das Manifest deklariert vorhandene Erweiterungstypen aus dem Formatregister, einen Full-Trust-Desktopeinstieg und kanalspezifische Menüklassen.

Anwendungsicons enthalten Skalenvarianten und exakte Zielgrößen in Standard-, dunkler Hintergrund ohne Grundplatte und heller Hintergrund ohne Grundplatte. MakePRI indexiert sie in `resources.pri`, damit Windows die gewünschte Größe ohne Systemgrundplatte wählen kann. Dateizuordnungen deklarieren eigenes Formatlogo und verwenden 16–256-Pixel-PNG-Frames der vorhandenen Windows-ICOs ohne Änderung von Artwork, sichtbarer Größe, Rand oder Position. Die Ressourcenpolitik steht in [msix-icons.json](../../../specs/visual/msix-icons.json). Dies sind Ressourcen des installierten Pakets; Partner-Center-Listinglogos sind getrennt. `verify --kind msix` prüft neue Icon-Belege gegen das tatsächliche Paket einschließlich Abmessungen, vollständiger Varianten, kanonischer Formatbilder und Ressourcenindex. Änderungen erreichen installierte Nutzer erst nach Build und Auslieferung eines aktualisierten Pakets.

Das Beispiel nutzt standardmäßig `--msix-channel sideload` und registriert nur das klassische Windows-11-Menü. Für Store-Upload `package --kind msix --msix-channel store` mit exakter Partner-Center-Identität verwenden. Dies deklariert modernes und klassisches Menü; der moderne Eintrag bleibt verborgen, bis das Paket tatsächlich eine Store-Systemsignatur besitzt. Kanalmarker werden bei Paketierung ergänzt und erfordern keinen Neubau der auditierten Laufzeit. Ausgaben in getrennten Verzeichnissen halten. Siehe [Menükanalpolitik](shell-actions.md#windows-paketgrenze). Lizenzierung, tatsächliche Installation, Zuordnungen, Deinstallation, Store-Signatur und reale Abnahme auf dem Zielsystem bleiben vom Paketierungserfolg getrennt.

Das MSIX-Manifest deklariert die zehn Sprachen des gemeinsamen Sprachregisters passend zu eingebauten Katalogen; Belege dokumentieren und prüfen die Liste. Dieser MakeAppx-Ablauf wandelt Qt-Übersetzungsressourcen nicht automatisch in Paketsprachdeklarationen um. „Vom Paket unterstützte Sprachen“ in Partner Center spiegelt das hochgeladene Paket; nach Quellkorrektur muss das Entwurfspaket neu gebaut und ersetzt werden. „Zusätzliche Store-Listingsprachen“ werden unabhängig konfiguriert und ersetzen keine Manifestdeklarationen. Siehe [Microsoft-Dokumentation zu Manifestsprachen](https://learn.microsoft.com/windows/apps/design/globalizing/manage-language-and-region).

## Belege und erneute Prüfung

Betroffene Prüfungen nach [allgemeiner Verifikation](../../../developers/qa/README.md) und [Desktopprüfung bei Bedarf](../../../developers/qa/private-test-manual.md) wählen; Routinearbeit benötigt kein manuelles Abnahmeformular. Build- und Testbelege sind `desktop-build.json` und `desktop-test.json` unter build-dir. Belege für Vorbereitung, Paketierung und Prüfung sind `<stage>.json` unter output-dir und verlinken zugrunde liegende Belege. Automatisierte Belege dokumentieren tatsächliche Stufen und Quellen. Offscreen-/Software-Rendering, native Cocoa-Prüfungen und ausgeführte manuelle Prüfungen werden getrennt erfasst und ersetzen einander nicht. Vorhandene Abnahme bleibt in [#5](https://github.com/xiaodaozhi/xdzip/issues/5) und wird nicht allein wegen neuer Kandidatenlisten wiederholt.

Windows-/Linux-Artefaktlieferung enthält Quell- und Lizenzhinweise desselben Commits; Actions hält Artefakte 14 Tage. Nach Ablauf mit Identitäten und festgelegten Eingaben neu bauen. deployment-audit.json und CycloneDX-Bestände auf Quell-/Dateiebene unterstützen Herkunftsprüfung, können unvollständig bleiben und belegen keine Lizenzfreigabe. [Quelllieferung](source-delivery.md) und [automatisierter Prüfvertrag](../../../developers/engineering/candidate-verification.md) erläutern Quellneubau und Qt-Ersatz.

Builds, Screenshots, Messungen, Berichte und Pakete bleiben nur ignorierte lokale oder CI-Artefakte. Erzeugte ausstehende Formulare, Offscreen-Prüfungen oder partielle SBOMs nicht als bestandene saubere Umgebung und vollständige Release-Freigabe darstellen.

## macOS-Update-Framework

Native Builds nutzen `prepare-sparkle.py` zur Vorbereitung von `build/toolchain/Sparkle/2.10.0-xdzip-proxy.2`. Die Hauptbibliothek des Frameworks wird aus festgelegter Upstream-Quelle und Proxy-Patch kompiliert; Installationshelfer, Ressourcen und Signierwerkzeuge behalten offizielle SDK-Herkunft. Siehe [Abhängigkeitsvertrag](dependencies.md#macos-更新提供者). Gültige Build-Belege können wiederverwendet werden; ein altes offizielles SDK ersetzt das abgeleitete Framework mit Proxy-Schnittstellen nicht. CI kann ausdrücklich ein temporäres SDK-Verzeichnis wählen.

`python3 scripts/prepare-sparkle.py --output build/toolchain/Sparkle/2.10.0-xdzip-proxy.2` nutzt natives Clang aus vorhandenen Xcode Command Line Tools oder Xcode und benötigt kein separates Qt-SDK. Originale SDK- und Quellarchive liegen im Cache `build/source-cache/`; Offline-Betrieb kann `--archive` und `--source-archive` separat übergeben. Der Build führt keine Skripte aus heruntergeladener Quelle aus, ändert keine Systemproxies, greift nicht auf Produktionssignierschlüssel zu und sammelt keine Projektquelllieferungsarchive.

`check-macos-proxy.py` prüft standardmäßig nur den projekteigenen Loopback-Downloader. `--system-conflicts` ist auf kurzlebige gehostete Apple-Silicon-Runner in einem dedizierten Workflow beschränkt. Ein begrenzter Helfer bereitet Systemproxy und hosts-Einstellungen fester Testdomänen vor, stellt sie wieder her und prüft sie. Diesen Schalter lokal nicht hinzufügen.
