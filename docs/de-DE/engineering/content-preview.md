<a name="xdzip-content-preview"></a>

# Inhaltsvorschau

[English](../../engineering/content-preview.md) | [简体中文](../../zh-CN/engineering/content-preview.md) | [繁體中文](../../zh-TW/engineering/content-preview.md) | [日本語](../../ja-JP/engineering/content-preview.md) | [한국어](../../ko-KR/engineering/content-preview.md) | [Français](../../fr-FR/engineering/content-preview.md) | Deutsch | [Español](../../es-ES/engineering/content-preview.md) | [Português (Brasil)](../../pt-BR/engineering/content-preview.md) | [Русский](../../ru-RU/engineering/content-preview.md)

Ist die Eigenschaftenkarte sichtbar, beginnt nach Stabilisierung der Auswahl die Vorschau einer einzelnen unterstützten Datei. Das Aktualisierungssymbol erscheint oben rechts, wenn der Zeiger über der Vorschau liegt oder die Schaltfläche Tastaturfokus erhält; es lädt die ausgewählte Datei neu. Die Karte meldet Lese- oder Dekodierfehler und zeigt bei Mehrfachauswahl einen Auswahlhinweis. Hauptfenster und eigenständige Eigenschaftenansicht teilen dieses Verhalten und das vorhandene Passwortfenster. Dateimetadaten bleiben unabhängig verfügbar.

<a name="xdzip-capabilities-and-budgets"></a>

## Fähigkeiten und Budgets

Unterstützt werden nur lesbare reguläre Dateien bekannter Größe ohne gefährliche Namen. Die Liste erlaubter Texterweiterungen umfasst txt, md, log, json, csv, tsv, toml, yaml, yml, ini, xml, rs, c, cpp, h und py. Zulässig sind nur striktes UTF-8 oder UTF-16 mit BOM. Eingabe und konvertiertes UTF-8 sind jeweils auf 256 KiB begrenzt; binäre Steuerbytes wie NUL sind verboten. Der Inhalt erscheint in einem PlainText-Steuerelement; Tags, URLs, Programmtext und Makros werden nicht ausgeführt.

| Bildtyp | Umfang und tatsächliche Semantik auf drei Plattformen |
| --- | --- |
| PNG, JPEG (jpg, jpeg, jfif, jpe) | Ein statisches Bild vollständig lesen und verfügbare Transformationsmetadaten für die Ausrichtung anwenden |
| GIF, WebP | Statisches Bild oder erster Animationsframe; Animationen werden nicht abgespielt |
| BMP, ICO/CUR | Gewöhnliches oder vom Decoder standardmäßig gewähltes Bild; CUR aktiviert keinen Systemcursor |
| ICNS | Eingebettete PNG- oder JPEG2000-Bilder, die ein verfügbarer Systemdecoder unterstützt, höchstens 256 Blöcke; unbekannte eingebettete Typen und alte Rohbitmap-/RLE-Symbole werden ausdrücklich abgelehnt |
| TGA | Unkomprimierte True-Color-Bilder mit 16/24/32 Bit ohne Farbpalette oder Bild-ID und Ursprung links; fehlenden alten Footer für dasselbe Layout ergänzen; kein RLE oder indizierte Farbe |
| TIFF (tif, tiff) | Nur erste Seite; Seiten werden nicht automatisch durchlaufen |
| SVG | Statischer UTF-8-Vektor, nach eingeschränktem Parsen gerastert; siehe Untermenge unten |
| HEIC/HEIF | Nur macOS mit offiziellem Qt-Plugin und System-ImageIO; anderswo ausdrücklich abgelehnt |

AVIF, RAW, PSD, EXR, PDF, Office und unabhängige Decoderbibliotheken Dritter werden nicht hinzugefügt. Dateiendung, Rust-Prüfung der Formatsignatur und festgelegte Qt-Decoder bestimmen gemeinsam die Zulassung. Zusätzlich systemweit installierte Plugins werden nicht zu Produktfähigkeiten; andere Decoder werden nicht automatisch sondiert.

