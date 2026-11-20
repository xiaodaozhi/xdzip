<a name="xdzip-requirements-and-verification-traceability"></a>

# Nachverfolgbarkeit von Anforderungen und Prüfungen

[English](../../design/requirements-traceability.md) | [简体中文](../../zh-CN/design/requirements-traceability.md) | [繁體中文（台灣）](../../zh-TW/design/requirements-traceability.md) | [日本語](../../ja-JP/design/requirements-traceability.md) | [한국어](../../ko-KR/design/requirements-traceability.md) | [Français](../../fr-FR/design/requirements-traceability.md) | Deutsch | [Español](../../es-ES/design/requirements-traceability.md) | [Português (Brasil)](../../pt-BR/design/requirements-traceability.md) | [Русский](../../ru-RU/design/requirements-traceability.md)

Die [Produktanforderungen](product-requirements.md) definieren die IDs und ihren aktuellen Umfang. Jede Zeile nennt Prüfungen für Änderungen am jeweiligen Verhalten. Ergebnisse, Plattformumfang und Grenzen gehören zum Änderungsnachweis. Die [Abnahmefälle](../../../developers/qa/acceptance.md) nennen die Prüfeinstiege.

<a name="xdzip-requirement-matrix"></a>

## Anforderungsmatrix

| Anforderung | Entwurfsreferenz | Prüfschwerpunkt |
| --- | --- | --- |
| P01 Ohne Entpacken durchsuchen | [Domäne](domain-data.md), [Desktop](qt-quick-ui.md) | T01 Ein echtes Archiv öffnen und prüfen, dass beim Durchsuchen keine Inhalte entpackt werden. |
| P02 Durchsuchen, abfragen und auswählen | [Domäne](domain-data.md), [Leistung](performance.md) | T02 Identität gleichnamiger Einträge, stabile Abfragen, veraltete Seiten, Speicherung großer Indizes und seitenübergreifende Auswahl. |
| P03 Den vorgesehenen Bereich entpacken | [Dateiberechtigung](file-security.md), [Aufgaben](task-scheduling.md) | T03 Verzeichniserweiterung, Duplikate, Umbenennen, Überspringen, Überschreiben und genaue Dateien auf dem Datenträger. |
| P04 Ein Archiv erstellen | [Formate](codec-extension-sdk.md), [Dateiberechtigung](file-security.md) | T04 Unabhängige Rundläufe für das geänderte Profil, Prüfung der Zwischenausgabe und Schutz vorhandener Dateien. |
| P05 Formatfähigkeiten beachten | [Brücke](qt-rust-bridge.md), [Formate](codec-extension-sdk.md) | T05 Register, sitzungsspezifische Aktionen und Laufzeit-Ablehnung nicht unterstützter Profile. |
| P06 Aufgaben begrenzen und Abbruch ermöglichen | [Brücke](qt-rust-bridge.md), [Aufgaben](task-scheduling.md) | T06 Volle Warteschlangen, Zuordnung von Rückfragen, Konkurrenz zwischen Abbruch und Übernahme sowie asynchrones Beenden. |
| P07 Datei- und Ressourcengrenzen schützen | [Dateiberechtigung](file-security.md), [Formate](codec-extension-sdk.md) | T07 Anwendbare native Link- und Identitätsprüfungen, Pfadvalidierung, Quoten und geschützte Objekte. |
| P08 Barrierefreien Desktop bieten | [Desktop](qt-quick-ui.md) | T08 Verhalten von Modellen und Bedienelementen, native Tastatur, IME, Barrierefreiheit, Skalierung und Fokus auf betroffenen Plattformen. |
| P09 Ressourcen begrenzen und messen | [Leistung](performance.md) | T09 Vergleichbarer Build, Prozessbaum-Messwerte, Start, Sortierung, Ausgabedurchsatz oder Paketgröße je nach Änderung. |
| P10 Formatlogik unabhängig von Qt halten | [Architektur](system-architecture.md), [Formate](codec-extension-sdk.md) | T10 Kern ohne Qt, ausdrückliche Adapterauswahl und formatspezifische Regressionen. |
| P11 Geheimnisse und Daten schützen | [Dateiberechtigung](file-security.md), [Datenschutz](../engineering/privacy.md) | T11 Begrenzte Passwortverwendung, bereinigte Diagnose und bewusster Netzwerkzugriff. |
| P12 Auslieferung prüfen | [Qualität](quality-engineering.md), [Veröffentlichung](build-release.md) | T12 Fixierte Eingaben und Artefaktidentität, Inventar, Lizenzmaterial, Paketvertrauen und passende Installationsprüfungen. |
| P13 Externe Erweiterungen kontrollieren | [Formate](codec-extension-sdk.md), [SDK-Versuch](../engineering/sdk-trial.md) | T13 Interoperabilität und native Berechtigungen des Versuchs. Die Installation von Erweiterungen im Produkt bleibt eine Entwurfsoption. |
| P14 Vorschau und Öffnen begrenzen | [Desktop](qt-quick-ui.md), [Dateiberechtigung](file-security.md) | T14 Automatische Vorschau, unterstützte Inhalte, Prozesslimits und private temporäre Ausgaben. |
| P15 Tatsächliches Ergebnis melden | [Brücke](qt-rust-bridge.md), [Dateiberechtigung](file-security.md), [Aufgaben](task-scheduling.md) | T15 Quell- und Zieländerungen, Fehler und Abbruch nach bereits übernommenen Dateien, Bereinigung und Unsicherheit. |

