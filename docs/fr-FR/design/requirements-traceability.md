<a name="xdzip-requirements-and-verification-traceability"></a>

# Traçabilité des exigences et de leur vérification

[English](../../design/requirements-traceability.md) | [简体中文](../../zh-CN/design/requirements-traceability.md) | [繁體中文（台灣）](../../zh-TW/design/requirements-traceability.md) | [日本語](../../ja-JP/design/requirements-traceability.md) | [한국어](../../ko-KR/design/requirements-traceability.md) | Français | [Deutsch](../../de-DE/design/requirements-traceability.md) | [Español](../../es-ES/design/requirements-traceability.md) | [Português (Brasil)](../../pt-BR/design/requirements-traceability.md) | [Русский](../../ru-RU/design/requirements-traceability.md)

Les [exigences produit](product-requirements.md) définissent les identifiants et leur périmètre actuel. Chaque ligne indique les vérifications à effectuer lorsque le comportement change. Les résultats, plateformes et limites sont consignés dans le dossier du changement. Les [cas d’acceptation](../../../developers/qa/acceptance.md) indiquent les points de départ des vérifications.

<a name="xdzip-requirement-matrix"></a>

## Matrice des exigences

| Exigence | Références de conception | Points à vérifier |
| --- | --- | --- |
| P01 Parcourir sans extraire | [Domaine](domain-data.md), [Interface](qt-quick-ui.md) | T01 Ouvrir une archive réelle et vérifier que la navigation ne produit aucun fichier extrait. |
| P02 Parcourir, rechercher et sélectionner | [Domaine](domain-data.md), [Performances](performance.md) | T02 Identité des éléments homonymes, requêtes stables, pages obsolètes, stockage des grands index et sélection entre pages. |
| P03 Extraire la sélection prévue | [Autorisation des fichiers](file-security.md), [Tâches](task-scheduling.md) | T03 Développement des dossiers, doublons, renommage, éléments ignorés, écrasement et résultats exacts sur disque. |
| P04 Créer une archive | [Formats](codec-extension-sdk.md), [Autorisation des fichiers](file-security.md) | T04 Allers-retours indépendants pour le profil modifié, vérification des fichiers temporaires et protection des anciens fichiers. |
| P05 Respecter les capacités des formats | [Passerelle](qt-rust-bridge.md), [Formats](codec-extension-sdk.md) | T05 Registre, actions propres à chaque session et rejet des profils non pris en charge à l’exécution. |
| P06 Limiter les tâches et permettre leur annulation | [Passerelle](qt-rust-bridge.md), [Tâches](task-scheduling.md) | T06 Files pleines, acheminement des demandes de décision, concurrence entre annulation et écriture définitive et arrêt asynchrone. |
| P07 Protéger les fichiers et les ressources | [Autorisation des fichiers](file-security.md), [Formats](codec-extension-sdk.md) | T07 Contrôles natifs applicables des liens et identités, validation des chemins, quotas et objets protégés. |
| P08 Fournir une interface accessible | [Interface](qt-quick-ui.md) | T08 Comportement des modèles et contrôles, clavier natif, IME, accessibilité, mise à l’échelle et focus sur les plateformes concernées. |
| P09 Limiter et mesurer les ressources | [Performances](performance.md) | T09 Build comparable, mesures de l’arbre de processus, démarrage, tri, débit de sortie ou taille de paquet selon le changement. |
| P10 Garder la logique des formats indépendante de Qt | [Architecture](system-architecture.md), [Formats](codec-extension-sdk.md) | T10 Cœur sans Qt, routage explicite des adaptateurs et régressions propres aux formats. |
| P11 Protéger les secrets et les données | [Autorisation des fichiers](file-security.md), [Confidentialité](../engineering/privacy.md) | T11 Portée des mots de passe, diagnostics expurgés et accès réseau intentionnel. |
| P12 Auditer la livraison | [Qualité](quality-engineering.md), [Publication](build-release.md) | T12 Entrées verrouillées et identité des artefacts, inventaire, licences, confiance des paquets et vérifications d’installation applicables. |
| P13 Contrôler les extensions externes | [Formats](codec-extension-sdk.md), [Essai du SDK](../engineering/sdk-trial.md) | T13 Interopérabilité et permissions natives de l’essai uniquement. L’installation d’extensions dans le produit reste une option de conception. |
| P14 Limiter l’aperçu et l’ouverture | [Interface](qt-quick-ui.md), [Autorisation des fichiers](file-security.md) | T14 Aperçu automatique, contenus pris en charge, limites du processus et sorties temporaires privées. |
| P15 Signaler le résultat réel | [Passerelle](qt-rust-bridge.md), [Autorisation des fichiers](file-security.md), [Tâches](task-scheduling.md) | T15 Changements de source ou de destination, échecs et annulations après des écritures déjà validées, nettoyage et résultats incertains. |

<a name="xdzip-cross-cutting-verification"></a>

## Vérifications transversales

| Suite | Changements concernés | Points à vérifier |
| --- | --- | --- |
| T16 Propriété et cycle de vie de la passerelle | Durée de vie de la passerelle, de la connexion ou du contrôleur | Identité de la demande courante, réveil du récepteur, tâches conservées et destruction sûre. |
| T17 Interopérabilité et cas d’échec des formats | Changement de format ou de codec | Échantillon valide indépendant lorsque nécessaire, troncature ou profil non pris en charge et sortie limitée. |
| T18 Installation, mise à jour et suppression | Changement de plateforme ou de canal de distribution | Paquet réel, dépendances dans un environnement vierge, identité d’installation, transfert de la mise à jour et suppression ou récupération prises en charge. |

<a name="xdzip-representative-workflows"></a>

## Parcours représentatifs

1. Ouvrir un ZIP et lancer une recherche, figer les résultats sélectionnés, puis extraire et comparer les octets produits. « Tout extraire » désigne toujours l’archive entière.
2. Extraire des éléments homonymes ou en conflit selon une politique définie et vérifier les noms et états obtenus.
3. Annuler avant et pendant la publication, vérifier que les fichiers achevés restent signalés et que les archives précédemment vérifiées sont conservées si la création échoue avant validation.
4. Modifier une source ou une destination dans un test contrôlé et vérifier le résultat réel. Après un arrêt anormal, examiner les éléments restants sans supposer une reprise automatique.
5. Provoquer une erreur du processus de format fourni ou de l’essai SDK distinct selon son contrat. Vérifier que les données incomplètes ne sont jamais publiées comme résultat réussi.

Les contrôles statiques de rédaction, de format et de liens valident la documentation. Les preuves issues de fixtures, du rendu hors écran, de l’automatisation native et de la validation manuelle ont chacune leur propre portée. [Ingénierie de qualité](quality-engineering.md), [Guide de vérification](../../../developers/qa/README.md)
