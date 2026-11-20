<a name="xdzip-linux-standalone-runtime-directory-and-verification"></a>

# Répertoire autonome d’exécution Linux et vérification

[English](../../engineering/linux-deployment.md) | [简体中文](../../zh-CN/engineering/linux-deployment.md) | [繁體中文](../../zh-TW/engineering/linux-deployment.md) | [日本語](../../ja-JP/engineering/linux-deployment.md) | [한국어](../../ko-KR/engineering/linux-deployment.md) | Français | [Deutsch](../../de-DE/engineering/linux-deployment.md) | [Español](../../es-ES/engineering/linux-deployment.md) | [Português (Brasil)](../../pt-BR/engineering/linux-deployment.md) | [Русский](../../ru-RU/engineering/linux-deployment.md)

La base actuelle de déploiement pour le développement est Ubuntu 24.04, x86_64 et X11. L’objectif est d’exécuter le produit actuel sans le SDK Qt de développement ; cela ne promet pas d’autres distributions, une glibc plus ancienne, Wayland ou un installateur complet. La CI macOS conserve compilation, tests et vérification de démarrage natifs. Les DMG locaux de développement suivent le [guide de déploiement Mac](macos-deployment.md) et ne sont pas affectés par ce processus.

<a name="xdzip-layout-and-generation"></a>

## Structure et génération

`scripts/deploy_linux.py` accepte uniquement des fichiers ELF de confiance appartenant au projet et un SDK Qt 6.8.2 explicitement indiqué, puis crée un nouveau répertoire build/ ou dist/ :

- `xdzip` est le lanceur ; il utilise bibliothèques, plugins et QML locaux sans hériter des chemins de recherche du SDK de développement.
- `bin/xdzip` est le produit ; le `qt.conf` adjacent utilise des chemins Qt relatifs.
- `lib/` contient les bibliothèques partagées Qt/SDK réellement importées ; `qml/` conserve le style Basic Controls et les sous-modules d’implémentation nécessaires. Les autres styles Controls, NativeStyle, Effects, variantes de dialogues propres aux styles, fichiers de concepteur et métadonnées de types QML sont supprimés avant la collecte des dépendances natives, afin de ne pas déployer leurs bibliothèques exclusives.
- `plugins/` contient xcb, la vérification hors écran, les images, icônes, entrées de plateforme, l’intégration graphique xcb et le `tls/libqopensslbackend.so` nécessaire ; le déploiement échoue sans backend OpenSSL. Les plugins de plateforme EGLFS, framebuffer et Wayland ne sont pas présentés comme livraison vérifiée.
- `translations/` conserve les traductions des contrôles standard Qt ; `share/notices/` livre les avis actuels et les documents des dépendances source.
- `share/deployment.json` consigne QML, dépendances d’exécution système, bibliothèques SDK copiées et limites ; README.md fournit l’utilisation.

