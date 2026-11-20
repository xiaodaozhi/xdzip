# xdzip: Architekturentscheidungen

[English](../../design/architecture-decisions.md) | [简体中文](../../zh-CN/design/architecture-decisions.md) | [繁體中文](../../zh-TW/design/architecture-decisions.md) | [日本語](../../ja-JP/design/architecture-decisions.md) | [한국어](../../ko-KR/design/architecture-decisions.md) | [Français](../../fr-FR/design/architecture-decisions.md) | Deutsch | [Español](../../es-ES/design/architecture-decisions.md) | [Português (Brasil)](../../pt-BR/design/architecture-decisions.md) | [Русский](../../ru-RU/design/architecture-decisions.md)

Die frühen Aufzeichnungen stammen vom 11. Oktober 2026. Spätere Entscheidungen nennen ihr eigenes Datum. Eine akzeptierte Entscheidung ist angenommen. Die Hinweise zur aktuellen Implementierung verknüpfen jede Entscheidung mit den gepflegten Verträgen und den Nachweisen ihrer Fähigkeiten. Wird eine Entscheidung ersetzt, bleibt die ursprüngliche Aufzeichnung erhalten und eine neue wird ergänzt.

## ADR-001: Qt Quick als einzige Produktoberfläche

**Status: akzeptiert.**

**Aktuelle Implementierung.** Der Desktop verwendet Qt Quick/QML mit schlanken C++-Adaptern. Der Windows-SFX-Host mit Systemdialogen ist die begrenzte Ausnahme aus ADR-013. [Desktopoberfläche](qt-quick-ui.md)

Kontext: Das Produkt benötigt eine moderne Desktopoberfläche, ein konsistentes Nutzungserlebnis und geringen Ressourcenverbrauch. Prototypen mit Slint, Tauri und Qt Quick auf demselben Kern wurden erstellt. Unter Berücksichtigung der Oberflächenfunktionen, der Konsistenz und vorhandener Messungen zur Softwaredarstellung unter Linux wurde Qt Quick für die Produktphase gewählt; die übrigen Plattformen müssen noch überprüft werden.

Entscheidung: Qt Quick/QML und Qt Quick Controls 2 mit schlanker C++-Anpassung. Slint und Tauri bleiben historische Experimente und werden nicht mit dem Produkt ausgeliefert.

Kosten: Qt-Laufzeit, QML-Module, sprachübergreifende Builds und Lizenzarbeit. Eingabe, Barrierefreiheit, GPU und Bereitstellung auf Mac und Windows erfordern weitere Tests. Die 0,80 MiB große ausführbare Prototypdatei ist kein vollständiges Qt-Installationspaket.

Anlass zur Neubewertung: Qt erfüllt die erforderlichen Funktionen, Lizenz- oder Sicherheitswartungsanforderungen auf den Zielplattformen nicht und verfehlt auch nach tatsächlicher Optimierung die wesentlichen Produktbudgets. Gewöhnliche Leistungsschwankungen lösen keine erneute Frameworkauswahl aus.

## ADR-002: Rust verantwortet die gesamte Archivlogik

**Status: akzeptiert.**

**Aktuelle Implementierung.** Die Rust-Laufzeit verwaltet Archivzustand und Aufgaben. Die Dateiauswahl drückt eine Absicht der Oberfläche aus; die E/A-Schicht des Hosts autorisiert den Zugriff auf das Dateisystem. [Systemarchitektur](system-architecture.md)

Entscheidung: Domäne, Index, Pläne, Aufgaben, Kontingente, Pfadrichtlinien und Formatregistrierung liegen in Rust. Qt stellt den Zustand dar, autorisiert die Systemdateiauswahl und übermittelt Entscheidungen der Nutzer. Fachliche Tests benötigen kein Qt.

Alternativen: libarchive oder 7zip unmittelbar mit C++/QML verbinden oder die Steuerlogik in jeder Oberfläche wiederholen. Das koppelt den Kern an plattformspezifische Oberflächen und verhindert eine einheitliche Prüfung von Sicherheit und Abbruch. Rust beseitigt Sicherheitsrisiken nativer Bibliotheken nicht; betroffene Bibliotheken können in Arbeitsprozessen isoliert werden.

Anlass zur Neubewertung: Kann eine Plattform-API nicht zuverlässig in Rust implementiert werden, ist eine kleine plattformspezifische C++-Portierung zulässig, ohne die Verantwortung für die Fachlogik zu ändern.

