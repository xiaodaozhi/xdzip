<a name="xdzip-simplified-chinese-ui-terminology"></a>

# Terminologia da interface em chinês simplificado

[English](../../ux/chinese-terminology.md) | [简体中文](../../zh-CN/ux/chinese-terminology.md) | [繁體中文](../../zh-TW/ux/chinese-terminology.md) | [日本語](../../ja-JP/ux/chinese-terminology.md) | [한국어](../../ko-KR/ux/chinese-terminology.md) | [Français](../../fr-FR/ux/chinese-terminology.md) | [Deutsch](../../de-DE/ux/chinese-terminology.md) | [Español](../../es-ES/ux/chinese-terminology.md) | Português (Brasil) | [Русский](../../ru-RU/ux/chinese-terminology.md)

O texto apresentado em chinês simplificado segue a [configuração de terminologia](../../../specs/ux/chinese-terminology.json). Os textos estáticos do Qt preservam seus IDs estáveis em resources/i18n/zh-CN.json. O HTML, os projetos atuais de alta fidelidade e a documentação de usuário usam a tabela abaixo. Documentos de engenharia e de formatos podem usar termos técnicos. Preservam-se as dimensões dos componentes, a densidade de 0.8 na janela principal e 1 no menu, o DPR e os parâmetros de escala.

| Significado | Termo padronizado em chinês simplificado | Uso |
| --- | --- | --- |
| Contêiner como ZIP ou TAR | 压缩包 | 压缩包名称, 空压缩包, 压缩包信息 |
| Nomes completos das ações | 打开压缩包, 创建压缩包, 解压 | Abreviar como “打开” e “新建” quando o contexto da barra de ferramentas estiver claro |
| Elementos no arquivo compactado | 文件, 文件夹, 内容 | 所选文件和文件夹, 全部内容 |
| Propriedades do arquivo | 原始大小, 压缩大小 | Usar os mesmos nomes nos cabeçalhos, nas propriedades da prévia e nos menus de ordenação |
| Restaurar a ordem física das entradas | 按原始顺序排列 | Distinguir da ordenação por nome ou tamanho |

“档案文件” está fora da terminologia do produto. “归档”, “归档文件” e “归档格式” ficam reservados às especificações de engenharia, às descrições de formato ou às APIs públicas. A interface usa “压缩包” mesmo quando o TAR apenas empacota arquivos. O formato e o nível indicam a capacidade de compactação.

“压缩文件” só pode indicar uma ação, como em “正在压缩文件”. A propriedade passa a se chamar “压缩大小” e o rótulo “压缩后” é desativado. A janela de criação sugere “新建压缩包.zip” e preserva os nomes dos arquivos existentes.

Preserve nomes e caminhos fornecidos pelo usuário, nomes internos aos arquivos compactados, campos de protocolo, identificadores do código e imagens históricas dos projetos. Em inglês, use “archive” conforme a gramática inglesa e “All contents” para “全部内容”. Em cabeçalhos estreitos, “Compressed” abrevia “compressed size”. Preserve os identificadores do código em inglês.

Consulte o [projeto de estados da terminologia](../../../design/hifi/terminology.html). As imagens originais, os hashes e as medições da área visível do texto permanecem como evidências históricas. O [script de medição do conteúdo](../../../scripts/check-html-body.py) verifica os textos novos com correspondências explícitas dos rótulos anteriores, mantendo as metas geométricas e as tolerâncias. Verifique a exibição completa dos textos longos na janela principal, nos cabeçalhos, na prévia e nos menus.

Execute `python3 scripts/check-ui-i18n.py` para verificar todos os recursos de idioma do produto e a terminologia em chinês simplificado. A verificação de terminologia cobre os textos estáticos do produto e admite o uso verbal autorizado de “压缩文件”. Conteúdo do usuário e termos técnicos ficam fora desse controle.

Os gerenciadores de arquivos preservam “添加到压缩文件...” e “解压每个压缩文件到单独的文件夹”, confirmados em [#270](https://github.com/xiaodaozhi/xdzip/issues/270). A configuração abre exceção somente para o texto integral desses dois IDs estáveis. Nas demais partes da interface, valem os termos estabelecidos para contêineres.
