<a name="xdzip-host-calendar-and-encoding-services-for-format-workers"></a>
# Services de calendrier et d’encodage de l’hôte pour les processus de format

[English](../../engineering/host-platform-services.md) | [简体中文](../../zh-CN/engineering/host-platform-services.md) | [繁體中文](../../zh-TW/engineering/host-platform-services.md) | [日本語](../../ja-JP/engineering/host-platform-services.md) | [한국어](../../ko-KR/engineering/host-platform-services.md) | Français | [Deutsch](../../de-DE/engineering/host-platform-services.md) | [Español](../../es-ES/engineering/host-platform-services.md) | [Português (Brasil)](../../pt-BR/engineering/host-platform-services.md) | [Русский](../../ru-RU/engineering/host-platform-services.md)

Rust traite les champs de format, les règles de compatibilité des mots de passe et la validation. Les règles historiques de fuseau horaire, les pages de codes OEM et ANSI ainsi que l’encodage C local relèvent de l’hôte. Ils sont accessibles par les services facultatifs de `ReadSource`. La couche de format ne doit pas les déduire du nom de fichier, de HostOS ou du décalage UTC actuel. Ces services ne dépendent pas de Qt et ne confèrent aucun droit d’accès à des chemins ni de publication de fichiers.

<a name="xdzip-calendar-projection"></a>

## Conversion en date et heure

`Timestamp` distingue les valeurs FILETIME en unités de 100 ns depuis 1601 des secondes entières non signées Unix depuis 1970. `ReadSource::local_calendar` renvoie un `Calendar` facultatif. Pour les fichiers ordinaires et les sources en mémoire, la règle par défaut utilise UTC de manière déterministe. Le processus de travail du produit utilise les règles de fuseau horaire locales du système via des rappels CXX synchrones. Windows emploie les API de fuseau horaire du système et Unix utilise `localtime_r`. Les deux vérifient le résultat. Si la conversion est indisponible, aucun calendrier n’est renvoyé et les champs non initialisés ne doivent pas être lus.

Rust valide année, mois, jour, heure, minute et seconde avant d’encoder une valeur d’affichage DOS. 7z conserve UTC et la plage d’affichage 1980..2107. RAR applique ses [règles de compatibilité figées](rar-format-sources.md) pour l’arithmétique modulaire interne, les substitutions d’horodatage et les écarts entre plateformes. Il utilise ensuite le calendrier de l’hôte et le bouclage historique de l’année. La précision d’origine, les marqueurs d’absence et les en-têtes physiques complets restent dans l’index privé du format. L’affichage ne les écrase pas. Quand une date ne peut pas être convertie, la règle sûre de RAR affiche 0. L’implémentation de référence peut, après un échec système non contrôlé, recopier des données indéfinies.

<a name="xdzip-controlled-encoding-mappings"></a>

## Conversions d’encodage contrôlées

| Service | Entrée et sortie | Limite de l’hôte |
| --- | --- | --- |
| `oem_to_ansi_wide` | 512 unités UTF-16 vers 512 unités UTF-16, NUL compris | Conversion Windows complète via OEM et ANSI, avec les correspondances et substitutions réelles du système |
| `wide_to_ansi` | Au plus 512 unités UTF-16, NUL compris, vers 512 octets | Page de codes ANSI Windows actuelle. Aucun caractère universel de substitution n’est supposé |
| `wide_to_local_prefix` | Au plus 128 scalaires larges, NUL compris, vers 512 octets | Encodage C Unix actuel. En cas d’échec, la règle de compatibilité fixe prescrit un effacement et un nouvel essai, avec conservation du préfixe converti |

Les longueurs renvoyées excluent NUL. Sans services de l’hôte, les implémentations par défaut renvoient `None`. Les appelants de la couche de format peuvent uniquement appliquer une solution de repli portable définie ou rejeter explicitement la conversion. Ils ne doivent pas choisir une page de codes par supposition. Les rappels du produit vérifient les tailles, NUL et les plages des résultats. Une conversion indisponible retourne l’erreur stable Unsupported.

Le paramètre de longueur de [CharToOemBuffW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-chartooembuffw) compte les caractères larges d’entrée et la conversion se poursuit sur le remplissage après NUL. Il ne représente pas la capacité en octets de la destination. L’hôte utilise d’abord `GetCPInfo(CP_OEMCP)` pour n’accepter que les pages à quatre octets au plus par unité. Il utilise ensuite un tampon de 2 048 octets pour convertir les 512 unités. Ce n’est qu’après cette étape qu’il fixe l’octet 511 à NUL et convertit en ANSI. Si la capacité requise ne peut pas être établie, la conversion est rejetée. Il est interdit de convertir 512 caractères larges vers 512 octets ou de rogner l’entrée pour contourner cette exigence. Les tests système couvrent toute la limite DBCS de 511 unités et comparent indépendamment les résultats avec un appel WinAPI direct.

La limite de 512 unités, le premier NUL, les conditions de compensation OEM, la troncature à 127 unités ou en octets, le protocole des marqueurs historiques et l’UTF8 explicite sous macOS relèvent des règles de compatibilité en Rust. En particulier, ne tronquez pas un mot de passe avant une conversion système qui doit la précéder. Ne déplacez pas arbitrairement une coupure interne à une séquence DBCS vers une frontière de caractère.

<a name="xdzip-ownership-and-failure"></a>

## Propriété des données et échecs

Les tranches sont empruntées uniquement pendant les appels synchrones. C++ ne conserve aucun pointeur. L’appelant Rust décide du moment où effacer les entrées et sorties secrètes. Les tampons temporaires C++ de taille bornée, larges ou étroits, sont effacés sur chaque chemin de retour. L’effacement interne au système ou de toutes les copies historiques en mémoire n’est pas garanti. Ces primitives ne journalisent pas les mots de passe et ne les placent ni dans les arguments du processus ni dans les variables d’environnement.

Les lecteurs vérifient l’annulation avant d’appeler les services. Les résultats ne comptabilisent aucune sortie d’extraction et ne modifient pas les compteurs d’octets lus, décodés ou écrits. Les échecs conservent les catégories source, annulation et quota et ne doivent pas être pris pour des erreurs de mot de passe. Seule la couche de format détermine, à partir de l’état réel du vérificateur, si le mot de passe et l’intégrité restent indissociables.