Kodierter Bildinhalt ist auf 16 MiB begrenzt. Rust-Ereigniswarteschlangen und Qt-Mailbox haben jeweils 32 MiB Grenze; das Befehlsbudget bleibt 8 MiB. Originalbreite und -höhe betragen jeweils höchstens 16.384, insgesamt höchstens 33.554.432 Pixel. Quellzulassung und Ausgabeminiatur werden getrennt abgerechnet: endgültige Breite und Höhe jeweils höchstens 512, Seitenverhältnis erhalten und RGBA8 höchstens 1 MiB. Fotos wie 12-MP-Bilder können innerhalb der Budgets als Miniaturen gelesen werden. Manche Decoder lesen vor der Skalierung das Original; eine Miniaturgröße beweist daher keine interne Nutzung von nur 1 MiB. Verfügbare Farbprofile werden in sRGB umgewandelt; unbekannte Profile behalten das Decoderergebnis. Professionelle Farbprüfung wird nicht zugesagt.

Der asynchrone Qt-Bildanbieter erhält nur zufällige Token, keine Systempfade oder Netzwerk-URLs. Global werden höchstens 32 MiB kodierter Inhalt gehalten; höchstens zwei Dekodieraufgaben laufen parallel. Abgebrochene, noch in Bereinigung befindliche Aufgaben dürfen jeweils höchstens eine Eingabe von 16 MiB halten. Jede Aufgabe startet den festgelegten benachbarten privaten Worker `xdzip-image-preview`, ohne Archivpfade, Passwörter, Zielhandles oder Anwendungszugangsdaten weiterzugeben. Die Workerausgabe erlaubt nur einen Header fester Größe und RGBA8 mit strikt passenden Breiten-, Höhen- und Längenwerten, ohne zusätzliche Ausgabe. Fehler, Absturz, ungültige Frames, Zeitüberschreitung und verspätete Ergebnisse dürfen nicht angezeigt werden.

Der Host setzt eine Gesamtfrist von fünf Sekunden ab Start und prüft die Gültigkeit der Anfrage alle 50 ms. Schließen, Abbruch oder Eintragswechsel beendet und räumt den Worker ab. Der Worker begrenzt QImage-Allokation auf 128 MiB. Windows verwendet einen Job mit 512 MiB Commit-Speicher des Prozesses und drei Sekunden Benutzer-CPU. Unix verwendet drei Sekunden CPU, deaktiviert Core-Dumps und begrenzt Dateideskriptoren. Linux begrenzt den Adressraum auf 512 MiB und reguläre Dateigröße auf null. macOS fügt über bestehende virtuelle dyld-Reservierungen hinaus 512 MiB Adressraum hinzu; dies ist keine harte RSS-Grenze. System-ImageIO kann dateibasierte Laufzeitressourcen nutzen, daher ist jede reguläre Datei auf 16 MiB begrenzt; Überschreitungen beenden die Dekodierung weiterhin. Die Anwendung exportiert keine Kopie des Archiveintrags; Systemlaufzeitressourcen und Eintragsausgabe werden getrennt verwaltet. Dies ist Absturz- und Ressourcenisolation eines privaten Decoders, keine Kernel-Berechtigungssandbox für beliebige Drittprogramme.

Jedes Lesen von Archivinhalten ist auf kumuliert 256 MiB Quelllesevorgänge, 64 MiB Dekodier- und Überspringarbeit sowie 256 MiB zusätzliche algorithmische Arbeit begrenzt. Formatwörterbücher, Worker und Methodengraphen unterliegen weiterhin getrennten Budgets des [Formatbackend-Vertrags](native-codecs.md). Ein solides Archiv oder sequenzieller komprimierter Strom kann die Vorschau einer kleinen Datei ablehnen, wenn vorausgehende Dekodierung das Arbeitsbudget überschreitet.

<a name="xdzip-static-svg-subset"></a>

## Statische SVG-Untermenge

SVG verwendet auf allen drei Desktopplattformen denselben Qt-Svg-Renderpfad. Vor dem Rendern rekonstruiert ein Streaming-XML-Parser unterstützte statische Grafik, Text, Stile, interne Definitionen, Verläufe, Beschneidungen, Masken und eingeschränkte Filter. Grenzen sind 8.192 Elemente, 64 Ebenen, 64 Attribute pro Element, 1 MiB pro Attribut und 16 MiB Gesamtausgabe. Metadaten nehmen nicht am Rendering teil. Stilelemente akzeptieren nur Text; XML-Textsegmente und CDATA vor Prüfung des vollständigen CSS kombinieren, damit Segmentierung externe Referenzbeschränkungen nicht umgehen kann.

