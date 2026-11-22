# Änderungsprotokoll

[English](CHANGELOG.md) | [简体中文](CHANGELOG.zh-CN.md) | [繁體中文](CHANGELOG.zh-TW.md) | [日本語](CHANGELOG.ja-JP.md) | [한국어](CHANGELOG.ko-KR.md) | [Français](CHANGELOG.fr-FR.md) | Deutsch | [Español](CHANGELOG.es-ES.md) | [Português (Brasil)](CHANGELOG.pt-BR.md) | [Русский](CHANGELOG.ru-RU.md)

<a id="unreleased"></a>

## Unveröffentlicht

- Vereinfachtes und traditionelles Chinesisch stehen mit den Bezeichnungen „中文 (简体)“ und „中文 (繁体)“ gemeinsam oben in der manuellen Sprachauswahl.
- Einen eigenständigen, schreibgeschützten SDK-Versuch mit begrenzter CBOR-Verarbeitung, kontrollierter Ein-/Ausgabe, zwei unabhängigen Protokollimplementierungen und einer Sandbox-Prüfung hinzufügen, die unsichere Ausführung ablehnt. Drittanbietererweiterungen bleiben im Produkt deaktiviert.

## 0.3.0

- Zehn Sprachen für die Desktopoberfläche, die Systemmenüs und die Benutzerdokumentation unterstützen. Die Systemsprache lässt sich automatisch übernehmen oder manuell ändern.
- Die Archivbearbeitung auf weitere Formate ausweiten. Geeignete Einträge lassen sich hinzufügen, ersetzen, verschieben, umbenennen und löschen.
- Dateivorschauen automatisch laden und bei Bedarf aktualisieren. Archivbefehle, Dateizuordnungen unter macOS sowie die Übergabe von Fenstern und Dateien unter Ubuntu verbessern.

Die [Versionshinweise](release-notes/0.3.0.md) und das [GitHub-Release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.0) enthalten Angaben zu unterstützten Plattformen, Prüfumfang und bekannten Einschränkungen.

## 0.2.1

- Das Durchsuchen und Entpacken unter anderem für LHA/LZH, ARJ, ARC, DEB, ASAR, Zstandard und LZ4 ausweiten. Das Erstellen von LZH-Archiven, ISO9660/Joliet-Datenabbildern und selbstentpackenden ZIP-Dateien für Windows x64 ergänzen.
- Gängige Rasterbilder und ein eingeschränktes statisches SVG-Profil in der Vorschau unterstützen. Unter macOS die HEIC/HEIF-Unterstützung des Systems nutzen und Miniaturansichten in einem privaten Prozess mit begrenzten Ressourcen erzeugen. Die Vorschau-Spezifikation beschreibt die Formatgrenzen.
- In gewöhnlichen, einteiligen ZIP/ZIP64-Archiven bis zu eine Million Datensätze mit einem fortlaufend aufgebauten Index auf dem Datenträger erfassen. Umfangreiche Suchergebnisse bei Bedarf laden und die Auswahl über mehrere Seiten hinweg beibehalten.
- Auf Anforderung begrenzte Klartext- und PNG/JPEG-Vorschauen in der Eigenschaftenkarte anzeigen. Die Behandlung von Passwörtern, Integritätsprüfungen und Abbrüchen gemeinsam nutzen.

Die [Versionshinweise](release-notes/0.2.1.md) und das [GitHub-Release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.2.1) enthalten Angaben zu unterstützten Plattformen, Prüfumfang.

## 0.2.0

- Native Updates in unter macOS installierten XDZip-Versionen über eigene HTTP-CONNECT- oder SOCKS5-Proxys und über Direktverbindungen unterstützen.
- Eigenständige CLI-Pakete für Windows, macOS und Linux bereitstellen. Sie bieten Archivbefehle, JSON-Ergebnisse, die ausdrückliche Registrierung des Befehls und anhand signierter Manifeste geprüfte Updates.
- Die offiziellen QtNetwork-Patches für CVE-2026-116337 und CVE-2026-105946 anwenden. Vollständige Patches, Herkunftsnachweise und Anweisungen zum Austausch beibehalten.
- Metadaten der mitgelieferten CLI unter macOS in gewöhnliche App-Ressourcen verschieben, die von der Integritätssignatur des App-Bundles erfasst werden.