## ADR-003: cxx-Brücke zur Buildzeit

**Status: akzeptiert.**

**Aktuelle Implementierung.** Das Produkt baut und verwendet die typisierte Brücke mit getrennten Handles für Befehle und Ereignisse. Ereignisempfänger und Lebenszyklus beim Herunterfahren sind im [Brückenvertrag](qt-rust-bridge.md) definiert.

Entscheidung: Innerhalb des Produkts eine von cxx erzeugte typsichere Brücke verwenden, die aus demselben Commit gebaut wird. Keine langfristige handgeschriebene C-ABI pflegen und auf kritischen Pfaden kein JSON serialisieren. Die Brücke enthält keine Qt-Fachlogik.

Alternativen: die experimentelle C-ABI beibehalten, QObject mit CXX-Qt erzeugen oder Oberfläche und gesamten Kern über lokales IPC verbinden. cxx erhält explizite Grenzen für Qt-Modelle und Threads, ohne zunächst ein Framework zur QObject-Generierung einzuführen. Dafür müssen die schlanke QObject-Schicht und die CMake/Cargo-Orchestrierung gepflegt werden.

Grenzen: cxx bietet keine dynamische Binärkompatibilität zwischen beliebigen Rust- und Qt-Versionen und erzeugt kein Formatprotokoll für Dritte.

Anlass zur Neubewertung: Die Prüfung des unterstützten Desktopbuilds oder Lebenszyklus schlägt fehl oder die Adapterschicht wächst deutlich. CXX-Qt oder eine ausdrücklich versionierte C-ABI dann über eine ADR bewerten statt stillschweigend zu wechseln.

## ADR-004: integriertes Quellcode-SDK und externes Prozessprotokoll trennen

**Status: Entwurfsgrenze akzeptiert. Das externe Protokoll ist experimentell.**

**Aktuelle Implementierung.** Produktformate werden explizit registriert und integriert. Der mitgelieferte Formatarbeitsprozess verwendet ein eigenes Protokoll. Die externe CBOR-Implementierung ist ein eigenständiger Versuch nur zum Lesen. [Formatadapter und Versuch](codec-extension-sdk.md)

Entscheidung: Rust-Quellcodeschnittstellen für integrierte Formate und ein getrenntes, begrenztes Prozessprotokoll für externe Erweiterungen verwenden. Die E/A-Befugnis bleibt beim Host.

Begründung: Die ABI dynamischer Rust-Bibliotheken ist instabil und beliebige native Plugins im Prozess beeinträchtigen Absturzisolierung und Ressourcenverwaltung. Wasm hat als allgemeine Formatlaufzeit wegen der Kosten für Migration, Threads, positionierten Zugriff und Leistungsprüfung vorhandener nativer Codecs derzeit keine Priorität; eine spätere Verwendung ist nicht ausgeschlossen.

Kosten: Protokoll, Vermittler, Sandbox und IPC-Overhead. Das Protokoll erst nach erfolgreicher Interoperabilität zweier unabhängiger Implementierungen einfrieren.

## ADR-005: Der Host kontrolliert tatsächliche Dateischreibvorgänge zentral

**Status: akzeptiert.**

**Aktuelle Implementierung.** Die Laufzeit bereitet Vorgänge vor und `xdzip-io` verwaltet Berechtigungen, temporäre Ausgabe und Übernahme. Formatadapter arbeiten mit Lesern und Ausgabesenken. [Dateiautorisierung](file-security.md)

Entscheidung: Backends schreiben nur in freigegebene `OutputSink`-Ziele. Der Host kontrolliert tatsächliche Pfade, Konflikte, temporäre Ausgabe und Veröffentlichung. Eine allgemeine Entpackfunktion einer Bibliothek nicht direkt in beliebige Verzeichnisse aufrufen, sofern deren Anpassungen nicht vollständig als vom Host beschränkt geprüft wurden.

Begründung: Separate Pfadschutzimplementierungen je Format führen leicht zu Unterschieden; Zeichenkettenprüfungen allein verhindern weder Links noch Race Conditions. Dafür müssen sichere Dateischichten auf drei Plattformen implementiert und auf Fehler getestet werden.

Grenzen: Der Host kann weder die semantische Richtigkeit der Ausgabe eines bösartigen Plugins beweisen noch einen bereits denselben Nutzer kontrollierenden bösartigen Prozess stoppen. Vertrauen und Berechtigungen von Plugins müssen ausdrücklich bleiben.

