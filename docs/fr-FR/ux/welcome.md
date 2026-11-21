<a name="xdzip-main-window-welcome-page"></a>

# Page d’accueil de la fenêtre principale

[English](../../ux/welcome.md) | [简体中文](../../zh-CN/ux/welcome.md) | [繁體中文](../../zh-TW/ux/welcome.md) | [日本語](../../ja-JP/ux/welcome.md) | [한국어](../../ko-KR/ux/welcome.md) | Français | [Deutsch](../../de-DE/ux/welcome.md) | [Español](../../es-ES/ux/welcome.md) | [Português (Brasil)](../../pt-BR/ux/welcome.md) | [Русский](../../ru-RU/ux/welcome.md)

Le design haute fidélité est confirmé et intégré à l’entrée officielle de revue HTML. Qt implémente l’ouverture et la création réelles ; l’acceptation manuelle du bureau est enregistrée séparément. Voir le design des états de la page d’accueil et [welcome-reference.json](../../../specs/visual/welcome-reference.json). Les couleurs et la densité de fenêtre conservent les jetons et layout-scale existants sans modifier les normes globales. Quatre PNG sont des exports des originaux dessinés par code, pas des captures du produit en exécution.

<a name="xdzip-information-and-layout"></a>

## Information et disposition

Le titre de la page de démarrage est « 打包与解压，都简单一点。 » en chinois et « Pack and unpack, a little simpler » en anglais ; le catalogue français indique « Compresser et extraire en toute simplicité ». Le texte secondaire décrit la navigation préalable, l’extraction au besoin et la compression pour partager.

La zone cliente par défaut mesure 1024×622, minimum 832×544 ; zone de titre 48, capsule 32 et barre inférieure 32. Le groupe de contenu, large de 608, est centré horizontalement seulement et commence à 28 pixels logiques sous le haut disponible du corps, sans centrage vertical. Le titre mesure 30, ou 28 dans une fenêtre étroite ; description 13, titre d’action 16 et description de carte 12. Deux cartes d’action mesurent 296 de large sur 124 de haut, avec écart 16, marge intérieure 20 et rayon 8. Ce sont des cartes d’entrée de la page d’accueil, pas des remplacements des boutons partagés de fenêtres d’opération.

La barre de titre réutilise entièrement les composants existants de la fenêtre principale, avec marque, onglet de démarrage, ajout d’onglet et paramètres d’apparence ; la page d’accueil n’en change ni disposition, ni dimensions, polices ou icônes. Le corps n’affiche ni arbre vide, ni en-tête de tableau, champ de recherche ou barre d’extraction ; la disposition de navigation apparaît après ouverture d’une archive. L’action principale est « Ouvrir une archive », la secondaire « Créer une archive » ; la zone de dépôt est suivie des types pris en charge. Chaque catégorie indique son nombre d’extensions enregistrées. « Archives » est ouverte initialement. Activer une autre catégorie change l’aperçu ; réactiver la même le replie. Couleur d’accent et graisse indiquent la catégorie active. L’aperçu présente les extensions sur une seule ligne de balises légèrement remplies et bordées, en police d’interface normale. Une ellipse indique le débordement ; survoler la ligne affiche les suffixes de la catégorie choisie. « Associations de fichiers » ouvre les contrôles existants dans les paramètres. Le message de traitement local de la barre inférieure décrit le lieu du traitement et ne garantit pas la sécurité. Le nombre de tâches utilise l’historique réel ; zéro dans le design représente un historique vide au premier usage.

Ne pas afficher de récents exemples, de tailles fictives ni de capacités de format absentes du produit. Les feux tricolores dans la zone de titre indiquent uniquement la position des décorations macOS ; l’implémentation utilise les mécanismes de fenêtre de la plateforme. Windows et Linux gardent leurs décorations sans copier les feux.

<a name="xdzip-interaction-design"></a>

## Conception des interactions

