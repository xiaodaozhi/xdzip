<a name="xdzip-platform-windows-and-title-bars"></a>

# Plattformfenster und Titelleisten

[English](../../ux/platform-windows.md) | [简体中文](../../zh-CN/ux/platform-windows.md) | [繁體中文](../../zh-TW/ux/platform-windows.md) | [日本語](../../ja-JP/ux/platform-windows.md) | [한국어](../../ko-KR/ux/platform-windows.md) | [Français](../../fr-FR/ux/platform-windows.md) | Deutsch | [Español](../../es-ES/ux/platform-windows.md) | [Português (Brasil)](../../pt-BR/ux/platform-windows.md) | [Русский](../../ru-RU/ux/platform-windows.md)

Die Bedienoberflächen von xdzip verwenden eigenständige Fenster der obersten Ebene, keine an das Hauptfenster angehängten Sheets oder darin gezeichneten Überlagerungen. Ein eigenständiges Fenster bedeutet weder einen eigenen Prozess noch für jedes Hilfsfenster einen Taskleisteneintrag. Aufklappmenüs und Tooltips bleiben leichtgewichtige Popups und sind keine Bedienfenster.

<a name="xdzip-window-contract"></a>

## Fenstervertrag

| Fenster | Beziehung zum Hauptfenster | Schließen / Escape | Minimieren / maximieren |
| --- | --- | --- | --- |
| Formular zum Entpacken oder Erstellen | Nicht modales `OperationWindow` mit transientem Eigentümer bei Öffnung aus der Archivansicht | Entwurf oder Bestätigung schließen, ohne eine neue Aufgabe zu starten | Deaktiviert / deaktiviert |
| Konfliktabfrage einer Aufgabe | Nicht modale Abfrage im Besitz des Hosts des Bedienfensters. Die Aufgabe wartet auf eine Entscheidung | Abfrage schließen. Über die Aufgabe erneut öffnen oder dort die Aufgabe ausdrücklich abbrechen | Deaktiviert / deaktiviert |
| Passwort-, Beenden- oder Neustartabfrage | Anwendungsmodal. Auch die native Dateiauswahl ist anwendungsmodal und hat einen transienten Aufrufer | Für Passwort, Beenden und Neustart gibt es jeweils ausdrückliche Schließ- oder Bestätigungsaktionen | Abhängig von den Fähigkeiten des nativen Dialogs |
| Aufgabenfortschritt und Ergebnis | Nicht modales `TaskWindow`. Ohne anderes aktives modales Fenster kann weiter navigiert werden | Details schließen und über den Aufgabeneintrag erneut öffnen. Eigenständige Arbeit behält einen Zugang zu aktiven Aufgaben | Minimieren / deaktiviert |
| Einstellungen und Über | Nicht modal. Ein Infobereichseintrag kann sie ohne besitzendes Archivfenster öffnen | Ansicht schließen. Bereits gespeicherte Einstellungen bleiben aktiv | Deaktiviert / deaktiviert |

Der Aufgabenabbruch ist ein ausdrücklicher Befehl und unabhängig vom Schließen des Fortschrittsfensters. Ein Abschluss im Hintergrund entzieht dem Vordergrund nicht den Fokus; offene Details oder andernfalls der Aufgabeneintrag werden aktualisiert. Das Beenden der Anwendung folgt weiterhin der Aufgaben-Beendigungsregel; das Schließen des Hauptfensters darf nicht unmittelbar das Schließen der Hilfsfenster bedeuten.

<a name="xdzip-platform-appearance"></a>

## Plattformdarstellung

- macOS: eine systemdekorierte Titelleiste mit roten, gelben und grünen Bedienelementen oben links verwenden; native Elemente nicht durch selbst gezeichnete farbige Punkte ersetzen. Bedienfenster fester Größe behalten das rote Schließen und deaktivieren nicht verfügbare Minimieren- und Zoomfunktionen; Fortschrittsfenster können minimiert werden. Die tatsächliche Verfügbarkeit hängt von AppKit/Qt-Fensterfähigkeiten ab und wird auf einem echten Gerät abgenommen. Das Hauptfenster behält normale Größenänderung, Minimierung und Vollbildfunktion.
- Windows verwendet eine in den Clientbereich erweiterte Haupttitelleiste mit QML-Fensterknöpfen, nativem größenveränderlichem Rahmen und DWM-Schatten. Verschieben, Größenänderung am Rand und System-Snapping nutzen Plattformfunktionen. Eigenständige Bedienfenster verwenden native Dekorationen. Siehe die [Hauptentwürfe für Windows und Ubuntu](../../../design/hifi/sources/main.html?platform=windows).
- Ubuntu X11 verwendet rechts integrierte Bedienelemente und im Normalzustand einen abgerundeten Umriss. Compositing-Sitzungen mit Eingabebereichsformung zeigen zusätzlich einen weichen, klickdurchlässigen Fensterschatten. Maximierte Fenster und Vollbildfenster haben eckige Kanten und füllen den verfügbaren Bereich. Das Fenstersystem übernimmt Verschieben und Größenänderung. Eigenständige Bedienfenster behalten Systemdekorationen. Andere Backends außerhalb des angepassten X11-Pfads nutzen normale Dekorationen. Wayland liegt außerhalb des unterstützten Umfangs. Siehe den [Ubuntu-Hauptentwurf](../../../design/hifi/sources/main.html?platform=linux).