## ADR-006: Extraktion je Datei und neue Archive vollständig veröffentlichen

**Status: akzeptiert.**

**Aktuelle Implementierung.** Die Extraktion veröffentlicht Dateien einzeln. Erstellung und Änderung prüfen vorbereitete Archive vor Ersetzung oder Veröffentlichung. Die Erstellung von Volumes veröffentlicht einen geprüften Satz Teil für Teil und versucht bei Teilfehlern eine begrenzte Rücknahme. [Veröffentlichungssemantik](file-security.md)

Entscheidung: Jede extrahierte Datei wird vor der Veröffentlichung vorbereitet und geprüft; fehlgeschlagene Aufgaben können Teilergebnisse behalten. Neue Archive werden vor der Veröffentlichung vollständig geschrieben. Standardmäßig wird nicht überschrieben.

Begründung: Die Zusammenführung in ein bestehendes Verzeichnis bietet plattformübergreifend nicht ohne Weiteres eine wirklich atomare Transaktion für das gesamte Verzeichnis. Eine vorgetäuschte Rücknahme der gesamten Aufgabe könnte spätere Änderungen der Nutzer löschen. Erforderlich sind detaillierte Ergebnisse und Wiederherstellungsprotokolle sowie eine klare Erklärung des Abbruchs.

Anlass zur Neubewertung: Ein neuer Modus zur Veröffentlichung eines vollständig neuen Verzeichnisses kann unter begrenzten Bedingungen separat implementiert werden, ohne die Semantik der Zusammenführung in bestehende Verzeichnisse zu ändern.

## ADR-007: Begrenzte Akteure und Arbeitspools mit gepushten Ereignissen

**Status: akzeptiert.**

**Aktuelle Implementierung.** Pro Verbindung werden getrennte Threads für Befehle und Aufgaben verwendet. Der Desktop lässt schwere Aufgaben seriell zu; Rust-Warteschlangen und Qt-Postfach sind begrenzt. Ein allgemeiner Ressourcen-Tokenpool bleibt eine Entwurfsoption. [Aufgabenplanung](task-scheduling.md), [Qt/Rust-Brücke](qt-rust-bridge.md)

Entscheidung: Ein schlanker Planer entscheidet über den Zustand, begrenzte Pools führen Arbeit aus und Qt besitzt einen eigenen Ereignisempfänger-Thread. Fortschritt zusammenfassen, Endzustände zustellen und für Abbruch und Herunterfahren eine unabhängige Steuerungszulassung vorsehen.

Alternativen: synchrone GUI-Aufrufe, ein Thread je Datei, unbegrenztes Tokio oder `spawn_blocking` oder ständiges Polling. Archivarbeit ist überwiegend synchrone CPU- und E/A-Arbeit; nicht allein wegen der Bezeichnung „asynchron“ unbegrenzt Aufgaben erzeugen. Künftige tatsächliche Netzwerk-E/A kann lokal eine asynchrone Laufzeit verwenden.

Kosten: ausdrückliche Rückstaukontrolle, Abschaltung und Ressourcenzuweisung. Vorteile: geringer Aufwand im Leerlauf und eine Oberfläche, die nicht von Aufgaben überflutet wird.

## ADR-008: Fähigkeitsabhängige, begrenzte Formate in der ersten Version

**Status: akzeptiert.**

**Aktuelle Implementierung.** Die ursprünglichen Formate sind historischer Kontext. Gepflegtes Register und Laufzeitzulassung definieren die aktuellen Fähigkeiten des Quellcodes, einschließlich erweiterter Lese- und Erstellungsprofile und operationsspezifischer Einschränkungen. [Formatregister](../engineering/format-registry.md)

Entscheidung: v0.1 verwendet ZIP Store/Deflate und TAR/GZIP; Verschlüsselung, Volumes, RAR, 7z und weitere Algorithmen werden einzeln geprüft. Die Oberfläche leitet vollständige Unterstützung nicht aus Dateiendungen ab.

Begründung: Ein Formatname beschreibt nicht alle tatsächlichen Funktionen. Die anfängliche Formattabelle ist kürzer, für Nutzer aber ehrlich und überprüfbar.

## ADR-009: Kleine Produktdaten von Nutzerinhalten trennen

**Status: akzeptiert.**

**Aktuelle Implementierung.** Einstellungen, Indizes und Journale haben getrennte Lebensdauern. Datenträgergestützte Indizes sind privat und auf die Sitzung beschränkt. Dauerhafte Indexwiederverwendung und Wiederaufnahme unvollendeter Aufträge bleiben Entwurfsoptionen. [Domänenmodell](domain-data.md), [Dateiveröffentlichung](file-security.md)

