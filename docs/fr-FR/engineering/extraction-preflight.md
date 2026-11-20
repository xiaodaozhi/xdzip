<a name="xdzip-extraction-preflight-metadata-phase"></a>

# Précontrôle d’extraction : phase de métadonnées

[English](../../engineering/extraction-preflight.md) | [简体中文](../../zh-CN/engineering/extraction-preflight.md) | [繁體中文（台灣）](../../zh-TW/engineering/extraction-preflight.md) | [日本語](../../ja-JP/engineering/extraction-preflight.md) | [한국어](../../ko-KR/engineering/extraction-preflight.md) | Français | [Deutsch](../../de-DE/engineering/extraction-preflight.md) | [Español](../../es-ES/engineering/extraction-preflight.md) | [Português (Brasil)](../../pt-BR/engineering/extraction-preflight.md) | [Русский](../../ru-RU/engineering/extraction-preflight.md)

Ce module fournit des règles de précontrôle des métadonnées. Il ne peut pas autoriser une destination, décoder, écrire, préparer temporairement ou valider une sortie.

<a name="xdzip-implementation-and-boundaries"></a>

## Implémentation et limites

`xdzip-extract-plan` dépend uniquement des interfaces de domaine et de métadonnées, sans Qt, analyseur de format ou dépendance tierce supplémentaire. L’entrée est un instantané de session, un ensemble d’EntryId physiques et l’annulation ; la sortie est seulement un rapport borné, sans chemins utilisables pour écrire ni indicateur d’approbation de sécurité. `xdzip-runtime` détient l’instantané ; Qt l’appelle via le pont du thread de travail.

La sélection doit être non vide, contenir au plus 128 ID physiques et appartenir entièrement à l’instantané courant. Sélectionner un répertoire développe tous ses descendants selon les séparateurs obliques des noms d’origine, indépendamment de la recherche, pagination ou portée du répertoire courant. Un enfant sélectionné avec son répertoire n’est compté qu’une fois. Les enregistrements physiques de même nom ne sont pas fusionnés par chemin. Les statistiques de répertoires ne comptent que les enregistrements explicites et ne créent pas d’objets de répertoire synthétiques.

Le précontrôle passe par le service de commandes et événements avec admission bornée et annulation indépendante. Le runtime vérifie l’identité de la source ouverte avant et après inspection. Pour un ZIP indexé sur disque, il charge depuis l’index les enregistrements physiques sélectionnés et les descendants nécessaires. Un changement de sélection, requête ou session efface le rapport. Les résultats sont des estimations des métadonnées enregistrées. Le précontrôle ne décode jamais les données des membres, ne réserve pas de chemins de destination et n’autorise aucune écriture.

<a name="xdzip-rules-performed"></a>

## Règles appliquées

- Les noms doivent être résolus en UTF-8, avec au plus 4 KiB, 64 composants et 255 octets par composant ; les noms d’origine ne sont ni réparés ni réécrits.
- Refuser chemins absolus, composants vides, `.`, `..`, barres obliques inverses, deux-points, caractères de contrôle ou de contrôle bidirectionnel, caractères interdits par Windows et espaces ou points finaux des composants.
- Refuser les noms courants de périphériques Windows, y compris avec extension et chiffres COM/LPT en exposant, en appliquant prudemment un sous-ensemble portable multiplateforme.
- Signaler les avertissements de métadonnées existants et utiliser les indicateurs de capacité de contenu et de mot de passe de l’enregistrement. Un enregistrement chiffré peut signaler un besoin de mot de passe pris en charge ou un chiffrement non pris en charge. Un enregistrement de taille connue dont le contenu n’est pas pris en charge signale aussi un problème de méthode. Le précontrôle utilise les métadonnées et capacités projetées par le lecteur ; il ne déchiffre, n’authentifie ni ne décode le contenu.
- Vérifier les noms exactement identiques, collisions par comparaison Unicode en minuscules, alias de répertoires parents différant par la casse et conflits fichier/répertoire dans les deux ordres d’insertion. Un répertoire parent implicite peut être complété par un répertoire explicite de même nom ; des enregistrements explicites de répertoire en double restent en conflit.
- Signaler les tailles déclarées par fichier au-delà de 8 GiB, les totaux au-delà de 20 GiB et les débordements entiers. Pour ZIP, un taux de compression déclaré supérieur à 1000:1 demande une revue. Les tailles de répertoires ne comptent pas dans les octets de sortie.

