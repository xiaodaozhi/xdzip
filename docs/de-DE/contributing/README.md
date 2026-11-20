# XDZip bauen und mitentwickeln

[English](../../contributing/README.md) | [简体中文](../../zh-CN/contributing/README.md) | [繁體中文](../../zh-TW/contributing/README.md) | [日本語](../../ja-JP/contributing/README.md) | [한국어](../../ko-KR/contributing/README.md) | [Français](../../fr-FR/contributing/README.md) | Deutsch | [Español](../../es-ES/contributing/README.md) | [Português (Brasil)](../../pt-BR/contributing/README.md) | [Русский](../../ru-RU/contributing/README.md)

Lesen Sie die [Beitragsrichtlinien](../../../CONTRIBUTING.md) und suchen oder erstellen Sie ein Issue zur Änderung. Beschreiben Sie das erwartete Verhalten, die betroffenen Plattformen und ein kleines nachprüfbares Beispiel. Verwenden Sie projekteigene oder weiterverteilbare Testdateien. Die [Repository-Struktur](../engineering/repository-layout.md) zeigt, wo die einzelnen Teile der Implementierung liegen.

<a name="xdzip-build-the-part-you-need"></a>

## Den benötigten Teil bauen

Führen Sie die Befehle im Repository-Stammverzeichnis aus. Verwenden Sie unter Windows `python` in einer x64-MSVC-Entwicklungsumgebung. Verwenden Sie unter macOS ein natives Apple-Silicon-Terminal und die Xcode Command Line Tools. Folgen Sie unter Ubuntu der Abhängigkeitsliste für die Plattform. Die [Build-Anleitung](../../engineering/developer-build.md#environment-and-entry-points) nennt die Voraussetzungen und die im Repository festgelegten Rust- und Qt-Toolchains.

Für den ersten Desktop-Build:

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

Die Setup-Optionen verwenden vorhandenes rustup für die festgelegte Toolchain und laden bei Bedarf das festgelegte Qt SDK herunter. Verwenden Sie die bereits vorbereiteten Abhängigkeiten in `build/toolchain/` erneut. Ist ein kompatibles Qt SDK außerhalb des Standardpfads installiert, geben Sie bei den betroffenen Schritten dasselbe `--qt-prefix` an. Wenn die Änderung Desktop-Tests erfordert, führen Sie `python3 scripts/desktop.py test` mit einem abgeschlossenen Build derselben Konfiguration aus. Die [Vorbereitung der Testeingaben](../../engineering/developer-build.md) beschreibt die vollständigen Worker-Tests.

Für die eigenständige CLI bauen und testen Sie die CLI und den dazugehörigen Worker:

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
```

Die Ausgabe liegt standardmäßig unter `build/cli`. Der Build benötigt die festgelegte Rust-Toolchain, CMake und die C++-Toolchain der Plattform. Unter Ubuntu kommen `libssl-dev` und `pkg-config` hinzu. Details nach Plattform stehen in der [CLI-Build-Spezifikation](../engineering/command-line.md#xdzip-build-and-command-installation).

<a name="xdzip-verify-the-affected-behavior"></a>

## Betroffenes Verhalten prüfen

Verwenden Sie nach Möglichkeit einen passenden vorhandenen Build. Bei reinen Dokumentationsänderungen beginnen Sie mit den folgenden Prüfungen und kontrollieren die Ausgabe betroffener Generatoren. Bauen Sie den gesamten Desktop nur, wenn die Änderung oder die erforderlichen Prüfungen es verlangen.

```sh
python3 scripts/check-docs.py
git diff --check
```

| Änderung | Code und Spezifikation | Ausführen oder überprüfen |
| --- | --- | --- |
| Dokumentation und Beispiele | Aktualisieren Sie Übersetzungen, Links und Anker. Das README im Stammverzeichnis und `docs/` liegen in zehn Sprachfassungen vor. Die Anleitungen in `developers/` verwenden US-Englisch und vereinfachtes Chinesisch. Prüfen Sie geänderte Befehle anhand der jeweiligen Spezifikation mit einer kleinen eigenen Testdatei. | Führen Sie die obige Dokumentationsprüfung aus. Öffnen Sie geänderte Abschnittslinks und prüfen Sie die erzeugte Ausgabe, wenn die Seite als Generatoreingabe dient. |
| CLI | Beginnen Sie in `crates/xdzip-cli` und folgen Sie der gemeinsamen Rust-Laufzeit. [CLI-Spezifikation](../engineering/command-line.md) | Führen Sie die eigenständigen CLI-Prüfungen mit Ihrem Build aus. Prüfen Sie bei Befehls- oder Ausgabeänderungen stdout, stderr, Rückgabecode und geschriebene Dateien, einschließlich eines passenden Fehlerszenarios. |
| Rust-Domänenlogik, Formate oder Archivaufgaben | Beginnen Sie bei `xdzip-domain`, dem betreffenden Codec, `xdzip-runtime` oder `xdzip-io`. Folgen Sie bei Desktop-Befehlen `BrowserController` und `RuntimeConnection`. [Formatregister](../engineering/format-registry.md), [Laufzeit](../engineering/product-runtime.md), [Dateisicherheit](../design/file-security.md) | Führen Sie `cargo fmt --all -- --check` und die betroffenen Crate-Tests aus. Prüfen Sie den tatsächlichen Laufzeitpfad und relevante Fehler, Abbrüche oder endgültige Schreibvorgänge. Nutzt der Pfad den nativen Format-Worker, verwenden Sie die [Einstiege für Worker-Tests](../../../developers/qa/runtime-worker-tests.md). Nach Änderungen am Register führen Sie `python3 scripts/generate-format-registry.py` aus, dann denselben Befehl mit `--check` und anschließend `python3 scripts/test-format-registry.py`. |
| QML-Fenster, Steuerelemente oder Lokalisierung | Beginnen Sie in `apps/desktop/ui`, `resources/i18n` und `specs/visual`. Lesen Sie vor Fensteränderungen die [visuelle Spezifikation](../../visual/design-standard.md) und den [Designablauf](../visual/design-workflow.md) und tragen Sie das Fenster in das [Fensterregister](../../../specs/visual/window-registry.json) ein. | Führen Sie für Fensteränderungen `python3 scripts/check-window-contracts.py --self-test` und `python3 scripts/check-operation-components.py` sowie die betroffenen zweisprachigen Layouttests mit mehreren Schriftgrößen aus. Für Lokalisierung verwenden Sie `python3 scripts/check-ui-i18n.py`. Prüfen Sie bei Bedarf die native Interaktion oder Darstellung auf der betroffenen Plattform. |
| Installation, Systemintegration oder Updates | Verwenden Sie den [Build- und Bereitstellungsleitfaden](../../engineering/developer-build.md), die [Dateizuordnungen](../../engineering/installers-associations.md) und die [Update-Spezifikation](../../engineering/software-update.md). | Prüfen Sie den betroffenen Installations- oder Registrierungspfad, die Identität des tatsächlichen Pakets, Fehler und Wiederherstellung. Die [CI-Richtlinie](../../../developers/engineering/ci-policy.md) hilft bei der Wahl des passenden Plattform-Workflows. |

<a name="xdzip-describe-the-evidence"></a>

## Prüfnachweise beschreiben

Nennen Sie im PR die Verhaltensänderung, die getesteten Befehle und Plattformen und die noch ausstehenden Prüfungen. Geben Sie separat an, welche Prüfungen ohne sichtbare Oberfläche, auf dem nativen Desktop und mit einem echten Installationspaket erfolgten. Der [Prüfleitfaden](../../../developers/qa/README.md) erklärt die Aussagekraft. Schnittstellenspezifikationen stehen in der [technischen Referenz](../reference/README.md), Veröffentlichungsabläufe im [Handbuch für Projektbetreuer](../../../developers/README.md).