Entscheidung: Einstellungen, temporäre Indizes, Transaktionsprotokolle und Erweiterungsmanifeste haben getrennte Lebenszyklen. Standardmäßig keine Telemetrie, keine dauerhaften Passwörter und kein Archivindex-Cache über Programmstarts hinweg.

Kosten: mehrere Schemata und Bereinigungsregeln. Eine erneute Bewertung startübergreifender Indizes erfordert ein neues Konzept für Quellenidentität, Invalidierung, Datenschutz und Kapazität.

## ADR-010: Vollständige Distribution und Lizenzen als Architekturhürden

**Status: akzeptiert.**

**Aktuelle Implementierung.** GitHub-Binärdateien und Microsoft-Store-MSIX folgen dokumentierten, getrennten Paketierungswegen. Tatsächlich bereitgestellte Abhängigkeiten, Artefaktintegrität und Lizenzmaterialien werden je Kanal geprüft. [Build und Veröffentlichung](build-release.md)

Entscheidung: Qt standardmäßig dynamisch bereitstellen und die Lizenzen der tatsächlich verwendeten Module prüfen. Die Installationsgröße umfasst Laufzeitumgebungen. Signatur, SBOM und Pflichten zu Austausch oder Quellcode gehören zu den Veröffentlichungsprüfungen.

Alternativen: nur ausführbare Dateien vergleichen oder `LICENSE` erst kurz vor der Veröffentlichung ergänzen. Beides belegt weder geringe Produktgröße noch rechtmäßige Verteilbarkeit.

Kosten: fortlaufende Wartung von Bereitstellung und Lizenzen. Statische Qt-Builds oder Store-Distribution erfordern eine separate ADR und Lizenzprüfung; eine PR zur Größenoptimierung darf sie nicht beiläufig ändern.

<a name="xdzip-adr-011-initial-support-scope-and-personal-project-maintenance-policy"></a>

## ADR-011: Ursprünglicher Unterstützungsumfang und Wartung eines persönlichen Projekts

**Status: am 19. Oktober 2026 mit ausdrücklicher Zustimmung des Projekteigentümers akzeptiert.**

**Aktuelle Implementierung.** Für aktuelle Unterstützung und Updates gelten die [Unterstützungsrichtlinie](../engineering/support-policy.md) und der [Updatevertrag](../../engineering/software-update.md). Die Klausel zum manuellen Update beschreibt v0.1.

Entscheidung: Zunächst macOS 14 und neuer auf Apple Silicon arm64, gewartete Versionen von Windows 11 x86_64 sowie Ubuntu 24.04 LTS x86_64 mit X11 und glibc 2.39 oder neuer unterstützen. Nur die neueste öffentliche Version priorisieren, Rückmeldungen nach Möglichkeit ohne feste Antwortzeit bearbeiten, für v0.1 manuelle Updates verwenden und Entwicklungsvorschauen experimentell halten. Siehe [Unterstützungsrichtlinie](../engineering/support-policy.md).

Begründung: Vorhandene native Builds, Installer und Laufzeitabhängigkeiten begrenzen den Umfang und reduzieren ungeprüfte System- und Architekturzusagen früherer Entwürfe auf die Wartungskapazität eines persönlichen Open-Source-Projekts. Intel-Mac, Windows 10, Windows/Linux ARM64, andere Distributionen und Wayland werden nicht zugesagt. Eine Erweiterung erfordert zunächst Implementierung und tatsächliche Prüfung.

Grenze: Persönliche Open-Source-Distribution über GitHub erfordert keine kostenpflichtigen Entwicklerkonten, Signaturzertifikate, Developer ID oder Notarisierung. macOS behält die nötige Ad-hoc-Integritätssignatur. Diese Richtlinie ersetzt weder die Prüfung der Lizenzherkunft und Pakete noch garantiert sie Kompatibilität mit allen zukünftigen Systemversionen.

## ADR-012: Jedes Format auf unabhängige Rust-Format- und Kompressionsalgorithmen migrieren

**Status: am 25. Oktober 2026 akzeptiert und vom Eigentümer ausdrücklich zur Implementierung freigegeben. Ersetzt die langfristige Wahl fremder Format-Engines; die Migration ist damit nicht abgeschlossen.**

