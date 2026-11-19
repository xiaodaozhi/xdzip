<a name="xdzip-lz4-stream-contract"></a>

# Contrat des flux LZ4

[English](../../engineering/lz4-streams.md) | [简体中文](../../zh-CN/engineering/lz4-streams.md) | [繁體中文（台灣）](../../zh-TW/engineering/lz4-streams.md) | [日本語](../../ja-JP/engineering/lz4-streams.md) | [한국어](../../ko-KR/engineering/lz4-streams.md) | Français | [Deutsch](../../de-DE/engineering/lz4-streams.md) | [Español](../../es-ES/engineering/lz4-streams.md) | [Português (Brasil)](../../pt-BR/engineering/lz4-streams.md) | [Русский](../../ru-RU/engineering/lz4-streams.md)

Le profil des trames standard suit les spécifications de l’auteur pour le [format de trame 1.6.5](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Frame_format.md), le [format de bloc du 2026-10-26](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Block_format.md) et [XXH32](https://github.com/Cyan4973/xxHash/blob/v0.8.3/doc/xxhash_spec.md). Les algorithmes sont implémentés indépendamment en Rust. Les outils de référence et les résultats d’exécution générés restent dans `build/` en local. Seuls les motifs d’entrée reproductibles du projet sont versionnés. Les [flux à fichier unique](single-file-streams.md) décrivent les autorisations de l’hôte et la publication.

<a name="xdzip-admission-and-framing"></a>

## Admission et trames

Pour les trames standard de version 1, le lecteur vérifie les bits réservés, les valeurs BD de 4 à 7, le XXH32 d’en-tête, la taille de contenu facultative, chaque XXH32 déclaré pour un bloc ou le contenu et le marqueur de fin. La taille des données d’un bloc ne peut pas dépasser le maximum déclaré, compris entre 64 KiB et 4 MiB. Les blocs indépendants réinitialisent l’historique. Les blocs liés conservent seulement les 64 KiB précédents. Les contrôles et le décompte s’appliquent aussi aux blocs bruts de longueur nulle. Les profils à dictionnaire externe et les anciennes trames sont hors périmètre. Une déclaration de dictionnaire n’autorise jamais la lecture d’un autre fichier.

Les trames standard concaténées et les trames ignorables intercalées sont bornées par l’entrée physique, les budgets de la tâche, l’annulation et une limite de 100 000 trames ou blocs. Un fichier autonome valide contient au moins une trame de données standard. Une fin arbitraire ou une entrée composée uniquement de trames ignorables provoque un échec. La détection peut examiner au plus 8 MiB d’entrée compressée réelle avant de produire un préfixe de contenu de 512 octets. Une identification certaine ou un TAR reconnu comme endommagé empêche tout repli vers un autre format en cas d’échec. Le balayage complet valide toutes les trames avant de publier la taille constatée et l’entrée virtuelle.

<a name="xdzip-block-processing-and-creation"></a>

## Traitement des blocs et création

Les longueurs utilisent des opérations arithmétiques vérifiées. Une correspondance exige un décalage non nul situé dans l’historique. Les octets en chevauchement sont copiés dans l’ordre croissant et la production reste dans la limite déclarée. La dernière séquence contient uniquement des littéraux. Son demi-octet de correspondance inutilisé est ignoré. Un flux avec correspondances se termine par au moins cinq littéraux. La dernière correspondance commence au moins douze octets décodés avant la fin du bloc. La lecture produit des fragments bornés et reste arrêtée après un échec. L’historique n’est transféré qu’après validation de la fin exacte du bloc. Les E/S de l’hôte, l’annulation, les limites et les profils non pris en charge gardent des catégories d’erreur distinctes.

Un encodeur autorisé doit produire de véritables correspondances LZ4, n’utiliser des blocs bruts que si la compression ne réduit pas la taille et inclure les sommes de contrôle des blocs et du contenu. Chaque niveau proposé limite le travail de recherche des correspondances et doit produire du contenu multibloc décodable indépendamment. Les tampons de trames et de correspondances doivent être autorisés avant allocation. La création emploie des tailles d’entrée exactes et figées, une sortie temporaire gérée par l’hôte, une vérification complète et des contrôles finaux de la source et de la sortie. LZ4 ordinaire prend un fichier d’entrée ordinaire. Une composition TAR peut être déclarée une fois couverte par le lecteur et l’encodeur TAR partagés et les contrôles automatiques correspondants. L’enregistrement d’une extension ne rend aucun des deux profils modifiable, chiffrable ou divisible en volumes.

La vérification utilise `format_declared_checks`, car certaines trames sont dépourvues de somme de contrôle du contenu. Présentez une trame sans somme de contrôle sans déclarer son contenu vérifié. Les limites de ressources, la validation réelle des plateformes, la validité du format et l’autorisation de publication sont évaluées séparément.
