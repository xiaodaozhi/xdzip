<a name="xdzip-linux-standalone-runtime-directory-and-verification"></a>

# Eigenständiges Linux-Laufzeitverzeichnis und Prüfung

[English](../../engineering/linux-deployment.md) | [简体中文](../../zh-CN/engineering/linux-deployment.md) | [繁體中文](../../zh-TW/engineering/linux-deployment.md) | [日本語](../../ja-JP/engineering/linux-deployment.md) | [한국어](../../ko-KR/engineering/linux-deployment.md) | [Français](../../fr-FR/engineering/linux-deployment.md) | Deutsch | [Español](../../es-ES/engineering/linux-deployment.md) | [Português (Brasil)](../../pt-BR/engineering/linux-deployment.md) | [Русский](../../ru-RU/engineering/linux-deployment.md)

Die aktuelle Entwicklungsbasis für Deployment ist Ubuntu 24.04, x86_64 und X11. Ziel ist die Ausführung des aktuellen Produkts ohne Entwicklungs-Qt-SDK; andere Distributionen, ältere glibc, Wayland oder vollständige Installer-Unterstützung werden damit nicht versprochen. macOS-CI behält native Build-, Test- und Startprüfung. Lokale Entwicklungs-DMGs folgen der [Mac-Deployment-Anleitung](macos-deployment.md) und sind von diesem Ablauf nicht betroffen.

<a name="xdzip-layout-and-generation"></a>

## Aufbau und Erzeugung

`scripts/deploy_linux.py` akzeptiert nur vertrauenswürdige projekteigene ELF-Dateien und ein ausdrücklich angegebenes Qt-6.8.2-SDK und erstellt ein neues Verzeichnis unter build/ oder dist/:

- `xdzip` ist der Starter und verwendet lokale Bibliotheken, Plugins und QML ohne Übernahme der Suchpfade des Entwicklungs-SDK.
- `bin/xdzip` ist das Produkt; das benachbarte `qt.conf` verwendet relative Qt-Pfade.
- `lib/` enthält tatsächlich importierte gemeinsame Qt-/SDK-Bibliotheken; `qml/` behält den Basic-Controls-Stil und notwendige Implementierungsuntermodule. Andere Controls-Stile, NativeStyle, Effects, stilspezifische Dialogvarianten, Designerdateien und QML-Typmetadaten werden vor Sammlung nativer Abhängigkeiten entfernt, sodass deren exklusive Bibliotheken nicht bereitgestellt werden.
- `plugins/` enthält xcb, Offscreen-Prüfung, Bild- und Symbolplugins, Plattformeingabe, xcb-Grafikintegration und das erforderliche `tls/libqopensslbackend.so`; ohne OpenSSL-Backend schlägt das Deployment fehl. EGLFS-, Framebuffer- und Wayland-Plattformplugins werden nicht als geprüft geliefert beschrieben.
- `translations/` behält Übersetzungen der Qt-Standardsteuerung; `share/notices/` liefert aktuelle Hinweise und Materialien zu Quellabhängigkeiten.
- `share/deployment.json` erfasst QML, Systemlaufzeitabhängigkeiten, kopierte SDK-Bibliotheken und Grenzen; README.md beschreibt die Verwendung.

