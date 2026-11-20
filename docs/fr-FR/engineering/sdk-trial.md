<a name="xdzip-external-read-only-sdk-trial"></a>

# Essai d’un SDK externe en lecture seule

[English](../../engineering/sdk-trial.md) | [简体中文](../../zh-CN/engineering/sdk-trial.md) | [繁體中文（台灣）](../../zh-TW/engineering/sdk-trial.md) | [日本語](../../ja-JP/engineering/sdk-trial.md) | [한국어](../../ko-KR/engineering/sdk-trial.md) | Français | [Deutsch](../../de-DE/engineering/sdk-trial.md) | [Español](../../es-ES/engineering/sdk-trial.md) | [Português (Brasil)](../../pt-BR/engineering/sdk-trial.md) | [Русский](../../ru-RU/engineering/sdk-trial.md)

`xdzip-plugin-protocol` et `xdzip-plugin-host` implémentent l’essai indépendant en lecture seule décrit dans la [conception d’extension des formats](../design/codec-extension-sdk.md). Il est exclu de la découverte de formats du bureau et de la CLI, de l’installation, du packaging et du démarrage automatique. L’utilisateur lance explicitement la commande d’essai et choisit paquet et source ; les extensions tierces restent désactivées dans le produit. Seul `xdp1` est négocié, sans création, mots de passe, volumes, liens, interface graphique ni autorisation arbitraire de chemins hôtes. Ce n’est ni un écosystème complet de plugins ni une publication du SDK.

<a name="xdzip-reproducible-entry-points"></a>

## Points d’entrée reproductibles

```sh
python3 scripts/sdk-trial.py test
python3 scripts/sdk-trial.py example --output-dir build/sdk-example
target/debug/xdzip-plugin-host scan build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp
mkdir build/sdk-example/output
target/debug/xdzip-plugin-host read build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp 0 build/sdk-example/output
```

Sous Windows, utiliser `python` et `.exe`. `test` compile le programme natif de référence et exécute les régressions Rust de cadrage, l’interopérabilité Python indépendante, les régressions de protocole malveillant, annulation, modification de source, conflit et permissions du noyau. Python n’est lancé que par le point d’entrée de conformité fiable compilé pour les tests ; l’hôte distribué n’accepte ni interpréteur, arguments, shell, ni option d’exécution sans bac à sable. L’exemple génère un paquet propre, un manifeste et XDP1 uniquement sous `build/` ; il ne déclenche ni publication ni téléchargement tiers. Les journaux omettent les chemins d’espace de travail et de répertoire personnel ; le JSON lie commit, état modifié et hachages binaires. Les deux sont des artefacts locaux ou CI. La même matrice native Foundation appelle ce point d’entrée ; le [workflow distinct d’essai SDK](../../../.github/workflows/sdk-trial.yml) traite d’abord la compatibilité des bacs à sable sans répéter les pipelines sans rapport de formats et Qt. Le workflow Foundation peut aussi appeler l’essai réutilisable avec `sdk_only=true` ; cela ignore la validation Foundation complète et ne prouve pas un résultat Foundation complet. Sous Windows, on vérifie le refus d’exécution.

<a name="xdzip-package-admission"></a>

## Admission des paquets

Le manifeste JSON strict utilise `deny_unknown_fields`. Les champs sont `manifest_version=1`, `plugin_id`, `version`, `author`, `protocol_majors=[1]`, `targets`, un `entrypoint` dans le paquet, `sha256` en hexadécimal minuscule, `formats=["xdp1"]`, `permissions`, `resources`, `license_files` et une `source_url` HTTPS. Le manifeste de référence généré par `example` ci-dessus est une entrée exécutable du schéma actuel.

Les `permissions` sont exactement `broker_source_read=true`, `broker_output_write=true`, `network=false` ; les `resources` sont exactement `memory_bytes=536870912`, `threads=1`. Le manifeste est limité à 32 KiB, l’exécutable natif à 64 MiB et quatre fichiers de licence au plus sont permis, chacun limité à 128 KiB. Points d’entrée et licences ne peuvent être absolus, contenir une remontée parentale ni sortir par liens ou points de réanalyse. Chaque membre est ouvert composant par composant sans suivre les liens à travers un répertoire cap-std détenu, sans réouverture par sa chaîne d’origine après vérification. L’hôte copie depuis le fichier ordinaire détenu, vérifie le nombre réel d’octets, SHA-256 et la version du fichier avant et après copie, puis n’exécute que la copie temporaire privée, conservée jusqu’à la fin du processus et de l’IPC. Le hachage déclaré établit la cohérence du contenu du paquet choisi ; il n’authentifie ni l’éditeur ni la sécurité du code.

<a name="xdzip-wire-protocol-and-state"></a>

## Protocole filaire et état