Die [Versionshinweise](release-notes/0.2.0.md) und das [GitHub-Release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.2.0) enthalten Angaben zu unterstützten Plattformen, Prüfumfang.

## 0.1.3

- Verhindern, dass der Finder den standardmäßigen, gleichnamigen Entpackordner eines Archivs mit der Endung `.app` für eine beschädigte Anwendung hält.

Die [Versionshinweise](release-notes/0.1.3.md) und das [GitHub-Release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.3) enthalten Angaben zu unterstützten Plattformen, Prüfumfang.

## 0.1.2

- Die Dateiliste beim Verzeichniswechsel, Suchen und Sortieren beibehalten, um Flackern zu reduzieren.
- Unter macOS die Erläuterungen und Schritte zur manuellen Installation verbessern, wenn die automatische Installation nicht verfügbar ist.
- Das Entpacken sicherer, auf das übergeordnete Verzeichnis bezogener symbolischer Links aus ZIP-Archiven unter macOS korrigieren und die Hinweise zur Qt-Laufzeit aktualisieren.

Die [Versionshinweise](release-notes/0.1.2.md) und das [GitHub-Release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.2) enthalten Angaben zu unterstützten Plattformen, Prüfumfang.

## 0.1.0

- Die erste stabile Version mit nativen Installationsprogrammen oder portablen Paketen für Windows, macOS und Linux veröffentlichen. Geeignete Installationen können auf Anforderung aktualisiert werden.
- Scrollen in den Einstellungen, Barrierefreiheit des Aktualisierungsstatus und Unterstützung für Systemproxys verbessern. Das native Verhalten beim Zeigen mit der Maus korrigieren.
- Die Erfassung von Drittanbieterhinweisen korrigieren und Lizenzausdrücke in der Quellcode-SBOM sowie im Abhängigkeitsgraphen der Rust-Laufzeit des Kandidaten vereinheitlichen.

Die [Versionshinweise](release-notes/0.1.0.md) und das [GitHub-Release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.0) enthalten Angaben zu unterstützten Plattformen, bekannten Einschränkungen und Veröffentlichungsmaterialien.

## 0.1.0-alpha.9

- Drei DLLs des Grafikcompilers aus dem Windows SDK aus Installationsprogramm und portablem Paket entfernen. Bei Upgrades werden alte mitgelieferte Kopien aufgeräumt.
- Den Zugang zu Einstellungen und Updates verbessern. Windows-Dateitypbeschreibungen folgen der Sprache der Anwendung.
- Die Minimierung bei macOS-Vollbildübergängen sowie mehrere Dateizuordnungen korrigieren. Den Start und das Laden von Schriften verbessern.
- Die Bereinigung beim Schließen von Archiven, die Prüfung des Entpackziels und Aufgabenhinweise korrigieren.
- Die reguläre CI vereinfachen und vollständige App-Builds und Release-Prüfungen bei Bedarf ausführen. Lizenzmaterialien für Quellcode und Drittanbieter ergänzen.

Die [Versionshinweise](release-notes/0.1.0-alpha.9.md) beschreiben die Plattform- und Lizenzgrenzen dieser experimentellen Vorabversion.

## 0.1.0-alpha.8