DTDs, Entitäten, Verarbeitungsanweisungen, Skripte, Ereignisse, Animation, foreignObject, image-Elemente, Schriftdateien, externe Stile und externe Ressourcenreferenzen ablehnen. href und CSS url akzeptieren nur einfache Fragmentidentitäten im aktuellen Dokument. xml:base, src, CSS-Imports und maskierte Formen sind verboten. Eingebettete Bitmaps und SVGZ liegen außerhalb der Untermenge. Unbekannte Zeichenelemente oder verbotener Inhalt lassen die gesamte Vorschau fehlschlagen, statt still verworfen und als vollständig dargestellt zu werden. Dies unterstützt gewöhnliches statisches SVG ohne alle SVG- oder Browserfähigkeiten zu behaupten.

<a name="xdzip-integrity-passwords-and-lifecycle"></a>

## Integrität, Passwörter und Lebenszyklus

Die Vorschau verwendet eingefrorene physische Identität, Quellversion, Nutzdatenposition und vorhandene Decoder. Inhalt muss bis EOF vollständig dekodiert sein und die verfügbaren Integritäts- und Authentifizierungsprüfungen des Formats bestehen, bevor Text oder kodierte Bilder zur Vorschau gelangen. Formate wie TAR ohne Inhaltsprüfsummen pro Datei behalten nur ihre bestehenden Struktur- und Lesbarkeitsgarantien. Große festplattengestützte Indizes lesen ausgewählte Nutzdaten begrenzt, ohne alle Metadaten nach Qt zu laden.

Passwörter gelten nur für die aktuelle Anfrage und verwenden den bestehenden nullenden Container; Fehler, Authentifizierungsmehrdeutigkeit und Abbruch folgen bestehenden Kategorien mit höchstens zehn Versuchen. Schließen des Passwortfensters bricht nur die Vorschau ab und erhält das nutzbare Verzeichnis. Öffnen, Navigation und andere Vordergrund-Lesebefehle haben Vorrang vor einer ausstehenden Vorschaulesung. Vorschau schließen, Eintrag wechseln, Archivseite verbergen oder Sitzung schließen leert den Inhalt; ungültige Anfragen und Decodergebnisse dürfen alte Inhalte nicht wiederherstellen. Fehler und Abbruch verwerfen ungeprüfte Teilinhalte, schreiben keine Eintragskopie und starten keine Systemanwendung. Temporäres Öffnen per Doppelklick folgt seinem [separaten Ablauf](archive-opening.md).

<a name="xdzip-toolchain-and-acceptance"></a>

## Toolchain und Abnahme

`desktop.py prepare --install-qt` ergänzt das festgelegte qtimageformats von Qt 6.8.2 in neuen oder vorhandenen SDKs; doctor prüft Svg und erforderliche Plugins vor dem Build. Offizielle Quellen, Digests, Hinweise und SDK-Zuordnung stehen in gemeinsamer Konfiguration. Von Qt mitgelieferte libwebp/libtiff und separat hinzugefügte Formatbibliotheken werden unabhängig verwaltet. `check-image-preview.py` prüft tatsächlichen Worker, Plugins und minimale Dekodierung in Standardtests, Windows-/Linux-Bereitstellungsprüfung und autorisierter Mac-Paketierung. Mac schließt offscreen ausdrücklich ein und entfernt SDK-Suchüberschreibungen vor dem Selbsttest; Pluginverzeichnisse des Entwicklungs-SDKs ersetzen keine Prüfung des tatsächlichen Laufzeitpakets.

Regressionen nutzen projekteigene Bilder für jeden Typ, erste Animationsframes und TIFF-Seiten, Transparenz, 12-MP-Fotos, macOS HEIC und Ablehnung auf anderen Plattformen, interne SVG-Definitionen und Ablehnung externer Ressourcen, beschädigte Eingaben, Pixelüberschreitungen, ungültige Frames, Zeitüberschreitungen und Abbruch. Echte Archivpfade decken weiterhin Passwörter, Integrität, Quellidentität und UI-Invalidierungsbereinigung ab. Das Layout folgt dem [visuellen Standard](../../visual/design-standard.md#content-preview); unveränderte Layoutnachweise gemäß [Prüfung](../../../developers/qa/README.md) wiederverwenden und tatsächliche Anzeige neu hinzugefügter Bilder beobachten.