Un sous-ensemble CBOR propre et borné suit [RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html) sans implémentation CBOR tierce. Le pair Rust de référence réutilise le crate de cadrage ; le [pair Python](../../../examples/sdk/reference.py) implémente indépendamment encodage, décodage et analyse XDP1 sans importer Rust ni code de protocole généré. L’interopérabilité couvre plusieurs blocs de contenu réel et le condensé de l’hôte.

Les trames sont `u32 LE length + CBOR` ; leur longueur est contrôlée avant allocation. Limites : trame 1 MiB, bytes/text individuels 256 KiB, profondeur 16, tableaux 128, maps 32 et nœuds décodés 8192. Seuls UInt, bytes, texte UTF-8 strict, tableaux, maps à clés UInt et bool sont acceptés. Entiers non minimaux, clés dupliquées, longueurs indéfinies, entiers négatifs, tags, flottants et données finales sont rejetés. Les clés d’enveloppe `1..5` représentent kind, request, operation, sequence et payload. La séquence commence à 1 et augmente consécutivement dans chaque direction ; request augmente à chaque invocation et operation est une identité hôte aléatoire non nulle. Chaque réponse doit correspondre exactement à l’identité courante. Les clés payload inconnues `<128` sont rejetées ; `>=128` désignent des champs facultatifs ignorables.

| kind | Message | Clés numériques du payload |
| --- | --- | --- |
| 1 | Hello | 1 major minimum, 2 major maximum, 3 ID du paquet, 4 `xdp1` |
| 2 | Welcome | 1 major, 2 limite de bloc, 3 crédit d’un bloc, 4 budget total de lecture, 5 budget de sortie |
| 10 | Probe | 1 jeton source, 2 longueur source |
| 11 | Open | 1 jeton source |
| 12 | Scan | 1 jeton de session |
| 13 | ReadEntry | 1 session, 2 entrée, 3 destination, 4 nombre de crédits 1, 5 budget de sortie, 6 jeton de crédit du premier bloc |
| 14 | Close | 1 session |
| 20, 21 | ReadAt, ReadData | 1 jeton source, 2 décalage, 3 longueur ou octets |
| 30 | EntryBatch | 1 session, 2 entrées ; par enregistrement : 1 jeton, 2 chemin brut, 3 type 0 fichier / 1 répertoire, 4 taille connue |
| 31 | ScanFinished | 1 nombre réel d’enregistrements |
| 40 | WriteData | 1 destination, 2 séquence de blocs à partir de 0, 3 octets, 4 jeton de crédit courant |
| 41 | FinishOutput | 1 destination, 2 nombre de sortie déclaré, 3 octets SHA-256 |
| 42 | WriteAck | 1 destination, 2 séquence du bloc suivant, 3 nouveau jeton de crédit aléatoire |
| 50, 51, 52 | Cancel, CancelAck, Shutdown | Map vide |
| 61 | OperationFinished | 1 résultat, 0 succès / 1 annulé ; Probe ajoute 2 format, Open ajoute 2 session |

Les autres kinds sont rejetés par la machine à états de cet essai même si le cadrage les reconnaît. Hello doit arriver en premier et les deux parties choisissent major 1 ; l’identité de négociation est zéro, puis les appels sont non nuls. ScanFinished doit encore être suivi d’un unique état terminal réussi. Une seule opération s’exécute à la fois, sans nouvelles tentatives d’extension en arrière-plan. Le crédit actuel d’un bloc est plus strict que le maximum de quatre de la conception ; un jeton imprévisible est renouvelé après acceptation réelle de chaque bloc de sortie et ne peut être prépayé ni rejoué.

<a name="xdzip-broker-budgets-and-publication"></a>

## Courtier, budgets et publication

Les jetons source, destinations et crédits par bloc sont des valeurs aléatoires de 128 bits non réutilisables entre opérations. Le pair ne reçoit ni chemins d’archive ni de destination. L’hôte renvoie au plus 256 KiB par lecture dans la plage source autorisée ; dépassement du décalage, lecture après EOF, longueur nulle et lectures cumulées supérieures à 64 MiB échouent. Il vérifie le `FileVersion` de l’objet détenu et du chemin correspondant avant et après chaque lecture et avant publication finale. Cela réutilise l’empreinte d’identité, temps et longueur existante sans prétendre détecter toute modification concurrente par un attaquant du même compte qui rétablit toutes les métadonnées.

Les métadonnées sont limitées à 1000 enregistrements physiques et 256 KiB au total pour chemins et jetons, avec jetons d’enregistrement uniques. L’hôte valide strictement les chemins bruts selon PortablePath ; les noms en double conservent des identités distinctes. Une entrée ordinaire est explicitement sélectionnée à la fois, de taille déclarée au plus 4 MiB ; les répertoires ne peuvent être une sortie fichier. L’hôte détient le Grant de destination, le Workspace privé et le Stage, mesure les octets réels et calcule SHA-256 sans donner au pair de descripteur de fichier ni d’autorité de publication.

