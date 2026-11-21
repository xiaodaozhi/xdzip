# Funcionalidade, componentes e contrato de interação da janela principal

[English](../../ux/main-window-components.md) | [简体中文](../../zh-CN/ux/main-window-components.md) | [繁體中文](../../zh-TW/ux/main-window-components.md) | [日本語](../../ja-JP/ux/main-window-components.md) | [한국어](../../ko-KR/ux/main-window-components.md) | [Français](../../fr-FR/ux/main-window-components.md) | [Deutsch](../../de-DE/ux/main-window-components.md) | [Español](../../es-ES/ux/main-window-components.md) | Português (Brasil) | [Русский](../../ru-RU/ux/main-window-components.md)

Os controles da janela principal fornecem aparência e comportamento, com estados explícitos normal, passagem do cursor, pressionado, foco do teclado, desativado, selecionado, ocupado e erro. HTML usa arquivos de exemplo para verificar a interação; Qt habilita ações conforme as capacidades reais de navegação, criação, extração, [verificação de integridade](../engineering/archive-checking.md) e [prévia automática de conteúdo](../engineering/content-preview.md). Exemplos HTML não representam resultados reais em segundo plano.

Nomes de arquivos, diretórios e arquivos compactados, bem como caminhos, usam preferencialmente a fonte padrão do sistema na inicialização, com fallback da plataforma para glifos ausentes. Textos fixos da interface continuam usando o subconjunto incluído. A árvore de diretórios e as prévias de arraste são medidas com suas respectivas fontes; consulte o [padrão visual](../visual/design-standard.md#界面字体与用户数据).

## Recursos e componentes

| Área | Componentes compartilhados / compostos | Interação e capacidade |
| --- | --- | --- |
| Barra de título do sistema | `Browser`, `ArchiveTabs`, moldura da plataforma | macOS usa os botões nativos. Windows usa botões integrados, redimensionamento nativo e decoração DWM. No Ubuntu X11, botões integrados e cantos arredondados no estado normal seguem o design X11. Consulte [janelas de plataforma](platform-windows.md) |
| Abas de arquivo | ArchiveTabs, ArchiveTab, botões de ícone | Novas abas por padrão, novas janelas opcionais; abas inativas com peso normal, que encolhem e depois rolam horizontalmente com entradas de rolagem e lista de abas. Arraste dentro da janela e Ctrl+Shift+Page Up/Down reordenam; fechar mantém tarefas em segundo plano |
| Barra de ferramentas | `ArchiveToolbar`, `ArchiveAction`, `CommandMenu` | Abrir, Novo, Verificar, Mais e o botão dividido de extração usam o mesmo despacho de comandos dos menus e do teclado. A verificação prévia está disponível no menu de extração para seleções elegíveis |
| Busca | SearchField, ação de limpar, indicação de escopo | Não enviar durante composição IME; buscar no diretório atual; limpar não altera o diretório; atalho coloca o foco |
| Resumo do arquivo | ArchiveSummary, selo de formato, barra de estatísticas | Distinguir pronto, lendo, vazio e falha de leitura; valores desconhecidos não são zero |
| Navegação de diretórios | DirectoryTree, DirectoryRow, Disclosure, linhas de conexão | Expansão multinível, espaços vazios de seta em folhas, diretório atual e foco independentes, caminhos completos acessíveis, marcador de ancestral recolhido e ações de contexto do diretório |
| Cabeçalho da área de arquivos | PaneHeader, Breadcrumb, ViewToggle | Navegação hierárquica do caminho, troca de lista/grade e menu de ordenação; altura e centro vertical compartilhados |
| Lista e grade | EntryView, SelectableEntry, caixa de seleção, SortHeader | Seleção única/múltipla/por intervalo, selecionar tudo, limpar, inverter e teclas de seta. Clique duplo/Enter entra em diretórios ou extrai temporariamente arquivos comuns e abre com a associação do sistema; propriedades continuam disponíveis no menu de contexto ou detalhes |
| Menu de contexto de arquivo | ContextMenu, CommandMenuItem, Submenu | Abrir aplica-se apenas ao item clicado com o botão direito, como no clique duplo; mantém a seleção múltipla existente. Clique direito em item não selecionado primeiro seleciona apenas ele. Habilitar extração, propriedades, cópia de nome/caminho e navegação de diretório conforme a seleção |
| Menu de contexto de área vazia | ContextMenu, RadioMenuItem, CheckboxMenuItem | Seleção, visualização e ordenação da lista atual e painel de detalhes; não limpar arbitrariamente a seleção |
| Menu de contexto do cabeçalho | Menu de ordenação | Campo, direção e restauração da ordem do arquivo; ordenar todos os resultados da consulta, não apenas os visíveis |
| Propriedades / prévia | `InspectorPanel` na página ou janela independente de detalhes | Mostra estado do arquivo, seleção ou arquivo compactado. O conteúdo é lido quando solicitado. Abaixo de 928 pixels lógicos, o painel embutido recolhe e a ação de detalhes abre janela separada |
| Estado e tarefas | Barra de status, botão de tarefas e `CommandMenu` | Mostra quantidade selecionada e tamanhos conhecidos. O menu de tarefas usa histórico de toda a aplicação. Tarefas ativas têm contagem separada e continuam após o fechamento da janela de progresso |
| Dicas e janelas de operação | Tooltip, Notice, Toast, janelas independentes de plataforma | Nomes completos e motivos de desativação; Escape, retorno do foco, cancelamento e fechamento seguem a semântica de cada ação |

Lista e grade compartilham um estado vazio. Uma busca enviada sem resultados sugere outro termo. Sem busca, um arquivo sem registros é identificado como vazio; caso contrário, a mensagem descreve apenas o conteúdo visível da pasta atual e não presume que um índice limitado de diretórios seja completo. Leitura e erros mantêm estados reais; texto de busca não enviado não muda mensagens do resultado atual. A orientação antes da abertura de um arquivo não enumera uma lista desatualizada de formatos.

## Comandos e menus unificados

Botões, atalhos, menus de contexto e listas suspensas invocam os mesmos comandos e verificam novamente a capacidade atual antes da execução. Menus fixam objeto e escopo de seleção ao abrir; feche-os quando o arquivo ou a consulta mudar ou o objeto desaparecer, impedindo que o comando de uma entrada antiga seja aplicado a um novo arquivo.

Os menus compartilham quatro colunas para espaço de ícone/marca, nome, atalho e seta de submenu, com separadores entre grupos de ações. Inclua apenas ações relevantes ao contexto: diretórios oferecem entrada e expansão, arquivos oferecem detalhes e seleções múltiplas mantêm o escopo de lote. Ações que não podem ser executadas permanecem desativadas, com motivos disponíveis apenas em descrições acessíveis; não mostre sublinhas explicativas ou dicas nem substitua a desativação por “em breve” ao clicar.

Quando o tamanho da fonte muda, a altura do rótulo do menu segue as métricas reais e permanece centralizada verticalmente na linha. A primeira apresentação deve respeitar um erro de centro de no máximo meio pixel lógico sem depender de uma passagem posterior de layout para corrigir um espaço antigo de texto.

Linhas normais, de caixa, de opção e de submenu e separadores são componentes compartilhados. Ordenação e visualização usam itens de opção para o estado atual; o painel de detalhes usa item de caixa. Ações perigosas nunca são ativadas automaticamente por padrão; clicar com o botão direito e abrir um menu não executam ação alguma.

## Entrada e ciclo de vida dos menus

- Baixo/cima movem entre itens disponíveis; Home/End vão ao primeiro/último; Enter/Espaço ativam, direita entra em submenu e esquerda volta ao pai.
- Escape fecha um nível por vez; fechar a raiz retorna o foco ao controle acionador ou à entrada original. Tab fecha o menu e continua a ordem normal de foco da página. A busca pelo primeiro caractere do nome é suportada; composição IME não aciona a busca do menu.
- A passagem do cursor destaca sem executar. Submenus abrem após um atraso e mantêm o item pai durante a entrada. Mouse e teclado podem se suceder sem múltiplos destaques ou menus raiz persistentes.
- Cliques externos fecham sem roubar o foco do alvo externo. Rolagem da área principal, redimensionamento da janela, entrada em janelas de operação ou troca de arquivo fecham menus; rolar o próprio menu não fecha. Controles compartilhados HTML registram as posições reais de rolagem dos contêineres existentes ao abrir; eventos concluídos antes da abertura, mas entregues tarde, não fecham o menu, enquanto mudanças posteriores de posição fecham.
- Evite bordas inferior e direita da área visível, abrindo à esquerda ou para cima quando necessário. Posicione em coordenadas do host, distinguindo dimensões lógicas, pixels originais de origem, DPR e escala de revisão.
- Shift+F10/tecla Menu abre o menu de contexto da entrada atual. Toque prolongado e clique direito compartilham contexto e não destroem a seleção múltipla como um clique simples faria.

## Estados de controle e acessibilidade

| Estado | Aparência | Comportamento |
| --- | --- | --- |
| Normal | Preservar borda, raio, ícone e linha de base do original aprovado | Mudanças de estado não alteram dimensões do controle |
| Cursor sobre | Preenchimento claro; botão principal usa cor principal mais intensa | Dicas atrasadas não cobrem alvos; toque não mantém hover preso |
| Pressionado | Preenchimento mais distinto que hover | Sem movimento de texto ou ícone; pressionamento cancelado não executa |
| Foco | Sem contorno, borda grossa ou brilho extra | Manter ordem Tab e ativação por teclado; clique externo, Escape e Enter deixam o foco do controle |
| Desativado | Cores explícitas de texto/gráfico desativados mantendo contornos | Sem resposta a clique ou atalho; descrição acessível explica o motivo |
| Selecionado | Fundo ou marca indica claramente o estado | Sincronizar `aria-selected` / `aria-checked` / Qt Accessible |
| Ocupado | Estado e escopo desativado seguem a tarefa | Impedir envio duplicado; permitir navegação durante trabalho em segundo plano |
| Erro | Texto adjacente explica problema e recuperação | Cor não é a única informação e sucesso não é inventado |

Ícones compartilhados vêm de `design/assets`. Controles implementam texto, marcas, fundos, linhas de conexão e posicionamento de menus; recortes em pixels originais servem para comparação. HTML mantém coordenadas de origem do design principal e converte primeiro novos parâmetros de controle de pixels lógicos; Qt usa a mesma especificação lógica.

## Aceitação

A contagem e os botões de ação à direita da área de arquivos formam um grupo de largura fixa. Lista, grade, ordenação e entradas visíveis de detalhes permanecem em 28×28 pixels lógicos com espaçamento 6.4; aumentar a janela dá espaço apenas ao caminho de navegação. O menu Mais da barra contém somente Configurações; ordem original, verificação prévia de metadados de extração, selecionar tudo e detalhes usam entradas dedicadas existentes.

Páginas inicial e de arquivo compartilham barra de status de no mínimo 32 pixels lógicos conforme statusbarHeight em layout-scale.json. O aumento de fonte de acessibilidade usa a altura real do botão de tarefas e margens verticais iguais, sem mudança de altura ao trocar de página. Ambos usam task-list em um espaço de ícone lógico centralizado de 16×16 definido por main.interaction.taskIconSize, sem mudar com página ou escala de fonte.

A barra de status do arquivo mostra a quantidade selecionada; para seleção não vazia, acrescenta separador e tamanho apenas quando o total é conhecido. Se um diretório virtual ou arquivo de tamanho desconhecido tornar o total desconhecido, mostre apenas a quantidade sem separador extra ou `—`. Um tamanho conhecido de zero bytes continua exibido; o texto inicial não muda.

A borda direita da caixa da lista permanece a 12 pixels lógicos do espaço de ícone; ícone, nome e cabeçalho Nome ficam alinhados. Fontes de acessibilidade mudam métricas reais do texto sem ampliar esse intervalo. Caixas das linhas e Selecionar tudo do cabeçalho mantêm o modelo de seleção existente.

As cinco colunas podem ser redimensionadas arrastando a borda direita, com área de toque de 8 pixels lógicos; clique duplo restaura o padrão da coluna. Nome preenche o espaço restante por padrão; as demais começam pelos valores do design original. As duas colunas de tamanho crescem por padrão para acomodar rótulos medidos e setas de ordenação sem corte. Larguras do usuário são compartilhadas entre abas e janelas durante a execução e redefinidas ao sair; larguras lógicas ficam na memória, não nas configurações nem no DPR. O mínimo de Nome é 160 e das demais colunas 64, aumentando proporcionalmente para fontes de acessibilidade; o excesso rola horizontalmente. Cabeçalhos e conteúdo compartilham uma fonte de largura, e o arraste não aciona ordenação. Marcadores crescente/decrescente precedem os rótulos Tamanho original e Tamanho compactado alinhados à direita e seguem os demais rótulos, sempre a 6 pixels lógicos do texto exibido. Colunas de ordenação ativas reservam espaço de seta no lado apropriado; ao estreitar manualmente, os rótulos são abreviados primeiro e as setas não podem sobrepor texto ou colunas vizinhas. Cabeçalhos numéricos permanecem alinhados à direita com o conteúdo.

Ícones de tipo de arquivo são gerados para seletores Qt e HTML a partir da [tabela de indicação de extensão](../../../specs/visual/file-types.json), cobrindo documentos de escritório, planilhas, apresentações, áudio, vídeo, arquivos compactados, instaladores, fontes e imagens de disco. Extensões não diferenciam maiúsculas e tipos desconhecidos usam ícone genérico de documento. Isso descreve apenas aparência, não conteúdo confiável nem suporte de navegação, criação ou extração do formato.

Use mouse e teclado reais para verificar entradas, desvio das bordas de menus, ida e volta em submenus, retenção de seleção, restauração de foco, proteção de itens desativados, isolamento de abas, troca de visualização e hierarquia de diretórios. Capture controles reais em temas claro/escuro, decorações de três plataformas, fontes normais/grandes e escalas. O alinhamento inicial existente de título, barra de ferramentas, lista e prévia não deve regredir por causa do trabalho nos menus.

Registre cobertura funcional, capturas reais e capacidades não integradas conforme o [método geral de verificação](../../../developers/qa/README.md), com conclusões nas issues correspondentes e evidências em build/ ou artefatos CI. Pranchas estáticas, comportamento do navegador e verificação Qt em execução são descritos separadamente e não substituem uns aos outros.

Menus Qt usam explicitamente `Popup.Item` com delegados compartilhados para que AppKit não ignore o estilo personalizado; janelas de operação permanecem independentes de nível superior. `CommandTooltip` usa parâmetros compartilhados de dica e texto simples para que nomes de arquivos não sejam interpretados como texto rico; oculte dicas quando menus receberem foco. Cores JSON compartilhadas de oito dígitos usam CSS RGBA; o gerador QML converte para QColor ARGB. Verifique cor e opacidade juntas.

A árvore de diretórios e os estados de cursor sobre Abrir, Novo, Verificar, Mais e Extrair da barra principal não têm dicas flutuantes. Ações de menu desativadas também não têm dica visível; outros botões usam dicas compartilhadas com atraso. Raiz e subdiretórios comuns listam apenas arquivos e pastas diretos, enquanto a busca cobre recursivamente o escopo atual. Nomes comuns mostram o último componente; a busca mostra caminhos completos. Hover na lista cobre a linha inteira; grades mantêm margem horizontal fixa e caixas ao passar o cursor ou selecionar. Menus usam sombras suaves em todos os lados e intervalo de 4 pixels lógicos dos botões que os invocam. Preserve evidências separadas de interação e escala conforme o [método geral](../../../developers/qa/README.md).

Todos os modos de ordenação mantêm diretórios antes dos arquivos. Ordem crescente e decrescente aplica-se dentro de cada grupo. A ordenação por tipo agrupa arquivos sem extensão separadamente dos diretórios e mantém ordem estável de identidade para tipos iguais. Ordene a consulta completa antes de dividi-la em lotes internos de transferência. Arquivos HTML sem propriedade folder são tratados como false; não converta undefined em NaN para comparação.

## Seleção, tarefas e dicas

Selecionar tudo mantém o foco dos detalhes no último item da lista atual, com metadados iguais aos da seleção individual. Limpar a seleção oculta quatro propriedades e “Extrair este arquivo”; esse botão usa estados compartilhados de hover, pressionado e foco. Selecionar tudo ainda se aplica à consulta atual completa sem mudar contratos de diretório ou identidade física.

“Tarefas N” na parte inferior mostra a contagem do histórico mantido; a contagem ativa é usada apenas para dicas e lógica de saída. Cada entrada do menu de tarefas mostra criação/extração e número à esquerda e cor de estado à direita; ativação por mouse ou teclado reabre a mesma tarefa. Menus de tarefas compartilham densidade, tamanho de fonte e limites de rolagem com outros menus. Sem seleção, ações de escopo selecionado e verificação prévia no menu de escopo de extração ficam desativadas sem linhas extras de dica.

Dicas flutuantes aparecem apenas após o ponteiro permanecer continuamente parado no mesmo controle por 1200 ms. Movimento, saída, pressionamento ou abertura de menu cancelam a exibição e temporizadores antigos; mover entre controles exige nova espera completa, não um despertar rápido da dica anterior. Prefira acima do controle; use abaixo se não houver espaço superior suficiente para impedir que limites da borda empurrem a dica sobre o controle e causem oscilação de hover. O CommandTooltip compartilhado do Qt usa temporizador explícito de imobilidade, sincronizado com data-tooltip do HTML. Nomes completos da lista não usam title nativo do navegador para contornar essa política. O tempo das dicas de menus e bandeja nativos é controlado pelo sistema e não se afirma que seja controlável por QML. O tempo não escala com fonte, DPR ou densidade; a geometria visual permanece igual.

Abas de arquivo também mostram nomes completos como texto simples na dica compartilhada com atraso de imobilidade; pressionar, reordenar ou abrir menu ou lista de abas a oculta. Entradas globais de tarefas primeiro fecham o menu e depois abrem apenas a janela da tarefa, sem trocar abas de arquivo ou reabrir páginas de arquivo fechadas.

A página inicial usa corpo rolável fixo no topo e centralizado horizontalmente. Erros e avisos aparecem abaixo das dicas de formato em texto vermelho e ícones semânticos sem deslocar o conteúdo superior. Fechar uma janela principal residente na bandeja limpa suas abas; a restauração comum mostra apenas a página inicial. Minimizar mantém abas; tarefas em segundo plano continuam acessíveis globalmente.

Consulte [arrastar e soltar arquivos e modificação](../engineering/archive-transfers.md) para depósitos bidirecionais na lista, confirmação de adição ao diretório atual e cópia externa em três plataformas.

Movimentos dentro do arquivo usam arraste interno. Destinos de pasta na lista/grade e na árvore usam o preenchimento de seleção existente; o retorno imediato reutiliza a geometria de dica, com no máximo quatro linhas e limitado à área cliente atual. Diretórios selecionados, seus descendentes, posições originais, busca e destinos criptografados ou divididos têm explicações explícitas de rejeição sem nova janela de confirmação. Conflitos de nome usam a janela de conflito de arquivos e tarefas de movimento têm nome distinto. Consulte o [design de movimento dentro do arquivo](../../../design/hifi/sources/archive-move.html) e o [contrato de arrastar e soltar](../engineering/archive-transfers.md) para transações e limites.
