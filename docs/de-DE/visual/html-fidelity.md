<a name="xdzip-html-visual-fidelity-review"></a>

# Prüfung der visuellen HTML-Treue

[English](../../visual/html-fidelity.md) | [简体中文](../../zh-CN/visual/html-fidelity.md) | [繁體中文（台灣）](../../zh-TW/visual/html-fidelity.md) | [日本語](../../ja-JP/visual/html-fidelity.md) | [한국어](../../ko-KR/visual/html-fidelity.md) | [Français](../../fr-FR/visual/html-fidelity.md) | Deutsch | [Español](../../es-ES/visual/html-fidelity.md) | [Português (Brasil)](../../pt-BR/visual/html-fidelity.md) | [Русский](../../ru-RU/visual/html-fidelity.md)

Hochauflösende Entwurfsbilder sind die visuelle Referenz; HTML ist die bearbeitbare Umsetzung. Maße oberhalb und unterhalb der Werkzeugleiste werden unabhängig am [eingefrorenen ursprünglichen Hauptfenster](../../../design/hifi/screens/main-light.png) gemessen und nicht rückwärts aus dem aktuellen CSS abgeleitet. Die aktuelle Anzeige verwendet den [hellen Entwurf mit Kapsel-Tabs](../../../design/hifi/screens/main-capsule-light.png) und den [dunklen Entwurf](../../../design/hifi/screens/main-capsule-dark.png), exportiert aus der [bearbeitbaren Quelle](../../../design/hifi/sources/main.html); sie ersetzen keine unabhängige Messung. Die Titelleiste wird anhand der ausdrücklichen Revision in layout-scale.json abgenommen. Originalgrafiken besitzen weiche Kanten und generierte Glyphen; eine Testanzahl belegt keine pixelgenaue Übereinstimmung der ganzen Seite.

<a name="xdzip-fixed-reproducible-conditions"></a>

## Feste reproduzierbare Bedingungen

- Die Quellfläche entspricht dem PNG mit 1586×992; das Fenster misst 1548×940 am Ursprung `(20,30)`. Ursprüngliche Ankerprüfungen verwenden weiterhin einen Browser-Viewport von 1588×1000 bei Gerätepixelverhältnis 1. Den oberen Bereich bei `(20,30)-(1568,167)` und den Inhalt bei `(20,167)-(1568,970)` zuschneiden, ohne das Original zu strecken.
- Originale SHA-256-Werte sind in [header-reference.json](../../../design/html/header-reference.json) und [body-reference.json](../../../design/html/body-reference.json) festgehalten. Änderungen am Original erfordern erneute Messung statt stillen Austauschs.
- Die mitgelieferte Offline-Schrift [Xdzip Design Sans SC](../../../design/assets/fonts/README.md) wird nach `document.fonts.ready` gemessen. Echte Schriftstärken 400–750 sind festgelegt, um Breitenänderungen durch plattformabhängige Ersatzschriften zu verringern.
- Die ursprüngliche Schriftfamilie lässt sich aus einem generierten Bild nicht feststellen. Glyphenkonturen und Systemkantenglättung können weiterhin abweichen; Unterschiede gesondert dokumentieren und keine Wiederherstellung der Originalschrift behaupten.

<a name="xdzip-display-size-and-source-pixels"></a>

## Anzeigegröße und Quellpixel

Die normale Entwurfsanzeige verwendet die [Fensterbreite von 1024 logischen Pixeln](README.md) und bildet Quellkoordinaten einheitlich mit `1024 / 1548` ab. Das Hauptfenster erscheint mit 1024 × ungefähr 621,81 CSS-Pixeln; das Original verwendet denselben Maßstab. Dieses Verhältnis ist eine ausdrückliche Gestaltungskonvention und keine aus einem PNG abgeleitete Aussage über Retina oder DPI.

`display-spec.js` ist die einzige Quelle aller Zeichenflächengrößen und Anzeigeskalen. Auf dem Desktop ist die normale Entwurfsgröße voreingestellt, mobil die Einpassung; diese wird auf die Normalgröße begrenzt, um automatische Vergrößerung auf breiten Bildschirmen zu verhindern. Der Originalpixelmodus verwendet `scale=1`, erhält ursprüngliche Pixelmaße und ermöglicht sowohl auf HTML- als auch auf Bildseite Scrollen. Das Gerätepixelverhältnis beeinflusst nur die Rasterdichte, nicht die logische CSS-Breite.

Original und HTML nicht unabhängig voneinander in ihre Container einpassen. Anker von Kopf und Inhalt bleiben in Quellpixelkoordinaten gemessen; Anzeigeskalierung ändert keine kalibrierten Abstände oder Schriftgrößen. Die Größenkonsistenz umfasst getrennt tatsächliche Bildschirmgrenzen, natürliche Originalgröße, mobiles Scrollen, eigenständige Vorschauen und Offline-Ausgaben als Einzeldatei. Nachweise nach der [Prüfmethode](../../../developers/qa/README.md) aufbewahren.

