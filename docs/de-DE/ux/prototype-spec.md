<a name="xdzip-v01-interaction-prototype-and-feature-mapping"></a>

# Interaktionsprototyp v0.1 und Funktionszuordnung

[English](../../ux/prototype-spec.md) | [简体中文](../../zh-CN/ux/prototype-spec.md) | [繁體中文（台灣）](../../zh-TW/ux/prototype-spec.md) | [日本語](../../ja-JP/ux/prototype-spec.md) | [한국어](../../ko-KR/ux/prototype-spec.md) | [Français](../../fr-FR/ux/prototype-spec.md) | Deutsch | [Español](../../es-ES/ux/prototype-spec.md) | [Português (Brasil)](../../pt-BR/ux/prototype-spec.md) | [Русский](../../ru-RU/ux/prototype-spec.md)

Status: interaktiver Entwurf zur Designvalidierung. Die Quellen liegen in [design/html](../../../design/html/README.md), Einstieg ist [index.html](../../../design/html/index.html). Der Figma-Entwurf bleibt eine historische Erkundung; aktuelle Funktionen und Zustände werden im Repository weiter verfeinert.

<a name="xdzip-covered-flows"></a>

## Abgedeckte Abläufe

| Zustand | Einstieg und Ausstieg | Anforderungen / Abnahme |
| --- | --- | --- |
| B01 Durchsuchen | Beispielarchiv öffnen; Verzeichnis mit Doppelklick oder Eingabe öffnen, über Breadcrumb zurückkehren | P01/P02/P08, AC-01/02/08 |
| B02 Suchen und auswählen | Kontrollkästchen, Einzel-, ⌘/Ctrl-Mehrfach-, Shift-Bereichs- und Gesamtauswahl; Suche im aktuellen Verzeichnis und seinen Nachkommen oder an der Wurzel im ganzen Archiv | P02/P03, AC-02/03 |
| E01 Entpacken bestätigen | Hauptschaltfläche oder Umfangsmenü; Ziel, gleichnamiges Verzeichnis, Umfang und Konfliktregel sind editierbar | P03/P07, AC-03/07 |
| E02 Konflikt | Bei Kollision unter Nachfragen-Regel eintreten; nach Überspringen, automatischem Umbenennen oder Überschreiben fortfahren | P03/P07, AC-03/07 |
| T01 Aufgabe | Start → Vorbereitung → Verarbeitung → Prüfung → Ergebnis; zum Durchsuchen zurückkehren oder Stopp anfordern | P06/P15, AC-06/14 |
| R01 Ergebnis | Erfolg, Teilfehler, Abbruch, Quelländerung; unerledigte Elemente nach erneuter Bestätigung wiederholen | P03/P04/P15, AC-03/04/14 |
| C01 Erstellen | Eingaben hinzufügen/entfernen, Format/Stufe/Teile/Passwort und Speicherort wählen; bestehende Ziele zuerst bestätigen | P04/P05, AC-04/05 |
| S01 Einstellungen | Thema, Schriftgröße, Dichte, Sprache und letzter Verlauf; Schließen führt zurück zum Durchsuchen | P08/P11, AC-08/11 |
| X01 Ausnahmen | Leeres Archiv, keine Ergebnisse, Laden, Beschädigung, nicht unterstützte Funktionen, Berechtigungen und Aufgabenfehler | P05/P07/P15, AC-05/07/14 |

„Designanmerkungen“ im Prototypkopf zeigt diese IDs und aktuellen Zustandsregeln. Ergebnisse zu Berechtigungen und Speicherplatz folgen Szenarien; ein erfolgreicher Prototyplauf besteht nicht die technische Abnahme P09/P10/P12.

<a name="xdzip-interaction-decisions"></a>

## Interaktionsentscheidungen

<a name="xdzip-selection-and-search"></a>

### Auswahl und Suche

- Jede Auswahl entspricht einer stabilen Eintrags-ID, keiner sortierten Zeilennummer. Ein Wechsel von Abfrage oder Verzeichnis leert die Auswahl, um Missverständnisse über versteckte Auswahl zu vermeiden.
- Alles in der aktuellen Liste auswählen und das gesamte Archiv entpacken sind verschiedene Konzepte. Ein angekreuzter Ordner wird zu deduplizierten untergeordneten Dateien erweitert.
- Während der IME-Eingabe keine Abfragen absenden; nach compositionend aktualisieren. Simulierte Browserereignisse ersetzen keine native IME-Abnahme auf macOS/Windows/Linux.
- Eingabe/Doppelklick auf eine gewöhnliche Datei entpackt eine temporäre Kopie und öffnet sie mit der Systemzuordnung; bei einem Verzeichnis wird dieses betreten. Der Eigenschafteneintrag bleibt erhalten. HTML demonstriert nur Fortschritt und automatisches Ausblenden, schreibt keine Dateien und ruft das System nicht auf. Der [Vorschauvertrag](../engineering/content-preview.md) beschreibt die automatische Inhaltsvorschau von Qt.

