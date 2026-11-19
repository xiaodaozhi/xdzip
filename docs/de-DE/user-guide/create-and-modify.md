# Archive erstellen und ändern

[English](../../user-guide/create-and-modify.md) | [简体中文](../../zh-CN/user-guide/create-and-modify.md) | [繁體中文（台灣）](../../zh-TW/user-guide/create-and-modify.md) | [日本語](../../ja-JP/user-guide/create-and-modify.md) | [한국어](../../ko-KR/user-guide/create-and-modify.md) | [Français](../../fr-FR/user-guide/create-and-modify.md) | Deutsch | [Español](../../es-ES/user-guide/create-and-modify.md) | [Português (Brasil)](../../pt-BR/user-guide/create-and-modify.md) | [Русский](../../ru-RU/user-guide/create-and-modify.md)

<a name="xdzip-create-an-archive"></a>

## Ein Archiv erstellen

1. Öffnen Sie das Erstellungsfenster auf der Startseite oder über **Neu**. Fügen Sie Dateien oder Ordner hinzu und prüfen Sie die Liste.
2. Legen Sie Name und Speicherort fest. Wählen Sie ein Format und eine verfügbare Kompressionseinstellung. ZIP ist für die Weitergabe weit verbreitet. Die [Formattabelle](../../../README.de-DE.md#xdzip-format-support) zeigt die Erstellungsoptionen. Ein Ein-Datei-Datenstrom wie GZIP nimmt eine reguläre Datei auf.
3. ZIP und 7z bieten je nach gewählten Optionen Verschlüsselung und mehrteilige Archive. Wählen Sie ISO für ein Datenabbild oder das SFX-Profil für Windows x64 für ein selbstentpackendes ZIP. Starten Sie den Vorgang und prüfen Sie im Ergebnis, welche Dateien tatsächlich gespeichert wurden. [Passwörter und mehrteilige Archive](passwords-and-volumes.md)
4. Prüfen Sie vor einem Abbruch oder erneuten Versuch zuerst das Ergebnis. Das Archiv wird vor der Freigabe des fertigen Ergebnisses in einer temporären Datei erstellt.

<a name="xdzip-add-replace-or-move-files"></a>

## Dateien hinzufügen, ersetzen oder verschieben

Öffnen Sie ein geeignetes ZIP-, 7z-, TAR- oder TAR.GZ-Archiv und wechseln Sie darin zum Zielordner. Ziehen Sie lokale Dateien oder Ordner auf die Liste oder das Raster und prüfen und bestätigen Sie das Hinzufügen. Zu den Konfliktoptionen gehört das Ersetzen geeigneter regulärer Dateien im Archiv. Ziehen Sie einen vorhandenen Eintrag zum Verschieben in einen anderen Ordner desselben Archivs und bestätigen Sie. Beim Ziehen aus dem Fenster wird eine Kopie exportiert. [Ziehen und Ablegen je nach Plattform](../engineering/archive-transfers.md)

Beim Erstellen und Hinzufügen werden `.DS_Store`-Dateien und die Verzeichnisse `__MACOSX` und `_MACOSX` ausgelassen. Ob ein Archiv geändert werden kann, hängt von Format, Inhalt und Quelle ab. Verschlüsselte, mehrteilige und auf der Festplatte indizierte Archive lassen sich auf diesem Weg nicht bearbeiten. Ist die Bearbeitung nicht möglich, entpacken Sie in einen neuen Ordner und erstellen Sie ein neues Archiv. [Regeln für Änderungen und Ziehen und Ablegen](../engineering/archive-transfers.md)