La comparaison Unicode en minuscules n’est ni un repli complet de casse ni l’équivalence NFC/NFD d’un système de fichiers. Tout composant non ASCII ajoute un avis indiquant que les équivalences et collisions doivent encore être vérifiées sur le système de fichiers de destination. Même un rapport sans constat n’autorise pas la sortie.

<a name="xdzip-resources-and-failures"></a>

## Ressources et échecs

Vérifier au plus 100,000 enregistrements de l’instantané ; les noms sélectionnés ou développés totalisent au plus 4 MiB. Les clés de comparaison et graphies originales stockées dans l’index de collisions totalisent au plus 8 MiB, hors surcharge du conteneur. La profondeur de parcours des composants est bornée et l’annulation est vérifiée à chaque enregistrement externe. Les rapports comptent tous les constats, mais au plus 64 problèmes représentatifs traversent le pont, avec seulement les ID physiques et raisons stables, sans texte d’erreur arbitraire de l’archive.

Une sélection vide, des ID étrangers, des limites d’index ou une annulation renvoient des erreurs, pas des rapports partiellement réussis. Les totaux en débordement affichent « Indisponible » au lieu d’être tronqués à zéro. Les constats élémentaires sont seulement signalés ; ils n’écrasent, ne renomment, n’ignorent ni ne sautent automatiquement rien.

Les tailles non compressées inconnues signalent la raison stable `unknown_size`. `declared_bytes=None` indique un total inconnu, distinct de `Some(0)` pour un fichier vide de taille connue. Les valeurs inconnues ne fabriquent pas de constats de taille totale dépassée ou de méthode inconnue ; un véritable débordement entier produit toujours un constat de limite totale. Les tâches de sortie refusent séparément les enregistrements de taille inconnue avant décodage, préparation ou interaction de mot de passe.

<a name="xdzip-desktop-entry-point"></a>

## Entrée bureau

Sélectionnez des entrées puis cliquez sur « Vérifier les métadonnées avant extraction… ». Les résultats utilisent une fenêtre supérieure `OperationWindow` indépendante et non modale, avec la fenêtre principale comme propriétaire transitoire et la barre de titre native conservée ; l’application ne dessine pas les boutons de fenêtre macOS. Fermer les résultats ne ferme que cette fenêtre. La fermeture de la fenêtre principale utilise l’annulation coopérative sans attendre les workers sur le thread graphique.

<a name="xdzip-boundary-with-actual-extraction"></a>

## Limite avec l’extraction réelle

L’[extraction sûre réelle](zip-safe-extraction.md) implémente indépendamment la comptabilisation de sortie, la vérification CRC et structurelle, l’autorisation de destination, les nouvelles vérifications de chemins par plateforme, les conflits, la préparation et la validation des fichiers. Elle n’est pas limitée aux 128 sélections du précontrôle et prend en charge des plans de 100,000 enregistrements physiques et des sélections de répertoires virtuels. Un rapport de précontrôle n’est pas réutilisé comme autorisation d’écriture. Le démarrage d’une tâche doit obtenir à nouveau l’autorisation de destination, revalider les sources et vérifier le contenu réel selon la [spécification de sécurité des fichiers](../design/file-security.md). Les comportements d’échec natifs et défauts connus sont suivis par les contrats et tickets concernés.

<a name="xdzip-window-and-result-presentation"></a>

## Présentation de la fenêtre et des résultats

Le précontrôle utilise une fenêtre d’opération partagée de 491 pixels logiques de large, avec corps adapté au contenu, hauteur maximale de zone cliente de 600, liste de problèmes longue défilante et pied fixe accessible. Les limites d’autorisation des phases de données et de métadonnées restent inchangées. Voir les [paramètres de précontrôle](../../../specs/visual/preflight-reference.json) pour les paramètres et la source de conception.

Le placement de l’explication du précontrôle a été révisé le 2026-10-15 : déplacer « Ce rapport vérifie uniquement les métadonnées… » à la fin du corps défilant, en texte explicatif de 12 points. Le pied ne contient plus cette explication ; il ne garde que l’action Retour et correspond à la hauteur d’un pied partagé contenant seulement des actions à taille de police égale, indépendamment de la longueur du rapport.
