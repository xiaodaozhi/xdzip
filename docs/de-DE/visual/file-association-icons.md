<a name="xdzip-file-association-icons"></a>

# Dateizuordnungssymbole

[English](../../visual/file-association-icons.md) | [简体中文](../../zh-CN/visual/file-association-icons.md) | [繁體中文](../../zh-TW/visual/file-association-icons.md) | [日本語](../../ja-JP/visual/file-association-icons.md) | [한국어](../../ko-KR/visual/file-association-icons.md) | [Français](../../fr-FR/visual/file-association-icons.md) | Deutsch | [Español](../../es-ES/visual/file-association-icons.md) | [Português (Brasil)](../../pt-BR/visual/file-association-icons.md) | [Русский](../../ru-RU/visual/file-association-icons.md)

Systemdateisymbole verwenden xdzip-Dokument mit umgefalteter Ecke, Badge mit stetiger Krümmung, vier verflochtene Bänder und offene U-förmige Marke. Parameter stammen aus [file-association-icons.json](../../../specs/visual/file-association-icons.json), Dokumentsymbolquellen und Produktplattformcontainer aus [file-associations](../../../design/assets/file-associations). Benutzer wählen ihre Standardanwendungen weiterhin selbst.

<a name="xdzip-shape-and-identification"></a>

## Form und Erkennung

Papier- und Markengeometrie verwenden [brand_assets.py](../../../scripts/brand_assets.py). Formate teilen Papier, Falz, Badgeposition und optisches Zentrum; Beschriftungen identifizieren Formate, Akzentfarben erleichtern die Übersicht. TAR.GZ, TGZ und GZ bilden eine türkisfarbene Familie. Der generische Rückfall nutzt neutrales Graublau und Dokumentlinien ohne Formatbeschriftung.

Beschriftungen folgen strikt dem ursprünglichen ZIP-Glyphenstil: Höhe 61, Strich 14, eckige Enden, gerundete Verbindungen, Oberkante bei Master-Y=817 und Farbe #39465D. ZIP verwendet unmittelbar den vollständigen Textpfad des ursprünglichen Masters; andere Buchstaben werden nach denselben Maßen und Strichregeln ergänzt. Es gibt keinen SVG-Text, keine Systemschriften, Fremdsymbole oder eingebetteten Bitmaps. Maximale Breite ist 504. Das vollständige TAR.GZ-Label ohne Streckung der Glyphen erhalten; nur bei Überschreitung des Breitenbudgets gleichmäßig skalieren und vertikal zentrieren. Farbe ist nicht das einzige Erkennungsmerkmal.

Der Master umfasst 1024 Einheiten. Generische PNG/ICO behalten die viewBox `32 32 960 960` mit etwa 90% Papierkörperhöhe; macOS-PNG/ICNS behalten `-20 -20 1064 1064` mit etwa 81,2% Körperhöhe. Dies setzt die optische Kalibrierung des Projekts fort und wird nicht als Systemanforderung behauptet. Symbole haben transparente Zeichenflächen ohne themenspezifisches Papier, Desktophintergrund oder zusätzliche abgerundete Masken.

Bei 16 und 24 Pixeln wird das Badge vergrößert und das Papier vereinfacht; 32 Pixel verwenden ein etwas kleineres Badge. Diese drei Größen lassen Beschriftungen und Schatten großer Bilder weg, bewahren aber das ursprüngliche Seitenverhältnis der Marke. Symbole allein unterscheiden bei diesen Größen nicht alle Formate zuverlässig, besonders die gleichfarbige GZIP-Familie; Dateinamen- und Typspalten ermöglichen genaue Erkennung. Beschriftete Master werden ab 48 Pixeln verwendet; lange Beschriftungen sind ab 64 Pixeln besser lesbar. Jede Größe direkt aus Vektoren rendern statt eine 1024-PNG wiederholt zu verkleinern.

