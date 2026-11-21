<a name="xdzip-main-window-welcome-page"></a>

# Willkommensseite des Hauptfensters

[English](../../ux/welcome.md) | [简体中文](../../zh-CN/ux/welcome.md) | [繁體中文](../../zh-TW/ux/welcome.md) | [日本語](../../ja-JP/ux/welcome.md) | [한국어](../../ko-KR/ux/welcome.md) | [Français](../../fr-FR/ux/welcome.md) | Deutsch | [Español](../../es-ES/ux/welcome.md) | [Português (Brasil)](../../pt-BR/ux/welcome.md) | [Русский](../../ru-RU/ux/welcome.md)

Der hochauflösende Entwurf ist bestätigt und in den offiziellen HTML-Prüfeinstieg integriert. Qt implementiert tatsächliches Öffnen und Erstellen; die manuelle Desktopabnahme wird getrennt erfasst. Siehe Zustandsentwurf der Willkommensseite und [welcome-reference.json](../../../specs/visual/welcome-reference.json). Farben und Hauptfensterdichte behalten bestehende Tokens und layout-scale bei, ohne globale Standards zu ändern. Vier PNGs sind Exporte codegezeichneter Entwurfsoriginale, keine Screenshots des laufenden Produkts.

<a name="xdzip-information-and-layout"></a>

## Informationen und Layout

Die Überschrift der Startseite lautet auf Chinesisch „打包与解压，都简单一点。“ und auf Englisch „Pack and unpack, a little simpler“; der deutsche Katalog verwendet „Packen und Entpacken leicht gemacht“. Der Begleittext beschreibt zuerst Durchsuchen, Entpacken nach Bedarf und Packen zum Teilen.

Der Clientbereich misst standardmäßig 1024×622, mindestens 832×544; Titelbereich 48, Kapsel 32 und untere Leiste 32. Die 608 breite Inhaltsgruppe ist nur horizontal zentriert und beginnt fest 28 logische Pixel unter dem verfügbaren oberen Inhaltsrand, nicht vertikal zentriert. Die Überschrift misst 30, bei schmalem Fenster 28; Beschreibung 13, Aktionstitel 16 und Kartenbeschreibung 12. Zwei Aktionskarten sind 296 breit und 124 hoch, mit Abstand 16, Innenabstand 20 und Radius 8. Dies sind Einstiegskarten der Willkommensseite und kein Ersatz für gemeinsame Schaltflächen von Vorgangsfenstern.

Die Titelleiste verwendet vollständig die vorhandenen Hauptfensterkomponenten mit Marke, Starttab, Tab hinzufügen und Darstellungseinstellungen; die Willkommensseite ändert weder Layout noch Maße, Schriftarten oder Symbole. Der Inhalt zeigt keinen leeren Dateibaum, Tabellenkopf, Suchfeld oder Entpack-Werkzeugleiste; das Navigationslayout erscheint nach Öffnen eines Archivs. Die Hauptaktion heißt „Archiv öffnen“, die sekundäre „Archiv erstellen“; auf den Ablagebereich folgen unterstützte Dateitypen. Jede Kategorie zeigt ihre Anzahl registrierter Erweiterungen. „Archive“ ist anfangs geöffnet. Eine andere Kategorie aktiviert deren Vorschau, dieselbe klappt sie wieder ein. Akzenttext und Schriftstärke kennzeichnen die aktive Kategorie. Die Vorschau zeigt Erweiterungen in einer Zeile leicht gefüllter, umrandeter Marken mit normaler UI-Schrift. Auslassungspunkte markieren Überlauf; beim Überfahren der Zeile erscheinen die Endungen der gewählten Kategorie. „Dateizuordnungen“ öffnet die vorhandenen Zuordnungseinstellungen. Der Hinweis zur lokalen Verarbeitung in der unteren Leiste beschreibt den Ort der Verarbeitung und ist keine Sicherheitsgarantie. Die Aufgabenanzahl nutzt den tatsächlichen Verlauf; null im Entwurf steht für einen leeren Verlauf beim ersten Start.

