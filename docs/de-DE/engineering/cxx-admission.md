<a name="xdzip-cxx-bridge-dependency-rationale"></a>

# Begründung für die cxx-Brücke

[English](../../engineering/cxx-admission.md) | [简体中文](../../zh-CN/engineering/cxx-admission.md) | [繁體中文（台灣）](../../zh-TW/engineering/cxx-admission.md) | [日本語](../../ja-JP/engineering/cxx-admission.md) | [한국어](../../ko-KR/engineering/cxx-admission.md) | [Français](../../fr-FR/engineering/cxx-admission.md) | Deutsch | [Español](../../es-ES/engineering/cxx-admission.md) | [Português (Brasil)](../../pt-BR/engineering/cxx-admission.md) | [Русский](../../ru-RU/engineering/cxx-admission.md)

Qt und Rust verwenden eine von `cxx` erzeugte typisierte Brücke mit festgelegten Besitzregeln. C++ verwaltet Qt-Objekte, Rust die Domänen- und Laufzeitlogik. `cxx` und `cxx-build` müssen gemäß [Cargo.lock](../../../Cargo.lock) und der [Toolchain-Richtlinie](toolchain.md) exakt dieselbe Version haben. cxx ist keine stabile ABI für externe Formaterweiterungen. Eine handgeschriebene C-ABI hätte weniger Abhängigkeiten, aber mehr Pflegeaufwand bei Zeichenfolgen, freien Funktionen und Besitzregeln.

<a name="xdzip-source-and-licensing"></a>

## Quelle und Lizenzierung

Die Quelle ist crates.io, Upstream ist [dtolnay/cxx](https://github.com/dtolnay/cxx). Hashwerte, Versionen, Features, Buildskripte und Lizenzdeklarationen stehen in der Lockdatei und im [Abhängigkeitsinventar der Brücke](../../../specs/engineering/bridge-dependencies.json). Prüfen und bündeln Sie für die Veröffentlichung die tatsächlich einschlägigen Hinweise. Die Lockdatei enthält Abhängigkeiten, deren Zielbedingungen möglicherweise nicht zutreffen. Tatsächlich gelinkte Binärdateien, Compiler, Qt und Systembibliotheken werden separat erfasst.

<a name="xdzip-build-and-ownership-boundaries"></a>

## Build und Besitzgrenzen

cxx-build erzeugt Quellen und Header und ruft C++-Compilerwerkzeuge auf. cxx kompiliert die mitgelieferte Brückenimplementierung, link-cplusplus linkt die C++-Standardbibliothek. Cargo prüft bezogene Abhängigkeiten anhand der Lockdatei. Neue Buildskripte oder optionale Features erfordern eine erneute Prüfung des Netzwerk- und Ausführungsverhaltens. Frühere Prüfungen gelten nicht als Sicherheitsnachweis für neue Versionen.

Behalten Sie std/alloc und die anderen benötigten Features. Einschränkungen erfordern neue Prüfungen für UTF-8, Fehlerweitergabe, eindeutige Handles, Rückstau, Beenden und Leeren der Warteschlangen. Der [Laufzeitvertrag](product-runtime.md) und der [Brückenentwurf](../design/qt-rust-bridge.md) definieren die tatsächlichen Schnittstellen. Für Lizenzen gilt die [Freigabeprüfung](../../../developers/qa/release-review.md).

Konkrete Tests, CI-Läufe und Ergebnisse stehen im [Issue WP-01](https://github.com/xiaodaozhi/xdzip/issues/1) und dessen Actions-Artefakten.
