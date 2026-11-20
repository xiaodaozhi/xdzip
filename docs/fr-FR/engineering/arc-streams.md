<a name="xdzip-arc-containers-and-compressed-stream-profiles"></a>

# Conteneurs ARC et profils de flux comprimés

[English](../../engineering/arc-streams.md) | [简体中文](../../zh-CN/engineering/arc-streams.md) | [繁體中文](../../zh-TW/engineering/arc-streams.md) | [日本語](../../ja-JP/engineering/arc-streams.md) | [한국어](../../ko-KR/engineering/arc-streams.md) | Français | [Deutsch](../../de-DE/engineering/arc-streams.md) | [Español](../../es-ES/engineering/arc-streams.md) | [Português (Brasil)](../../pt-BR/engineering/arc-streams.md) | [Русский](../../ru-RU/engineering/arc-streams.md)

`ArcReader` implémente le décodage brut borné des méthodes ARC 3–9. Les méthodes 1 et 2 sont des profils Store du conteneur. `xdzip-codec-arc` et `ArcCodec` relient structure complète et CRC-16 obligatoire au runtime, à la CLI et à la navigation, vérification et extraction de bureau. Le [registre des formats](../../../specs/engineering/archive-formats.json) déclare cette portée en lecture seule ; création, modification, mots de passe et volumes sont indisponibles.

<a name="xdzip-container-and-host-admission"></a>

## Admission du conteneur et de l’hôte

Le profil admis utilise la structure ARC 5.x ordinaire ; les métadonnées supplémentaires ARC 6.x et extensions Spark ne sont pas revendiquées. Chaque membre commence par `0x1a` et son octet de méthode d’origine. La méthode 1 utilise 25 octets ; les autres méthodes non nulles utilisent l’en-tête ordinaire de 29 octets. Le champ de nom de treize octets doit contenir un nom non vide terminé par NUL. Tous les entiers sont en ordre petit-boutiste : longueur comprimée à 15, date/heure DOS à 19/21, CRC-16 obligatoire à 23 et longueur décodée ordinaire à 25. L’ancien Store hérite de la longueur comprimée. Les deux méthodes Store exigent des longueurs égales. Les plages comprimées doivent tenir, le marqueur suivant est obligatoire et le terminateur exact de deux octets `1a00` doit coïncider avec l’EOF physique. Aucune resynchronisation, recherche de préfixe, remplissage ou fichier voisin n’est admis.

Le conteneur préserve tous les doublons physiques et méthodes inconnues. Les lectures de métadonnées ont une limite stricte de 64 MiB, au plus 100000 enregistrements, des noms bruts bornés par l’appelant et une limite physique d’entrée de 24 GiB. SHA-256 lie chaque octet d’en-tête et le terminateur, y compris le remplissage de nom ignoré, lors de la réouverture d’un `Index` exact. Les condensats d’en-tête n’authentifient pas le contenu. Chaque lecteur possède des compteurs cumulés de 24 GiB d’entrée réelle et d’octets décodés et 96 GiB de travail ; il réserve le travail en octets du CRC par table avant décodage et termine à toute erreur sans remboursement ni publication d’index partiel.

L’hôte conserve la priorité des conteneurs forts et ZIP/SFX et valide complètement les propositions ARC faibles sous le compteur partagé de sondes. Les noms sont décodés comme champs complets avant conversion des barres inverses DOS, selon la décision d’encodage d’archive bornée partagée. Les octets d’origine et ID de méthode restent disponibles. Les noms suspects/non résolus et méthodes inconnues restent visibles sans autorité sur le contenu. ARC n’a aucun profil admis de dossier, lien, restauration de mode ou objet externe. L’ancien XOR n’a pas d’indicateur de chiffrement ; validation de flux, taille et CRC restent obligatoires, sans deviner le mot de passe ni repli. Ce profil ne revendique aucun enregistrement MIME Linux dédié.

L’ouverture du contenu reprojette et compare les entrées figées ; les ID sélectionnés désignent les lignes physiques et doivent correspondre exactement. Pré-vérification runtime, identité de source hôte figée, préparation privée, annulation, protection des anciens fichiers et publication finale restent déterminantes. La vérification complète `supported_content` décode réellement chaque membre ordinaire admis, valide taille exacte et CRC-16 et échoue sur les lignes non prises en charge. Une sélection physique prise en charge peut néanmoins être extraite à côté d’un voisin non pris en charge.

