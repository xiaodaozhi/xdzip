<a name="xdzip-product-version-and-build-identity"></a>

# Produktversion und Build-Identität

[English](../../engineering/versioning.md) | [简体中文](../../zh-CN/engineering/versioning.md) | [繁體中文（台灣）](../../zh-TW/engineering/versioning.md) | [日本語](../../ja-JP/engineering/versioning.md) | [한국어](../../ko-KR/engineering/versioning.md) | [Français](../../fr-FR/engineering/versioning.md) | Deutsch | [Español](../../es-ES/engineering/versioning.md) | [Português (Brasil)](../../pt-BR/engineering/versioning.md) | [Русский](../../ru-RU/engineering/versioning.md)

Das optionale Feld `prerelease_number` trennt die Vorabversionsnummer von der Buildnummer. So kann alpha.1 Build 14 verwenden, während die Buildnummer bei alpha.2 weiter erhöht wird. Ohne Feld gilt die bisherige Auslegung `channel.build`. Das [Veröffentlichungsprotokoll](release-protocol.md) legt Manifeste und Dateinamen fest.

[VERSION.json](../../../VERSION.json) ist die einzige gepflegte Quelle für Version, Kanal und Buildnummer. Der Build liest die aktuellen Werte. Versionsänderungen stehen in der [Änderungshistorie](../../../CHANGELOG.md) und Lieferungen und Prüfungen in den [Projekt-Issues](https://github.com/xiaodaozhi/xdzip/issues). Eine höhere Versionsnummer genehmigt keine Veröffentlichung.

Kompatibilitätsprüfungen verwenden festgelegte frühere Quellstände zusammen mit dem aktuellen Build, um Upgrades, Rückkehr zur früheren Version und Einstellungsübernahme zu prüfen. Das [Skript für das frühere Produkt](../../../scripts/prepare-previous-product.py) pflegt den vollständigen Produktstand, das [Kompatibilitätsskript für Einstellungen](../../../scripts/check-settings-compat.py) den Einstellungsstand. Wechselnde Commits und Versionswerte werden hier nicht kopiert. Einstellungsschema und Produktversion werden getrennt verwaltet.

- `version` ist `major.minor.patch` ohne führende Nullen. Inkompatible Änderungen, kompatible Funktionen und Korrekturen nutzen die jeweilige Komponente. Kompatibilitätsgrenzen werden auch bei 0.x erläutert.
- `channel` unterstützt development, alpha, beta, rc und stable. Vorabversionen erscheinen als `version-channel-abbreviation.build`, development als dev. Bei stable erscheint die Basisversion.
- `build` ist eine Ganzzahl von 1 bis 65535. Erhöhen Sie sie für einen neuen Build zur externen Auslieferung. Lokale Kompilierungen erhöhen sie nicht automatisch. Die Änderung erzeugt weder automatisch einen Tag noch eine Veröffentlichung. Für Windows-Ressourcen ist jede numerische Versionskomponente ebenfalls höchstens 65535.
- CMake liest JSON direkt. `python3 scripts/sync-version.py` synchronisiert die abgeleiteten Cargo-Workspace- und HTML-Versionen. CI nutzt `--check`. Interne Rust-Pakete übernehmen die Workspace-Version. Aktualisieren Sie anschließend Cargo.lock und führen Sie `cargo check --workspace --locked` aus.
- Die vierteilige numerische Windows-Ressource enthält Basisversion und Build. Der Versionstext behält den Kanal. Unter macOS enthält `CFBundleShortVersionString` die Basisversion und `CFBundleVersion` die Buildnummer, die Langversion behält den Kanal. Beachten Sie die [Apple-Dokumentation](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleversion). Verwenden Sie das erlaubte Zahlenformat und keine Git-Commits in numerischen Systemfeldern.
- Windows verwendet `xdzip Archive Manager` als `FileDescription`, etwa im Task-Manager. `ProductName` bleibt `xdzip`. EXE-Dateiname und Zuordnungsidentität hängen nicht vom Anzeigenamen ab. Buildprüfungen lesen die tatsächlichen EXE-Ressourcen.
- Jeder Desktopbuild erzeugt den kurzen Git-Commit und einen Marker für nicht eingecheckte Änderungen. Ohne Git gilt der Commit als unbekannt. Rechnerpfade, Benutzernamen und instabile Zeitstempel fehlen. Der Header wird nur bei geändertem Inhalt neu geschrieben. Nach dem Commit erzeugt ein erneuter Build die korrekte saubere Commit-Identität.

Die Versionen von QML-Modul, Brückenprotokoll und Einstellungsschema sind unabhängig von der Produktversion.

`xdzip --version`, Einstellungen → Über xdzip und die Vorschauanwendung lesen dieselben Buildinformationen. Die CLI verarbeitet Versionsargumente vor dem Anlegen der GUI. Über xdzip zeigt Version, Build, Commit, tatsächlich verwendetes Qt, System und Architektur. Entwicklungskanäle und nicht eingecheckte Änderungen werden ausdrücklich angezeigt. Kopiertes JSON enthält keine persönlichen Dateien, Pfade, Rechnernamen oder Zugangsdaten. Die Projektschaltfläche öffnet den Browser nur bei Bedienung.

Das große Info-Symbol verwendet das plattformspezifische PNG mit 512 px. HTML verwendet den Vektor derselben Quelle und vergrößert kein 72-px-Oberflächensymbol.

Die Info-Seite übernimmt die logischen Einstellungsparameter ohne Fußzeile. Sie unterstützt die [mitgelieferten Oberflächensprachen](../ux/settings-localization.md), helle und dunkle Darstellung sowie Textgrößen von 100%, 125% und 150%. Siehe [Info-Entwurf](../../../design/hifi/sources/settings.html?tab=about). Die HTML-Informationen stammen aus demselben JSON. Nicht verfügbare native Qt- und Commitwerte bleiben unbekannt.

Prüfeinstiege sind `scripts/check-build-version.py` für CLI und native Versionsressourcen, `scripts/test-version.py`, `scripts/check-html-about.py` und `BrowserTest::versionAboutWindow`. Screenshots und Messungen bleiben unter `build/review/about/`. Quellen, Systemmetadaten und laufende Version müssen übereinstimmen. Eine Versionsnummer belegt weder Signierung noch manuelle Abnahme oder Kompatibilität.
