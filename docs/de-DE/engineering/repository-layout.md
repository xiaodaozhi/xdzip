<a name="xdzip-repository-layout-and-dependencies"></a>

# Repository-Struktur und Abhängigkeiten

[English](../../engineering/repository-layout.md) | [简体中文](../../zh-CN/engineering/repository-layout.md) | [繁體中文](../../zh-TW/engineering/repository-layout.md) | [日本語](../../ja-JP/engineering/repository-layout.md) | [한국어](../../ko-KR/engineering/repository-layout.md) | [Français](../../fr-FR/engineering/repository-layout.md) | Deutsch | [Español](../../es-ES/engineering/repository-layout.md) | [Português (Brasil)](../../pt-BR/engineering/repository-layout.md) | [Русский](../../ru-RU/engineering/repository-layout.md)

Cargo.toml und CMakeLists.txt im Stammverzeichnis definieren die aktuellen Produktmodule. Die Tabelle beschreibt vorhandene Verzeichnisse. Vorgesehene Aufteilungen in Architekturentwürfen sind noch keine eigenständigen Crates.

| Verzeichnis | Zuständigkeit und Abhängigkeiten |
| --- | --- |
| apps/desktop | Qt/QML, schlanke C++-Controller, Modelle, Einstellungen und Systemintegration. Zugriff auf den Kern über die Brücke. |
| crates/xdzip-cli | Argumente, Terminal, JSON, verwaltete Installation und Update-Host ohne Qt. Direkte Abhängigkeiten zur Laufzeit, sicheren E/A und zum Update-Kern. Die Formatauswertung ist ausgelagert. |
| crates/xdzip-domain | Reine Typen und Invarianten. Keine Abhängigkeit von Qt oder konkreten Codecs. |
| crates/xdzip-codec-api | Verträge für Formatfähigkeiten, geliehene Eingaben, kodierte Ausgabe und Abbruch. Hängt von der Domäne ab. |
| crates/xdzip-compression | Gemeinsame Algorithmen und Rahmen für Deflate/Deflate64/GZIP, LZMA/LZMA2, BZip2, PPMd7, MSZIP, LZX und Quantum. Keine Produktionsabhängigkeiten von Drittanbietern und keine Berechtigung für Pfade oder Veröffentlichung. Siehe [Vertrag](independent-rust-codecs.md). |
| crates/xdzip-crypto | Formatunabhängige SHA/HMAC- und BLAKE-Hashes mit festem Speicherbedarf und löschbarem vertraulichem Zustand. Nutzt zugelassene Primitive. Archivpasswörter und Veröffentlichungsentscheidungen liegen außerhalb des Moduls. |
| crates/xdzip-codec-zip, xdzip-codec-tar | ZIP- und TAR/GZIP-Implementierungen ohne beliebige Ausgabepfade. |
| crates/xdzip-codec-lha, xdzip-codec-arj, xdzip-codec-arc, xdzip-codec-bh | Eigenständige Rahmen- und Eintragsprofile für [LHA](lha-containers.md)/[PMA](pma-streams.md), [ARJ](arj-streams.md), [ARC](arc-streams.md) und [BH](bh-containers.md) ohne Dateisystemberechtigung. Das Formatregister bestimmt die Verfügbarkeit im Produkt. |
| crates/xdzip-codec-7z | Eigener 7z-Container, Methodengraphen, Filter, AES-Kombination, Streaming-Erstellung und begrenztes Umschreiben ohne Zuständigkeit für Dateipfade. |
| crates/xdzip-codec-cab | Eigener schreibgeschützter CAB-Container mit vom Host autorisierten Volumes und gemeinsamen Algorithmen. Erhält Rohnamen, UTF-8-Deklarationen und gewöhnliche Namensduplikate. Volumesuche und Veröffentlichung liegen beim Host. Siehe [Vertrag](independent-rust-codecs.md#cab只读规格) und [Quellen](cab-format-sources.md). |
| crates/xdzip-codec-optical | Leser für ISO9660/Joliet/Rock Ridge/UDF und ISO9660/Joliet-Schreiber für neue Datenabbilder. Arbeitet mit Datenströmen ohne Zuständigkeit für Pfade. |
| crates/xdzip-codec-bridge | Schlanke CXX-Grenze zwischen isoliertem Prozess und den Rust-Codecs für 7z, RAR, CAB und optische Medien. Autorisierte Datenströme, Ressourcenlimits und Abbruch. |
| crates/xdzip-codec-native, tools/native-codec | Isolierter Prozess für Rust-Backends für 7z/RAR/optische Medien/CAB. Das schlanke C++ liefert nur Protokoll, Kalender- und Kodierungsdienste des Systems. Der native Empfänger prüft vorab angerechnete Ausgabe und Zusatzarbeit selbstständig. Siehe [E/A-Vertrag](native-codecs.md). |
| crates/xdzip-extract-plan | Metadaten-Vorprüfung und begrenzte Berichte ohne Zielberechtigung oder Schreibrechte. |
| crates/xdzip-io | Eingabebäume, native Berechtigungen und Identitäten, sichere Zwischenablage und Übernahme von Dateien. |
| crates/xdzip-runtime | Sitzungen, begrenzte Indizes im Speicher oder auf Datenträger, Abfragen, Auswahl, begrenzte Vorschauen, Erstellungs- und Entpackaufgaben und Planung. |
| crates/xdzip-qt-bridge | CXX-Typbrücke, zwei Handles und Lebenszyklus. Hängt von der Laufzeit ab. |
| crates/xdzip-plugin-protocol | Begrenztes CBOR und Nachrichtenrahmen nur mit der Standardbibliothek, ohne Zuständigkeit für Pfade, Prozesse oder Qt. |
| crates/xdzip-plugin-host, examples/sdk | Eigenständiger schreibgeschützter SDK-Versuch, kontrollierte E/A, native Sandbox-Voraussetzungen und unabhängiges Python-Gegenstück. Gehört weder zur Formaterkennung noch zu den Laufzeitabhängigkeiten des Produkts. |
| tools | Eigenständige Prüfprogramme für Brücke, Formate, E/A und Prozesszahlen. Die grafische Oberfläche hat eigene Einstiegspunkte. |
| scripts, .github | Generierung, Prüfungen, Bereitstellungswerkzeuge, CI und Vorlagen für die Zusammenarbeit. |
| release-notes | Prüfbare Versionshinweise und Versions- und Build-Metadaten. Laufberichte und erzeugte Pakete liegen separat. |
| design/hifi, design/assets | Originalentwürfe und Eingaberessourcen für die Laufzeit. |
| docs | Dauerhafte Markdown-Entwürfe, technische Verträge, wiederverwendbare Prüfmethoden und erforderliche Bilder. Laufberichte liegen separat. |
| specs | Maschinenlesbare Inventare für Gestaltung, Terminologie und Abhängigkeiten. Sie dienen Produktressourcen, Generatoren und Prüfungen. |
| third_party | Quellen und Lizenzmaterialien Dritter, die mit dem Code aufbewahrt werden. |

Eigenständige Module xdzip-index und xdzip-platform sowie vollständige Prüfwerkzeuge für externe Formate gehören zur künftigen Architektur. Der Index liegt derzeit in der Laufzeit. Plattformspezifische Dateioperationen liegen in io und den Desktopadaptern. Der vorhandene plugin-host dient einem [schreibgeschützten SDK-Versuch](sdk-trial.md). Tests liegen hauptsächlich bei Rust-Modulen, unter apps/desktop/tests und bei den Skriptprüfungen. Leere Modulhüllen werden nicht angelegt.

Das Produkt verwendet für CAB fest das Rust-Backend. Offizielle Cab-Handler und CAB-spezifische LZX/Quantum-Varianten werden nicht kompiliert. Die Aufrufe bleiben im selben isolierten Prozess, ohne zusätzliche Produktprogramme, Testoptionen oder Ausweichpfad bei Fehlern. codec-api legt die Vorabverbuchung für vollständige Frames fest. Container halten Guthaben pro Frame. Native Empfänger und Laufzeit prüfen das jeweilige Guthaben und die Ausgabelimits unabhängig. Siehe [internen Vertrag](independent-rust-codecs.md#cab只读规格).

Die [Systemarchitektur](../design/system-architecture.md) beschreibt die Abhängigkeiten. Vermeiden Sie Rückabhängigkeiten der Domäne zur Laufzeit, direkte Meldungen von Codecs an QML und UI-Texte aus Plattformschichten. Generierte Bindings, Beispiele, Berichte und Build-Ausgaben gehören in build/, target/ oder dist/ außerhalb der Quellverzeichnisse. [Installation und Zuordnungen](installers-associations.md#兼容基线的长期输入) beschreibt die langfristigen Eingaben für historische Kompatibilität.

`tools/image-preview/` ist ein interner QtGui/QtSvg-Prozess für statische Bilder. Er empfängt geprüfte, begrenzte Inhalte und liefert begrenzte RGBA-Daten. Er speichert keine Archiv- oder Zielpfade. Plattformen und Fehlerbehandlung stehen im [Vorschauvertrag](content-preview.md).