**Aktuelle Implementierung.** Das Produktrouting verwendet Rust-Formatimplementierungen. Die betreffenden Lesevorgänge für 7z, RAR, CAB und optische Medien verwenden den mitgelieferten Arbeitsprozess. Die aktuelle Operationsmatrix enthält einen ISO-Schreiber im Prozess. Jede Operation im [Formatregister](../engineering/format-registry.md) und im [Adapterentwurf](codec-extension-sdk.md) prüfen.

Entscheidung: Unabhängige Rust-Implementierungen für Formatanalyse sowie Kompressions- und Dekompressionsalgorithmen verwenden; allgemeine Bibliotheken für Kryptografie, Zeichenkodierung und Systemgrundlagen sind zulässig. Zuerst Verhaltensspezifikationen aus offiziellen Spezifikationen ableiten und anschließend unabhängig implementieren; Referenzquellcode weder zeilenweise übersetzen noch unter umbenannten Bezeichnern kopieren. Entwickler, die Quellcode gesehen haben, beanspruchen keine strenge Clean-Room-Entwicklung. Unzureichende Quellen und Lizenzbeschränkungen erfordern separate Prüfung, insbesondere wird kein RAR-Encoder entwickelt.

Der Migrationsplan sah vor, ZIP, TAR, TAR.GZ, 7z, RAR, CAB, ISO und UDF entsprechend den damaligen Fähigkeiten einzeln abzunehmen und zu ersetzen. Für noch nicht abgenommene Formate die bisherige Implementierung behalten; keinen stillschweigenden Fallback und keine Erweiterung des Nur-Lesen-Umfangs von RAR, CAB, ISO oder UDF. Gemeinsame Kompressionsalgorithmen wiederverwenden. Namen, Identitäten, autorisierte E/A, Ressourcen, Abbruch und Übernahmen bleiben beim Rust-Host; die Isolierung von Arbeitsprozessen entfällt durch Rust nicht.

Kosten: Kompatibilität, Sicherheit, Leistung, Fuzzing und Algorithmen langfristig unabhängig warten. Referenzen Dritter dürfen Tests dienen; migrierte Formate müssen die entsprechenden Engines aus Produktabhängigkeiten und Lieferdateien entfernen. Der [Vertrag für eigene Backends](../../engineering/independent-rust-codecs.md) beschreibt Quellen und Schnittstellenregeln; Fortschritt und tatsächliche Abnahme werden in [#45](https://github.com/xiaodaozhi/xdzip/issues/45) verfolgt.

<a name="xdzip-adr-013-standalone-windows-sfx-host"></a>

## ADR-013: Eigenständiger Windows-SFX-Host

**Status: am 16. November 2026 für das vom Eigentümer genehmigte erste SFX-Profil akzeptiert.**

**Aktuelle Implementierung.** Der [SFX-Vertrag](../engineering/sfx-creation.md) definiert das Windows-Profil und den eigenständigen Rust-Einstieg mit Systemdialogen.

Kontext: Ein erzeugtes selbstextrahierendes Archiv für Windows x64 muss ohne installiertes xdzip oder Qt funktionieren. Eine zweite vollständige Qt-Anwendung in jedem Archiv würde eine zusätzliche, sachfremde Laufzeit mitliefern.

Entscheidung: Der Desktop bleibt bei Qt Quick und verwendet sein Erstellungsfenster weiter. Die eigenständige projekteigene SFX-Datei nutzt ausschließlich Windows-Systemdialoge für Ordner und Aufgaben; Schrift, DPI, Größe und Barrierefreiheit kontrolliert das System. ZIP, Prüfung, Ausgabeautorisierung, Konflikte, Fortschritt, Abbruch und Bereinigung verwenden den vorhandenen Rust-Kern. Dies ist eine begrenzte Ausnahme für den Distributionshost zu ADR-001, kein alternatives Desktopframework und keine neue Bibliothek eigener Steuerelemente.

Kosten: eine zusätzliche ausführbare Windows-Datei, eine versionierte Hülle, statische CRT-Verknüpfung, Abhängigkeitsaudit ohne Qt und nativen Dialoghost warten und automatisch testen. Der Host führt extrahierten Inhalt niemals aus. Die Signatur der erzeugten ausführbaren Datei ist von der Signatur eines festen Stubprogramms getrennt.

Anlass zur Neubewertung: Anforderungen an Installer, automatische Ausführung von Nutzlasten, weitere ausführbare Plattformen oder eine eigene SFX-Oberfläche benötigen einen eigenen Umfang und eine Architekturprüfung.
