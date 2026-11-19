# Alle oder ausgewählte Dateien entpacken

[English](../../user-guide/extract.md) | [简体中文](../../zh-CN/user-guide/extract.md) | [繁體中文（台灣）](../../zh-TW/user-guide/extract.md) | [日本語](../../ja-JP/user-guide/extract.md) | [한국어](../../ko-KR/user-guide/extract.md) | [Français](../../fr-FR/user-guide/extract.md) | Deutsch | [Español](../../es-ES/user-guide/extract.md) | [Português (Brasil)](../../pt-BR/user-guide/extract.md) | [Русский](../../ru-RU/user-guide/extract.md)

<a name="xdzip-from-the-archive-window"></a>

## Aus dem Archivfenster

1. Öffnen Sie das Archiv. Die Hauptschaltfläche in der Symbolleiste entpackt die aktuelle Auswahl. Ohne Auswahl entpackt sie das ganze Archiv. Über den Pfeil neben der Schaltfläche können Sie ausdrücklich **Alles entpacken** oder **Auswahl entpacken** wählen.
2. Prüfen Sie den Umfang und wählen Sie einen Zielordner mit Schreibzugriff. Falls verfügbar, können Sie einen Ordner mit dem Namen des Archivs anlegen lassen.
3. Entscheiden Sie, wie Namenskonflikte verarbeitet werden. **Nachfragen** wartet auf Ihre Entscheidung. **Überspringen** behält die vorhandene Datei. **Umbenennen** speichert die neue Datei separat. **Überschreiben** ersetzt nach der Prüfung eine Datei, sofern dies möglich ist.
4. Starten Sie den Vorgang und prüfen Sie das Ergebnis. Bei Fehlern oder einem Abbruch können bereits entpackte Dateien erhalten bleiben. Lesen Sie vor einem erneuten Versuch in den Ergebnisdetails nach, was noch aussteht. [Prüfungen, Aufgaben und Ergebnisse](checks-and-tasks.md)

Im Menü gibt es außerdem **Vor dem Entpacken Metadaten prüfen**. Wählen Sie dafür bis zu 128 tatsächliche Archiveinträge aus. Gemeldet werden beispielsweise unsichere Namen, nicht unterstützte Methoden und angegebene Grenzwerte. Ein nur zur Navigation angezeigter impliziter Ordner kann entpackt werden, sofern das Format dies zulässt. Die Vorprüfung liest keine Nutzdaten und erteilt keine Schreibberechtigung. [Vorprüfung und Grenzen beim Entpacken](../engineering/extraction-preflight.md)

Aus großen ZIP-Archiven mit Index auf der Festplatte lassen sich innerhalb der Aufgabengrenzen ausgewählte reguläre Dateien entpacken. Überschreitet das gesamte Archiv das Limit, wählen Sie eine kleinere Gruppe. Wurde die Quelle geändert oder ersetzt, öffnen Sie sie erneut und treffen Sie eine neue Auswahl. [Grenzen für große Archive](../engineering/large-index.md)

<a name="xdzip-from-the-file-manager"></a>

## Aus dem Dateimanager

Unter Windows 11 öffnen Sie das klassische Menü mit **Weitere Optionen anzeigen**. Die Store-Version kann die Aktionen auch im modernen Menü anzeigen. Ubuntu verwendet die Nautilus-Erweiterung. Unter macOS aktivieren Sie die Finder-Erweiterung in den Systemeinstellungen oder verwenden den registrierten Finder-Dienst. Ist das Menü nicht verfügbar, öffnen Sie das Archiv in xdzip und entpacken Sie es dort.

Über das Menü des Dateimanagers können Sie jedes Archiv im bisherigen Verzeichnis entpacken, einen eigenen Ordner pro Archiv anlegen, den Zielordner automatisch bestimmen lassen oder selbst einen Ordner wählen. Die Automatik verzichtet nur dann auf einen zusätzlichen Ordner, wenn das geprüfte Ergebnis bereits genau einen Ordner auf der obersten Ebene enthält. Bei einer Stapelverarbeitung erscheint ein Ergebnis pro Archiv. Nach einem einzelnen Fehler wird die Verarbeitung fortgesetzt. Ein Abbruch verhindert weitere Vorgänge und fordert an, den laufenden Vorgang zu stoppen. [Aktionen und Einstellungen des Dateimanagers](../engineering/shell-actions.md)
