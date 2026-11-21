# Especificações visuais do xdzip

[English](../../visual/README.md) | [简体中文](../../zh-CN/visual/README.md) | [繁體中文](../../zh-TW/visual/README.md) | [日本語](../../ja-JP/visual/README.md) | [한국어](../../ko-KR/visual/README.md) | [Français](../../fr-FR/visual/README.md) | [Deutsch](../../de-DE/visual/README.md) | [Español](../../es-ES/visual/README.md) | Português (Brasil) | [Русский](../../ru-RU/visual/README.md)

Entradas de especificação: [padrão de design de interface](design-standard.md), [fluxo obrigatório de design](design-workflow.md) e [contrato de densidade e unidades](../../../specs/visual/layout-scale.json). A reverificação das proporções da janela principal e dos menus é registrada pelo [método geral](../../../developers/qa/README.md).

As referências de projeto estão em [design/hifi](../../../design/hifi/README.md).

[tokens.json](../../../specs/visual/tokens.json) contém parâmetros de tema e implementação compartilhada; entradas visuais legíveis por máquina ficam em `specs/visual/`. Geometria e fontes da janela principal são geradas a partir de [qt-main-reference.json](../../../specs/visual/qt-main-reference.json), congelado independentemente com base nos originais de alta fidelidade, dimensionado por 1024/1548. A base anterior de largura 1280 foi convertida para densidade normal. Arredondamento, capturas reais e recursos não implementados são registrados separadamente pelo [método de verificação](../../../developers/qa/README.md); parâmetros de implementação não podem se tornar sua própria base de aceitação de design.

## Base de pesquisa e adaptação

