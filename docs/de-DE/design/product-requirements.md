<a name="xdzip-xdzip-product-requirements"></a>

# Produktanforderungen von XDZip

[English](../../design/product-requirements.md) | [简体中文](../../zh-CN/design/product-requirements.md) | [繁體中文](../../zh-TW/design/product-requirements.md) | [日本語](../../ja-JP/design/product-requirements.md) | [한국어](../../ko-KR/design/product-requirements.md) | [Français](../../fr-FR/design/product-requirements.md) | Deutsch | [Español](../../es-ES/design/product-requirements.md) | [Português (Brasil)](../../pt-BR/design/product-requirements.md) | [Русский](../../ru-RU/design/product-requirements.md)

XDZip lässt Archive wie Ordner durchsuchen, ausgewählte Inhalte bearbeiten und Archive erstellen oder ändern, wenn das Format dies unterstützt. Desktop und [eigenständige CLI](../engineering/command-line.md) teilen sich die Archivlaufzeit. Die [Projekt-README](../../../README.de-DE.md) beschreibt aktuelle Funktionen und Distributionen. Die folgenden Anforderungen legen das zu bewahrende und zu prüfende Verhalten fest.

<a name="xdzip-requirements-and-implementation"></a>

## Anforderungen und Implementierung

| ID | Anforderung | Aktuelles Verhalten oder Umfang |
| --- | --- | --- |
| P01 | Ohne Entpacken durchsuchen | Beim Öffnen wird die Quelle erkannt und der begrenzte Scan abgeschlossen, bevor die erste Seite erscheint. Das Dekodieren von Eintragsinhalt erfordert eine eigene Aktion. |
| P02 | Navigieren, suchen, sortieren und auswählen | Stabile physische IDs, getrennte Navigationsidentitäten, paginierte Abfragen und begrenzte Auswahl sind implementiert. Gewöhnliches ZIP/ZIP64 in einer Datei hat einen größeren Metadatenindex und eigene Aufgabenlimits. |
| P03 | Den vorgesehenen Bereich entpacken | Die Laufzeit fixiert den Bereich, prüft Namen und autorisiert das Ziel. Sie verarbeitet Konfliktentscheidungen und meldet tatsächliche Übernahmen, übersprungene Einträge, Fehler und Abbruch. |
| P04 | Archive aus Dateien und Ordnern erstellen | Optionen werden gegen das Erstellungsregister geprüft. Die Ausgabe wird zwischengespeichert und vor der Übernahme geprüft. Formate und Optionen stehen im Register. |
| P05 | Formatfähigkeiten beachten | Die Oberfläche kombiniert Formatbeschreibung und Sitzungszustand. Jede Anforderung wird zusätzlich zur Laufzeit geprüft. Lesen, Vorschau, Prüfung, Erstellung, Passwörter und Volumes haben eigene Grenzen. |
| P06 | Aufgaben begrenzen und Abbruch ermöglichen | Metadaten und aufwendige Aufgaben nutzen getrennte Warteschlangen. Aufwendige Desktopvorgänge laufen nacheinander. Eine angenommene Aufgabe kann nach dem Schließen ihres Tabs weiterlaufen. |
| P07 | Schreibvorgänge auf geprüfte Berechtigungen und Grenzen beschränken | Der Host prüft Namen, Links, Dateiidentität, Zwischenspeicherung, tatsächliche Ausgabe und Übernahme. Formatprüfungen validieren die jeweilige Struktur und Integrität. |
| P08 | Einheitliche und barrierefreie Bedienelemente bereitstellen | QML und C++ bieten gemeinsame Bedienelemente, Tastaturnavigation, Fokus und Barrierefreiheit. Native IME, Bildschirmleser und Anzeigeverhalten benötigen eigene Prüfungen. |
| P09 | Ressourcen begrenzen und messen | Der Code erzwingt unabhängige Budgets. Leistungsziele und Messergebnisse werden getrennt verfolgt. |
| P10 | Formatlogik unabhängig von Qt halten | Rust-Traits, explizite Registerauswahl und eigene Prozessschnittstellen ermöglichen integrierte Implementierungen. Desktop und CLI nutzen denselben Kern. |
| P11 | Geheimnisse und Daten schützen | Archivvorgänge arbeiten offline. Passwörter liegen in begrenztem, löschbarem Speicher. Datenschutz, Diagnose und ausdrücklicher Updatezugriff folgen eigenen Verträgen. |
| P12 | Auslieferung prüfen | Definierte Paketkanäle, fixierte Eingaben, Manifeste, Prüfsummen, Lizenzmaterial und Plattformprüfungen verwenden. Ergebnisse gelten für den tatsächlich geprüften Build und Vertriebsweg. |
| P13 | Erweiterungen Dritter nur mit geprüftem Vertrauen und Berechtigungen zulassen | Der schreibgeschützte XDP1-Versuch ist separat implementiert. Installation und Aktivierung von Erweiterungen im Produkt bleiben Entwurfsarbeit. |
| P14 | Vorschau und Öffnen begrenzen | Ist der Eigenschaftsbereich sichtbar, startet die Auswahl einer unterstützten Datei eine begrenzte Text- oder Bildvorschau. Das ausdrückliche Öffnen eines geeigneten regulären Eintrags nutzt eine geprüfte private Kopie und die Systemzuordnung. |
| P15 | Fehler und ungewisse Ergebnisse genau melden | Stabile Fehler und begrenzte Ergebnisse erfassen Quelländerungen, Datenträgerfehler, übersprungene Einträge und Bereinigung. Automatische Wiederherstellung nach Neustart bleibt eine Entwurfsoption. |

