<a name="xdzip-simplified-chinese-ui-terminology"></a>

# Terminologie de l’interface en chinois simplifié

[English](../../ux/chinese-terminology.md) | [简体中文](../../zh-CN/ux/chinese-terminology.md) | [繁體中文（台灣）](../../zh-TW/ux/chinese-terminology.md) | [日本語](../../ja-JP/ux/chinese-terminology.md) | [한국어](../../ko-KR/ux/chinese-terminology.md) | Français | [Deutsch](../../de-DE/ux/chinese-terminology.md) | [Español](../../es-ES/ux/chinese-terminology.md) | [Português (Brasil)](../../pt-BR/ux/chinese-terminology.md) | [Русский](../../ru-RU/ux/chinese-terminology.md)

Les textes destinés aux utilisateurs en chinois simplifié suivent la [configuration terminologique](../../../specs/ux/chinese-terminology.json). Les textes statiques Qt gardent leurs ID stables dans resources/i18n/zh-CN.json. Le HTML, les maquettes haute fidélité actuelles et la documentation utilisateur emploient les termes du tableau. Les documents techniques et de format peuvent utiliser leur propre terminologie. Les dimensions des composants, les densités de 0.8 pour la fenêtre principale et de 1 pour les menus, le DPR et les paramètres de mise à l’échelle sont conservés.

| Sens | Terme normalisé en chinois simplifié | Emploi |
| --- | --- | --- |
| Conteneur comme ZIP ou TAR | 压缩包 | 压缩包名称, 空压缩包, 压缩包信息 |
| Noms d’action complets | 打开压缩包, 创建压缩包, 解压 | Abréger en « 打开 » et « 新建 » lorsque le contexte de la barre d’outils est clair |
| Objets de l’archive | 文件, 文件夹, 内容 | 所选文件和文件夹, 全部内容 |
| Propriétés de fichier | 原始大小, 压缩大小 | Employer ces noms dans les en-têtes, les propriétés de l’aperçu et les menus de tri |
| Rétablir l’ordre physique des entrées | 按原始顺序排列 | Distinguer du tri par nom ou taille |

« 档案文件 » est hors du vocabulaire du produit. « 归档 », « 归档文件 » et « 归档格式 » sont réservés aux spécifications techniques, aux descriptions de formats ou aux API publiques. Même quand TAR regroupe des fichiers sans les compresser, l’interface emploie « 压缩包 ». Le format et le niveau indiquent les possibilités de compression.

« 压缩文件 » désigne seulement une action, comme dans « 正在压缩文件 ». Le nom de la propriété est « 压缩大小 ». L’ancien « 压缩后 » est abandonné. La fenêtre de création propose « 新建压缩包.zip » par défaut et laisse intacts les noms des fichiers existants.

Conservez les noms et chemins fournis par l’utilisateur, les noms internes aux archives, les champs des protocoles, les identifiants du code et les images historiques des maquettes. En anglais, employez « archive » selon la grammaire anglaise et « All contents » pour « 全部内容 ». Dans les en-têtes étroits, « Compressed » abrège « compressed size ». Conservez les identifiants anglais du code.

Voir la [maquette des états terminologiques](../../../design/hifi/terminology.html). Les images originales, les hachages et les mesures de l’encre du texte restent des preuves historiques. Le [script de mesure du contenu](../../../scripts/check-html-body.py) vérifie les nouveaux textes à partir de correspondances explicites avec les anciens libellés en conservant les cibles géométriques et les tolérances. Vérifiez que les libellés plus longs s’affichent en entier dans la fenêtre principale, les en-têtes, l’aperçu et les menus.

Exécutez `python3 scripts/check-ui-i18n.py` pour vérifier toutes les ressources linguistiques du produit et la terminologie en chinois simplifié. Le contrôle terminologique porte sur les textes statiques du produit. Il admet les usages verbaux expressément autorisés de « 压缩文件 » et ne restreint pas le contenu des utilisateurs ni la terminologie technique.

Les gestionnaires de fichiers conservent « 添加到压缩文件... » et « 解压每个压缩文件到单独的文件夹 », validés par le responsable du projet dans [#270](https://github.com/xiaodaozhi/xdzip/issues/270). La configuration fait exception pour le texte intégral de ces deux ID stables. Ailleurs, les termes établis pour les conteneurs restent applicables.
