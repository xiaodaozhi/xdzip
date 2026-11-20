<a name="xdzip-targzip-implementation-contract"></a>

# Contrat d’implémentation TAR/GZIP

[English](../../engineering/tar-gzip.md) | [简体中文](../../zh-CN/engineering/tar-gzip.md) | [繁體中文](../../zh-TW/engineering/tar-gzip.md) | [日本語](../../ja-JP/engineering/tar-gzip.md) | [한국어](../../ko-KR/engineering/tar-gzip.md) | Français | [Deutsch](../../de-DE/engineering/tar-gzip.md) | [Español](../../es-ES/engineering/tar-gzip.md) | [Português (Brasil)](../../pt-BR/engineering/tar-gzip.md) | [Русский](../../ru-RU/engineering/tar-gzip.md)

Ce contrat définit les limites de navigation, création et extraction sûre de TAR/TAR.GZ. Les résultats par plateforme et l’acceptation manuelle sont consignés dans les tickets ou artefacts CI. Il conserve l’autorisation d’entrée, la préparation temporaire, les conflits, l’annulation et la publication de l’archive entière de l’[extraction sûre](zip-safe-extraction.md) et de la [création ZIP](zip-creation.md).

<a name="xdzip-scope-and-capabilities"></a>

## Périmètre et capacités

La navigation, la création et l’extraction sûre de TAR et TAR.GZ sont prises en charge. TAR est un conteneur et GZIP un filtre de compression ; la lecture détecte le contenu au lieu de choisir un backend selon l’extension. Le comportement ZIP reste compatible. L’expansion imbriquée automatique, le chiffrement et le fractionnement TAR ne sont pas activés. Un GZIP ordinaire à fichier unique relève du [contrat de flux](single-file-streams.md) distinct.

Les en-têtes TAR, nombres, PAX, encodage en flux et réécriture utilisent une couche d’enregistrements Rust indépendante. Le cadrage GZIP, CRC-32 et Deflate LZ77/Huffman utilisent le module partagé `xdzip-compression`. `tar` et `flate2` servent seulement de références aux tests de développement et n’entrent pas dans les dépendances de production de ces crates. Voir le [contrat des formats indépendants](../../engineering/independent-rust-codecs.md) pour les spécifications, sources et limites.

TAR accepte les fichiers ordinaires et répertoires et analyse les chemins PAX et noms longs GNU. Liens symboliques, liens physiques, périphériques, FIFO, fichiers creux et types inconnus ne peuvent pas être extraits. Tous les noms de sortie passent toujours par PortablePath et la vérification préalable des conflits de l’hôte ; les bibliothèques de format ne reçoivent aucun chemin de fichier ni autorisation de déballage. Les permissions de répertoire, propriétaires et modes spéciaux ne sont pas restaurés. Les dates sont des métadonnées d’affichage sans garantie de restauration.

<a name="xdzip-bounded-streaming"></a>

## Traitement en flux borné

Les anciens encodages de noms, noms longs GNU et déclarations PAX UTF-8/BINARY sont résolus uniformément selon le [contrat d’encodage](filename-encoding.md). Les index ne sont publiés qu’après analyse complète ; nombres d’entrées et totaux sont inconnus pendant l’analyse. Les métadonnées conservent les limites de 100 000 entrées, 4 096 octets par nom et les budgets cumulés de noms. L’entrée encodée du flux et le travail de décodage sont chacun limités à 24 GiB ; un en-tête d’extension GNU/PAX est limité à 64 KiB et chaque entrée à quatre en-têtes d’extension. PAX global n’accepte que des commentaires sans sémantique de fichier, pour compatibilité avec git archive. Leur longueur d’enregistrement déclarée est entièrement vérifiée, puis ils sont ignorés sans héritage par les entrées. Les champs globaux path, size, mtime, linkpath, creux et inconnus restent explicitement refusés ; les liens longs GNU et variantes creuses ne sont pas pris en charge. Les commentaires globaux comptent aussi dans les quatre en-têtes par entrée et les budgets d’octets du flux ; ils ne rendent pas valides des extensions locales inachevées. Le préfixe courant `./` des noms TAR est normalisé ; les marqueurs racine ne sont pas des sorties. Les lectures et sauts séquentiels vérifient aussi l’annulation et les budgets d’octets ; le positionnement ne contourne pas les contrôles de corruption. GZIP doit atteindre EOF et vérifier CRC et ISIZE. Plusieurs membres ne forment qu’un seul flux continu d’octets TAR. Déchets non nuls en fin de flux, troncature et corruption échouent. Les index partiels non vérifiés sont refusés.

