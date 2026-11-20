<a name="xdzip-high-fidelity-states-and-implementation-contract"></a>

# High-Fidelity-Zustände und Implementierungsvertrag

[English](../../ux/hifi-state-spec.md) | [简体中文](../../zh-CN/ux/hifi-state-spec.md) | [繁體中文](../../zh-TW/ux/hifi-state-spec.md) | [日本語](../../ja-JP/ux/hifi-state-spec.md) | [한국어](../../ko-KR/ux/hifi-state-spec.md) | [Français](../../fr-FR/ux/hifi-state-spec.md) | Deutsch | [Español](../../es-ES/ux/hifi-state-spec.md) | [Português (Brasil)](../../pt-BR/ux/hifi-state-spec.md) | [Русский](../../ru-RU/ux/hifi-state-spec.md)

Dieser Leitfaden beschreibt Verhalten, Fokus und Schließaktionen für Erstellungs-, Extraktions- und Aufgabenfenster und verweist auf die [Originalentwürfe der Operationszustände](../../../design/hifi/README.md) und die [gemeinsame Ressourcenbibliothek](../../../design/assets/README.md). Die Kennungen in der Tabelle bezeichnen Zeichenflächen des Designs und ändern das Aufgabenprotokoll nicht.

<a name="xdzip-screens-and-task-semantics"></a>

## Ansichten und Aufgabensemantik

| Zustand / Zeichenfläche | Eintrittsbedingung | Standardfokus und Aktion | Schließen und Wiederaufnahme |
| --- | --- | --- | --- |
| C01 / 01 | Erstellung, keine laufende Aufgabe | Erste Eingabeschaltfläche oder Dateiname; vor der Erstellung Eingaben, Namen, Ziel und Fähigkeiten prüfen | Entwurf dieses Fensters verwerfen; kein Schreibvorgang |
| E02 / 02 | Ein Aufgaben-Snapshot enthält eine aktive Konfliktabfrage | Überspringen, Umbenennen und Überschreiben sind ausdrückliche Entscheidungen. Umbenennen ist die Hauptaktion. Auf folgende Konflikte anwenden ist anfangs nicht markiert | Schließen blendet die Abfrage aus und lässt die Aufgabe warten. Über die Aufgabe erneut öffnen oder mit deren Abbrechen-Aktion stoppen |
| T01 / 03, 08 | Plan angenommen und Ausführungsereignis empfangen | Zurück zur Ansicht und Abbrechen sind gewöhnliche Schaltflächen; keine erfundene Pause | Nur Ansicht ausblenden; Aufgabeneintrag kann erneut öffnen; Abschluss im Hintergrund stiehlt keinen Fokus |
| T01 / Integritätsprüfung | Gleiche Komponente wie der Aufgabenfortschritt, schreibgeschütztes Archiv und Prüfphasen | Zurück zur Ansicht, Prüfung abbrechen und tatsächliches Aufgabenergebnis; HTML-Demo und Qt-Komponentenvorschau bleiben getrennt gekennzeichnet | Schließen bricht nicht ab; echte Prüfungen gelangen über die Symbolleiste in die Hintergrundwarteschlange |
| R01 / 04 | Aufgabe hat veröffentlichte und unvollendete Einträge | Erneut versuchen wählt nur unvollendete Einträge und kehrt zur Planbestätigung zurück | Schließen der Ansicht löscht keine übernommenen Dateien |
| S01 / 05 | Einstellungen öffnen und einen von sechs Reitern wählen | Alle Änderungen per Tastatur zugänglich; nach Erfolg sofort wirksam, Adressen müssen gültig sein | Schließen behält angewendete Einstellungen; bei Fehler bleiben vorherige Werte erhalten |
| R02 / 06 oben | Einzigartiger erfolgreicher Endzustand empfangen | Extraktion kann den Zielordner nach ausdrücklicher Auswahl öffnen; Erstellung öffnet unterstützte Ergebnisse nur in der Anwendung, nie automatisch einen Systemordner. Null Fehler verwendet einen neutralen Zustand | Details schließen, ohne die Abschlussmeldung zu wiederholen |
| T02 / 06 Mitte | Abbruch angenommen, Bereinigung unvollständig | Wiederholter Abbruch deaktiviert; Rückkehr zur Ansicht bleibt möglich | Ausblenden möglich; „Abgebrochen“ darf nicht vorzeitig erscheinen |
| R03 / 06 unten | Endzustand des Abbruchs empfangen | Veröffentlichte Einträge behalten; falls Wiederholung erlaubt ist, unvollendeten Umfang neu planen | Schließen macht veröffentlichte Einträge nicht rückgängig |
| C02 / 07 | Der Entwurf zeigt ein vorhandenes Ausgabearchiv | Ein realer Erstellungskonflikt verwendet die Regeln für sichere Namen und Veröffentlichung der laufenden Aufgabe | Abfrage schließen lässt die Aufgabe warten. Ausdrückliches Umbenennen, Ersetzen oder Abbrechen folgt dem Aufgabenergebnis |