<a name="xdzip-workflows-and-scope"></a>

## Abläufe und Umfang

- **Öffnen und ansehen.** Eine Öffnungsanforderung erreicht die Desktop-Sitzungssteuerung. Ein erfolgreicher Scan veröffentlicht eine Sitzung. Suche und Sortierung erzeugen neue Abfragen. Doppelklick öffnet einen Ordner oder eine geeignete Datei über eine temporäre Kopie. Bei sichtbarem Eigenschaftsbereich startet die Auswahl einer unterstützten Datei deren Vorschau. [Durchsuchen und Vorschau](../user-guide/browse.md), [Temporäres Öffnen](../engineering/archive-opening.md)
- **Entpacken.** „Alles entpacken“ umfasst das ganze Archiv. Für einzelne Treffer diese in der Suche auswählen und den Bereich prüfen. Eine Metadaten-Vorprüfung liefert Hinweise vor der Schreibautorisierung. Bereits übernommene Dateien bleiben bei späterem Fehler oder Abbruch erhalten. [Entpacken](../engineering/zip-safe-extraction.md), [Aufgabenplanung](task-scheduling.md)
- **Erstellen, ändern oder prüfen.** Erstellung und zulässige Änderungen prüfen die Zwischenausgabe vor der Übernahme. „Prüfen“ kontrolliert ein geeignetes Archiv entsprechend seinem Format ohne Ausgabeberechtigung. Verfügbare Aktionen hängen von Format, Eintrag und Quellprofil ab. [Formatregister](../engineering/format-registry.md), [Archivtransfers](../engineering/archive-transfers.md), [Archivprüfung](../engineering/archive-checking.md)

<a name="xdzip-planned-scope-and-verification"></a>

## Geplanter Umfang und Prüfung

Externe Erweiterungsverwaltung, dauerhafte Aufgabenwiederherstellung und weitere Formatprofile benötigen eigene Gestaltung, Implementierung und Prüfung. Versionsnummern allein belegen keine Format- oder Plattformabdeckung. Cloud-Archivabgleich, Einhängen von Datenträgerabbildern und ein Malware-Prüfdienst gehören nicht zum Produktumfang. Die [Nachverfolgbarkeitsmatrix](requirements-traceability.md) ordnet Anforderungen Prüfungen zu. [Support-Richtlinie](../engineering/support-policy.md) und Versionshinweise beschreiben die ausgelieferten Builds.
