<a name="xdzip-html-visual-fidelity-review"></a>

# Revisão de fidelidade visual HTML

[English](../../visual/html-fidelity.md) | [简体中文](../../zh-CN/visual/html-fidelity.md) | [繁體中文（台灣）](../../zh-TW/visual/html-fidelity.md) | [日本語](../../ja-JP/visual/html-fidelity.md) | [한국어](../../ko-KR/visual/html-fidelity.md) | [Français](../../fr-FR/visual/html-fidelity.md) | [Deutsch](../../de-DE/visual/html-fidelity.md) | [Español](../../es-ES/visual/html-fidelity.md) | Português (Brasil) | [Русский](../../ru-RU/visual/html-fidelity.md)

As imagens de alta fidelidade são a referência visual; HTML é a implementação editável. As dimensões acima e abaixo da barra de ferramentas são medidas separadamente na [janela principal inicial congelada](../../../design/hifi/screens/main-light.png), sem dedução retroativa do CSS atual. A apresentação atual usa o [desenho claro com abas em cápsula](../../../design/hifi/screens/main-capsule-light.png) e o [desenho escuro](../../../design/hifi/screens/main-capsule-dark.png), exportados da [fonte editável](../../../design/hifi/sources/main.html); eles não substituem a medição independente. A barra de título é aceita conforme a revisão explícita em layout-scale.json. Os originais têm bordas suavizadas e glifos gerados; contagens de testes não comprovam identidade perfeita de pixels na página inteira.

<a name="xdzip-fixed-reproducible-conditions"></a>

## Condições fixas reproduzíveis

- A área de origem corresponde ao PNG de 1586×992; a janela mede 1548×940 na origem `(20,30)`. As verificações originais de âncoras continuam usando viewport de navegador de 1588×1000 com proporção de pixels do dispositivo 1. Recorte o topo em `(20,30)-(1568,167)` e o conteúdo em `(20,167)-(1568,970)` sem esticar o original.
- Os valores SHA-256 originais estão fixados em [header-reference.json](../../../design/html/header-reference.json) e [body-reference.json](../../../design/html/body-reference.json). Mudanças nos originais exigem nova medição, não substituição silenciosa.
- A fonte offline incluída [Xdzip Design Sans SC](../../../design/assets/fonts/README.md) é medida após `document.fonts.ready`. Pesos reais 400–750 são fixados para reduzir alterações de largura causadas por fontes alternativas da plataforma.
- A família de fonte original não pode ser determinada a partir de uma imagem gerada. Contornos de glifos e suavização do sistema ainda podem diferir; registre as diferenças separadamente e não alegue ter restaurado a fonte original.

<a name="xdzip-display-size-and-source-pixels"></a>

## Tamanho de exibição e pixels de origem

A exibição normal usa a [janela com largura de 1024 pixels lógicos](README.md), mapeando uniformemente coordenadas de origem por `1024 / 1548`. A janela principal aparece com 1024 × aproximadamente 621,81 pixels CSS; o original usa a mesma escala. Essa proporção é uma convenção explícita de design, não uma dedução sobre Retina ou DPI a partir de um PNG.

`display-spec.js` é a única fonte das dimensões das pranchetas e escalas de exibição. O desktop usa por padrão o tamanho normal e dispositivos móveis usam ajuste; a escala ajustada é limitada ao tamanho normal para evitar ampliação automática em telas largas. O modo de pixels originais usa `scale=1`, preservando dimensões originais e rolagem tanto no HTML quanto na imagem. A proporção de pixels do dispositivo afeta apenas a densidade de rasterização, não a largura lógica CSS.

Não ajuste original e HTML a seus contêineres de forma independente. As âncoras de cabeçalho e corpo continuam medidas em coordenadas de pixels de origem; a escala de exibição não altera espaçamentos nem tamanhos de fonte calibrados. A consistência de tamanho cobre separadamente limites reais de tela, tamanho natural do original, rolagem móvel, prévias independentes e saídas offline em arquivo único. Guarde evidências conforme o [método de verificação](../../../developers/qa/README.md).

<a name="xdzip-canvas-and-content-scrolling"></a>

## Prancheta e rolagem do conteúdo

O HTML incorporado à ferramenta de revisão é uma prancheta de tamanho fixo com `overflow:hidden` no documento raiz. Isso impede que barras de rolagem clássicas consumam as dimensões da área e acionem umas às outras. A página de revisão rola verticalmente; a área externa desloca horizontalmente apenas quando falta largura. `scrollbar-gutter:stable` reserva a largura da barra clássica para que uma altura de limiar não altere repetidamente a escala de ajuste; prévias independentes usam `auto`. Listas de arquivos, propriedades longas e janelas de operação mantêm rolagem nos contêineres de conteúdo; áreas de origem abertas diretamente sem incorporação mantêm a rolagem de página. Isso não altera as dimensões da área de origem, janela ou componentes.

Playwright usa `--hide-scrollbars` por padrão; portanto, matrizes anteriores de capturas e tamanhos não cobriam barras clássicas reais. As verificações de tamanho e prévia agora removem explicitamente esse argumento e conferem reserva da barra raiz, visibilidade das quatro bordas da janela, deslocamento horizontal externo e acesso ao último botão de propriedades longas. Uma área 1 px pequena demais e um caso negativo sem regras de incorporação reproduzem e detectam a disputa por espaço de barra. Mantenha capturas de reprodução e correção nos artefatos de execução conforme o [método de verificação](../../../developers/qa/README.md); isso não comprova a aparência das barras em todos os navegadores ou sistemas.

<a name="xdzip-header-corrections"></a>

## Correções do cabeçalho

