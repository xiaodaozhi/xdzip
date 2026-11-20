<a name="xdzip-extraction-preflight-metadata-phase"></a>

# Extraktionsvorprüfung: Metadatenphase

[English](../../engineering/extraction-preflight.md) | [简体中文](../../zh-CN/engineering/extraction-preflight.md) | [繁體中文](../../zh-TW/engineering/extraction-preflight.md) | [日本語](../../ja-JP/engineering/extraction-preflight.md) | [한국어](../../ko-KR/engineering/extraction-preflight.md) | [Français](../../fr-FR/engineering/extraction-preflight.md) | Deutsch | [Español](../../es-ES/engineering/extraction-preflight.md) | [Português (Brasil)](../../pt-BR/engineering/extraction-preflight.md) | [Русский](../../ru-RU/engineering/extraction-preflight.md)

Dieses Modul stellt Regeln für die Metadatenvorprüfung bereit. Es kann kein Ziel autorisieren, dekodieren, schreiben, zwischenspeichern oder festschreiben.

<a name="xdzip-implementation-and-boundaries"></a>

## Implementierung und Grenzen

`xdzip-extract-plan` hängt nur von Domänen- und Metadatenschnittstellen ab, ohne Qt, Formatparser oder zusätzliche Drittanbieterabhängigkeiten. Eingabe sind ein Session-Snapshot, eine Menge physischer EntryIds und Abbruch; Ausgabe ist nur ein begrenzter Bericht, keine zum Schreiben nutzbaren Pfade oder ein Sicherheitsfreigabe-Flag. `xdzip-runtime` hält den Snapshot; Qt ruft es über die Worker-Thread-Brücke auf.

Die Auswahl muss nicht leer sein, ist auf 128 physische IDs begrenzt und alle IDs müssen zum aktuellen Snapshot gehören. Ein ausgewähltes Verzeichnis erweitert alle Nachfahren anhand der Schrägstrichgrenzen der Originalnamen, unabhängig von Suche, Seiteneinteilung oder aktuellem Verzeichnisbereich. Ein mit seinem Verzeichnis ausgewähltes Kind zählt einmal. Gleichnamige physische Datensätze werden nicht nach Pfad zusammengeführt. Verzeichnisstatistiken zählen nur explizite Datensätze und erzeugen keine Verzeichnisobjekte.

Die Vorprüfung läuft über den Befehls- und Ereignisdienst mit begrenzter Zulassung und unabhängigem Abbruch. Die Laufzeit prüft die Identität der geöffneten Quelle vor und nach der Untersuchung. Bei plattenindiziertem ZIP lädt sie gewählte physische Datensätze und benötigte Nachfahren aus dem Index. Änderungen von Auswahl, Abfrage oder Session löschen den Bericht. Ergebnisse sind Schätzungen aus erfassten Metadaten. Die Vorprüfung dekodiert niemals Mitgliedsinhalte, reserviert keine Zielpfade und autorisiert keine Schreibvorgänge.

<a name="xdzip-rules-performed"></a>

## Ausgeführte Regeln

- Namen müssen als UTF-8 auflösbar sein, höchstens 4 KiB, 64 Komponenten und 255 Bytes je Komponente; Originalnamen werden weder repariert noch umgeschrieben.
- Absolute Pfade, leere Komponenten, `.`, `..`, Backslashes, Doppelpunkte, Steuer- und bidirektionale Steuerzeichen, ungültige Windows-Zeichen sowie abschließende Leerzeichen oder Punkte in Komponenten ablehnen.
- Übliche Windows-Gerätenamen einschließlich Formen mit Erweiterung und hochgestellten COM/LPT-Ziffern ablehnen; dabei vorsichtig eine plattformübergreifende portable Namensmenge anwenden.
- Vorhandene Metadatenwarnungen melden und Nutzdaten- und Passwortfähigkeitsflags des Datensatzes verwenden. Ein verschlüsselter Datensatz kann einen unterstützten Passwortbedarf oder nicht unterstützte Verschlüsselung melden. Ein Datensatz mit bekannter Größe und nicht unterstützten Nutzdaten meldet zusätzlich einen Methodenbefund. Die Vorprüfung nutzt vom Leser projizierte Metadaten und Fähigkeiten; sie entschlüsselt, authentifiziert oder dekodiert keine Nutzdaten.
- Exakte Namensduplikate, Kollisionen durch Unicode-Kleinschreibvergleich, sich in Großschreibung unterscheidende Elternverzeichnis-Aliasse und Datei-/Verzeichniskonflikte in beiden Einfügereihenfolgen prüfen. Ein implizites Elternverzeichnis darf durch ein explizites gleichen Namens ergänzt werden; doppelte explizite Verzeichnisdatensätze bleiben Konflikte.
- Deklarierte Dateigrößen über 8 GiB, Gesamtsummen über 20 GiB und Ganzzahlüberlauf melden. Bei ZIP-Datensätzen fordert ein deklariertes Kompressionsverhältnis über 1000:1 eine Prüfung. Verzeichnisgrößen zählen nicht zu Ausgabebytes.