<a name="xdzip-canvas-and-content-scrolling"></a>

## Zeichenfläche und Inhaltsscrollen

In das Prüfwerkzeug eingebettetes HTML ist eine feste Zeichenfläche mit `overflow:hidden` am Wurzeldokument. Dadurch verbrauchen klassische Bildlaufleisten keine Zeichenflächengröße und lösen einander nicht aus. Die Prüfseite scrollt vertikal; die äußere Fläche lässt sich nur bei unzureichender Breite horizontal verschieben. `scrollbar-gutter:stable` reserviert die Breite klassischer Bildlaufleisten, damit eine Höhe am Schwellenwert den Einpassungsmaßstab nicht wiederholt ändert; eigenständige Vorschauen verwenden `auto`. Dateilisten, lange Eigenschaften und Vorgangsfenster behalten Scrollen in Inhaltscontainern, direkt geöffnete nicht eingebettete Quellflächen das Seitenscrollen. Quellfläche, Fenster- und Komponentengrößen ändern sich dadurch nicht.

Playwright verwendet standardmäßig `--hide-scrollbars`; frühere Screenshot- und Größenmatrizen deckten daher echte klassische Bildlaufleisten nicht ab. Größen- und Vorschauprüfungen entfernen dieses Argument jetzt ausdrücklich und prüfen reservierten Platz am Wurzeldokument, Sichtbarkeit aller vier Fensterkanten, äußeres horizontales Verschieben und Erreichbarkeit der letzten Schaltfläche langer Eigenschaften. Eine um 1 px zu kleine Zeichenfläche und ein Negativfall ohne Einbettungsregeln reproduzieren und erkennen konkurrierenden Bildlaufleistenplatz. Screenshots der Reproduktion und Korrektur nach der [Prüfmethode](../../../developers/qa/README.md) in Ausführungsartefakten aufbewahren; dies belegt nicht das Aussehen der Leisten auf jedem Browser oder System.

<a name="xdzip-header-corrections"></a>

## Korrekturen des Kopfbereichs

| Bereich | Vorher | Am Original korrigiert |
| --- | --- | --- |
| Aktiver Tab | Start x299, Breite 244 | x293, Breite 246 |
| Inaktiver Tab | x557, Breite 244 | x542, Breite 262 |
| Tab-Abstand | 14 px | 3 px; etwa 1 px Unsicherheit für weiche Kanten erlaubt |
| Plus- und Schließen-Schaltflächen | Getrennte vertikale Anordnung, Mittelpunkte 5 px auseinander | Gleiche Tab-Zeile, mit etwa 1 px optischem Versatz zum Original positioniert |
| Schließen-Steuerung | Eine Schaltfläche und ein bloßes span | Gleiche Symbol/Name/Schließen-Struktur und ausdrückliche Plätze |
| Einstellungssymbol | Sichtbare Grafik klein und nach links oben verschoben | Mittelpunkt `(1532,61)`; tatsächliche Grafik prüfen, nicht nur den SVG-Container |
| Unterer Rand des aktiven Tabs | Überdeckte die Linie der Titelleiste | Schließt an die weiße Werkzeugleistenfläche an; Übergang der linken Schulter unabhängig festgelegt |
| Suche und Entpacken | Horizontale und vertikale Grenzen verschoben | Jeweils mit `(906,101,449,51)` und `(1368,101,183,51)` vergleichen |
| Linke Werkzeugleistensymbole | Anordnung durch generische Schaltflächengrößen und Abstände bestimmt | An ursprünglichen Grafikankern positionieren und prüfen |

Plätze und Typografie des Kopfbereichs sind in den Abschnitten für Titelleiste, Tabs und Werkzeugleiste von `hifi.css` zentralisiert, ohne generische `.icon-button`-Maße oder geerbte Zeilenhöhe vorauszusetzen. Helles und dunkles Thema teilen die Geometrie. Windows/Linux-Titeldekorationen bleiben Browserdemonstrationen; native Produktfenster erhalten eine getrennte Abnahme.

<a name="xdzip-content-below-the-toolbar"></a>

## Inhalt unter der Werkzeugleiste

