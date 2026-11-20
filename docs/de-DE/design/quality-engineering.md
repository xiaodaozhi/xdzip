<a name="xdzip-coding-conventions-and-quality-engineering"></a>

# Programmierkonventionen und Qualitätssicherung

[English](../../design/quality-engineering.md) | [简体中文](../../zh-CN/design/quality-engineering.md) | [繁體中文](../../zh-TW/design/quality-engineering.md) | [日本語](../../ja-JP/design/quality-engineering.md) | [한국어](../../ko-KR/design/quality-engineering.md) | [Français](../../fr-FR/design/quality-engineering.md) | Deutsch | [Español](../../es-ES/design/quality-engineering.md) | [Português (Brasil)](../../pt-BR/design/quality-engineering.md) | [Русский](../../ru-RU/design/quality-engineering.md)

<a name="xdzip-code-and-ownership"></a>

## Code und Besitz

- **Rust.** Verwenden Sie Toolchain, Lints und Lockdatei des Workspace. Prüfen Sie externe Längen, Offsets und Ganzzahlkonvertierungen. Erhalten Sie unbekannte Größen und typisierte Fehler. Dokumentieren Sie Vorbedingungen für unsafe und FFI, Besitz, Lebensdauer und Fehlerverhalten. Halten Sie kritische Abschnitte kurz und führen Sie Archiv-E/A außerhalb davon aus.
- **C++ und Qt.** Verwenden Sie C++17, RAII und klare QObject-Besitzregeln. Ändern Sie Modelle in ihrem GUI-Thread, übergeben Sie Werte mit eigenem Besitz zwischen Threads und binden Sie eingereihte Benachrichtigungen an einen noch lebenden Empfänger. Das Beenden bleibt asynchron, wenn der Empfänger die GUI zum Abarbeiten von Ereignissen benötigt. Rust bleibt für Archiv- und Dateiregeln zuständig. [Qt/Rust-Brücke](qt-rust-bridge.md)
- **QML.** Drücken Sie Zustand und Benutzerabsicht über Controller aus. Verwenden Sie gemeinsame Steuerelemente und generierte Grafik- und Textressourcen. Dateizugriffe und Archivinterpretation gehören außerhalb von Delegates. Fensteränderungen folgen Registry, nativem Verhalten und visuellen Standards. [Qt-Quick-Oberfläche](qt-quick-ui.md), [Visueller Ablauf](../visual/design-workflow.md)

Dokumentieren Sie Invarianten und den Zustand nach Fehlern. Bei Dateicode erklären Sie etwa, was eine Übernahmefreigabe schützt, welche Identitäten geprüft werden und welche Teile nach einem Fehler bereits sichtbar sein können. [Dateiveröffentlichung](file-security.md)

<a name="xdzip-verification-layers"></a>

## Prüfebenen

| Änderung | Geeignete Nachweise |
| --- | --- |
| Domäne, Abfragen und Dienst | Rust-Tests zu Identität, Grenzen, doppelten Namen, Reihenfolge, veralteten Antworten, begrenzter Ausgabe und Abbruch. |
| Formatvorgang | Eigenes Testmaterial, gegebenenfalls unabhängige Rundläufe sowie Fälle für Ablehnung, Beschädigung, nicht unterstützte Profile und Quoten. |
| Native Dateien und Aufgaben | Tatsächliche Dateiidentitäten und geschriebene Bytes, Rennen zwischen Abbruch und Übernahme, Schutz vorhandener Ausgabe und Bereinigung auf der betroffenen Plattform. |
| Desktop, Qt und Barrierefreiheit | Modell- und Controllertests, QML- und Steuerelementprüfungen, bei Bedarf native Bedienung und tatsächliche visuelle oder assistive Nachweise für das Verhalten. |
| Leistung und Auslieferung | Commitgebundene Arbeitslasten, Renderer- und Ressourcenwerte oder kanalbezogene Build-Identität, Installations- und Update-Ergebnisse sowie Abhängigkeitsnachweise. |

Der [Prüfleitfaden](../../../developers/qa/README.md) bestimmt notwendige Prüfungen. Protokollkonformität, Testmaterial oder Offscreen, native Automatisierung und manuelle Desktop-Beobachtung belegen unterschiedliche Aspekte. Vorhandene Prüfeinstiege und Liefergrenzen stehen in der [Build-Anleitung](../../engineering/developer-build.md) und der [Nachverfolgbarkeit](requirements-traceability.md).

<a name="xdzip-existing-ci-and-additional-techniques"></a>

## CI und weitere Verfahren

Die reguläre [CI](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml) prüft Dokumentation, generierte Ressourcen, Layoutverträge, Formatierung und Workflow-Struktur. Aufwendige native Rust-, Format- und Qt-Prüfungen laufen in separat ausgelösten Foundation- und Qt-Workflows. Veröffentlichungsartefakte entstehen in dafür vorgesehenen Jobs. Die [CI-Richtlinie](../../../developers/engineering/ci-policy.md) beschreibt Auslöser und Bedingungen. Welche lokalen oder gestarteten Prüfungen nötig sind, richtet sich nach den berührten Pfaden und Plattformen.

Setzen Sie Fuzzing, Sanitizer, Modellprüfung und Benchmarks ein, wenn sie die Änderung abdecken. Wählen Sie begrenztes, weiterverteilbares Testmaterial und bewahren Sie Ergebnisse getrennt von dieser Anleitung auf. Für vertrauliche Sicherheitsmeldungen gilt die [Sicherheitsrichtlinie](../../../SECURITY.md). Pläne und Prüfbeispiele gelten weder als angesetzte Läufe noch als Abnahme.

<a name="xdzip-maintaining-the-design"></a>

## Entwurf aktuell halten

Ändert die Implementierung Identitäten, Befehle, Fähigkeiten, Aufgaben oder Schreibgrenzen, aktualisieren Sie Entwurf, technische Referenz und Abnahmeverweise. Halten Sie alle Sprachfassungen auf demselben Stand. [Architekturentscheidungen](architecture-decisions.md) begründen die Wahl, der [Leistungsentwurf](performance.md) gibt Messregeln vor. Erfassen Sie tatsächliche Ergebnisse und Einschränkungen bei der betreffenden Änderung.
