# Installer XDZip et ouvrir une archive

[English](../../user-guide/install.md) | [简体中文](../../zh-CN/user-guide/install.md) | [繁體中文](../../zh-TW/user-guide/install.md) | [日本語](../../ja-JP/user-guide/install.md) | [한국어](../../ko-KR/user-guide/install.md) | Français | [Deutsch](../../de-DE/user-guide/install.md) | [Español](../../es-ES/user-guide/install.md) | [Português (Brasil)](../../pt-BR/user-guide/install.md) | [Русский](../../ru-RU/user-guide/install.md)

Téléchargez le paquet de votre système depuis la [dernière version stable sur GitHub](https://github.com/xiaodaozhi/xdzip/releases/latest) ou utilisez le Microsoft Store sous Windows. La page GitHub indique le nom et la version du fichier et fournit `SHA256SUMS` pour vérifier les téléchargements directs. Consultez la [politique de prise en charge](../engineering/support-policy.md) pour connaître la configuration requise.

| Système | Installation ou lancement |
| --- | --- |
| Windows 11 x64 | Recherchez **XDZip** dans le Microsoft Store ou exécutez le fichier GitHub `-setup.exe` pour une installation par utilisateur. Cet installateur n’est pas signé. Pour la version ZIP portable, extrayez tout le paquet et lancez `xdzip.exe` depuis le dossier obtenu. |
| macOS 14 ou ultérieur, Apple Silicon (arm64), Intel (x86_64) à partir de la version 0.3.2 | Choisissez `macos-arm64` pour Apple Silicon ou `macos-x86_64` pour Intel. Ouvrez le DMG et faites glisser `xdzip.app` dans **Applications**. Pour un fichier `.app.zip`, extrayez-le d’abord et déplacez-y l’application. Lancez la copie qui se trouve dans Applications. Les versions GitHub ont une signature d’intégrité ad hoc et ne sont pas notarisées. Si macOS affiche un avertissement, vérifiez la provenance du fichier et sa somme de contrôle. |
| Ubuntu 24.04 LTS x64, X11, glibc 2.39 ou ultérieur | Installez le paquet `.deb` avec votre gestionnaire de paquets. Pour la version portable `.tar.gz`, extrayez tous les fichiers et exécutez `./xdzip` depuis le dossier obtenu. Gardez le lanceur avec les fichiers d’exécution. |

L’application de bureau ne prend pas en charge Windows ARM64, Linux ARM64 ni Wayland. L’interface en ligne de commande autonome sous Linux fonctionne sans X11 et nécessite OpenSSL 3 et les certificats du système. Choisissez un fichier dont le nom commence par `xdzip-cli-` et suivez le [guide de démarrage en ligne de commande](../integration/README.md).

Après l’installation du paquet DEB, vous pouvez retrouver et désinstaller XDZip depuis la liste des applications installées dans le Centre d’applications d’Ubuntu.

<a name="xdzip-open-an-archive"></a>

## Ouvrir une archive

Choisissez **Ouvrir une archive** dans XDZip ou déposez un fichier sur la page d’accueil. Dans le gestionnaire de fichiers, choisissez **Ouvrir avec** si le paquet installé a enregistré cette action. Pour définir XDZip comme application par défaut pour un type de fichier, sélectionnez-le dans les réglages système. L’édition du Microsoft Store propose **Ouvrir les applications par défaut de Windows** sur sa page d’associations de fichiers. Les applications par défaut et les actions de compression du menu contextuel se règlent séparément. Voir [l’extraction depuis le gestionnaire de fichiers](extract.md#xdzip-from-the-file-manager) et [l’installation et les associations](../engineering/installers-associations.md).

Si macOS continue de répertorier XDZip comme seule application capable d’ouvrir un type de fichier, la désactivation de l’association affiche un message et le système conserve XDZip par défaut.

Étape suivante, [parcourir et prévisualiser les fichiers](browse.md).