| Área | Antes | Corrigido conforme o original |
| --- | --- | --- |
| Aba ativa | Início x299, largura 244 | x293, largura 246 |
| Aba inativa | x557, largura 244 | x542, largura 262 |
| Espaço entre abas | 14 px | 3 px; cerca de 1 px de incerteza é permitido para bordas suavizadas |
| Botões adicionar e fechar | Arranjos verticais separados com centros a 5 px | Mesma linha de abas, com cerca de 1 px de deslocamento óptico em relação ao original |
| Controles de fechar | Um botão e um span simples | Mesma estrutura de ícone/nome/fechamento e espaços explícitos |
| Ícone de configurações | Gráfico visível pequeno e deslocado para cima e esquerda | Centro `(1532,61)`; examine a arte real, não apenas o contêiner SVG |
| Borda inferior da aba ativa | Sobrepunha a linha da barra de título | Une-se à superfície branca da barra de ferramentas, com transição do ombro esquerdo definida separadamente |
| Pesquisa e extração | Limites horizontais e verticais deslocados | Compare com `(906,101,449,51)` e `(1368,101,183,51)`, respectivamente |
| Ícones à esquerda da barra | Layout definido por tamanhos e espaços genéricos de botões | Posicione e confira contra as âncoras gráficas originais |

Espaços e tipografia do cabeçalho são centralizados nas seções de barra de título, abas e barra de ferramentas de `hifi.css`, sem depender das dimensões genéricas de `.icon-button` ou de altura de linha herdada. Temas claro e escuro compartilham geometria. Decorações de título Windows/Linux continuam demonstrações no navegador; janelas nativas do produto recebem aceitação separada.

<a name="xdzip-content-below-the-toolbar"></a>

## Conteúdo abaixo da barra de ferramentas

O corpo inclui cartão do arquivo, árvore de diretórios, barra de caminho, cabeçalhos e linhas da tabela, seleção, prévia e barra de status inferior. Registre limites de colunas, alturas de linhas, preenchimentos, centros de ícones e traços visíveis e confira contra a [referência independente](../../../design/html/body-reference.json). As tolerâncias de geometria estrita e traço isolado de ícone são 2 px; contornos de texto são registrados apenas para revisão manual. O tamanho óptico da pasta é uma revisão explícita: o novo contorno foi ampliado uniformemente em cerca de 25,7 % nos recursos compartilhados. Cinco exemplos antigos de traço de pasta permanecem para comparação e são conferidos contra novos limites de design registrados independentemente; as demais âncoras originais não mudam.

A revisão do corpo usa recortes de conteúdo de mesmo tamanho e mantém comparações regionais de barra lateral, lista, prévia e status. Capturas antes/depois, commits e medições pertencem apenas aos artefatos do issue ou Actions correspondente, não à especificação como resultados de uma execução específica. A revisão móvel é por região e não deve alterar o layout de desktop para ocultar diferenças.

O original seleciona um item enquanto deixa Selecionar tudo vazio e informa economia de 26 % enquanto preenche a barra em cerca de 56 %; esses estados estáticos são contraditórios. O HTML mantém, respectivamente, seleção parcial `indeterminate` e preenchimento real de 26 % como exceções semânticas documentadas. Verificações específicas de prévia cobrem transbordamento normal e rolagem de conteúdo longo; verificações de interação existentes cobrem seleção, navegação, ordenação, pesquisa e ações de exemplo. Verificações geométricas não substituem a validação funcional.

Salve revisão do corpo, comparações regionais de mesmo tamanho e medições separadamente conforme o [método de verificação](../../../developers/qa/README.md). Preserve as explicações sobre fontes originais desconhecidas, bordas suaves geradas, iluminação e diferenças de suavização; essa evidência HTML não comprova calibração correspondente do produto Qt.

<a name="xdzip-checks-and-delivery"></a>

## Verificações e entrega

Execute:

```sh
python scripts/build-html-design.py
python scripts/check-html-header.py
python scripts/check-html-body.py
python scripts/check-html-preview.py
python scripts/check-html-references.py
python scripts/check-html-scale.py
python scripts/check-html-design.py
python scripts/build-html-header-review.py
python scripts/build-html-body-review.py
```

As verificações de cabeçalho usam âncoras originais independentes: limites/centros DOM desviam no máximo 1 px e limites de traços reais de ícones em regiões isoladas no máximo 2 px. O traço branco da seta suspensa tem tolerância de 1 px; o ombro direito da aba amostra a superfície branca e a curva originais com tolerância de canal único de 10/255 para suavização. Contornos de glifos são apenas medidos e registrados. Sobreposições e imagens de diferença incluem diferenças de suavização, iluminação e glifos; não converta a diferença total de pixels em similaridade inventada.

O arquivo offline de cabeçalho é `dist/xdzip-header-review.html` e o de corpo `dist/xdzip-body-review.html`. Ambos alternam entre original, antes, depois, sobreposição transparente e diferenças de pixels; a revisão do corpo acrescenta seleção de região móvel. A interface em arquivo único é `dist/xdzip-hifi-design.html`.

Execute essas verificações HTML e revisões offline localmente quando necessário; elas não fazem parte da CI padrão do produto desktop. Investigue falhas de âncora e examine sobreposições mesmo após aprovação geométrica. Verificações HTML da janela principal não substituem a calibração de outras páginas ou do produto Qt nem a aceitação nativa de desktop.

A densidade normal da janela principal e do menu segue o [padrão de interface](design-standard.md). As proporções de tamanho de fonte de menus e listas Qt devem ser verificadas juntas. Ao revisar HTML localmente, confira o zoom real da página conforme o [método de verificação](../../../developers/qa/README.md); corresponder às coordenadas antigas da imagem de origem não substitui essa verificação.
