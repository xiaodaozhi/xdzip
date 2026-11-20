<a name="xdzip-split-volume-reading-and-temporary-opening"></a>

# Lecture des volumes et ouverture temporaire

[English](../../engineering/archive-opening.md) | [简体中文](../../zh-CN/engineering/archive-opening.md) | [繁體中文](../../zh-TW/engineering/archive-opening.md) | [日本語](../../ja-JP/engineering/archive-opening.md) | [한국어](../../ko-KR/engineering/archive-opening.md) | Français | [Deutsch](../../de-DE/engineering/archive-opening.md) | [Español](../../es-ES/engineering/archive-opening.md) | [Português (Brasil)](../../pt-BR/engineering/archive-opening.md) | [Русский](../../ru-RU/engineering/archive-opening.md)

La sélection dans le bureau, les demandes du gestionnaire de fichiers et la CLI suivent les mêmes règles Rust d’ouverture des sources. L’hôte résout les ensembles de volumes autorisés, relève leur identité et examine les octets autorisés. Le moteur sélectionné fournit les [métadonnées de navigation](readonly-browser.md), parfois par le [processus de format isolé](native-codecs.md). L’ouverture explicite d’une entrée crée une copie temporaire vérifiée pour l’application associée. Une [vérification d’intégrité](archive-checking.md) distincte permet de contrôler l’archive entière.

<a name="xdzip-split-zip"></a>

## ZIP fractionné

La couche de lecture prend en charge les flux continus numérotés comme `name.zip.001`, `.002` et les volumes PKZIP comme `name.z01`, `.z02` et le dernier `name.zip`. L’ouverture peut partir de n’importe quel volume. Les volumes doivent se trouver dans le même dossier, avec des noms consécutifs, et leur nombre est limité à 1 024 ; les volumes manquants, liens et fichiers spéciaux sont refusés. Les extensions servent uniquement à découvrir les volumes voisins. Le conteneur réel est identifié par ses octets sources puis intégralement analysé par son moteur.

Si le numéro de disque classique ZIP64 vaut 0xFFFF, le nombre total borné de volumes dans le locator adjacent sert d’abord à découvrir l’ensemble réel. Cette valeur de remplacement ne signifie pas que 65 535 volumes manquent. Cette découverte ne remplace pas la validation complète des enregistrements de fin et du répertoire.

L’hôte Rust conserve l’identité, la longueur et la date de modification de chaque volume. L’entrée logique lit les volumes à la demande sans réunir toute l’archive. Les offsets PKZIP relatifs aux disques sont convertis en une vue bornée du répertoire central sur un seul volume. Le codec existant vérifie toujours le répertoire, les en-têtes locaux, le CRC, l’authentification par mot de passe et les quotas d’extraction. Tous les volumes sources sont revérifiés à chaque utilisation et avant validation ; toute modification arrête la tâche. Leurs identités sont protégées contre l’écrasement par la sortie.

La normalisation PKZIP ne traite que les enregistrements centraux admis et ne prend pas en charge l’enregistrement final de signature numérique du répertoire central. Ce refus ne signifie pas que la vérification d’une signature a échoué. Une archive ordinaire à un seul volume peut lire cet enregistrement dans ses limites, mais ne vérifie pas sa signature cryptographique et refuse toute modification pour éviter de perdre ou d’endommager des métadonnées inconnues lors du déplacement.

L’acceptation couvre les volumes numérotés, PKZIP, ZIP64, les en-têtes et contenus à cheval sur plusieurs volumes, les volumes absents ou non consécutifs, le remplacement de la source, les numéros de disque invalides, les noms chinois et la non-régression des archives ordinaires à un volume.

<a name="xdzip-7z-and-rar-volumes"></a>

## Volumes 7z et RAR

Les volumes 7z numérotés forment un flux continu comme name.7z.001 et .002. RAR utilise name.partN.rar ou les anciens noms name.rar et name.r00 ; les limites physiques de chaque volume sont conservées lors de leur transmission au lecteur RAR Rust isolé. Les deux formats retrouvent l’ensemble dans le même dossier à partir de n’importe quelle partie existante. Les limites sont de 1 024 volumes et 100 000 entrées examinées dans le dossier. Les volumes manquants, numéros de noms conflictuels, liens et points de réanalyse sont explicitement refusés.

Rust conserve l’identité, la longueur et les dates du handle de chaque volume et communique les limites autorisées à l’adaptateur natif par ReadSource::volume_lengths(). Le processus natif ne peut demander à l’hôte que la lecture de plages autorisées ; un changement de volume n’accepte aucun chemin hôte arbitraire fourni par l’archive. Tous les volumes participent à la revérification de la source et à la protection contre l’écrasement. Une modification du premier volume ou de tout volume suivant arrête la tâche.

Le chiffrement des noms 7z et RAR peut exiger un mot de passe dès l’ouverture. Si Open renvoie password_required, aucun index n’est publié. Qt réutilise la fenêtre de mot de passe et renvoie le fichier initialement choisi ainsi que le secret borné avec OpenWithPassword ; une valeur de vérification incorrecte renvoie wrong_password. Après réussite, seules la session et ses tâches admises conservent le mot de passe. La fermeture de la session libère sa copie ; il n’est écrit ni dans les paramètres ni dans les données de récupération.

<a name="xdzip-cab-volumes"></a>

## Volumes CAB

