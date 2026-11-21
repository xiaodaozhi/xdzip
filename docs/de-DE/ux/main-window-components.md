# Hauptfenster: Funktionalität, Komponenten und Interaktionsvertrag

[English](../../ux/main-window-components.md) | [简体中文](../../zh-CN/ux/main-window-components.md) | [繁體中文](../../zh-TW/ux/main-window-components.md) | [日本語](../../ja-JP/ux/main-window-components.md) | [한국어](../../ko-KR/ux/main-window-components.md) | [Français](../../fr-FR/ux/main-window-components.md) | Deutsch | [Español](../../es-ES/ux/main-window-components.md) | [Português (Brasil)](../../pt-BR/ux/main-window-components.md) | [Русский](../../ru-RU/ux/main-window-components.md)

Die Steuerungen des Hauptfensters liefern Aussehen und Verhalten mit ausdrücklichen Zuständen für normal, Hover, gedrückt, Tastaturfokus, deaktiviert, ausgewählt, beschäftigt und Fehler. HTML verwendet Beispielarchive zur Interaktionsprüfung; Qt aktiviert Aktionen entsprechend den tatsächlichen Fähigkeiten für Navigation, Erstellung, Extraktion, [Integritätsprüfung](../engineering/archive-checking.md) und [automatische Inhaltsvorschau](../engineering/content-preview.md). HTML-Beispiele stellen keine echten Hintergrundresultate dar.