- In „Über“ die vom Benutzer ausgelöste Update-Suche ergänzen. Geeignete Installationen lassen sich nach Prüfung von Signatur, Version und Plattform aktualisieren. Bei der zuvor veröffentlichten alpha.6 ist ein manuelles Upgrade nötig.
- Für Archivnamen, Dateinamen und Pfade Systemschriften mit plattformspezifischen Ersatzschriften verwenden. Feste Oberflächentexte bleiben in der Produktschrift. Leere Ordner, leere Archive und ergebnislose Suchen erhalten eigene Hinweise.
- Die Software-OpenGL-Bibliothek aus Windows-Entwicklungspaketen entfernen. Direct3D 11 bleibt voreingestellt, WARP steht für Software-Rendering zur Verfügung.
- Doppelt gezählte aktive Aufgaben korrigieren, wenn ein Statusabbild vor der Aufgabenannahme eintrifft. Annahme, Abbruch und Bereinigung beim Beenden beibehalten.
- Übliche ältere Dateinamencodierungen beim Durchsuchen, Navigieren, Entpacken, temporären Öffnen, Exportieren und Bearbeiten einheitlich erkennen. Unicode-Angaben bevorzugen, Rohdateinamen weiterhin prüfen und geänderte Namen als UTF-8 schreiben. Kurze Namen und gemischte Codierungen können mehrdeutig bleiben.
- Ausgediente Freigabeprüfungen und Berichtsvorlagen entfernen. Beitragsregeln, die Zusammenfassung des Freigabestands und Materialgeneratoren abstimmen. Herkunft und Lizenzierung in der Release-Checkliste verfolgen.
- Geprüfte versionsspezifische Hinweise verwenden und Version und Buildnummer prüfen. Bei Abweichungen des Entwurfs von der geprüften Datei anhalten, damit Wiederholungen die dokumentierten Abnahmegrenzen nicht überschreiben.
- Die Dokumentation zum eigenen macOS-Release, zum Lesen geteilter Archive und zum Änderungsprotokoll aktualisieren.

## 0.1.0-alpha.6

- Veraltete Zeigerzustände nach dem Ausblenden oder Minimieren von Fenstern zurücksetzen. Dazu gehört das kurze rote Aufleuchten der Schließen-Schaltfläche beim Wiederherstellen aus dem Windows-Infobereich. Die Zeigerbereinigung für Browser- und Aktionsfenster gemeinsam nutzen.
- Nummerierte ZIP- und PKZIP-Volumes durchsuchen und entpacken. Volumeübergreifendes Lesen, ZIP64-Offsets, Passwörter und Integritätsprüfungen folgen dem gemeinsamen Entpackpfad. Alle Volumes sind vor Quelländerungen und Überschreiben geschützt.
- Reguläre Archiveinträge über das System aus einer privaten temporären Kopie öffnen. Den Fortschritt nach Annahme durch das System ausblenden und Fehler für einen erneuten Versuch beibehalten. Die Kopien werden nach einer Sieben-Tage-Regel aufbewahrt. Änderungen daran bleiben außerhalb des Archivs.
- Das Speichern des Archivöffnungsmodus korrigieren. Scrollen für überzählige Tabs, eine Tab-Liste und Umsortieren per Ziehen hinzufügen. Inaktive Tabs in normaler Schriftstärke anzeigen.
- Lange GNU-Dateinamen in Git-Quell-TARs unterstützen. Fehlermeldungen auf der Startseite oben fixieren. Beim Schließen eines Fensters dessen Tabs entfernen und Hintergrundaufgaben weiterführen. Den Aufgabenzähler im Infobereich entsprechend beschriften.

## 0.1.0-alpha.1