Les noms des volumes CAB proviennent des en-têtes et peuvent être n’importe quels noms de fichiers valides ; les règles de concaténation d’octets des volumes numérotés ne s’appliquent pas. La référence suivante avance d’un volume à la fois. Une référence logique au volume précédent peut désigner un volume antérieur où un fichier continu a commencé, plutôt que le volume immédiatement précédent ; des exemples existants utilisent aussi des références adjacentes. Depuis le volume choisi, Rust suit les références précédentes en arrière puis découvre l’ensemble complet dans le même dossier en avançant. Après vérification de l’identité de l’ensemble, de l’ordre consécutif et des identités, il fige la longueur et le nom d’origine de chaque volume. Le décodeur vérifie tous les fragments ; un volume précédent non adjacent doit être le véritable volume initial d’un fichier continu. Les limites sont de 1 024 volumes et 100 000 entrées de découverte dans le même dossier. Sont refusés les volumes manquants ou incorrects, cycles, identités répétées, chemins absolus, séparateurs de dossier, échappements vers un parent, liens et points de réanalyse. Un sous-ensemble ouvert n’est pas publié comme archive complète.

Les rappels de changement de volume du processus ne peuvent choisir que des noms de cet ensemble autorisé ; chaque volume fournit son propre flux borné. Tous participent à la revérification de la source avant les tâches et la validation des fichiers, ainsi qu’à la protection contre l’écrasement. Les avertissements du gestionnaire natif et les résultats indiquant un volume manquant ne peuvent pas devenir un index réussi. Les archives CAB à un ou plusieurs volumes sont en lecture seule.

<a name="xdzip-double-clicking-regular-files"></a>

## Double-clic sur un fichier ordinaire

Un double-clic sur un dossier continue de l’ouvrir. Sur un fichier ordinaire, il lance en arrière-plan une extraction de cette seule entrée physique vers un dossier au nom aléatoire dans le cache privé de l’utilisateur. Les liens et objets spéciaux ne peuvent pas être ouverts par cette voie. Les vérifications du mot de passe, du CRC et de l’authentification, l’annulation et l’admission sérielle des tâches lourdes au niveau de l’application restent applicables ; l’utilisateur n’a pas à choisir de destination.

Après réussite, l’association système est invoquée avec une URL de fichier local, sans interpréteur de commandes. La fenêtre de progression correspondante ne se masque automatiquement que si le système accepte la demande d’ouverture. En cas d’échec d’extraction, d’annulation ou de refus système, elle reste visible avec une erreur lisible. L’acceptation ne prouve pas que l’application externe a terminé sa lecture ; le fichier temporaire ne peut pas être supprimé dès l’envoi de la demande.

Les dossiers temporaires portent des noms aléatoires imprévisibles de 128 bits et séparent contenu et marqueur d’identité. Le dossier de sortie n’autorise que l’utilisateur courant et les principaux système de confiance. Sous Windows, une DACL protégée est définie via le handle détenu uniquement sur un dossier vide créé exclusivement pour cette opération, puis revérifiée avant l’écriture en clair ; les permissions partagées d’un dossier choisi par l’utilisateur ne sont pas modifiées. POSIX utilise 0700 et vérifie les ACL macOS héritées. Si les permissions ne peuvent pas être garanties, la sortie est refusée ; voir l’[API Win32 DACL](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setsecurityinfo).

Aucun nettoyage n’a lieu tant qu’un verrou est détenu. La sortie terminée est conservée au moins sept jours, puis les dossiers expirés et déverrouillés sont nettoyés lors d’une ouverture temporaire ultérieure. Le cache conserve au plus 256 entrées et refuse explicitement au-delà ; chaque analyse est limitée à 4 096 entrées. Le nettoyage vérifie permissions privées, marqueurs et noms et ne suit pas les liens. Les modifications de la copie temporaire ne sont pas réécrites dans l’archive.

Le nettoyage se limite au cache privé créé par l’application ; il ne parcourt ni ne supprime les dossiers d’extraction de l’utilisateur. L’acceptation couvre le masquage automatique après succès, le maintien des fenêtres en cas d’échec, l’annulation, les nouvelles tentatives de mot de passe, les noms en double, Unicode et caractères spéciaux, le routage des tâches après fermeture de page, la sécurité du dossier temporaire et la politique de nettoyage à la fermeture.

L’admission en lecture seule de [Brotli/TAR.BR](brotli-streams.md) valide complètement les candidats sans en-tête avant publication des lignes, partage le compteur de lectures des candidats Brotli/LZMA, conserve la priorité des sources fortes et de la découverte TAR/ZIP valide, et ne rétrograde jamais un TAR décodé endommagé en flux ordinaire.

Les propositions [LHA/LZH](../../engineering/lha-containers.md) exigent la validation complète et bornée des en-têtes et noms avant publication, y compris pour une archive vide constituée d’un seul zéro. Les propositions faibles LHA, Brotli et LZMA partagent un compteur des lectures réelles. La découverte de conteneurs valides forts garde la priorité ; une admission LHA complète peut lever l’ambiguïté d’un marqueur TAR brut invalide. Aucun profil de mot de passe, de volumes ou de réparation n’est admis.

[PMA](pma-streams.md) utilise le même compteur partagé des sondes faibles et l’admission complète des métadonnées, avec un terminateur zéro exact et un remplissage CP/M facultatif et borné. Les contenus PM0/PM1/PM2 de niveau 0 sont pris en charge ; les méthodes inconnues restent visibles mais non prises en charge. Un conteneur CP/M vide et rempli identifie PMA ; un terminateur zéro d’un octet indiscernable conserve le choix LZH antérieur. Aucun profil de création, mot de passe, volumes, modification ou réparation n’est admis.
