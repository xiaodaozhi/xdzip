<a name="xdzip-product-runtime-and-qt-lifecycle"></a>

# Produktlaufzeit und Qt-Lebenszyklus

[English](../../engineering/product-runtime.md) | [简体中文](../../zh-CN/engineering/product-runtime.md) | [繁體中文](../../zh-TW/engineering/product-runtime.md) | [日本語](../../ja-JP/engineering/product-runtime.md) | [한국어](../../ko-KR/engineering/product-runtime.md) | [Français](../../fr-FR/engineering/product-runtime.md) | Deutsch | [Español](../../es-ES/engineering/product-runtime.md) | [Português (Brasil)](../../pt-BR/engineering/product-runtime.md) | [Русский](../../ru-RU/engineering/product-runtime.md)

Diese Anleitung beschreibt die Verbindung der Desktopanwendung zur Rust-Laufzeit über Befehle und Ereignisse einschließlich Warteschlangenbegrenzung, Herunterfahren und plattformspezifischer Builds. Die jeweiligen Fähigkeiten stehen in den Verträgen zum [schreibgeschützten Durchsuchen](../../engineering/readonly-browser.md) und [sicheren Entpacken](zip-safe-extraction.md).

<a name="xdzip-implemented-services"></a>

## Implementierte Dienste

`xdzip-runtime::service` implementiert den Produktdienst. Die Oberfläche übernimmt `CommandClient` genau einmal, ein eigener Qt-Empfangsthread übernimmt `EventReceiver` genau einmal. Der Rust-Befehlsthread besitzt das geöffnete Archiv der Verbindung und den Navigations-Snapshot. Ein separater Rust-Aufgabenthread empfängt vorbereitete Arbeit samt benötigter Quelle und Einstellungen und veröffentlicht Aufgabenaktualisierungen. Archiv-Handles und synchrone Sucheinstiege bleiben in Rust. Die [Qt/Rust-Brücke](../design/qt-rust-bridge.md) beschreibt die ABI, die [Aufgabenplanung](../design/task-scheduling.md) Besitzverhältnisse und Zulassung.

Zu den Kernbefehlen gehören Open, Browse, Preflight, CloseSession und Snapshot; alle vorhandenen tatsächlichen Navigations- und Vorprüfungen nutzen sie. Jede angenommene gewöhnliche Anfrage erhält eine nicht wiederverwendete ID und liefert höchstens ein begrenztes Datenereignis und dann genau ein Finished; Fehler liefern unmittelbar Finished. Seiten tragen Sitzungs-, Anfrage- und Offset-Identität. Jede RuntimeConnection besitzt eine Sitzung und serielle ausstehende GUI-Anfragen. Abfragen enthalten zudem eine vollständige Abfrage-ID, GUI-Generation und Snapshot-Revision; siehe [ZIP-Navigationsvertrag](zip-readonly-browsing.md). Sitzungsübergreifende Anfragen liefern stale_session.

Die gewöhnliche GUI-Zulassung verwendet try_lock, ohne auf freien Warteschlangenplatz zu warten, und prüft Pfade bis 4 KiB, Suche bis 1 KiB, 128 ausgewählte Elemente, 21 Byte lange Verzeichnis-IDs und Sortiercodes 0–12. Eine Ablehnung verbraucht keine ID; Busy kann erneut versucht werden. Abbruch hat einen eigenen Steuerkanal mit höchstens 128 Elementen und eigener Finished-Bestätigung; diese garantiert nicht, dass das Ziel nicht schon beendet ist. Das Herunterfahren behält eine unabhängige idempotente RequestId und reserviert die letzte verfügbare ID, damit es auch bei erschöpften gewöhnlichen IDs oder voller Warteschlange funktioniert.

Der Qt-Controller hält nur einen Befehl, der auf Zulassung wartet. Vorübergehendes Busy wird in der Ereignisschleife mit einem einmaligen Timer von 1 ms höchstens 32-mal erneut versucht; erst anhaltende Auslastung fordert zum erneuten Versuch auf. Das ist keine Frist für den Abschluss. Abbruch, Herunterfahren oder Trennung ziehen nicht angenommene Befehle sofort zurück; im Leerlauf gibt es keinen Wiederholungstimer. Ungültige Argumente und erschöpfte IDs werden getrennt gemeldet und nicht als Busy ausgegeben. Nach einer Trennung ist Open deaktiviert.

<a name="xdzip-bounded-transport"></a>

## Begrenzter Transport

