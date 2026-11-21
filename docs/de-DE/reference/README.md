<a name="xdzip-product-design-and-technical-reference"></a>

# Produktdesign und technische Referenz

[English](../../reference/README.md) | [简体中文](../../zh-CN/reference/README.md) | [繁體中文](../../zh-TW/reference/README.md) | [日本語](../../ja-JP/reference/README.md) | [한국어](../../ko-KR/reference/README.md) | [Français](../../fr-FR/reference/README.md) | Deutsch | [Español](../../es-ES/reference/README.md) | [Português (Brasil)](../../pt-BR/reference/README.md) | [Русский](../../ru-RU/reference/README.md)

Die [Systemarchitektur](../design/system-architecture.md) zeigt die Ausführungspfade von Desktop und CLI. Die folgenden Abschnitte führen zum Vertrag für das jeweilige Verhalten. Die [Formattabelle](../../../README.de-DE.md#xdzip-format-support) zeigt die unterstützten Vorgänge. Bedienung und Einrichtung erläutern das [Benutzerhandbuch](../user-guide/README.md), die [CLI-Beispiele](../integration/README.md) und der [Leitfaden zum Mitwirken](../contributing/README.md).

<a name="xdzip-architecture-and-interfaces"></a>

## Architektur und Schnittstellen

- [Entwurfsübersicht](../design/index.md)
- [Produktanforderungen und Vorgangsgrenzen](../design/product-requirements.md)
- [Domänenmodell, Eintragsidentität und Quellversionen](../design/domain-data.md)
- [Qt/Rust-Brücke für Befehle und Ereignisse](../design/qt-rust-bridge.md)
- [Aufgabenplanung, Fortschritt, Abbruch und Beenden](../design/task-scheduling.md)
- [Leistungsbudgets und Prüfung](../design/performance.md)
- [Begriffe und Implementierungsgrenzen](../design/glossary.md)
- [Nachverfolgbarkeit von Anforderungen und Prüfungen](../design/requirements-traceability.md)
- [Architekturentscheidungen](../design/architecture-decisions.md)
- [Vom Prototyp zum Desktop und zur Rust-Laufzeit](../design/architecture-evolution.md)
- [Repository-Struktur und Abhängigkeiten](../engineering/repository-layout.md)
- [Schnittstellenverträge und Prüfungen](../engineering/contracts.md)
- [Produktlaufzeit und Qt-Lebenszyklus](../engineering/product-runtime.md)
- [Anwendungssitzungen und Beenden](../engineering/application-sessions.md)
- [Begründung für die cxx-Brücke](../engineering/cxx-admission.md)
- [Kalender- und Kodierungsdienste des Hosts](../engineering/host-platform-services.md)

<a name="xdzip-reading-and-writing-archives"></a>

## Archive lesen und schreiben

- [Dateiberechtigungen, Zwischenspeicherung und sichere Übernahme](../design/file-security.md)
- [Schreibgeschützte Archivansicht](../engineering/readonly-browser.md)
- [Schreibgeschützte ZIP-Ansicht](../engineering/zip-readonly-browsing.md)
- [Große Archivindizes und bedarfsgesteuerte Ansicht](../engineering/large-index.md)
- [Lesen geteilter Volumes und temporäres Öffnen](../engineering/archive-opening.md)
- [Adaptive Dateinamenskodierung](../engineering/filename-encoding.md)
- [Inhaltsvorschau](../engineering/content-preview.md)
- [Metadaten-Vorprüfung vor dem Entpacken](../engineering/extraction-preflight.md)
- [Sicheres Entpacken von ZIP](../engineering/zip-safe-extraction.md)
- [Integritätsprüfung von Archiven](../engineering/archive-checking.md)
- [Einträge öffnen, Drag-and-drop und Änderungen](../engineering/archive-transfers.md)
- [Archivpasswörter und Öffnen verschlüsselter Verzeichnisse](../engineering/password-extraction.md)
- [Vertrag für die ZIP-Erstellung](../engineering/zip-creation.md)
- [ISO9660- und Joliet-Erstellung](../engineering/iso-creation.md)
- [Eigene selbstentpackende ZIP-Archive für Windows](../engineering/sfx-creation.md)

<a name="xdzip-format-support-and-implementation"></a>

## Formatunterstützung und Implementierung

- [Integrierte Fähigkeiten, Erstellungsprofile und Prüfungsumfang](../engineering/format-registry.md)
- [Formatadapter und Entwurf des externen SDK-Versuchs](../design/codec-extension-sdk.md)
- [Externer schreibgeschützter SDK-Versuch und Ausführungsumfang](../engineering/sdk-trial.md)
- [Vertrag für isolierte Formatprozesse](../engineering/native-codecs.md)
- [Eigene Rust-Formate und -Algorithmen](../../engineering/independent-rust-codecs.md)
- [Abrechnung zusätzlicher Algorithmusarbeit](../engineering/auxiliary-decoding-work.md)
- [Zulassung der Abhängigkeit für ZIP-Metadaten](../engineering/zip-metadata-admission.md)
- [Zulassung der Abhängigkeiten für ZIP-Entpacken und Ausgabe](../engineering/zip-extraction-admission.md)
- [7z-Formatquellen und eigenständige Implementierung](../engineering/sevenzip-format-sources.md)
- [Grundlagen der RAR-Implementierung und Kompatibilitätsgrenzen](../../engineering/rar-format-sources.md)
- [CAB-Formatquellen und Implementierungsgrundlagen](../engineering/cab-format-sources.md)
- [Implementierungsvertrag für TAR/GZIP](../engineering/tar-gzip.md)
- [Einzeldateiströme und ZIP-basierte Pakete](../engineering/single-file-streams.md)
- [Vertrag für LZ4-Datenströme](../engineering/lz4-streams.md)
- [Vertrag für Zstandard-Datenströme](../engineering/zstandard-streams.md)
- [Vertrag für Unix compress Z](../engineering/compress-streams.md)
- [Vertrag für gewöhnliche Brotli-Datenströme](../engineering/brotli-streams.md)
- [Datenstruktur und Algorithmen von LHA](../engineering/lha-containers.md)
- [PMA-Container und PMarc-Verfahren](../engineering/pma-streams.md)
- [Komprimierte ARJ-Datenströme und Containerzulassung](../engineering/arj-streams.md)
- [ARC-Container und Profile komprimierter Datenströme](../engineering/arc-streams.md)
- [BH-Container und unterstütztes BlakHole-Profil](../engineering/bh-containers.md)
- [Quellen und Lizenzanforderungen für ACE, ALZ und EGG](../engineering/ace-alz-egg-format-sources.md)
- [Schreibgeschützte AES-Crypt-Hüllen](../engineering/aescrypt-envelopes.md)
- [Vertrag für DEB-Container](../engineering/deb-containers.md)
- [Vertrag für ASAR-Container](../engineering/asar-containers.md)
- [Texthüllen UU, UUE und XXE](../engineering/text-envelopes.md)
- [Quellen und Lizenzen für WIM, MSI und Installer](../engineering/wim-msi-installer-format-sources.md)
- [Quellen und Lizenzen von Datenträgerabbildern und anderen Archivformaten](../engineering/disk-image-archive-format-sources.md)
- [Bandizip-Formatinventar und Machbarkeit in xdzip](../engineering/bandizip-format-assessment.md)

<a name="xdzip-desktop-interaction-and-visual-design"></a>

## Desktop-Interaktion und Gestaltung

- [Qt-Quick-Ansichten und Controller-Aufgaben](../design/qt-quick-ui.md)
- [Visuelle Desktop-Spezifikationen](../../visual/README.md)
- [UI-Designstandard](../../visual/design-standard.md)
- [Ablauf für UI-Entwurf und Überarbeitung](../visual/design-workflow.md)
- [Symbole für Dateizuordnungen](../visual/file-association-icons.md)
- [Prüfung der visuellen HTML-Übereinstimmung](../visual/html-fidelity.md)
- [Ergänzende Interaktions- und Textspezifikation](../ux/interaction-spec.md)
- [Funktionen, Komponenten und Interaktion des Hauptfensters](../ux/main-window-components.md)
- [Entwurf und Integration des Verzeichnisbaums](../ux/directory-tree.md)
- [Startseite im Hauptfenster](../ux/welcome.md)
- [Ladeanzeige der Startseite und Abschluss der Erstellung](../ux/welcome-opening.md)
- [Passworteingabe beim Entpacken](../ux/password-extraction.md)
- [Aufgabenfenster und Integritätsprüfung](../ux/task-windows.md)
- [Einstellungsfenster und Oberflächensprache](../ux/settings-localization.md)
- [Terminologie der Oberfläche in vereinfachtem Chinesisch](../ux/chinese-terminology.md)
- [Systemmenüs und Infobereich](../ux/desktop-integration.md)
- [Plattformfenster und Titelleisten](../ux/platform-windows.md)
- [Detailentwürfe und Implementierungsvertrag](../ux/hifi-state-spec.md)
- [Interaktionsprototyp v0.1 und Funktionszuordnung](../ux/prototype-spec.md)

<a name="xdzip-platform-integration-releases-and-trust"></a>

## Plattformintegration, Veröffentlichung und Vertrauen

- [Installer und Dateizuordnungen](../../engineering/installers-associations.md)
- [Systemunterstützung und Wartungsrichtlinie](../engineering/support-policy.md)
- [Dateimanager-Aktionen und eigenständiger Aufgabenhost](../engineering/shell-actions.md)
- [Vom Benutzer gestartete Updates](../engineering/software-update.md)
- [Datenlebenszyklus und Diagnose](../engineering/privacy.md)
- [Produktversion und Build-Identität](../engineering/versioning.md)
- [GitHub-Release-Protokoll v1](../engineering/release-protocol.md)
- [Lokales macOS-Entwicklungs-DMG](../engineering/macos-deployment.md)
- [Eigenständiges Linux-Laufzeitverzeichnis und Prüfung](../engineering/linux-deployment.md)
- [Prüfung der Bereitstellungsabhängigkeiten und SBOM](../engineering/deployment-audit.md)
- [Namensnennungen und Lizenzen der tatsächlich verwendeten Komponenten](../engineering/runtime-attribution.md)
- [Herkunft und Hinweise nativer Laufzeitkomponenten](../engineering/native-runtime-materials.md)
- [Prüfung der Übereinstimmung von Qt-SDK und Quellcode](../engineering/qt-sdk-correspondence.md)
- [Lizenzen für Windows-Entwicklungswerkzeuge und Laufzeit](../engineering/windows-licensing.md)
- [Lizenzentscheidung und Drittanbietermaterial](../engineering/licensing.md)
- [Zulassung von Abhängigkeiten und Format-Backends](../../engineering/dependencies.md)
- [Zugehöriger Quellcode und Materialien für den erneuten Build](../engineering/source-delivery.md)
- [ICU-73.2-Daten aus CLDR neu erstellen](../engineering/icu-cldr-rebuild.md)

<a name="xdzip-build-and-cli-reference"></a>

## Build- und CLI-Referenz

- [Build, Bereitstellung und Veröffentlichung](../design/build-release.md)
- [Programmierkonventionen und Prüfebenen](../design/quality-engineering.md)
- [Vorbereitung der Entwicklungsumgebung](../engineering/development.md)
- [Entwicklungsbuilds und Plattformprüfung](../../engineering/developer-build.md)
- [Festgelegte Toolchain und Plattformen](../engineering/toolchain.md)
- [CLI-Befehle, JSON, Installation und Aktualisierungsbelege](../engineering/command-line.md)
