<a name="xdzip-bandizip-format-inventory-and-xdzip-feasibility"></a>

# Inventaire des formats Bandizip et faisabilité pour xdzip

[English](../../engineering/bandizip-format-assessment.md) | [简体中文](../../zh-CN/engineering/bandizip-format-assessment.md) | [繁體中文](../../zh-TW/engineering/bandizip-format-assessment.md) | [日本語](../../ja-JP/engineering/bandizip-format-assessment.md) | [한국어](../../ko-KR/engineering/bandizip-format-assessment.md) | Français | [Deutsch](../../de-DE/engineering/bandizip-format-assessment.md) | [Español](../../es-ES/engineering/bandizip-format-assessment.md) | [Português (Brasil)](../../pt-BR/engineering/bandizip-format-assessment.md) | [Русский](../../ru-RU/engineering/bandizip-format-assessment.md)

Cette évaluation compare les formats énumérés par Bandizip au périmètre d’implémentation de xdzip et identifie les capacités exigeant une vérification des sources, échantillons et tests automatisés. L’inventaire provient de la page produit Bandizip et de ses pages d’aide sur les formats, consultées le 2026-11-15 :

- La [page produit Bandizip](https://www.bandisoft.com/bandizip/) énumère : `7Z`, `ACE`, `AES`, `ALZ`, `ARJ`, `BH`, `BIN`, `BR`, `BZ`, `BZ2`, `CAB`, `Compound(MSI)`, `DAA(1.0)`, `DEB`, `EGG`, `GZ`, `IMG`, `ISO`, `ISZ`, `LHA`, `LZ`, `LZH`, `LZMA`, `PMA`, `RAR`, `RAR5`, `SFX(EXE)`, `TAR`, `TBZ/TBZ2`, `TGZ`, `TLZ`, `TXZ`, `UDF`, `WIM`, `XPI`, `XZ`, `Z`, `ZIP`, `ZIPX`, `ZPAQ`, `PEA`, `UU`, `UUE`, `XXE`, `ASAR`, `ZSTD`, `NSIS` et `LZ4`.
- La même page indique que Bandizip crée `ZIP`, `7Z`, `ZIPX`, `EXE (SFX)`, `TAR`, `TGZ`, `LZH`, `ISO`, `GZ` et `XZ` ; la liste plus longue concerne principalement l’extraction.
- L’[aide Bandizip sur ALZ et EGG](https://www.bandisoft.com/bandizip/help/how-to-open-alz-and-egg-files/) décrit Deflate, Deflate obfusqué, BZip2 et ZipCrypto pour ALZ, ainsi que Deflate, BZip2, LZMA, AZO, le chiffrement AES/LEA, les blocs solides et archives divisées pour EGG.
- L’[aide Bandizip sur ZPAQ](https://www.bandisoft.com/bandizip/help/what-is-a-zpaq/) décrit ZPAQ comme un format d’archive journalisé distinct et énumère les méthodes de compression prises en charge par Bandizip.
- Le [guide des sources de formats d’images disque et autres archives](disk-image-archive-format-sources.md) consigne les vérifications des sources publiques, licences, corpus et développement en salle blanche pour DAA, ISZ, ZPAQ, PEA, BOND et J2J. BOND et J2J ne sont pas confirmés comme formats d’archive officiels actuels de Bandizip et ne restent que des pistes historiques.

<a name="xdzip-current-xdzip-disposition"></a>

## Situation actuelle dans xdzip

Le tableau utilise les sens suivants. **Admis** signifie que le registre actuel, le runtime, la CLI et le parcours bureau disposent de preuves automatisées pour le profil indiqué. **Alias/profil** signifie que l’extension est admise par un conteneur existant seulement après validation du contenu. **Candidat** signifie que le suffixe est une entrée future utile mais qu’aucun décodeur n’est revendiqué. **Bloqué** signifie que le format ne peut pas être admis de manière responsable avant disponibilité d’une spécification filaire publique et testable indépendamment, et d’une implémentation compatible ou d’un algorithme propre. **Exclu** signifie que le projet a explicitement décidé de ne pas prendre en charge le format ; aucun registre ni détecteur ne lui est associé tant que cette décision produit n’est pas revue.

| Nom Bandizip | Statut xdzip | Portée actuelle défendable ou raison |
| --- | --- | --- |
| 7Z | Admis | Backend 7z Rust indépendant, avec les profils déclarés de mot de passe, solide et volumes. |
| ACE | Exclu | Décision produit : abandon définitif d’ACE ; aucun décodeur, détecteur, suffixe, MIME ni association ne sera ajouté. |
| AES | Admis | AES Crypt v0–v3, profil de mot de passe et HMAC ; il s’agit de l’enveloppe `.aes`, pas d’archives chiffrées arbitraires. |
| ALZ | Admis | Store et Deflate brut ordinaires à volume unique en lecture seule, tailles exactes et CRC-32 ; BZip2 modifié, ZipCrypto et volumes divisés restent hors profil. |
| ARJ | Admis | Store et méthodes 1–4 en lecture seule avec en-têtes complets, tailles et CRC-32 des membres. |
| BH | Admis | Membres ordinaires `BH 05 07` en lecture seule, méthode 8 Deflate brut, tailles exactes et CRC-32. Store, chiffrement, volumes et autres méthodes sont exclus. |
| BIN | Candidat | Extension ambiguë entre disque, exécutable et données. Détecter seulement un format interne validé ; ne jamais enregistrer BIN générique comme décodeur d’archive. |
| BR | Admis | Flux Brotli borné et profils TAR.BR. |
| BZ / BZ2 | Admis | Flux BZip2 à fichier unique et alias TAR.BZ2 déclarés. |
| CAB | Admis | Backend CAB Rust indépendant avec Store, MSZip, Quantum, LZX et ensembles de cabinets autorisés. |
| Compound(MSI) | Admis | Structure CFB, noms de stockages/flux et copie bornée de flux bruts en lecture seule pour un contenu `.msi` validé. Tables MSI, interprétation CAB intégré, transformations, actions personnalisées et installation restent hors capacité. |
| DAA(1.0) | Exclu | Définitivement non pris en charge par décision produit ; aucun détecteur, codec, registre, lecteur ou écrivain n’est admis. |
| DEB | Admis | Enveloppe `ar` courante, vues control/data et membres comprimés admis. |
| EGG | Admis | Store et Deflate brut ordinaires, non chiffrés, non solides et à volume unique en lecture seule, tailles exactes des blocs et CRC-32 ; AZO, BZip2, LZMA, chiffrement, volumes divisés et autres profils restent hors périmètre. |
| GZ | Admis | Membres GZIP, y compris flux concaténés et TAR.GZ. |
| IMG | Candidat | Suffixe d’image disque ambigu. L’admission ISO/UDF fondée sur le contenu ne signifie pas une prise en charge générique d’IMG. |
| ISO | Admis | Lecture ISO9660/Joliet/Rock Ridge et création d’images de données ordinaires ISO9660 + Joliet. |
| ISZ | Exclu | Décision produit : abandonner ISZ ; aucun détecteur, registre ni traitement de clés ne sera ajouté. |
| LHA / LZH | Admis | Niveaux LHA 0–3, Store, LH1–LH7, LZS/LZ5 et création LH5. |
| LZ | Admis | Lzip v1 et TAR.LZ. |
| LZMA | Admis | LZMA Alone et TAR.LZMA. |
| PMA | Admis | Profils PM0/PM1/PM2 en lecture seule avec cadrage CP/M et contrôles CRC. |
| RAR / RAR5 | Admis | Backend RAR Rust indépendant, avec anciens formats, RAR5, extensions RAR7, mots de passe, solide et volumes dans le périmètre déclaré. |
| SFX(EXE) | Alias/profil | Les préfixes ZIP SFX sont validés par le lecteur ZIP. L’extraction ou l’exécution générique d’exécutables n’est jamais autorisée. La création ZIP Store/Deflate Windows x64 propre au projet est disponible par le profil explicite `.sfx.exe`, sans exécution automatique. |
| TAR | Admis | TAR et profils filtrés TAR.GZ/TAR.BZ2/TAR.LZMA/TAR.LZ/TAR.XZ/TAR.Z/TAR.BR/TAR.LZ4/TAR.ZST énumérés par le registre. |
| TBZ / TBZ2 | Alias/profil | Alias TAR filtrés par BZip2. |
| TGZ | Alias/profil | Alias TAR filtré par GZIP. |
| TLZ | Alias/profil | Alias TAR filtré par LZMA. |
| TXZ | Alias/profil | Alias TAR filtré par XZ. |
| UDF | Admis | Images UDF en lecture seule, avec règles d’espace de noms des images hybrides. |
| WIM | Exclu | Décision produit : ne pas poursuivre le lecteur, modèle de ressources et corpus propres à l’imagerie requis pour WIM. |
| XPI | Alias/profil | Contenu ZIP sous suffixe XPI ; la validation du contenu fait autorité. |
| XZ | Admis | XZ et TAR.XZ avec LZMA2, contrôles déclarés et filtres admis. |
| Z | Admis | LZW Unix compress et alias TAR.Z/TAZ. |
| ZIP | Admis | ZIP/ZIP64, AES-256 et profils pris en charge de volumes, méthodes et réécriture. |
| ZIPX | Alias/profil | ZIPX est admis pour des méthodes ZIP validées telles que XZ ; ce n’est pas une implémentation de conteneur distincte. La création ZIPX n’est pas actuellement exposée. |
| ZPAQ | Exclu | Décision produit : ne pas implémenter la machine virtuelle ZPAQL requise par ce format solide et journalisé. |
| PEA | Admis | Profil en lecture seule PEA 1.x à volume unique, UTF-8 et sans chiffrement pour fichiers/répertoires stockés PCOMPRESS0 ; contrôles, chiffrement, compression, volumes, création et modification restent hors profil. |
| UU / UUE | Admis | Enveloppes UU/UUE bornées avec validation des membres physiques et lignes. |
| XXE | Admis | Profil d’enveloppe XXE borné. |
| ASAR | Admis | Métadonnées ordonnées, plages stockées et contrôles SHA-256 déclarés. |
| ZSTD | Admis | Trames Zstandard standard et profils TAR.ZST/TZST. |
| NSIS | Admis | Métadonnées statiques `firstheader` seulement. La détection du contenu valide un overlay PE borné ; `.exe` n’est pas une entrée de glisser-déposer ni d’association shell, et aucune exécution de PE, script, plugin ou charge utile n’est permise. |
| LZ4 | Admis | Trames LZ4 standard et TAR.LZ4. |

L’inventaire ne justifie donc pas l’ajout de gestionnaires génériques pour `BIN`, `IMG` ou `SFX(EXE)`. Ces noms peuvent désigner des données exécutables, de disque ou d’installateur, et un suffixe seul ne prouve pas une capacité d’archivage. NSIS dispose d’un lecteur plus limité de métadonnées statiques validées par contenu ; il ne donne jamais autorité d’exécution ni sur la charge utile et son entrée `.exe` est masquée pour le glisser-déposer et les associations shell. Compound(MSI) possède désormais un lecteur CFB validé par contenu ; le suffixe `.msi` n’accorde toujours aucune autorité d’exécution.

Voir [sources des formats d’images disque et d’archives](disk-image-archive-format-sources.md) pour DAA, ISZ, ZPAQ, PEA, BOND et J2J. PEA est disponible dans le profil conservateur en lecture seule documenté. ACE, DAA, ISZ et ZPAQ sont exclus par décision produit. BOND et J2J sont exclus car leur identité comme formats d’archive Bandizip n’est pas confirmée.

<a name="xdzip-delivery-plan"></a>

## Plan de livraison

1. **Inventaire et registre.** Conserver une entrée lisible par machine par profil admis dans `specs/engineering/archive-formats.json`. Générer les consommateurs Rust, C++, JavaScript et d’associations, puis exécuter les deux contrôles du registre. Chaque nouveau format reçoit un contrat et un paragraphe d’acceptation correspondant.
2. **Contrat filaire avant le code.** Fixer révisions publiques et SHA-256, documenter champs d’en-tête, encodage des noms, terminaison physique, méthodes, sommes de contrôle, identité des doublons et limites de ressources. Un outil de référence ne constitue une preuve que s’il produit des échantillons complets propres au projet ; un crash ou un seul échantillon Store est une limite enregistrée.
3. **Tests d’algorithme brut.** Implémenter le lecteur borné indépendamment de Qt. Couvrir charges vides et limites, chaque méthode admise, budgets exacts d’entrée/sortie/travail/mémoire, annulation, E/S typées, modification de source, troncature, octets supplémentaires, CRC et état terminal après erreur. Les méthodes inconnues restent visibles sans autorité sur la charge utile.
4. **Admission du conteneur et du produit.** Ajouter le crate codec, la détection faible/forte du runtime, list/test/extract de la CLI, libellés bureau et échantillons d’association. Vérifier les métadonnées figées et la sélection des doublons physiques, puis exécuter les contrôles Qt et de disposition automatiques pertinents. Aucune acceptation visuelle manuelle n’est nécessaire pour cette tranche de recherche et de format.
5. **File de priorités.** Les tranches conservatrices ALZ Store/Deflate brut, EGG non chiffré et non solide Store/Deflate brut, métadonnées CFB/Compound(MSI), stockage conservateur PEA et métadonnées statiques NSIS `firstheader` sont admises. ACE, DAA, WIM, ISZ, ZPAQ, BOND et J2J sont définitivement exclus par décision produit. La création ZIPX reste exclue des écrivains actuels. L’[écrivain SFX propre au projet](sfx-creation.md) est implémenté séparément et ne découle pas de la prise en charge des alias en lecture.
6. **Limite de publication.** Garder les formats candidats hors du tableau public des capacités tant que les preuves codec, runtime, CLI et bureau ne sont pas complètes. Consigner les méthodes non prises en charge et limites des outils de référence dans l’issue, sans repli silencieux vers Store ou un autre conteneur.

La tranche BH actuelle suit ce plan : lecture seule, sans parcours mot de passe ni volume, avec lecteur Deflate borné indépendant, CRC-32 obligatoire et terminaison physique exacte. La tranche Compound(MSI) suit la même limite : validation CFB FAT/DIFAT, arborescence, mini-flux et cycles indépendante de Qt, et copie de flux bruts seulement après correspondance de l’instantané immuable de métadonnées. La tranche NSIS s’arrête à l’enregistrement fixe de métadonnées firstheader et renvoie une absence de prise en charge pour l’accès à la charge utile. Aucun comportement d’installateur MSI ou NSIS n’est exécuté.