Keine beispielhaften zuletzt verwendeten Dateien, Beispielgrößen oder nicht vorhandenen Formatfähigkeiten anzeigen. Ampeln im Titelbereich zeigen nur die Position der macOS-Dekoration; die Implementierung verwendet Plattformfensterfunktionen. Windows und Linux behalten ihre Dekorationen ohne kopierte Ampeln.

<a name="xdzip-interaction-design"></a>

## Interaktionsentwurf

- Die Willkommensseite kann beim ersten Start, nach Schließen des letzten Archivtabs oder nach einem fehlgeschlagenen Öffnen ohne durchsuchbares Archiv erscheinen. Hintergrundaufgaben laufen weiter und werden nicht zurückgesetzt.
- Öffnen ruft die Systemdateiauswahl auf; Abbrechen lässt die Willkommensseite stehen. Erfolg wechselt zum echten Archivtab; ein Fehler zeigt eine behebbare Meldung und darf keine erfolgreiche Liste vortäuschen.
- Erstellen öffnet das vorhandene Erstellungsfenster; Abbrechen behält die Willkommensseite. Vorgeschlagene Kürzel sind ⌘O/⌘N auf macOS und Ctrl+O/Ctrl+N auf Windows/Linux. Vor Implementierung tatsächliche Bindungen prüfen; dieser Entwurf behauptet keine Integration.
- Das Ziehen einer zulässigen Auswahl hebt den Ablagebereich mit 2 logischen Pixeln Akzentrand und Auswahlfüllung hervor. Verlassen stellt den Normalzustand wieder her. Desktop und HTML-Prüfung haben getrennte Zulassungsregeln. Siehe [Tatsächliche Qt-Integration](#xdzip-real-qt-integration) für das wirkliche Öffnen.
- Die ganze Karte ist anklickbar. Die Tastaturreihenfolge nach der Titelleiste ist Öffnen, Erstellen, Aufgaben. Tastaturfokus zeichnet keinen zusätzlichen Umriss und folgt der fensterweiten Fokusverlustregel. Hover und Drücken nutzen gemeinsames accentHover und accentPressed, nicht nur Deckkraft.
- Beim Öffnen ersetzt die Öffnen-Karte ihren Titel durch einen Beschäftigtindikator und „Wird geöffnet…“ und verhindert vorübergehend erneutes Öffnen und Ablegen; der deaktivierte Zustand nutzt accentDisabled und accentDisabledText. Ob das Erstellungsfenster gleichzeitig laufen kann, bestimmen Hostregeln; die Seite ändert keine Aufgabensemantik. Beschäftigt, deaktiviert und Fehler sind Implementierungsverträge, die in den vier statischen Originalen nicht dargestellt sind.
- Einstellungs- und Aufgabenmenüs behalten Breite 240, Schrift 13 und minimale Zeilenhöhe 32, ein Verhältnis von 1:1 zur Inhaltsschrift 13; die Seite führt keinen eigenen Menüstil ein. Kartenschrift 16 ist eine Einstiegsüberschrift und keine Vergleichsbasis für Menüdichte.

<a name="xdzip-verification-and-boundaries"></a>

## Prüfung und Grenzen

Helles/dunkles Thema, Mindestfenster, Ziehzustände und Menü-/Titelleistenproportionen abdecken und DPR 1, 1,25 und 2 sowie tatsächlichen Browser-Seitenzoom prüfen. Ein herunterskaliertes Bildschirmbild ersetzt kein Layout, CSS-Skalierung keinen Browser-Seitenzoom. Ergebnisse, Bilder und Geometrieberichte gehören ausschließlich in lokale/CI-Artefakte; der Status steht im zugehörigen Issue.

Qt verwendet Regressionstests für Startseite und Browser und öffnet Dateien über den tatsächlichen Produkteinstieg. Automatische Tests ersetzen keine native Prüfung von Drag-and-drop, Bildschirmleser oder IME.

<a name="xdzip-real-qt-integration"></a>

## Tatsächliche Qt-Integration

WelcomePage und WelcomeAction drücken nur Layout und Absicht aus. Die Willkommensseite erscheint beim Start und nach dem Schließen eines Archivs; erfolgreiches Öffnen führt zur passenden Archivsitzung. Ein leeres Archiv bleibt ein gültiger Navigationszustand. Das anfängliche Laden deaktiviert erneutes Öffnen und Erstellen, zeigt „Wird geöffnet…“ und erlaubt den Abbruch des Ladens. Bei Fehler bleiben Willkommensseite und tatsächliche Fehlermeldung für eine neue Auswahl erhalten. Das Schließen eines Erstellungsentwurfs erzeugt keine Aufgabe. Hintergrundaufgabenlisten werden nicht zurückgesetzt.

Öffnen und Systemkürzel verwenden chooseArchive, Erstellen verwendet CreateWindow. Die native Dateiauswahl bleibt ein Systemfenster und Abbrechen erstellt keine Sitzung. Qt leitet gemäß gespeicherter Tab-/Fenstereinstellung, nutzt bevorzugt eine leere Startseite und aktiviert bei doppelten Archiven vorhandene Sitzungen.

Mit dem Desktop-Sitzungsmanager kann die Willkommensseite bis zu 128 lokale Dateien aus einem Drop öffnen. Jedes Element muss einer generierten Drag-and-drop-Endung oder einem unterstützten Namen für geteilte Volumes entsprechen. Der Entwicklungseinstieg mit einer Sitzung akzeptiert ein Element. Verzeichnisse und nichtlokale URLs werden abgelehnt. Diese Prüfungen wählen einen Öffnungsweg; die Rust-Laufzeit identifiziert weiterhin das tatsächliche Format und prüft die Quelle. Für Formate mit ausschließlich explizitem Öffnungseinstieg **Archiv öffnen** verwenden. Siehe [Formatregister](../engineering/format-registry.md).

Maße gelangen aus welcome-reference.json über generate-qml-theme.py in Theme. Größere Schrift vergrößert Karten und Inhalt; der Inhalt scrollt und bringt fokussierte Tastatureinträge automatisch ins Sichtfeld. Status- und Titelbereiche wachsen unabhängig ohne Item.scale. Qt nutzt gemeinsame PNG-Symbole mit der vorhandenen dunklen/hellen Grafik für die Hauptaktion; Laufzeitressourcen werden nicht aus Originalen ausgeschnitten.

Automatisierungseinstiege sind tst_welcome.qml, welcomeOpensRealArchivesAndReturns in browser_test.cpp und scripts/check-qt-welcome.py. Qt-Workflows auf Abruf für drei Plattformen prüfen tatsächliches Öffnen, Schließen, Wiederherstellung nach Fehlern und 36 Geometrieelemente bei DPR 1, 1,25 und 2 sowie Menübreite 240 und Schrift 13. Nativer Drop, Dateiauswahl, Screenreader und IME werden nach Bedarf geprüft; Offscreen-Ergebnisse werden nicht als nicht ausgeführte Desktopaktionen dargestellt.

Die Startseite ändert nur den Inhalt und verwendet dieselbe Qt-ArchiveTabs-Komponente. `screens/welcome-titlebar-*.png` bewahrt den Titelleistenentwurf. Der Regressionstest vergleicht Titelleiste, Tabs und Darstellungsschaltfläche vor und nach dem Öffnen eines Archivs.

Ladezustand: Der Abbrucheinstieg des anfänglichen Öffnens wurde in eine schwebende Inhaltsüberlagerung verschoben. Unbekannter Fortschritt nutzt einen aktiven Fortschrittsbalken ohne simulierte Prozentwerte; siehe [Vertrag der Ladeüberlagerung](welcome-opening.md). Ursprüngliche Willkommensbitmaps bleiben erhalten; der dynamische Entwurf bietet `opening=1`.

Fehler- und Warnhinweise erscheinen unter Formathinweisen mit gemeinsamer Gefahrenfarbe und Statuswarnsymbol. Anzeigen, Umbrechen, Löschen oder Ersetzen eines Fehlers ändert die oberen Koordinaten von Überschrift und Einstiegskarten nicht; langer Inhalt scrollt im Körper. Wird das Hauptfenster nach Schließen in den Infobereich erneut geöffnet, erscheint nur eine Startseite und Hintergrundaufgaben werden nicht abgebrochen. Die bearbeitbare Quelle ergänzt den Zustand `error=1`; alte Willkommens-PNGs behalten historische Bytes und dürfen den Vertrag zur oberen Positionierung nicht durch ihr früheres zentriertes Layout ersetzen.
