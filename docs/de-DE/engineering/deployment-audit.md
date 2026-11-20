<a name="xdzip-deployment-dependency-verification-and-sbom"></a>

# Prüfung von Bereitstellungsabhängigkeiten und SBOM

[English](../../engineering/deployment-audit.md) | [简体中文](../../zh-CN/engineering/deployment-audit.md) | [繁體中文（台灣）](../../zh-TW/engineering/deployment-audit.md) | [日本語](../../ja-JP/engineering/deployment-audit.md) | [한국어](../../ko-KR/engineering/deployment-audit.md) | [Français](../../fr-FR/engineering/deployment-audit.md) | Deutsch | [Español](../../es-ES/engineering/deployment-audit.md) | [Português (Brasil)](../../pt-BR/engineering/deployment-audit.md) | [Русский](../../ru-RU/engineering/deployment-audit.md)

Die Bereitstellungsprüfung erzeugt standardmäßig Entwicklungsprüfmaterialien mit ausdrücklichem Status `incomplete`; sie belegt nicht automatisch Release-Bereitschaft. Ein offizieller Release sammelt Materialien desselben Kandidaten getrennt und liefert sie nach Prüfung des Release-Prozesses mit dem Paket. Das Format nutzt das [offizielle CycloneDX-1.6-Schema](https://github.com/CycloneDX/specification/blob/55343ba19dee1785acf1ce9191540d5fd7b590db/schema/bom-1.6.schema.json). Upstream-Commit und SHA-256 von drei Schemadateien sind in der [Konfiguration](../../../scripts/sbom-schemas.json) festgelegt. Die Validierung lädt und analysiert nur diese Ressourcen und cached sie unter build/; sie lädt keine Schemas von beliebigen URLs in einer SBOM.

| Artefakt | Datenquelle und Zweck | Was es nicht belegen kann |
| --- | --- | --- |
| source-build.cdx.json | Cargo-hostaufgelöster Workspace-Abhängigkeitsgraph einschließlich Entwicklungs- und Build-Werkzeugen; deklarierte Qt-Module; Schrift- und Markenquellhashes | Dass ein Rust-Paket zwingend statisch ins Produkt gelinkt ist oder alle Qt-Plugins und eingebetteten Dritten identifiziert sind |
| deployment-audit.json | Datei-SHA-256 unter der angegebenen tatsächlichen Artefaktwurzel, native Binärimporte, SDK-, System- und unaufgelöste Abhängigkeiten sowie tatsächliche CLI-Build-Identität | Alle dynamisch per Namen geladenen QML-Module oder Plugins, saubere Installation, System-ABI, Signaturen oder Lizenzkonformität |
| artifact-files.cdx.json | Tatsächliche Artefakte und beobachtete externe SDK-Dateien, bekannte Kanten nativer Importe, gebunden an Quellinventar und tatsächliche ausführbare Version, Commit und dirty-Status | Vollständige Identität bereitgestellter Komponenten, Zuordnung statischer Links, Lizenzierung eingebetteter Qt-Bibliotheken oder eine abgeschlossene formelle Produkt-SBOM |
| Optionale Desktopnachweise | Ausgeführte Desktopprüfungen und Grenzen passend zu Änderung und verfügbarer Umgebung | Nicht ausgeführte Aktionen, andere Plattformen oder Release-Freigabe |

Quellnachweise enthalten keine pfadbasierten opaken Cargo-IDs und erfinden keine crates.io-purls für Workspace-Pakete. Artefaktpfade sind relativ zur angegebenen Wurzel oder zum Qt SDK. Andere externe Abhängigkeiten werden durch Inhaltshash und Dateinamen ohne private absolute SDK-Pfade identifiziert. Schriftarten verwenden OFL-1.1; ursprüngliche Markenquellen behalten Apache-2.0 der Assetbibliothek. Deklarierte Qt-Module erlauben keine Ableitung der Lizenzen aller tatsächlich bereitgestellten Komponenten.

<a name="xdzip-automated-verification"></a>

## Automatisierte Prüfung

`deployment_audit.py` prüft nur ausdrücklich angegebene vertrauenswürdige Projektbuilds, keine Benutzerarchive. macOS nutzt otool für Importe und RPATH, Windows MSVC dumpbin für normale und verzögerte DLL-Importe und Linux ldd, das nicht für ausführbare Dateien unbekannter Herkunft verwendet werden darf. Ein Dateisymlink außerhalb der angegebenen Artefaktwurzel lässt die Prüfung fehlschlagen. Berichte müssen außerhalb der Wurzel geschrieben werden, damit ihre Ausgabe nicht im eigenen Inventar erscheint.

- `--mode sdk` erlaubt die Auflösung zu einem externen Qt SDK und zeigt ausdrücklich, dass der Build nicht eigenständig ist. Unaufgelöste Importe schlagen weiterhin fehl. Rekursion umfasst Artefakt und deklariertes Qt SDK; andere externe Bibliotheken werden nur mit Hash und nicht durchlaufener Anzahl erfasst, nicht als abschlussgeprüft betrachtet.
- `--mode deployed` verbietet nichtsystemische externe Abhängigkeiten und unaufgelöste Importe. Windows-VC++-Importe wie MSVCP und VCRUNTIME müssen aus dem Paket stammen; System32 des Entwicklungsrechners darf fehlende Dateien nicht verdecken. System-UCRT und API-Verträge bleiben systembereitgestellt. Native Start-Smoke-Tests mit entfernten Qt-Entwicklungssuchpfaden weiterhin ausführen.
- Unaufgelöste Importe, Abweichungen von Version, Commit oder dirty-Status zwischen Quelle und tatsächlichem Binärprogramm, Hash- oder Formatfehler des offiziellen Schemas sowie doppelte oder ins Leere zeigende Komponentenreferenzen blockieren die jeweilige Prüfung.
- Der Abschluss nativer Importe ersetzt weder die Abdeckung per Namen geladener Plugins noch Prüfungen auf sauberer Maschine. Bestandene Importe aus einem bereitgestellten Windows-Verzeichnis sind keine Installer-Abnahme.

Der bedarfsgesteuerte Qt-Workflow für drei Plattformen speichert zwei cdx.json-Dateien im Wurzelverzeichnis und deployment-audit.json, ohne leere manuelle Formulare automatisch zu erzeugen. macOS liest nur lokal gebaute .app und SDK; gewöhnliche Prüfungen führen macdeployqt nicht aus und signieren, komprimieren oder laden keine App-Pakete hoch. Linux stellt Qt/QML/Plugins über ein [unabhängiges Laufzeitverzeichnis](linux-deployment.md) bereit und nutzt deployed-Prüfung sowie einen SDK-freien Container; heruntergeladenes tar.gz behält ausführbare Bits. Derzeit ist nur Ubuntu 24.04, x86_64, X11 als Basis deklariert, kein universeller Linux-Installer.

Für vertrauenswürdige eigene Windows-Artefakte den Installer getrennt entpacken und sein App-Verzeichnis mit dem portablen ZIP desselben Kandidaten mittels `python scripts/check-windows-package-payload.py --installer-app <extracted-app-directory> --portable <portable-package.zip> --output build/review/payload-comparison.json` vergleichen. Geprüft werden Datei-Hashes, Installationsmanifest und auszuschließende SDK-Grafik-DLLs. Der Ausgabepfad muss neu sein. Dieser statische Vergleich führt weder Installer noch Update aus und ersetzt keine tatsächliche Installationsprüfung.

Lokales macOS-Beispiel nach Neubau aus konsistenten Quellen:

```sh
python3 scripts/collect-notices.py build/review/wp12/notices
python3 scripts/deployment_audit.py \
  --root build/ui/apps/desktop/xdzip.app \
  --app build/ui/apps/desktop/xdzip.app/Contents/MacOS/xdzip \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --mode sdk \
  --output build/review/wp12/deployment-audit.json
python3 scripts/sbom.py \
  --audit build/review/wp12/deployment-audit.json \
  --source build/review/wp12/notices/source-build.cdx.json \
  --output build/review/wp12/artifact-files.cdx.json
```

Nach Installation von jsonschema==4.23.0 in einer isolierten Prüfumgebung deren Python verwenden, um `scripts/validate-sbom.py` mit den beiden obigen cdx.json-Dateien auszuführen. Die [jsonschema-4.23-Dokumentation](https://python-jsonschema.readthedocs.io/en/v4.23.0/referencing/) beschreibt die Registry-Nutzung. Prüfwerkzeuge werden nicht im Produkt bereitgestellt.

<a name="xdzip-attribution-results-and-verification-limits"></a>

## Zuordnungsergebnisse und Prüfgrenzen

[Zuordnung tatsächlicher Komponenten](runtime-attribution.md) ist integriert: SDK-SPDX verbindet tatsächliche Dateien, Qt-Module und eingebettete Dritte; offizielle Lizenztexte mit festgelegten Hashes werden geliefert; SDK/upstream-Hashunterschiede und unbestätigte Zuordnungen werden dokumentiert. Gewöhnliche Rust-Laufzeitkandidaten werden von Build-, Entwicklungs- und Proc-Macro-Werkzeugen getrennt. Neue Komponentenbeziehungen binden den tatsächlichen Audit und werden in die Artefakt-SBOM übernommen, die ausdrücklich unvollständig bleibt.

Im installierten Baum jede ausgelieferte DLL, jedes Framework und Plugin mit upstream-Komponente, Version, Lizenz und korrespondierendem Quellcode verknüpfen. Eingebettete Qt-Komponenten, Zuordnung statischer Rust-Links und Herkunft der Toolchain getrennt prüfen. Für Kandidatenentscheidungen den [Release-Review-Leitfaden](../../../developers/qa/release-review.md) verwenden.