Les E/S sûres existantes ne peuvent publier sans écraser qu’après réussite de tous les blocs, du nombre et condensé FinishOutput, de l’unique état terminal, de Close, de la sortie normale après Shutdown et EOF propre, des vérifications d’identité de source et d’annulation finale. Toute anomalie, état terminal dupliqué, crash ou délai dépassé révoque d’abord la sortie ; la destruction retire le stage avant de nettoyer le workspace. Les cibles existantes restent inchangées. L’analyse et les condensés attendus proviennent toujours de l’extension ; le calcul de l’hôte ne peut pas prouver que la sortie d’une extension malveillante est fidèle à l’archive.

La négociation est limitée à 2 secondes et chaque invocation d’essai à 5 secondes au total, sans réinitialisation du délai par la progression. Le CPU de chaque processus est limité à 3 secondes, les descripteurs à 32 ; la sortie directe sur fichier est interdite et les core dumps désactivés. Les files IPC reçoivent au plus quatre trames et en envoient une, avec une limite de 2 secondes pour l’écriture dans le pipe. stderr est directement écarté avec un budget nul ; aucun contenu non fiable n’est transmis. L’annulation révoque immédiatement la destination, accorde 2 secondes de coopération puis tue et récupère le processus et les pipes ; Ack n’autorise pas la publication. Ces budgets limitent un petit essai et ne promettent pas les performances des grandes archives ordinaires.

<a name="xdzip-platform-gates"></a>

## Conditions par plateforme

| Plateforme | Politique d’essai et comportement en cas d’échec |
| --- | --- |
| macOS | La politique deny-default de `/usr/bin/sandbox-exec` fixé autorise précisément la copie privée exécutable, les bibliothèques système et le cache dyld, les lectures du chargeur à la racine et les métadonnées `/var`. Fichiers utilisateur, écritures, réseau et fork sont interdits ; l’environnement est vidé et seul stdio est hérité. Les rlimits natifs et la surveillance RSS toutes les 20 ms terminent au-dessus de 512 MiB ou si la mesure échoue. La RSS est échantillonnée : de brefs pics entre mesures peuvent dépasser le seuil ; cette interface n’est pas promise pour une future distribution officielle. |
| Linux x86_64 / aarch64 | Landlock ABI ≥3 traite tous les droits fichier v3, sans règle d’écriture et avec lecture limitée à l’exécutable et aux bibliothèques du chargeur système. `no_new_privs` et une liste d’appels seccomp vérifiée selon l’architecture incluent readv/writev nécessaires aux pipes standard Rust, tout en interdisant socket, fork/clone, ptrace, io_uring et autres. Une limite CPU de 3 secondes, une limite d’espace d’adressage de 512 MiB et les autres rlimits s’appliquent. Règles, ABI, seccomp ou appels requis manquants provoquent un échec avant exec. Seule la CI native établit les résultats réels de cette plateforme. |
| Windows et autres plateformes | L’exécution est systématiquement refusée. AppContainer, Job et liste autorisée de handles ne sont ni implémentés ni vérifiés ; aucun sous-processus ordinaire ne remplace le bac à sable. |

Les choix d’API Linux suivent la [documentation Landlock du noyau](https://docs.kernel.org/userspace-api/landlock.html) et l’UAPI ; les futurs travaux Windows suivent [l’isolation AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation). Le code unsafe natif de l’hôte est limité au module de bac à sable et documente dispositions, préconditions d’appel et comportement en cas d’échec ; pre-exec n’alloue ni ne journalise. Le pair de référence ajoute des sondes contrôlées fork/_exit/waitpid uniquement en mode audit explicite ; l’enfant n’exécute ni nettoyage Rust ni logique de format. Les régressions de plateforme utilisent d’abord des marqueurs propres lisibles et inscriptibles, un écouteur localhost propre et self-exec comme témoins positifs, puis vérifient le refus par le noyau de lecture, écriture, réseau, fork direct et self-exec selon l’état réel des marqueurs, fichiers et connexions. Les plateformes non testées ne sont pas déclarées réussies et l’exécution tierce du bureau reste désactivée.

<a name="xdzip-owned-xdp1-example-format"></a>

## Format d’exemple XDP1 propre

XDP1 est uniquement un échantillon d’interopérabilité et ne remplace aucun format d’archive réel. Les quatre octets `XDP1` sont suivis d’un nombre d’enregistrements `u32 LE`. Chaque enregistrement contient un nombre d’octets de nom UTF-8 `u16 LE`, un nombre d’octets de contenu `u32 LE`, le nom et le contenu non comprimé. Au plus 1000 enregistrements sont permis sans données finales ; noms et contenu sont lus dans les budgets de l’hôte. Les jetons d’entrée sont des ordinaux physiques LE sur quatre octets, sans dédoublonnage par chemin. Les modes de faute Python ne sont choisis explicitement que dans les tests de conformité pour vérifier accès non autorisé, séquence, rejeu, identité dupliquée, chemins, crédits, condensés, crashs, annulation et états terminaux dupliqués.