Der Inhalt umfasst Archivkarte, Verzeichnisbaum, Pfadleiste, Tabellenköpfe und Zeilen, Auswahl, Dateivorschau und untere Statusleiste. Spaltengrenzen, Zeilenhöhen, Innenabstände, Symbolmittelpunkte und sichtbare Grafik erfassen und mit der [unabhängigen Referenz](../../../design/html/body-reference.json) vergleichen. Die Toleranz für strenge Geometrie und isolierte Symbolgrafik beträgt 2 px; Textkonturen werden nur zur manuellen Prüfung dokumentiert. Die optische Ordnergröße ist eine ausdrückliche Entwurfsrevision: Die neue Kontur ist in gemeinsamen Ressourcen einheitlich um etwa 25,7 % vergrößert. Fünf alte Ordnergrafikbeispiele bleiben zum Vergleich und werden gegen unabhängig erfasste neue Entwurfsgrenzen geprüft; andere ursprüngliche Anker bleiben unverändert.

Die Inhaltsprüfung verwendet gleich große Ausschnitte und behält regionale Vergleiche für Seitenleiste, Liste, Vorschau und Statusleiste. Vorher/Nachher-Screenshots, Commits und Messungen gehören nur in die entsprechenden Issue- oder Actions-Artefakte, nicht als Ergebnisse eines einzelnen Laufs in die Spezifikation. Die mobile Prüfung erfolgt regional und darf das Desktoplayout nicht ändern, um Unterschiede zu verdecken.

Das Original wählt ein Element, lässt aber Alle auswählen leer, und bezeichnet die Ersparnis mit 26 %, während der Balken zu etwa 56 % gefüllt ist; diese statischen Zustände widersprechen sich. HTML behält entsprechend `indeterminate` für die teilweise Auswahl und eine tatsächliche Füllung von 26 % als dokumentierte semantische Ausnahmen. Vorschauprüfungen decken normalen Überlauf und Scrollen langer Inhalte ab; vorhandene Interaktionsprüfungen decken Auswahl, Navigation, Sortierung, Suche und Beispielaktionen ab. Geometrieprüfungen ersetzen keine Funktionsprüfung.

Inhaltsprüfung, gleich große regionale Vergleiche und Messnachweise getrennt nach der [Prüfmethode](../../../developers/qa/README.md) speichern. Erklärungen zu unbekannten Originalschriften, generierten weichen Kanten, Beleuchtung und Kantenglättungsunterschieden erhalten; diese HTML-Nachweise belegen keine entsprechende Kalibrierung des Qt-Produkts.

<a name="xdzip-checks-and-delivery"></a>

## Prüfungen und Auslieferung

Ausführen:

```sh
python scripts/build-html-design.py
python scripts/check-html-header.py
python scripts/check-html-body.py
python scripts/check-html-preview.py
python scripts/check-html-references.py
python scripts/check-html-scale.py
python scripts/check-html-design.py
python scripts/build-html-header-review.py
python scripts/build-html-body-review.py
```

Kopfprüfungen verwenden unabhängige Originalanker: DOM-Grenzen/Mittelpunkte weichen höchstens 1 px ab, tatsächliche Symbolgrafikgrenzen in isolierten Regionen höchstens 2 px. Weiße Dropdown-Pfeilgrafik hat 1 px Toleranz; die rechte Tab-Schulter prüft originale weiße Fläche und Kurve mit 10/255 Toleranz für einen Kanal wegen der Kantenglättung. Glyphenkonturen werden nur gemessen und dokumentiert. Überlagerungen und Differenzbilder enthalten Unterschiede in Kantenglättung, Beleuchtung und Glyphen; die gesamte Pixeldifferenz darf nicht in eine erfundene Ähnlichkeit umgerechnet werden.

Die Offline-Datei für den Kopf ist `dist/xdzip-header-review.html`, die für den Inhalt `dist/xdzip-body-review.html`. Beide schalten zwischen Original, Vorher, Nachher, transparenter Überlagerung und Pixeldifferenzen um; die Inhaltsprüfung bietet zusätzlich mobile Regionsauswahl. Die Einzeldatei-Oberfläche ist `dist/xdzip-hifi-design.html`.

Diese HTML-Prüfungen und Offline-Ansichten bei Bedarf lokal ausführen; sie gehören nicht standardmäßig zur Desktopprodukt-CI. Ankerfehler untersuchen und Überlagerungen auch nach bestandener Geometrie prüfen. HTML-Prüfungen des Hauptfensters ersetzen weder die Kalibrierung anderer Seiten oder des Qt-Produkts noch die native Desktopabnahme.

Die normale Dichte von Hauptfenster und Menü folgt dem [UI-Standard](design-standard.md). Schriftgrößenverhältnisse von Qt-Menüs und Listen müssen gemeinsam geprüft werden. Bei lokaler HTML-Prüfung den tatsächlichen Seitenzoom nach der [Prüfmethode](../../../developers/qa/README.md) prüfen; Übereinstimmung mit alten Quellbildkoordinaten ersetzt dies nicht.
