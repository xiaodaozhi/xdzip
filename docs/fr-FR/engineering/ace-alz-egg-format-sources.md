<a name="xdzip-ace-alz-and-egg-source-and-licensing-gates"></a>

# Sources et conditions de licence pour ACE, ALZ et EGG

[English](../../engineering/ace-alz-egg-format-sources.md) | [简体中文](../../zh-CN/engineering/ace-alz-egg-format-sources.md) | [繁體中文](../../zh-TW/engineering/ace-alz-egg-format-sources.md) | [日本語](../../ja-JP/engineering/ace-alz-egg-format-sources.md) | [한국어](../../ko-KR/engineering/ace-alz-egg-format-sources.md) | Français | [Deutsch](../../de-DE/engineering/ace-alz-egg-format-sources.md) | [Español](../../es-ES/engineering/ace-alz-egg-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/ace-alz-egg-format-sources.md) | [Русский](../../ru-RU/engineering/ace-alz-egg-format-sources.md)

Cette page décrit la prise en charge d’ACE, d’ALZ et d’EGG dans xdzip et les références techniques publiques et restrictions de licence qui motivent les décisions. Les fichiers de référence restent dans `build/toolchain/references`, hors des dépendances du produit.

<a name="xdzip-ace"></a>

## ACE

- Note technique de Marcel Lemke, *Technical information of the archiver ACE v1.2*, obtenue auprès de `wdlkmpx/unace`, SHA-256 `F645C91348D5DCD560A19CA21DEF849D25C304EFFAACF65B03EFBB0A893E51F9`.
- Implémentation indépendante `droe/acefile`, commit `11ed076ae19e8228148731cfbc05f019a395b3b9`. SHA-256 du texte de licence BSD-2-Clause `AD376BD9695F0E355F5C523443ABF822BB896999A3252AAA614D313CC4F1F5B5`. SHA-256 de l’implémentation `BC701569AD8521964C62256472AB44976AC57FB46E52B147A21018BC96348F1F`.
- L’implémentation annonce la lecture d’ACE 1.0 et 2.0, dont LZ77, DELTA, EXE, PIC, SOUND, les mots de passe et les volumes. Sa licence couvre uniquement l’implémentation Python et ne revendique expressément aucun droit sur la technologie ACE.

La documentation technique permet de définir les en-têtes de blocs, les CRC, les membres stockés sans compression et l’ancienne famille LZ77. Elle ne constitue pas une autorisation commerciale pour la technologie ACE. Les renvois de l’implémentation à WinAce et unace ne forment pas un corpus indépendant d’interopérabilité. ACE est définitivement exclu de xdzip. Il ne sera ni implémenté ni enregistré, et aucun suffixe, détecteur, type MIME ou association ACE ne doit être ajouté.

<a name="xdzip-alz"></a>

## ALZ

- Spécification `alkegi/docs`, commit `36230c34d3056604cc63519e9855a79e894496b7`. SHA-256 de la licence CC0 `F4E7F373B9B996950337E8D41A4A2939C2D90B7725E9BAF3D5084A22717AD328`.
- SHA-256 de `alz.md` `CBD35E77AB53CFD08C0FF192FA2793438150E4935522608936B1A0942560E237`.
- Le document spécifie les signatures ALZ et de fichiers locaux, les tailles de largeur variable, CRC-32, les noms UTF-8/CP949, Store, le Deflate brut de RFC 1951 et un flux BZip2 modifié par ALZ. Il décrit aussi le chiffrement traditionnel PKWARE et la disposition des volumes fractionnés.

xdzip prend en charge un sous-ensemble ALZ en lecture seule. Les archives ordinaires à un volume peuvent utiliser Store ou Deflate brut sans chiffrement, avec vérification exacte du CRC, des tailles et de la terminaison. Des tests automatiques couvrent le codec, l’exécution, la CLI et les associations. BZip2 modifié, ZipCrypto et les volumes fractionnés exigent des échantillons complets indépendants et des tests distincts de ressources et de sécurité. La licence CC0 du document ne suffit pas à établir l’absence d’obligations de tiers en matière de brevets ou de marques pour tous les détails d’implémentation.

<a name="xdzip-egg"></a>

## EGG

- Spécification publique `alkegi/docs`, même commit `36230c34d3056604cc63519e9855a79e894496b7` sous la licence CC0 ci-dessus.
- SHA-256 de `egg.md` `4CA2CB6E8FA9A4F40C30EDE05A5E977BE113A96E1527E28A7B1ED0836D8A43A3`.
- Module de référence `dterracino/UnEgg`, commit `f21e653b02f7ab467bbc16af879e68a48dabcb2b`. SHA-256 du PDF de spécification EGG `0BB7E6F174CAFB9F6726D70B7DEE8DBF33C034717058B1B0EB2B372D9A8FBB37`.
- SHA-256 de la licence source UnEGG `9B012928756D9243CC549FA0A7D7B5742ED8F69B00A2BE1C964F42EF21F1D8D6` pour le paquet source et `30F2E45148D43D7A77574403807C13AA359EE9ACE841A00B6A5348AD961423F0` pour le paquet du module.

La licence UnEGG exige l’accord d’ESTsoft pour un usage commercial, limite la redistribution à un usage non commercial et interdit d’utiliser le module pour développer un compresseur EGG. Elle indique qu’ESTsoft conserve tous les droits. Le module sert uniquement de référence de test et ne peut être copié, lié ou utilisé comme dépendance du produit.

Le format peut être implémenté indépendamment du module soumis à restrictions. xdzip admet une implémentation indépendante de Store et Deflate brut sur un seul volume, sans chiffrement ni mode solide. Les tests du codec, de l’exécution, de la CLI et des associations couvrent l’exactitude des tailles de blocs et le CRC-32. BZip2 standard et l’encapsulation LZMA documentée nécessitent encore des échantillons complets distincts et une revue. AZO, AES/LEA, ZipCrypto et les archives fractionnées ou solides restent hors du périmètre actuel.

ALZ et EGG valident séparément leurs en-têtes de conteneur et vérifient les entrées physiques par rapport à leurs instantanés de métadonnées. Après admission, les deux formats utilisent le même lecteur borné pour Store et Deflate brut. Il contrôle la longueur déclarée et le CRC-32 et comptabilise les entrées, les octets décodés et le travail au fil des éléments.

<a name="xdzip-required-implementation-gates"></a>

## Conditions d’admission d’une implémentation

1. Conserver les références hors de l’arborescence du produit et vérifier les hachages fixés avant usage.
2. Constituer des échantillons complets indépendants pour chaque méthode admise. Un échantillon limité à l’en-tête ou à Store ne vérifie pas l’interopérabilité d’une méthode.
3. Introduire ensemble le conteneur, l’exécution, la CLI et le bureau. Les méthodes inconnues restent visibles sans autorité sur les données.
4. Désactiver les mots de passe, volumes, état solide et objets spéciaux jusqu’à validation de leurs propres sources et tests de sécurité et d’annulation.
5. Consigner la décision juridique et les identités des échantillons dans le ticket avant de modifier le registre public.
