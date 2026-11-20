<a name="xdzip-adaptive-filename-encoding-contract"></a>

# Contrat d’encodage adaptatif des noms de fichiers

[English](../../engineering/filename-encoding.md) | [简体中文](../../zh-CN/engineering/filename-encoding.md) | [繁體中文（台灣）](../../zh-TW/engineering/filename-encoding.md) | [日本語](../../ja-JP/engineering/filename-encoding.md) | [한국어](../../ko-KR/engineering/filename-encoding.md) | Français | [Deutsch](../../de-DE/engineering/filename-encoding.md) | [Español](../../es-ES/engineering/filename-encoding.md) | [Português (Brasil)](../../pt-BR/engineering/filename-encoding.md) | [Русский](../../ru-RU/engineering/filename-encoding.md)

<a name="xdzip-resolution-and-identity"></a>

## Résolution et identité

Le décodage a lieu dans la couche de formats Rust indépendante de Qt. Il n’utilise ni pages de codes du système hôte, ni langue de l’interface, ni suppositions sur le système source, contenu des fichiers ou services réseau. Pour ZIP, raw_name conserve les octets originaux du champ de nom. Pour TAR, il conserve le nom brut choisi par PAX/GNU après retrait du `./` initial selon les règles existantes. L’EntryId physique est indépendant du nom. decoded_name stocke l’écriture UTF-8 nécessitant transcodage ou déclaration d’encodage supplémentaire ; name_encoding conserve l’origine de la résolution et name_bytes fournit le nom effectif figé. display_name n’est qu’un affichage échappé et ne peut pas servir de chemin de sortie.

ZIP résout les noms dans cet ordre :

1. Si l’indicateur UTF-8 est activé, exiger un UTF-8 valide dans le champ original ; ne pas réparer une déclaration corrompue avec un ancien encodage.
2. Sinon lire Info-ZIP Unicode Path 0x7075 et accepter la version 1 si le CRC32 du nom original correspond et si l’UTF-8 est complet et valide. Ignorer les versions inconnues et CRC non concordants conformément à la spécification. Un UTF-8 invalide malgré une correspondance, des champs valides en double ou une structure endommagée sont marqués pour examen.
3. Sans extension utilisable, si tous les noms non déclarés du groupe sont en UTF-8 valide, accepter cet UTF-8 non marqué.
4. Sinon détecter statistiquement à l’échelle de l’archive les noms non ASCII sans déclaration : GBK/GB18030, Big5, Shift-JIS, EUC-JP, coréen et autres jeux pris en charge. Pour ZIP, une hypothèse occidentale par défaut revient au défaut historique CP437. Si GBK est détecté, utiliser le décodeur strict GB18030 compatible.

Les en-têtes TAR ordinaires, noms longs GNU et chemins PAX explicitement marqués hdrcharset=BINARY utilisent la même stratégie pour les noms non déclarés. Les chemins PAX ordinaires exigent un UTF-8 valide ; la détection ne doit pas masquer une déclaration corrompue. Les valeurs explicites hdrcharset autres que UTF-8 et BINARY ne sont pas prises en charge. TAR n’hérite pas du défaut CP437 de ZIP. TAR.GZ utilise la même résolution de noms que TAR.

