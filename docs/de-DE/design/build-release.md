<a name="xdzip-build-deployment-and-release-architecture"></a>

# Architektur für Build, Bereitstellung und Veröffentlichung

[English](../../design/build-release.md) | [简体中文](../../zh-CN/design/build-release.md) | [繁體中文](../../zh-TW/design/build-release.md) | [日本語](../../ja-JP/design/build-release.md) | [한국어](../../ko-KR/design/build-release.md) | [Français](../../fr-FR/design/build-release.md) | Deutsch | [Español](../../es-ES/design/build-release.md) | [Português (Brasil)](../../pt-BR/design/build-release.md) | [Русский](../../ru-RU/design/build-release.md)

<a name="xdzip-build-and-package-inputs"></a>

## Eingaben für Build und Paketierung

CMake baut die Qt-Desktopanwendung, C++-Adapter und nativen Hilfsprogramme. Cargo baut die Rust-Laufzeit und erzeugt die eingebundene CXX-Brücke. Die eigenständige CLI hat einen eigenen Build-Einstiegspunkt und kommt ohne Qt aus. `scripts/desktop.py` und `scripts/cli.py` koordinieren Build und Tests. Der Quellcode legt Rust 1.99.0 und das Qt-SDK 6.8.2 fest. Beim Vergleich von Ergebnissen gehören Build-Identität, Zielarchitektur, Sperrdateien und Herkunft der Toolchain zusammen. [Toolchain](../engineering/toolchain.md), [Build-Anleitung](../../engineering/developer-build.md)

Die Desktopbereitstellung umfasst die Qt-Bibliotheken, QML-Module, Plattformressourcen und mitgelieferten Hilfsprogramme, die das jeweilige Paket braucht. Der interne Bildprozess nutzt Qt zur Bilddekodierung und zur Darstellung statischer SVGs. Die CLI-Distribution enthält den benötigten Archivprozess. Build-Profile, Listen zulässiger Paketdateien und Bereitstellungsprüfungen stehen in den Skripten und Plattformverträgen. Für die installierten Abhängigkeiten und die Gesamtgröße zählt das vollständige Paket. [Bereitstellungs- und Abhängigkeitsprüfung](../engineering/deployment-audit.md), [CLI-Distribution](../engineering/command-line.md)

<a name="xdzip-current-distribution-channels"></a>

## Aktuelle Vertriebskanäle

| Ziel | Paket und Vertrauensgrenze |
| --- | --- |
| macOS Apple Silicon | GitHub stellt ein DMG und ein ZIP mit dem App-Bundle bereit. Die Mindestversion ist macOS 14. Das Paket nutzt eine Ad-hoc-Signatur zur Integritätsprüfung. Installation und Updates sind in der Plattformanleitung beschrieben. |
| Windows 11 x86_64 | GitHub stellt ein unsigniertes Installationsprogramm pro Benutzer und ein portables ZIP bereit. Die MSIX-Identität, Signatur und Updates der Store-Ausgabe liegen beim Microsoft Store. |
| Ubuntu 24.04 LTS x86_64 | GitHub stellt ein Archiv des Laufzeitverzeichnisses und ein DEB bereit. Die unterstützte Basis umfasst X11 und glibc 2.39 oder neuer. |
| Eigenständige CLI | Hat eigene Paket- und Update-Identitäten. Für die mit der Desktopanwendung installierte CLI gelten die Integrations- und Übergaberegeln ihres Pakets. |

Die [Support-Richtlinie](../engineering/support-policy.md) legt unterstützte Betriebssysteme und Architekturen fest. Verfügbare Dateien stehen in der [Projekt-README](../../../README.de-DE.md) und den zugehörigen Versionshinweisen. Grenzen der Dateizuordnung und Integration beschreibt die [Installationsanleitung](../../engineering/installers-associations.md).

<a name="xdzip-routine-checks-native-builds-and-release"></a>

## Reguläre Prüfungen, native Builds und Veröffentlichungen

Der reguläre [CI-Workflow](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml) führt schnelle Prüfungen aus. Die nativen Foundation- und Qt-Aufgaben haben getrennte manuelle Workflows. Die reguläre CI auf Apple Silicon baut und testet die App und prüft ihren Start, ohne eine herunterladbare App bereitzustellen. Der dedizierte Release-Workflow hat eigene autorisierte Schritte für Bereitstellung, Signierung, Paketierung und Veröffentlichung. [Veröffentlichungsprotokoll](../engineering/release-protocol.md) und [CI-Richtlinie](../../../developers/engineering/ci-policy.md) definieren die Freigabebedingungen und Nachweise.

Release-Manifeste, Prüfsummen, Inventare, SBOM und Lizenzunterlagen beschreiben die ausgelieferten Artefakte. Der Release-Workflow erzeugt keine Projektquellcode-ZIPs und lädt keine hoch. GitHub erzeugt seine eigenen Tag-Snapshots separat. Die Verpflichtungen ergeben sich aus den tatsächlich ausgelieferten Komponenten, dem [Lizenzverzeichnis](../engineering/licensing.md) und der [Richtlinie zur Quellcodebereitstellung](../../engineering/source-delivery.md).

<a name="xdzip-update-and-future-distribution"></a>

## Updates und künftige Distribution

Die Auswahl und Prüfung von Updates erfolgt unabhängig von Archivaufgaben. Unterstützte Installationen folgen dem Updatevertrag für Vertrauensmetadaten, geprüfte Dateien, Installationsidentität und die Übergabe bei laufenden Aufgaben. Windows und Linux verwenden das jeweilige Hilfsprogramm, macOS verwendet Sparkle und die Store-Ausgabe wird vom Microsoft Store aktualisiert. Portable und Entwicklungskonfigurationen haben eigene Grenzen. [Softwareupdates](../../engineering/software-update.md)

Weitere Architekturen, Betriebssystem-Mindestversionen, Paketformate oder Stores erfordern eigene Build- und Abnahmenachweise. Frühere Distributionsentscheidungen und die Bedingungen für ihre Neubewertung stehen in den [Architekturentscheidungsprotokollen](architecture-decisions.md).