Unicode-Kleinschreibvergleich ist weder vollständiges Casefolding noch NFC/NFD-Dateisystemäquivalenz. Jede Nicht-ASCII-Komponente ergänzt einen Hinweis, dass Äquivalenz und Kollisionen auf dem Zieldateisystem noch zu prüfen sind. Auch ein Bericht ohne Befunde autorisiert keine Ausgabe.

<a name="xdzip-resources-and-failures"></a>

## Ressourcen und Fehler

Höchstens 100.000 Snapshot-Datensätze prüfen; ausgewählte oder erweiterte Namen umfassen zusammen höchstens 4 MiB. Im Kollisionsindex gespeicherte Vergleichsschlüssel und Originalschreibweisen umfassen höchstens 8 MiB, ohne Container-Overhead. Die Traversierungstiefe ist begrenzt, und bei jedem äußeren Datensatz wird Abbruch geprüft. Berichte zählen alle Befunde, aber höchstens 64 repräsentative Probleme passieren die Brücke; sie enthalten nur physische Datensatz-IDs und stabile Gründe, keinen beliebigen Archivfehlertext.

Leere Auswahl, fremde IDs, Indexlimits oder Abbruch liefern Fehler statt teilweise erfolgreicher Berichte. Überlaufende Summen zeigen „Nicht verfügbar“ statt auf null abgeschnitten zu werden. Grundbefunde werden nur gemeldet; sie überschreiben, benennen, ignorieren oder überspringen nichts automatisch.

Unbekannte unkomprimierte Größen melden den stabilen Grund `unknown_size`. `declared_bytes=None` bedeutet unbekannte Gesamtsumme und unterscheidet sich von `Some(0)` für eine bekannte leere Datei. Unbekannte Werte erfinden weder Gesamtsummen- noch unbekannte Methodenbefunde; tatsächlicher Ganzzahlüberlauf erzeugt weiterhin einen Gesamtlimitbefund. Ausgabeaufgaben lehnen Datensätze mit unbekannter Größe separat vor Dekodierung, Staging oder Passwortinteraktion ab.

<a name="xdzip-desktop-entry-point"></a>

## Desktop-Einstieg

Einträge auswählen und „Vor dem Entpacken Metadaten prüfen…“ anklicken. Ergebnisse verwenden ein eigenständiges nichtmodales Top-Level-`OperationWindow` mit dem Hauptfenster als transientem Besitzer und nativer Titelleiste; macOS-Ampelknöpfe zeichnet die Anwendung nicht selbst. Das Schließen der Ergebnisse schließt nur dieses Fenster. Das Schließen des Hauptfensters verwendet kooperativen Abbruch und wartet nicht im GUI-Thread auf Worker.

<a name="xdzip-boundary-with-actual-extraction"></a>

## Grenze zur tatsächlichen Extraktion

[Tatsächliche sichere Extraktion](zip-safe-extraction.md) implementiert Ausgabezählung, CRC- und Strukturprüfung, Zielautorisierung, plattformspezifische Pfadnachprüfungen, Konflikte, Staging und Datei-Commits unabhängig. Sie ist nicht auf die 128 Vorprüfungsauswahlen beschränkt und unterstützt Pläne für 100.000 physische Datensätze und virtuelle Verzeichnisauswahlen. Ein Vorprüfungsbericht wird nicht als Schreibberechtigung wiederverwendet. Beim Aufgabenstart müssen Zielautorisierung erneut eingeholt, Quellen neu validiert und tatsächliche Inhalte gemäß [Dateisicherheitsspezifikation](../design/file-security.md) geprüft werden. Native Fehler und aktuell bekannte Mängel werden in den betreffenden Verträgen und Issues verfolgt.

<a name="xdzip-window-and-result-presentation"></a>

## Fenster und Ergebnisdarstellung

Die Vorprüfung verwendet ein gemeinsames Operationsfenster von 491 logischen Pixeln Breite mit inhaltsangepasstem Körper, maximaler Clientbereichshöhe 600, scrollbarer langer Problemliste und zugänglicher fester Fußleiste. Berechtigungsgrenzen der Daten- und Metadatenphase bleiben unverändert. Parameter und Designquelle stehen in den [Vorprüfungsparametern](../../../specs/visual/preflight-reference.json).

Die Platzierung der Vorprüfungserklärung wurde am 2026-10-15 geändert: „Dieser Bericht prüft nur Metadaten…“ steht am Ende des scrollbaren Körpers in 12-Punkt-Erklärungstext. Die Fußleiste enthält diese Erklärung nicht mehr; sie behält nur Zurück und entspricht bei gleicher Schriftgröße unabhängig von der Berichtslänge der Höhe einer gemeinsamen reinen Aktionsfußleiste.
