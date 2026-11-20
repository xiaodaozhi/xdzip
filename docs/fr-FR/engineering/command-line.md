# Contrat d’exécution en ligne de commande

[English](../../engineering/command-line.md) | [简体中文](../../zh-CN/engineering/command-line.md) | [繁體中文](../../zh-TW/engineering/command-line.md) | [日本語](../../ja-JP/engineering/command-line.md) | [한국어](../../ko-KR/engineering/command-line.md) | Français | [Deutsch](../../de-DE/engineering/command-line.md) | [Español](../../es-ES/engineering/command-line.md) | [Português (Brasil)](../../pt-BR/engineering/command-line.md) | [Русский](../../ru-RU/engineering/command-line.md)

La CLI est un hôte Rust indépendant qui appelle les services de formats et d’E/S sûres par le même `xdzip-runtime`. Elle ne dépend ni de Qt, ni de QML, ni d’une session graphique. Les formats 7z, RAR, CAB, ISO et UDF continuent d’utiliser les processus isolés fournis, sans rechercher d’autres programmes d’extraction dans PATH. Les plateformes et architectures suivent la [politique de support](support-policy.md). La CLI autonome Linux ne nécessite pas X11 ; son exécution exige OpenSSL 3 et les certificats du système, et sa compilation exige aussi libssl-dev et pkg-config.