- La page peut apparaître au premier lancement, après fermeture du dernier onglet d’archive ou après un échec d’ouverture sans archive navigable. Les tâches en arrière-plan continuent et ne sont pas réinitialisées.
- Ouvrir appelle le sélecteur de fichiers système ; l’annulation laisse la page d’accueil visible. Le succès passe au véritable onglet d’archive ; l’échec affiche une erreur récupérable sans inventer une liste réussie.
- Créer ouvre la fenêtre de création existante ; l’annulation conserve la page d’accueil. Les raccourcis candidats sont ⌘O/⌘N sur macOS et Ctrl+O/Ctrl+N sur Windows/Linux. Vérifier les raccourcis réels avant implémentation ; ce design ne prétend pas qu’ils sont intégrés.
- Faire glisser une sélection admissible met en évidence la zone de dépôt avec une bordure d’accent de 2 pixels logiques et un remplissage de sélection. En sortir rétablit l’état normal. Le bureau et la revue HTML ont des règles d’admission distinctes. Voir [Intégration Qt réelle](#xdzip-real-qt-integration) pour l’ouverture effective.
- Toute la carte est cliquable. Après la barre de titre, l’ordre clavier est ouvrir, créer, tâches. Le focus clavier n’ajoute pas de contour et suit la règle de perte de focus de la fenêtre. Survol et pression utilisent accentHover et accentPressed partagés, pas seulement l’opacité.
- Pendant l’ouverture, la carte remplace son titre par un indicateur occupé et « Ouverture… » et empêche temporairement ouvertures répétées et dépôts ; l’état désactivé utilise accentDisabled et accentDisabledText. La possibilité d’exécuter la fenêtre de création simultanément suit les règles de l’hôte ; la page d’accueil ne change pas la sémantique des tâches. Les états occupé, désactivé et erreur sont des contrats d’implémentation absents des quatre originaux statiques.
- Les menus de paramètres et de tâches gardent largeur 240, police 13 et hauteur de ligne minimale 32, soit un rapport 1:1 avec la police de corps 13 ; la page n’introduit pas de style de menu distinct. La police de carte 16 est un titre d’entrée, pas une référence de comparaison de densité des menus.

<a name="xdzip-verification-and-boundaries"></a>

## Vérification et limites

Couvrir thèmes clair/sombre, fenêtre minimale, états de glisser-déposer et proportions menu/barre de titre, en vérifiant DPR 1, 1,25 et 2 ainsi que le zoom réel du navigateur. Une capture réduite ne remplace pas la mise en page et une échelle CSS ne remplace pas le zoom navigateur. Résultats, images et rapports géométriques restent dans les artefacts locaux/CI ; l’état est consigné dans l’issue correspondante.

Qt utilise les tests de régression de la page d’accueil et du navigateur et ouvre les fichiers par le point d’entrée réel. Les tests automatisés ne remplacent pas la validation native du glisser-déposer, du lecteur d’écran ou de l’IME.

<a name="xdzip-real-qt-integration"></a>

## Intégration Qt réelle

WelcomePage et WelcomeAction n’expriment que disposition et intention. La page apparaît au démarrage et après fermeture d’une archive ; une ouverture réussie entre dans la session appropriée. Une archive vide reste un état de navigation valide. Le chargement initial désactive les ouvertures répétées et la création, affiche « Ouverture… » et permet l’annulation du chargement. En cas d’échec, la page et l’erreur réelle restent visibles pour choisir à nouveau. Fermer un brouillon de création ne crée aucune tâche. La page ne réinitialise pas les listes de tâches en arrière-plan.

Ouvrir et les raccourcis système réutilisent chooseArchive ; la création réutilise CreateWindow. Le sélecteur natif reste une fenêtre système et son annulation ne crée aucune session. Qt route selon le mode onglet/fenêtre enregistré, réutilise de préférence une page de démarrage vide et active les sessions existantes pour des archives déjà ouvertes.

Avec le gestionnaire de sessions de bureau, la page peut ouvrir jusqu’à 128 fichiers locaux en un dépôt. Chaque élément doit correspondre à un suffixe généré pour le glisser-déposer ou à un motif de nom de volume fractionné pris en charge. L’entrée de développement à session unique accepte un élément. Les dossiers et URL non locales sont refusés. Ces contrôles choisissent une voie d’ouverture ; le runtime Rust identifie toujours le format réel et valide la source. Utiliser **Ouvrir une archive** pour les formats accessibles uniquement par ouverture explicite. Voir le [registre des formats](../engineering/format-registry.md).

Les dimensions proviennent de welcome-reference.json et entrent dans Theme via generate-qml-theme.py. Un texte plus grand agrandit les cartes et le corps ; celui-ci défile et amène automatiquement les éléments au focus clavier dans la zone visible. Les zones de statut et de titre grandissent indépendamment sans Item.scale. Qt utilise les icônes PNG partagées, avec le graphisme existant à encre sombre/claire pour l’action principale ; les ressources d’exécution ne sont pas découpées dans les originaux.

Les entrées d’automatisation sont tst_welcome.qml, welcomeOpensRealArchivesAndReturns dans browser_test.cpp et scripts/check-qt-welcome.py. Les workflows Qt à la demande sur trois plateformes vérifient ouverture réelle, fermeture, récupération après échec et 36 éléments géométriques à DPR 1, 1,25 et 2, ainsi que largeur de menu 240 et police 13. Dépôt natif, sélecteur de fichiers, lecteur d’écran et IME sont vérifiés selon les besoins ; les résultats hors écran ne sont pas présentés comme des opérations de bureau non effectuées.

La page d’accueil ne modifie que le contenu et réutilise le même composant Qt ArchiveTabs. Les images `screens/welcome-titlebar-*.png` conservent la conception de la barre de titre. La régression compare la géométrie de la barre, des onglets et du bouton d’apparence avant et après ouverture d’une archive.

État de chargement : la commande d’annulation de l’ouverture initiale a été déplacée dans une superposition flottante du corps. Une progression inconnue utilise une barre active sans pourcentage simulé ; voir le [contrat de superposition de chargement](welcome-opening.md). Les images d’accueil originales sont conservées ; le design dynamique fournit `opening=1`.

Les erreurs et avertissements apparaissent sous les indications de format, avec la couleur de danger et l’icône d’avertissement d’état partagées. Afficher, renvoyer à la ligne, effacer ou remplacer une erreur ne change pas les coordonnées supérieures du titre et des cartes ; le contenu long défile dans le corps. Rouvrir la fenêtre principale après fermeture dans la zone de notification n’affiche qu’une page de démarrage et n’annule pas les tâches en arrière-plan. La source modifiable ajoute un état `error=1` ; les anciens PNG conservent leurs octets historiques et leur ancienne disposition centrée ne doit pas remplacer le contrat de positionnement supérieur.
