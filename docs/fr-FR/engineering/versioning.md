<a name="xdzip-product-version-and-build-identity"></a>

# Version du produit et identité du build

[English](../../engineering/versioning.md) | [简体中文](../../zh-CN/engineering/versioning.md) | [繁體中文（台灣）](../../zh-TW/engineering/versioning.md) | [日本語](../../ja-JP/engineering/versioning.md) | [한국어](../../ko-KR/engineering/versioning.md) | Français | [Deutsch](../../de-DE/engineering/versioning.md) | [Español](../../es-ES/engineering/versioning.md) | [Português (Brasil)](../../pt-BR/engineering/versioning.md) | [Русский](../../ru-RU/engineering/versioning.md)

La clé facultative `prerelease_number` sépare le numéro de préversion du numéro de build. Par exemple, alpha.1 peut correspondre au build 14, et le numéro de build continue d’augmenter pour alpha.2. Sans cette clé, l’interprétation historique `channel.build` s’applique. Le [protocole de publication](release-protocol.md) définit les manifestes et les noms d’artefacts.

[VERSION.json](../../../VERSION.json) est la seule source maintenue pour la version, le canal et le numéro de build. Le build lit les valeurs actuelles. Le [journal des changements](../../../CHANGELOG.md) décrit les versions et les [tickets](https://github.com/xiaodaozhi/xdzip/issues) suivent la livraison et les vérifications. Une augmentation du numéro de version ne vaut pas approbation d’une publication.

Les contrôles de compatibilité utilisent des sources historiques figées et le build actuel pour vérifier la mise à niveau, le retour à une version antérieure et les paramètres. Le [script du produit précédent](../../../scripts/prepare-previous-product.py) maintient la référence du produit complet. Le [script de compatibilité des paramètres](../../../scripts/check-settings-compat.py) maintient la référence de configuration. Les versions et les commits courants ne sont pas recopiés ici. Le schéma des paramètres et la version du produit évoluent séparément.

- `version` suit `major.minor.patch` sans zéros initiaux. Les changements incompatibles, les fonctions compatibles et les corrections emploient leurs composantes respectives. Les limites de compatibilité doivent aussi être indiquées pendant 0.x.
- `channel` accepte development, alpha, beta, rc et stable. Une préversion s’affiche sous la forme `version-channel-abbreviation.build` et development devient dev. Une version stable affiche la version de base.
- `build` est un entier de 1 à 65535. Incrémentez-le pour un nouveau build destiné à la livraison externe. Les compilations locales ne l’incrémentent pas automatiquement et sa modification ne crée pas automatiquement de tag ni de publication. Chaque composante numérique est aussi limitée à 65535 pour les ressources Windows.
- CMake lit le JSON directement. `python3 scripts/sync-version.py` synchronise les versions dérivées pour l’espace Cargo et HTML. La CI utilise `--check`. Tous les paquets Rust internes héritent de la version de l’espace de travail. Actualisez Cargo.lock après la synchronisation et exécutez `cargo check --workspace --locked`.
- La ressource numérique Windows à quatre composantes contient la version de base et le build. La version textuelle conserve le canal. Sur macOS, `CFBundleShortVersionString` contient la version de base, `CFBundleVersion` le build et la version longue conserve le canal. Respectez la [documentation Apple](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleversion). Utilisez le format numérique de la plateforme et n’insérez aucun commit Git dans ces champs.
- Sous Windows, `FileDescription` vaut `xdzip Archive Manager` notamment dans le Gestionnaire des tâches. `ProductName` reste `xdzip`. Le nom de l’exécutable et l’identité des associations restent indépendants du nom affiché. Les contrôles lisent les ressources de l’EXE réel.
- Chaque build de bureau produit le commit Git court et l’indicateur de modifications locales. Sans Git, il indique une valeur inconnue. Il n’inclut ni chemin de machine, ni nom d’utilisateur, ni horodatage instable. L’en-tête ne change que si son contenu change. Recompiler après le commit fournit l’identité du commit propre.

Les versions du module QML, du protocole de la passerelle et du schéma des paramètres sont indépendantes de celle du produit.

`xdzip --version`, Paramètres → À propos et l’application d’aperçu lisent les mêmes informations. Les arguments de version de la CLI sont traités avant la création de l’interface graphique. À propos affiche version, build, commit, Qt effectivement utilisé, système et architecture. Le canal de développement et les modifications non validées sont signalés. Le JSON copié ne contient ni fichiers personnels, ni chemins, ni noms de machine, ni identifiants. Le bouton du projet ouvre le navigateur à la demande de l’utilisateur.

La grande icône À propos utilise le PNG de 512 px de la plateforme. HTML emploie le vecteur de même origine, sans agrandir l’icône d’interface de 72 px.

À propos conserve les dimensions logiques des paramètres sans pied de page. Il prend en charge les [langues fournies](../ux/settings-localization.md), les thèmes clair et sombre et les tailles 100%, 125% et 150%. Voir la [maquette À propos](../../../design/hifi/sources/settings.html?tab=about). HTML génère les valeurs à partir du même JSON. La version native de Qt et le commit restent inconnus quand ils sont indisponibles.

Vérifiez les ressources de version de la CLI et du système avec `scripts/check-build-version.py`, `scripts/test-version.py`, `scripts/check-html-about.py` et `BrowserTest::versionAboutWindow`. Les captures et mesures restent sous `build/review/about/`. Les sources, métadonnées système et versions en cours d’exécution doivent correspondre. Un numéro de version ne prouve ni la signature, ni la validation manuelle, ni la compatibilité.