Depuis `v0.2.0`, les versions officielles fournissent des paquets CLI autonomes pour trois plateformes, tandis que les paquets de bureau incluent une CLI privée. Les fichiers des versions précédentes ne sont pas modifiés rétroactivement. Les téléchargements sont sur la [page des versions](https://github.com/xiaodaozhi/xdzip/releases). Les sections suivantes et le [protocole de publication](release-protocol.md) précisent les limites de compilation, installation et mise à jour. L’avancement de l’implémentation et de l’acceptation est suivi dans [#254](https://github.com/xiaodaozhi/xdzip/issues/254).

## Utilisation et sortie

```sh
xdzip --help
xdzip version
xdzip list archive.zip --json
xdzip extract archive.zip docs/ -C output
xdzip extract archive.zip --entry-id 3 -C output
xdzip create backup.7z docs/ --level normal
xdzip create data.iso docs/
xdzip create portable.sfx.exe docs/ --level normal
xdzip create private.zip docs/ --encrypt --password-stdin
xdzip add backup.zip notes.txt --into docs
xdzip test backup.zip
xdzip update --check
xdzip update --download --output downloads
xdzip update --yes
xdzip update --status
xdzip gui archive.zip
```

Sans argument ou avec `--help`, xdzip affiche l’aide. `version`, `--version` et `--version-json` indiquent l’identité issue de `VERSION.json` et de l’état Git de compilation. `--version-json` émet un objet indépendant contenant product, version, build, revision, dirty, component, channel, configuration, distribution, platform et architecture. Pour l’aide et les messages, utilisez `--language zh-CN|en-US`. Par défaut, `LC_ALL`, `LC_MESSAGES` puis `LANG` sont examinés dans cet ordre ; le chinois est utilisé si la valeur choisie commence par `zh`. Les identifiants publics et les codes d’erreur sont en anglais. `l`, `x`, `c`, `a` et `t` sont des alias de `list`, `extract`, `create`, `add` et `test`. Utilisez `--` pour transmettre des arguments de fichier littéraux.

Toutes les commandes métier acceptent `--json`. stdout émet exactement un objet JSON avec les champs fixes schema_version (1), command, status, error_code, data, summary et warnings. Les tailles inconnues valent null, y compris pour les enfants partageant un flux TAR comprimé ; les membres ar bruts de DEB conservent leurs tailles comprimées exactes. Les données de liste comprennent le format et les entrées, chacune avec id, path, display_name, size, packed_size, method, directory, link, payload_supported, encrypted et warning. path est le nom effectif figé ; les noms invalides ou non résolus valent null. display_name sert uniquement à l’affichage échappé. Les résumés de tâche comprennent les nombres réels lus, décodés, vérifiés, validés, ignorés, de dossiers et de volumes, ainsi que les avertissements de nettoyage et les résultats incertains.

La progression, les erreurs et les demandes interactives sont envoyées à stderr. Les appels JSON, sans terminal ou avec `--non-interactive` ne posent pas de question. Si un mot de passe ou une décision de conflit manque, la tâche en attente est annulée, son nettoyage attendu et interaction_required renvoyé. Les codes de sortie sont 0 pour la réussite ou un transfert explicite, 1 pour l’échec, une réalisation partielle ou un avertissement de nettoyage, 2 pour des arguments invalides, 3 pour une interaction, autorisation ou condition d’installation manquante, et 130 pour une annulation confirmée. Ignorer explicitement est une politique exécutée ; les éléments ignorés restent comptés séparément. Si l’annulation n’est pas acceptée et que la validation est terminée, le résultat réel est conservé.

<a name="xdzip-archive-behavior"></a>

## Comportement des archives

`list` renvoie les enregistrements physiques en conservant les doublons. `extract` accepte des chemins effectifs complets, exacts et sensibles à la casse avec des barres obliques. Les dossiers sont développés récursivement et conservent leur hiérarchie sous la racine. Un chemin en double renvoie `ambiguous_member`. Passez l’ID numérique renvoyé par `list`, à partir de 1, avec `--entry-id` ; répétez le drapeau pour plusieurs ID. Les chemins ou ID explicites non résolus échouent avant la création du dossier cible. Les jokers, la lecture d’une archive sur stdin et l’écriture du contenu extrait sur stdout ne sont pas pris en charge.

`--limit` ou `--offset` active la pagination. L’offset commence à zéro et ne dépasse pas 1 000 000. Avec `--limit` seul, l’offset vaut zéro ; avec `--offset` seul, la limite vaut 128. Les limites valides vont de 1 à 4 096. Le `data` JSON contient `offset`, `total` et `next_offset` pouvant valoir null. `summary.entries` compte les lignes réellement renvoyées. Un ZIP/ZIP64 ordinaire en un fichier peut indexer un million d’entrées, tandis qu’une liste non paginée refuse plus de 100 000 entrées. Les deux modes disposent de 16 MiB pour les lignes JSON sérialisées et le texte rendu. Tout dépassement renvoie `resource_limit` sans lignes partielles. Les limites de page ne changent pas les ID physiques. Voir [indexation des grandes archives](large-index.md).

extract utilise par défaut le dossier courant. Si `-C` désigne un dossier absent, l’autorisation est obtenue composant par composant depuis le parent sûr existant le plus proche, au lieu de créer récursivement des dossiers sans condition. Destination, identité de source, liens, collisions Unicode ou de casse, préparation, vérification et validation continuent de suivre l’[extraction sûre](zip-safe-extraction.md) et l’[encodage des noms](filename-encoding.md). La politique de conflit par défaut est Ask ; `--conflict ask|skip|rename|overwrite` la choisit explicitement. `--allow-high-ratio` confirme seulement le taux élevé de compression et ne supprime pas les limites strictes de ressources.

`create` choisit son profil selon le [suffixe](format-registry.md) de sortie et exige que le dossier parent existe. `--level store|fast|normal|best` est validé pour le profil choisi. Pour TAR simple et ISO, omettez le niveau ou choisissez `store`. LHA (`.lzh` ou `.lha`) utilise les niveaux LH5 1, 6 et 9 pour `fast`, `normal` et `best`. ZIP et 7z prennent en charge `--encrypt` et `--volume-size` ; 7z prend aussi en charge `--encrypt-names` et `--solid` selon son profil. La taille de volume accepte un nombre entier positif d’octets ou les suffixes `KiB`, `MiB` et `GiB`. Les arguments non pris en charge par le profil sont refusés avant écriture.

Les mots de passe sont transmis uniquement par saisie masquée au terminal ou par une ligne UTF-8 avec `--password-stdin` ; les fins de ligne sont retirées sans rogner le mot de passe. La création chiffrée interactive exige une seconde confirmation. Les arguments contenant un mot de passe en clair sont refusés ; les mots de passe ne sont lus ni depuis les variables d’environnement ni écrits dans les réglages ou journaux. Longueur et effacement suivent le [contrat des mots de passe](password-extraction.md) et le [contrat de création](zip-creation.md).

`add` soumet une mise à jour à l’hôte et utilise par défaut la racine de l’archive. `--into` sélectionne un dossier sûr existant par son nom relatif à l’archive. Les entrées admissibles peuvent ajouter ou remplacer des éléments. Les déplacements internes sont une opération du bureau. La CLI partage les contrôles de [modification des archives](archive-transfers.md) concernant formats et enregistrements admissibles, protection des entrées, reconstruction temporaire et remplacement final. `test` vérifie toute l’archive sans produire de fichiers de membres. Le JSON `data.check_kind` indique la force d’une vérification terminée selon le [contrat de vérification](archive-checking.md).

Chaque opération CLI possède son propre environnement d’exécution et ne rejoint pas les sessions de navigation du bureau. Ctrl+C et les signaux de terminaison demandent une annulation coopérative puis attendent l’état terminal, le nettoyage et la fin de l’environnement ; les fichiers déjà validés restent en place. Le `submit_wait` du service partagé prend séquentiellement un bref verrou d’état seulement, sans attendre de place dans la file ni modifier l’admission non bloquante du GUI. Les pages du catalogue contiennent au plus 128 entrées physiques. Les budgets existants de nombre et de noms SelectMembers restent applicables ; toutes les requêtes sont liées à une identité de session immuable.

<a name="xdzip-build-and-command-installation"></a>

## Compilation et installation de la commande

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
python3 scripts/cli.py package --output build/package/cli
build/cli/bin/xdzip --help
```

Cette entrée utilise Rust, CMake et la chaîne C++ de plateforme fixés par le dépôt ainsi que le processus isolé existant, sans exiger Qt. L’empaquetage Windows exige aussi `prepare-windows-runtime.py` pour préparer l’environnement VC++ local fixé et sa licence d’origine. L’entrée de compilation unifiée Qt compile aussi la CLI dans un dossier cli privé, sans renommer l’exécutable GUI, l’icône ou les entrées d’association.

L’exécutable CLI inclus sous macOS est dans `Contents/Helpers/cli/`. Son contexte JSON ordinaire est dans `Contents/Resources/xdzip-cli-context.json`, protégé par la signature des ressources de l’application. La CLI ne trouve cette ressource que dans la structure fixe Contents/Helpers/cli et vérifie le schéma, le type bureau et le chemin relatif fixe du GUI. L’absence de marqueur indique une identité portable ; un marqueur invalide est refusé. Les compilations répétées retirent les anciens marqueurs générés sous Helpers ; ne réduisez pas la vérification de signature pour corriger une erreur de structure.

Les nouveaux paquets de bureau déclarent aussi `worker_relative` dans ce contexte et partagent le processus de formats du bureau. Seuls les chemins relatifs fixes de la plateforme sont acceptés : processus voisin du bureau sous Windows, `Contents/MacOS/xdzip-native-codec` sous macOS, et processus voisin de la compilation ou déployé dans `bin/` sous Linux. Un processus absent, lié ou inattendu fait refuser le contexte sans recherche dans PATH ou d’une autre copie. Les anciens contextes sans ce champ et les paquets CLI autonomes ou gérés conservent la recherche du processus voisin. Une substitution explicite du développeur reste prioritaire. Vérifiez la recherche déployée avec `scripts/check-cli.py --default-worker` ; les seules substitutions de test ne l’établissent pas.

Après décompression d’un paquet autonome, exécutez bin/xdzip, avec .exe sous Windows. `bin/xdzip setup` installe explicitement une entrée stable et un répertoire d’exécution versionné au niveau utilisateur. L’entrée Unix par défaut est ~/.local/bin ; sous Windows elle se trouve dans xdzip/cli/bin du LocalAppData utilisateur. Les répertoires de gestion utilisent les emplacements de données utilisateur de la plateforme. `--bin-dir` et `--install-dir` remplacent explicitement les emplacements. L’utilisateur ajoute lui-même le dossier à PATH ; l’outil ne modifie pas les fichiers de démarrage du shell. Les entrées et fichiers d’enregistrement existants ne sont jamais écrasés.

Une identité gérée exige un installation.json valide. Un dossier nommé versions sans fichier d’enregistrement reste portable.

Lors du retrait d’un lanceur Windows en cours d’exécution, la copie appartenant à l’outil dont l’empreinte correspond encore est déplacée dans le sous-dossier privé .xdzip-retired du même dossier de commandes. L’enregistrement est immédiatement retiré et le résultat indique retained_launcher. Un setup explicite ultérieur ne nettoie que les copies des reçus bornés dont les empreintes correspondent et dont les processus sont terminés. Les fichiers modifiés restent en place et ne sont pas écrasés. La CLI conservée dans le dossier versionné peut relancer setup pour rétablir l’enregistrement.

Le champ distribution distingue une identité source d’une identité de publication. Une compilation ordinaire depuis les sources, même propre et en configuration Release, ne permet que la vérification et le téléchargement. Le remplacement automatique exige à la fois une identité distribution marquée par le pipeline Release dédié, une installation gérée valide et une signature cible authentifiée.

L’option PATH de l’installateur de bureau Windows est décochée par défaut et refuse les conflits avec d’autres commandes xdzip. Sous Mac/Linux, setup peut être lancé depuis la CLI privée du paquet de bureau ; il n’enregistre que la CLI de cette installation et ne copie pas un cœur pouvant être mis à jour séparément. `setup --remove` ne retire que les entrées et fichiers d’enregistrement appartenant à l’outil et vérifiés par empreinte. Les dossiers versionnés et éléments de récupération restent en place ; les données utilisateur et répertoires d’exécution éventuellement utilisés ne sont pas supprimés.

Le lanceur stable n’utilise pas l’échappement du shell. Sous Unix il exécute la CLI réelle avec exec ; sous Windows il attend l’enfant console et transmet annulation et code de sortie. Le dossier d’exécution courant est choisi par un pointeur atomique current.json. Une défaillance de l’installation initiale, de l’autovérification de la nouvelle version ou de la validation des fichiers ne doit pas produire un résultat d’installation réussie.

## Mises à jour, transfert au bureau et récupération

update cible l’installation courante. Une CLI autonome ne met à jour que la CLI ; une CLI incluse dans le paquet de bureau met à jour toute l’édition de bureau. `--check` interroge, `--download` télécharge et vérifie, l’opération par défaut installe après confirmation et `--status` vérifie les reçus et l’identité réelle. `--yes` ne confirme que cette mise à jour et ne remplace pas l’autorisation système. Les compilations portables, modifiées, issues des sources ou d’identité inconnue ne permettent que la vérification et le téléchargement. Le remplacement automatique autonome exige une installation Release gérée et propre.

La CLI utilise les GitHub Releases publics fixes, le canal et les règles de numéros de compilation monotones. Les fichiers CLI des trois plateformes ont leurs propres cli-release.json et cli-release.json.sig ; la clé publique intégrée vérifie les octets bruts avant les contrôles de taille et de SHA-256. Une signature absente ou une clé non approuvée ne fournit que des informations d’acquisition manuelle ; une signature invalide échoue explicitement. Le release.json des six paquets de bureau ne contient aucune entrée CLI et les anciens clients continuent d’utiliser le protocole d’origine.

Le réseau utilise par défaut les certificats système et les variables d’environnement standard de proxy. `--proxy` choisit un proxy HTTP/SOCKS5 sans identifiants ; `--no-proxy` se connecte directement ; `--mirror` choisit un préfixe HTTPS sans identifiants, requête ni fragment. Les URL GitHub d’origine, hôtes de redirection, tailles de réponse, délais et budgets de téléchargement restent bornés. Réseau, TLS, hors ligne, limitation de débit et absence de nouvelle version compatible sont signalés séparément ; aucun sondage ou nouvel essai automatique n’est effectué.

Une mise à jour autonome est préparée dans un dossier privé appartenant à l’utilisateur, extraite sûrement par le même environnement d’archives, puis vérifie l’inventaire fixe complet et l’empreinte de chaque fichier, le produit, le composant, la plateforme et l’architecture locales et le canal du manifeste signé. L’autovérification `--version-json` de la nouvelle version est exécutée avant le changement du pointeur. Les transactions s’excluent mutuellement et les reçus lient l’identité cible. En cas d’échec, l’ancien pointeur est conservé ou une récupération est explicitement engagée ; aucun retour arrière automatique n’est inventé.

La vérification et le téléchargement du bureau interrogent seulement l’entrée non graphique d’identité d’installation du programme principal ; ils ne créent pas QApplication et ne consomment pas de reçus d’installation. L’intention d’installation passe par le canal existant d’instance unique du même utilisateur, qui n’accepte qu’un dossier de requête borné et une version candidate, pas des commandes, paquets ou racines de confiance arbitraires. Les fichiers de requête et résultat réutilisent l’autorisation native de dossiers Rust et les validations sûres. Le contrôleur du bureau revérifie et compare indépendamment le candidat, en réutilisant À propos, l’installateur, Sparkle et la barrière de jonction de toutes les tâches. Les tâches CLI de bureau détiennent aussi la protection d’occupation de l’installation ; celle-ci ne peut pas les détruire.

Après acceptation du transfert, le terminal ne signale que handed_off. L’installation et la récupération effectives dépendent des reçus du bureau et de l’identité réellement en cours ; téléchargement ou transfert ne signifient pas installation. Le mode non interactif refuse une installation de bureau exigeant GUI ou autorisation système. Voir le [contrat de mise à jour](../../engineering/software-update.md) pour les limites de sécurité et d’acceptation native. Les acceptations automatique, native et manuelle restent distinctes.

Le profil `.iso` crée des images de données ISO9660/Joliet non comprimées au niveau 0. Omettez `--level` ou utilisez `--level store`. Le profil Windows x64 `.sfx.exe` encapsule un ZIP Store/Deflate avec le `xdzip-sfx.exe` voisin fourni. Il refuse chiffrement, mode solide et volumes. Aucun des deux profils ne modifie une sortie existante. Voir [création ISO](iso-creation.md) et [création SFX propre](sfx-creation.md).