<a name="xdzip-sources-and-references"></a>

## Sources et références

Le port Unix ARC original maintenu par Howard Chu et Anibal Monsalve Salazar est fixé au [commit `70f0b93516d0ee222db5cf1241aad0db1b721c0d`](https://github.com/ani6al/arc/tree/70f0b93516d0ee222db5cf1241aad0db1b721c0d). Son `Arcinfo` consigne la description de structure ARC 5.12 de Robert Freed. Les observations limitées du format filaire utilisent `arcunp.c`, `arcusq.c` et `arclzw.c`. Les fichiers `LICENSE` et `COPYING` des auteurs indiquent une licence GPL. Les algorithmes de référence sont compilés inchangés uniquement sous build/toolchain avec un adaptateur de test propre au projet. Ils ne sont ni compilés dans le produit ni déployés avec lui.

La comparaison indépendante utilise nomarch 1.4 de Russell Marks, sous GPL version 2 ou ultérieure. Son paquet source original est disponible dans le [dépôt source Debian](https://deb.debian.org/debian/pool/main/n/nomarch/), SHA-256 `fe20da34e0d3ba0cf6388701f44ac22224cf65130ddbc5fcbc27bc4949a6e1ad`, 22411 octets, conforme au descripteur source du mainteneur. Aucune salle blanche stricte n’est revendiquée. Le code produit exprime les champs filaires avec les primitives propres de bits, entrée, travail et allocation. Le [registre des échantillons propres](../../../tests/fixtures/arc/README.md) distingue sortie de l’encodeur natif, entrées filaires rédigées, comparaisons indépendantes des octets complets et limites des références.

<a name="xdzip-method-scope"></a>

## Portée des méthodes

| Méthode | Profil filaire | Étape suivante |
| --- | --- | --- |
| 1 | Ancienne structure de 25 octets à longueur d’origine implicite | Store |
| 2 | Structure ordinaire de 29 octets | Store |
| 3 | Octets avec séquences DLE `0x90` et DLE échappé | Aucune |
| 4 | Arbre squeeze à enfants signés explicites, codes de poids faible en premier et symbole EOF 256 | RLE de méthode 3 |
| 5 | Codes fixes 12 bits de poids fort en premier, dictionnaire haché d’origine | Aucune |
| 6 | Même dictionnaire haché d’origine | RLE de méthode 3 |
| 7 | Codes fixes 12 bits, hachage amélioré | RLE de méthode 3 |
| 8 | Octet initial de largeur maximale 12, LZW groupé de 9–12 bits de poids faible en premier | RLE de méthode 3 |
| 9 | Sans en-tête de largeur, LZW groupé de 9–13 bits de poids faible en premier | Aucune |

Les méthodes inconnues, extensions Spark, mots de passe et structures SFX exigent une admission séparée du conteneur. Un nom d’algorithme commun n’autorise pas l’ordre des bits, le dictionnaire, l’alignement, la réinitialisation ou l’étape RLE d’une autre méthode.

<a name="xdzip-rle-and-squeeze-trees"></a>

## RLE et arbres squeeze

Un octet normal est émis et devient l’historique de répétition. DLE suivi de zéro émet un DLE littéral sans changer cet historique. Un compte non nul émet `count - 1` copies du précédent octet normal ; un compte sans historique est invalide. L’échappement doit être complet et la taille décodée exacte borne chaque répétition. L’état RLE survit aux chaînes du dictionnaire et aux blocs de sortie du demandeur.

Squeeze commence par un nombre de nœuds petit-boutiste, au plus 256, suivi de deux enfants signés de 16 bits par nœud. Les enfants non négatifs désignent des nœuds internes ; les valeurs négatives codent le symbole `-child - 1`, de 0 à 256. La racine est zéro ; l’ancienne forme sans nœud a des enfants EOF implicites. Le graphe complet doit être un arbre enraciné sans cycle, enfant interne partagé, nœud interne inaccessible ou arête hors limites, et EOF doit être accessible. Parcourir les bits de poids faible en premier jusqu’à EOF. L’entrée comprimée doit finir dans le même octet que ce terminateur ; les bits finaux inutilisés sont ignorés. Décoder jusqu’à la taille déclarée ne remplace pas le contrôle EOF.

<a name="xdzip-fixed-dictionaries"></a>

## Dictionnaires fixes

Le dictionnaire de 4096 cases insère d’abord les chaînes d’octets 0–255 avec le prédécesseur `0xffff`. Les codes identifient les cases physiques de hachage. L’ancien hachage utilise les douze bits centraux du carré de `((prefix + suffix) mod 65536) | 0x0800`. La méthode 7 multiplie à la place la somme sur 16 bits par 15073 et prend les douze bits faibles. Ces constantes définissent des identifiants filaires historiques.

Les collisions suivent la chaîne de cases existante, commencent la recherche 101 cases après son dernier élément, puis cherchent linéairement avec retour circulaire. Chaque sonde et expansion de préfixe est bornée et comptabilisée. Chaque nouvelle chaîne est la précédente suivie du premier octet de la chaîne courante. Un code absent n’est admis que pour la case KwKwK exacte nouvellement insérée. Un dictionnaire plein cesse de croître. Les codes de douze bits placent le demi-octet haut en premier à travers les octets, contrairement aux profils LSB ultérieurs. Le dernier demi-octet inutilisé est ignoré ; les octets de code manquants et les octets complets supplémentaires sont invalides.

<a name="xdzip-dynamic-dictionaries"></a>

## Dictionnaires dynamiques

Le premier code doit être un octet. Le code 256 est CLEAR et 257 commence les ajouts séquentiels au dictionnaire. La largeur croît aux seuils du dictionnaire jusqu’au maximum de la méthode. Les codes sont regroupés par huit ; un changement de largeur ou CLEAR termine l’ancien groupe, consomme ses bits d’alignement inutilisés et commence un nouveau groupe. Ces bits ne sont pas spécifiés. CLEAR réinitialise la prochaine case et la largeur, exige un littéral suivant et ne peut pas emprunter une chaîne antérieure à la réinitialisation. Aucun code EOF ou octet synthétique n’est fourni.

Croissance du dictionnaire, KwKwK, fonctionnement à largeur maximale et réinitialisations adaptatives sont distincts de l’étape RLE. Les entrées natives de référence à LZW forcé établissent toutes les largeurs et les réinitialisations réelles. Les entrées CLEAR rédigées couvrent indépendamment chacune des huit positions de l’ancien groupe.

<a name="xdzip-budgets-and-failures"></a>

## Budgets et échecs

Les longueurs comprimée et décodée ont chacune une limite stricte de 24 GiB et le travail une limite de 96 GiB. Les limites inférieures du demandeur prévalent. L’admission de mémoire privée complète est de 16 KiB pour la méthode 3, 32 KiB pour la méthode 4, 64 KiB pour les méthodes 5–7 et 96 KiB pour les méthodes 8–9. Ces limites incluent le tampon d’entrée de 8192 octets, dictionnaires, expansion inverse, validation de graphe et état.

L’initialisation du dictionnaire, les bits consommés, arêtes d’arbre, sondes de hachage, expansion de préfixe, livraison de pile et travail de sortie/répétition sont comptabilisés. `work_remaining()` inclut le travail du constructeur pour un compteur cumulé possédé par le conteneur. L’interrogation coopérative couvre les travaux dépendants ou non de l’entrée. Les lectures de sortie sont d’au plus 8192 octets. Entrée courte, erreurs typées de l’hôte, épuisement de quota, annulation, codes/arbres invalides, données finales et désaccord de taille décodée terminent l’instance ; la première erreur conserve sa charge et les lectures suivantes ne reprennent pas. Les lecteurs terminés renvoient un EOF stable sans dépenser de travail supplémentaire.

Le CRC-16 obligatoire du contenu et la structure physique complète appartiennent au conteneur ARC. Le succès brut seul n’authentifie pas le contenu et n’accorde aucune autorité sur système de fichiers, chemin, processus, fichier voisin ou publication.
