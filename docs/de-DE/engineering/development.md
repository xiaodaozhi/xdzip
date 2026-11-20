<a name="xdzip-developer-preparation-guide"></a>

# Vorbereitung der Entwicklungsumgebung

[English](../../engineering/development.md) | [简体中文](../../zh-CN/engineering/development.md) | [繁體中文](../../zh-TW/engineering/development.md) | [日本語](../../ja-JP/engineering/development.md) | [한국어](../../ko-KR/engineering/development.md) | [Français](../../fr-FR/engineering/development.md) | Deutsch | [Español](../../es-ES/engineering/development.md) | [Português (Brasil)](../../pt-BR/engineering/development.md) | [Русский](../../ru-RU/engineering/development.md)

Unter [Bauen und beitragen](../contributing/README.md) finden Sie passende Build- und Prüfpfade für Ihre Änderung. [Systemarchitektur](../../design/system-architecture.md) und [Repository-Struktur](repository-layout.md) erklären die Grenzen zwischen Qt, Rust-Laufzeit und Codecs.

<a name="xdzip-obtain-and-modify"></a>

## Quellcode beziehen und ändern

Beginnen Sie mit dem aktuellen `main` und bewahren Sie andere Änderungen im Arbeitsverzeichnis. Die [Beitragsregeln](../../../CONTRIBUTING.md) beschreiben Umfang und Review. Die [Auslieferungsanleitung](../../../developers/engineering/repository-hygiene.md) beschreibt die autorisierte Integration. Ändert sich ein Vertrag, aktualisieren Sie seine Sprachfassungen und die betroffenen Prüffälle. Farben und Geometrie stammen aus `specs/visual`.

<a name="xdzip-documentation-builds"></a>

## Dokumentation bauen und prüfen

Für jede unterstützte Oberflächensprache gehören ein passendes README im Stammverzeichnis und entsprechende Seiten unter docs dazu. Englisch verwendet standardmäßig `README.md` und `docs/`. Übersetzungen nutzen `README.<locale>.md` und `docs/<locale>/`. Das Wartungshandbuch erscheint auf US-Englisch und vereinfachtem Chinesisch. Platzieren Sie die Sprachauswahl unter dem Titel. Die aktuelle Sprache steht als Text. Die anderen verlinken dasselbe Thema. Von einer Seite in vereinfachtem Chinesisch führt der Wartungslink zur vereinfachten Fassung. Von allen anderen Seiten führt er zur englischen Standardfassung. Aktualisieren Sie beim Hinzufügen oder Verschieben Indexe, eingehende Links und Abschnittsanker.

Die Dokumentationswerkzeuge benötigen Python 3.10+, lxml, Pandoc und Graphviz. Systemschriften für CJK werden empfohlen. `python3 scripts/build-design-html.py` erzeugt 16 ausführliche Software-Entwurfsdokumente und eine Ein-Datei-Fassung unter build/docs-html aus den chinesischen Quellen in `docs/zh-CN/design/`. Das Downloadpaket liegt unter dist/. Weitere Technik- und Beitragsdokumente stehen in docs/README.md. Diese HTML-Ausgabe dient der Dokumentation. Die Produktoberfläche nutzt Qt Quick.

`python3 scripts/check-docs.py` prüft Übersetzungsabdeckung, wechselseitige Sprachlinks, Zielsprache von Leselinks, Markdown-Anker, JSON und Entwurfsressourcen. Die Prüfung läuft offline. Der HTML-Generator prüft Links und Struktur seiner Ausgaben. Führen Sie bei Änderungen an der Prüfung `python3 scripts/test-check-docs.py` aus.

<a name="xdzip-product-toolchain-preparation"></a>

## Produkt-Toolchain vorbereiten

Unter macOS werden Xcode Command Line Tools und Qt benötigt. Unter Windows sind es MSVC Build Tools, Windows SDK und ein passendes Qt. Linux benötigt einen Compiler, eine glibc-Umgebung für das Ziel und Qt-Abhängigkeiten für Wayland/X11. Die Toolchain-Dateien im Stammverzeichnis und CI legen die Versionen fest. Quellen und Lizenzen stehen in der [Toolchain-Tabelle](toolchain.md). Die [Support-Richtlinie](support-policy.md) erklärt unterstützte Systeme und Wartung. Für Herkunft und Lizenzen dient das Ticket zur jeweiligen Freigabeprüfung.

Die [Qt-Entwicklungsanwendung](../../../apps/desktop/README.md) dokumentiert Befehle für Build und Start aus einem sauberen Checkout, CMake-Optionen, macOS-Architektur und Bereitstellungsziele sowie die Grenzen der Offscreen-Tests. Prüfen Sie nach Änderungen an Build oder Installation erneut Pfade mit Leerzeichen und chinesischen Zeichen. Für die lokale Entwicklung ist keine kostenpflichtige Signaturidentität erforderlich. Die [Build-Anleitung](../../engineering/developer-build.md) beschreibt die lokale Ad-hoc-Integritätssignatur der DMG und die CI-Grenze für Mac-App-Bereitstellung.

<a name="xdzip-independent-engineering-probes"></a>

## Eigenständige technische Proben

Die [Toolchain-Probe](../../../tools/toolchain-probe/README.md) bietet einen Minimalbuild und CTest-Befehle für ein eigenständiges Qt-Quick-Fenster. Versionen und Testumfang stehen dort. Ergebnisse stehen im [Ticket zur ursprünglichen Prüfung](https://github.com/xiaodaozhi/xdzip/issues/1).

Die [gemeinsame CMake/Cargo-Brückenprobe](../../../tools/bridge-probe/README.md) bietet einen Build-Einstieg im Stammverzeichnis, fixierte Abhängigkeiten und Lebenszyklusprüfungen. Siehe [Begründung für cxx](cxx-admission.md) für die Abhängigkeiten.