L’analyseur utilise [qmlimportscanner](https://doc.qt.io/qt-6.8/qtqml-tooling-qmlimportscanner.html) de la version Qt fixée. Xdzip.Settings et Xdzip.Native sont des modules intégrés enregistrés ; les bibliothèques statiques de développement ne sont pas copiées. Dépendances ordinaires absentes, chemins QML sortant du SDK, conflits de bibliothèques homonymes et bibliothèques externes inconnues provoquent un échec. La collecte récursive des imports natifs ne copie que les bibliothèques SDK, pas la glibc ni les pilotes graphiques de l’hôte ; les dépendances système sont listées selon leur véritable paquet Debian propriétaire.

L’élagage valide l’arbre QML copié, les plugins Basic nécessaires, les imports forts conservés et les structures de styles connues avant suppression. Liens inattendus, styles ou sélecteurs de dialogue inconnus et dépendances fortes envers des modules omis provoquent un échec. Dialogues par défaut, méthodes de saisie, accessibilité, plugins d’image, TLS et traductions système conservent leurs règles existantes. `share/deployment.json` enregistre les fichiers QML retirés et leur nombre d’octets. Le `cli/` privé partage `bin/xdzip-native-codec` via son chemin de contexte fixe ; les paquets CLI autonomes restent indépendants.

HTTPS exige en outre `libssl3t64` et `ca-certificates` d’Ubuntu. Vérifier l’appartenance aux paquets système de `libssl.so.3` et `libcrypto.so.3` et un magasin de confiance CA système non vide. Le backend OpenSSL de Qt peut charger ces bibliothèques à l’exécution ; ldd seul ne prouve donc pas l’exhaustivité des dépendances. Le manifeste de déploiement enregistre les versions des paquets binaires installés, les paquets source et leurs versions. Les installateurs déclarent les deux dépendances ; la SBOM des fichiers réels les enregistre comme paquets OS requis sans intégrer les bibliothèques système dans l’application.

patchelf définit un RPATH `$ORIGIN` relatif sur chaque ELF, en calculant la distance relative de chaque plugin à lib/. La redirection des chemins Qt suit les [règles officielles de qt.conf](https://doc.qt.io/qt-6.8/qt-conf.html), permettant de déplacer tout le répertoire. Les bibliothèques restent liées dynamiquement et remplaçables ; le remplacement par un Qt compatible n’est pas restreint. Voir [déploiement Qt Linux](https://doc.qt.io/qt-6.8/linux-deployment.html) pour les limites générales.

Après réécriture du RPATH relatif, le déploiement retire les tables de symboles natives inutiles et vérifie que chaque entrée DT_NEEDED et RPATH/RUNPATH reste inchangée. Il enregistre la version de l’outil, les hachages et nombres d’octets avant/après pour chaque ELF dans `symbol_stripping` ; les entrées SDK et binaires de compilation originaux restent intacts. Pour une copie de diagnostic, le script de déploiement sous-jacent accepte `--keep-symbols`. L’attribution SDK finale et les hachages de l’application décrivent les fichiers transformés. Cela réduit la taille du paquet, sans démontrer un gain mesuré de mémoire ou de débit.

Utiliser par étapes le [point d’entrée de compilation unifié](developer-build.md) pour la génération et la vérification courantes. Terminer d’abord compilation, tests et préparation du packaging avec la même configuration, puis exécuter :

```sh
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

Le répertoire de compilation par défaut est build/desktop, la sortie de livraison build/package et le SDK Qt `build/toolchain/Qt/6.8.2/gcc_64` ; fournir les options communes à chaque étape pour d’autres emplacements. `package --kind runtime` génère le répertoire d’exécution et les documents d’audit ; puis `package --kind installer` avec le même output-dir génère tar.gz et .deb. Le défaut `all` effectue les deux dans cet ordre. Les scripts sous-jacents [deploy_linux.py](../../../scripts/deploy_linux.py), [package_linux.py](../../../scripts/package_linux.py) et [verify_linux_runtime.py](../../../scripts/verify_linux_runtime.py) conservent la logique de plateforme ; consulter leur `--help` pour diagnostiquer.

Le déploiement ne s’exécute que sur Linux x86_64 natif et n’écrase pas les répertoires existants. Le précontrôle du packaging vérifie qmlimportscanner, ldd, patchelf, dpkg-query et dpkg-deb requis par l’installateur ; ces outils de compilation n’entrent pas dans le produit. La vérification d’exécution en conteneur exige aussi Docker et un démon accessible ; les constructions sont limitées à l’image Ubuntu fixée et aux paquets OS enregistrés. L’absence de Docker ne bloque pas compilation et tests ordinaires, mais verify échoue explicitement ; une vérification d’isolation non exécutée n’est pas déclarée réussie. verify ordinaire n’installe pas le .deb sur l’hôte et ne modifie pas les associations de fichiers ; le cycle d’installation reste un contrôle CI jetable.

<a name="xdzip-automated-evidence-without-the-sdk"></a>

## Preuves automatisées sans SDK

Le conteneur utilise Ubuntu 24.04 fixé par condensé et installe les dépendances d’exécution OS réelles et la prise en charge de test Xvfb, polices et paramètres régionaux, mais pas Qt, Rust ni compilateur, dont l’absence est revérifiée. Il ne monte ni SDK de développement ni arbre source complet, seulement le répertoire produit en lecture seule, un ZIP distinct de 100 000 entrées appartenant au projet et les scripts de vérification. La vérification d’exécution d’archive désactive le réseau et l’élévation de privilèges ; seul le répertoire temporaire de configuration est inscriptible.

Monter l’application sous un nouveau chemin contenant des espaces. Après suppression des variables de recherche de développement, utiliser le lanceur pour vérifier :

1. L’exécution réelle de `--version`.
2. Le démarrage et la fermeture sûre d’une vraie fenêtre QML sous X11.
3. La sonde native du produit effectue première image, lecture de 100 000 entrées, annulation d’une requête active et sortie ; nombres et résultats d’annulation respectent le contrat.
4. Après suppression des variables de développement de l’hôte, vérifier tous les ELF empaquetés en mode déployé sans repli sur le SDK. La SBOM des fichiers réels conserve la même identité de version et commit et valide le schéma officiel.
5. Activer le réseau dans un nouveau processus de la même image sans SDK, exécuter `--update-probe` du produit avec `XDZIP_SETTINGS_PATH` isolé et effectuer une vraie requête HTTPS GitHub. Il doit signaler un backend OpenSSL disponible et terminer le contrôle de mise à jour ; conserver séparément `.https.json` et `.https.log`. Les erreurs réseau, certificat ou TLS bloquent la vérification.

La sonde HTTPS enregistre séparément le backend OpenSSL, le fichier CA système et les résultats réels de la requête GitHub. Une requête réelle réussie ne prouve pas à elle seule la vérification de signature ni l’installation native de mise à jour. L’analyse de signature et les workflows de mise à jour sans SDK sont vérifiés par le [workflow distinct de régression des mises à jour](../../../.github/workflows/update-regression.yml) ; ces preuves ne se remplacent pas.

Conserver les journaux bruts et rapports structurés de chaque tentative sous `.https.attempt-N.log` et `.https.attempt-N.json`, ainsi que `.https.log` pour la dernière tentative. Rapports absents ou invalides, délais dépassés et sorties du processus restent des échecs. Réessayer uniquement lorsque le contrôleur du produit confirme la limitation de débit de la liste de publications GitHub anonymes et que statut, phase de requête, en-têtes de réponse et diagnostics sont complets. Attendre l’échéance du serveur ou utiliser une temporisation progressive bornée, avec au plus trois tentatives et 3 700 secondes d’attente totale, comme la sonde d’acceptation macOS. Une échéance dépassant le budget restant, une attente interrompue ou une erreur TLS, réseau ou autre échoue immédiatement. La sonde doit terminer une vraie requête de mise à jour pour réussir. Enregistrer chaque décision et durée réelle d’attente dans `.https-waits.json`.

Avant de réutiliser un chemin de sortie, déplacer l’ancien rapport et les journaux dans un répertoire `.history` portant le nom du rapport dans le même parent, avec un sous-répertoire distinct par exécution. Si une nouvelle exécution échoue à mi-parcours, l’ancien rapport réussi ne reste pas au chemin de sortie courant, évitant toute confusion ; les artefacts historiques restent consultables séparément.

linux-runtime.json enregistre les événements réels et le SHA-256 de l’exécutable. Le rendu logiciel Xvfb/Qt Quick ne prouve que le fonctionnement du processus d’exécution, pas le GPU physique, les fenêtres de candidats chinoises, les lecteurs d’écran réels ni une installation complète sur le bureau.

<a name="xdzip-download-delivery-and-remaining-gates"></a>

## Livraison téléchargeable et conditions restantes

La CI génère xdzip-linux-x86_64.tar.gz et enregistre SHA-256 et hachages exécutables. tar.gz conserve les permissions d’exécution, évitant leur perte à l’extraction ZIP d’Actions. Chemins d’archive, UID/GID et horodatages sont normalisés sans répertoires privés. Après téléchargement, extraire et exécuter `./xdzip` dans le répertoire ; ne pas déplacer uniquement bin/xdzip. Le répertoire linux/ brut n’est pas téléversé à nouveau.

Le répertoire d’exécution `build/package`, tar.gz et .deb générés ici sont des livrables de développement expérimentaux qui n’installent rien et ne modifient pas automatiquement les associations par défaut. Les tar.gz et .deb Ubuntu officiels de Release sont générés par le workflow Release dédié et fournis selon le [protocole de publication](release-protocol.md). Le périmètre des plateformes suit la [politique de prise en charge](support-policy.md) ; les autres distributions et bureaux, Wayland et extensions similaires exigent des issues distinctes. L’acceptation existante du bureau reste dans [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5). Pour les modifications ultérieures, revérifier les contrôles affectés de vrais IME et lecteurs d’écran ainsi que d’installation, mise à niveau et désinstallation propres ; reconstruire un répertoire seul n’exige pas de répéter l’acceptation existante. La revue historique des composants `v0.1.0` est dans [#4](https://github.com/xiaodaozhi/xdzip/issues/4) et [#53](https://github.com/xiaodaozhi/xdzip/issues/53) ; l’exécution réussie du répertoire ne remplit pas automatiquement les futures conditions de publication.

Voir [installation et associations](installers-associations.md) pour installation, mise à niveau, retour arrière et désinstallation Ubuntu .deb, verrous d’exécution et automatisation MIME du bureau. La réussite du conteneur d’exécution ne remplace ni les vérifications réelles du bureau ni la revue des licences de publication.