Die 24-Einheiten-Symbole der UI sowie Hauptfenster- und Menüdichte folgen weiterhin [layout-scale.json](../../../specs/visual/layout-scale.json). Diese Ressourcen fügen kein QML-Fenster hinzu und ändern keine logischen Hauptfenster- oder Menümaße. Plattform-PNG-Abmessungen sind Bildabtastmaße und dürfen nicht in logische QML-Komponentengeometrie eingehen.

<a name="xdzip-format-and-platform-mapping"></a>

## Format- und Plattformzuordnung

| Symbol-ID | Label | Erweiterung / Nutzung | Akzent |
| --- | --- | --- | --- |
| `zip` | ZIP | `.zip` | Markenblau |
| `7z` | 7Z | `.7z` | Indigo |
| `rar` | RAR | `.rar` | Violett |
| `tar` | TAR | `.tar` | Bernstein |
| `tar-gz` | TAR.GZ | Bestätigte zusammengesetzte Endung `.tar.gz` | Türkis |
| `tgz` | TGZ | `.tgz` | Türkis |
| `gz` | GZ | Systemkandidat `.gz` | Türkis |
| `cab` | CAB | `.cab` | Stahlblau |
| `iso` | ISO | `.iso` | Seeblau |
| `udf` | UDF | `.udf` | Cyanblau |
| `archive` | Keines | Rückfall für erkannte Archive ohne eigenes Symbol | Neutrales Graublau |

Die Tabelle beschreibt Entwurfsabsicht, keine einheitliche plattformübergreifende Registrierung. Wo möglich wird die vollständige Endung `.tar.gz` vor `.gz` zugeordnet; Systeme mit Erkennung nur der letzten Endung können für beide GZ zeigen. Nicht versprechen, dass alle Plattformen zusammengesetzte Endungen automatisch unterscheiden. Das GZ-Symbol bezeichnet eine gemeinsame Erweiterung von eigenständigem GZIP und äußerem TAR.GZ-Strom; tatsächliches Lesen folgt Inhaltsprüfung und Formatverträgen.

Verschlüsselung und Aufteilung sind Archivzustände und werden nicht aus Erweiterungssymbolen abgeleitet; keine Schlösser oder Erfolgszeichen zeichnen. Erkannte ZIP-, 7z- und RAR-Volumes dürfen innerhalb der Anwendung ihre Familie nutzen, ohne generische `.001`, `.rNN`, `.zNN` oder `.img` zu registrieren. ISO-/UDF-Symbole bedeuten weder Einhängen noch Ausführen. Eine passende Endung allein belegt keinen gültigen Archivinhalt.

Der gewöhnliche Windows-Installer verwendet ProgIDs und ICOs je Erweiterung. MSIX deklariert „Öffnen mit“-Kandidaten mit PNGs aus den entsprechenden ICO-Frames. Beide folgen der generierten Zuordnung, die Grafiken über Endungen hinweg wiederverwenden kann. macOS nutzt ICNS je Dokumenttyp einschließlich komprimiertem TAR und eigenständigem GZIP. Linux nutzt System-MIME-Symbole. Gewählter Standard und Systemsymbolcaches beeinflussen die tatsächliche Anzeige. Siehe [Installation und Zuordnungen](../engineering/installers-associations.md).

MSIX deklariert für jede Formatzuordnung ein separates `uap:Logo`. Dessen Zielgrößen-PNGs verwenden die kanonischen Windows-ICO-Frames bytegleich und bewahren kleine vereinfachte Größen, Formatlabels großer Größen, sichtbare Grenzen und Zentrierung. Anwendungslogos sind separate Ressourcen mit Standard- und beiden ungeplatteten Themenformen. [msix-icons.json](../../../specs/visual/msix-icons.json) definiert Paketgrößen und Varianten; MakePRI indiziert vor MakeAppx-Paketierung. Weder Systemhinterlegung noch Rückfall auf das Anwendungslogo gehören zur vorgesehenen Dokumentgrafik.

