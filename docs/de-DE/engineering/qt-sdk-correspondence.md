<a name="xdzip-qt-sdk-and-source-correspondence-verification"></a>

# Überprüfung der Übereinstimmung von Qt SDK und Quellcode

[English](../../engineering/qt-sdk-correspondence.md) | [简体中文](../../zh-CN/engineering/qt-sdk-correspondence.md) | [繁體中文（台灣）](../../zh-TW/engineering/qt-sdk-correspondence.md) | [日本語](../../ja-JP/engineering/qt-sdk-correspondence.md) | [한국어](../../ko-KR/engineering/qt-sdk-correspondence.md) | [Français](../../fr-FR/engineering/qt-sdk-correspondence.md) | Deutsch | [Español](../../es-ES/engineering/qt-sdk-correspondence.md) | [Português (Brasil)](../../pt-BR/engineering/qt-sdk-correspondence.md) | [Русский](../../ru-RU/engineering/qt-sdk-correspondence.md)

Die Prüfung der Quellübereinstimmung verwendet die tatsächlichen Bytes offizieller Archive; gleiche Versionsnummern belegen keine Gleichwertigkeit des SDK-Builds. `scripts/qt_sdk_correspondence.py` verbindet festgelegte Quellarchive, vom SDK gelieferte Quell-SPDX, Build-Deklarationen, installierte Konfiguration und bekannte Patches der SDK-Nutzer und schreibt die Ausgabe nach build/. Das Werkzeug verändert das SDK nicht und genehmigt keine Lizenzzweige.

```sh
python3 scripts/qt_sdk_correspondence.py \
  --qt-prefix build/toolchain/Qt/6.8.2/macos \
  --output build/review/qt-sdk-correspondence.json
```

Standardmäßig wird build/source-cache offline gelesen und bei Fehlen ein Fehler ausgegeben. Nur ein ausdrücklich angegebenes --fetch lädt offizielle Archive aus dem [festgelegten Manifest](../../../scripts/corresponding-sources.json) und prüft sie. Die bei Bedarf gestarteten Qt-Workflows für drei Plattformen führen --fetch nach Installation des festgelegten SDK aus und bewahren JSON mit den anderen Prüfnachweisen auf. Auf dem Mac werden weiterhin nur Nachweise hochgeladen, ohne Apps zu deployen, zu signieren oder zu paketieren. Erhält der Sammler für Quelllieferungen --qt-prefix, nimmt er diesen Nachweis, Patches und das festgelegte Manifest in die Lieferhashes auf; siehe [Quelllieferung](source-delivery.md).

<a name="xdzip-verification-layers"></a>

## Prüfschichten

