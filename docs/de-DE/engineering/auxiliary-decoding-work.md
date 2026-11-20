<a name="xdzip-auxiliary-algorithm-work-accounting"></a>

# Abrechnung zusätzlicher Algorithmusarbeit

[English](../../engineering/auxiliary-decoding-work.md) | [简体中文](../../zh-CN/engineering/auxiliary-decoding-work.md) | [繁體中文](../../zh-TW/engineering/auxiliary-decoding-work.md) | [日本語](../../ja-JP/engineering/auxiliary-decoding-work.md) | [한국어](../../ko-KR/engineering/auxiliary-decoding-work.md) | [Français](../../fr-FR/engineering/auxiliary-decoding-work.md) | Deutsch | [Español](../../es-ES/engineering/auxiliary-decoding-work.md) | [Português (Brasil)](../../pt-BR/engineering/auxiliary-decoding-work.md) | [Русский](../../ru-RU/engineering/auxiliary-decoding-work.md)

Entpacken und Prüfen zählen dekodierte Dateibytes, tatsächliche Ausgabe und zusätzliche Algorithmusarbeit getrennt. PPM-Modellinitialisierung, Neustarts wegen Kapazitätsgrenzen und Dekodierung von Steuerdaten oder Parametern sind kein Dateiinhalt. Sie dürfen `decoded_bytes`, `written_bytes`, Dateifortschritt und Expansionsverhältnis nicht erhöhen und kein Guthaben für vorab angerechnete Ausgabe erzeugen. Aufgabensnapshots haben keinen sichtbaren Zusatzarbeitszähler.

`DecodedSink::charge_auxiliary(bytes)` meldet nur bereits geleistete Zusatzarbeit. Standardmäßig bleiben Dateizähler unverändert. Algorithmen lassen geplante Arbeit zunächst anhand ihres Restbudgets zu. Nach der Ausführung rechnen sie die tatsächlich geleistete Arbeit ab, auch vor Fehler oder Abbruch. Nicht ausgeführte Reservierungen werden nicht als Arbeit gemeldet. Fehler, Modellrücksetzung und erneute Passworteingabe erstatten keine erledigte Arbeit. Das private RAR-Gesamtbudget begrenzt weiter die Summe aus Dateidekodierung und Zusatzarbeit.

<a name="xdzip-isolation-boundary"></a>

## Isolationsgrenze

- `AUX_WORK=24` überträgt einen Little-Endian-`u64`-Zähler ohne Inhaltsdaten. Jedes Paket enthält einen Wert von 1 bis 65.536. Der Frame muss genau dort enden. Nach erfolgreicher Abrechnung antwortet der Host mit einem leeren `ACK`. Null, gekürzte Pakete, nachgestellte Felder, überhöhte Werte und ungültige Phasen werden zurückgewiesen.
- Diese Nachricht ist ausschließlich für RAR-Nutzdatendekodierung zulässig. Metadatenscan, Erstellung, Änderung und andere Formate verwenden sie nicht. CXX überträgt Werte synchron und behält keine Rust-Zeiger. Quell- und Ausgabeslices werden nur während des Aufrufs ausgeliehen. C++-Ausnahmen werden innerhalb von Rückrufen in stabile Fehler übersetzt. Rust-Unwinding überschreitet die CXX-Grenze nicht. Abbruch oder Prozessende lässt unfertige Ausgabe fehlschlagen.
- Eine native Nutzdateninstanz zählt maximal 20 GiB Zusatzarbeit und behält den Gesamtwert über Einträge, Prozessneustarts und Passwortänderungen hinweg. Abgeschlossene Arbeit wird vor dem Aufruf des Host-Sinks erfasst. Lehnt der Sink ab, wird bei einem erneuten Versuch nichts erstattet. `AUX_WORK` erhöht `WorkerCredit` nicht.
- Entpack- und Prüfaufgaben der Laufzeit zählen unabhängig bis zu 20 GiB Zusatzarbeit. Das Abrechnungsobjekt liegt außerhalb der Schleifen für Einträge und Passwortwiederholungen. Das Budget wird getrennt von tatsächlichen Dateibytes und Ausgabelimits geprüft. Ein Limitfehler oder Abbruch meldet einen Fehler, ohne eine unfertige Datei zu veröffentlichen. Änderungen öffentlicher Zähler umgehen die Grenzen nicht.
- `WORK=17` steht weiterhin für dekodierte Dateibytes. Nur seine erfolgreiche Zulassung gewährt demselben noch gültigen Prozess einmaliges Guthaben für `PRECHARGED_DATA=23`. Guthaben wird vor Ausgabe abgezogen und beim Neustart gelöscht. RAR, 7z und CAB nutzen diesen Pfad. Scan und Erstellung nutzen ihn nicht. Zusatzarbeit kann damit keine Dateiausgabe finanzieren.

Die Zulassung hoher Kompressionsverhältnisse nutzt die fixierten deklarierten Größen und die gepackten oder gesamten Archivgrößen. Limits der Laufzeit für Dateidekodierung und Ausgabe gelten unabhängig. Benötigt eine 87-Byte-Datei mehrere MiB Modellinitialisierung, zählen die Dateizähler weiterhin nur ihre tatsächlichen 87 Byte.

<a name="xdzip-required-acceptance-conditions"></a>

## Erforderliche Abnahmebedingungen

- Eine normale kleine Datei kann nach mehr als 64 KiB Zusatzarbeit und Modellinitialisierung erfolgreich entpackt und geprüft werden. Dateizähler bleiben exakt und die Inhaltsprüfung gelingt.
- Native Schicht und Laufzeit prüfen jeweils die Rückweisung, wenn wiederholte tatsächliche Zusatzpakete 20 GiB erreichen. Wiederholte Passworteingabe und Prozesswechsel setzen die Summe nicht zurück. Unfertige Dateien werden nicht übernommen.
- Anforderungen eines Testprozesses für andere Formate, Metadatenphasen, leere oder zu große Pakete, nachgestellte Felder und allein durch Zusatzarbeit gedeckte Vorausausgabe schlagen fehl.
- Reguläres `WORK` kann vorab angerechnete RAR-Ausgabe desselben Prozesses finanzieren. Guthaben ist einmalig und kann nach Fehler oder Neustart nicht wiederverwendet werden.
- Protokoll-Testdateien prüfen Zustandsautomat und Abrechnung. Der vollständige RAR-Prozess in Rust, der Laufzeitpfad, die drei Plattformen und Desktopabnahme erfordern eigene Prüfnachweise.

Die Protokollfixture kompiliert nur das eigene `tools/native-codec/main.cpp` und den zugehörigen Stub ohne Kompressionsalgorithmus.

```sh
cmake -S crates/xdzip-codec-native/tests -B build/auxiliary-protocol
cmake --build build/auxiliary-protocol --config Release
```

`XDZIP_NATIVE_CODEC_WORKER` auf die Fixture setzen, `XDZIP_AUXILIARY_PROTOCOL_FIXTURE=1` einstellen und ausdrücklich `cargo test -p xdzip-codec-native auxiliary_protocol -- --ignored` ausführen. Der gewöhnliche gezielte Test ist `cargo test -p xdzip-codec-native -p xdzip-runtime auxiliary`. Tests der Codec-API prüfen die Kompatibilität des Standardsinks. Erzeugte Nachweise bleiben unter build oder in CI. Ergebnisse und offene Punkte stehen in [Migrationsaufgabe #51](https://github.com/xiaodaozhi/xdzip/issues/51).
