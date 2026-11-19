# Créer et modifier des archives

[English](../../user-guide/create-and-modify.md) | [简体中文](../../zh-CN/user-guide/create-and-modify.md) | [繁體中文（台灣）](../../zh-TW/user-guide/create-and-modify.md) | [日本語](../../ja-JP/user-guide/create-and-modify.md) | [한국어](../../ko-KR/user-guide/create-and-modify.md) | Français | [Deutsch](../../de-DE/user-guide/create-and-modify.md) | [Español](../../es-ES/user-guide/create-and-modify.md) | [Português (Brasil)](../../pt-BR/user-guide/create-and-modify.md) | [Русский](../../ru-RU/user-guide/create-and-modify.md)

<a name="xdzip-create-an-archive"></a>

## Créer une archive

1. Ouvrez la fenêtre de création depuis la page d’accueil ou la commande **Créer**. Ajoutez les fichiers ou dossiers et vérifiez la liste des sources.
2. Choisissez le nom et la destination. Sélectionnez un format et l’un de ses réglages de compression. ZIP est couramment utilisé pour partager des fichiers. Consultez le [tableau des formats](../../../README.fr-FR.md#xdzip-format-support) pour connaître les options. Un flux à fichier unique tel que GZIP accepte un seul fichier ordinaire.
3. ZIP et 7z proposent le chiffrement et les volumes fractionnés lorsque les options choisies le permettent. Choisissez ISO pour obtenir une image disque de données ou le profil SFX Windows x64 pour un ZIP auto-extractible. Lancez l’opération et consultez le résultat pour savoir quels fichiers ont été enregistrés. [Mots de passe et volumes](passwords-and-volumes.md)
4. Si vous devez annuler ou réessayer, consultez d’abord le résultat. L’archive est écrite dans un emplacement temporaire avant que le résultat terminé soit publié.

<a name="xdzip-add-replace-or-move-files"></a>

## Ajouter, remplacer ou déplacer des fichiers

Ouvrez une archive ZIP, 7z, TAR ou TAR.GZ compatible et accédez au dossier de destination dans l’archive. Déposez des fichiers ou dossiers locaux sur la liste ou la grille, puis vérifiez et confirmez l’ajout. Les options de conflit permettent notamment de remplacer certains fichiers ordinaires de l’archive. Pour déplacer une entrée existante, faites-la glisser vers un autre dossier de la même archive et confirmez. Faire glisser un élément hors de la fenêtre exporte une copie. [Glisser-déposer selon le système](../engineering/archive-transfers.md)

La création et l’ajout omettent les fichiers `.DS_Store` et les dossiers `__MACOSX` et `_MACOSX`. La possibilité de modifier une archive dépend de son format, de son contenu et de sa source. La procédure habituelle ne permet pas de modifier les archives chiffrées, fractionnées ou indexées sur disque. Si la modification est indisponible, extrayez dans un nouveau dossier et créez une nouvelle archive. [Règles de modification et de glisser-déposer](../engineering/archive-transfers.md)
