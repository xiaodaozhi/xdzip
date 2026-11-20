<a name="xdzip-sources-and-licenses-for-disk-images-and-other-archive-formats"></a>

# Sources et licences des images disque et autres formats d’archive

[English](../../engineering/disk-image-archive-format-sources.md) | [简体中文](../../zh-CN/engineering/disk-image-archive-format-sources.md) | [繁體中文](../../zh-TW/engineering/disk-image-archive-format-sources.md) | [日本語](../../ja-JP/engineering/disk-image-archive-format-sources.md) | [한국어](../../ko-KR/engineering/disk-image-archive-format-sources.md) | Français | [Deutsch](../../de-DE/engineering/disk-image-archive-format-sources.md) | [Español](../../es-ES/engineering/disk-image-archive-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/disk-image-archive-format-sources.md) | [Русский](../../ru-RU/engineering/disk-image-archive-format-sources.md)

Cette page décrit les références techniques publiques, restrictions de licence et conditions de prise en charge de DAA, ISZ, ZPAQ, PEA, BOND, J2J et formats associés. Une description publique de format n’autorise pas la distribution d’un décodeur et une implémentation de référence n’est pas nécessairement une dépendance produit redistribuable. Aucune des références ci-dessous n’est liée à xdzip. Avant l’entrée d’un format au registre produit, révision source, texte de licence, échantillons complets, limites de ressources et comportement en cas d’échec doivent être établis.

<a name="xdzip-daa-10"></a>

## DAA 1.0

- **Spécification publique :** Aucune spécification DAA 1.0 publiée par le fournisseur n’a été trouvée. Des descriptions publiques identifient DAA comme image disque/archive propriétaire de PowerISO et décrivent un en-tête, une table de longueurs de blocs et des blocs ISO comprimés indépendamment. Les documents publics couvrent les variantes zlib et LZMA et des versions internes ultérieures comme 110, mais ne définissent pas un contrat 1.0 stable.
- **Implémentation de référence :** `daa2iso` 0.1.7e, paquet Debian/Fedora sous GPL-2.0-or-later. Sa source ne sert qu’à la recherche historique. Aucun commit Git amont ni spécification du fournisseur ne peut actuellement être fixé depuis un dépôt public faisant autorité.
- **Licence et redistribution :** L’implémentation est sous GPL-2.0-or-later. Sa licence ne donne pas de droits sur la technologie propriétaire DAA et le projet ne dispose d’aucune autorisation indépendante enregistrée pour livrer un décodeur dans xdzip.
- **Échantillons complets :** Les exemples publics sont souvent des images disque ou archives multiparties dont provenance et version ne sont pas fixées. Aucun corpus redistribuable couvrant anciens en-têtes, version 110, ordre multipartie, zlib, LZMA, obfuscation, sommes de contrôle et blocs mal formés n’est actuellement admis.
- **Faisabilité en salle blanche :** Une reconstruction ISO bornée peut être techniquement possible après établissement d’un format filaire stable et décision juridique. La rétro-ingénierie d’un seul décodeur GPL ne suffit pas comme preuve pour une implémentation produit.
- **Limite des capacités :** Définitivement non pris en charge par décision produit. Ne pas ajouter de détecteur, codec, registre, type MIME, association, lecteur ou écrivain DAA. La recherche historique n’autorise pas une admission future.
- **Conclusion :** **Exclu définitivement par décision produit.** xdzip ne poursuivra pas DAA. Les résultats sur sources, licences et échantillons restent une recherche historique, non des conditions de livraison en attente.

<a name="xdzip-isz"></a>

## ISZ