Der Windows-Anwendungseintrag in Standard-Apps und „Öffnen mit“ verwendet das [Marken-Anwendungs-ICO](../../../design/assets/brand/xdzip.ico), eingebettet in Hauptprogramm und Zuordnungsstarter. Dokumentsymbole pro Format ersetzen nicht die Anwendungsidentität. Das System soll einen optionalen xdzip-Eintrag mit Markensymbol zeigen; der alte gemeinsame Befehl dient der Kompatibilität vorhandener Auswahlen. Dies nutzt vorhandene Ressourcen und ändert keine logische Hauptfenster-, Menü- oder QML-Geometrie. Native Anwendungssymbole, Dokumentsymbole und Kandidatenzahlen werden getrennt abgenommen; eine generische OS-Anwendungsbox darf nicht als Problem eines Formatdokumentsymbols gelten.

<a name="xdzip-generation-and-delivery"></a>

## Generierung und Lieferung

Im Repository-Stamm ausführen:

```sh
python scripts/build-association-icons.py
python scripts/build-design-assets.py --metadata-only
python scripts/build-association-icons.py --check --check-runtime
# Lokale Artefakte mit vorhandenem Pillow, Playwright und Chromium exportieren.
python scripts/build-association-icons.py --export build/file-association-icons
python scripts/build-association-icons.py --check --export build/file-association-icons
# Nach Kopieren der vorgesehenen ICO/ICNS-Exporte in den kanonischen Ressourcenordner:
python scripts/build-design-assets.py --metadata-only --refresh-association-icons
```

`XDZIP_CHROMIUM` kann einen vorhandenen Chromium-/Edge-Pfad angeben. SVG-Master, Quellen für kleine Größen, und Spezifikation sind versioniert; Ressourcenquellen stehen im Haupt-[manifest.json](../../../design/assets/manifest.json). Der Generierungseinstieg ist [build-association-icons.py](../../../scripts/build-association-icons.py), Kompositionen stehen in [association_icon_assets.py](../../../scripts/association_icon_assets.py).

Exporte enthalten generische und macOS-PNGs von 16 bis 1024 Pixeln pro Format sowie ICO und ICNS. Produkt-ICO und ICNS, vom Formatregister erfasst, liegen neben Vektorquellen in `file-associations/`; ZIP-Container behalten ihre ursprünglichen Bytes vor der Migration, die anderen stammen aus entsprechenden Exporten. Gewöhnliche Builds lesen diese Container direkt und benötigen keinen Browser. Temporäre PNGs, vollständige Exporte, Kontaktbögen und Belege bleiben in `build/` oder `dist/`.

<a name="xdzip-verification-and-acceptance"></a>

## Prüfung und Abnahme

Generierungsprüfungen verifizieren SVG-Übereinstimmung mit der Spezifikation und das Fehlen von Schrift- und externen Bildabhängigkeiten. Plattformprüfungen umfassen Zeichenfläche und RGBA, transparente Ecken, sichtbare Körperhöhe und Zentrierung, Vektordigests und Übereinstimmung dekodierter Pixel jeder ICO-/ICNS-Darstellung. Körpergrenzen werden bei alpha ≥ 128 gemessen, mit der Markentoleranz von 1,25 physischen Pixeln.

Visuelle Prüfung umfasst helle, dunkle und ausgewählte Hintergründe, Größen von 16 bis 256 Pixeln und tatsächliches Rendering bei normalem und hohem DPR. Die native Abnahme erfordert eine Prüfung auf der tatsächlichen Plattform. Spätere Integration muss echte Dateinamen, zusammengesetzte Endungen, Symbolcaches, vor und nach Standardauswahl sowie Anzeige nach Installation und Deinstallation prüfen. Phasenstatus und tatsächlichen Umfang in GitHub-Issues dokumentieren statt hier Testberichte anzusammeln.
