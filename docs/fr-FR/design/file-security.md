<a name="xdzip-file-authorization-and-safe-publication"></a>

# Autorisation des fichiers et publication sûre

[English](../../design/file-security.md) | [简体中文](../../zh-CN/design/file-security.md) | [繁體中文](../../zh-TW/design/file-security.md) | [日本語](../../ja-JP/design/file-security.md) | [한국어](../../ko-KR/design/file-security.md) | Français | [Deutsch](../../de-DE/design/file-security.md) | [Español](../../es-ES/design/file-security.md) | [Português (Brasil)](../../pt-BR/design/file-security.md) | [Русский](../../ru-RU/design/file-security.md)

L’hôte contrôle les fichiers pouvant être ouverts et les emplacements où une sortie vérifiée peut être publiée. Un moteur reçoit des lecteurs et récepteurs bornés, y compris dans le processus d’archive isolé. Les noms, tailles déclarées, sommes de contrôle et contenus d’archive sont des données d’entrée. Les contrôles de fichiers de l’hôte s’appliquent indépendamment de ceux du format. [Architecture système](system-architecture.md), [Protocole du processus d’archive](../engineering/native-codecs.md)

<a name="xdzip-authorizing-the-source-and-destination"></a>

## Autoriser source et destination

- **Source de l’archive.** `Source::open` ouvre le chemin choisi via `xdzip-io::open_regular_readonly` et vérifie le type du fichier ouvert. Sur Unix, l’ouverture de classification est non bloquante afin qu’un FIFO ne la bloque pas. Un chemin choisi par l’utilisateur peut résoudre un lien vers un fichier ordinaire. La découverte des volumes est bornée et propre au format. L’hôte fige les parties voisines admises et contrôle leurs identités et versions. Un codec ne peut adresser que les flux fournis. [Implémentation de la source](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/source.rs), [Gestion des volumes](../engineering/archive-opening.md)
- **Entrées de création.** `InputTree` parcourt les entrées choisies avec des métadonnées et identités de fichiers bornées. Les lectures ouvrent les membres de cet arbre en vérifiant leur version. Le dossier privé de préparation courant est exclu de son propre scan d’entrée. Un changement détecté arrête le travail. Les contrôles d’identité, taille et dates détectent les changements au mieux mais ne constituent pas un instantané complet du contenu. [Arbre d’entrée](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-io/src/input.rs), [Création d’archive](../engineering/zip-creation.md)
- **Destination.** `Grant` détient l’objet dossier sélectionné et son identité. L’extraction transmet aussi les identités de tous les volumes sources comme fichiers protégés. Création et modification contrôlent les identités des entrées figées ou de l’archive d’origine dans leurs propres chemins de tâche. Sous-dossiers et préparation sont accessibles par le dossier détenu ; les identités parentes sont revérifiées avant publication. `CheckTarget` et la vérification préalable d’extraction peuvent fournir des constats utiles avant le démarrage. L’autorisation d’écriture est de nouveau vérifiée pendant l’opération. [Autorisations de destination](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-io/src/target.rs), [Pré-vérification d’extraction](../engineering/extraction-preflight.md)

<a name="xdzip-name-mapping-platform-protection-and-links"></a>

Quand une création ou une extraction utilise une destination vérifiée, la tâche d’écriture compare l’identité du répertoire avec celle validée par l’appelant. Les entrées à archiver conservent l’identité vérifiée lors de la sélection. L’ouverture des volumes détectés automatiquement ne suit pas le lien du dernier composant du chemin. Sous Windows, la modification sur place préserve le propriétaire, les droits de lecture et la zone d’origine de l’archive. Si ces protections ne peuvent être conservées, l’archive d’origine reste en place.

## Correspondance des noms, protection par plateforme et liens

1. **Valider l’écriture relative à l’archive.** Le runtime résout la sélection figée, retire le préfixe du sous-arbre sélectionné si nécessaire et utilise `PortablePath` pour accepter des composants UTF-8 relatifs. Chemins absolus ou de lecteur, caractères de contrôle, séparateurs ambigus, composants parents, noms de périphériques réservés et espaces ou points finaux sont refusés. Une écriture validée n’est qu’une entrée pour la capacité de destination.
2. **Planifier les collisions avant de créer la sortie.** L’extraction compare les alias courants de casse et Unicode par des clés NFD en minuscules. Écritures ambiguës de dossiers et collisions fichier/dossier font échouer la préparation. Les alias de fichiers ordinaires convergent vers une même écriture afin d’appliquer la politique de conflit. La publication native exclusive détermine toujours si le nom demandé est disponible sur le système de fichiers réel. Écriture affichée et identité physique restent distinctes.
3. **Parcourir l’arbre autorisé.** La sortie descend depuis une capacité de dossier détenue, en ouvrant les dossiers enfants sans suivre les liens. Elle revérifie la racine autorisée, les identités parentes et la feuille attendue pour l’écrasement. Une cible remplacée ou un lien dangereux est refusé. Une dernière vérification avant renommage sur le même volume réduit les courses. Un renommage ordinaire ne garantit pas une comparaison-échange du contenu contre un processus disposant des droits du même compte.