TAR.GZ ne fournit pas de tailles compressées par entrée. L’admission à fort taux compare le total déclaré sélectionné à la taille complète de la source figée. Au-delà de 1000:1 et de 1 MiB, une autorisation explicite reste requise ; elle ne contourne pas les quotas de travail d’analyse ou de sortie. La vérification préalable des métadonnées n’invente pas de taux de compression par entrée.

L’extraction traite les entrées sélectionnées dans l’ordre physique. Le saut de contenu non sélectionné est comptabilisé et annulable ; aucun accès aléatoire n’est promis. PayloadReader ne reçoit que la source et le récepteur de l’hôte. L’hôte vérifie toujours les versions de source aux limites d’analyse, de décodage et de validation. La validation par entrée et les échecs de création globale suivent le contrat d’E/S sûre existant.

<a name="xdzip-creation-and-interface"></a>

## Création et interface

La fenêtre de création active les segments TAR et TAR.GZ existants sans changer de densité. Le changement de format met à jour l’extension. TAR désactive le niveau de compression ; les deux formats TAR désactivent et effacent mots de passe et volumes fractionnés. TAR.GZ utilise les niveaux GZIP 0/1/6/9. La taille compressée par entrée est inconnue et affichée par un tiret, pas 0 octet. Rust dérive uniformément le dossier cible nommé d’après l’archive à partir du nom source et le fournit via le pont borné pour l’aperçu. TAR.GZ retire toute l’extension composée sans distinction de casse ; un nom de base vide revient à archive. L’interface ne dérive pas une autre cible ; PortablePath et la racine autorisée la revérifient. Les étiquettes de format de la fenêtre principale s’adaptent au format réel et à la largeur du texte. Les dates TAR sont affichées en UTC et restent inconnues hors de la plage d’affichage existante. La création écrit explicitement les chemins PAX UTF-8 pour Unicode ou les noms longs et la taille PAX pour les gros fichiers, évitant les suppositions d’encodage local des lecteurs de référence. Figez l’arbre d’entrée, écrivez en flux, terminez TAR/GZIP, réanalysez puis décodez et vérifiez chaque entrée ; ne publiez l’ensemble qu’après réussite. Le renommage en cas de conflit conserve toute l’extension composée.

<a name="xdzip-verification-and-limits"></a>

## Vérification et limites

Couvrez l’interopérabilité bidirectionnelle avec tar système/Python tarfile, les chemins longs, Unicode, répertoires vides, doublons, PAX/GNU, entrées spéciales, échappement de chemin, troncature, CRC, limites de ressources, annulation et protection des anciens fichiers. La création, réouverture et extraction Qt réelles réutilisent les fenêtres de tâches et de conflits existantes. Les preuves réelles ne sont générées que sous build/ ou dans les artefacts CI.

Choisissez les vérifications supplémentaires de bureau et d’accessibilité affectées selon la [vérification générale](../../../developers/qa/README.md) ; l’automatisation native et les contrôles hors écran ne remplacent pas les vérifications manuelles. Consignez les résultats et limites réels dans le ticket approprié. L’acceptation de bureau existante figure dans [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5) ; les blocages ponctuels de permission ou d’environnement ne sont pas conservés dans ce contrat. La revue historique de licence de `v0.1.0` figure dans [#4](https://github.com/xiaodaozhi/xdzip/issues/4) et [#53](https://github.com/xiaodaozhi/xdzip/issues/53) ; les futurs candidats nécessitent de nouveaux tickets de publication.

L’entrée de création CLI accepte .tgz comme alias de même format que .tar.gz, avec le même encodage GZIP, la même vérification et la même publication. Le GZIP ordinaire non TAR utilise le codec distinct à fichier unique.