„Neuen Namen verwenden“ schlägt nur einen Namen vor; beim tatsächlichen Start wird weiterhin geprüft, ob das Ziel existiert. Die Dateinamenprüfung ist keine endgültige Schreibberechtigung. Eine ersetzende Erstellung muss ein temporäres Archiv vollständig erzeugen und prüfen, bevor die Zielbedingungen für die Veröffentlichung erneut geprüft werden. Ein nach dem linearisierten Commit-Punkt eintreffender Abbruch zeigt gemäß Kernergebnis den Abschluss, statt Erfolg als Abbruch umzudeuten.

<a name="xdzip-creation-capabilities-and-field-errors"></a>

## Erstellungsfähigkeiten und Feldfehler

- Die vier ZIP-Stufen entsprechen Store/Deflate 1/6/9, Standard ist 2 normal. TAR und TAR.GZ können erstellt werden; ein Formatwechsel synchronisiert Endungen und löscht Passwort und Teilarchiveinstellung. TAR deaktiviert die Stufe; TAR.GZ-Stufen 0/1/2/3 entsprechen GZIP 0/1/6/9. Ein Formatwechsel synchronisiert die Endung, darf Ziele aber nicht stillschweigend überschreiben.
- Ohne Eingabe, bei leerem Namen, ungültigem Ziel oder fehlender Berechtigung ist Erstellen deaktiviert und der Grund erscheint am Feld. Fehlertext wird von Bildschirmlesern erfasst, statt nur die Rahmenfarbe zu ändern.
- Der Kern behandelt doppelte Eingaben und Verzeichniserweiterung; die Oberfläche zeigt nur Auswahlzusammenfassungen. Bildsymbole bei der Erstellung zeigen den Typ, keine Bildvorschau.
- ZIP und 7z zeigen ihre unterstützten Passwort- und Teilarchiveinstellungen. 7z kann auch Dateinamen verschlüsseln. Das Formatregister nennt die tatsächlichen Schreibfähigkeiten. [ZIP-Erstellung](../engineering/zip-creation.md), [Formatregister](../engineering/format-registry.md)

<a name="xdzip-task-data-and-exceptions"></a>

## Aufgabendaten und Ausnahmen

Beispiel A enthält drei Dateien: 8,4 + 6,2 + 12,8 = 27,4 MB. Bei zwei Erfolgen sind 14,6 MB gespeichert und das verbleibende PDF ist 12,8 MB groß; Zahlen für erfolgreiche, übersprungene und unvollendete Einträge dürfen nicht verwechselt werden. Beispiel B ist ein ganzes Archiv mit 30 Dateien und 77,5 MB; 46,5 MB entsprechen 60% und nicht demselben Zeitpunkt wie Beispiel A.

Prozentwerte folgen aus bekannten Gesamtbytes und tatsächlicher Messung. Bei unbekannter Gesamtmenge wird unbestimmter Fortschritt ohne erfundene Restzeit angezeigt. Nach einem Phasenwechsel die Geschwindigkeit neu schätzen; die dargestellten 6,2 MB/s und ungefähr 5 Sekunden sind nur ein Beispiel für eine stabile Phase. Dateizahlen unterscheiden abgeschlossen und in Bearbeitung; die laufende Eintragsnummer ist keine Erfolgszahl.

Datenträger voll: stoppen und veröffentlichte Einträge behalten; nach Freigabe von Speicher oder Zieländerung neu planen. Quelle geändert: Erneut versuchen öffnet das Archiv erneut; alte Indizes dürfen nicht wiederverwendet werden. Nur Abbruch: Unvollendetes verwendet Bernstein oder neutral, ohne Beschädigung zu melden. Fehlercodes erscheinen in Details; normale Ansichten zeigen weder Aufrufstapel noch interne Brückenbegriffe.

<a name="xdzip-immediately-applied-settings"></a>

## Sofort angewendete Einstellungen

Design, Schriftgröße, Dichte, Sprache, Öffnungsmodus und Datenschutzschalter werden nach erfolgreicher Änderung sofort gespeichert und angewendet. Das Leeren des Verlaufs geschieht sofort, ohne Originaldateien zu löschen. Bei Fehler bleiben alte Einstellungen erhalten und eine Meldung erscheint. Über und Aktualisierungen stehen auf der Über-Seite der Einstellungen; Dateizuordnungen erfordern eine Bestätigung des Betriebssystems und spiegeln tatsächliche Ergebnisse wider. Die Netzwerkseite speichert geprüfte Proxy- und Spiegeladressen. Siehe den [Einstellungsvertrag](settings-localization.md).

Die standardmäßigen Ausgabebudgets des vorhandenen Hosts betragen 8 GiB pro Datei und 20 GiB pro Aufgabe; das aktuelle Einstellungsfenster erlaubt keine Quotenänderung. Werden später Grenzen hinzugefügt, positive Ganzzahlen und Überlauf prüfen, die Dateigrenze höchstens so groß wie die Aufgabengrenze setzen und nur auf neue Pläne anwenden. Unveränderliche Berechtigung aktiver Aufgaben nicht ändern und Speicherplatzprüfungen nicht umgehen.

