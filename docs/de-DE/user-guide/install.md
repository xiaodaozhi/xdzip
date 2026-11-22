# XDZip installieren und ein Archiv öffnen

[English](../../user-guide/install.md) | [简体中文](../../zh-CN/user-guide/install.md) | [繁體中文](../../zh-TW/user-guide/install.md) | [日本語](../../ja-JP/user-guide/install.md) | [한국어](../../ko-KR/user-guide/install.md) | [Français](../../fr-FR/user-guide/install.md) | Deutsch | [Español](../../es-ES/user-guide/install.md) | [Português (Brasil)](../../pt-BR/user-guide/install.md) | [Русский](../../ru-RU/user-guide/install.md)

Laden Sie das Paket für Ihr System von der [letzten stabilen Version auf GitHub](https://github.com/xiaodaozhi/xdzip/releases/latest) herunter oder verwenden Sie unter Windows den Microsoft Store. Die GitHub-Seite enthält Dateiname, Version und `SHA256SUMS` zur Prüfung direkt heruntergeladener Dateien. Die Systemanforderungen stehen in den [Angaben zu unterstützten Systemen](../engineering/support-policy.md).

| Betriebssystem | Installieren oder starten |
| --- | --- |
| Windows 11 x64 | Suchen Sie im Microsoft Store nach **XDZip**, oder führen Sie die GitHub-Datei `-setup.exe` für eine Installation im Benutzerkonto aus. Das GitHub-Installationsprogramm ist nicht signiert. Für die portable ZIP-Version entpacken Sie das gesamte Paket und starten `xdzip.exe` aus dem entpackten Ordner. |
| macOS 14 oder neuer, Apple Silicon | Öffnen Sie die DMG-Datei und ziehen Sie `xdzip.app` in **Programme**. Entpacken Sie eine `.app.zip` zuerst und verschieben Sie die App dann dorthin. Starten Sie die Kopie aus dem Ordner Programme. GitHub-Builds haben eine Ad-hoc-Integritätssignatur und sind nicht notarisiert. Prüfen Sie bei einer macOS-Warnung die Downloadquelle und die Prüfsumme. |
| Ubuntu 24.04 LTS x64, X11, glibc ab 2.39 | Installieren Sie das `.deb` über die Paketverwaltung. Bei der portablen `.tar.gz`-Version entpacken Sie alle Dateien und starten `./xdzip` aus dem entpackten Laufzeitordner. Lassen Sie die Startdatei bei den Laufzeitdateien. |

Die Desktopversion unterstützt Windows ARM64, Intel-Macs, Linux ARM64 und Wayland nicht. Die eigenständige Befehlszeilenversion unter Linux funktioniert ohne X11 und benötigt OpenSSL 3 sowie die Systemzertifikate. Wählen Sie dazu eine Datei mit dem Namensanfang `xdzip-cli-` und folgen Sie der [Kurzanleitung für die Befehlszeile](../integration/README.md).

Nach der Installation des DEB-Pakets können Sie XDZip im Ubuntu App Center unter den installierten Anwendungen finden und entfernen.

<a name="xdzip-open-an-archive"></a>

## Ein Archiv öffnen

Wählen Sie **Archiv öffnen** in XDZip oder ziehen Sie das Archiv auf die Startseite. Im Dateimanager können Sie **Öffnen mit** verwenden, sofern das installierte Paket registriert ist. Um XDZip als Standard-App für einen Dateityp einzurichten, wählen Sie es in den Systemeinstellungen aus. In der Microsoft-Store-Version steht auf der Seite für Dateizuordnungen **Windows-Standard-Apps öffnen** zur Verfügung. Standard-Apps und Komprimierungsaktionen im Kontextmenü werden getrennt eingestellt. Siehe [Entpacken aus dem Dateimanager](extract.md#xdzip-from-the-file-manager) und [Installation und Dateizuordnungen](../engineering/installers-associations.md).

Wenn macOS XDZip weiterhin als einzige Anwendung für einen Dateityp führt, erscheint beim Ausschalten der Zuordnung ein Hinweis und XDZip bleibt die Standardanwendung.

Als Nächstes können Sie [Dateien durchsuchen und in der Vorschau anzeigen](browse.md).
