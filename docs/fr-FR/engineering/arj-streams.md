<a name="xdzip-arj-compressed-streams-and-container-admission"></a>

# Flux comprimés ARJ et admission du conteneur

[English](../../engineering/arj-streams.md) | [简体中文](../../zh-CN/engineering/arj-streams.md) | [繁體中文](../../zh-TW/engineering/arj-streams.md) | [日本語](../../ja-JP/engineering/arj-streams.md) | [한국어](../../ko-KR/engineering/arj-streams.md) | Français | [Deutsch](../../de-DE/engineering/arj-streams.md) | [Español](../../es-ES/engineering/arj-streams.md) | [Português (Brasil)](../../pt-BR/engineering/arj-streams.md) | [Русский](../../ru-RU/engineering/arj-streams.md)

`ArjReader` fournit le décodage brut borné des méthodes ARJ 1–4. Store appartient au conteneur, pas à ce lecteur. `xdzip-codec-arj` fournit une structure physique complète et la livraison de membres vérifiés par CRC sur une source détenue par l’hôte. `ArjCodec` relie les profils admis au runtime, à la CLI et à la navigation, vérification et extraction de bureau. Le [registre des formats](../../../specs/engineering/archive-formats.json) déclare la limite en lecture seule ; création, réécriture, mots de passe et volumes ne sont pas disponibles.

<a name="xdzip-host-admission-and-names"></a>

## Admission par l’hôte et noms

Une proposition d’en-tête principal de douze octets ne publie jamais de format ni de lignes. Le runtime préserve d’abord la priorité des conteneurs forts validés et de ZIP/SFX, puis scanne entièrement les métadonnées ARJ proposées sous le compteur partagé d’entrée des sondes faibles. Un marqueur TAR brut non validé est désambiguïsé par une admission ARJ complète. Les hypothèses invalides ne publient rien ; les erreurs typées de l’hôte, l’annulation et les quotas restent déterminants.

L’adaptateur conserve les ordinaux physiques des membres, les octets de méthode et les noms bruts. Les noms marqués ANSI et les noms OEM ont des groupes d’encodage séparés partageant une limite de 1 MiB d’échantillonnage de champs complets, sans choix selon la locale ambiante. Décoder entièrement un nom multioctet avant d’interpréter les séparateurs. Seuls les hôtes de type DOS sans indicateur de chemin portable convertissent les barres inverses en barres obliques. Les barres inverses littérales portables ou Unix restent suspectes. Les dossiers explicites reçoivent une barre finale ; les noms suspects ou non résolus, méthodes inconnues et profils d’objets non pris en charge restent visibles sans autorité sur le contenu. Les noms décodés sont limités à 4 096 octets et les allocations conservées d’affichage/décodage à 16 MiB, ou aux limites inférieures du demandeur. Aucun attribut n’est restauré.

L’ouverture de contenu reprojette et compare l’ensemble complet des entrées figées avant livraison. Chaque enregistrement choisi doit correspondre exactement à sa ligne physique ; une sélection non prise en charge ou tout échec de décodage empoisonne l’instance de contenu. Vérification d’identité hôte, pré-vérification, préparation privée, annulation et publication finale gardent la sémantique commune du runtime. Les contrôles `supported_content` décodent complètement tous les membres ordinaires admis et vérifient tailles exactes et CRC-32 obligatoire. Les lignes non prises en charge empêchent le succès de toute l’archive. Le texte historique est livré comme les octets d’archive couverts par CRC, sans conversion de fin de ligne par l’hôte.

<a name="xdzip-sources-and-reference-boundary"></a>

## Sources et limites des références

