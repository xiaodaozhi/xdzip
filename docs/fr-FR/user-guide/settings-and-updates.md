# Paramètres et mises à jour

[English](../../user-guide/settings-and-updates.md) | [简体中文](../../zh-CN/user-guide/settings-and-updates.md) | [繁體中文（台灣）](../../zh-TW/user-guide/settings-and-updates.md) | [日本語](../../ja-JP/user-guide/settings-and-updates.md) | [한국어](../../ko-KR/user-guide/settings-and-updates.md) | Français | [Deutsch](../../de-DE/user-guide/settings-and-updates.md) | [Español](../../es-ES/user-guide/settings-and-updates.md) | [Português (Brasil)](../../pt-BR/user-guide/settings-and-updates.md) | [Русский](../../ru-RU/user-guide/settings-and-updates.md)

<a name="xdzip-set-up-your-workspace"></a>

## Configurer l’espace de travail

Ouvrez les **Paramètres** depuis le menu de l’application ou la barre d’outils. Choisissez la langue, le thème, la taille du texte et la densité de l’interface dans les réglages correspondants. La détection automatique utilise la langue du système si elle est prise en charge. Choisissez si les archives s’ouvrent dans des onglets ou des fenêtres. Vous pouvez également régler l’historique récent et l’icône de notification lorsque ces options sont disponibles. Les modifications prennent effet immédiatement. Si un réglage ne peut pas être enregistré, consultez le message sur la page. [Fonctionnement des paramètres](../ux/settings-localization.md)

Changez les associations de fichiers dans le système. Les actions du menu contextuel se règlent séparément. Sous macOS, le système gère l’activation de l’extension Finder. Les services Finder sont disponibles si l’installation les a enregistrés. [Actions du gestionnaire de fichiers](../engineering/shell-actions.md)

<a name="xdzip-update-the-app"></a>

## Mettre l’application à jour

- **Édition du Microsoft Store.** Effectuez les mises à jour depuis le Microsoft Store. Si l’application s’exécute comme paquet MSIX, la page À propos ne propose pas de commande de mise à jour GitHub.
- **Installations depuis GitHub.** Ouvrez **À propos de xdzip** et lancez la recherche de mises à jour. Pour installer une mise à jour depuis l’application, il faut une installation Windows par utilisateur compatible, une copie macOS compatible dans Applications ou une installation Ubuntu DEB, ainsi qu’un paquet de version vérifié. Windows peut demander de confirmer l’installation. Ubuntu utilise l’autorisation du système. Si des tâches en cours retardent l’installation, attendez leur fin ou annulez l’attente.
- **Versions portables ou de développement.** Téléchargez le paquet correspondant. Sous Windows et Linux, extrayez l’intégralité du nouveau paquet ZIP ou TAR.GZ dans un dossier distinct. Sous macOS, quittez l’application avant de la remplacer dans Applications. Voir [les étapes d’installation](install.md).

Vous lancez vous-même les recherches de mises à jour depuis GitHub. Elles utilisent le réglage de proxy pris en charge. Si l’application propose uniquement un téléchargement manuel, suivez son lien vers la version et vérifiez le paquet. Une fois téléchargé, le paquet doit encore être installé. Après la mise à jour, ouvrez de nouveau **À propos de xdzip** pour confirmer la version en cours d’exécution. Le Microsoft Store et GitHub publient leurs versions indépendamment. [Fonctionnement et récupération des mises à jour](../engineering/software-update.md)