<a name="xdzip-extraction-and-conflicts"></a>

### Entpacken und Konflikte

- Den endgültigen Pfad laufend anzeigen; ein gleichnamiges Verzeichnis nur einmal anhängen. Leere oder nachweislich nicht beschreibbare Beispielpfade blockieren den Start; nach Zieländerung erneut vorprüfen.
- Bei Konflikten standardmäßig nachfragen; Überspringen, automatisches Umbenennen und Überschreiben sind verfügbar. Benachbarter Text erklärt die Folgen des Überschreibens.
- Der anfängliche Fokus im Konfliktfenster liegt auf Überspringen; Überschreiben ist nicht die Standardaktion für Eingabe. Die Anwendung auf weitere Konflikte gilt nur für diese Aufgabe; Beispieldaten erzeugen nur einen Konflikt.
- Umbenennungen erscheinen in Abschlussübersicht und Dateidetails. Übersprungene Elemente sind keine Fehler; bestehende Beispieldateien zählen nicht als abgeschlossene Schreibvorgänge.

<a name="xdzip-tasks-and-results"></a>

### Aufgaben und Ergebnisse

- Die Rückkehr zum Durchsuchen bricht die Aufgabe nicht ab; die Statusleiste kann sie erneut öffnen.
- Abbruch anfordern → wird angehalten, erneuter Abbruch deaktiviert → Abbruchübersicht. Entpacken behält veröffentlichte Elemente; Archiverstellung veröffentlicht keine Zwischenergebnisse.
- Ein voller Datenträger stoppt die Aufgabe und behält Erfolge. Wiederholen wählt nur fehlgeschlagene oder unerledigte Elemente und führt zur Planbestätigung zurück.
- Bei Quelländerungen muss das Archiv neu geöffnet werden, statt den alten Index still weiterzuverwenden.
- Eine Erstellungsaufgabe zählt ein endgültiges Archiv; Abbruch zeigt null erstellte Archive und erhält das alte Archiv ausdrücklich.

<a name="xdzip-archive-creation-and-capabilities"></a>

### Archiverstellung und Fähigkeiten

Der Prototyp veranschaulicht ZIP-Stufen, TAR-Packen und GZIP-Kompressionsoptionen. Er prüft grundlegende Eingabe-, Dateinamen- und Zielfelder und ändert die Erweiterung mit dem Format. Aktuelle Produktoptionen verwenden das [integrierte Formatregister](../engineering/format-registry.md), einschließlich der Erstellungsprofile [ISO](../engineering/iso-creation.md) und [Windows SFX](../engineering/sfx-creation.md). Die zugelassenen Optionen stehen unter [Passwortinteraktion](password-extraction.md) und im [Teilarchivvertrag](../engineering/archive-opening.md).

<a name="xdzip-visuals-and-adaptation"></a>

### Darstellung und Anpassung

Warmgraue, kobaltblaue und graphitfarbene Themen beibehalten. Die normale Zeilenhöhe beträgt 40, kompakt 32. Der Textakzent des dunklen Themas ist unabhängig von der Schaltflächenfüllung, damit kleine Schrift lesbar bleibt. Bei geringer Desktopbreite werden Details zu einer schließbaren Überlagerung. Die mobile Prüfung klappt Seitenleiste und sekundäre Tabellenspalten ein; Vorgangsfenster passen sich schmalen Bildschirmen an und scrollen.

Vorgangsoberflächen sind gemäß [Plattformfensterspezifikation](platform-windows.md) eigenständige Fenster. HTML simuliert sie mit dialog: Entscheidungsfenster begrenzen den Fokus; Fortschritt, Ergebnisse und Einstellungen sind nicht modal; beim Schließen wird versucht, den Fokus wiederherzustellen. Das Schließen eines Konfliktfensters bricht die Aufgabe ab, statt still zu überspringen; das Schließen des Fortschritts blendet nur Details aus. Qt verwendet echte eigenständige Fenster der obersten Ebene.

<a name="xdzip-theme-selection"></a>

### Themenauswahl

Die Einstellungsseite ordnet drei native Radiobuttons für System, hell und dunkel mit `fieldset#setting-theme` und der Legende „Appearance theme“ an. Jede Themenkarte verwendet die gemeinsame Ressource `theme-system`, `theme-light` oder `theme-dark` und verknüpft Textbeschriftung und Eingabe. Die tatsächlichen Radiobuttons nehmen an Kartenklicks, Tastaturwechsel und Fokus teil. Die Auswahl verwendet die gemeinsame Radiomarkierung und synchronisiert Kartenrand und Vorschau, statt einen nicht interaktiven Screenshot einzusetzen.

