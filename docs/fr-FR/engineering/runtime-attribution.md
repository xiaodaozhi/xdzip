<a name="xdzip-actual-component-attribution-and-license-materials"></a>

# Attribution des composants réels et documents de licence

[English](../../engineering/runtime-attribution.md) | [简体中文](../../zh-CN/engineering/runtime-attribution.md) | [繁體中文（台灣）](../../zh-TW/engineering/runtime-attribution.md) | [日本語](../../ja-JP/engineering/runtime-attribution.md) | [한국어](../../ko-KR/engineering/runtime-attribution.md) | Français | [Deutsch](../../de-DE/engineering/runtime-attribution.md) | [Español](../../es-ES/engineering/runtime-attribution.md) | [Português (Brasil)](../../pt-BR/engineering/runtime-attribution.md) | [Русский](../../ru-RU/engineering/runtime-attribution.md)

Ce processus complète les SBOM au niveau des fichiers en associant les bibliothèques et plugins Qt réels aux déclarations SPDX du SDK. Le résultat constitue une preuve vérifiable, pas une sélection automatique de licence ni une approbation de conformité. L’exhaustivité nécessaire à une publication officielle reste incomplète.

<a name="xdzip-inputs-and-sources"></a>

## Entrées et sources

À partir de Qt 6.8, des [inventaires SPDX](https://doc.qt.io/qt-6.8/licenses-used-in-qt.html) correspondent aux compilations. Les outils ne lisent que les déclarations d’exécution telles que qtbase, qtdeclarative, qtsvg et qttranslations dans sbom/ du SDK Qt 6.8.2 fixé, en excluant les outils de compilation et de test qttools. Ils valident la version SPDX, le nom du module et la version Qt, et conservent le SHA-256, l’espace de noms et les références stables aux paquets du document d’origine. La sortie ne garde que les champs nécessaires, sans copier les commentaires de compilation amont contenant des chemins de machines de développement.

`scripts/qt-license-sources.json` fixe les commits officiels des sources Qt et de SPDX license-list-data, les URL des textes de licence et les SHA-256. Les téléchargements sont mis en cache sous build/ ; toute discordance de hachage du cache ou d’un téléchargement échoue. Les URL SPDX arbitraires ne sont pas suivies pour télécharger les documents. Outre les textes standard de LICENSES/, inclure les avis originaux des algorithmes de redimensionnement fluide Qt GUI et de SVG, ainsi que les textes LGPL-2.1 référencés par les déclarations de dépendances système. Les textes LicenseRef et droits d’auteur amont restent dans le JSON des déclarations.

Les avis d’exécution Qt incluent aussi la source modifiable fixée de la Public Suffix List utilisée par Qt Network, le texte MPL-2.0 et les instructions de récupération. Le collecteur vérifie le blob amont, le SHA-256 et les octets de licence. La livraison des sources vérifie séparément que cette entrée reproduit deux formes de données générées dans l’archive Qt ; voir les [sources correspondantes](source-delivery.md). Cela ne remplace pas les vérifications de source et de remplacement des autres modules Qt.

Les avis MIME Tika de Qt Core conservent les attributions pertinentes de l’Apache Software Foundation et de PRONOM des Archives nationales britanniques, les textes Apache-2.0 et OGL-UK-3.0 et le NOTICE amont complet fixé. Le NOTICE complet décrit aussi d’autres composants Tika amont ; cela ne signifie pas qu’ils sont livrés avec ce produit. Voir les [sources correspondantes](source-delivery.md) pour les données MIME originales et la vérification de leur conversion Qt.

<a name="xdzip-file-and-component-association"></a>

## Association des fichiers et composants

1. Recalculer les hachages des fichiers dans la racine de vérification et rejeter ceux modifiés après deployment-audit.json.
2. Le mode SDK macOS utilise les chemins relatifs Qt SDK réellement résolus et l’identité des octets, sans revendiquer un déploiement autonome.
3. Le déploiement Linux enregistre le chemin et SHA-256 du SDK d’origine ainsi que le chemin et SHA-256 de l’artefact après patchelf. Les contrôles ultérieurs vérifient les enregistrements dans les deux sens et rejettent les preuves de copie périmées ou modifiées.
4. Windows privilégie l’identité des octets des fichiers SDK. Les candidats au nom unique mais aux octets différents sont seulement marqués filename-only-unverified-rewrite et consignés comme lacunes, non comme sources vérifiées.
5. Utiliser SPDX CONTAINS pour trouver les propriétaires des fichiers, puis résoudre récursivement DEPENDS_ON, y compris les références entre dépôts Qt et les éléments tiers intégrés. Les références de paquet manquantes restent des lacunes. Ne pas inclure à tort tous les outils dans l’exécution via un CONTAINS portant sur tout un dépôt.
6. Les QML et traductions non natifs sans déclaration par fichier sont associés seulement au périmètre du dépôt source concerné ; leurs licences individuelles ne sont pas déduites. Les bibliothèques natives SDK sans entrée de fichier SPDX, fichiers natifs hors Qt, licences NOASSERTION et textes manquants sont enregistrés séparément.

Les octets réels du SDK peuvent différer du SHA-1 SPDX de compilation amont, par exemple après une installation ou une signature ultérieure. Les outils conservent les valeurs attendues, comparaisons et lacunes sans en déduire la cause. L’association des octets SDK, les enregistrements de réécriture du déploiement et les hachages originaux amont sont trois preuves distinctes.

Le packaging macOS dédié génère les associations déployées tant que l’application finale signée existe, en rapprochant les entrées SDK par octets exacts des ressources ou par sections arm64, UUID et versions d’import, puis ajoute le rapport au SBOM des fichiers réels. La comparaison exclut les signatures, la disposition des commandes de chargement et LINKEDIT. Voir [provenance et avis des composants natifs d’exécution](native-runtime-materials.md) pour le périmètre et les avis dédiés. Le mode SDK ordinaire ne prouve pas pour autant les conclusions sur le déploiement.

Pour les plugins déplacés des dossiers QML vers `Contents/PlugIns/quick/`, l’association de déploiement peut utiliser les liens symboliques QML audités dans le paquet afin de retrouver une entrée SDK unique. Revérifier les liens réels, leurs cibles auditées et les hachages de fichiers lors de la production et de l’utilisation des enregistrements, puis effectuer la même comparaison d’octets ou Mach-O. Une provenance manquante reste non vérifiée ; plusieurs sources différentes ou des liens modifiés sont rejetés. Une correspondance de nom seule ne renforce pas l’attribution et ne supprime ni les différences de hachage SDK amont ni les lacunes de revue des licences.

Rust conserve le graphe complet de compilation de l’hôte et produit des candidats d’exécution en suivant les dépendances ordinaires depuis `xdzip-qt-bridge` pour le bureau, `xdzip-codec-bridge` pour le processus de formats et l’`xdzip-update-helper` réellement utilisé sous Windows/Linux ; macOS n’a pas de racine distincte pour un assistant de mise à jour. Les outils de compilation, de développement et proc-macro ainsi que leurs dépendances sont exclus. Ce sont des entrées potentielles d’exécution, pas la preuve que le code machine de chaque paquet a survécu à l’édition de liens ; l’élimination par l’éditeur de liens, le C/C++ natif et les bibliothèques du compilateur demandent un examen supplémentaire.

Les alternatives simples de licences Cargo historiques séparées par des barres obliques deviennent SPDX `OR` dans le CycloneDX source, tandis que l’original amont est conservé dans `xdzip:cargo_license_raw`. Les expressions avec barres obliques à priorité ambiguë sont rejetées. Les inventaires et avis de compilation originaux ne sont pas réécrits ; les opérateurs standard `AND`, `OR` et `WITH` restent inchangés. Il s’agit d’une normalisation de format, pas du choix d’une branche de licence ni d’une approbation de publication.

<a name="xdzip-execution-and-delivery"></a>

## Exécution et livraison

Le dérivé QtNetwork corrigé connu ajoute `network_security_variant` aux enregistrements de fichiers ; les rapports conservent aussi les reçus de compilation complets et revérifiés. Les différences de condensé SDK amont restent visibles ; la revue de publication utilise les sources fixées, les correctifs complets, les reçus et les preuves de transformation du déploiement pour les expliquer. La préparation de publication exige de relier l’attribution finale de Network à ces octets ; une correspondance de nom ne suffit pas. Les SBOM source et artefact identifient séparément le dérivé sans transformer l’état global incomplet en approbation de licence.

```sh
python3 scripts/prepare-runtime-notices.py --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --output build/review/notices/Qt-runtime
python3 scripts/runtime_attribution.py --audit build/review/deployment-audit.json \
  --root build/review/linux --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --deployment build/review/linux-deployment.json --output build/review/runtime-attribution.json
python3 scripts/sbom.py --audit build/review/deployment-audit.json \
  --source build/review/source-build.cdx.json --attribution build/review/runtime-attribution.json \
  --output build/review/artifact-files.cdx.json
```

Linux transmet les enregistrements de réécriture du déploiement ; les autres plateformes omettent --deployment. Écrire les rapports hors de la racine de vérification pour éviter une récursion d’inventaire. Les workflows Qt à la demande et Release préparent les documents de licence avant le déploiement et le hachage final. Windows et Linux placent les avis dans les environnements d’exécution réels ; l’archive Linux tar.gz les livre aussi. Sous macOS, Qt à la demande lit le SDK et ne conserve que les journaux et JSON. Un workflow Release dédié et explicitement autorisé déploie, signe ad hoc, empaquette et vérifie séparément les documents réels selon le [protocole de publication](release-protocol.md), en conservant un périmètre déployé et des hachages distincts. Les enregistrements SDK ne remplacent pas l’attribution du paquet final de l’application.

runtime-attribution.json contient fichiers, composants, relations, déclarations amont, droits d’auteur, sources des textes et lacunes. artifact-files.cdx.json ajoute les mêmes composants et relations de propriété connues, en liant le rapport aux mêmes hachages d’audit réel et à la même identité de compilation. Les contrôles du schéma officiel CycloneDX et des références continuent ; compositions reste incomplet.

Les contrôles par défaut autorisent les lacunes de revue de développement explicitement consignées, mais ne les marquent jamais silencieusement comme réussies. Les erreurs d’identité, chemin, octets, version, enregistrement de copie ou hachage de texte de licence échouent immédiatement. --require-complete bloque la publication et échoue actuellement si l’attribution ou les documents de publication sont incomplets. Les tests couvrent fichiers réécrits, enregistrements de copie périmés, sortie du chemin autorisé, versions incorrectes, omissions entre documents, SBOM partiels et exclusion des outils Rust.

<a name="xdzip-unresolved-release-gates"></a>

## Conditions de publication non résolues

Un [collecteur de livraison des sources](source-delivery.md) conserve les sources réelles du projet, Rust vendor, quatre archives de modules Qt d’exécution, deux archives de modules de compilation et les instructions de reconstruction et remplacement dynamique. Il ne prouve pas l’équivalence de compilation SDK/amont élément par élément. Chaque publication doit encore examiner l’attribution exacte des liens statiques, les bibliothèques de l’OS et redistribuables, les licences de chaque ressource intégrée, le choix des branches de licences libres et le respect des obligations. La modification ou le remplacement de métadonnées dans la [vérification locale du déploiement Mac](macos-deployment.md) et la [compilation, le chargement et le remplacement réels de Qt depuis les sources](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement) sont des processus distincts aux preuves séparées ; aucun ne prouve l’équivalence de compilation octet par octet pour un SDK arbitraire. Les inventaires source et champs SPDX ne remplacent pas les obligations de licence. La provenance et les licences du candidat historique `v0.1.0` figurent dans [#4](https://github.com/xiaodaozhi/xdzip/issues/4) et [#53](https://github.com/xiaodaozhi/xdzip/issues/53) ; les candidats suivants exigent une revue distincte, sans identité de signature payante comme condition. L’acceptation du bureau existante reste dans [#5](https://github.com/xiaodaozhi/xdzip/issues/5).

La [vérification de correspondance du SDK Qt](qt-sdk-correspondence.md) enregistre aussi les commits d’archives fixés, les hachages par fichier des déclarations source SDK, la configuration installée et le correctif consommateur AGL connu. Les correspondances dans le périmètre déclaré ne prouvent pas l’équivalence complète de compilation du SDK ; les obstacles de publication existants subsistent.

La clôture automatisée ajoute les cartes de l’éditeur de liens et membres de bibliothèques statiques réels, les hachages des ressources d’exécution, les versions fiables des dépendances natives et l’analyse des différences binaires précisément réversible ; voir le [contrat complet](../../../developers/engineering/candidate-verification.md). Ne marquer une transformation comme expliquée que si les hachages des octets reconstruits correspondent exactement aux déclarations amont. Ne pas supprimer les différences SDK originales ni deviner l’historique non documenté de l’éditeur.
