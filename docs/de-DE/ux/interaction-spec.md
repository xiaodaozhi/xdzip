<a name="xdzip-additional-interaction-and-copy-specification"></a>

# Ergänzende Interaktions- und Textspezifikation

[English](../../ux/interaction-spec.md) | [简体中文](../../zh-CN/ux/interaction-spec.md) | [繁體中文（台灣）](../../zh-TW/ux/interaction-spec.md) | [日本語](../../ja-JP/ux/interaction-spec.md) | [한국어](../../ko-KR/ux/interaction-spec.md) | [Français](../../fr-FR/ux/interaction-spec.md) | Deutsch | [Español](../../es-ES/ux/interaction-spec.md) | [Português (Brasil)](../../pt-BR/ux/interaction-spec.md) | [Русский](../../ru-RU/ux/interaction-spec.md)

Hauptfenster und Entpackbestätigung werden durch die [Vorgangszustände](hifi-state-spec.md) für Erstellung, Konflikte, Aufgaben, Ergebnisse, Einstellungen und Plattformvergleiche ergänzt. Willkommen und Ladefortschritt stehen im [Vertrag der Startseite](welcome-opening.md). Aktuelle Entwürfe und Layoutwerte folgen der [Fensterregistrierung](../../../specs/visual/window-registry.json). Die jeweiligen technischen und Interaktionsverträge erläutern, welche Zustände implementiert und nativ visuell geprüft sind.

| Situation | Anzeige und Aktion | Fortsetzung |
| --- | --- | --- |
| Erster Start | Archiv öffnen oder erstellen. Lokale Dateien per Ziehen übernehmen. | Vollständige Datenträgerscans und automatische Zuordnung aller Formate vermeiden. |
| Verzeichnis lesen | Gelesene Anzahl und unbestimmter Fortschritt. Gesamtzahl unbekannt. | Sitzung lässt sich schließen. Teilergebnisse als unvollständig kennzeichnen. |
| Leeres Archiv | „Dieses Archiv ist leer“. Dateieigenschaften bleiben sichtbar. | Anderes Archiv öffnen. Entpacken deaktivieren und den Grund anzeigen. |
| Keine Treffer | Suchbegriff und Suchbereich anzeigen. | Suche löschen. Das Archiv bei einer erfolglosen Suche nicht als leer anzeigen. |
| Nicht unterstützte Funktion | Betroffenes Format, Verfahren oder Verschlüsselung nennen. | Lesbare Metadaten erhalten und unsichere Teilausgabe unterbinden. |
| Archiv erstellen | Eingaben, Format, Ziel, Stufe und Vorprüfung. | Fehler beim Feld anzeigen. Vorhandene Dateien standardmäßig erhalten. |
| Namenskonflikt | Beide Einträge, Überspringen, Umbenennen und Überschreiben sowie den Geltungsbereich anzeigen. | Beim Schließen wartet die Aufgabe weiter. Im Aufgabenbereich erneut öffnen oder abbrechen. |
| Aufgabe beendet | Anzahl der Erfolge, übersprungenen Einträge, Fehler und übernommenen Ergebnisse. | Nach dem Entpacken das Ziel auf Wunsch öffnen. Nach der Erstellung kann die App ein unterstütztes Einzelarchiv oder dessen erstes Volume öffnen. Für einen erneuten Versuch neu planen. |
| Teilfehler | Übernommene Dateien erhalten und Fehler erläutern. | Bereits geschriebene Benutzerdateien am Ziel lassen. |
| Anwendung beenden | Aktive Aufgaben zählen. Auf Abschluss warten oder abbrechen und beenden. | Der Kern bestätigt die Bereinigung. Die GUI bleibt während des Beendens bedienbar. |
| Einstellungen | System, helles oder dunkles Farbschema, Dichte, Textgröße, Sprache, Verlauf und Infobereich. | Nach erfolgreichem Speichern sofort anwenden. Bei Fehlern alte Werte erhalten. Quoten können derzeit nicht bearbeitet werden. |
| Formaterweiterungen | Das Produkt nutzt integrierte Formate. Der [externe SDK-Versuch](../engineering/sdk-trial.md) prüft schreibgeschützte Ausführung separat. | Unterstützte Vorgänge stehen im [Formatregister](../engineering/format-registry.md). |

Die Entpackschaltfläche richtet sich nach der Auswahl. Das Menü bietet beide Bereiche, sofern der aktuelle Zustand sie unterstützt. Die Suche erfasst das aktuelle Verzeichnis und seine Unterordner. Bei Auswahl einer unterstützten Datei zeigt der sichtbare Eigenschaftsbereich automatisch eine [Text- oder Bildvorschau](../engineering/content-preview.md). Unterhalb der nötigen Breite öffnet sich der Bereich bei Bedarf separat. Größere Schrift vergrößert den Inhalt. Scrollen hält die Bestätigung erreichbar.

Fehlermeldungen beschreiben das Problem und die nächste mögliche Aktion. Fehlercodes gehören in die Details. Die Bedienoberfläche zeigt weder Aufrufstapel noch IPC-Fachbegriffe. Zeigen Sie gefährliche Dateinamen als Text und verarbeiten Sie Erweiterungsmeldungen ohne HTML-Auswertung. Tastenkürzel, IME, Fokus und Bildschirmleser folgen dem [Qt-Quick-Entwurf](../design/qt-quick-ui.md).