- Höchstens 128 gewöhnliche Anfragen sind ausstehend; ihre Kapazität wird erst frei, wenn der Empfänger Finished entnimmt. Pro Anfrage sind zwei Ereignisplätze reserviert, damit Daten Endzustände nicht verdrängen.
- Rust-Datenereignisse haben ein kumuliertes Inhaltsbudget von 32 MiB, einschließlich Bildern von höchstens 16 MiB und deren Containern. Bei Überschreitung wartet die Veröffentlichung; Abbruch oder Herunterfahren weckt Wartende und kann stattdessen einen abgebrochenen Endzustand ausgeben. Ein einzelnes übergroßes Ereignis liefert resource_limit.
- Rust-Befehle behalten ein kumuliertes Inhaltslimit von 8 MiB; Überschreitungen werden bei der Zulassung abgelehnt und das größere Bildereignisbudget ändert daran nichts.
- Die Qt-Mailbox fasst höchstens 256 Ereignisse und 32 MiB Inhaltsbytes. Höchstens 128 Plätze sind gewöhnliche Daten; die übrigen sind für End- und Steuerereignisse reserviert. Sie erzeugt nicht pro Rust-Ereignis einen Qt-Rückruf.
- Eine Benachrichtigungskette verarbeitet pro Durchlauf höchstens 32 Ereignisse oder etwa 4 ms und plant bei Restarbeit einen weiteren Durchlauf; die Verarbeitung eines einzelnen Ereignisses ist nicht unterbrechbar. Leere Warteschlangen beenden die Planung, der Empfänger blockiert an einer Bedingungsvariablen.
- Open-Page enthält zusätzlich den vollständigen begrenzten Verzeichnisindex mit höchstens 32.768 Knoten und 4 MiB Namen/Präfixen, durch `replace_directories` zum Ersetzen markiert; auch ein leerer Index ersetzt ausdrücklich. Browse sendet den Index nicht erneut; `navigation_limited` kennzeichnet ausdrücklich eine eingeschränkte Indizierung. Rust zählt Namens- und Präfixbytes; die Qt-Mailbox zählt außerdem die von der Brücke erzeugten Knoten- und Eltern-ID-Zeichenfolgen. Knotencontainer und Tiefe haben eigene Grenzen. Neue Seite und Verzeichnismodell erst nach erfolgreichem Finished der zugehörigen Anfrage gemeinsam übernehmen.
- `total` einer Page zählt die Zeilen der aktuellen Liste; `archive_total` hält unabhängig die Zahl physischer Archivdatensätze fest. Normales Durchsuchen zeigt direkte Dateien und Verzeichnisse des aktuellen Verzeichnisses, die Suche liefert rekursiv physische Treffer. Verzeichnisse ohne ausdrücklichen Datensatz nutzen `navigation_id` und `physical=false`; Qt-Zeilenschlüssel sind `dN`, rohe physische IDs bleiben getrennt und virtuelle Knoten können kein Preflight auslösen. Vollständige Pfade bleiben in den Metadaten; angezeigt werden beim Durchsuchen der letzte Name und bei Suche der vollständige Pfad.
- Zeichenfolgenbudgets sind kein RSS: Container, Snapshots und ein Ereignis in Konvertierung benötigen zusätzlichen Speicher, Einträge und Seiten bleiben jedoch begrenzt. Brückenbatches enthalten höchstens 128 Elemente. Der [ZIP-Navigationsvertrag](zip-readonly-browsing.md) beschreibt die ursprüngliche Basis von 100.000 Einträgen, die [Indizierung großer Archive](large-index.md) die spätere Erweiterung gewöhnlicher ZIP-Dateien mit einer Datei.

Steuer- und gewöhnliche Ereignisse werden abwechselnd entnommen, damit anhaltende Abbruchbestätigungen gewöhnliche Endzustände nicht verdrängen. Bei der Auslieferung werden steigende Sequenznummern vergeben. Qt verwirft alte Sequenzen; Steuerbestätigungen und veraltete request_ids ändern die aktuelle Ansicht nicht.

`Task`/`TaskUpdate`-Umschläge auf dem unabhängigen Aufgabenkanal behalten die ursprüngliche request_id der Aufgabenerstellung. Korrelationsdatensätze bleiben bei begrenzten Aufgabensteuerplätzen und werden mit der Auslieferung des Aufgabenendzustands freigegeben. Sie können vor Annahmedaten eintreffen oder auf Finished der Anfrage folgen. Sie verbrauchen keinen zweiten gewöhnlichen Datenplatz und geben die gewöhnliche Anfrage nicht erneut frei oder schließen sie erneut ab; sie ordnen Vorbereitungsplatzhalter korrekt der tatsächlichen TaskId und Quellsitzung zu. Diese ID ist keine Abbruchanfrage-ID; die Aufgabensteuerung verwendet weiterhin TaskId.

