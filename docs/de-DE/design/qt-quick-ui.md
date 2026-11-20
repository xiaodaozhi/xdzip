<a name="xdzip-qt-quick-desktop-interface"></a>

# Desktopoberfläche mit Qt Quick

[English](../../design/qt-quick-ui.md) | [简体中文](../../zh-CN/design/qt-quick-ui.md) | [繁體中文](../../zh-TW/design/qt-quick-ui.md) | [日本語](../../ja-JP/design/qt-quick-ui.md) | [한국어](../../ko-KR/design/qt-quick-ui.md) | [Français](../../fr-FR/design/qt-quick-ui.md) | Deutsch | [Español](../../es-ES/design/qt-quick-ui.md) | [Português (Brasil)](../../pt-BR/design/qt-quick-ui.md) | [Русский](../../ru-RU/design/qt-quick-ui.md)

Die Desktopanwendung nutzt Qt Quick für Anzeige und Interaktion. C++-Controller leiten Benutzerabsichten weiter und halten den Ansichtsstatus. Die Rust-Laufzeit interpretiert Archive und verantwortet Aufgabenregeln und Dateiberechtigungen. Die [Systemarchitektur](system-architecture.md) und der [Brückenvertrag](qt-rust-bridge.md) beschreiben die folgenden Aufrufe.

<a name="xdzip-windows-sessions-and-models"></a>

## Fenster, Sitzungen und Modelle

| Komponente | Aktuelle Zuständigkeit |
| --- | --- |
| `Browser.qml` und `SessionPage.qml` | Startseite, Archiv-Tabs, Pfad- und Suchfelder, Verzeichnisnavigation, Liste oder Raster, Eigenschaften und Vorgangsaktionen. Eine Seite zeigt den aktuellen Controllerzustand an und ruft Befehle auf. |
| `ApplicationSessions` | Leitet Öffnungsanforderungen an unabhängige Ansichtsitzungen weiter und koordiniert Tabs oder Fenster, den gemeinsamen Aufgabenbereich und das Beenden. Der Öffnungsmodus gilt für künftige Anforderungen. Beim Schließen einer Seite wird die Ansicht freigegeben. Der Controller einer Aufgabe kann weiterlaufen. |
| `BrowserController` und `ArchiveTableModel` | Verwalten Auswahl, Auflistungsstatus und Ergebnisidentität. Kleine Ergebnisse werden als vollständiges Modell übernommen. Bei großen Ergebnissen gibt es eine logische Zeilenzahl mit höchstens 1.024 gleichzeitig gehaltenen Zeilen. Fehlende Seiten werden nachgeladen. Aktionen für ungeladene Zeilen sind gesperrt. |
| `TaskListModel` und Vorgangsfenster | Zeigen begrenzte Aufgabensnapshots, Fortschritt, Ergebnisse und die anstehende Entscheidung. Die zusammengesetzte Desktopidentität leitet Befehle an den zuständigen Controller. |
| Einstellungen und Plattformadapter | Verwalten Erscheinungsbild, Sprache, Textgröße und Anwendungsverhalten. Native Adapter übernehmen je nach Betriebssystem Infobereich, Aktivierung, Dateidialoge, Zuordnungen und die Übergabe von Transfers. |

[Desktop-Einstieg](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/browser_main.cpp), [Sitzungsseite](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/ui/SessionPage.qml), [Controller und Modelle](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/BrowserController.h), [Anwendungssitzungen](../engineering/application-sessions.md), [Einstellungen und Lokalisierung](../ux/settings-localization.md)

<a name="xdzip-browsing-and-operation-flow"></a>

## Navigation und Vorgangsablauf

Beim Öffnen wird ein abgeschlossener Metadatensnapshot veröffentlicht. Jede Suche, Sortierung oder Navigation hat ihre eigene Ergebnisidentität. Eine veraltete Seite kann das aktuelle Modell nicht ersetzen. Während einer Abfrage zeigen Ansichten das übernommene Ergebnis, solange der Controller es für gültig hält. Der Kerntransport sendet höchstens 128 Zeilen und 64 KiB Namensdaten pro Stapel. [Domänendaten](domain-data.md), [Indizierung großer Archive](../engineering/large-index.md)

Das Öffnen eines Verzeichnisses navigiert hinein. Doppelklicken auf einen zulässigen regulären Eintrag oder der entsprechende Öffnungsbefehl startet die Aufgabe zum temporären Öffnen. Im Eigenschaftsbereich gibt es eine eigene Vorschauaktion. Konflikt- und Passwortdialoge behalten die Identität des aktuellen Vorgangs. Fortschritt und Ergebnisse sind über den Aufgabenbereich auch nach dem Schließen der Seite verfügbar. [Archive durchsuchen](../user-guide/browse.md), [Temporäres Öffnen](../engineering/archive-opening.md), [Inhaltsvorschau](../engineering/content-preview.md), [Aufgabenfenster](../ux/task-windows.md)

Ablegen öffnet je nach Ziel Archive oder liefert Eingaben für die Erstellung. Zulässige Verschiebungen im Archiv und das Herausziehen werden als Aufgaben vorbereitet und plattformspezifisch übergeben. Die Plattform bestimmt die Transfermöglichkeiten, einschließlich asynchroner Inhalte und Schreibvorgänge am Ziel. [Archivtransfers](../engineering/archive-transfers.md)

<a name="xdzip-shared-controls-and-native-boundaries"></a>

## Gemeinsame Bedienelemente und native Grenzen

Vorgangsfenster verwenden `OperationButton`, `OperationFooterButton`, `OperationFrame`, `OperationSelect` und `OperationSegmentedGroup`. Gemeinsame Komponenten legen Maße und Interaktion fest. Gemeinsame Auswahlkomponenten leiten Maus-, Tastatur- und Barrierefreiheitsaktionen an dasselbe Zustandsmodell. UI-Texte und Dateidaten befolgen die Schrift- und Lokalisierungsregeln im [Gestaltungsstandard](../../visual/design-standard.md). Archivnamen bleiben Daten und werden sicher angezeigt.

Eigenständige Vorgangsfenster folgen der Fensterregistrierung und den Regeln für Modalität und Schließen des jeweiligen Fensters. macOS nutzt native Elemente der Titelleiste. Windows und Linux verwenden ihre Systemadapter. Der [Vertrag für Plattformfenster](../ux/platform-windows.md) legt die Abnahme echter Fenster fest. Der [Vertrag für Desktopintegration](../ux/desktop-integration.md) beschreibt Infobereich und Aktivierung.

<a name="xdzip-design-and-verification-scope"></a>

## Umfang der Gestaltung und Prüfung

Quellcode und Tests beschreiben das Produktverhalten. Detailentwürfe in HTML zeigen Layouts und simulierte Zustände zur Überprüfung. Änderungen an nativem Fokus, Menüs, IME, Barrierefreiheitsaktionen, Bildschirmskalierung und Systemdateimanagern brauchen Nachweise auf den betroffenen Plattformen. Neue Desktopkonfigurationen und neue Vorgangsfenster erfordern die zugehörige Implementierung und Abnahme. [Visueller Arbeitsablauf](../visual/design-workflow.md), [Qualitätssicherung](quality-engineering.md)