- **Spécification publique :** EZB Systems, *ISZ File Format Specification*, version 1.00, révisée le 2006-07-03, documente la signature `IsZ!`, l’en-tête fixe, la table facultative de définition des segments, la table de définition des blocs et les blocs de données. Les champs d’en-tête comprennent taille et nombre de secteurs, mode de mot de passe, taille des segments, nombre et taille des blocs, longueur des pointeurs, numéro de segment et décalages des tables et données. Les indicateurs de bloc distinguent zéro, stockage, zlib et BZip2. La spécification est reproduite dans [la copie du forum ImgBurn](https://forum.imgburn.com/topic/14260-isz-images-supporting/#comment-115553).
- **Implémentations de référence :** Filtre ISZ de CDEmu `libmirage`, commit `39fcffe09739e1dc40bef4781646f9ffb8b7dc4e`, et `IszImageStream` de Play-, commit `5bb010f27d8e2309f37b1f8eba0f74ecd546b29b`. Les deux servent uniquement à l’analyse de référence. `Helias/isz-tool` est un lecteur Python, mais aucun commit immuable faisant autorité ni enregistrement de licence n’a pu être obtenu dans cet environnement.
- **Licence et redistribution :** La section 8 de la spécification EZB indique qu’incorporer la technologie ISZ à un produit exige de contacter EZB pour une licence gratuite soumise à conditions. Un texte public ou un lecteur indépendant sous licence permissive ne supprime pas cette condition de licence produit. CDEmu et Play- ne sont pas des dépendances de xdzip.
- **Échantillons complets :** Des blocs Store, zlib et BZip2 valides peuvent être générés par des outils publics, mais une matrice redistribuable complète exige aussi images simples et segmentées, blocs nuls, toutes les longueurs de pointeurs, blocs de fin exacts, modes de mot de passe, obfuscation des tables, troncature, décalages et modification de source. Aucun tel corpus propre n’est enregistré dans le dépôt.
- **Faisabilité en salle blanche :** Le sous-ensemble non chiffré, à volume unique et en lecture seule est techniquement séparable de la navigation ISO. Une première implémentation pourrait décoder les quatre types de blocs documentés en un fichier virtuel `image.iso`, sans montage ni exécution de démarrage. Toutefois, autorisation juridique, référence immuable fixée et ensemble complet d’échantillons doivent précéder le code.
- **Limite des capacités :** Avant satisfaction de ces conditions, ne pas enregistrer `.isz`, ne pas déduire la prise en charge de `IsZ!` et ne pas exposer mot de passe, AES, segmentation, écriture ou navigation optique directe. En cas d’admission ultérieure, le décodeur doit valider en-tête/version, tables de pointeurs et segments, règles XOR/désobfuscation, bornes de blocs et données, arithmétique de taille logique, terminaison complète des blocs, annulation et limites d’entrée/travail/sortie.
- **Conclusion :** **Exclu par décision produit.** xdzip ne poursuivra pas ISZ ni ne contactera EZB pour une licence produit dans ce lot. Aucun détecteur `.isz` ni entrée de registre ne peut être ajouté sans revoir la décision.

<a name="xdzip-zpaq"></a>

## ZPAQ

- **Spécification publique :** Matt Mahoney, *The ZPAQ Open Standard Format for Highly Compressed Data*, versions 2.03 et 2.06. Le format stocke un programme de décompression en bytecode ZPAQL et prend en charge blocs solides, journalisation, dédoublonnage, enregistrements SHA-1, chiffrement aux niveaux ultérieurs et archives multiparties.
- **Implémentation de référence :** `zpaq/zpaq`, tag `6.22`, et les sources `libzpaq` correspondantes. Le tag est une référence de recherche fixée pour cette évaluation ; un hachage de commit et du texte de licence doit être enregistré avant toute implémentation.
- **Licence et redistribution :** L’implémentation de référence et les fichiers de configuration sont sous GPLv3. Ils ne peuvent pas être liés au produit actuel selon la politique excluant les runtimes d’archive tiers. Le standard public permet l’analyse en salle blanche, mais la machine virtuelle de décodage et le corpus de configurations exigent une implémentation et une revue séparées.
- **Échantillons complets :** Un corpus utile doit couvrir blocs de niveaux 1 et 2, programmes ZPAQL arbitraires, mises à jour journalisées, dédoublonnage, enregistrements chiffrés et multiparties, vérification SHA-1 et limites mal formées de machine virtuelle ou de blocs. Le dépôt ne possède pas ce corpus propre.
- **Faisabilité en salle blanche :** Possible en théorie, mais il s’agit d’un projet de machine virtuelle et de comptabilisation des ressources plutôt que d’un petit analyseur de conteneur. Les limites sûres doivent facturer interprétation du bytecode, mémoire du modèle, sortie décompressée, historique et état de dédoublonnage.
- **Limite des capacités :** Une première tranche future serait en lecture seule, non chiffrée, sans multipartie ni journalisation, avec un sous-ensemble de machine virtuelle déclaré. Aucune création, mise à jour, gestion de mot de passe ou extraction solide illimitée n’est implicite.
- **Conclusion :** **Exclu par décision produit.** xdzip n’implémentera pas la machine virtuelle ZPAQL et n’acceptera aucun profil ZPAQ qui en dépend. Aucun détecteur `.zpaq` ni entrée de registre ne peut être ajouté sans revoir la décision.

<a name="xdzip-pea"></a>

## PEA

- **Spécification publique :** PeaZip, spécification *Pack, Encrypt, Authenticate*, couvrant PEA 1.x et PEA 2.0 encore proposé, flux, contrôles de volumes et objets, marqueurs de fin et niveaux PCOMPRESS. La spécification est publiée sous [`pea_help.pdf`](https://peazip.github.io/pea_help.pdf) et résumée dans [pea-file-format.html](https://peazip.github.io/pea-file-format.html). La page officielle actuelle place explicitement les spécifications PEA et PCOMPRESS dans le domaine public.
- **Implémentation de référence :** `peazip/PeaZip`, commit `81b31f7e487b462980a52ac0f2e0b3cf700fe799`, comprenant `peazip-sources/dev/pea_utils.pas` et `unit_pea.pas`. L’implémentation est sous LGPLv3 et les unités cryptographiques Wolfgang Ehrhardt incluses sous licence zlib/libpng. L’URL de publication des sources est [peazip-11.4.0.src.zip](https://github.com/peazip/PeaZip/releases/download/11.4.0/peazip-11.4.0.src.zip). La source fixée sert seulement à l’analyse de référence.
- **Licence et redistribution :** La spécification de format elle-même ne bloque pas la licence produit, mais le moteur LGPLv3 et ses dépendances cryptographiques ne peuvent pas être copiés ou liés par défaut au produit actuel. Une implémentation Rust en salle blanche doit utiliser la description du format dans le domaine public et des algorithmes choisis indépendamment ; le binaire de référence reste réservé aux tests.
- **Échantillons complets :** Le profil admis utilise des échantillons Store complets écrits par le projet couvrant fichiers, répertoires, noms UTF-8 valides, données vides et non vides, troncature, octets finaux, modification de source et limites. Un corpus plus large doit encore inclure PEA 1.x et 2.0, plusieurs flux, limites de volumes, niveaux PCOMP, échecs d’authentification, en-têtes chiffrés, marqueurs de fin mal formés et grandes tailles déclarées ; ces profils restent hors du périmètre produit.
- **Faisabilité en salle blanche :** Un lecteur Store/Deflate non chiffré et à flux unique est techniquement plausible, mais authentification et chiffrement doivent faire l’objet d’un travail de sécurité distinct. La conception d’intégrité du format interdit de présenter la seule analyse structurelle comme vérification du contenu.
- **Limite des capacités :** Une admission future commencerait en lecture seule, sans mot de passe, chiffrement, plusieurs volumes, création ni modification. Chaque flux extrait exigerait les contrôles de condensé déclaré et d’authentification terminale avant publication.
- **Conclusion :** **Admis comme profil conservateur en lecture seule.** xdzip admet maintenant PEA 1.x, un volume physique, noms UTF-8, fichiers/répertoires stockés PCOMPRESS0 et aucune vérification d’objet, de flux ou de volume. Mots de passe, flux authentifiés ou chiffrés, algorithmes de somme, PCOMPRESS1–3, volumes divisés, PEA 2.0, création et modification restent non pris en charge. La référence LGPL reste réservée aux tests et n’est pas liée au produit.

<a name="xdzip-bond"></a>

## BOND

- **Spécification publique :** Aucune spécification, signature, version ou contrat de compression d’archive Bandizip BOND n’a été trouvé. Des notes récentes de Bandizip mentionnent des vulnérabilités dans « certains fichiers BOND », sans définir le format filaire. Les bases publiques d’extensions `.bond` décrivent surtout les données structurées Microsoft Bond, qui sont un autre format de sérialisation.
- **Implémentation de référence :** Aucune implémentation avec identité d’archive faisant autorité, révision fixe et licence redistribuable n’a été trouvée.
- **Licence et redistribution :** Inconnue. Aucun produit ni décodeur ne peut être admis tant que propriétaire du format, droits technologiques et conditions de référence sont inconnus.
- **Échantillons complets :** Aucun échantillon d’une source fiable et versionnée. Une extension de fichier seule n’identifie pas un échantillon acceptable.
- **Faisabilité en salle blanche :** Impossible à évaluer sans signature et suffisamment d’échantillons complets distinguant une archive des données Microsoft Bond ou d’une autre charge binaire.
- **Limite des capacités :** Aucune détection, enregistrement de suffixe, navigation, extraction, vérification, création, prise en charge de mot de passe ou volumes.
- **Conclusion :** **Exclu par décision produit et non confirmé comme format d’archive Bandizip.** Aucun détecteur `.bond` ni entrée de registre ne sera ajouté dans ce lot.

<a name="xdzip-j2j"></a>

## J2J

- **Spécification publique :** Aucune spécification, signature, version ou trace de développeur d’archive J2J n’a été trouvée. D’anciennes listes de fonctions Bandizip mentionnent J2J en extraction seule, tandis que les pages officielles modernes ne donnent aucune définition technique. D’autres résultats J2J concernent des ensembles de données job-to-job ou des outils JPEG-to-JPEG sans rapport.
- **Implémentation de référence :** Aucune implémentation de révision fixe et licence identifiée ne peut être reliée à un lecteur d’archives Bandizip.
- **Licence et redistribution :** Inconnue. Les pages d’extensions tierces et anciennes listes de logiciels ne constituent pas une autorisation de licence.
- **Échantillons complets :** Aucun ensemble d’échantillons fiable et redistribuable n’existe dans le dépôt. Un nom `.j2j` ne peut établir l’identité d’un conteneur.
- **Faisabilité en salle blanche :** Impossible à évaluer avant documentation de la signature, du modèle d’enregistrement et de la compression ou du chiffrement, et disponibilité d’un corpus complet.
- **Limite des capacités :** Aucune détection, enregistrement de suffixe, navigation, extraction, vérification, création, prise en charge de mot de passe ou volumes.
- **Conclusion :** **Exclu par décision produit et non confirmé comme format d’archive Bandizip.** Aucun détecteur `.j2j` ni entrée de registre ne sera ajouté dans ce lot.

<a name="xdzip-requirements-for-format-support"></a>

## Conditions de prise en charge des formats

1. Enregistrer une spécification faisant autorité ou un protocole de salle blanche revu juridiquement. Un blog, une base d’extensions ou la seule source d’un décodeur ne suffit pas.
2. Fixer chaque source de recherche à un commit immuable ou un hachage de contenu et enregistrer le texte de licence applicable.
3. Construire une matrice d’échantillons complète et redistribuable pour chaque méthode, version, mode de volume, état de chiffrement et condition terminale revendiqués. Des exemples limités aux en-têtes ou à Store ne prouvent pas l’interopérabilité.
4. Définir un modèle de ressources borné pour lectures d’entrée, métadonnées, travail de décompression, mémoire, sortie logique, historique, état cryptographique et annulation. Rejeter décalages mal formés, cycles, dépassements et modifications de source avant publication.
5. Garder chaque décodeur indépendant de Qt et des API de montage, démarrage, installateur, scripts, registre et exécution arbitraire de processus.
6. Relier runtime, CLI, associations et documentation utilisateur seulement après réussite des contrôles ciblés sur codec, identité d’instantané, comportement exact des données et classification des échecs.
7. Un candidat bloqué reste visible comme bloqué dans l’évaluation. Il ne doit pas recevoir de suffixe, type MIME ou capacité produit au seul motif que Bandizip ou une liste tierce le nomme.

Le contrôle du dépôt `python scripts/test-m10-source-gates.py` vérifie que les six sections bilingues conservent les champs de preuve requis, que chaque conclusion reste explicitement bloquée, exclue ou admise et qu’aucun candidat bloqué ou exclu n’est entré dans `specs/engineering/archive-formats.json`.