Les règles suivent [ZIP APPNOTE 6.3.10](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT), [POSIX pax](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/pax.html) et la [norme WHATWG Encoding](https://encoding.spec.whatwg.org/). Un champ Unicode Path peut être absent de l’en-tête local, mais une valeur valide fournie doit correspondre au nom effectif figé. La vérification des données contrôle toujours nom brut, en-têtes local et central, CRC, taille, authentification par mot de passe et terminaison du flux.

CAB conserve les octets originaux du nom et l’attribut UTF-8. L’UTF-8 explicite est strictement validé ; les autres noms réutilisent une détection bornée à l’échelle de l’archive sans page de codes système. Les barres obliques inverses CAB sont des séparateurs de répertoire du format, normalisés en barres obliques uniquement après décodage complet, afin de ne pas séparer l’octet final multioctet Shift-JIS 0x5C. Les octets bruts restent inchangés et les chemins effectifs subissent les mêmes contrôles de sécurité et de collision. Les noms ISO non Joliet conservent leurs octets bruts et utilisent la même résolution bornée, sans conversion OEM de l’hôte pour Rock Ridge. Joliet/UDF utilisent les noms Unicode fournis par le processeur et les valident strictement ; les caractères de remplacement avec perte ne sont pas considérés comme noms originaux.

<a name="xdzip-detection-boundaries-and-resources"></a>

## Limites de détection et ressources

Le détecteur lit au plus 1 MiB de noms complets non déclarés et non UTF-8, séparés par des retours à la ligne pour empêcher la combinaison de séquences multioctets tronquées entre noms. Les déclarations Unicode explicites et extensions invalides ne sont pas échantillonnées. Si l’échantillon contient des noms non UTF-8, appliquer systématiquement le résultat global aux noms non déclarés afin qu’un fragment GBK fortuitement valide en UTF-8 ne sépare pas un répertoire en deux noms. Ce n’est qu’après l’échec du décodage strict d’un nom individuel qu’un UTF-8 valide ou un résultat local à ce nom peut être essayé. La détection ISO-2022-JP est désactivée ; les états d’échappement ne sont pas interprétés. Le décodage strict n’insère pas de caractères de remplacement, ne retire pas le BOM, ne convertit pas les barres obliques, ne normalise pas Unicode et ne répare pas les chemins.

Les noms bruts totalisent au plus 16 MiB. Un nom brut ou décodé est limité à 4 096 octets, et les noms décodés avec leur affichage échappé à 16 MiB au total. Les allocations par entrée et cumulées de l’extension Unicode ZIP sont vérifiées pendant l’analyse, non après la fin de l’archive. Résolution, détection et conversion par entrée vérifient l’annulation. Les données ZIP conservent au plus 16 MiB supplémentaires pour la table de vérification des noms effectifs ; un décodage ultérieur ne peut changer les noms figés.

La détection statistique ne peut reconstituer de manière unique des noms courts arbitraires ou des encodages anciens mélangés, ni reconnaître du texte déjà corrompu mais enregistré en Unicode valide par un autre logiciel. Unicode déclaré et encodage ancien peuvent coexister dans une archive ; plusieurs encodages non déclarés ne sont détectés qu’au mieux. Il n’existe actuellement aucun sélecteur manuel d’encodage. Les noms impossibles à décoder strictement, aux déclarations corrompues ou aux chemins anormaux restent affichés sous forme échappée et marqués pour examen, sans autoriser extraction, ouverture temporaire, export ou modification. L’identification correcte de toutes les archives historiques n’est pas garantie. Si des noms ne sont pas résolus, leur appartenance réelle aux répertoires peut être inconnue ; l’extraction par expansion de répertoire est donc refusée pour éviter d’omettre silencieusement des éléments dans un résultat réussi. Les fichiers réguliers résolus et explicitement sélectionnés restent extractibles individuellement.

<a name="xdzip-end-to-end-use-and-saving"></a>

## Utilisation de bout en bout et sauvegarde

L’arbre de répertoires, l’index des enfants directs, la recherche récursive, l’expansion de sélection de répertoire, le précontrôle, l’extraction, l’ouverture temporaire, l’export externe et la modification utilisent le même nom effectif. Après décodage, effectuer quand même les contrôles PortablePath, alias Unicode NFD ou de casse, doublons et autorisation de sortie. L’octet final multioctet Shift-JIS 0x5C n’est pas un séparateur de répertoire. Hormis les séparateurs propres à CAB et LHA, les véritables barres obliques inverses décodées restent refusées.

Après planification, les ajouts et déplacements dans ZIP, TAR et TAR.GZ ordinaires redéclarent en UTF-8 les noms à ancien encodage, extension Unicode et UTF-8 non marqué, tout en conservant positions d’origine et sémantique d’omission du plan logique de déplacement. ZIP conserve les données comprimées ; TAR utilise des chemins PAX UTF-8 explicites. Après écriture, analyser à nouveau, vérifier chaque nom et contenu, puis appliquer le remplacement existant sur le même volume. Si la suppression d’une déclaration BINARY pour un chemin PAX lors de la réécriture TAR changerait l’interprétation d’autres textes PAX non UTF-8, échouer explicitement et préserver l’archive originale. Les archives chiffrées et fractionnées utilisent les mêmes noms pour lecture et extraction ; les restrictions d’ajout et déplacement ne changent pas.

<a name="xdzip-reproducible-acceptance"></a>

## Acceptation reproductible

Utiliser des jeux d’essai de bytes bruts propres au projet et des archives encodées indépendamment avec la bibliothèque standard Python, couvrant UTF-8 non marqué, GBK, GB18030 à quatre octets, Big5, Shift-JIS contenant 0x5C, coréen, russe, ZIP CP437 et Unicode Path. Vérifier noms complets, octets originaux, répertoires, recherche, sélection, précontrôle, octets réellement produits, ouverture temporaire, export, ajout et déplacement d’archives ordinaires, AES et volumes fractionnés. Vérifier aussi déclarations endommagées, priorité des extensions, refus de chemins, collisions Unicode et de casse, annulation et budgets.

scripts/check-filename-encoding.py extrait des ZIP/TAR/TAR.GZ générés par Python via une sonde réelle du service. Python lit aussi indépendamment les archives réécrites par Rust pour contrôler indicateurs UTF-8/PAX, noms et contenus. Les données de test proviennent uniquement de constantes du code source, pas d’archives utilisateur. Les résultats réels et l’acceptation manuelle restante sont consignés dans les tickets et artefacts build/CI, pas comme état réussi dans ce contrat.

LHA conserve les limites originales des champs dirname/filename et les séparateurs dirname bruts `0xff`. La détection bornée partagée échantillonne séparément chaque champ complet. Décoder strictement chaque composant avant d’interpréter les barres inverses décodées et de composer `/` ; les limites originales empêchent d’emprunter des octets multioctets entre champs. L’expansion de répertoire et la sortie utilisent ce même nom effectif. Voir le [contrat LHA](lha-containers.md).