Jedes Bedienfenster besitzt eigenen Titel, Rahmen, Schatten und eine verschiebbare Titelleiste; das Hauptfenster dahinter wird nicht wie eine Webseite abgedunkelt oder weichgezeichnet. Beim ersten Öffnen wird es im verfügbaren Bereich des Bildschirms seines Eigentümers zentriert. Wiederhergestellte Positionen müssen auf sichtbare Bildschirme begrenzt werden. Werkzeugleisten und Inhalt dürfen die Titelleiste nicht verdecken.

<a name="xdzip-qt-quick-implementation-constraints"></a>

## Qt-Quick-Implementierungsgrenzen

`OperationWindow` nutzt ein eigenständiges `Window` mit optionalem `transientParent` und standardmäßig `Qt.NonModal`. `PasswordWindow` und `OperationConfirmationWindow` verwenden ausdrücklich `Qt.ApplicationModal`. Die native Auswahl nimmt das tatsächlich aufrufende Fenster als transienten Eigentümer und stellt dessen Aktivierung nur wieder her, solange es sichtbar ist. Siehe [Fensterregister](../../../specs/visual/window-registry.json) und [Aufgabenfenster](task-windows.md).

Angepasste Windows-/Linux-Titelleisten benötigen echte Plattformfunktionen für Fensterverschiebung, Größenänderung und Systemmenü; Systemverschiebung darf nicht durch manuelles Ändern von QML-Positionen simuliert werden. `WindowController` verwaltet Fensterlebenszyklus, Eigentümerschaft, Schließanfragen und erneutes Öffnen von Aufgabenansichten. Der Kern besitzt die Aufgaben; das Zerstören einer Ansicht darf eine aktive Aufgabe nicht zerstören. Threadübergreifende Aktualisierungen des Fensterzustands verwenden nur definierte Brückenereignisse.

<a name="xdzip-prototype-evidence-and-limits"></a>

## Prototypnachweise und Grenzen

Der HTML-Prototyp simuliert das Aussehen eigenständiger Fenster, das Ziehen der Titelleiste, blockierende und nicht blockierende Interaktionen sowie das Schließverhalten. Die obere Auswahl „Platform“ vergleicht macOS, Windows und Linux und ist keine Produkteinstellung. Der Browser-`dialog` bleibt innerhalb einer Browserseite und bietet keine echten nativen Fenster, Systemknöpfe, Systemminimierung, Taskleiste oder Mehrbildschirmfunktion. Die Minimierung eines Aufgabenfensters wird durch Ausblenden der Details simuliert.

Die Plattformbedienelemente des Prototyp-Hauptfensters veranschaulichen das Aussehen. Bei der Rückkehr von der Überschreibbestätigung zum Erstellen wird das Beispielformular noch zurückgesetzt; die Produktimplementierung muss den Entwurf erhalten. Diese Grenzen sind kein Nachweis einer abgeschlossenen nativen Abnahme.

<a name="xdzip-real-device-acceptance-criteria"></a>

## Abnahmekriterien auf echten Geräten

1. Jedes Bedienfenster ist ein eigenständiges Plattformfenster mit korrekter Zuordnung zum Hauptfenster, ohne Sheet oder webseitenartige Überlagerung.
2. Reihenfolge der Ampel- beziehungsweise rechten Knöpfe, deaktivierter Zustand und Schließanfragen entsprechen Plattform- und Fensterfähigkeiten; Titel und Systemknöpfe überlappen bei verschiedenen Schriftgrößen und Skalierungen nicht.
3. Fensterverschiebung, Bildschirmgrenzen, mehrere Bildschirme, 125%/150%/200% Skalierung sowie helles und dunkles Design funktionieren korrekt.
4. Das Schließen der Aufgabendetails lässt die Aufgabe laufen und der Aufgabeneintrag kann sie erneut öffnen. Das Schließen eines Konfliktfensters lässt die Aufgabe warten, bis die Abfrage erneut geöffnet oder die Aufgabe ausdrücklich abgebrochen wird.
5. Eigentümermodalität, Fokuswiederherstellung, Tastaturnavigation, Titel für Bildschirmleser und Systemkürzel funktionieren; nicht zugehörige Archivfenster werden nicht unnötig blockiert.
6. Windows-Systemmenü und Snapping sowie Linux X11 getrennt prüfen; Wayland liegt außerhalb der anfänglichen Unterstützung. Bestehende Qt-Ergebnisse auf echten Geräten stehen in [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5). Diese Liste dient der späteren erneuten Prüfung betroffener Punkte und setzt abgeschlossene Abnahmen nicht zurück.