1. [Interface oficial do Bandizip para Windows](https://www.bandisoft.com/bandizip/screenshots/): manter árvore de diretórios, tabela de arquivos, entrada clara de extração e barra de status; simplificar a barra grande e colorida em texto com ícones de linhas finas e destacar uma ação principal.
2. [Interface oficial do Bandizip para macOS](https://www.bandisoft.com/bandizip.mac/screenshots/): revisar a demonstração oficial de navegação de arquivos e o painel de criação de arquivos; manter o fluxo de navegar antes de escolher conteúdo e destino de extração.
3. [Instruções de extração do Bandizip](https://www.bandisoft.com/bandizip/help/how-to-unzip/): abrir um arquivo não o extrai automaticamente. As tarefas começam somente após o usuário confirmar destino, escopo e tratamento de conflitos.
4. [Interface oficial do Transmit](https://panic.com/transmit/): usar como referência a densidade de informação de gerenciadores de arquivos, navegação por caminho e separadores discretos.
5. [Renovação de design do Linear em 2026](https://linear.app/now/behind-the-latest-design-refresh): referenciar barras laterais mais discretas, navegação compacta, bordas suaves e fundos neutros que destacam o conteúdo central.

A pesquisa foi realizada em 2026-10-11 com capturas, demonstrações e artigos oficiais públicos; não se alega que representem a versão instalada mais recente de cada produto. Materiais de pesquisa de terceiros ficam em um espaço experimental separado e não são incluídos neste repositório.

## Sistema visual

Use fundos cinza quente, ações principais em cobalto e tema escuro grafite. Não substitua a hierarquia de informação por grandes gradientes ou fundos foscos. Listas comuns de arquivos mantêm estrutura plana clara; sombras são reservadas a sobreposições. A esquerda responde «Onde?», o centro «O que há aqui?» e a direita «O que está selecionado?».

Qt compartilham glifos e métricas de [Xdzip Design Sans SC](../../../design/assets/fonts/README.en.md); o Qt usa TTF da mesma fonte e fallback da plataforma para nomes de arquivos fora do subconjunto. O texto da lista principal tem 13, cabeçalhos e barra de ferramentas 12 e títulos de painéis 13 pixels lógicos, arredondados para pixels inteiros de fonte Qt. Nomes de arquivos mantêm nomes acessíveis completos; o truncamento visual usa reticências e dica ao passar o cursor com o nome completo. Ícones de arquivo compartilham um espaço de aproximadamente 31.75 pixels lógicos sem compensação de tamanho por tipo.

A janela principal inicial mede 1024 × 622 e o mínimo é 832 × 544 pixels lógicos. As medições de origem da barra lateral e pré-visualização são aproximadamente 193.16 / 257.98, arredondadas para os limites estruturais Qt 193 / 258. Linhas de diretório medem aproximadamente 43 e linhas de arquivo 44.65. Títulos de painel têm altura 46, cabeçalhos de tabela 34 e busca e botão principal aproximadamente 33.74. A árvore de diretórios B mantém separadamente altura 32 e espaço de ícone 16. Abaixo de largura 928 a pré-visualização é recolhida e aberta por botão em janela nativa independente. Configurações oferece tamanho de fonte 100%/125%/150%; o 200% histórico é lido como 150%. Fontes grandes usam layout real, rolagem e janela independente de detalhes em vez de escala global para ocultar transbordamentos. Arraste de painéis e persistência de largura não estão implementados.

## Interação da janela principal

- O Qt suporta sessões independentes de arquivos compactados, criando abas em cápsula ou janelas principais conforme o modo salvo de abertura; reabrir ativa uma sessão existente. As páginas mantêm caminho, seleção, ordenação e rolagem; tarefas continuam após o fechamento de páginas. Consulte o [contrato de sessões](../engineering/application-sessions.md).
- O botão principal é «Extrair tudo» sem seleção e «Extrair seleção» com seleção; o menu suspenso sempre oferece o outro escopo.
- A tabela de arquivos suporta ordenação, seleção múltipla, seleção por intervalo e navegação por teclado. Clique duplo em uma pasta a abre; em arquivo comum extrai uma cópia temporária e a abre com o sistema. O cartão de propriedades apresenta automaticamente uma pré-visualização de um arquivo compatível selecionado. Consulte o [contrato de pré-visualização](../engineering/content-preview.md) para escopo e segurança.
- A busca cobre recursivamente o diretório atual e seus descendentes; a busca na raiz cobre todo o conteúdo. Os resultados mostram caminhos completos dentro do arquivo compactado, enquanto listas comuns de diretório mostram os nomes finais.
- Detalhes podem ser fechados e reabertos. A pré-visualização exige leitura assíncrona e limites de recursos; itens não suportados exibem propriedades e entrada de extração. Descarte resultados tardios após o cancelamento da tarefa.
- A barra de status mostra quantidade selecionada e tamanho original, estado do índice e quantidade de entradas reabríveis do histórico de tarefas. A quantidade de tarefas ativas controla separadamente dicas e proteção de saída. Marque um índice incompleto como «Lendo» sem inventar uma contagem completa.

## Interação do painel de extração

O painel agrupa arquivo compactado, destino, escopo de seleção, política de conflito e comportamento de conclusão. A seleção de exemplo é 8.4 + 6.2 + 12.8 = 27.4 MB. A lista de arquivos é um resumo; listas maiores oferecem «Ver tudo».

O destino padrão é o último diretório. «Criar pasta com o nome do arquivo» atualiza imediatamente a prévia do caminho final para evitar aninhamento redundante. Mostre texto para falhas de permissão de caminho, espaço livre e leitura do arquivo; verificação é um estado separado e espaço não confirmado é «Desconhecido».

O padrão para conflitos é Perguntar; Ignorar, renomeação automática e Sobrescrever também estão disponíveis. Marque claramente as consequências de sobrescrever. Durante a execução mostre etapa real, número de arquivos processados, progresso em bytes e velocidade quando calculável; use progresso indeterminado quando os totais forem desconhecidos. Após cancelamento mostre «Parando» até o processo em segundo plano confirmar a parada.

## Estados dos componentes e acessibilidade

| Componente | Estados e feedback exigidos |
| --- | --- |
| Botão principal | Normal, cursor sobre, pressionado, foco de teclado, desabilitado e enviando; impedir início duplicado |
| Linha de arquivo | Normal, cursor sobre, selecionada, selecionada sem foco da janela e foco de teclado; a seleção altera mais que a cor do texto |
| Busca | Vazia, entrada, pesquisando, sem resultados e limpar; Escape retira o foco preservando o texto |
| Pré-visualização | Sem seleção, carregando, carregada, não suportada, erro e cancelada |
| Tarefa | Aguardando, em execução, aguardando senha ou decisão de conflito, parando, concluída, falha parcial e falha |
| Painel modal | Restrição de foco, Escape fecha painel não enviado e o foco retorna ao botão de origem |

Após implementar em Qt, verifique contraste WCAG AA do texto para as cores do tema. Aparência e saída do foco seguem o padrão de design atual sem desenhar anéis de foco adicionais. Forneça nomes e anúncios de estado para leitores de tela e respeite a configuração de movimento reduzido do sistema. Transições comuns sugeridas são 120–160 ms; não anime continuamente efeitos decorativos da lista de arquivos.

## Restrições de implementação Qt Quick

O Qt usa `Theme` compartilhado e dimensões geradas. `ArchiveTabs`, `ArchiveToolbar` e as seções de lista, grade e diretório em `SessionPage` compartilham o layout principal. `InspectorPanel`, `ExtractWindow`, `CreateWindow` e `TaskWindow` mantêm seus próprios limites de dados e layout. QML apresenta estado e intenção do usuário. O runtime Rust é responsável por indexação, leitura e gravação de arquivos compactados e validação de formatos.

macOS, Windows e Linux compartilham layout de conteúdo e modelos de interação; controles de janela, menus e atalhos seguem convenções de cada plataforma. A moldura macOS ilustrada não exige que outras plataformas copiem os botões de semáforo. Desfoque grande em tempo real não é exigência de implementação; uma máscara estática pode reproduzir a redução de destaque do fundo em capturas.

Revise a janela principal em claro ou escuro, junto a estados de tarefas, configurações e integração de desktop. Use o [registro de janelas](../../../specs/visual/window-registry.json) para localizar outros estados ou exceções de tamanho. Uma página de design pode mostrar o conteúdo de uma janela ou menu nativo. Decoração, ativação, escala e comportamento de menus reais exigem a plataforma correspondente.

## Revisão de janelas de plataforma

Os painéis de operação passam uniformemente a [janelas de plataforma independentes](../ux/platform-windows.md). O botão de fechar na área de conteúdo, painel arredondado e máscara das ilustrações antigas de extração são substituídos por barra de título independente e decoração de plataforma; cores, tipografia e estrutura do conteúdo permanecem. Ícones atuais de tipo de arquivo, alinhamento de listas e barras de título das três plataformas têm protótipo interativo como material complementar de revisão.

## Originais e biblioteca de assets compartilhados

Os [originais completos](../../../design/hifi/README.md) ficam em `design/hifi/screens/`, incluindo janela principal, confirmação de extração e oito painéis de estados de operação. Consulte o [contrato de estados](../ux/hifi-state-spec.md) para recursos e recuperação. Preserve os bytes originais; originais e recortes não devem ser alterados para acomodar a implementação.

A [biblioteca de assets compartilhados](../../../design/assets/README.md) reúne assets da janela principal e estados de operação em vez de separá-los por lote de entrega. SVG é a única fonte gráfica dos ícones transparentes de runtime, renderizados como PNGs de várias resoluções da mesma fonte no Qt. Quarenta e cinco recortes de comparação em pixels originais em `reference/` mantêm imagens, recortes e fundos originais. Um manifesto identifica seus propósitos para evitar que recortes com fundo sejam confundidos com camadas transparentes.

Todos os ícones transparentes compartilham uma tela 24×24. Ícones PDF, texto, código e Markdown compartilham centro e altura visível de 22 pixels, mantendo larguras de silhueta e estilos internos distintos. Pastas usam uma silhueta visível horizontal centralizada de 22×aproximadamente 18.86. Essa restrição de altura vale para ícones de runtime, não para recortes originais. PNGs 1×/2×/3× são 24×24, 48×48 e 72×72; escala não altera tamanho lógico. Qt usa aproximadamente 31.75×39.69 pixels lógicos após conversão equivalente. Remova ampliação, deslocamento e compensação de traço CSS por tipo.

A navegação estrutural da lista e os títulos de pré-visualização/detalhes à direita compartilham altura de barra, tamanho de fonte e centro vertical do texto. Qt usa `PaneHeader` e `Theme.mainPaneTitleSize`. Títulos permanecem acima do conteúdo rolável e fontes grandes aumentam juntas as alturas dos dois painéis. Medições reais são mantidas em artefatos de execução conforme o [método de verificação](../../../developers/qa/README.md).

Pré-visualizações fotográficas começam logo abaixo do título do painel, com topo alinhado ao cabeçalho da lista de arquivos e sem espaço superior extra. Quando dicas de busca aparecem, alinhe conforme o espaço realmente ocupado. Mantenha a altura existente das imagens e o espaçamento inferior. Reverificações de alinhamento de fotos cobrem duas imagens de exemplo, temas claro/escuro, três decorações de janela e mudanças de busca.

Cabeçalhos e valores de tamanho original e compactado da lista alinham-se à direita; cabeçalho e conteúdo de tipo alinham-se à esquerda, com preenchimento de conteúdo compartilhado por coluna. A reverificação registra bordas de layout de texto de todas as linhas visíveis.

Registre separadamente silhueta do ícone, dimensões do recorte em pixels originais e dimensões da tela de runtime. Pastas não precisam ser esticadas em formato de papel; SVGs não devem ser distorcidos para preencher espaços. Faça alterações visuais na fonte gráfica, revise-as na galeria de assets e sincronize ambas as implementações.

## Revisão da árvore de diretórios B (2026-10-12)

Três painéis formais definem expansão multinível, interação de nós e limites de navegação; consulte a [especificação da árvore](../ux/directory-tree.md). Vinte e seis recortes em pixels originais entram em `assets/reference` compartilhado; pontos de ramificação, ícones de estado preenchidos e indicadores segmentados usam SVG para dois temas e PNG em três resoluções. Linhas de conexão, fundos de estado, foco e texto são desenhados dinamicamente a partir dos [parâmetros de componentes](../../../specs/visual/directory-tree.json); eles não substituem automaticamente tokens globais. Os painéis são explicações visuais ampliadas; dimensões lógicas exatas e retângulos de pixels de origem são registrados separadamente. Qt integra diretórios recursivos, linhas de conexão e marcadores de ramo atual oculto; ícones e esqueletos específicos de limites continuam recursos de design e um painel escuro completo permanece pendente.

## Revisão da barra do design principal atual (2026-10-12)

As sugestões gerais anteriores de controles não regem mais a barra principal. As medidas atuais do original são altura de 51 pixels de origem, fonte de 18 e ícone de 32, convertidas por 1024/1548 para aproximadamente altura 33.74, fonte inteira 12 e tela de ícone 21.17. A altura da região da barra é 53. A conversão anterior de largura 1280 com 42.17/15/26.46 foi substituída por densidade normal; janelas comuns de operação mantêm seus mapeamentos registrados.

Ações comuns esquerda/direita e ações de busca/principal formam grupos. Fontes grandes podem aumentar a altura e a busca pode encolher para manter todas as ações da direita visíveis no tamanho mínimo da janela. Texto desabilitado não recebe opacidade do grupo; o botão principal tem fundo desabilitado próprio. O botão principal de extração mantém a aparência desabilitada do divisor e da seta; extração real e menus de escopo seguem a capacidade do runtime. Sem seleção, o escopo selecionado e a verificação prévia de metadados são desabilitados; a verificação de integridade segue a capacidade do arquivo. Registre a revalidação pelo [método geral](../../../developers/qa/README.md).

## Marca do aplicativo e ícones de plataforma

Os ícones de documentos de sistema para cada formato usam a [especificação de ícones de associação](file-association-icons.md). Essa família reutiliza a geometria da marca abaixo com rótulos de formato, acentos e simplificações para tamanhos pequenos distintos como assets de design para integração futura por formato.

A marca usa quatro faixas entrelaçadas e uma parte inferior aberta em U, com geometria mantida em `scripts/brand_assets.py`. Marca independente, ícone do aplicativo e mestre de documento ZIP ficam em `design/assets/brand/`. Os itens de interface `app-mark`, `archive` e `archive-grey` identificam o aplicativo, arquivo azul e arquivo cinza, todos gerados da mesma geometria de marca. O aplicativo usa exterior azul arredondado e documentos usam papel com canto dobrado: a identidade é compartilhada, enquanto as molduras têm funções distintas.

A marca atual mantém proporção esbelta de 2:3 e não é esticada para a largura da marca antiga. Sua base usa curva de curvatura contínua com simetria rotacional nos quatro cantos, aumentando suavemente a curvatura de zero nas bordas retas até o ponto médio diagonal, evitando um centro chanfrado. Iluminação e base compartilham um caminho; não sobreponha traços de raios diferentes ou linhas de brilho abertas. Insígnias de documento usam a mesma definição de contorno.

Esta é uma revisão explícita de marca nova, não uma reprodução indistinguível do ícone de zíper ZIP dos originais antigos de alta fidelidade. Dez originais completos, 45 referências de ícones em pixels originais e três fotos de exemplo mantêm os bytes originais. O novo `brand/identity-reference.png` gerado é referência de revisão da direção atual da marca, não asset de runtime. O manifesto principal registra a cobertura das referências e identifica explicitamente referências da marca antiga como papéis históricos de design.

Ícones pequenos de interface mantêm telas 24×24. Ícones de plataforma exportam PNGs transparentes de 16/24/32/48/64/128/256/512/1024 pixels. Tamanhos 16/24/32 simplificam texto ilegível e iluminação de imagem grande, preservando as faixas entrelaçadas e a estrutura reconhecível em U. Ícones do aplicativo e documento ZIP oferecem cada um ICO com sete tamanhos e ICNS com onze representações de densidade. Não amplie indefinidamente um ícone de interface de 24 unidades para uso do sistema nem reduza sombras grandes até virarem ruído em tamanho pequeno.

Ícones de aplicativo do sistema são integrados às janelas Qt, bundles de aplicativos macOS, recursos de executáveis Windows e lançadores de desktop/diretórios hicolor Linux. Formatos de plataforma do documento ZIP são usados por instaladores e associações; solicitações de abertura do sistema seguem para navegação real por roteamento unificado. Consulte o [contrato de instalação e associação](../engineering/installers-associations.md). Registre apenas candidatos de abertura sem mudar os padrões do usuário. Registre a verificação de plataforma pelo [método geral](../../../developers/qa/README.md).

Gráficos ausentes de tema, sucesso, cancelamento, conflito e progresso são completados a partir de referências em pixels originais como assets semânticos distintos. A ilustração grande de cancelamento usa símbolo quadrado de parada; o tema de sistema mantém a estrutura completa da janela. Qt usa IntentRadioButton para direcionar ações de mouse, teclado e acessibilidade ao mesmo modelo de seleção; assets tratam apenas da aparência. Registre separadamente verificações de geração, exibição na plataforma de destino e diferenças dos originais; aceitação existente não substitui os resultados desta revisão.

## Especificação de controles interativos

[controls.json](../../../specs/visual/controls.json) define colunas de menu, alturas de linha, raios, cores, foco e tempos de dicas; [directory-tree.json](../../../specs/visual/directory-tree.json) define hierarquia de diretórios. Ambos geram QML Theme; verificações evitam duplicar manualmente dimensões e temas. As proporções de pixels lógicos e de origem do design principal vêm de `layout-scale.json`. Listas de recursos e contratos de entrada estão na [descrição dos componentes da janela principal](../ux/main-window-components.md); renderização e comportamento reais são registrados nas issues e artefatos correspondentes conforme o [método de verificação](../../../developers/qa/README.md).