| Plateforme | Contrôles des objets de sortie |
| --- | --- |
| Linux et macOS | Le propriétaire doit être l’utilisateur effectif courant. Les droits d’écriture du groupe et des autres sont refusés et un fichier ordinaire existant doit avoir un seul lien physique. macOS vérifie aussi l’ACL native, y compris les droits de modification hérités. |
| Windows | Vérifier points de réanalyse, propriétaire, ID de fichier, nombre de liens physiques des fichiers ordinaires et DACL des objets détenus. Les règles d’accès effectives et celles d’héritage seul doivent limiter la modification aux principaux de confiance. Le partage existant en lecture seule peut rester. |
| Texte en clair privé ou travail temporaire | Des contrôles de confidentialité supplémentaires excluent les lecteurs non concernés. Sous Windows, un dossier de cache privé créé exclusivement peut recevoir une DACL protégée. Les destinations ordinaires choisies par l’utilisateur gardent leur politique de partage en lecture. |

La sortie de liens symboliques ZIP se limite à un graphe Unix admis. Le runtime valide le graphe sélectionné et les feuilles devant être absentes avant publication des liens, après validation des cibles ordinaires. La tâche ne suit jamais un lien créé par l’archive pour écrire du contenu. Les autres profils de lien ou fichiers spéciaux non pris en charge échouent explicitement. Les modes POSIX ordinaires peuvent restaurer des bits sûrs de lecture/exécution sur la préparation, en plus de lecture/écriture du propriétaire. Les bits de privilège et d’écriture externe sont exclus. L’extraction ne restaure pas les dates après validation.

[Analyseur de chemin portable](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-domain/src/path.rs), [Planification d’extraction](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/extraction/plan.rs), [Contrôles d’objets privés](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-io/src/private.rs), [Détails des liens et plateformes](../engineering/zip-safe-extraction.md)

<a name="xdzip-staging-verification-and-commit"></a>

## Préparation, vérification et validation

Une autorisation de destination crée un espace de travail privé exclusif sur le même volume. L’hôte mesure la sortie du codec vers son fichier de préparation. Le décodage, la taille attendue et les contrôles d’intégrité ou d’authentification fournis par le format doivent réussir avant publication. `Stage::finish` vide et synchronise le fichier et conserve sa version vérifiée. L’hôte revérifie préparation et destination lors de la validation. La synchronisation des données et la visibilité atomique d’un fichier ne prouvent pas la durabilité des métadonnées de dossier après coupure de courant.

| Opération | Sémantique de publication |
| --- | --- |
| Extraire ou publier une promesse de fichier | Publier chaque fichier admissible après vérification. Sans écrasement, créer exclusivement un lien physique sur le même volume puis retirer le lien privé de préparation. Avec écrasement, l’extraction exige le fichier ordinaire sûr attendu et le remplace par renommage sur le même volume sans le tronquer d’abord. Une promesse de fichier est toujours publiée sans écrasement. L’absence de primitive nécessaire ou une destination modifiée fait échouer l’écriture concernée. Il n’y a pas de retour arrière global des fichiers ou dossiers déjà publiés. |
| Créer une archive unique | Encoder dans la préparation, rouvrir et vérifier entrées et contenu attendus pour le profil, puis publier une fois l’archive entière. Un conflit de nom peut choisir un autre nom ou autoriser un remplacement admissible. Une entrée protégée ne peut pas devenir cible d’écrasement. |
| Créer des volumes fractionnés | Vérifier le contenu de l’archive préparée, préparer chaque partie et publier le nouvel ensemble plat sans écraser les volumes existants. Les parties deviennent visibles individuellement. En cas d’échec partiel, le retour arrière tente de supprimer seulement les objets nouvellement publiés dont l’identité correspond encore. Un échec de retour arrière peut laisser un résultat incertain. |
| Modifier une archive existante | `ArchiveUpdate` lie le parent autorisé et la version attendue de l’archive d’origine. Rouvrir et vérifier l’archive réécrite puis remplacer une fois l’original par cette autorité. L’extraction ordinaire ne peut pas utiliser ce chemin spécial pour écraser sa source. |

