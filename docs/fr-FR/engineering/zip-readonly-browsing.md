<a name="xdzip-read-only-zip-browsing-contract"></a>

# Contrat de navigation ZIP en lecture seule

[English](../../engineering/zip-readonly-browsing.md) | [简体中文](../../zh-CN/engineering/zip-readonly-browsing.md) | [繁體中文（台灣）](../../zh-TW/engineering/zip-readonly-browsing.md) | [日本語](../../ja-JP/engineering/zip-readonly-browsing.md) | [한국어](../../ko-KR/engineering/zip-readonly-browsing.md) | Français | [Deutsch](../../de-DE/engineering/zip-readonly-browsing.md) | [Español](../../es-ES/engineering/zip-readonly-browsing.md) | [Português (Brasil)](../../pt-BR/engineering/zip-readonly-browsing.md) | [Русский](../../ru-RU/engineering/zip-readonly-browsing.md)

Ce contrat définit les entrées situées directement à la racine, la recherche dans les sous-dossiers, l’identité physique stable et le tri qui place les dossiers en premier. La navigation ZIP lit uniquement les métadonnées, sans décoder le contenu des entrées, vérifier leur CRC ni extraire de fichiers. Les grandes archives ZIP ordinaires peuvent utiliser un [index privé de métadonnées sur disque](large-index.md).

Consultez l’[indexation des grandes archives](large-index.md) pour connaître les limites actuelles des ZIP ordinaires et le fonctionnement du modèle Qt à la demande. Cette page conserve la limite initiale de 100 000 entrées et les exigences d’identité des requêtes. Les augmentations de capacité ne dispensent pas de tester les erreurs, l’annulation et l’identité.

<a name="xdzip-browsing-scope-and-limits"></a>

## Périmètre et limites de la navigation

- Jusqu’à 100 000 enregistrements physiques de métadonnées ZIP/ZIP64. L’[hôte des volumes fractionnés](archive-opening.md) convertit les volumes avant lecture. Le codec continue de refuser les métadonnées brutes de plusieurs disques.
- Limites de 4 KiB par nom, 16 MiB pour les noms bruts, 16 MiB pour les noms d’affichage décodés et échappés, 64 MiB de lecture totale et 256 KiB pour le tampon de balayage. Les tailles déclarées sur 64 bits restent des entiers exacts.
- L’index des dossiers comporte jusqu’à 32 768 nœuds, 64 niveaux et 4 MiB de noms et préfixes. Au-delà, il est abandonné en entier et le mode dégradé est expliqué. Aucun arbre partiel n’est publié.
- Les index des entrées directes et les résultats de requête sont réutilisés. Les lots d’une même requête ne refiltrent ni ne retrient les données. Chaque lot limite les noms à 64 KiB. L’interface affiche le dossier courant en entier, sans boutons de pagination.
- Les pages valident la session, l’identifiant de requête, la génération de l’interface, la révision de l’instantané et la position. Un changement de requête annule le travail antérieur et retient la dernière demande. Une réponse obsolète ne remplace pas les résultats récents.
- La longueur, la date de modification, le périphérique et l’inode Unix ainsi que le numéro de volume et l’index de fichier Windows sont vérifiés avant et après le balayage, lors des requêtes et des contrôles préalables. En cas de `source_changed`, les requêtes Qt et les contrôles préalables effacent la liste, la sélection, l’arborescence et les détails devenus invalides. Rouvrez l’archive pour créer une session. La détection fonctionne dans la mesure du possible et n’offre ni instantané atomique du système de fichiers ni autorisation d’écriture sécurisée.

<a name="xdzip-acceptance-exit-criteria"></a>

## Critères de validation

Un ZIP64 réellement généré avec 100 000 entrées doit être analysé, trié dans son intégralité et consultable. Vérifiez la navigation dans les dossiers, la liste Qt complète et la sélection. Tous les lots partagent la même identité de requête. Testez un ZIP64 endommagé, les volumes fractionnés, les limites de noms, d’affichage et de lecture, l’annulation, les réponses obsolètes ainsi que la modification et le remplacement de la source. Consignez les durées, les ressources, la file d’événements et le nombre de délégués. Si les builds des trois plateformes ou les paquets de développement Apple Silicon sont vérifiés à la demande, consignez leurs résultats à part.

Filtrez la liste des exécutions CI du dépôt par head_sha et par événement effectivement déclenché. Les contrôles courants s’exécutent lors d’un push ou d’une PR. Les contrôles complets Foundation et Qt sont manuels, à la demande. Consignez les résultats et les vérifications non exécutées dans le ticket concerné. Un résultat vide pour un autre événement ne détermine pas l’état du contrôle.
