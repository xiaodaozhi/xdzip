# Passwörter und mehrteilige Archive

[English](../../user-guide/passwords-and-volumes.md) | [简体中文](../../zh-CN/user-guide/passwords-and-volumes.md) | [繁體中文](../../zh-TW/user-guide/passwords-and-volumes.md) | [日本語](../../ja-JP/user-guide/passwords-and-volumes.md) | [한국어](../../ko-KR/user-guide/passwords-and-volumes.md) | [Français](../../fr-FR/user-guide/passwords-and-volumes.md) | Deutsch | [Español](../../es-ES/user-guide/passwords-and-volumes.md) | [Português (Brasil)](../../pt-BR/user-guide/passwords-and-volumes.md) | [Русский](../../ru-RU/user-guide/passwords-and-volumes.md)

<a name="xdzip-passwords"></a>

## Passwörter

AES Crypt und Archive mit verschlüsselten Dateinamen, etwa ein 7z-Archiv mit Namensverschlüsselung, benötigen vor dem Anzeigen der Dateiliste ein Passwort. Andere Archive können beim Entpacken, Prüfen oder Öffnen der Vorschau danach fragen. Beachten Sie bei der Eingabe Leerzeichen sowie Groß- und Kleinschreibung. ZIP AES-256 wird in unterstützten Profilen verarbeitet. ZIP AES-128/192 und klassisches ZipCrypto werden nicht unterstützt. Schlägt die Verarbeitung mit dem richtigen Passwort weiterhin fehl, prüfen Sie, ob die Verschlüsselung unterstützt und die Datei beschädigt ist. [Unterstützte Verschlüsselung](../engineering/password-extraction.md)

Wählen Sie für ein passwortgeschütztes Archiv ein Format mit Verschlüsselung. ZIP AES-256 schützt den Dateiinhalt, während Dateinamen sichtbar bleiben. Soll auch die Dateiliste geschützt sein, wählen Sie 7z und aktivieren Sie die Namensverschlüsselung. Bewahren Sie das Passwort getrennt vom Archiv auf.

<a name="xdzip-split-volumes"></a>

## Mehrteilige Archive

1. Legen Sie sämtliche Teile desselben Archivs in einen Ordner und behalten Sie ihre ursprünglichen Namen bei.
2. Öffnen Sie einen unterstützten Teil. XDZip sucht die übrigen Teile im selben Ordner. Nummerierte ZIP- und 7z-Teile, PKZIP sowie unterstützte RAR- und CAB-Sätze folgen unterschiedlichen Namensregeln.
3. Fehlen Teile, beschaffen Sie diese und öffnen Sie das Archiv erneut. Manuelles Zusammenfügen oder Umbenennen kann die Teile unlesbar machen.

Wählen Sie für ein mehrteiliges Archiv ZIP oder 7z und legen Sie im Erstellungsfenster die Größe der Teile fest. Geben Sie den vollständigen Satz weiter. Die Aufteilung macht ein Archiv in XDZip nicht bearbeitbar. [Benennung und unterstützte Profile](../engineering/archive-opening.md)