Der Scanner verwendet [qmlimportscanner](https://doc.qt.io/qt-6.8/qtqml-tooling-qmlimportscanner.html) aus dem fixierten Qt. Xdzip.Settings und Xdzip.Native sind eingebaute registrierte Module; statische Entwicklungsbibliotheken werden nicht kopiert. Fehlende gewöhnliche Abhängigkeiten, QML-Pfade außerhalb des SDK, Konflikte gleichnamiger Bibliotheken und unbekannte externe Bibliotheken führen zum Fehler. Die rekursive Sammlung nativer Imports kopiert nur SDK-Bibliotheken, nicht glibc oder Grafiktreiber des Hosts; Systemabhängigkeiten werden nach tatsächlicher Debian-Paketzugehörigkeit aufgeführt.

Vor dem Entfernen prüft das Ausdünnen den kopierten QML-Baum, erforderliche Basic-Plugins, beibehaltene starke Imports und bekannte Stilstrukturen. Unerwartete Links, unbekannte Stile oder Dialogselektoren und starke Abhängigkeiten zu ausgelassenen Modulen führen zum Fehler. Standarddialoge, Eingabemethoden, Barrierefreiheit, Bildplugins, TLS und Systemübersetzungen behalten ihre bisherigen Regeln. `share/deployment.json` erfasst entfernte QML-Dateien und Bytezahlen. Das private `cli/` teilt `bin/xdzip-native-codec` über seinen festen Kontextpfad; eigenständige CLI-Pakete bleiben unabhängig.

HTTPS benötigt zusätzlich Ubuntus `libssl3t64` und `ca-certificates`. Systempaketzugehörigkeit von `libssl.so.3` und `libcrypto.so.3` sowie einen nicht leeren System-CA-Vertrauensspeicher prüfen. Das Qt-OpenSSL-Backend kann diese Bibliotheken zur Laufzeit laden, daher belegt ldd allein keine vollständigen Abhängigkeiten. Das Deployment-Manifest erfasst installierte Binärpaketversionen, Quellpakete und deren Versionen. Installer deklarieren beide Abhängigkeiten; die SBOM tatsächlicher Dateien erfasst sie als benötigte OS-Pakete, ohne Systembibliotheken in die App zu bündeln.

patchelf setzt auf jedem ELF einen relativen `$ORIGIN`-RPATH und berechnet für jedes Plugin die relative Entfernung zu lib/. Qt-Pfadümleitung folgt den [offiziellen qt.conf-Regeln](https://doc.qt.io/qt-6.8/qt-conf.html), sodass das gesamte Verzeichnis verschoben werden kann. Bibliotheken bleiben dynamisch gelinkt und austauschbar; kompatibler Qt-Austausch wird nicht eingeschränkt. Allgemeine Grenzen stehen unter [Qt-Linux-Deployment](https://doc.qt.io/qt-6.8/linux-deployment.html).

Nach relativer RPATH-Umschreibung entfernt das Deployment unnötige native Symboltabellen und prüft, dass alle DT_NEEDED- und RPATH/RUNPATH-Einträge unverändert bleiben. Für jedes ELF werden Werkzeugversion, Hashes und Bytezahlen vor und nachher in `symbol_stripping` erfasst; SDK-Eingaben und ursprüngliche Build-Binärdateien bleiben unangetastet. Für eine Diagnosekopie akzeptiert das zugrunde liegende Deployment-Skript `--keep-symbols`. Endgültige SDK-Zuordnung und Anwendungshashes beschreiben die transformierten Dateien. Dies verringert Paketgröße, belegt aber keinen gemessenen Speicher- oder Durchsatzgewinn.

Zur regelmäßigen Erzeugung und Prüfung den [einheitlichen Build-Einstieg](../engineering/developer-build.md) stufenweise verwenden. Zuerst Build, Test und Paketvorbereitung mit derselben Konfiguration abschließen, danach ausführen:

```sh
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

Das Standard-Buildverzeichnis ist build/desktop, die Auslieferung build/package und das Qt-SDK `build/toolchain/Qt/6.8.2/gcc_64`; für andere Orte gemeinsame Optionen bei jeder Stufe angeben. `package --kind runtime` erzeugt Laufzeitverzeichnis und Auditmaterialien; anschließend erzeugt `package --kind installer` mit demselben output-dir tar.gz und .deb. Standardmäßig führt `all` beide Schritte in dieser Reihenfolge aus. Die zugrunde liegenden Skripte [deploy_linux.py](../../../scripts/deploy_linux.py), [package_linux.py](../../../scripts/package_linux.py) und [verify_linux_runtime.py](../../../scripts/verify_linux_runtime.py) behalten die Plattformlogik; zur Fehlersuche deren `--help` verwenden.

Deployment läuft nur auf nativem Linux x86_64 und überschreibt keine vorhandenen Verzeichnisse. Die Paketvorprüfung kontrolliert qmlimportscanner, ldd, patchelf, dpkg-query und das für Installer nötige dpkg-deb; diese Build-Werkzeuge gelangen nicht ins Produkt. Container-Laufzeitprüfung benötigt zusätzlich Docker und einen erreichbaren Daemon; Builds sind auf das fixierte Ubuntu-Image und erfasste OS-Pakete begrenzt. Fehlendes Docker blockiert gewöhnliche Builds und Tests nicht, verify schlägt jedoch ausdrücklich fehl; eine nicht ausgeführte Isolationsprüfung wird nicht als bestanden erfasst. Gewöhnliches verify installiert kein .deb auf dem Host und ändert keine Dateizuordnungen; der Installationslebenszyklus bleibt eine Prüfung in einer wegwerfbaren CI-Umgebung.

<a name="xdzip-experimental-strict-snap"></a>

## Experimenteller strikter Snap

Der strikte `core24`-Snap ist eine separate experimentelle Auslieferung. Eine eingeschränkte Anwendung übernimmt die XKB-Konfiguration des Hosts nicht nach `$SNAP/usr/share/X11/xkb`. Setzt der Starter `XKB_CONFIG_ROOT` auf diesen Pfad, muss der Snap den vollständigen XKB-Baum aus der fixierten Ubuntu-Build-Eingabe enthalten. Fehlt er, kann das Qt-Wayland-Plugin laden und sich mit dem Compositor verbinden, aber beim Erstellen des Tastaturkontexts mit einem `xkbcommon`-Include-Pfadfehler scheitern. Der Payload muss den Copyright-Hinweis von `xkb-data` enthalten; der Kandidatenbeleg erfasst Paketversion, deterministischen Verzeichnis-Hash, Dateizahl und Hash des Hinweises. Den fertigen Snap nach dem Entpacken prüfen und anschließend Offscreen- und Wayland-Smoke-Tests getrennt ausführen. Diese Prüfungen ersetzen keine Installations- und Interaktionsprüfung in einer tatsächlichen strikten Snap-Desktop-Sitzung.

<a name="xdzip-automated-evidence-without-the-sdk"></a>

## Automatisierte Nachweise ohne SDK

Der Container verwendet Ubuntu 24.04 mit fixiertem Digest und installiert tatsächliche OS-Laufzeitabhängigkeiten sowie Xvfb-, Schrift- und Locale-Testunterstützung, aber weder Qt, Rust noch einen Compiler; deren Abwesenheit wird erneut geprüft. Er bindet weder das Entwicklungs-SDK noch den vollständigen Quellbaum ein, sondern nur das schreibgeschützte Produktverzeichnis, ein separates projekteigenes ZIP mit 100.000 Einträgen und Prüfscripts. Die Archiv-Laufzeitprüfung deaktiviert Netzwerk und Rechteausweitung; nur das temporäre Konfigurationsverzeichnis ist schreibbar.

Die Anwendung unter einem neuen Pfad mit Leerzeichen einbinden. Nach Entfernen der Entwicklungssuchvariablen mit dem Starter prüfen:

1. Tatsächliches `--version` läuft.
2. Ein echtes QML-Fenster startet unter X11 und schließt sicher.
3. Die native Produktsonde absolviert erstes Bild, Lesen von 100.000 Einträgen, Abbruch einer aktiven Anfrage und Beenden; Zahlen und Abbruchergebnisse erfüllen den Vertrag.
4. Nach Entfernen der Host-Entwicklungsvariablen alle paketierten ELFs im Deployment-Modus ohne SDK-Rückfall prüfen. Die SBOM tatsächlicher Dateien behält dieselbe Versions-/Commit-Identität und besteht das offizielle Schema.
5. In einem neuen Prozess desselben SDK-freien Images Netzwerk aktivieren, Produkt-`--update-probe` mit isoliertem `XDZIP_SETTINGS_PATH` ausführen und tatsächlich GitHub per HTTPS anfragen. Es muss ein verfügbares OpenSSL-Backend melden und die Updateprüfung abschließen; `.https.json` und `.https.log` getrennt aufbewahren. Netzwerk-, Zertifikat- oder TLS-Fehler blockieren die Prüfung.

Die HTTPS-Sonde erfasst OpenSSL-Backend, System-CA-Datei und tatsächliche GitHub-Anfrageergebnisse getrennt. Eine erfolgreiche reale Anfrage allein belegt weder Signaturprüfung noch native Updateinstallation. Signaturanalyse und SDK-freie Updateabläufe prüft der [separate Update-Regressionsworkflow](../../../.github/workflows/update-regression.yml); diese Nachweise ersetzen einander nicht.

Rohprotokolle und strukturierte Berichte jedes Sondenversuchs als `.https.attempt-N.log` und `.https.attempt-N.json` sowie `.https.log` für den letzten Versuch bewahren. Fehlende oder ungültige Berichte, Zeitüberschreitungen und Prozessausstiege bleiben Fehler. Nur wiederholen, wenn der Produktcontroller eine Ratenbegrenzung der anonymen GitHub-Release-Liste bestätigt und Status, Anfragephase, Antwortheader und Diagnosen vollständig sind. Bis zum Servertermin warten oder begrenztes Backoff verwenden, mit höchstens drei Versuchen und insgesamt 3.700 Sekunden Wartezeit wie bei der macOS-Abnahmesonde. Ein Termin jenseits des verbleibenden Budgets, eine unterbrochene Wartezeit oder TLS-, Netzwerk- oder andere Fehler führen sofort zum Fehlschlag. Zum Bestehen muss die Sonde eine echte Updateanfrage abschließen. Jede Warteentscheidung und tatsächliche Dauer in `.https-waits.json` erfassen.

Vor erneuter Verwendung eines Ausgabepfads vorherigen Bericht und Logs in ein nach dem Bericht benanntes `.history`-Verzeichnis im selben übergeordneten Verzeichnis verschieben, mit eigenem Unterverzeichnis je Lauf. Schlägt ein neuer Lauf unterwegs fehl, bleibt der alte Erfolgsbericht nicht am aktuellen Ausgabepfad und kann nicht verwechselt werden; historische Artefakte bleiben einzeln prüfbar.

linux-runtime.json erfasst tatsächliche Ereignisse und ausführbare SHA-256. Xvfb-/Qt-Quick-Softwarerendering belegt nur den Laufzeitablauf, nicht physische GPU, chinesische Kandidatenfenster, echte Screenreader oder eine vollständige Desktopinstallation.

<a name="xdzip-download-delivery-and-remaining-gates"></a>

## Download und verbleibende Bedingungen

CI erzeugt xdzip-linux-x86_64.tar.gz und erfasst SHA-256 sowie ausführbare Hashes. tar.gz bewahrt Ausführungsrechte und verhindert deren Verlust beim Entpacken eines Actions-ZIP. Archivpfade, UID/GID und Zeitstempel werden ohne private Verzeichnisse normalisiert. Nach dem Download entpacken und `./xdzip` im Verzeichnis ausführen; nicht nur bin/xdzip verschieben. Das rohe linux/-Verzeichnis wird nicht erneut hochgeladen.

Das hier erzeugte Laufzeitverzeichnis `build/package`, tar.gz und .deb sind experimentelle Entwicklungsartefakte und installieren nichts oder ändern Standardzuordnungen nicht automatisch. Offizielle Ubuntu-tar.gz und .deb für Release erzeugt der dedizierte Release-Workflow gemäß [Veröffentlichungsprotokoll](release-protocol.md). Der Plattformumfang folgt der [Unterstützungsrichtlinie](support-policy.md); andere Distributionen und Desktops, Wayland und Erweiterungen benötigen eigene Issues. Die bestehende Desktop-Abnahme bleibt in [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5). Für spätere Änderungen betroffene Prüfungen mit echtem IME/Screenreader und sauberer Installation, Upgrade und Deinstallation wiederholen; der reine Neubau eines Verzeichnisses verlangt keine Wiederholung bestehender Abnahme. Die historische Komponentenprüfung zu `v0.1.0` steht in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) und [#53](https://github.com/xiaodaozhi/xdzip/issues/53); erfolgreiche Verzeichnisausführung erfüllt zukünftige Release-Bedingungen nicht automatisch.

[Installation und Zuordnungen](installers-associations.md) beschreibt Ubuntu-.deb-Installation, Upgrade, Rollback und Deinstallation, Laufzeitsperren und Desktop-MIME-Automatisierung. Ein bestandener Laufzeitcontainer ersetzt weder echte Desktopprüfungen noch die Lizenzprüfung für Veröffentlichungen.