<a name="xdzip-shutdown-destruction-and-failure"></a>

## Herunterfahren, Zerstörung und Fehler

Das Schließen eines Archivs oder Fensters kann Anwendung und Aufgaben im Infobereich weiterlaufen lassen. Beim Beenden oder Neustart stoppt der Koordinator die Zulassung, fordert Abbruch an und leert Ereignisse, während Rust seine Sitzungen freigibt. Der Empfänger wartet auf beide Rust-Threads, bevor er `ShutdownReady` zurückgibt. Der Desktop wartet auf das Ende des Qt-Empfängers und das Leeren der Mailbox. Der GUI-Thread wartet nie auf Threads, die selbst auf Dateiarbeit oder Ereigniszustellung warten.

Die Abbruch- und Abschaltsteuerung verwendet kurze kritische Abschnitte mit begrenzten Zustandsoperationen, unabhängig von der Kapazität der Datenwarteschlange. Kooperativer Abbruch kann einen vom Betriebssystem blockierten Lesevorgang nicht unterbrechen und garantiert kein hartes Echtzeitende. Schreibgeschützte Anfragen erzeugen hier keine Ausgabetransaktionen. Abschlüsse von Erstellung/Extraktion, temporäre Bereinigung und Endergebnisse folgen getrennt den Verträgen für [Erstellung](zip-creation.md) und [sicheres Entpacken](zip-safe-extraction.md).

Wird ein QObject frühzeitig zerstört, Mailbox schließen, Empfänger wecken und Dienst stoppen. Hintergrundarbeit hält nur gemeinsame Warteschlangen und das Rust-Empfänger-Handle, keine Controller-Zeiger. Qt storniert Benachrichtigungen zerstörter Objekte automatisch. QPointer schützt die Lebensdauer auch, wenn ein Ereignisrückruf die Verbindung synchron zerstört. Beim Aufgeben des Empfangs im Destruktor nicht auf Threads warten; die Arbeit endet kooperativ. Nur ein normales ShutdownReady belegt, dass das Warten abgeschlossen wurde.

Löst ein schreibgeschützter Worker-Handler einen abwickelbaren Panic aus, fängt die Grenze ihn ab, verwirft den gesamten Dienst, sendet RuntimeFault und beendet unvollständige Anfragen als runtime_unavailable; die Sitzung wird nicht wiederverwendet. Das definiert nur Lesefehler und verspricht keine Wiederherstellung halb abgeschlossener Erstellungs- oder Extraktionstransaktionen. Bestätigte Elemente, ungewisse Ergebnisse und Bereinigung bei Schreibvorgängen folgen ihren Verträgen. Nicht behebbare Allokationsfehler, Prozessabbruch oder vergiftete interne Sperren garantieren keine Endzustandszustellung. Qt beendet bei unerwartetem EOF oder Verbindungsfehler ausstehende Zustände einheitlich, um dauerhafte Auslastung zu verhindern.

<a name="xdzip-verification-and-build"></a>

## Prüfung und Build

Siehe [allgemeine Prüfung](../../../developers/qa/README.md). Rust-Kerntests benötigen kein Qt; Qt-Fenster-, Modell-, Last- und Abschalttests werden auf drei Plattformen nativ gebaut und ausgeführt. macOS-CI prüft arm64 und Cocoa-Start in der SDK-Umgebung ohne App-Deployment oder Upload; lokale Entwicklungs-DMGs sind auf ausdrückliche Anfrage möglich. Windows/Linux deployen das mitgelieferte Qt und prüfen Ausführung ohne SDK; siehe [Build-Anleitung](../../engineering/developer-build.md). Diese Prüfungen sind keine Freigabe einer offiziellen Veröffentlichung.

Erste Abfragen vergeben eine unveränderliche Ergebnisidentität; spätere Batches müssen allen QueryKey-Feldern und Parametern entsprechen. Qt bricht alte Anfragen ab und bündelt die neueste Absicht, während Sequenz und Anfrage-ID weiter geprüft werden. Ein Batch begrenzt Namen zusätzlich auf 64 KiB und filtert oder sortiert nicht erneut. Historische Abnahme bleibt auf die damals verfügbaren Fähigkeiten begrenzt.

Die Anwendungsschicht kann mehrere unabhängige Verbindungen erzeugen, während die Domänenlaufzeit Sitzungen pro Verbindung isoliert. Der [Vertrag zu Anwendungssitzungen](application-sessions.md) beschreibt globale Aufgabenidentität und serielle Zulassung schwerer Aufgaben.