- Die Debian-Epoche 1 einführen, damit das System Alpha-Pakete als Upgrades früherer Dev-Pakete einordnet. Die Identität vorhandener Testdateien beibehalten.
- Normale, maximierte und Vollbildzustände bei Dialogen, Wiederherstellung aus dem Infobereich und Öffnen externer Dateien beibehalten.
- Einen Prozess und einen Infobereichseintrag je Anmeldesitzung gemeinsam nutzen. Aktivierungs- und Sammelöffnungsanfragen beim Start weiterleiten, Zeitüberschreitungen ablehnen und nach Abstürzen einen veralteten Sitzungszustand bereinigen.
- Wahlweise Tabs oder getrennte Fenster mit unabhängigen Archiv-Controllern und Browserzuständen hinzufügen. Aufgaben und Ergebnisse nach Schließen einer Seite behalten, nach Sitzungs- und Aufgabenkennung zuordnen und die Annahme aufwendiger Aufgaben serialisieren.
- Vorabversionsnummer und Buildnummer trennen. Sechs plattformübergreifende Pakettypen, unveränderliche Manifeste, Prüfsummen, Quellcode- und Lizenzmaterialien sowie Veröffentlichungsschritte vorbereiten. Automatische Updates standen zu diesem Zeitpunkt noch aus, die Release-Lizenzierung wurde separat verfolgt.
- Alle ausdrücklich gewählten regulären Eingaben beim Erstellen beibehalten, auch ein vorhandenes Ziel oder ein älteres Volume. Die Ausgabe privat zwischenspeichern und Konflikte bei der endgültigen Ausgabe behandeln. Ein auch als Eingabe gewähltes Ziel bewahren und einen anderen Namen oder den Abbruch anbieten.
- Aufgabenmenüeinträge nach Aufgaben-ID direkt aktualisieren, damit der Fortschritt kein Flackern auslöst. Zeigerzustand, Hervorhebung und Tastaturfokus beibehalten und nur in den Verlauf aufgenommene oder daraus entfernte Einträge ändern.
- macOS-Entwicklungs-DMGs auf arm64-Qt-Binärdateien, den Qt-Quick-Stil Basic und benötigte gemeinsame Abhängigkeiten beschränken. Architektur, Stil und Größe dokumentieren und das endgültige Paket prüfen. Ursprüngliche App und SDK erhalten und eine lokale Ad-hoc-Integritätssignatur verwenden.
- Unter Windows nur den festgelegten Qt-Quick-Stil Basic ausliefern, QML-Debug-Plug-ins und Entwicklungs-Typmetadaten entfernen sowie nativen Stil und Grafikersatz beibehalten. Das Laden von Qt-Systemübersetzungen korrigieren und dieselbe festgelegte VC++-Laufzeit in portablem Paket und benutzerbezogenem Installationsprogramm mitliefern. Die Bereitstellungsprüfung lehnt Laufzeiten des Hostsystems ab.
- xdzip als Namen der ausführbaren Hauptdatei für Windows, macOS und Linux vereinheitlichen. Startmenüeinträge, Ressourcen, CI, Quellcode-Builds und Dokumentation abstimmen und die Upgrade-Prüfung älterer Builds weiterhin unterstützen.
- Einteilige AES-256-ZIP-Archive mit AE-1/AE-2 und Store/Deflate entpacken. Passwörter innerhalb von Aufgaben abfragen und Wiederholen sowie Abbrechen ermöglichen. Dateien erst nach Authentifizierung endgültig schreiben. Entwürfe, Oberflächentexte, Protokollabdeckung und Interoperabilitätsprüfungen aktualisieren.
- „Öffnen mit“-Einträge für macOS-Entwicklungskopien gezielt bereinigen. Neue Registrierungen nach Prüfung oder Fehler entfernen und Haupt-App, Dateien und Systemvorgaben bewahren. Grenzfall- und Fehlerbereinigungstests ergänzen.
- Die Entwicklungs- und Abnahmearbeiten mit Genehmigung des Eigentümers abschließen. Zehn automatisierte und acht manuelle Prüfungen sowie die Support-Richtlinie für den eingefrorenen Kandidaten dokumentieren. Herkunft und Lizenzen in der Release-Checkliste und in Issues verfolgen. Die formelle Veröffentlichung stand noch aus.
- Die native macOS-Menüleiste und Infobereichsmenüs auf drei Plattformen hinzufügen. Befehle, Bearbeitungsfokus und Aufgabenstatus gemeinsam nutzen. Nach Schließen des letzten Fensters läuft die Anwendung standardmäßig im Infobereich weiter. Das lässt sich in den Einstellungen abschalten. Beim Beenden und Neustart bleiben Aufgabenbestätigung und Bereinigung erhalten. Zweisprachige Texte, kleine Vorlagensymbole, Entwürfe und Lebenszyklustests ergänzen.
- Die Versionierung in VERSION.json, nativen Metadaten und „Über“ vereinheitlichen, zweisprachige Texte und das Kopieren von Build-Informationen ergänzen. Nachweise für Gesamtprozessleistung und Lizenzen weiter sammeln. Desktop-, Neuinstallations- und Lizenzprüfungen getrennt halten.
- Abmessungen des Qt-Hauptfensters, Teilbereiche, Listen, Texte, Symbole, Tabs und Vorschau an Messungen des eingefrorenen HTML und der Entwürfe ausrichten. Schriftquellen gemeinsam nutzen und unabhängige CI-Messungen und Bildschirmaufnahmen echter ZIP-Archive ergänzen. Tabs schließen und alle Einträge der aktuellen Seite auswählen können. Deaktivierte Zustände und unbekannte Datumsangaben erhalten.
- Gemeinsame Hauptfenstermenüs, Hinweise, Schaltflächenzustände und Pfadnavigation fertigstellen. HTML erhält Kontext-, Dropdown- und Untermenüs, Beispiel-Tabs, Raster, Sortieren, Auswahl, Touch-Bedienung und Demo-Prüfaufgaben. Qt erhält echte schreibgeschützte Auswahl-, Kopier-, Ansichts- und Verzeichnismenüs. 22 neue Interaktionen, 20 bestehende Abläufe und fünf lokale Qt-Gruppen prüfen. Nicht verfügbare Aktionen deaktiviert und Demos gekennzeichnet lassen.
- In HTML und Qt Verbindungslinien für Verzeichnisbäume, feste Zähler, Anzeigen eingeklappter übergeordneter Ordner und gemeinsame Symbolplätze ergänzen. Frühere Hauptentwürfe beibehalten. Grenzhinweise, Ladeplatzhalter und manuelle Barrierefreiheit separat verfolgen.
- Drei Entwurfstafeln für den Verzeichnisbaum, 26 Ausschnitte in Originalauflösung, helle und dunkle Markierungen, Layout- und Interaktionsvorgaben sowie eine einzelne HTML-Datei und Ressourcen liefern. Die HTML- und Qt-Integration stand noch aus. Originalentwürfe und Fotos beibehalten.
- Detailgetreue Entwürfe nach design/hifi verschieben und Übersichten über Haupt- und Aktionsfenster, frühere Vorschläge und Hashes ergänzen. Ressourcenquellen und Links abstimmen. Der zugehörige dunkle Hauptentwurf stand noch aus.
- Einen begrenzten Verzeichnisbaum mit impliziten Ordnern, Sortierung aller Treffer und per Tastatur bedienbaren Spaltenköpfen hinzufügen. Nach dem Sortieren in Seiten aufteilen und Kennungen doppelter Datensätze erhalten. Knoten-, Tiefen- und Bytegrenzen, Ersatzhinweise, Interaktionstests und 54 visuelle Prüfungen ergänzen.
- Die Laufzeitgrundlage mit zwei cxx-Handles, begrenzten Befehlen und Ereignissen, unabhängiger Abbruch- und Beendigungssteuerung, Qt-Mailbox und Abschlussbarrieren ergänzen. Produkt-CI auf drei Plattformen, zehn Lebenszyklen je Plattform und nativen Start und Abschluss nach Bereitstellung auf macOS und Windows prüfen. Einen Apple-Silicon-Entwicklungsbuild und eine Anleitung für die einmalige Prüfung bereitstellen.
- Kurze Konflikte an der Aufgabenannahmesperre innerhalb fester Grenzen asynchron erneut versuchen. Befehle bei unterbrochener Verbindung deaktivieren, ungültige Argumente melden und die Zerstörung während der Ereignisverarbeitung absichern.
- Eine begrenzte Entpackvorprüfung mit ausschließlich Metadaten ergänzen. Sie löst Verzeichnisse auf, prüft Pfade, Konflikte und Größen und zeigt das Ergebnis in einem nativen Fenster. Die Berechtigung zur Ausgabe bleibt bei der Entpackoperation.
- Den dreigeteilten Archivbrowser mit hellen und dunklen Themen, Details, Verzeichnisbereich, Suche und Mehrfachauswahl nach physischer ID verbessern. Maus- und Filtertests ergänzen.
- Echte schreibgeschützte ZIP-Navigation mit unabhängigem Rust-Adapter, begrenztem Scan, doppelten und chinesischen Namen, Seiten zu 128 Datensätzen und asynchronem Abbruch und Schließen ergänzen. Die Aufnahme von rawzip und Qt-Bildschirmaufnahmen dokumentieren.
- Qt-Aufgabenfortschrittsfenster nach den Entwürfen um Pfadkarten, Kennzahlen in drei Spalten, einen Verlaufsbalken und einen gemeinsamen Fußbereich ergänzen. Umbruch bei großer Schrift und Demo-Kennzeichnung beibehalten. Helle, dunkle und 200-%-Textansichten erfassen.
- Invarianten für Domänenidentität und Seiteneinteilung festlegen. Eigenständige Qt-Aufgabenfenstervorschauen, gemeinsame Hinweise und erzeugte Themenparameter ergänzen. Layout-, Schließ- und Offscreen-CI-Prüfungen hinzufügen.
- Ein eigenständiges Diagnoseprogramm für die Rust/cxx-Brücke, gemeinsame CMake- und Cargo-Builds, festgelegte Abhängigkeiten und native CI hinzufügen. Unter Linux Besitz, Fehlerbehandlung, UTF-8 und das Beenden prüfen.
- Ein eigenständiges Diagnoseprogramm für die Qt-Quick-Toolchain ergänzen und Linux-Offscreen-Tests und Builds in Pfaden mit chinesischen Zeichen und Leerzeichen belegen. Prüfungen für drei Plattformen und cxx standen noch aus.
- Acht detailgetreue Entwurfstafeln und Ressourcen für Erstellung, Konflikte, Fortschritt, Ergebnisse, Einstellungen und Plattformvergleiche ergänzen. Abläufe zum Speichern, Abbrechen und Wiederherstellen festlegen.
- 24 Ausschnitte in Originalauflösung, 27 helle und dunkle Vektorsymboltypen sowie transparente PNGs in verschiedenen Auflösungen mit Angaben zu Quelle und Genauigkeit liefern.
- Dateitypsymbole, Listengeometrie und die Typspalte abstimmen. Eigenständige Fenster, Titelleisten und Schließverhalten für alle drei Plattformen festlegen.
- Einen Offline-HTML-Interaktionsprototyp für Durchsuchen, Auswahl, Entpacken, Erstellen, Aufgaben und Fehlerzustände hinzufügen und mit Anforderungen und Prüfungen verknüpfen.
- Apache-2.0 übernehmen, den vollständigen Lizenztext hinzufügen und Beitrags- und Release-Dokumente aktualisieren.

<a id="pre-development-baseline-2026-10-11"></a>

### Ausgangsstand vor der Entwicklung (2026-10-11)

- Softwareentwurf, Nachverfolgbarkeit der Anforderungen und Architekturentscheidungen festhalten.
- Qt Quick mit einer schlanken C++-Schicht und Rust auswählen.
- Gestaltungsgrundlagen und Themenparameter für helle, dunkle und Entpackansichten definieren.
- Dokumentation für Beiträge, Projektführung, Sicherheit, Entwicklung, Abnahme, Risiken und Arbeitspakete ergänzen.
- Produktcode und Binärveröffentlichungen standen noch aus.
