<a name="xdzip-auxiliary-algorithm-work-accounting"></a>

# Comptabilisation du travail auxiliaire des algorithmes

[English](../../engineering/auxiliary-decoding-work.md) | [简体中文](../../zh-CN/engineering/auxiliary-decoding-work.md) | [繁體中文（台灣）](../../zh-TW/engineering/auxiliary-decoding-work.md) | [日本語](../../ja-JP/engineering/auxiliary-decoding-work.md) | [한국어](../../ko-KR/engineering/auxiliary-decoding-work.md) | Français | [Deutsch](../../de-DE/engineering/auxiliary-decoding-work.md) | [Español](../../es-ES/engineering/auxiliary-decoding-work.md) | [Português (Brasil)](../../pt-BR/engineering/auxiliary-decoding-work.md) | [Русский](../../ru-RU/engineering/auxiliary-decoding-work.md)

L’extraction et la vérification comptabilisent séparément les octets décodés des fichiers, la sortie réelle et le travail auxiliaire des algorithmes. L’initialisation des modèles PPM, les réinitialisations dues à la capacité et le décodage de commandes ou de paramètres sont extérieurs au contenu des fichiers. Ils ne doivent pas augmenter `decoded_bytes`, `written_bytes`, la progression des fichiers ou le taux d’expansion, ni produire de crédit de livraison précomptabilisée. Les instantanés de tâches n’ajoutent aucun compteur auxiliaire visible par l’utilisateur.

`DecodedSink::charge_auxiliary(bytes)` signale uniquement le travail auxiliaire déjà effectué. Son implémentation par défaut ne modifie aucun compteur de fichiers. Les algorithmes autorisent d’abord le travail prévu selon leur budget restant, puis imputent le travail réel après exécution, y compris ce qui a été effectué avant une erreur ou une annulation. Les réservations non utilisées ne sont pas déclarées comme travail réel. Les erreurs, réinitialisations du modèle ou nouvelles tentatives de mot de passe ne remboursent pas le travail déjà effectué. Le budget privé de RAR continue de limiter la somme du décodage de fichiers et du travail auxiliaire.

<a name="xdzip-isolation-boundary"></a>

## Frontière d’isolation

- Le message `AUX_WORK=24` transmet un compteur `u64` little-endian sans données de contenu. Chaque paquet contient une valeur de 1 à 65 536 et la trame doit se terminer exactement à cet endroit. L’hôte répond par un `ACK` vide après comptabilisation. Refuser zéro, troncatures, champs supplémentaires, valeurs excessives et phases invalides.
- Ce message est autorisé uniquement pendant le décodage des données RAR, à l’exclusion de l’analyse des métadonnées, de la création, des modifications et des autres formats. CXX transmet les compteurs de manière synchrone et ne conserve pas de pointeurs Rust. Les tranches source et de sortie sont empruntées pour la durée de l’appel. Les exceptions C++ deviennent des erreurs stables dans les rappels. Le déroulement de pile Rust ne traverse pas CXX. Un arrêt brutal ou la sortie du processus fait échouer la sortie inachevée.
- Une instance native de décodage accumule jusqu’à 20 GiB de travail auxiliaire, en conservant le total entre entrées, redémarrages et changements de mot de passe. Elle enregistre le travail achevé avant d’appeler la destination de l’hôte. Un refus de celle-ci n’accorde aucun remboursement lors d’une nouvelle tentative. `AUX_WORK` n’augmente pas `WorkerCredit`.
- Les tâches d’extraction et de vérification du cœur comptabilisent indépendamment jusqu’à 20 GiB de travail auxiliaire. L’objet de comptabilisation reste hors des boucles d’entrées et de nouvelles tentatives de mot de passe. Ce budget est vérifié indépendamment des octets effectifs des fichiers et des limites de sortie. Le dépassement ou l’annulation renvoie un échec et ne publie pas de fichier inachevé. Modifier des compteurs publics ne contourne pas ces limites.
- `WORK=17` continue de représenter les octets de fichiers décodés. Seule son admission réussie accorde au même processus encore valide un crédit utilisable une fois pour `PRECHARGED_DATA=23`. Le crédit est déduit avant livraison et effacé au redémarrage. RAR, 7z et CAB utilisent cette voie. L’analyse et la création ne l’utilisent pas. Le travail auxiliaire ne peut pas payer la sortie des fichiers.

L’autorisation des taux élevés est déterminée par les tailles déclarées et les tailles compressées ou d’archive entière qui ont été figées. Le décodage de fichiers et les limites de sortie restent appliqués indépendamment. Si un fichier de 87 octets nécessite plusieurs MiB d’initialisation du modèle, les compteurs de fichiers reflètent toujours ses seuls 87 octets réels.

<a name="xdzip-required-acceptance-conditions"></a>

## Conditions de validation

- Un petit fichier normal peut terminer l’extraction et la vérification après plus de 64 KiB de travail auxiliaire et d’initialisation du modèle. Les compteurs restent exacts et la vérification finale du contenu réussit.
- La couche native et le cœur vérifient chacun le refus lorsque des paquets auxiliaires effectivement traités atteignent 20 GiB. Les nouveaux essais de mot de passe et le remplacement du processus ne réinitialisent pas les coûts cumulés. Les fichiers inachevés ne sont pas publiés.
- Les requêtes d’un processus simulé échouent pour les autres formats, les phases de métadonnées, les paquets vides ou trop grands, les champs supplémentaires et la sortie précomptabilisée financée uniquement par du travail auxiliaire.
- `WORK` ordinaire peut financer une sortie RAR précomptabilisée pour le même processus. Le crédit est consommé une fois et ne peut être réutilisé après une erreur ou un redémarrage.
- Les fixtures du protocole vérifient la machine d’état et la comptabilisation. Le processus RAR complet en Rust, le chemin opérationnel du cœur, les trois plateformes et la validation du bureau ont leurs propres preuves d’exécution.

La fixture du protocole compile uniquement le fichier `tools/native-codec/main.cpp` du projet et son simulacre, sans algorithme de compression.

```sh
cmake -S crates/xdzip-codec-native/tests -B build/auxiliary-protocol
cmake --build build/auxiliary-protocol --config Release
```

Faites pointer `XDZIP_NATIVE_CODEC_WORKER` vers la fixture, définissez `XDZIP_AUXILIARY_PROTOCOL_FIXTURE=1` et lancez explicitement `cargo test -p xdzip-codec-native auxiliary_protocol -- --ignored`. La régression ordinaire ciblée utilise `cargo test -p xdzip-codec-native -p xdzip-runtime auxiliary`. Les tests de l’API codec vérifient la compatibilité de la destination par défaut. Les preuves générées restent dans build ou la CI. Les résultats et les points restants figurent dans la [tâche de migration #51](https://github.com/xiaodaozhi/xdzip/issues/51).