Dies beschreibt das Verhalten der HTML-Designvalidierung. Aktuelle Einstellungen liegen im Speicher; das Systemthema wird nur bei Auswahl aufgelöst. Laufende Systemthemenänderungen, startübergreifende Speicherung sowie sofortiges Speichern und Fehlerbehandlung in den produktiven Qt-Einstellungen müssen getrennt geprüft werden.

<a name="xdzip-verification-and-remaining-boundaries"></a>

## Prüfung und verbleibende Grenzen

Das Browserprüfskript ist scripts/check-html-design.py. Es prüft echte Klickabläufe, Statistiken, deaktivierte Bedingungen, Formatkopplung, Wiederherstellung, mobilen Überlauf und Offline-Betrieb als Einzeldatei. Berichte begleiten das exportierte Prototyppaket; Browserprüfungen verwenden nur Chromium.

Die aktuelle Hauptfensterlieferung enthält zwei unabhängige Beispielarchivsitzungen, Liste/Raster, Feldsortierung, rekursiven Verzeichnisbaum, hierarchische Breadcrumbs, Kontext-/Dropdown-/Untermenüs und Schaltflächenzustände. Siehe [Komponenten- und Interaktionsvertrag](main-window-components.md) und [Prüfmethode](../../../developers/qa/README.md). `scripts/check-html-controls.py` prüft sie mit echten Browsereingaben; die [Komponentenprüfseite](../../../design/html/components.html) nutzt dieselbe Implementierung wie die Hauptoberfläche.

HTML prüft nur Beispielzustände; es beweist keine echte Aufgabenplanung, Autorisierung, Pfadrace-Bedingungen, CRC oder Plattformzugänglichkeit. Qt integriert echte Erstellung/Extraktion, Drag-and-drop, Systemauswahl, persistente Einstellungen und unabhängige Archivsitzungen in optionalen Tabs oder mehreren Fenstern. Der tatsächliche Umfang mehrerer Formate, der Indizierung einer Million Einträge gewöhnlicher Einzeldatei-ZIPs und der begrenzten Vorschau steht jeweils unter [Format-Backends](../engineering/native-codecs.md), [großer Indizierung](../engineering/large-index.md) und [Inhaltsvorschau](../engineering/content-preview.md); die Abnahme steht in den jeweiligen Issues. HTML-Einstellungen behalten den Demozustand nur innerhalb der Seite, nicht über Starts hinweg, und belegen weder natives Folgen des Systemthemas noch eine Zugänglichkeitsabnahme.

Diese Grenzen ändern die Zielfähigkeiten des Softwareentwurfs nicht. „Abgeschlossen“ im Prototyp bezeichnet nur einen Demo-Aufgabenstatus und belegt keine Auslieferung echter Kompression oder Extraktion.

<a name="xdzip-current-directory-queries-and-card-revisions"></a>

## Aktuelle Verzeichnisabfragen und Kartenrevisionen

Der Wurzeleintrag heißt „Root“ und listet nur direkte Dateien und Ordner. Die Suche findet Nachkommen im aktuellen Bereich und zeigt vollständige Pfade im Archiv. Gewöhnliche Listen/Raster zeigen nur den letzten Namen; Kopieren und Navigation erhalten vollständige Pfade. Rechts im Breadcrumb steht die aktuelle Listenzahl. Der Verzeichnisbaumtitel heißt „Archive“ und ist vertikal mit dem Breadcrumb zentriert; beim Überfahren des Baums erscheint kein schwebender Tooltip.

Listen-Hover umfasst die ganze Zeile. Raster haben ungefähr 16 logische Pixel horizontalen Innenabstand; ein Kontrollkästchen von 8 Pixeln oben links erscheint beim Überfahren und bleibt bei Auswahl sichtbar. Über Schaltflächen geöffnete Menüs stehen 4 Einheiten von der Schaltfläche entfernt und nutzen einen weichen Schatten. Die Schaltfläche zum erneuten Öffnen der Vorschau ist quadratisch mit Kantenlänge 28. Parameter stehen in layout-scale.json, directory-tree.json und controls.json; die tatsächliche Prüfung folgt der [allgemeinen Methode](../../../developers/qa/README.md).

<a name="xdzip-settings-window"></a>

## Einstellungsfenster

Die Einstellungen behalten ursprüngliche Symbolüberschriften, Themenvorschaukarten, Dichte, letzten Verlauf und Datenschutzhinweis bei und ergänzen Textgröße und Oberflächensprache. Themen- und Schriftvorschauen können abgebrochen werden; Speichern und Standardwerte wiederherstellen verwenden Entwürfe. HTML behält den Demozustand nur innerhalb der aktuellen Seite. In der HTML-Demo zeigt die Sprachauswahl eine Konfiguration, die beim nächsten Qt-Start wirksam wird; weder die Designprüfseite noch der Inhalt des Beispielarchivs belegt die Übersetzungsabdeckung des Produkts. Für aktuelle Qt-Sprachumschaltung, mitgelieferte Ressourcen und Speicherung siehe [Einstellungen und Lokalisierung](settings-localization.md).
