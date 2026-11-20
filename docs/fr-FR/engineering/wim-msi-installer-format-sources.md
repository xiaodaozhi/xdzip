<a name="xdzip-wim-msi-and-installer-format-sources-and-licenses"></a>

# Sources et licences des formats WIM, MSI et des programmes d’installation

[English](../../engineering/wim-msi-installer-format-sources.md) | [简体中文](../../zh-CN/engineering/wim-msi-installer-format-sources.md) | [繁體中文（台灣）](../../zh-TW/engineering/wim-msi-installer-format-sources.md) | [日本語](../../ja-JP/engineering/wim-msi-installer-format-sources.md) | [한국어](../../ko-KR/engineering/wim-msi-installer-format-sources.md) | Français | [Deutsch](../../de-DE/engineering/wim-msi-installer-format-sources.md) | [Español](../../es-ES/engineering/wim-msi-installer-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/wim-msi-installer-format-sources.md) | [Русский](../../ru-RU/engineering/wim-msi-installer-format-sources.md)

Cette page décrit les références techniques, les restrictions de licence et les exigences liées à la prise en charge des formats WIM, MSI et des programmes d’installation. Les références restent dans `build/toolchain/references` ou renvoient à des spécifications publiques ; aucune ne constitue une dépendance du produit.

<a name="xdzip-wim"></a>

## WIM

- Implémentation de référence : `ebiggers/wimlib`, commit `cd5e231c348c255ae5088873b5a66ee0eb96fa07`.
- Le projet propose la licence GPLv3+ et, sous conditions, la licence LGPL-2.1+ pour sa bibliothèque. Le choix de LGPL suppose notamment que des dépendances tierces telles que `libntfs-3g` n’imposent pas les conditions GPL.
- WIM est un conteneur d’images, et non une archive ordinaire. Un lecteur soumis à des limites doit traiter les tables de ressources, les flux de métadonnées, les ressources découpées en blocs, la compression Xpress/LZX/LZMS, les tables d’intégrité, les métadonnées de liens physiques et la sélection d’image, sans montage ni restauration des droits de sécurité.

L’implémentation de référence est utile pour étudier le format et les échantillons, mais la politique actuelle excluant les dépendances d’exécution tierces interdit de la lier à xdzip. WIM est **exclu par décision produit** : le projet ne prend pas en charge le lecteur propre aux images, le modèle d’amplification à la décompression, les ressources d’intégrité, la sémantique des liens physiques et le corpus complet nécessaires. Aucune entrée de registre, MIME, d’exécution ou de CLI ne peut être ajoutée sans réexamen explicite de cette décision.

<a name="xdzip-compound-file-and-msi"></a>

## Compound File et MSI

- Microsoft [MS-CFB] est la principale spécification publique : https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b . Elle publie des révisions PDF/DOCX et une déclaration de droits de propriété intellectuelle autorisant l’implémentation de la technologie décrite. Les licences de brevets restent régies par les programmes Microsoft applicables.
- Avant toute interprétation propre à MSI, un lecteur CFB doit gérer l’allocation des secteurs et FAT/DIFAT, le parcours de l’arbre rouge-noir des répertoires, l’allocation des mini-flux, les limites de taille des flux et la détection des cycles.

Un lecteur autonome des métadonnées CFB est techniquement réalisable et existe désormais dans la partie en lecture seule `xdzip-codec-cfb`. Il vérifie les en-têtes CFB 3.0/4.0, notamment les tailles de secteur de 512 et 4096 octets, les chaînes FAT/DIFAT, l’arbre des répertoires, l’allocation des mini-flux, les limites de taille des flux et les cycles avant d’exposer les lignes des stockages et des flux. La copie brute d’un flux n’est possible qu’après correspondance avec l’instantané immuable des métadonnées. Les tables MSI, cabinets incorporés, actions personnalisées et transformations restent hors contrat : aucun programme d’installation n’est exécuté, aucune action personnalisée n’est interprétée, aucun paquet n’est installé et aucune écriture en retour n’a lieu. Le suffixe `.msi` ne confère aucun droit d’exécution. Les échantillons et les preuves d’exécution sont automatisés dans les tests du codec et de l’environnement d’exécution.

<a name="xdzip-nsis-and-other-sfx"></a>

## NSIS et autres SFX

- Source de référence NSIS : `kichik/nsis`, commit `a96303eae3115221471bb127fa916a13d8bf9ecd`.
- Son fichier `COPYRIGHT` indique zlib/libpng pour la majeure partie des sources et le compresseur zlib, la licence bzip2 pour le module bzip2 et la Common Public License 1.0 pour le module LZMA.
- Les fichiers NSIS sont des programmes d’installation exécutables, avec des scripts et des données incorporées. Un analyseur doit rester en dehors de l’exécution, borner les plages d’en-tête et de données, et distinguer les méthodes de compression du comportement d’installation.
- La source publique définit l’enregistrement fixe `firstheader` : `flags`, `0xDEADBEEF`, les mots de signature `NullsoftInstallSystem`, `length_of_header` et `length_of_all_following_data`. Un lecteur statique peut localiser et valider cet enregistrement sans ouvrir le programme d’amorçage PE, mais il lui faut encore un modèle indépendant des blocs d’en-tête compressés et des programmes d’installation complets appartenant au projet.

NSIS est désormais **admis pour les métadonnées statiques uniquement**. Le codec ne parcourt que les enregistrements alignés sur 512 octets dans une zone supplémentaire PE bornée, accepte les deux valeurs magiques `firstheader` recensées, limite les indicateurs et les longueurs, et exige que la plage suivante déclarée tienne dans l’entrée physique. Il publie ensuite une ligne de métadonnées `NSIS/firstheader`. Il ne décompresse pas l’en-tête du programme d’installation et n’expose aucun octet des données incorporées. L’entrée de registre `.exe` n’est volontairement ni candidate au glisser-déposer ni associée au shell ; la détection par contenu fait autorité. L’analyseur est indépendant de Qt et des API d’exécution des programmes d’installation. Il n’exécute jamais le programme d’amorçage PE, le script, les pages personnalisées, les extensions ou les données extraites. Les autres formats SFX ne restent des alias que lorsque leur archive interne réussit la validation du contenu.

<a name="xdzip-admission-gates"></a>

## Conditions d’admission

1. Figer les révisions des sources et les empreintes des textes de licence avant d’utiliser un outil de référence.
2. Créer des échantillons complets et redistribuables pour chaque conteneur et méthode de compression admis ; un échantillon limité à l’en-tête est insuffisant.
3. Garder l’analyse de CFB/WIM/NSIS indépendante de Qt et des API d’installation, de montage, de registre ou d’exécution de scripts.
4. Ajouter des entrées à l’environnement d’exécution, à la CLI et au bureau seulement après réussite des tests portant sur l’identité physique exacte, les entrées en double, les limites des cycles, les limites cumulées d’entrée, de travail et de sortie, l’annulation, la modification de la source et les échecs terminaux.
5. Conserver les composants de référence GPL/LGPL/CPL dans build/toolchain, sauf si une décision distincte sur les dépendances et les licences modifie explicitement la politique du produit.
