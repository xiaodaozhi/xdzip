# Utiliser XDZip dans des scripts

[English](../../integration/README.md) | [简体中文](../../zh-CN/integration/README.md) | [繁體中文](../../zh-TW/integration/README.md) | [日本語](../../ja-JP/integration/README.md) | [한국어](../../ko-KR/integration/README.md) | Français | [Deutsch](../../de-DE/integration/README.md) | [Español](../../es-ES/integration/README.md) | [Português (Brasil)](../../pt-BR/integration/README.md) | [Русский](../../ru-RU/integration/README.md)

La CLI autonome fonctionne sans Qt ni session de bureau. Choisissez un paquet `xdzip-cli-` pour votre système dans les [versions GitHub](https://github.com/xiaodaozhi/xdzip/releases). Décompressez le paquet complet et conservez son programme auxiliaire à côté de l’exécutable. Depuis le répertoire extrait, lancez `./bin/xdzip --help` sous macOS ou Linux, ou `.\bin\xdzip.exe --help` dans PowerShell.

Les exemples supposent que `xdzip` est dans PATH. Lancez `setup` depuis la CLI extraite pour enregistrer un lanceur utilisateur, puis ajoutez son répertoire de commande à PATH si nécessaire. Vérifiez avec `xdzip --version-json` que le programme trouvé indique `"component":"cli"`. Le lanceur du bureau et la CLI peuvent avoir le même nom de commande. En cas de conflit, utilisez le chemin explicite de la CLI extraite. [Installer et retirer la commande](../engineering/command-line.md#xdzip-build-and-command-installation)

<a name="xdzip-create-and-check"></a>

## Créer et vérifier

Dans un nouveau répertoire de travail privé accessible en écriture, créez le dossier `docs` contenant `hello.txt`. Placez un second fichier, `notes.txt`, à côté du dossier. Les commandes suivantes créent un ZIP, ajoutent le second fichier dans le dossier `docs/` existant de l’archive et vérifient le résultat.

```sh
xdzip create sample.zip docs --level normal --json
xdzip add sample.zip notes.txt --into docs --conflict skip --json
xdzip test sample.zip --json
```

Choisissez l’extension et les options de création dans le [tableau des formats](../../../README.fr-FR.md#xdzip-format-support). Placez la sortie hors de l’arborescence d’entrée, dans un dossier parent existant. `add` nécessite une archive existante et admissible. Utilisez `--into` pour cibler un dossier existant de l’archive. Sans cette option, l’ajout se fait à la racine. `test` ne produit aucun fichier extrait. En cas de réussite, `data.check_kind` indique le niveau de vérification. Consultez le [registre des formats](../engineering/format-registry.md) et les [vérifications d’intégrité](../engineering/archive-checking.md).

<a name="xdzip-list-select-and-extract"></a>

## Lister, sélectionner et extraire

Poursuivez avec l’archive créée ci-dessus, dans le même répertoire de travail. Utilisez une nouvelle destination ou choisissez explicitement le traitement des fichiers existants.

```sh
xdzip list sample.zip --limit 2 --json
xdzip extract sample.zip docs/hello.txt -C output --conflict skip --json
```

Les chemins de membres sont exacts, distinguent les majuscules des minuscules et utilisent des barres obliques. Si plusieurs enregistrements ont le même chemin, sélectionnez-les avec l’`id` renvoyé par `list`. Les identifiants commencent à 1. Par exemple, remplacez `docs/hello.txt` par `--entry-id 2` dans la seconde commande si la liste indique l’identifiant 2 pour ce fichier. Répétez `--entry-id` pour d’autres membres. Utilisez `path` pour retrouver le membre et `display_name` pour son affichage. Relancez la liste après toute modification de l’archive.

La première page commence au décalage 0. Passez `data.next_offset` à `--offset` avec le même `--limit` pour lire la page suivante. Arrêtez lorsque `next_offset` vaut null. Le décalage est une position de ligne qui commence à zéro. Un identifiant désigne un enregistrement physique. Chaque commande rouvre l’archive. Laissez la source intacte entre les appels paginés. [Limites de sélection et de liste](../engineering/command-line.md#xdzip-archive-behavior)

<a name="xdzip-run-without-prompts"></a>

## Exécuter sans invite

`--json` désactive les invites interactives. Pour une exécution sans surveillance, choisissez explicitement la règle en cas de conflit. Pour les commandes qui acceptent un mot de passe, comme `extract`, `test` ou `create` avec chiffrement, utilisez `--password-stdin` et transmettez une ligne UTF-8 contenant le mot de passe par un canal privé d’entrée standard. La création chiffrée exige aussi `--encrypt`. Si une décision ou un mot de passe requis manque, la CLI renvoie `interaction_required`. [Options de mot de passe et de format](../engineering/command-line.md#xdzip-archive-behavior)

<a name="xdzip-interpret-results"></a>

## Interpréter les résultats

Pour les opérations d’archive avec `--json`, lisez l’objet UTF-8 sur stdout. Consultez `schema_version`, `status`, `error_code`, `data`, `summary` et `warnings`. Conservez stderr pour le diagnostic. `--version-json` renvoie un objet distinct qui identifie la compilation.

| Code de sortie | Traitement |
| --- | --- |
| 0 | Lisez le résultat et le nombre de fichiers ignorés. Pour une commande qui le permet, `handed_off` confirme la transmission à l’application de bureau. |
| 1 | Consultez l’erreur, les avertissements et le résumé pour identifier l’échec, un résultat partiel ou des avertissements de nettoyage. |
| 2 | Corrigez la commande ou ses arguments. |
| 3 | Vérifiez l’exigence signalée d’interaction, d’autorisation, de plateforme ou d’installation. |
| 130 | L’annulation est confirmée. Vérifiez quels fichiers ont déjà été enregistrés définitivement. |

Les valeurs inconnues peuvent être null. Après un échec ou une annulation, des fichiers enregistrés précédemment peuvent rester sur disque. Vérifiez `summary.committed_files`, quand le champ existe, avant de réessayer. La [référence CLI](../engineering/command-line.md) couvre la transmission au bureau, les reçus de mise à jour et le contrat de sortie complet.