<a name="xdzip-cross-cutting-verification"></a>

## Übergreifende Prüfungen

| Prüfgruppe | Betroffene Änderungen | Prüfschwerpunkt |
| --- | --- | --- |
| T16 Eigentümerschaft und Lebenszyklus der Brücke | Lebensdauer von Brücke, Verbindung oder Controller | Aktuelle Anforderungsidentität, Aufwecken des Empfängers, erhaltene Aufgaben und sichere Freigabe. |
| T17 Formatinteroperabilität und Fehlerpfade | Format- oder Codecänderungen | Soweit nötig unabhängige gültige Testdatei, Kürzung oder nicht unterstützte Profile und begrenzte Ausgabe. |
| T18 Installation, Update und Entfernung | Änderungen der Plattform oder des Vertriebskanals | Tatsächliches Paket, Laufzeitabhängigkeiten in sauberer Umgebung, Installationsidentität, Updateübergabe und unterstützte Entfernung oder Wiederherstellung. |

<a name="xdzip-representative-workflows"></a>

## Beispielabläufe

1. ZIP öffnen und durchsuchen, ausgewählte Ergebnisse fixieren, entpacken und Ausgabebytes vergleichen. „Alles entpacken“ bezeichnet weiterhin das ganze Archiv.
2. Gleichnamige oder kollidierende Einträge mit festgelegter Konfliktregel entpacken und resultierende Namen und Status prüfen.
3. Vor und während der Übernahme abbrechen, die Meldung bereits abgeschlossener Dateien prüfen und bei einem Erstellungsfehler vor der Übernahme den Erhalt früher geprüfter Archive bestätigen.
4. Quelle oder Ziel während eines kontrollierten Tests ändern und das tatsächliche Ergebnis prüfen. Nach abnormalem Ende verbliebene Daten untersuchen, ohne automatische Wiederholung vorauszusetzen.
5. Einen mitgelieferten Formatprozess oder den separaten SDK-Versuch gemäß Vertrag fehlschlagen lassen und prüfen, dass unvollständige Daten nicht als Erfolg veröffentlicht werden.

Statische Schreib-, Format- und Linkprüfungen validieren die Dokumentation. Fixture-, Offscreen-, native automatisierte und manuelle Abnahmenachweise gelten jeweils für ihren eigenen Umfang. [Qualitätssicherung](quality-engineering.md), [Prüfanleitung](../../../developers/qa/README.md)
