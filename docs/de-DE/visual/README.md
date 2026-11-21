# Visuelle Spezifikationen von xdzip

[English](../../visual/README.md) | [简体中文](../../zh-CN/visual/README.md) | [繁體中文](../../zh-TW/visual/README.md) | [日本語](../../ja-JP/visual/README.md) | [한국어](../../ko-KR/visual/README.md) | [Français](../../fr-FR/visual/README.md) | Deutsch | [Español](../../es-ES/visual/README.md) | [Português (Brasil)](../../pt-BR/visual/README.md) | [Русский](../../ru-RU/visual/README.md)

Einstiege in die Spezifikationen: [UI-Designstandard](design-standard.md), [verbindlicher Designablauf](design-workflow.md) und [Dichte- und Einheitenvertrag](../../../specs/visual/layout-scale.json). Die erneute Prüfung der Proportionen von Hauptfenster und Menü wird nach der [allgemeinen Methode](../../../developers/qa/README.md) erfasst.

Aktuelle Originale liegen in [design/hifi](../../../design/hifi/README.md); die interaktive HTML-Auslieferung in [design/html](../../../design/html/README.md). Nicht übernommene frühe Vorschläge sind nicht die aktuelle Grundlage. Alte Designs nicht anhand der reorganisierten main-Historie suchen; maßgeblich sind aktuelle Originale und Manifest.

[tokens.json](../../../specs/visual/tokens.json) enthält Themen und gemeinsame Implementierungsparameter; visuelle maschinenlesbare Eingaben liegen in `specs/visual/`. Geometrie und Schriften des Hauptfensters werden aus dem unabhängig eingefrorenen [qt-main-reference.json](../../../specs/visual/qt-main-reference.json) erzeugt, basierend auf hochauflösenden Originalen und akzeptierten HTML-Messungen, skaliert mit 1024/1548. Die frühere Grundlage mit Breite 1280 wurde in normale Dichte umgerechnet. Rundung, tatsächliche Screenshots und nicht implementierte Fähigkeiten werden nach der [Prüfmethode](../../../developers/qa/README.md) getrennt erfasst; Implementierungsparameter dürfen nicht ihre eigene Designabnahmegrundlage werden.

## Recherchegrundlage und Anpassung

