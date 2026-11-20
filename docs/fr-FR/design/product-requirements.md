<a name="xdzip-xdzip-product-requirements"></a>

# Exigences produit de XDZip

[English](../../design/product-requirements.md) | [简体中文](../../zh-CN/design/product-requirements.md) | [繁體中文](../../zh-TW/design/product-requirements.md) | [日本語](../../ja-JP/design/product-requirements.md) | [한국어](../../ko-KR/design/product-requirements.md) | Français | [Deutsch](../../de-DE/design/product-requirements.md) | [Español](../../es-ES/design/product-requirements.md) | [Português (Brasil)](../../pt-BR/design/product-requirements.md) | [Русский](../../ru-RU/design/product-requirements.md)

XDZip permet de parcourir les archives comme des dossiers, de travailler sur une sélection et de créer ou modifier les archives lorsque le format le permet. Le bureau et la [CLI autonome](../../engineering/command-line.md) partagent le cœur de traitement des archives. Le [README du projet](../../../README.fr-FR.md) décrit les capacités et distributions actuelles. Les exigences suivantes définissent le comportement à préserver et vérifier.

<a name="xdzip-requirements-and-implementation"></a>

## Exigences et implémentation

| ID | Exigence | Comportement ou périmètre actuel |
| --- | --- | --- |
| P01 | Parcourir sans extraire | L’ouverture sonde la source et termine une analyse bornée avant de publier la première page. Le décodage du contenu exige une action distincte. |
| P02 | Naviguer, rechercher, trier et sélectionner | Les identités physiques stables, identités de navigation distinctes, requêtes paginées et sélections bornées sont implémentées. Les ZIP/ZIP64 ordinaires à fichier unique disposent d’un index de métadonnées plus grand, avec des limites de tâches distinctes. |
| P03 | Extraire la sélection prévue | Le cœur fige la sélection, valide les noms et autorise une destination. Il gère les décisions de conflit et rapporte les fichiers publiés, ignorés, les erreurs et l’annulation. |
| P04 | Créer des archives à partir de fichiers et de dossiers | Les options sont validées par le registre de création. La sortie est préparée et vérifiée avant publication. Le registre définit les formats et options disponibles. |
| P05 | Respecter les capacités des formats | L’interface combine le descripteur du format et l’état de la session. Chaque demande reste validée par le cœur. Lecture, aperçu, vérification, création, mots de passe et volumes ont des limites propres. |
| P06 | Limiter les tâches et permettre leur annulation | Les métadonnées et les tâches lourdes utilisent des files distinctes. Les opérations lourdes du bureau sont sérialisées. Une tâche acceptée peut continuer après la fermeture de son onglet. |
| P07 | Limiter les écritures aux autorisations et limites vérifiées | L’hôte contrôle les noms, liens, identités, préparation, sortie réelle et validation. Le format contrôle la structure et l’intégrité applicables. |
| P08 | Fournir des contrôles de bureau cohérents et accessibles | QML et C++ fournissent des contrôles communs, la navigation au clavier, le focus et l’accessibilité. Les IME, lecteurs d’écran et affichages natifs exigent leurs propres vérifications. |
| P09 | Limiter et mesurer les ressources | Le code applique des budgets indépendants. Les objectifs de performance et les mesures sont suivis séparément. |
| P10 | Garder la logique des formats indépendante de Qt | Les traits Rust, le routage explicite du registre et les interfaces de processus dédiées prennent en charge les implémentations intégrées. Le bureau et la CLI partagent le cœur. |
| P11 | Protéger les secrets et les données | Les opérations d’archives fonctionnent hors ligne. Les mots de passe sont conservés dans une mémoire à portée limitée et effaçable. Confidentialité, diagnostics et accès explicite aux mises à jour suivent leurs propres contrats. |
| P12 | Auditer la livraison | Utiliser les canaux de paquet définis, les entrées verrouillées, manifestes, sommes de contrôle, licences et vérifications de plateforme. Les résultats concernent le build et la distribution effectivement vérifiés. |
| P13 | Autoriser les extensions tierces sous réserve de confiance et de permissions vérifiées | L’essai XDP1 en lecture seule est indépendant. L’installation et l’activation des extensions dans le produit restent à concevoir. |
| P14 | Limiter l’aperçu et l’ouverture | Lorsque le panneau des propriétés est visible, sélectionner un fichier pris en charge lance son aperçu borné de texte ou d’image. L’ouverture explicite d’une entrée ordinaire admissible utilise une copie privée vérifiée transmise à l’association système. |
| P15 | Signaler précisément les échecs et les résultats incertains | Les erreurs stables et les résultats bornés couvrent les changements de source, les pannes de disque, les éléments ignorés et le nettoyage. La reprise automatique après redémarrage reste une option de conception. |

<a name="xdzip-workflows-and-scope"></a>

## Parcours et périmètre

- **Ouvrir et examiner.** La demande d’ouverture atteint le coordinateur de sessions. Une analyse réussie publie une session. Recherche et tri créent une nouvelle requête. Un double-clic ouvre un dossier ou un fichier admissible via une copie temporaire. Avec le panneau des propriétés visible, sélectionner un fichier pris en charge lance son aperçu. [Navigation et aperçu](../user-guide/browse.md), [Ouverture temporaire](../engineering/archive-opening.md)
- **Extraire.** « Tout extraire » concerne l’archive entière. Pour limiter l’extraction aux résultats, sélectionnez-les dans la recherche et vérifiez la portée. La vérification préalable des métadonnées fournit des constats avant l’autorisation d’écriture. Les validations fichier par fichier conservent les fichiers achevés après un échec ou une annulation ultérieurs. [Extraction](../../engineering/zip-safe-extraction.md), [Ordonnancement](task-scheduling.md)
- **Créer, modifier ou vérifier.** La création et les modifications admissibles vérifient la sortie temporaire avant publication. Vérifier contrôle une archive admissible selon son format sans obtenir d’autorisation de sortie. Les opérations dépendent du format, de l’entrée et du profil source. [Registre des formats](../engineering/format-registry.md), [Transferts](../../engineering/archive-transfers.md), [Vérification des archives](../engineering/archive-checking.md)

<a name="xdzip-planned-scope-and-verification"></a>

## Fonctions prévues et vérification

La gestion des extensions externes, la reprise durable des tâches et les profils de format supplémentaires nécessitent leur propre conception, implémentation et vérification. Un numéro de version ne suffit pas à établir la couverture des formats ou plateformes. La synchronisation cloud des archives, le montage d’images disque et l’analyse antivirus en tant que service sont hors du périmètre du produit. La [matrice de traçabilité](requirements-traceability.md) relie les exigences aux vérifications. La [politique de support](../engineering/support-policy.md) et les notes de version décrivent les builds distribués.