1. Der SHA-256 des Archivs muss dem festgelegten Wert entsprechen; Archivwurzel, Mitgliedspfade und festgelegte `.tag`-Markierung müssen übereinstimmen. Der Git-Objekttyp der Markierung wird getrennt erfasst. Bei einer Baum-Markierung zuerst den Hash des festgelegten ursprünglichen Commit-Objekts neu berechnen und sein tree-Feld prüfen. Dateien einzeln lesen, ohne upstream-Quellen zu entpacken.
2. Stellt das SDK für ein Modul eine .source.spdx bereit, jeden deklarierten SHA-1 mit dem Archivinhalt vergleichen. Übereinstimmungszahlen, beide Hashes bei Abweichungen, im SDK deklarierte, aber im Archiv fehlende Dateien sowie die Anzahl nicht abgedeckter Archivdateien festhalten. SHA-1 prüft nur die ursprüngliche upstream-Deklaration; das Gesamtarchiv wird getrennt durch den festgelegten SHA-256 gebunden. Fehlende Deklarationen, Abweichungen und nicht abgedeckte Dateien werden nicht als bestanden ergänzt.
3. SPDX-Hashes des SDK-Modulbuilds, Compilerdeklarationen und configure-Argumente aufbewahren. Zusätzlich installierte modules/*.json, qconfig*.pri und QT_FEATURE-Konfigurationsheader lesen und Inhalt und Hashes bewahren. Absolute Verzeichnisse des upstream-Builders in der Ausgabe durch Platzhalter ersetzen; Dateihashes binden die ursprünglichen Nachweise. Die Konfiguration ist weder der vollständige CMake-Cache noch Build-Umgebung oder Toolchain-Material. SDK-Standardarchitekturen geben nicht alle Architekturen tatsächlicher Bibliotheken wieder und dürfen Universalbibliotheken nicht fälschlich als x86_64 einstufen.
4. Der [AGL-Patch](../../../scripts/qt-patches/macos-no-agl.patch) ändert nur lib/cmake/Qt6/FindWrapOpenGL.cmake im SDK, damit dieses Projekt bei Qt-Nutzung nicht gegen altes AGL linkt. Das Werkzeug liest das Original aus dem festgelegten QtBase-Archiv, spielt den Unified Diff nach, prüft die SHA-256-Werte von Patch, Original und Ergebnis und anschließend die tatsächliche SDK-Datei. Nur Original oder bekannte gepatchte Version sind zulässig. Windows-SDK-CRLF wird erst nach Normalisierung der Zeilenenden verglichen; SHA-256 der tatsächlichen Bytes, LF-SHA-256 und ein Konvertierungsmarker bleiben erhalten. Andere unbekannte Änderungen führen zum Fehler. Dies ändert keinen Qt-Laufzeitmaschinencode und beweist keinen Qt-Neubau. Die vollständige SDK-Änderungshistorie muss weiterhin geprüft werden.

Quellkorrespondenznachweise verwenden Schema 2 und bewahren festgelegtes archive_commit, tatsächliches archive_marker, archive_marker_type und commit_tree_verified getrennt auf. Die `.tag` von Qt Declarative 6.8.2 ist der Git-Baum `0a8139c574fe150aba92d5f2484a8f3fdb774977`; das Release-Tag verweist auf Commit `75534f3e7fff24ed7ccb364e2ed9950a73da879f`. Ein im Repository festgelegtes ursprüngliches Commit-Objekt in Base64 erlaubt die Offline-Neuberechnung des Commit-Hashes und die Prüfung seines tree-Felds. Andere derzeit festgelegte Modulmarkierungen sind Commits. Die Baumbeziehung belegt nur die Beziehung dieser beiden Git-Objekte, nicht den vollständigen Bauminhalt des Archivs oder eine Build-Gleichwertigkeit des SDK.

source_commits in qt-license-sources.json bezeichnet ausschließlich festgelegte Abrufcommits für Lizenztexte und wird getrennt von Archivmarkierungen und Release-Commits gespeichert. Diese Werte dürfen nicht als SDK-Build-Commits dargestellt werden. Historische Schema-1-Materialien schrieben die Qt-Declarative-Baummarkierung in archive_commit; veröffentlichte Artefakte bleiben unverändert. Spätere Erläuterungen sollen den Objekttyp korrigieren und Lizenztexte weiterhin mit dem ursprünglichen SHA-256 prüfen.

<a name="xdzip-interpreting-results-and-release-boundaries"></a>

## Ergebnisse und Release-Grenzen

Die QtNetwork-Sicherheitsableitung erfasst `network_security` getrennt; Belege binden Patches, Builder, Digests des ursprünglichen QtCore und des tatsächlichen QtNetwork, Funktionen und echte Transportprüfung. Beim Lesen der Belege prüft das Werkzeug die tatsächlichen SDK-Bytes erneut. Dieser Nachweis gilt nur für die veränderte Bibliothek; er schreibt upstream-SPDX nicht als Nachweis der Ableitung um und verändert keine nicht abgedeckten Fakten anderer Module. Eingaben und Wartung stehen in den [Qt-Patchhinweisen](../../../patches/qt/README.md).

Ob Module Quell-SPDX liefern, die Zahlen übereinstimmender, fehlender und nicht abgedeckter Dateien, Konfiguration und Patchstatus ergeben sich aus dem JSON des aktuellen Laufs. Beobachtungen eines SDK dürfen nicht auf andere übertragen werden. Ohne dateiweise Deklarationen darf keine bestandene Korrespondenz angenommen werden; konkrete Lücken bleiben erhalten.

Der Berichtstatus lautet stets evidence-collected-not-build-equivalence. Der [tatsächliche Qt-Quellbuild und das Laden oder Ersetzen](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement) haben einen eigenen Ablauf, belegen aber keine bytegenaue Reproduktion des ursprünglichen SDK. Solange vollständige Toolchain- und System-SDK-Materialien, nicht abgedeckte oder generierte Dateien, Laufzeitherkunft und Lizenzerfüllung nicht vollständig geklärt sind, dürfen weder alle Pflichten zu korrespondierendem Quellcode noch byteweise reproduzierbare Builds als erfüllt behauptet werden. Der unvollständige Status der [Laufzeitzuschreibung](runtime-attribution.md), Lizenzentscheidungen und plattformspezifische Laufzeitprüfungen bleiben bestehen.
