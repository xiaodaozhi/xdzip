# Mots de passe et volumes fractionnés

[English](../../user-guide/passwords-and-volumes.md) | [简体中文](../../zh-CN/user-guide/passwords-and-volumes.md) | [繁體中文](../../zh-TW/user-guide/passwords-and-volumes.md) | [日本語](../../ja-JP/user-guide/passwords-and-volumes.md) | [한국어](../../ko-KR/user-guide/passwords-and-volumes.md) | Français | [Deutsch](../../de-DE/user-guide/passwords-and-volumes.md) | [Español](../../es-ES/user-guide/passwords-and-volumes.md) | [Português (Brasil)](../../pt-BR/user-guide/passwords-and-volumes.md) | [Русский](../../ru-RU/user-guide/passwords-and-volumes.md)

<a name="xdzip-passwords"></a>

## Mots de passe

AES Crypt et les archives aux noms de fichiers chiffrés, par exemple un 7z avec chiffrement des noms, demandent un mot de passe avant d’afficher la liste. D’autres archives peuvent le demander lors de l’extraction, de la vérification ou de l’aperçu. Respectez les espaces et la casse en saisissant le mot de passe. Certains profils ZIP AES-256 sont pris en charge. ZIP AES-128/192 et ZipCrypto classique ne le sont pas. Si le mot de passe est correct et que l’archive ne fonctionne toujours pas, vérifiez la prise en charge du chiffrement et si le fichier est endommagé. [Chiffrement pris en charge](../engineering/password-extraction.md)

Pour créer une archive protégée par mot de passe, choisissez un format doté du chiffrement. ZIP AES-256 protège le contenu et laisse les noms visibles. Si la liste des fichiers doit aussi être protégée, choisissez 7z et activez le chiffrement des noms. Conservez le mot de passe séparément de l’archive.

<a name="xdzip-split-volumes"></a>

## Volumes fractionnés

1. Placez tous les volumes d’une même archive dans un seul dossier et conservez leurs noms d’origine.
2. Ouvrez un volume pris en charge. XDZip recherche les autres parties dans le même dossier. Les volumes numérotés de ZIP et 7z, PKZIP et les ensembles RAR et CAB pris en charge suivent des conventions différentes.
3. Si l’ensemble est incomplet, récupérez les parties manquantes et ouvrez de nouveau l’archive. Les assembler ou les renommer manuellement peut les rendre illisibles.

Pour créer une archive fractionnée, choisissez ZIP ou 7z et indiquez la taille des volumes dans la fenêtre de création. Partagez l’ensemble complet. Le fractionnement ne rend pas l’archive modifiable dans XDZip. [Noms des volumes et profils pris en charge](../engineering/archive-opening.md)
