# Archive prüfen und Aufgaben verwalten

[English](../../user-guide/checks-and-tasks.md) | [简体中文](../../zh-CN/user-guide/checks-and-tasks.md) | [繁體中文（台灣）](../../zh-TW/user-guide/checks-and-tasks.md) | [日本語](../../ja-JP/user-guide/checks-and-tasks.md) | [한국어](../../ko-KR/user-guide/checks-and-tasks.md) | [Français](../../fr-FR/user-guide/checks-and-tasks.md) | Deutsch | [Español](../../es-ES/user-guide/checks-and-tasks.md) | [Português (Brasil)](../../pt-BR/user-guide/checks-and-tasks.md) | [Русский](../../ru-RU/user-guide/checks-and-tasks.md)

<a name="xdzip-test-an-archive"></a>

## Ein Archiv prüfen

1. Öffnen Sie das Archiv und wählen Sie in der Symbolleiste **Prüfen**.
2. Geben Sie bei Aufforderung das Passwort ein und warten Sie auf das Ergebnis. Die Aufgabe prüft unterstützte Inhalte, ohne entpackte Dateien zu schreiben.
3. Lesen Sie bei einem Fehler die angezeigte Ursache und prüfen Sie gegebenenfalls den betroffenen Eintrag. Manche Formate bieten Prüfsummen oder Authentifizierung, während andere die Struktur und Lesbarkeit prüfen. Eine erfolgreiche Prüfung gilt nur für die vom Format vorgesehenen Kontrollen. [Integritätsprüfung](../engineering/archive-checking.md)

Einige Archive lassen sich nicht prüfen. Ein ZIP-Archiv mit Index auf der Festplatte erlaubt beispielsweise das Entpacken ausgewählter Dateien innerhalb der Aufgabengrenzen. Bei NSIS stehen nur Installationsmetadaten zur Verfügung. Siehe [Grenzen für große Archive](../engineering/large-index.md) und [unterstützte Formate](../../../README.de-DE.md#xdzip-format-support).

<a name="xdzip-progress-cancellation-and-results"></a>

## Fortschritt, Abbruch und Ergebnisse

Öffnen Sie das Aufgabenmenü, um laufende Aufgaben oder Ergebnisse aufzurufen. Wenn Sie einen Archiv-Tab schließen, läuft seine Hintergrundaufgabe weiter. Passwort- oder Konfliktdialoge können eine Antwort erfordern, bevor die Verarbeitung fortgesetzt wird.

Wählen Sie im Aufgabenfenster **Abbrechen**, um einen Stopp anzufordern, und warten Sie auf den Endstatus. Bereits endgültig gespeicherte Dateien können erhalten bleiben. Klappen Sie die Ergebnisdetails auf, um abgeschlossene, übersprungene und noch nicht fertiggestellte Einträge zu unterscheiden. Entscheiden Sie anhand der Anzahlen und Ursachen, was Sie erneut ausführen möchten. Das Löschen des Verlaufs abgeschlossener Aufgaben entfernt deren Aufgabeneinträge. Die Dateien bleiben auf der Festplatte. [Aufgabenfenster](../ux/task-windows.md)