1. [Offizielle Bandizip-Windows-Oberfläche](https://www.bandisoft.com/bandizip/screenshots/): Verzeichnisbaum, Dateitabelle, klaren Extraktionseinstieg und Statusleiste beibehalten; die große bunte Werkzeugleiste zu Text mit feinen Liniensymbolen vereinfachen und eine Hauptaktion betonen.
2. [Offizielle Bandizip-macOS-Oberfläche](https://www.bandisoft.com/bandizip.mac/screenshots/): offizielle Dateibrowsing-Demonstration und Archiverstellung prüfen; den Ablauf mit Browsing vor Auswahl von Inhalt und Extraktionsort beibehalten.
3. [Bandizip-Extraktionsanleitung](https://www.bandisoft.com/bandizip/help/how-to-unzip/): Das Öffnen einer Datei extrahiert sie nicht automatisch. Aufgaben beginnen erst, nachdem der Benutzer Ziel, Umfang und Konfliktbehandlung bestätigt hat.
4. [Offizielle Transmit-Oberfläche](https://panic.com/transmit/): Informationsdichte von Dateimanagern, Pfadnavigation und zurückhaltende Trenner als Referenz verwenden.
5. [Linear-Designüberarbeitung 2026](https://linear.app/now/behind-the-latest-design-refresh): ruhigere Seitenleisten, kompakte Navigation, weiche Grenzen und neutrale Hintergründe als Referenz nutzen, die den zentralen Inhalt betonen.

Die Recherche erfolgte am 2026-10-11 anhand öffentlicher offizieller Screenshots, Demonstrationen und Artikel; es wird nicht behauptet, dass sie die aktuellste installierte Version jedes Produkts darstellen. Recherchematerial Dritter bleibt in einem separaten experimentellen Arbeitsbereich und ist nicht in diesem Repository enthalten.

## Visuelles System

Warmgraue Hintergründe, kobaltblaue Hauptaktionen und ein graphitfarbenes dunkles Thema verwenden. Informationshierarchie nicht durch große Farbverläufe oder Milchglashintergründe ersetzen. Normale Dateilisten behalten eine klare flache Struktur; Schatten sind Overlays vorbehalten. Links beantwortet „Wo?“, die Mitte „Was ist hier?“ und rechts „Was ist ausgewählt?“.

HTML und Qt teilen Glyphen und Metriken von [Xdzip Design Sans SC](../../../design/assets/fonts/README.en.md); Qt verwendet TTF aus derselben Quelle und Plattformfallback für Archivnamen außerhalb der Teilmenge. Hauptlistentext hat 13, Header und Werkzeugleiste 12, Fenstertitel 13 logische Pixel, auf ganzzahlige Qt-Schriftpixel gerundet. Dateinamen behalten vollständige barrierefreie Namen; visuelle Kürzung verwendet Auslassungspunkte und einen Hoverhinweis mit vollem Namen. Dateisymbole teilen einen Slot von etwa 31.75 logischen Pixeln ohne typspezifische Größenkompensation.

Das Hauptfenster startet mit 1024 × 622, das Minimum beträgt 832 × 544 logische Pixel. Quellenmessungen für Seitenleiste und Vorschau betragen etwa 193.16 / 257.98, gerundet auf Qt-Strukturgrenzen 193 / 258. Verzeichniszeilen messen etwa 43, Dateizeilen 44.65. Fenstertitel sind 46 hoch, Tabellenheader 34, Suche und Hauptschaltfläche etwa 33.74. Verzeichnisbaum B behält unabhängig Höhe 32 und Symbolslot 16. Unter Breite 928 wird die Vorschau eingeklappt und über eine Schaltfläche in einem eigenständigen nativen Fenster geöffnet. Einstellungen bieten Schriftgröße 100%/125%/150%; historisches 200% wird als 150% gelesen. Große Schriften verwenden tatsächliches Layout, Scrollen und ein separates Detailfenster statt globaler Skalierung zum Verbergen von Überläufen. Ziehen von Bereichen und Speicherung ihrer Breite sind nicht implementiert.

## Hauptfensterinteraktion

- Qt unterstützt unabhängige Archivsitzungen und erzeugt abhängig vom gespeicherten Öffnungsmodus kapselartige Tabs oder Hauptfenster; erneutes Öffnen aktiviert eine bestehende Sitzung. Seiten behalten Pfad, Auswahl, Sortierung und Scrollzustand; Aufgaben laufen nach dem Schließen von Seiten weiter. Siehe den [Anwendungssitzungsvertrag](../engineering/application-sessions.md). HTML-Mehrtabverhalten validiert nur das Design.
- Die Hauptschaltfläche heißt ohne Auswahl „Alle extrahieren“ und mit Auswahl „Auswahl extrahieren“; das Dropdown bietet stets den anderen Umfang.
- Die Dateitabelle unterstützt Sortierung, Mehrfachauswahl, Bereichsauswahl und Tastaturnavigation. Doppelklick auf einen Ordner öffnet ihn; Doppelklick auf eine reguläre Datei extrahiert eine temporäre Kopie und öffnet sie mit dem System. Die Eigenschaftenkarte zeigt automatisch eine Vorschau einer einzelnen unterstützten Datei. Umfang und Sicherheit stehen im [Vorschauvertrag](../engineering/content-preview.md). HTML-Beispielfotos sind keine tatsächlichen Dekodierergebnisse.
- Die Suche umfasst rekursiv das aktuelle Verzeichnis und dessen Nachfahren; Suche im Stamm umfasst den gesamten Inhalt. Ergebnisse zeigen vollständige archivinterne Pfade, normale Verzeichnislisten die letzten Namen.
- Details können geschlossen und erneut geöffnet werden. Vorschau erfordert asynchrone Lesevorgänge und Ressourcengrenzen; nicht unterstützte Elemente zeigen Eigenschaften und Extraktionseinstieg. Verspätete Vorschauergebnisse nach Aufgabenabbruch verwerfen.
- Die Statusleiste zeigt Auswahlanzahl und Originalgröße, Indexzustand und Anzahl erneut öffnbarer Aufgabenverlaufseinträge. Die Anzahl aktiver Aufgaben steuert Hinweise und Beendigungsschutz separat. Einen unvollständigen Index als „Wird gelesen“ kennzeichnen, ohne eine vollständige Anzahl zu erfinden.

## Interaktion des Extraktionspanels

Das Panel gruppiert Archiv, Ziel, Auswahlumfang, Konfliktrichtlinie und Verhalten nach Abschluss. Die Beispielauswahl beträgt 8.4 + 6.2 + 12.8 = 27.4 MB. Die Dateiliste ist eine Zusammenfassung; längere Listen bieten „Alle anzeigen“.

Standardziel ist das letzte Verzeichnis. „Ordner mit Archivnamen erstellen“ aktualisiert die Vorschau des endgültigen Pfads sofort, um redundante Verschachtelung zu vermeiden. Text für Pfadberechtigungs-, Freiplatz- und Archivlesefehler anzeigen; Prüfung ist ein eigener Zustand und nicht bestätigter Platz „Unbekannt“.

Bei Konflikten ist Nachfragen Standard; Überspringen, automatisches Umbenennen und Überschreiben stehen ebenfalls bereit. Folgen des Überschreibens klar kennzeichnen. Während der Ausführung tatsächliche Phase, Anzahl verarbeiteter Dateien, Bytefortschritt und berechenbare Geschwindigkeit anzeigen; bei unbekannten Gesamten unbestimmten Fortschritt verwenden. Nach Abbruch „Wird angehalten“ anzeigen, bis der Hintergrund den Stopp bestätigt.

## Komponentenzustände und Barrierefreiheit

| Komponente | Erforderliche Zustände und Rückmeldung |
| --- | --- |
| Hauptschaltfläche | Normal, Hover, gedrückt, Tastaturfokus, deaktiviert und Übermittlung; mehrfachen Start verhindern |
| Dateizeile | Normal, Hover, ausgewählt, ausgewählt bei unfokussiertem Fenster und Tastaturfokus; Auswahl ändert mehr als Textfarbe |
| Suche | Leer, Eingabe, Suche, keine Ergebnisse und Leeren; Escape verlässt den Fokus und behält Text |
| Vorschau | Keine Auswahl, Laden, geladen, nicht unterstützt, Fehler und abgebrochen |
| Aufgabe | Wartend, laufend, wartend auf Passwort oder Konfliktentscheidung, anhaltend, abgeschlossen, teilweise fehlgeschlagen und fehlgeschlagen |
| Modales Panel | Fokusbegrenzung, Escape schließt ein nicht übermitteltes Panel, Fokus kehrt zur auslösenden Schaltfläche zurück |

Nach der Qt-Implementierung WCAG-AA-Textkontrast für Themenfarben prüfen. Fokusdarstellung und Verlassen folgen dem aktuellen Designstandard ohne zusätzliche Fokusringe. Namen und Zustandsansagen für Screenreader bereitstellen und die Systemeinstellung für reduzierte Bewegung respektieren. Empfohlene gewöhnliche Übergänge dauern 120–160 ms; dekorative Dateilisteneffekte nicht dauerhaft animieren.

## Qt-Quick-Implementierungsgrenzen

Qt verwendet gemeinsames `Theme` und erzeugte Maße. `ArchiveTabs`, `ArchiveToolbar` sowie Listen-, Raster- und Verzeichnisabschnitte in `SessionPage` teilen das Hauptlayout. `InspectorPanel`, `ExtractWindow`, `CreateWindow` und `TaskWindow` behalten eigene Daten- und Layoutgrenzen. QML stellt Zustand und Benutzerabsicht dar. Die Rust-Runtime verantwortet Indexierung, Archivlesen und -schreiben sowie Formatvalidierung.

macOS, Windows und Linux teilen Inhaltslayout und Interaktionsmodelle; Fenstersteuerungen, Menüs und Tastenkürzel folgen Plattformkonventionen. Der abgebildete macOS-Rahmen verlangt nicht, dass andere Plattformen Ampelknöpfe kopieren. Große Echtzeitunschärfe ist keine Implementierungsanforderung; eine statische Maske kann Hintergrundabschwächung in Screenshots nachbilden.

Das Hauptfenster in [hell](../../../design/hifi/sources/main.html?theme=light) oder [dunkel](../../../design/hifi/sources/main.html?theme=dark) zusammen mit [Aufgabenzuständen](../../../design/hifi/sources/task-windows.html), [Einstellungen](../../../design/hifi/sources/settings.html) und [Desktopintegration](../../../design/hifi/sources/desktop-integration.html) prüfen. Das [Fensterregister](../../../specs/visual/window-registry.json) dient zum Auffinden weiterer Zustände oder Größenausnahmen. Eine Designseite kann Inhalte eines nativen Fensters oder Menüs zeigen. Tatsächliche Dekoration, Aktivierung, Skalierung und Menüverhalten erfordern die betreffende Plattform.

## Revision der Plattformfenster

Operationspanels werden einheitlich zu [eigenständigen Plattformfenstern](../ux/platform-windows.md). Die Schließen-Schaltfläche im Inhaltsbereich, das abgerundete Panel und die Maske älterer Extraktionsgrafiken werden durch eigenständige Titelleiste und Plattformdekoration ersetzt; Farben, Typografie und Inhaltsstruktur bleiben. Aktuelle Dateitypsymbole, Listenausrichtung und Titelleisten der drei Plattformen besitzen einen interaktiven Prototyp als ergänzendes Prüfmaterial.

## Originale und gemeinsame Assetbibliothek

[Vollständige Originale](../../../design/hifi/README.md) liegen in `design/hifi/screens/`, einschließlich Hauptfenster, Extraktionsbestätigung und acht Tafeln mit Operationszuständen. Funktionen und Wiederherstellung beschreibt der [Zustandsvertrag](../ux/hifi-state-spec.md). Ursprüngliche Bytes bewahren; Originale und Ausschnitte dürfen nicht zur Anpassung an die Implementierung geändert werden.

Die [gemeinsame Assetbibliothek](../../../design/assets/README.md) bündelt Assets für Hauptfenster und Operationszustände statt sie nach Liefercharge aufzuteilen. SVG ist die einzige Grafikquelle für transparente Runtimesymbole, direkt in HTML eingebettet und in Qt als mehrstufige PNGs aus derselben Quelle gerendert. Fünfundvierzig Vergleichsausschnitte in Originalpixeln in `reference/` bewahren Originalgrafik, Zuschnitte und Hintergründe; drei HTML-Beispielfotos liegen in `demo/`. Ein Manifest kennzeichnet ihre Zwecke, damit Ausschnitte mit Hintergrund nicht mit transparenten Ebenen verwechselt werden.

Alle transparenten Symbole teilen eine 24×24-Zeichenfläche. PDF-, Text-, Code- und Markdown-Symbole teilen Mittelpunkt und 22 Pixel sichtbare Höhe, behalten aber unterschiedliche Silhouettenbreiten und innere Stile. Ordner verwenden eine zentrierte horizontale sichtbare Silhouette von 22×etwa 18.86. Diese Höhenvorgabe gilt für Runtimesymbole, nicht für Originalausschnitte. 1×/2×/3×-PNGs sind 24×24, 48×48 und 72×72; Skalierung ändert die logische Größe nicht. HTML-Dateilisten verwenden 48×48-Slots im Quellraum; Qt verwendet nach äquivalenter Umrechnung etwa 31.75×39.69 logische Pixel. Typspezifische CSS-Vergrößerung, Verschiebung und Strichkompensation entfernen.

Dateilisten-Breadcrumbs und rechte Vorschau-/Detailtitel teilen Balkenhöhe, Schriftgröße und vertikale Textmitte. HTML verwendet gemeinsame Titelleistenvariablen; Qt verwendet `PaneHeader` und `Theme.mainPaneTitleSize`. Titel bleiben über scrollendem Inhalt, und große Schriften erhöhen beide Bereichshöhen gemeinsam. Tatsächliche Messungen werden gemäß [Prüfmethode](../../../developers/qa/README.md) in Ausführungsartefakten bewahrt.

HTML-Fotovorschauen beginnen unmittelbar unter dem Bereichstitel, oben am Dateilistenheader ausgerichtet und ohne zusätzlichen oberen Abstand. Bei Suchhinweisen entsprechend dem tatsächlich belegten Raum ausrichten. Bestehende Bildhöhe und unteren Abstand beibehalten. Erneute Fotoausrichtungsprüfungen umfassen zwei Beispielbilder, helle/dunkle Themen, drei Fensterdekorationen und Suchänderungen.

Header und Werte für Original- und komprimierte Größe in Dateilisten sind rechtsbündig; Typheader und -inhalt linksbündig, mit gemeinsamer Inhaltsauffüllung je Spalte. HTML behält ursprüngliche Spaltenbreiten und Zeilenhöhen: rechte Auffüllung der Größenspalten 26 und 22 Quellraumpixel, linke Auffüllung der Typspalte 28 Quellraumpixel. Die erneute Prüfung erfasst Textlayoutkanten aller sichtbaren Zeilen.

Symbolsilhouette, Abmessungen von Originalpixelausschnitten und Laufzeitzeichenfläche getrennt erfassen. Ordner müssen nicht in Papierformen gestreckt werden; SVGs dürfen nicht zum Füllen von Slots verzerrt werden. Visuelle Änderungen an der Grafikquelle vornehmen, in der Assetgalerie prüfen und dann beide Implementierungen synchronisieren.

## Revision Verzeichnisbaum B (2026-10-12)

Drei formale Tafeln definieren mehrstufiges Aufklappen, Knoteninteraktion und Navigationsgrenzen; siehe die [Verzeichnisbaumspezifikation](../ux/directory-tree.md). Sechsundzwanzig Originalpixelausschnitte gehen in gemeinsames `assets/reference` ein; Verzweigungspunkte, gefüllte Statussymbole und segmentierte Spinner verwenden SVGs für zwei Themen und PNGs in drei Auflösungen. Verbindungslinien, Zustandshintergründe, Fokus und Text werden dynamisch aus [Komponentenparametern](../../../specs/visual/directory-tree.json) gezeichnet; diese überschreiben globale Tokens nicht automatisch. Tafeln sind vergrößerte visuelle Erläuterungen; genaue logische Maße und Quellpixelrechtecke werden separat erfasst. HTML und Qt integrieren rekursive Verzeichnisse, Verbindungslinien und Marker für verborgene aktuelle Zweige; spezielle Grenzsymbole/Skelette bleiben Designressourcen und eine vollständige dunkle Tafel steht noch aus.

## Aktuelle Hauptdesign-Werkzeugleistenrevision (2026-10-12)

Frühere allgemeine Steuerungsvorschläge oben gelten nicht mehr für die Hauptwerkzeugleiste. Aktuelle Originalmessungen sind 51 Quellpixel Höhe, 18 Quellpixel Schrift und 32 Quellpixel Symbol, umgerechnet mit 1024/1548 auf etwa 33.74 Höhe, 12 ganzzahlige Schrift und 21.17 Symbolzeichenfläche. Die Höhe des Werkzeugleistenbereichs ist 53. Die frühere Umrechnung bei Breite 1280 von 42.17/15/26.46 wurde durch normale Dichte ersetzt; gewöhnliche Operationsfenster behalten ihre registrierten Zuordnungen.

Gewöhnliche linke/rechte Werkzeugleistenaktionen und Such-/Hauptaktionen bilden Gruppen. Große Schriften können die Höhe vergrößern und die Suche verkleinern, damit rechte Aktionen bei minimaler Fenstergröße vollständig sichtbar bleiben. Deaktivierter Text erhält keine Gruppenopazität; die Hauptschaltfläche hat einen eigenen deaktivierten Hintergrund. Die Extraktionshauptschaltfläche behält das deaktivierte Aussehen von Trenner und Pfeil; tatsächliche Extraktion und Umfangsmenüs folgen der Runtimefähigkeit. Ohne Auswahl sind Auswahlumfang und Metadatenvorprüfung deaktiviert; Integritätsprüfung folgt der Archivfähigkeit. Erneute Validierung nach der [allgemeinen Prüfmethode](../../../developers/qa/README.md) erfassen.

## Anwendungsmarke und Plattformsymbole

Systemdokumentsymbole je Format verwenden die [Dateizuordnungssymbol-Spezifikation](file-association-icons.md) und die [hochaufgelöste Prüfseite](../../../design/hifi/sources/file-association-icons.html). Diese Familie verwendet die untenstehende Markengeometrie mit unterschiedlichen Formatlabels, Akzenten und Vereinfachungen für kleine Größen als Designassets für spätere formatspezifische Integration.

Die Marke verwendet vier verflochtene Bänder und einen offenen U-förmigen unteren Abschluss; Geometrie wird in `scripts/brand_assets.py` gepflegt. Eigenständiges Zeichen, Anwendungssymbol und ZIP-Dokumentmaster liegen in `design/assets/brand/`. UI-`app-mark`, `archive` und `archive-grey` kennzeichnen Anwendung, blaues Archiv und graues Archiv und werden aus derselben Markengeometrie erzeugt. Die Anwendung verwendet einen blauen abgerundeten Außenrahmen, Dokumente Papier mit gefalteter Ecke: Die Identität ist gemeinsam, die Rahmen haben unterschiedliche Rollen.

Das aktuelle Zeichen behält ein schlankes Seitenverhältnis 2:3 und wird nicht auf die Breite des alten Zeichens gestreckt. Seine Basis verwendet an allen vier Ecken eine rotationssymmetrische Kurve mit stetiger Krümmung, die von null an geraden Kanten bis zur diagonalen Mitte gleichmäßig zunimmt und eine abgeschrägte Mitte vermeidet. Beleuchtung und Basis teilen einen Pfad; Striche unterschiedlicher Radien oder offene Glanzlinien nicht überlagern. Dokumentabzeichen verwenden dieselbe Konturdefinition.

Dies ist eine ausdrückliche neue Markenrevision und keine ununterscheidbare Nachbildung des ZIP-Reißverschlusssymbols in alten hochaufgelösten Originalen. Zehn vollständige Originale, 45 Originalpixel-Symbolreferenzen und drei Beispielfotos behalten ihre ursprünglichen Bytes. Das neu erzeugte `brand/identity-reference.png` ist eine Prüfreferenz für die aktuelle Markenrichtung, kein Runtimeasset. Das Hauptmanifest erfasst Referenzabdeckung und kennzeichnet alte Markenreferenzen ausdrücklich als historische Designrollen.

Kleine UI-Symbole behalten 24×24-Zeichenflächen. Plattformsymbole exportieren transparente PNGs mit 16/24/32/48/64/128/256/512/1024 Pixeln. Die Größen 16/24/32 vereinfachen unleserlichen Text und Beleuchtung großer Bilder und bewahren verflochtene Bänder und U-förmige Erkennungsstruktur. Anwendungs- und ZIP-Dokumentsymbole bieten jeweils ICO mit sieben Größen und ICNS mit elf Dichtedarstellungen. Ein 24-Einheiten-UI-Symbol nicht unbegrenzt für Systemnutzung vergrößern oder große Schatten zu Kleingrößenrauschen verkleinern.

Systemanwendungssymbole sind in Qt-Fenster, macOS-App-Bundles, Windows-Programmressourcen und Linux-Desktoplauncher/hicolor-Verzeichnisse integriert. Plattformformate des ZIP-Dokuments werden von Installern und Dateizuordnungen verwendet; Systemöffnungsanfragen führen über einheitliches Routing zu tatsächlichem Browsing. Siehe [Installations- und Zuordnungsvertrag](../engineering/installers-associations.md). Nur Öffnungskandidaten registrieren, ohne Benutzerstandards zu ändern. Plattformprüfung nach der [allgemeinen Methode](../../../developers/qa/README.md) erfassen.

Fehlende Themen-, Erfolgs-, Abbruch-, Konflikt- und Fortschrittsgrafiken werden aus Originalpixelreferenzen als unterschiedliche semantische Assets vervollständigt. Große Abbruchgrafik verwendet ein quadratisches Stoppsymbol; das Systemthema behält die vollständige Fensterstruktur. HTML verwendet native Radioeingaben, Qt verwendet IntentRadioButton, um Maus-, Tastatur- und Barrierefreiheitsaktionen zu einem Auswahlmodell zu leiten; Assets behandeln nur das Aussehen. Generierungsprüfungen, Anzeige auf Zielplattform und Abweichungen von Originalen separat erfassen; bestehende Abnahme ersetzt die Ergebnisse dieser Revision nicht.

## Spezifikation interaktiver Steuerelemente

[controls.json](../../../specs/visual/controls.json) definiert Menüspalten, Zeilenhöhen, Radien, Farben, Fokus und Hinweiszeiten; [directory-tree.json](../../../specs/visual/directory-tree.json) definiert die Verzeichnishierarchie. Beide erzeugen HTML-Steuerungsparameter und QML Theme; Prüfungen vermeiden manuelle Duplizierung von Maßen und Themen. Logische/Quellpixel-Verhältnisse des Hauptdesigns stammen aus `display-spec.js`; die [Komponentenprüfseite](../../../design/html/components.html) verwendet logische Maße 1:1, während die Hauptzeichenfläche Quellkoordinaten verwendet. Funktionslisten und Eingabeverträge stehen in der [Hauptfensterkomponentenbeschreibung](../ux/main-window-components.md); tatsächliche Darstellung und Verhalten werden gemäß [Prüfmethode](../../../developers/qa/README.md) in entsprechenden Issues und Artefakten erfasst.
