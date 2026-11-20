<a name="xdzip-system-support-and-maintenance-policy"></a>
# Richtlinie für Systemunterstützung und Wartung

[English](../../engineering/support-policy.md) | [简体中文](../../zh-CN/engineering/support-policy.md) | [繁體中文](../../zh-TW/engineering/support-policy.md) | [日本語](../../ja-JP/engineering/support-policy.md) | [한국어](../../ko-KR/engineering/support-policy.md) | [Français](../../fr-FR/engineering/support-policy.md) | Deutsch | [Español](../../es-ES/engineering/support-policy.md) | [Português (Brasil)](../../pt-BR/engineering/support-policy.md) | [Русский](../../ru-RU/engineering/support-policy.md)

Der Projektinhaber genehmigte den ursprünglichen Supportumfang am 2026-10-19. Die Version `v0.1.0` wurde am 2026-11-06 öffentlich veröffentlicht. Aktuelle Versionen und Downloads stehen auf der [Veröffentlichungsseite](https://github.com/xiaodaozhi/xdzip/releases). Diese Richtlinie legt den Supportumfang fest. Die Prüfungen der Mindestversion bleiben gültig. CI-Ergebnisse und die Abnahme durch den Nutzer gelten für die jeweils geprüfte Umgebung. Der Nutzer hat die manuelle Prüfung abgenommen.

<a name="xdzip-initial-support-scope"></a>
## Ursprünglicher Supportumfang

| Plattform | Ursprünglicher Umfang | Prüfungen und Grenzen |
| --- | --- | --- |
| macOS | macOS 14 oder neuer, Apple Silicon arm64 | Bereitstellungsziel 14.0, native CI und lokal erzeugtes Entwicklungs-DMG. Kein Intel- oder Universalpaket. Keine Zusage für alle künftigen Systemversionen. |
| Windows | Von Microsoft weiterhin gewartete x86_64-Versionen von Windows 11 | MSVC-Build, Installation pro Benutzer sowie tatsächliche Installation, Upgrade, Rückkehr zur Vorversion und Deinstallation in der CI. Tests auf Windows Server decken nicht jede Windows-11-Version ab. Kein ARM64-Paket. |
| Linux | Ubuntu 24.04 LTS x86_64, X11, glibc 2.39 oder neuer | Prüfung des aktuellen .deb-Pakets und der isolierten Laufzeitumgebung ohne Qt-SDK. Andere Distributionen, ältere glibc-Versionen, Wayland und ARM64 gehören nicht zum Umfang. |

Die macOS-Mindestversion folgt dem aktuellen Bereitstellungsziel. Version 13 aus älteren Entwürfen wurde nicht geprüft. Der Standardsupport für Windows 10 endete am 2025-10-14. Windows 10 gehört nicht zum ursprünglichen Umfang. LTSC und ESU sind nicht automatisch eingeschlossen. Siehe [Microsofts Ankündigung zum Produktlebenszyklus](https://learn.microsoft.com/en-us/lifecycle/announcements/windows-10-end-of-support). Die [von Qt 6.8 unterstützten Plattformen](https://doc.qt.io/qt-6.8/supported-platforms.html) beschreiben den Umfang der Abhängigkeit. xdzip prüft seine Plattformen eigenständig.

<a name="xdzip-maintenance"></a>
## Wartung

Dieses persönliche Projekt wird im Rahmen der verfügbaren Möglichkeiten gepflegt. Es gibt keinen kostenpflichtigen Dienst, keine feste Reaktionszeit und keine Sicherheitszusagen über die geleistete Arbeit hinaus. Vorrang hat die neueste öffentliche stabile Version. Entwicklungsversionen können Änderungen an der Kompatibilität mitbringen. Die Versionshinweise nennen bekannte Probleme und Möglichkeiten zur Rückkehr zur Vorversion. Bei Sicherheitsproblemen können betroffene Downloads zurückgezogen werden. Unter „Über xdzip“ können Nutzer nach Updates suchen und die Installation anstoßen. Die Anwendung sucht weder im Hintergrund nach Updates noch installiert sie diese still. Der aktuelle Quellstand enthält auch einen CLI-Befehl zur Aktualisierung. Der [Vertrag der Befehlszeilenschnittstelle](command-line.md) beschreibt die geeigneten Installationen und die Unterschiede zu den veröffentlichten Dateien.

Versionskandidaten bleiben auf Qt 6.8.2 und die bereits festgeschriebenen Rust- und Cargo-Versionen festgelegt, damit Prüfungen reproduzierbar sind. Daraus folgt weder, dass ältere Patches frei von Schwachstellen sind, noch eine Zusage dauerhaft kostenloser LTS-Updates. Vor jeder Veröffentlichung prüft die Projektleitung offizielle Sicherheitshinweise von Qt und verfügbare quelloffene Updates. Die Prüfung gewarteter öffentlicher Versionen hat Vorrang. Betrifft ein ungepatchtes Sicherheitsproblem einer Abhängigkeit xdzip, wird die betroffene Veröffentlichung ausgesetzt. Ein Qt-Upgrade erfordert erneute Prüfungen des Quellcodes, der Modullizenzen, der privaten Windows-Schnittstellen, des dynamischen Austauschs, der Builds für alle drei Plattformen sowie der tatsächlichen Menü- und Fensterfunktionen. Kommerzielle Wartung ist freiwillig.

<a name="xdzip-approved-scope-and-changes"></a>
## Genehmigter Umfang und Änderungen

Genehmigt sind die ursprünglichen Plattformen, die Wartung ausschließlich der neuesten öffentlichen Version im Rahmen der verfügbaren Möglichkeiten, der Verzicht auf eine feste Reaktionszeit und die Kennzeichnung von Entwicklungsbuilds als experimentell. Die Ergebnisse für `v0.1.0` stehen in der Veröffentlichung und in den zugehörigen Prüftickets. Diese Ergebnisse erlauben keine allgemeinen Leistungsgarantien für alle Systemversionen. Weitere Architekturen, Systeme oder Vertriebskanäle müssen vor einer Änderung dieser Richtlinie umgesetzt und geprüft werden. Siehe [ADR-011](../design/architecture-decisions.md#xdzip-adr-011-initial-support-scope-and-personal-project-maintenance-policy).

Offizielle Signatur, Notarisierung und kostenpflichtige Konten sind keine Vorgaben mehr. Melden Sie Schwachstellen gemäß der [Sicherheitsrichtlinie](../../../SECURITY.md).