Le [projet ARJ open source](https://arj.sourceforge.net/) fournit la [source 3.10.22](https://arj.sourceforge.net/files/arj-3.10.22.tar.gz), SHA-256 `589e4c9bccc8669e7b6d8d6fcd64e01f6a2c21fe10aad56a83304ecc3b96a7db`, et son [paquet Win32 3.10](https://arj.sourceforge.net/files/arjw_310.exe), SHA-256 `e5cc5d8f82c5f08246e1c0a8c5be77e5f796c2dd30f987e6bbb27a2c1df1e371`. La source comprend le texte GNU GPL version 2. Les observations limitées du format filaire utilisent `defines.h`, `decode.c` et `arj_arcv.c` ; la référence native reste inchangée et sous ses conditions d’origine, uniquement dans build/toolchain.

La comparaison indépendante utilise [7-Zip](https://www.7-zip.org/) officiel, avec observations bornées de son [gestionnaire ARJ](https://github.com/ip7z/7zip/blob/main/CPP/7zip/Archive/ArjHandler.cpp), blob Git `da890f279d35c00bc61d332ce55f4d96ecd6d373`. Aucune salle blanche stricte n’est revendiquée. L’implémentation Rust réutilise les primitives internes de bits, préfixes et LZ et exprime directement les champs unaires rapides. Sources de référence, binaires et moteurs de compression n’entrent pas dans les dépendances du produit. Les identités exactes des composants et entrées propres sont consignées avec les [échantillons](../../../tests/fixtures/arj/README.md).

<a name="xdzip-static-methods-13"></a>

## Méthodes statiques 1–3

Les méthodes 1–3 partagent le modèle filaire Huffman/LZ statique, même si les encodeurs natifs peuvent choisir différentes recherches. Le nombre de commandes de bloc sur 16 bits, l’alphabet de longueurs à 19 symboles, l’alphabet littéral/longueur à 510 symboles, l’alphabet de positions à 17 symboles et le compte de positions sur cinq bits réutilisent l’implémentation bornée de préfixes LZH par un profil ARJ explicite. Ils n’acquièrent ni les anciens champs d’arbre LH3 ni un identifiant public de méthode LHA. Les nombres de commandes de bloc nuls ne sont pas admis.

Les symboles littéraux sont 0–255 ; les autres donnent les longueurs `symbol - 253`, de 3 à 256. Le symbole de position zéro donne le décalage zéro ; sinon le décalage est `2^(symbol - 1)` plus autant de bits faibles. La distance réelle est ce décalage plus un, au plus 26624, et doit référencer des octets déjà émis. Un anneau de 32 KiB implémente l’historique logique borné ; son allocation plus grande n’élargit pas l’admission. Les espaces initiaux LHA restent séparés et ne peuvent pas initialiser la sortie ARJ. Le retour circulaire du dictionnaire et les copies chevauchantes gardent une sémantique séquentielle par octet. Les arbres canoniques complets, champs à symbole unique, séquences de zéros et profondeur de code de 16 bits conservent la validation et la comptabilisation du préfixe partagé.

<a name="xdzip-fast-method-4"></a>

## Méthode rapide 4

Le flux est de poids fort en premier. Un champ de longueur compte de 0 à 7 bits initiaux à un, avec arrêt à zéro sauf au maximum. Lire autant de bits faibles et ajouter `2^count - 1`. Zéro indique un littéral de huit bits ; une valeur non nulle indique une copie de cette valeur plus deux, de 3 à 256 octets. Le pointeur compte de même de 0 à 4 bits initiaux à un, avec une largeur basse `9 + count` et une base `512 * (2^count - 1)`. Le décalage plus un donne une distance maximale de 15872. Chaque référence doit viser l’historique déjà émis, avec un anneau de 16 KiB autorisant chevauchement et retour circulaire ; aucune réparation de l’historique initial n’est permise.

<a name="xdzip-limits-and-failures"></a>

## Limites et échecs

Les profils statiques admettent 32 KiB d’historique et 64 KiB supplémentaires pour entrée, tables et travail temporaire. Le décodage rapide admet 16 KiB d’historique et 32 KiB pour entrée, état et travail temporaire. Tailles comprimée et décodée ont chacune une limite stricte de 24 GiB, le travail une limite de 96 GiB ; les budgets inférieurs du demandeur prévalent. Les lectures de sortie sont d’au plus 8192 octets. Opérations de préfixe, travail d’historique et copies indépendantes de la source conservent une interrogation coopérative. `work_remaining()` prend en charge le compteur cumulé de travail réel du conteneur.

Les longueurs déclarées ne peuvent être dépassées. Les octets comprimés logiques doivent se terminer exactement ; le remplissage final inférieur à un octet est ignoré, les octets complets en trop ou l’entrée manquante échouent et aucun octet zéro EOF n’est inventé. Seul `Interrupted` sans charge associée est retenté. Les erreurs typées conservent leur première charge et terminent ; toute erreur de source, données malformées, quota ou annulation empoisonne le lecteur. Les lecteurs bruts n’ont aucune autorité sur chemins, secrets, exécution, publication sur le système de fichiers ou CRC.

<a name="xdzip-bounded-physical-containers"></a>

## Conteneurs physiques bornés

`inspect` valide une archive à décalage zéro, une largeur du premier en-tête principal de 30/34, des largeurs de fichier 30/34/46 et des blocs de base complets d’au plus 2600 octets. Noms et commentaires se terminent exactement, le champ de position du nom n’autorise pas la troncature et tous les CRC-32 de base et étendus doivent réussir. Chaque enregistrement admet au plus 256 blocs étendus de longueur sur 16 bits. Les attributs opaques de fichier sont validés et inclus dans l’identité des métadonnées, sans application ni exécution. Le terminateur exact `60 ea 00 00` doit finir la source physique ; aucun préfixe SFX, remplissage ou reste de récupération n’est déduit.

Les lectures de métadonnées sont limitées à 64 MiB, les entrées physiques à 100000, les noms bruts à 4096 octets chacun et 16 MiB au total, avec maintien des limites inférieures du demandeur. Aucun échec ultérieur ne publie d’index partiel. SHA-256 couvre toutes les métadonnées physiques et le terminateur, y compris commentaires, modificateurs et attributs ignorés ; `Reader::open` rescane et compare ce condensat, chaque enregistrement et la longueur physique avant sortie. Les erreurs de source restent terminales et ne deviennent pas des suppositions de données malformées. Les horodatages d’hôte Unix n’utilisent que le calendrier hôte facultatif ; les projections absentes deviennent zéro, tandis que les erreurs de rappel terminent.

Le profil de membre admet les versions requises jusqu’à 11, hôtes DOS/Unix/OS2/Win95/WinNT, méthodes 0–4 binaires ou texte historique non chiffrées et dossiers Store vides. Méthodes non prises en charge, versions futures, types spéciaux, modes spéciaux Unix et champs de chiffrement/continuation/chapitre restent explicites et ne peuvent être décodés. Les profils principaux de volumes/protection/sécurité/double nom/chiffrement/chapitre et les structures de fichier à double nom ne sont pas admis. Modes d’accès et attributs n’autorisent jamais permissions ou création de fichiers spéciaux. Le texte historique est livré en octets couverts par la taille et le CRC de l’archive, comme 7-Zip ; ARJ Win32 natif étend au contraire LF en CRLF pour ce profil. Aucune conversion de fin de ligne dépendante de l’hôte n’est effectuée.

Un lecteur conserve 24 GiB d’entrée réelle cumulée, y compris le nouveau scan des métadonnées, 24 GiB de sortie décodée et 96 GiB de travail. Le travail CRC par bit de la sortie déclarée est réservé avant de prêter le reste au décodeur avec état. Les membres réussis comptabilisent exactement cette réserve plus le travail réel de construction/modèle/Store ; un échec termine sans remboursement. La comptabilisation du récepteur du demandeur inclut la construction et ne commence pas après l’initialisation. Toutes les lectures restent limitées à 8192 octets, les instantanés et indices physiques sont vérifiés et toute erreur de contenu, CRC, récepteur, annulation ou quota empoisonne l’instance.

<a name="xdzip-required-container-and-host-admission"></a>

## Admission requise du conteneur et de l’hôte

En-têtes principal et de fichier ARJ, dispositions bornées des premiers en-têtes, CRC-32, chaînes d’en-têtes étendus, plages comprimées exactes et marqueur terminal de longueur zéro doivent tous être validés avant publication d’un index complet. Les ID physiques conservent les doublons. Noms, pages de codes, séparateurs, dossiers, types texte historique ou binaire, objets spéciaux et méthodes inconnues exigent une projection et une admission explicites du contenu ; un marqueur reconnu ne l’autorise pas.

Chaque CRC de fichier déclaré doit réussir après décodage complet. Les en-têtes et champs de sécurité facultatifs de l’archive ne signifient pas une authenticité cryptographique. Mots de passe, méthodes chiffrées, volumes, entrées continuées, chapitres, fins de récupération/sécurité et enveloppes auto-extractibles exigent des profils et preuves séparés. Aucun script, commande automatique, attribut externe, chemin voisin ou indicateur exécutable n’accorde d’autorité. Gel de la source par l’hôte, extraction sélectionnée, budgets cumulés d’entrée/travail/sortie, préparation privée, annulation, protection des anciens fichiers et publication finale suivent le [contrat d’extraction](zip-safe-extraction.md).

Le générateur d’échantillons ne crée que des entrées binaires propres au projet, consigne méthodes demandées et réelles et lit indépendamment la sortie complète avec ARJ natif et 7-Zip. Un repli Store natif reste Store. Il ne normalise que les horodatages non chiffrés, le modificateur inutilisé dérivé du temps et les CRC d’en-tête correspondants ; octets comprimés et CRC du contenu restent intacts. Les entrées vides natives produisent des membres Store et ne prouvent pas un flux comprimé vide encodé. L’admission du produit exige aussi l’automatisation réelle runtime/CLI/Qt et des candidats/dispositions applicables selon le [périmètre QA](../../../developers/qa/README.md).
