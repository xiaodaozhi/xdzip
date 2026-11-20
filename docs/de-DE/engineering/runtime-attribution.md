<a name="xdzip-actual-component-attribution-and-license-materials"></a>

# Zuordnung tatsächlicher Komponenten und Lizenzunterlagen

[English](../../engineering/runtime-attribution.md) | [简体中文](../../zh-CN/engineering/runtime-attribution.md) | [繁體中文](../../zh-TW/engineering/runtime-attribution.md) | [日本語](../../ja-JP/engineering/runtime-attribution.md) | [한국어](../../ko-KR/engineering/runtime-attribution.md) | [Français](../../fr-FR/engineering/runtime-attribution.md) | Deutsch | [Español](../../es-ES/engineering/runtime-attribution.md) | [Português (Brasil)](../../pt-BR/engineering/runtime-attribution.md) | [Русский](../../ru-RU/engineering/runtime-attribution.md)

Dieser Ablauf ergänzt dateibezogene SBOMs, indem er tatsächliche Qt-Bibliotheken und Plugins den SPDX-Erklärungen des SDK zuordnet. Seine Ausgabe sind überprüfbare Nachweise, keine automatische Lizenzwahl oder Konformitätsfreigabe. Die Vollständigkeit für eine offizielle Veröffentlichung ist weiterhin nicht erreicht.

<a name="xdzip-inputs-and-sources"></a>

## Eingaben und Quellen