Datei-, Verzeichnis- und Archivnamen sowie Pfade verwenden bevorzugt die beim Start eingestellte Systemstandardschrift mit Plattform-Fallback für fehlende Glyphen. Feste UI-Texte verwenden weiterhin die mitgelieferte Teilmenge. Verzeichnisbaum und Ziehvorschauen werden mit ihren jeweiligen Textschriften gemessen; siehe [visueller Standard](../visual/design-standard.md#界面字体与用户数据).

## Funktionen und Komponenten

| Bereich | Gemeinsame / zusammengesetzte Komponenten | Interaktion und Fähigkeit |
| --- | --- | --- |
| Systemtitelleiste | `Browser`, `ArchiveTabs`, Plattformrahmen | macOS nutzt native Ampelknöpfe. Windows nutzt integrierte Knöpfe, native Größenänderung und DWM-Dekoration. Unter Ubuntu X11 folgen integrierte Knöpfe und abgerundete Ecken im Normalzustand dem X11-Design. Siehe [Plattformfenster](platform-windows.md) |
| Archivetabs | ArchiveTabs, ArchiveTab, Symbolknöpfe | Standardmäßig neue Tabs, optional neue Fenster; inaktive Tabs in normaler Schriftstärke, erst schrumpfen, dann horizontal mit Scroll- und Tablisteinträgen scrollen. Ziehen im Fenster und Ctrl+Shift+Page Up/Down ordnen um; Schließen erhält Hintergrundaufgaben |
| Werkzeugleiste | `ArchiveToolbar`, `ArchiveAction`, `CommandMenu` | Öffnen, Neu, Prüfen, Mehr und der geteilte Extraktionsknopf nutzen dieselbe Kommandoausführung wie Menü und Tastatur. Vorprüfung ist für geeignete Auswahlen im Extraktionsmenü verfügbar |
| Suche | SearchField, Löschen-Aktion, Bereichshinweis | Während IME-Komposition nicht absenden; aktuellen Verzeichnisbereich durchsuchen; Löschen ändert das Verzeichnis nicht; Tastenkürzel fokussiert |
| Archivzusammenfassung | ArchiveSummary, Formatplakette, Statistikleiste | Bereit, Lesen, leer und Lesefehler unterscheiden; unbekannte Werte sind nicht null |
| Verzeichnisnavigation | DirectoryTree, DirectoryRow, Disclosure, Verbindungslinien | Mehrstufiges Aufklappen, leere Pfeilplätze für Blätter, unabhängiges aktuelles Verzeichnis/Fokus, barrierefreie vollständige Pfade, Marker zugeklappter Vorfahren, Verzeichniskontextaktionen |
| Dateibereichskopf | PaneHeader, Breadcrumb, ViewToggle | Hierarchische Pfadnavigation, Listen-/Rasterwechsel und Sortiermenü; gemeinsame Höhe und vertikale Mitte |
| Liste und Raster | EntryView, SelectableEntry, Kontrollkästchen, SortHeader | Einzel-, Mehrfach- und Bereichsauswahl, alles auswählen, löschen, umkehren und Pfeiltasten. Doppelklick/Eingabe betritt Verzeichnisse oder extrahiert reguläre Dateien temporär und öffnet sie mit der Systemzuordnung; Eigenschaften bleiben über Kontextmenü oder Details verfügbar |
| Dateikontextmenü | ContextMenu, CommandMenuItem, Submenu | Öffnen gilt wie Doppelklick nur für das rechts angeklickte Element; bestehende Mehrfachauswahl bleibt. Rechtsklick auf ein nicht ausgewähltes Element wählt zunächst nur dieses. Extraktion, Eigenschaften, Name/Pfad kopieren und Verzeichnisnavigation entsprechend der Auswahl aktivieren |
| Kontextmenü leerer Bereich | ContextMenu, RadioMenuItem, CheckboxMenuItem | Auswahl, Ansicht und Sortierung der aktuellen Liste sowie Detailbereich; Auswahl nicht willkürlich löschen |
| Kopfkontextmenü | Sortiermenü | Sortierfeld, Richtung und Archivreihenfolge wiederherstellen; alle Abfrageergebnisse sortieren, nicht nur sichtbare Einträge |
| Eigenschaften / Vorschau | `InspectorPanel` auf der Seite oder separates Detailfenster | Zeigt Datei-, Auswahl- oder Archivzustand. Inhalt wird auf Anforderung gelesen. Unter 928 logischen Pixeln klappt der eingebettete Bereich zu und die Detailaktion öffnet ein separates Fenster |
| Status und Aufgaben | Statusleiste, Aufgabenknopf und `CommandMenu` | Zeigt Auswahlanzahl und bekannte Größen. Das Aufgabenmenü nutzt den anwendungsweiten Verlauf. Aktive Aufgaben haben eine separate Anzahl und bleiben nach Schließen ihres Fortschrittsfensters erhalten |
| Hinweise und Operationsfenster | Tooltip, Notice, Toast, separate Plattformfenster | Vollständige Namen und Deaktivierungsgründe; Escape, Fokusrückgabe, Abbruch und Schließen folgen der Semantik der jeweiligen Aktion |

Liste und Raster teilen einen Leerzustand. Eine abgesendete Suche ohne Treffer schlägt einen anderen Begriff vor. Ohne Suche wird ein Archiv ohne Datensätze als leer bezeichnet; andernfalls beschreibt die Meldung nur die sichtbaren Inhalte des aktuellen Ordners und leitet nicht die Vollständigkeit eines begrenzten Verzeichnisindex ab. Lesen und Fehler behalten tatsächliche Zustände; nicht abgesendeter Suchtext ändert aktuelle Ergebnismeldungen nicht. Hinweise vor dem Öffnen einer Datei zählen keine veraltete Formatliste auf.

## Einheitliche Kommandos und Menüs

Knöpfe, Tastenkürzel, Kontextmenüs und Dropdowns rufen dieselben Kommandos auf und prüfen vor der Ausführung die aktuelle Fähigkeit erneut. Menüs fixieren beim Öffnen Objekt und Auswahlbereich; bei Änderung von Archiv oder Abfrage oder beim Verschwinden des Objekts schließen, damit ein Kommando eines alten Eintrags nicht auf ein neues Archiv angewendet wird.

Menüs teilen vier Spalten für Symbol-/Häkchenplatz, Name, Tastenkürzel und Untermenüpfeil mit Trennlinien zwischen Aktionsgruppen. Nur kontextrelevante Aktionen aufnehmen: Verzeichnisse bieten Betreten und Aufklappen, Dateien Details, und Mehrfachauswahl behält den Stapelbereich. Aktuell nicht ausführbare Aktionen bleiben deaktiviert; Gründe sind nur in barrierefreien Beschreibungen verfügbar. Keine erklärenden Unterzeilen oder Tooltips anzeigen und Deaktivierung nicht durch „demnächst“ beim Anklicken ersetzen.

Bei Änderung der Schriftgröße folgt die Höhe der Menübezeichnung den tatsächlichen Schriftmetriken und bleibt in ihrer Zeile vertikal zentriert. Bereits die erste Darstellung muss eine Zentrierabweichung von höchstens einem halben logischen Pixel einhalten, ohne darauf zu vertrauen, dass ein späterer Layoutdurchlauf einen alten Textplatz korrigiert.

Normale, Kontrollkästchen-, Radio- und Untermenüzeilen sowie Trennlinien sind gemeinsame Komponenten. Sortierung und Ansicht verwenden Radioelemente für den aktuellen Zustand; der Detailbereich ein Kontrollkästchenelement. Gefährliche Aktionen werden standardmäßig nie automatisch ausgelöst; Rechtsklick und Öffnen eines Menüs führen keine Aktion aus.

## Menüeingabe und Lebenszyklus

- Ab/auf wechseln zwischen verfügbaren Einträgen; Pos1/Ende gehen zum ersten/letzten; Eingabe/Leertaste aktiviert, rechts betritt ein Untermenü und links kehrt zum Elternmenü zurück.
- Escape schließt jeweils eine Ebene; das Schließen der Wurzel gibt den Fokus an die auslösende Steuerung oder den ursprünglichen Eintrag zurück. Tab schließt das Menü und setzt die normale Fokusreihenfolge der Seite fort. Suche nach dem ersten Namenszeichen wird unterstützt; IME-Komposition löst keine Menütippsuche aus.
- Hover hebt hervor, ohne auszuführen. Untermenüs öffnen verzögert und behalten beim Betreten das Elternelement. Maus und Tastatur können aufeinander folgen, ohne mehrere verbleibende Hervorhebungen oder Wurzelmenüs zu hinterlassen.
- Außenklicks schließen, ohne dem äußeren Ziel den Fokus zu entziehen. Scrollen des Hauptbereichs, Fenstergrößenänderung, Betreten von Operationsfenstern oder Archivwechsel schließen Menüs; Scrollen des Menüs selbst nicht. Gemeinsame HTML-Steuerungen erfassen beim Öffnen tatsächliche Scrollpositionen vorhandener Container; vor dem Öffnen abgeschlossene, aber spät gelieferte Ereignisse schließen nicht, spätere Positionsänderungen dagegen schon.
- Untere und rechte Ränder des Ansichtsbereichs meiden und bei Bedarf links oder nach oben öffnen. In Hostkoordinaten positionieren und dabei logische Maße, ursprüngliche Quellpixel, DPR und Prüfmaßstab unterscheiden.
- Umschalt+F10/Menütaste öffnet das Kontextmenü des aktuellen Eintrags. Langes Berühren und Rechtsklick teilen den Kontext und zerstören die Mehrfachauswahl nicht wie ein einfacher Klick.

## Steuerungszustände und Barrierefreiheit

| Zustand | Aussehen | Verhalten |
| --- | --- | --- |
| Normal | Genehmigte ursprüngliche Umrandung, Radius, Symbol und Grundlinie erhalten | Zustandsänderungen ändern keine Steuerungsmaße |
| Hover | Helle Füllung; Hauptknopf mit kräftigerer Hauptfarbe | Verzögerte Hinweise verdecken keine Ziele; Berührung hinterlässt keinen festhängenden Hover |
| Gedrückt | Deutlichere Füllung als Hover | Keine Text-/Symbolbewegung; abgebrochener Druck führt nicht aus |
| Fokus | Kein zusätzlicher Umriss, dicker Rand oder Schein | Tab-Reihenfolge und Tastaturaktivierung erhalten; Außenklick, Escape und Eingabe verlassen den Steuerungsfokus |
| Deaktiviert | Ausdrückliche deaktivierte Text-/Grafikfarben bei erhaltenem Umriss | Keine Reaktion auf Klick oder Tastenkürzel; barrierefreie Beschreibung erklärt den Grund |
| Ausgewählt | Hintergrund oder Häkchen zeigt Zustand eindeutig | `aria-selected` / `aria-checked` / Qt Accessible synchronisieren |
| Beschäftigt | Zustand und deaktivierter Bereich folgen der Aufgabe | Doppelte Absendung verhindern; Navigation während Hintergrundarbeit erlauben |
| Fehler | Benachbarter Text erklärt Problem und Wiederherstellung | Farbe ist nicht die einzige Information und Erfolg wird nicht vorgetäuscht |

Gemeinsame Symbole stammen aus `design/assets`. Steuerungen implementieren Text, Häkchen, Hintergründe, Verbindungslinien und Menüpositionierung; Ausschnitte in Originalpixeln dienen dem Vergleich. HTML behält Quellkoordinaten des Hauptdesigns und rechnet neue Steuerungsparameter zuerst aus logischen Pixeln um; Qt verwendet dieselbe logische Spezifikation.

## Abnahme

Anzahl und Aktionsknöpfe rechts im Dateibereich bilden eine Gruppe fester Breite. Listen-, Raster-, Sortier- und sichtbare Detaileinträge bleiben bei 28×28 logischen Pixeln mit Abstand 6.4; ein breiteres Fenster gibt nur der Brotkrümelnavigation mehr Platz. Das Mehr-Menü der Werkzeugleiste enthält nur Einstellungen; ursprüngliche Reihenfolge, Metadatenvorprüfung der Extraktion, Alles auswählen und Details verwenden vorhandene eigene Einstiegspunkte.

Start- und Archivseiten teilen eine mindestens 32 logische Pixel hohe Statusleiste gemäß statusbarHeight in layout-scale.json. Barrierefreie Schriftvergrößerung nutzt die tatsächliche Höhe des Aufgabenknopfs und gleiche vertikale Abstände, ohne Höhenänderung beim Seitenwechsel. Beide Aufgabenknöpfe nutzen task-list in einem zentrierten logischen 16×16-Symbolplatz aus main.interaction.taskIconSize, unverändert durch Seite oder Schriftmaßstab.

Die Archivstatusleiste zeigt die Auswahlanzahl; bei nicht leerer Auswahl nur dann Trennzeichen und Größe ergänzen, wenn die Gesamtsumme bekannt ist. Macht ein virtuelles Verzeichnis oder eine Datei unbekannter Größe sie unbekannt, nur die Anzahl ohne zusätzliches Trennzeichen oder `—` anzeigen. Eine bekannte Größe von null Bytes wird weiterhin angezeigt; der Startseitentext bleibt unverändert.

Die rechte Kante des Kontrollkästchens der Dateiliste bleibt 12 logische Pixel vom Dateisymbolplatz entfernt; Symbol, Dateiname und Name-Kopf sind ausgerichtet. Barrierefreie Schriftgrößen ändern tatsächliche Textmetriken, vergrößern aber diesen Abstand nicht. Zeilenkästchen und Alles auswählen im Kopf behalten das bestehende Auswahlmodell.

Alle fünf Spalten lassen sich durch Ziehen der rechten Grenze mit einem Trefferbereich von 8 logischen Pixeln ändern; Doppelklick stellt den Standard dieser Spalte wieder her. Name füllt standardmäßig den verbleibenden Platz; andere Spalten beginnen mit Werten des Originaldesigns. Beide Größenspalten wachsen standardmäßig, damit gemessene Beschriftungen und Sortierpfeile ohne Abschneiden passen. Benutzerbreiten werden während der Laufzeit über Tabs und Fenster geteilt und beim Beenden zurückgesetzt; logische Breiten bleiben im Speicher, nicht in Einstellungen oder DPR. Das Minimum für Name ist 160 und für andere Spalten 64, proportional steigend bei barrierefreien Schriften; Überlauf scrollt horizontal. Kopf und Inhalt teilen eine Breitenquelle, und Ziehen löst keine Sortierung aus. Auf-/Abwärtsmarker stehen vor rechtsbündigen Beschriftungen für Originalgröße und komprimierte Größe und nach anderen Beschriftungen, immer 6 logische Pixel vom angezeigten Text entfernt. Aktive Sortierspalten reservieren Pfeilplatz auf der passenden Seite; bei manuellem Verengen werden zuerst Beschriftungen gekürzt, und Pfeile dürfen weder Text noch Nachbarspalten überlappen. Numerische Köpfe bleiben wie ihre Inhalte rechtsbündig.

Dateitypsymbole werden für Qt- und HTML-Auswahlen aus der [Erweiterungs-Hinweistabelle](../../../specs/visual/file-types.json) erzeugt und umfassen Bürodokumente, Tabellen, Präsentationen, Audio, Video, Archive, Installer, Schriften und Datenträgerabbilder. Erweiterungen sind unabhängig von Groß-/Kleinschreibung, unbekannte Typen erhalten ein allgemeines Dokumentsymbol. Dies beschreibt nur das Aussehen, nicht vertrauenswürdigen Inhalt oder Unterstützung des Archivformats für Navigation, Erstellung oder Extraktion.

Mit tatsächlicher Maus und Tastatur Einträge, Vermeidung von Menürändern, Untermenü-Hin- und Rückwege, Auswahlbeibehaltung, Fokusrückgabe, Schutz deaktivierter Aktionen, Tab-Isolation, Ansichtswechsel und Verzeichnishierarchie prüfen. Tatsächliche Steuerungen in hellen/dunklen Themen, drei Plattformdekorationen, normalen/großen Schriften und Skalierungen aufnehmen. Die bestehende anfängliche Ausrichtung von Hauptfenstertitel, Werkzeugleiste, Dateiliste und Vorschau darf durch Menüarbeit nicht zurückfallen.

Funktionsabdeckung, tatsächliche Screenshots und nicht integrierte Fähigkeiten nach der [allgemeinen Verifizierungsmethode](../../../developers/qa/README.md) festhalten, mit Ergebnissen in zugehörigen Issues und Nachweisen in build/ oder CI-Artefakten. Statische Boards, Browserverhalten und Qt-Laufzeitprüfung werden separat beschrieben und ersetzen einander nicht.

Qt-Menüs verwenden ausdrücklich `Popup.Item` mit gemeinsamen Delegates, damit AppKit benutzerdefiniertes Styling nicht ignoriert; Operationsfenster bleiben separate Fenster oberster Ebene. `CommandTooltip` verwendet gemeinsame Hinweisparameter und Klartext, damit Archivnamen nicht als Rich Text interpretiert werden; Hinweise ausblenden, wenn Menüs Fokus erhalten. Gemeinsame achtstellige JSON-Farben verwenden CSS RGBA; der QML-Generator wandelt in QColor ARGB um. Farbe und Deckkraft gemeinsam prüfen.

Verzeichnisbäume und Hover-Zustände von Öffnen, Neu, Prüfen, Mehr und Extrahieren der Hauptwerkzeugleiste haben keine schwebenden Tooltips. Deaktivierte Menüaktionen haben ebenfalls keinen sichtbaren Hinweis oder Tooltip; andere Knöpfe verwenden gemeinsame verzögerte Hinweise. Wurzel und gewöhnliche Unterverzeichnisse listen nur direkte Dateien und Ordner, während die Suche den aktuellen Bereich rekursiv umfasst. Gewöhnliche Namen zeigen die letzte Komponente, die Suche vollständige Pfade. Listen-Hover umfasst die ganze Zeile; Raster behalten feste horizontale Abstände und Kontrollkästchen bei Hover/Auswahl. Menüs nutzen weiche Schatten auf allen Seiten und einen Abstand von 4 logischen Pixeln zu auslösenden Knöpfen. Separate Interaktions- und Skalierungsnachweise gemäß [allgemeiner Methode](../../../developers/qa/README.md) aufbewahren.

Alle Sortiermodi halten Verzeichnisse vor Dateien. Auf- und absteigende Reihenfolge gelten innerhalb jeder Gruppe. Dateitypsortierung gruppiert Dateien ohne Erweiterung separat von Verzeichnissen und erhält stabile Identitätsreihenfolge bei gleichen Typen. Die vollständige Abfrage sortieren, bevor interne Übertragungsbatches geteilt werden. HTML-Dateien ohne gesetztes folder werden als false behandelt; undefined für den Vergleich nicht in NaN umwandeln.

## Auswahl, Aufgaben und Hinweise

Alles auswählen behält den Detailfokus auf dem letzten Element der aktuellen Liste, mit denselben Metadaten wie bei Einzelauswahl. Löschen der Auswahl blendet vier Eigenschaften und „Diese Datei extrahieren“ aus; dieser Knopf nutzt gemeinsame Hover-/Gedrückt-/Fokuszustände. Alles auswählen gilt weiterhin für die gesamte aktuelle Abfrage, ohne Verzeichnis- oder physische Identitätsverträge zu ändern.

„Aufgaben N“ unten zeigt die Anzahl des erhaltenen Verlaufs; die aktive Anzahl dient nur Hinweisen und Beendigungslogik. Jeder Aufgabenmenüeintrag zeigt links Erstellung/Extraktion und Nummer, rechts die Statusfarbe; Aktivierung mit Maus oder Tastatur öffnet dieselbe Aufgabe erneut. Aufgabenmenüs teilen Dichte, Schriftgröße und Scrollgrenzen mit anderen Menüs. Ohne Auswahl sind Aktionen für den ausgewählten Bereich und die Vorprüfung im Extraktionsbereichsmenü ohne zusätzliche Hinweiszeilen deaktiviert.

Schwebende Hinweise erscheinen erst, nachdem der Zeiger 1200 ms ununterbrochen im selben Steuerelement stillsteht. Bewegung, Verlassen, Drücken oder Menüöffnung brechen Anzeige und alte Timer ab; beim Wechsel zwischen Steuerungen ist eine vollständig neue Wartezeit nötig, kein schnelles Aufwachen aus dem vorherigen Hinweis. Oberhalb der Steuerung bevorzugen; bei zu wenig Platz oben darunter verwenden, damit Randbegrenzungen den Tooltip nicht auf die Steuerung schieben und Hover-Oszillation auslösen. Das gemeinsame Qt-CommandTooltip nutzt einen ausdrücklichen Stillstandstimer, synchronisiert mit HTML data-tooltip. Vollständige Listennamen verwenden kein natives Browser-title zur Umgehung dieser Regel. Das Timing nativer Systemmenü- und Trayhinweise wird vom System gesteuert und nicht als durch QML steuerbar behauptet. Das Timing skaliert nicht mit Schriftgröße, DPR oder Dichte; die visuelle Geometrie bleibt unverändert.

Archivetabs zeigen vollständige Dateinamen ebenfalls als Klartext im gemeinsamen Tooltip mit Stillstandsverzögerung; Drücken, Umordnen oder Öffnen des Tabmenüs oder der Tabliste blendet ihn aus. Globale Aufgabeneinträge schließen zuerst ihr Menü und öffnen dann nur das Aufgabenfenster, ohne Archivetabs zu wechseln oder geschlossene Archivseiten erneut zu öffnen.

Die Startseite nutzt einen scrollbaren Inhalt, der oben fixiert und horizontal zentriert ist. Fehler und Warnungen erscheinen unter Formathinweisen in rotem Text und mit semantischen Symbolen, ohne oberen Inhalt zu verschieben. Schließen eines im Tray verbleibenden Hauptfensters löscht seine Tabs; gewöhnliches Wiederherstellen zeigt nur die Startseite. Minimieren behält Tabs; Hintergrundaufgaben bleiben global erreichbar.

Siehe [Datei-Drag-and-drop und Änderung](../engineering/archive-transfers.md) für bidirektionale Ablagen in der Dateiliste, Bestätigung des Hinzufügens zum aktuellen Verzeichnis und externes Kopieren auf drei Plattformen.

Archivinterne Verschiebungen verwenden internes Ziehen. Ordnerziele in Liste/Raster und Verzeichnisbaum nutzen die bestehende Auswahlfüllung; unmittelbares Feedback verwendet Tooltip-Geometrie wieder, höchstens vier Zeilen und begrenzt auf den aktuellen Clientbereich. Ausgewählte Verzeichnisse, ihre Nachfahren, ursprüngliche Positionen, Suche und verschlüsselte/geteilte Ziele erhalten ausdrückliche Ablehnungsgründe ohne neues Bestätigungsfenster. Namenskonflikte verwenden das Dateikonfliktfenster und Verschiebeaufgaben einen eigenen Namen. Siehe Design archivinterner Verschiebung und [Drag-and-drop-Vertrag](../engineering/archive-transfers.md) für Transaktionen und Grenzen.