Une annulation avant la barrière de publication empêche la validation courante. Une publication déjà en cours peut aboutir et rapporter son résultat réel. Les fichiers d’extraction déjà validés restent visibles. L’instantané terminal peut rapporter des données validées avec `cancelled`, `failed` ou un avertissement de nettoyage. Une publication complète d’archive réussie conserve son succès malgré une annulation tardive. [Annulation de tâche](task-scheduling.md), [Modification d’archive](../engineering/archive-transfers.md), [Extraction sûre](../engineering/zip-safe-extraction.md)

<a name="xdzip-output-budgets-and-integrity-scope"></a>

## Budgets de sortie et portée de l’intégrité

L’extraction limite séparément la sortie déclarée et réelle : 8 GiB par fichier ordinaire et 20 GiB par tâche. Une vérification de taux élevé peut refuser la demande à l’admission dans les formats pris en charge. Pour ZIP, un rapport déclaré supérieur à 1 000 pour un pour une entrée de plus de 1 MiB nécessite une autorisation explicite. Cette autorisation conserve les limites strictes. Les décodeurs solides ou séquentiels peuvent travailler sur des données non sélectionnées. Ce travail, les lectures ou tentatives répétées et le travail auxiliaire sont comptabilisés séparément des octets publiés.

Navigation des métadonnées, admission des tâches, mémoire des processus et aperçu ont leurs propres budgets. Un grand index ZIP sur disque n’augmente pas les limites d’extraction ou d’écriture. Les contrôles d’intégrité dépendent du format. Par exemple, la structure et la lisibilité ISO/UDF ne fournissent pas une somme de contrôle par fichier qui manque. [Grands index d’archive](../engineering/large-index.md), [Vérification d’intégrité](../engineering/archive-checking.md), [Comptabilisation auxiliaire du décodage](../engineering/auxiliary-decoding-work.md)

<a name="xdzip-preview-temporary-opening-and-secrets"></a>

## Aperçu, ouverture temporaire et secrets

- **Aperçu.** Sélectionner un fichier pris en charge lance un aperçu dans la carte de propriétés visible. Rust lit et vérifie l’entrée admise entière dans ses budgets. Le texte est limité à 256 KiB. Les images encodées sont limitées à 16 MiB, avec des limites distinctes de dimensions et pixels d’origine. Le processus privé d’image reçoit des octets et renvoie une vignette bornée. Il ne reçoit ni nom d’archive, mot de passe ou handle de destination. Les formats et restrictions actuels sont dans le [contrat d’aperçu](../engineering/content-preview.md).
- **Ouverture temporaire.** Une entrée ordinaire est extraite dans le cache privé selon les règles normales de vérification de la tâche. Le bureau remet le fichier vérifié à l’association système par URL locale. L’acceptation ne prouve pas que le programme externe a terminé sa lecture. Lors d’une opération ultérieure après la période de conservation, le nettoyage vérifie identité du cache et verrous. Les modifications de la copie sont indépendantes de l’archive d’origine. [Ouverture temporaire](../engineering/archive-opening.md), [Transferts d’archive](../engineering/archive-transfers.md)
- **Secrets.** Les mots de passe d’ouverture et d’extraction restent dans des conteneurs propres à l’opération et effacés à la libération pendant la durée de la session ou tâche. Ils n’entrent ni dans les instantanés de tâche ni dans le journal de préparation. Les règles de stockage privé, conservation et divulgation des données utilisateur figurent dans le [contrat de confidentialité](../engineering/privacy.md) et le [contrat de mot de passe](../engineering/password-extraction.md).

<a name="xdzip-journals-leftovers-and-verification"></a>

## Journaux, restes et vérification

Les journaux de préparation enregistrent le contexte de tâche et les noms relatifs de sortie à la préparation et à la validation. Le nettoyage normal vérifie propriétaire et identités et ne supprime que les objets de préparation connus. Journaux privés et objets restants peuvent fournir des indices après une fin anormale. Le runtime compte les restes possibles lorsqu’il vérifie une destination. Il n’utilise pas un journal découvert pour rejouer des écritures ou supprimer un contenu inconnu. L’expiration du cache de sortie temporaire privé possède ses propres contrôles d’identité et de verrou.

Pour modifier cette frontière, vérifier publication et nettoyage ainsi que les codes d’erreur. Les [tests d’E/S](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-io/src/tests.rs), [tests d’extraction](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/extraction/tests.rs) et le [contrat d’extraction sûre](../engineering/zip-safe-extraction.md) couvrent des couches différentes. Un changement du comportement natif d’une plateforme exige des preuves propres à cette plateforme.