Ab Qt 6.8 gibt es zur jeweiligen Erstellung passende [SPDX-Inventare](https://doc.qt.io/qt-6.8/licenses-used-in-qt.html). Die Werkzeuge lesen nur Laufzeiterklärungen wie qtbase, qtdeclarative, qtsvg und qttranslations aus sbom/ des fixierten Qt-6.8.2-SDK und schließen qttools für Build und Tests aus. SPDX-Version, Modulname und Qt-Version werden geprüft; SHA-256, Namensraum und stabile Paketverweise des Originaldokuments bleiben erhalten. Die Ausgabe enthält nur nötige Felder und kopiert keine Build-Kommentare mit Pfaden von Entwicklungsrechnern.

`scripts/qt-license-sources.json` fixiert offizielle Qt-Quellcode- und SPDX-license-list-data-Commits, Lizenztext-URLs und SHA-256. Downloads werden unter build/ zwischengespeichert; jede Hash-Abweichung bei Cache oder Download führt zum Fehler. Beliebigen SPDX-URLs wird für Materialdownloads nicht gefolgt. Zusätzlich zu den Standardtexten in LICENSES/ werden Originalhinweise für Qt-GUI-Algorithmen zur glatten Skalierung und SVG sowie LGPL-2.1-Texte aus Systemabhängigkeitserklärungen aufgenommen. Upstream-LicenseRef-Texte und Urheberrechtsangaben bleiben im Erklärungs-JSON.

Qt-Laufzeithinweise enthalten auch die fixierte bearbeitbare Quelle der von Qt Network verwendeten Public Suffix List, den MPL-2.0-Text und Abrufanweisungen. Der Sammler prüft Upstream-Blob, SHA-256 und Lizenzbytes. Die Quellcodebereitstellung prüft getrennt, ob diese Eingabe zwei generierte Datenformen im Qt-Archiv reproduziert; siehe [zugehörige Quellmaterialien](../../engineering/source-delivery.md). Dies ersetzt keine Quell- und Austauschprüfungen anderer Qt-Module.

Qt-Core-Tika-MIME-Hinweise bewahren relevante Angaben zur Apache Software Foundation und zu PRONOM der britischen National Archives, Apache-2.0- und OGL-UK-3.0-Lizenztexte und die vollständige fixierte Upstream-NOTICE. Die vollständige NOTICE beschreibt auch andere Upstream-Tika-Komponenten; daraus folgt nicht, dass sie mit diesem Produkt ausgeliefert werden. Siehe [zugehörige Quellmaterialien](../../engineering/source-delivery.md) für ursprüngliche MIME-Daten und die Prüfung der Qt-Umwandlung.

<a name="xdzip-file-and-component-association"></a>

## Zuordnung von Dateien und Komponenten

1. Dateihashes innerhalb der Prüfungswurzel neu berechnen und nach deployment-audit.json veränderte Dateien ablehnen.
2. Der macOS-SDK-Modus verwendet tatsächlich aufgelöste relative Qt-SDK-Pfade und Byteidentität, ohne eigenständiges Deployment zu behaupten.
3. Linux-Deployment erfasst ursprünglichen SDK-Pfad/SHA-256 und Artefaktpfad/SHA-256 nach patchelf. Spätere Prüfungen kontrollieren beide Richtungen und lehnen veraltete oder veränderte Kopiernachweise ab.
4. Windows bevorzugt Byteidentität der SDK-Datei. Eindeutige Dateinamen mit anderen Bytes werden nur als filename-only-unverified-rewrite markiert und als Lücken registriert, nicht als geprüfte Quellen.
5. SPDX CONTAINS ermittelt Dateieigentümer, danach DEPENDS_ON rekursiv auflösen, einschließlich Verweisen zwischen Qt-Repositories und eingebetteten Dritten. Fehlende Paketverweise bleiben Lücken. Nicht über ein repositoryweites CONTAINS fälschlich alle Werkzeuge in die Laufzeit aufnehmen.
6. Nichtnative QML-Dateien und Übersetzungen ohne Einzeldateierklärungen werden nur dem zugehörigen Quell-Repository zugeordnet; individuelle Lizenzen werden nicht abgeleitet. Native SDK-Bibliotheken ohne SPDX-Dateieintrag, native Dateien außerhalb von Qt, NOASSERTION-Lizenzen und fehlende Texte werden getrennt erfasst.

Tatsächliche SDK-Dateibytes können vom SHA-1 im Upstream-Build-SPDX abweichen, etwa durch spätere Installation oder Signatur. Die Werkzeuge bewahren Sollwerte, Vergleiche und Lücken, ohne Ursachen abzuleiten. SDK-Bytezuordnung, Deployment-Umschreibungsnachweise und ursprüngliche Upstream-Hashes sind drei getrennte Nachweisarten.

Das dedizierte macOS-Paketverfahren erzeugt Deployment-Zuordnungen, solange die endgültige signierte App vorhanden ist. SDK-Eingaben werden anhand exakter Ressourcenbytes oder anhand von arm64-Abschnitten, UUID und Importversionen zugeordnet; der Bericht wird der SBOM tatsächlicher Dateien hinzugefügt. Der Vergleich schließt Signaturen, Load-Command-Layout und LINKEDIT aus. Umfang und spezielle Hinweise stehen unter [Herkunft und Hinweise nativer Laufzeitkomponenten](native-runtime-materials.md). Der gewöhnliche SDK-Modus belegt dadurch keine Deployment-Ergebnisse.

Für Plugins, die aus QML-Verzeichnissen nach `Contents/PlugIns/quick/` verschoben werden, darf die Deployment-Zuordnung geprüfte QML-Symlinks im Paket verwenden, um eine eindeutige SDK-Eingabe zu finden. Bei Erzeugung und Nutzung der Nachweise tatsächliche Links, geprüfte Ziele und Dateihashes erneut prüfen und denselben Byte- oder Mach-O-Vergleich ausführen. Fehlende Herkunft bleibt ungeprüft; mehrere verschiedene Quellen oder geänderte Links werden abgelehnt. Dateinamen allein stärken weder die Zuordnung noch beseitigen sie Upstream-SDK-Hashdifferenzen oder Lücken der Lizenzprüfung.

Rust behält den vollständigen Host-Buildgraphen und erzeugt Laufzeitkandidaten entlang gewöhnlicher Abhängigkeiten von Desktop `xdzip-qt-bridge`, Format-Worker `xdzip-codec-bridge` und dem unter Windows/Linux tatsächlich verwendeten `xdzip-update-helper`; macOS hat keine eigene Update-Helper-Wurzel. Build-, Entwicklungs- und Proc-Macro-Werkzeuge samt Abhängigkeiten sind ausgeschlossen. Dies sind mögliche Laufzeiteingaben, kein Beweis, dass Maschinencode jedes Pakets das Linken übersteht; Linker-Eliminierung, natives C/C++ und Compiler-Laufzeiten erfordern weitere Prüfung.

Einfache alte Cargo-Lizenzalternativen mit Schrägstrichen werden im Quell-CycloneDX zu SPDX `OR`, wobei das Upstream-Original in `xdzip:cargo_license_raw` erhalten bleibt. Schrägstrichausdrücke mit mehrdeutiger Priorität werden abgelehnt. Ursprüngliche Build-Inventare und Hinweise werden nicht umgeschrieben; Standard-`AND`, `OR` und `WITH` bleiben unverändert. Dies normalisiert das Format und wählt keinen Lizenzzweig oder genehmigt eine Veröffentlichung.

<a name="xdzip-execution-and-delivery"></a>

## Ausführung und Bereitstellung

Die bekannte gepatchte QtNetwork-Variante ergänzt Dateieinträge um `network_security_variant`; Berichte bewahren auch vollständige, erneut geprüfte Build-Belege. Upstream-SDK-Hashunterschiede bleiben sichtbar; die Veröffentlichungsprüfung nutzt fixierte Quellen, vollständige Patches, Build-Belege und Nachweise der Deployment-Transformation zur Erklärung. Die Vorbereitung verlangt eine Zuordnung des endgültigen Network zu diesen Bytes; ein Dateiname genügt nicht. Quell- und Artefakt-SBOM kennzeichnen die Variante getrennt, ohne den global unvollständigen Status in eine Lizenzfreigabe umzuwandeln.

```sh
python3 scripts/prepare-runtime-notices.py --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --output build/review/notices/Qt-runtime
python3 scripts/runtime_attribution.py --audit build/review/deployment-audit.json \
  --root build/review/linux --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --deployment build/review/linux-deployment.json --output build/review/runtime-attribution.json
python3 scripts/sbom.py --audit build/review/deployment-audit.json \
  --source build/review/source-build.cdx.json --attribution build/review/runtime-attribution.json \
  --output build/review/artifact-files.cdx.json
```

Linux übergibt Deployment-Umschreibungsnachweise; andere Plattformen lassen --deployment weg. Berichte außerhalb der Prüfungswurzel schreiben, um rekursive Selbstinventarisierung zu vermeiden. Qt-Workflows auf Anforderung und Release-Workflows bereiten Lizenzmaterial vor Deployment und endgültiger Hashbildung vor. Windows und Linux legen Hinweise in den tatsächlichen Laufzeitumgebungen ab; Linux tar.gz liefert sie ebenfalls aus. Qt auf Anforderung liest unter macOS das SDK und protokolliert nur Logs/JSON. Ein ausdrücklich autorisierter dedizierter Release-Workflow führt Deployment, Ad-hoc-Signatur, Paketierung und Prüfung tatsächlicher Release-Materialien nach dem [Veröffentlichungsprotokoll](release-protocol.md) separat aus und bewahrt eigenen Deployment-Umfang und Hashes. SDK-Nachweise ersetzen nicht die Zuordnung des endgültigen App-Bundles.

runtime-attribution.json enthält Dateien, Komponenten, Beziehungen, Upstream-Erklärungen, Urheberrechte, Textquellen und Lücken. artifact-files.cdx.json ergänzt dieselben Komponenten und bekannten Eigentümerbeziehungen und bindet Bericht, tatsächliche Audit-Hashes und Build-Identität aneinander. Prüfungen des offiziellen CycloneDX-Schemas und der Referenzen laufen weiter; compositions bleiben unvollständig.

Standardprüfungen erlauben ausdrücklich erfasste Entwicklungslücken, markieren sie aber nie stillschweigend als bestanden. Fehler bei Identität, Pfad, Bytes, Version, Kopiernachweis oder Lizenztext-Hash schlagen sofort fehl. --require-complete blockiert die Veröffentlichung und schlägt derzeit bei unvollständiger Zuordnung oder unvollständigen Release-Materialien fehl. Tests decken überschriebene Dateien, veraltete Kopiernachweise, Pfadausbruch, falsche Versionen, dokumentübergreifende Auslassungen, unvollständige SBOMs und den Ausschluss von Rust-Werkzeugen ab.

<a name="xdzip-unresolved-release-gates"></a>

## Offene Veröffentlichungsbedingungen

Ein [Sammler für Quellcodebereitstellung](../../engineering/source-delivery.md) bewahrt tatsächlichen Projektquellcode, Rust vendor, vier Qt-Laufzeitmodularchive, zwei Build-Modularchive und Anweisungen zu Neubau und dynamischem Austausch. Er beweist keine SDK-/Upstream-Buildgleichheit je Element. Jede Veröffentlichung muss weiterhin exakte statische Linkzuordnung, Betriebssystem- und weiterverteilbare Laufzeiten, Einzellizenzen aller eingebetteten Ressourcen, Entscheidungen über Open-Source-Lizenzzweige und Erfüllung prüfen. Metadatenänderung oder -austausch bei der [lokalen Mac-Deployment-Prüfung](../../engineering/macos-deployment.md) und [tatsächlicher Qt-Quellbuild mit Laden und Austausch](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement) sind unterschiedliche Abläufe mit getrennten Nachweisen; keiner belegt Bytegleichheit beliebiger SDK-Builds. Quellinventare und SPDX-Felder ersetzen keine Lizenzpflichten. Herkunft und Lizenzen des historischen Kandidaten `v0.1.0` stehen in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) und [#53](https://github.com/xiaodaozhi/xdzip/issues/53); spätere Kandidaten benötigen eigene Prüfung, ohne bezahlte Signaturidentität als Voraussetzung. Die bestehende Desktop-Abnahme bleibt in [#5](https://github.com/xiaodaozhi/xdzip/issues/5).

Die [Qt-SDK-Korrespondenzprüfung](qt-sdk-correspondence.md) erfasst zusätzlich fixierte Archiv-Commits, Einzeldateihashes in SDK-Quellerklärungen, installierte Konfiguration und den bekannten AGL-Consumer-Patch. Übereinstimmungen im erklärten Umfang belegen keine vollständige SDK-Buildgleichheit; bestehende Veröffentlichungshindernisse bleiben bestehen.

Die automatisierte Nacharbeit ergänzt tatsächliche Linker-Maps und statische Bibliotheksmitglieder, Laufzeitressourcen-Hashes, vertrauenswürdige Versionen nativer Abhängigkeiten und eine exakt umkehrbare Binärdifferenzanalyse; siehe den [vollständigen Vertrag](../../../developers/engineering/candidate-verification.md). Eine Transformation nur als erklärt markieren, wenn rekonstruierte Byte-Hashes exakt den Upstream-Erklärungen entsprechen. Ursprüngliche SDK-Unterschiede nicht löschen und undokumentierte Herausgeberhistorie nicht erraten.
