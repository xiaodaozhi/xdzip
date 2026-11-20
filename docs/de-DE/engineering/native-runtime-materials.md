<a name="xdzip-native-runtime-provenance-and-notices"></a>

# Herkunft und Hinweise nativer Laufzeitkomponenten

[English](../../engineering/native-runtime-materials.md) | [简体中文](../../zh-CN/engineering/native-runtime-materials.md) | [繁體中文（台灣）](../../zh-TW/engineering/native-runtime-materials.md) | [日本語](../../ja-JP/engineering/native-runtime-materials.md) | [한국어](../../ko-KR/engineering/native-runtime-materials.md) | [Français](../../fr-FR/engineering/native-runtime-materials.md) | Deutsch | [Español](../../es-ES/engineering/native-runtime-materials.md) | [Português (Brasil)](../../pt-BR/engineering/native-runtime-materials.md) | [Русский](../../ru-RU/engineering/native-runtime-materials.md)

Dieser Vertrag umfasst native Laufzeitkomponenten außerhalb von Qt in tatsächlichen Paketen und wird zusammen mit [Laufzeitzuschreibung](runtime-attribution.md) und [korrespondierendem Quellcode](source-delivery.md) verwendet. Quellversion, Dateiidentität, Lizenzbedingungen und tatsächliche Erfüllung werden getrennt geprüft; Hashes von Lieferantendateien sind keine Lizenzfreigabe. Historische Kandidatenfeststellungen und ergänzende Nachweise zu `v0.1.0` stehen in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) und [#53](https://github.com/xiaodaozhi/xdzip/issues/53). Zukünftige Kandidaten verwenden neue Release-Issues; Nachweise entstehen nur unter build/ oder in CI-Artefakten.

| Komponente | Ursprüngliche Grundlage und Liefermaterialien | Prüfgrenze |
| --- | --- | --- |
| Qt 6.8.2 | SDK-SPDX, ursprüngliche Modulquellen und Patches, Qt-Hinweise, LGPLv3- und GPLv3-Texte, Anleitungen zum App-Neubau und kompatiblen Ersatz | Die Laufzeit nutzt den LGPL-Zweig; Werkzeug- und Drittanbieterlizenzen bleiben getrennt. SDK-Dateikorrespondenz ist keine vollständige upstream-Build-Gleichwertigkeit |
| Rust-Laufzeitcode | Cargo.lock, tatsächliche Linkeingaben, Kandidaten für Laufzeitabhängigkeiten, vollständige Vendor- und Originalhinweise, Rust-Standardbibliotheksquelle | Hinweise des Build-Graphen vorsorglich liefern, ohne build/dev/proc-macro-Werkzeuge fälschlich als gelinkten Laufzeitcode auszugeben |
| Abgeleitete Schriftarten aus Noto Sans SC, TC, JP, KR und Noto Sans | Schriftmanifest, ursprüngliche OFL-Texte, umbenannte Schriftarten, Generierungseinstieg | Apache-2.0 des Projekts gilt nicht für Schriftarten; Erklärungen zu Änderungen und Benennung erhalten |
| ICU 73.2 | Tatsächliche Bibliotheksversion, vollständige upstream-ICU-LICENSE einschließlich Dritthinweisen, Herkunftsnachweise | Allgemeiner MIT-Text ersetzt nicht ICUs eigene Hinweise; Systemhüllen nicht mit tatsächlich paketierten Dateien verwechseln |
| Microsoft VC++ Runtime | Festgelegtes offizielles VC_redist-Paket und DLL-Hashes, Originallizenz, Versions- und Distributionsinventar | Dateien bleiben unverändert; Berechtigung der Maintainer und Weitergabebedingungen separat prüfen |
| Grafikcompiler-Bibliotheken des Windows SDK | Aus neuen Paketen ausgeschlossen; historische Pakete behalten ursprüngliche SDK-Lizenz, Hashes je DLL und Herkunft | Historische Distributionsbedingungen bleiben separat geprüft und ändern sich nicht durch Entfernung aus neuen Paketen |
| Inno Setup 6.7.3 Installationsengine | Festgelegtes Compilerpaket und Quellcommit, ursprüngliche Inno- und RemObjects-Pascal-Script-Lizenzen, zweisprachige Installer-CREDITS.txt | Bestehende Copyright-Oberfläche der Installationsengine erhalten. Pascal Script verlangt zusätzlich Name und Quell-URL in Über oder Dokumentation; die allgemeine Inno-Lizenz allein reicht nicht |

Microsofts [Hinweise zur Runtime-Weitergabe](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files) und die [VS-2022-Weitergabeliste](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution) binden Weitergaberechte an geltende Softwarelizenzen. Der Lizenztext des Installers beweist nicht die Berechtigung der Maintainer. Die [Open-Source-Pflichten von Qt](https://www.qt.io/development/open-source-lgpl-obligations) verlangen, dass Nutzer korrespondierenden Quellcode erhalten, ändern, erneut linken und veränderte Bibliotheken ausführen können; dynamisches Linken ersetzt keine anderen Pflichten.

Der [Windows-Lizenzleitfaden](windows-licensing.md) beschreibt den kostenlosen Community-Weg, die separate Berechtigung für gehostetes Enterprise und Weitergabebedingungen. Die Bereitstellung enthält ursprüngliche CRT/SDK-Lizenzen, Herkunft und zweisprachige Windows-runtime-NOTICE.txt sowohl in der Laufzeit als auch in den Materialien. Vor der Kompilierung prüft Setup den Hash der ursprünglichen CRT-Vereinbarung und zeigt deren Lizenzseite. Microsoft-DLLs werden nicht unter Apache-2.0 neu lizenziert.

<a name="xdzip-generation-and-verification"></a>

## Generierung und Prüfung

`scripts/native-runtime-sources.json` legt ICU-Lizenzversion, offizielle URL und SHA-256 fest. `collect-notices.py` sammelt unter Linux die vollständige ICU-Lizenz und prüft Hashes sowohl bei Caches als auch Downloads erneut. Aktuelle Windows-Entwicklungspakete enthalten keine `opengl32sw.dll` und sammeln dafür keine Mesa/LLVM-Hinweise mehr. Enthält das Ausgabehinweisverzeichnis alte spezifische Materialien, verweigert die Sammlung dessen Wiederverwendung, um sie nicht falsch einem neuen Kandidaten hinzuzufügen.

`prepare-source-delivery.py` enthält ICU-Lizenz und festgelegtes Quellinventar. Materialien behalten Originalbytes; das Liefermanifest berechnet Größen und Hashes neu. Historische Pakete mit Mesa-Software-Rendering-Bibliotheken benötigen weiterhin entsprechende historische Quellen, Hinweise und Pflichten; die neue Regel kann frühere Lieferungen nicht umschreiben.

Plattformmaterial-Anhänge müssen die im Zuordnungsbericht unter `license_materials_base` referenzierten Verzeichnisse enthalten. Jeden Original-Lizenztexthash vor dem Paketieren erneut prüfen. Ein identischer Text anderswo im Laufzeitverzeichnis rechtfertigt keinen Bericht mit einem fehlenden relativen Verzeichnis.

Die Windows-Bereitstellung deaktiviert Umgebungssuchen von windeployqt nach Software-OpenGL, D3D-Compilern und DXC und kopiert die drei Grafikcompiler-Bibliotheken nicht mehr aus dem Windows SDK. Staging mit diesen Namen ablehnen. Eine verbliebene Software-OpenGL-DLL im Wurzelverzeichnis des kontrollierten Stagings entfernen und gleichnamige Reste anderswo ablehnen.

Die macOS-Paketierung erzeugt `deployment-audit.json`, `macos-deployment.json` und `runtime-attribution.json`, solange die endgültig signierte App noch existiert. Die Datei-SBOM verwendet denselben Audit- und Zuordnungsbericht. Native Qt-Dateien anhand von arm64-Slices, allen Abschnittsbytes, UUID und Importversionen vergleichen und die für die Bereitstellung nötige Qt-Pfadverlagerung zulassen. Signaturen, Load-Command-Layout und LINKEDIT liegen außerhalb dieser Vergleichsaussage. Nicht native Ressourcen anhand exakter Bytes zuordnen. Nicht zugeordnete Dateien und ursprüngliche SPDX-Lücken bleiben bestehen; dies ist keine vollständige binäre Gleichwertigkeit.

Werden fehlende Hinweise nach Veröffentlichung entdeckt, dürfen ergänzende Materialien nicht als bereits im Originalpaket enthalten bezeichnet werden. Ursprüngliche Versionsartefakte und Prüfsummen bleiben unveränderlich; korrigierte Pakete erfordern eine neue Version und erneute Prüfung. Ergänzungen und ihre Zuordnung ausdrücklich dokumentieren.
