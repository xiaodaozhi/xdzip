# Verzeichnisbaum B: Design, Ressourcen und Integration

[English](../../ux/directory-tree.md) | [简体中文](../../zh-CN/ux/directory-tree.md) | [繁體中文](../../zh-TW/ux/directory-tree.md) | [日本語](../../ja-JP/ux/directory-tree.md) | [한국어](../../ko-KR/ux/directory-tree.md) | [Français](../../fr-FR/ux/directory-tree.md) | Deutsch | [Español](../../es-ES/ux/directory-tree.md) | [Português (Brasil)](../../pt-BR/ux/directory-tree.md) | [Русский](../../ru-RU/ux/directory-tree.md)

Status: **Die grundlegende Verzeichnisinteraktion ist in HTML-Beispielen und der schreibgeschützten Qt-Navigation integriert; Ressourcen für Grenzzustände warten auf Integration**. Die Interaktion folgt dem [Vertrag für schreibgeschützte Navigation](../engineering/readonly-browser.md). Die gewöhnliche Verzeichnisnavigation verwendet direkte Kinder, die Suche erfasst den aktuellen Bereich rekursiv, und die Ausgabeberechtigung bleibt von der schreibgeschützten Navigation getrennt.

Die drei aktuellen Artboards zeigen [mehrstufiges Aufklappen](../../../design/hifi/screens/directory-tree-capsule.png), [Knoten- und Interaktionszustände](../../../design/hifi/screens/directory-tree-states.png) sowie [Leerzustände und Navigationsgrenzen](../../../design/hifi/screens/directory-tree-boundaries.png). Die gemeinsame Galerie zeigt vollständige Artboards, die [Ressourcenbibliothek](../../../design/assets/index.html#directory-tree) einzelne Ausschnitte. Das mehrstufige Aufklappen wird aus der editierbaren Quelle exportiert; die Titelleiste ist mit den aktuellen Kapseltabs synchronisiert. Die obere und mittlere Liste auf dem Board liefern nur Kontext und definieren Titelleiste, Werkzeugleiste oder Dateitabelle nicht neu. Das [historische mehrstufige Original](../../../design/hifi/screens/directory-tree.png) bleibt unverändert als Quelle von 26 Ausschnitten und unabhängige Messreferenz erhalten.

Artboards sind vergrößert, und einzelne Karten haben unterschiedliche Maßstäbe. Text, Radien und Pixelmaße in Bildern dürfen Steuerungsgrößen nicht direkt festlegen; [Komponentenparameter](../../../specs/visual/directory-tree.json) dokumentieren implementierte logische Maße ausdrücklich, und das Ressourcenmanifest hält physische Ausschnittskoordinaten separat fest. Die Implementierung nutzt eine logische Breite von 1024 und eine gemeinsame Transformation der Quellzeichenfläche; den gesamten neuen Komponentensatz nicht erneut vergrößern.

Qt verwendet die folgenden logischen Werte direkt. Das bestehende HTML-Hauptdesign bildet gemäß layout-scale.json ein Fenster mit 1548 Quellpixeln auf 1024 logische Pixel ab: `source CSS = logical / (1024/1548)`. Damit entsprechen Zeilenhöhe 32 genau 48.375 Quell-CSS-Pixeln, Schrift 11.2 entspricht 16.93125, Symbolplatz 16 entspricht 24.1875 und Einzug 12.8 entspricht 19.35. Die Vorschauskalierung gilt nach der Umrechnung gleichmäßig für das gesamte Fenster. DPR bestimmt nur Abtastdichte und Linienausrichtung und geht nicht in die Umrechnung logischer Größen ein.

## Baumstruktur und Anzahlen

Der vollständige Index liefert explizite und implizite Verzeichnisse. Ein Blatt bedeutet **keine Unterverzeichnisse**, nicht keine Dateien. Hat Glacier Lake ein Unterverzeichnis, besitzt es unabhängig vom Pfeil des Elternknotens einen eigenen Aufklapppfeil.

```text
Root                               ← current directory
▼ 01 Photos                     12
  ▼ Glacier Lake                 6
    ▸ Lakeside Trail             3  ← deeper directory, not expanded
      Snow Mountain View          3  ← files only, retains empty arrow slot
  ▸ Mountain Sunrise             4
    Other Photos                 2  ← no subdirectories
▸ 02 Documents                   8
  03 Maps                        4  ← no subdirectories
```

Das Beispielarchiv enthält 12 Fotos, 8 Dokumente, 4 Karten und 6 Dateien in der Wurzel; die Wurzelliste zeigt drei direkte Verzeichnisse und sechs direkte Dateien ohne aufgeklappte Nachfahren. Auch Nachfahren zugeklappter Zweige sind vollständig in der Parameterdatei aufgeführt, damit Blätter keine falschen Pfeile erhalten. Das explizit leere Verzeichnis auf einem anderen Board ist ein separates Zustandsbeispiel und wird diesem Beispiel mit 30 Einträgen nicht hinzugefügt.

Verzeichniszahlen stehen für alle **physischen Archivdatensätze** unter dem Präfix, einschließlich doppelter Namen, aber ohne den mit dem Präfix identischen Verzeichnisdatensatz. Sie zählen keine Unterverzeichnisse und ändern sich nicht durch Namenssuche. Baumzeilen zeigen keine Anzahlen. Die Archivzusammenfassung behält die vollständige Anzahl physischer Datensätze, während rechts im Listentitel die Zeilenzahl des aktuellen Verzeichnisses steht; die Definitionen unterscheiden sich. Gewöhnliche Listen zeigen den letzten Namen, Suchergebnisse den vollständigen Pfad im Archiv.

Pfeile hängen nur von bekannten Unterverzeichnissen ab: nach rechts bei zugeklapptem und nach unten bei aufgeklapptem Elternknoten. Knoten mit nur Dateien oder leere Knoten haben keinen Pfeil, aber gleichrangige Knoten behalten identische Pfeil- und Symbolplätze sowie Textanfänge. Beim Laden des vollständigen Index keine knotenweisen Pfeile, Zahlen oder Strukturen erraten und keine asynchronen Ladezustände pro Knoten hinzufügen.

## Layout, Schrift und dynamische Geometrie

| Element | Normale logische Größe / Verhalten |
| --- | --- |
| Standardbreite der Seitenleiste | 193.16; die strukturelle Grenze des Hauptfensters wird auf 193 gerundet |
| Zeilenhöhe | Mindestens 32; wächst bei größerer Schrift gemäß Parameterformel |
| Einzug je Ebene | 12.8; unabhängig vom Knotentyp |
| Pfeil-Trefferbereich / Grafikplatz | Mindestens 19.2×19.2 / 14.4×14.4; Blätter behalten denselben Platz |
| Ordnersymbolplatz | 16×16; gemeinsames SVG mit 24 Einheiten wird proportional skaliert und ersetzt den 18er-Platz des aktuellen Baums |
| Symbol-Text-Abstand / Endabstand | 6.4 / 16; keine sichtbare Anzahl |
| Fließtext | 11.2; Qt rundet auf 11 und verwendet Schrift aus derselben Quelle |
| Datensatzanzahl | In Daten und zugänglicher Semantik erhalten; nicht in Zeilen angezeigt und keine Zahlenspalte reserviert |
| Aktuelles Verzeichnis | Hellblauer Hintergrund, rechte Ecken 6.4, gerade linke Kante und 2.4 breiter Indikator über die gesamte Höhe |
| Tastaturfokus | Kein zusätzlicher Umriss oder Fokusbalken und keine Änderung der Zeilenbelegung; Indikator des aktuellen Verzeichnisses bleibt |
| Hierarchielinien | Breite 1; zurückhaltende Hierarchie gegenüber dem Hintergrund |
| Marker für verborgenes aktuelles Verzeichnis | Punkt mit 4.8 Einheiten in transparentem Platz mit 19.2 Einheiten, auf Zeilenmitte des Namens ausgerichtet |

Verbindungslinien werden aus der sichtbaren Vorfahrenkette und den Markierungen letzter Geschwister dynamisch gezeichnet. Vertikale Elternlinien verlaufen durch aufgeklappte Teilbäume bis zur Mitte des letzten Geschwisterknotens; die letzte Verbindung endet L-förmig und reicht nicht in die nächste Gruppe der obersten Ebene. Die X-Mitte jeder vertikalen Linie liegt auf der X-Position der abwärts gerichteten Pfeilspitze des Elternknotens, auch bei größeren barrierefreien Schriften. Innere Linien enden unabhängig; ein allgemeines Linienbild kann nicht alle Zeilen überspannen. Segmente am Ziel-DPR ausrichten; Text, Pfeile und Ordner teilen die vertikale Zeilenmitte.

Der Hierarchiebereich scrollt horizontal. Die Breite verwendet nur tatsächliche Namen, Einzug und 16 Endabstand der aktuell aufgeklappten sichtbaren Knoten. Namen bleiben ohne überlagerte Zahlen vollständig. Nach dem Zuklappen die Breite neu berechnen, damit verborgene tiefe Knoten keinen Leerraum hinterlassen; der linke Indikator bleibt am Rand des Ansichtsbereichs fixiert. Beim Überfahren einer Zeile erscheint kein schwebender Tooltip; vollständige Pfade sind in barrierefreien Beschreibungen und Details verfügbar. Vertikales Scrollen findet nur im Verzeichnisbereich statt; Scrollleisten umschließen nicht den Hauptfensterrahmen.

Alle Verzeichnisse verwenden dieselbe Ordnerressource. Zweigpunkte, gefüllte Informations-/Warnsymbole und segmentierte Ladeindikatoren liegen als helle/dunkle SVGs und 1×/2×/3×-PNGs vor; Linien, Fokusrahmen, Hintergründe, Skelette, Text und Hinweise sind dynamische Komponenten. Dunkle Parameter verwenden das globale Thema mit zusätzlichen Linien- und Markerfarben; **das vollständige dunkle Verzeichnisbaumoriginal steht noch aus**, und dunkle Symbolexporte ersetzen nicht das vollständige Artboard. Bestehende Ergebnisse zu Rendering und Desktop-Abnahme werden separat in zugehörigen Issues und Artefakten dokumentiert.

Konsistenz der HTML-/Qt-Interaktion gemäß [Verifizierungsmethode](../../../developers/qa/README.md) festhalten. Artboards und Ausschnitte bewahren die ursprüngliche Designhistorie; aktuelle Zeilenzahlen, Fokus und Ecken folgen dieser Seite und der logischen Spezifikation.

## Interaktionen und unabhängige Zustände

Aufklappzustand, aktuelles Verzeichnis, Hover und Tastaturfokus werden unabhängig gespeichert und können gleichzeitig bestehen. Das aktuelle Verzeichnis unterscheidet sich auch von der Kontrollkästchenauswahl der Dateien rechts.

Die Wurzel ist ein separat navigierbarer Eintrag, immer aufgeklappt und nicht an gewöhnlichem Auf- oder Zuklappen von Knoten beteiligt. Öffnen Rechtsklick, Menütaste oder Umschalt+F10 ihr Menü, bleibt „Unterverzeichnisse zuklappen“ ohne sichtbare Erklärungszeile oder Tooltip deaktiviert. Die barrierefreie Beschreibung lautet „根目录始终展开“ / „Root is always expanded“; das Fehlen einer gewöhnlichen Baumzeile bedeutet nicht, dass keine Unterverzeichnisse vorhanden sind. Das Auf- und Zuklappen gewöhnlicher Eltern hängt von tatsächlichen Unterverzeichnissen ab; gewöhnliche Blätter behalten die barrierefreie Beschreibung „This node has no subdirectories“. Gründe sind nur barrierefrei zugänglich, nicht als sichtbare Hinweise. Das Öffnen des Menüs oder Anklicken deaktivierter Elemente darf Verzeichnis, Dateiauswahl oder Baumaufklappung nicht verändern.

| Aktion | Ergebnis |
| --- | --- |
| Pfeil anklicken | Nur diesen Knoten auf- oder zuklappen; Verzeichnisabfrage, Dateiauswahl und Details bleiben unverändert |
| Name anklicken oder Eingabetaste | Mit Verzeichnis-ID der aktuellen Sitzung eintreten; nach erfolgreichem Ersetzen der Abfrage Abfrage des aktuellen Verzeichnisses, Dateiauswahl und Details gemäß bestehendem Vertrag zurücksetzen |
| Auf / ab | Fokus zwischen sichtbaren Verzeichniszeilen bewegen, ohne die Dateiliste automatisch umzuschalten |
| Rechts | Zugeklappten Elternknoten aufklappen, bei bereits aufgeklapptem zum ersten Kind wechseln; Blätter bleiben stehen |
| Links | Aufgeklappten Elternknoten zuklappen, sonst zum Elternknoten wechseln; Knoten oberster Ebene bleiben stehen |
| Vorfahren des aktuellen Verzeichnisses zuklappen | Aktuelles Verzeichnis und Dateiliste behalten; verborgener Fokus kehrt zum nächsten sichtbaren Vorfahren zurück |
| Neue Archivsitzung | Alte Knoten-IDs, Aufklappzustand, Fokus und Zweigpunkte löschen; keinen Zustand eines anderen Archivs wiederverwenden |

Beim Pfad `01 Photos / Glacier Lake` bleibt zum Beispiel Glacier Lake rechts angezeigt, wenn `01 Photos` zugeklappt wird. Die Photos-Zeile zeigt nur einen blauen Punkt am Ende, ohne den Hintergrund des aktuellen Verzeichnisses von Glacier Lake zu erhalten; auch die Wurzel wird nicht aktuell. Der blaue Punkt steht am **nächsten sichtbaren zugeklappten Vorfahren**; ein verborgenes aktuelles Verzeichnis hat einen Zweigmarker. Erneutes Aufklappen stellt den aktuellen Hintergrund von Glacier Lake wieder her.

Der Baum selbst ist ein einzelner Einstieg für die Tastaturnavigation. Knoten behalten barrierefreie Semantik für stabile ID, Name, Ebene, Aufklappzustand, Position und Datensatzanzahl. Blätter geben sich nicht als aufklappbar aus; dekorative Ordner, Linien und Punkte werden nicht redundant angekündigt. Verborgene aktuelle Pfade besitzen barrierefreie Beschreibungen und verlassen sich nicht allein auf Blau. Pfeiltasten und Pfeilklicks dürfen nicht an die Zeilenaktivierung weitergegeben werden.

## Zustandsabdeckung und Wiederherstellung

| Zustand | Verzeichnisbaum | Dateiliste / Wiederherstellung |
| --- | --- | --- |
| Lesen | Lademeldung, Anzahl unbekannt; Skelett ist zukünftige Designressource; Navigation deaktiviert | Globalem Ladezustand folgen; Baum nach Veröffentlichung des vollständigen Index einmal ersetzen |
| Leeres Archiv | Wurzelzeile ohne Zahl; „Dieses Archiv ist leer“ | Leere Liste; ein anderes Archiv kann weiterhin geöffnet werden |
| Nur Dateien in der Wurzel | Wurzelzeile ohne Zahl; „Keine Unterverzeichnisse“ | Sechs Dateien normal anzeigen; nicht die ganze Seite in einen Leerzustand versetzen |
| Verzeichnisindexgrenze überschritten | Keinen halben Baum veröffentlichen; „Verzeichnis überschreitet Anzeigegrenze“ anzeigen | Wurzel-, Such- und zulässige explizite Verzeichnisnavigation innerhalb aktueller Fähigkeiten fortsetzen |
| Lesefehler | Navigation abdunkeln und ausdrücklichen Fehler anzeigen | Archiv erneut öffnen; Fehler nicht als null oder leeres Verzeichnis interpretieren |
| Knoten mit Unterverzeichnissen | Unabhängiger Pfeil auf jeder Ebene | Aufklappen betritt das Verzeichnis nicht |
| Nicht leeres / leeres Blatt | Anzahl bleibt in Daten; keines zeigt Zeilenzahl oder Pfeil | Name bleibt aktivierbar; leeres Verzeichnis kann betrachtet werden |
| Langer Name / tiefe Verschachtelung | Vollständiger Name, barrierefreier Pfad, horizontales Scrollen und Breite nach sichtbaren Knoten | Pfadfilterung bleibt konsistent |
| 150 % Schrift (200 % nur historischer Belastungsfall) | Plätze und Zeilen wachsen; Linien werden neu auf Zeilen zentriert | Namen, Pfeile und Fokus werden nicht abgeschnitten |
| Verborgenes aktuelles Verzeichnis | Nächster sichtbarer Vorfahr zeigt Zweigpunkt | Liste behält aktuellen Pfad; erneutes Aufklappen stellt aktuelle Zeile wieder her |

Navigationsgrenzen und globaler Lesefehler sind verschiedene Zustände; Gründe, verfügbare Aktionen und Quellen der Anzahlen dürfen keine Vorlage „leeres Verzeichnis“ teilen. Die bestehenden Budgets von 32.768 Knoten, 64 Ebenen und 4 MiB Verzeichnisnamen stammen aus dem schreibgeschützten Vertrag; das Design hebt sie weder an noch umgeht es sie.

## Ressourcenlieferung und tatsächliche Integration

Die 26 Ausschnitte in Originalpixeln liegen unter `design/assets/reference/tree-*.png`, jeweils mit Quelle, SHA-256, `x,y,width,height` und Zweck. `component_reference_coverage` verweist einzeln auf gemeinsame Vektoren oder dynamische Komponenten. Statische Bilder ganzer Zeilen nicht als interaktive Steuerungen beschreiben und weiße Hintergründe der Ausschnitte nicht entfernen, um Transparenz vorzutäuschen.

| Teil | Ressource / Zeichnung | Aktuelle Integration |
| --- | --- | --- |
| Ordner, Rechts-/Abwärtspfeile | `folder`, `chevron-right`, `chevron-down` aus gemeinsamen Ressourcen | Bestehende gemeinsame Ressourcen verfügbar; B-Symbole und Maße in rekursiven HTML-Zeilen und Qt-Delegates verwendet |
| Laden, Information, Warnung | Neue `tree-spinner`, `tree-info`, `tree-warning` | Teil von acht hellen/dunklen SVGs und 24 transparenten PNGs; gefüllte und segmentierte Boardformen beibehalten statt vorhandener Umrisssymbole |
| Verborgener aktueller Zweig | Neues `tree-active-branch` | Sechs transparente PNGs für zwei Themen geliefert; Qt markieren damit den nächsten sichtbaren zugeklappten Vorfahren |
| Linien, aktueller Hintergrund, Fokus, Skelett, Hinweise | Dynamisch mit Maßen und Themen der Parameterdatei gezeichnet | Stil und Logik spezifiziert; kein zweiter Satz Bitmap-Steuerungen geliefert |
| Offizielle Artboards | `hifi/screens/directory-tree*.png` | Bytes bestätigter Originale zum Designvergleich erhalten |

Qt integrieren Verbindungslinien, Zweigpunkte, Ordnerplätze mit 16 Einheiten und Verzeichniskontextmenüs. HTML verwendet eine vollständige rekursive Beispielhierarchie, Qt den tatsächlichen begrenzten Index. Das Zuklappen des aktuellen Vorfahren sendet keine Abfrage. Ladeskelette und eigene Informations-/Fehlersymbole bleiben Designressourcen; das vollständige dunkle Artboard steht aus. Symbolintegration belegt keine plattformweise Screenreader- oder HiDPI-Abnahme; bestehende manuelle Ergebnisse folgen [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5).

## Abnahme nach Integration

1. Den tatsächlichen Verzeichnisbereich bei gleicher logischer Breite 1024, Schriftkonfiguration, DPR und Zustandsdaten erfassen und mit dem entsprechenden bestätigten Artboardbereich und der Spezifikation vergleichen. Maßstabsverhältnis dokumentieren; visuelle Beurteilung und strukturelle Assertions separat aufbewahren.
2. Pfeil-, Symbol- und Namensanfänge auf jeder Ebene ausrichten; keine numerische Zeilenspalte. Symbole, Text, Punkte und Pfeile teilen Zeilenmitten. Angestrebte Geometrieabweichung höchstens ein physisches Pixel; bei Schriftunterschieden Plattform und tatsächlichen Fallback dokumentieren.
3. Mindestens drei Ebenen einschließlich eines tieferen zugeklappten Zweigs und eines nicht leeren Blatts aufklappen. Kontinuität und Ende der Verbindungslinien bis zu letzten Geschwistern und durch aufgeklappte Teilbäume abnehmen.
4. Aktuell, Hover und Fokus getrennt und gemeinsam prüfen. Pfeilklicks senden keine Verzeichnisabfrage und löschen keine Dateiauswahl; Aktivierung mit Eingabe folgt der Rücksetzregel bei erfolgreicher Abfrage.
5. Bei aktuellem Glacier Lake Photos zuklappen: Listenpfad bleibt, Wurzel ist nicht ausgewählt und Photos zeigt nur einen Zweigpunkt. Erneut aufklappen, um die aktuelle Zeile wiederherzustellen. Tastaturnavigation bleibt möglich, wenn der Fokus verborgen ist.
6. Leere Archive, Wurzeldateien, explizite leere Verzeichnisse, Lesefehler und Verzeichnisgrenze mit einem echten begrenzten Index prüfen; Anzahlen, Deaktivierung und Wiederherstellungsaktionen separat prüfen.
7. Bei langen Namen, 100 %/125 %/150 % Schrift und historischem 200-%-Belastungsfall, minimalem Fenster und maximal zulässiger Indextiefe bleiben Namen vollständig sichtbar, horizontales Scrollen betrifft nur die Hierarchie und vollständige Pfade sind barrierefrei zugänglich. Baumscrollen bewegt weder Archivzusammenfassung noch untere Statistik.
8. Tatsächliche helle/dunkle Darstellung, normale/Retina/hohe Windows-DPI, Tastatur und Screenreader abnehmen. Erfolgreiche Bildprüfungen oder Ressourcengenerierung ersetzen diese Laufzeitprüfungen nicht.

Gemäß [Verifizierungsmethode](../../../developers/qa/README.md) Originale, Ausschnitte, Ressourcen und tatsächliches Maus-, Tastatur-, Menü- und Baumverhalten separat prüfen. Nicht getestete Barrierefreiheit und Plattformunterschiede ausdrücklich kennzeichnen.