<a name="xdzip-window-layout-and-asset-mapping"></a>

## Zuordnung von Fenstern, Layout und Ressourcen

Alle Operationsfenster folgen der [plattformbezogenen Fensterspezifikation](platform-windows.md). Native macOS-Titelleisten verwenden keine ausgeschnittenen Ampelknöpfe. Die Plattformschicht prüft Position und deaktivierte Zustände der Titelschaltflächen unter Windows/Linux, Systemmenüs und Verschieben. Rechtsseitige Linux-Bedienelemente in Plattformvergleichsbildern sind kein fest codiertes Versprechen für jede Desktopumgebung.

Aktuelle Maße folgen [layout-scale.json](../../../specs/visual/layout-scale.json), dem [Fensterregister](../../../specs/visual/window-registry.json) und dem [Designstandard](../visual/design-standard.md) und ersetzen frühe übergroße Vorschläge. Erstellung, Extraktion, Konflikt, Fortschritt und Ergebnisse teilen Breite 491; Einstellungen verwendet 612 als registrierten zweckbezogenen Unterschied. Titel 20, Text 13, Fußschaltflächen 13 und Zusatztext 12. Die Höhe folgt Inhalt und Grenze des jeweiligen Fensters; der Inhalt ist scrollbar. Operationsfenster mit Fußbereich halten ihn fest und erreichbar; Einstellungen hat keinen Fußbereich. Einstellungen bietet Schriftgrößen 100%/125%/150%; Komponentenmaße nicht aus globaler Skalierung oder physischen Screenshot-Pixeln ableiten.

| Verwendung | Hinzugefügte Ressourcen | Gemeinsame / dynamische Teile |
| --- | --- | --- |
| Erstellungstitel, Eingaben hinzufügen | archive-add, image-file, remove | folder, file-pdf, new |
| Konflikt, teilweiser Fehler | document-warning, status-warning, info | file-pdf; Vergleichskarten und Text durch QML gezeichnet |
| Laufender Fortschritt | document-download, image-file, info | Fortschrittsbalken und Zahlen dynamisch gezeichnet |
| Erfolg, Überspringen, Abbrechen, Stoppen | status-success, status-skipped, status-stopped, spinner, clock | Qt-Steuerelement dreht spinner |
| Einstellungen | settings-tile, theme-system | Designvorschauen, Schalter und Zahleneingaben sind Komponenten; settings ist ein kleines Symbol |

Ressourcen liegen unter `design/assets/`. `reference/` enthält Ausschnitte in Originalpixeln zum Vergleich des Aussehens; `svg/` und `png/` sind transparente Laufzeitressourcen und dürfen nicht Originalebenen genannt werden. Alle SVGs verwenden 24×24-Zeichenflächen; PNGs werden mit 24×24, 48×48 und 72×72 exportiert und behalten dieselbe logische Anzeigegröße. HTML-Dateilisten verwenden einen Platz von 48×48 im Quellraum; Qt verwendet ungefähr 31,75×39,69 logische Pixel. PDF-, Text-, Code- und Markdown-Symbole teilen einen Mittelpunkt und 22 Pixel sichtbare Höhe bei unterschiedlichen Konturbreiten; Ordner sind auf derselben Zeichenfläche zentriert. Dunkle SVGs sind Ressourcenanpassungen; vollständiges dunkles Verhalten erfordert weiterhin echte Qt- und Kontrastprüfungen.

<a name="xdzip-directory-tree-addition"></a>

## Ergänzung des Verzeichnisbaums

Die B01-Navigation ergänzt die [Verzeichnisbaum-Spezifikation B](directory-tree.md) mit drei formalen Zeichenflächen und 26 gemeinsamen Ausschnitten. Knotenpfeile hängen von Unterverzeichnissen ab; Aufklappen, aktuelles Verzeichnis und Tastaturfokus sind unabhängig. Laden, leere Archive, Dateien im Wurzelverzeichnis, Navigationsgrenzen, Fehler und verborgene aktuelle Pfade sind getrennt spezifiziert; der bestehende schreibgeschützte Domänenvertrag bleibt unverändert. HTML und Qt integrieren Kernnavigation, Verbindungslinien und Anzeigen für verborgene aktuelle Zweige; Skelette und spezielle Grenzsymbole bleiben Designressourcen. Den Integrationsumfang beschreibt die Verzeichnisbaum-Spezifikation.

<a name="xdzip-changes-and-revalidation"></a>

## Änderungen und erneute Validierung

Wichtige Zustände wurden in HTML und Qt verwendet. Bei späteren Änderungen weiterhin Originalentwürfe und überarbeitete Parameter, Zuordnung von Steuerelementen zu Funktionen, gemeinsame Komponenten und Fensterverträge prüfen und betroffene Plattformen verifizieren. Entwürfe oder statische Prüfungen allein belegen weder native Funktion noch Barrierefreiheit. Tatsächliche Ergebnisse im zugehörigen Issue dokumentieren; frühere Abnahmen werden durch diese Liste nicht zurückgesetzt.
