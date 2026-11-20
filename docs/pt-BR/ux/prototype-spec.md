<a name="xdzip-v01-interaction-prototype-and-feature-mapping"></a>

# Protótipo de interação v0.1 e mapeamento de recursos

[English](../../ux/prototype-spec.md) | [简体中文](../../zh-CN/ux/prototype-spec.md) | [繁體中文](../../zh-TW/ux/prototype-spec.md) | [日本語](../../ja-JP/ux/prototype-spec.md) | [한국어](../../ko-KR/ux/prototype-spec.md) | [Français](../../fr-FR/ux/prototype-spec.md) | [Deutsch](../../de-DE/ux/prototype-spec.md) | [Español](../../es-ES/ux/prototype-spec.md) | Português (Brasil) | [Русский](../../ru-RU/ux/prototype-spec.md)

Estado: rascunho interativo de validação de design. As fontes estão em [design/html](../../../design/html/README.md), com [index.html](../../../design/html/index.html) como entrada. O rascunho do Figma permanece uma exploração histórica; o refinamento atual de recursos e estados continua no repositório.

<a name="xdzip-covered-flows"></a>

## Fluxos cobertos

| Estado | Entrada e saída | Requisitos / aceitação |
| --- | --- | --- |
| B01 Navegar | Abrir arquivo de exemplo; entrar em diretório com clique duplo ou Enter e voltar pelo caminho de navegação | P01/P02/P08, AC-01/02/08 |
| B02 Pesquisar e selecionar | Caixa de seleção, seleção única, múltipla com ⌘/Ctrl, intervalo com Shift e selecionar tudo; pesquisa cobre diretório atual e descendentes ou todo o arquivo na raiz | P02/P03, AC-02/03 |
| E01 Confirmar extração | Botão principal ou menu de escopo; destino, diretório de mesmo nome, escopo e política de conflito editáveis | P03/P07, AC-03/07 |
| E02 Conflito | Entrar ao haver colisão sob a política de perguntar; continuar após ignorar, renomear automaticamente ou sobrescrever | P03/P07, AC-03/07 |
| T01 Tarefa | Início → preparação → processamento → verificação → resultado; retornar à navegação ou solicitar parada | P06/P15, AC-06/14 |
| R01 Resultado | Sucesso, falha parcial, cancelamento, alteração da fonte; tentar novamente itens inacabados após nova confirmação | P03/P04/P15, AC-03/04/14 |
| C01 Criar | Adicionar/remover entradas, formato/nível/volumes/senha e local de salvamento; confirmar destinos existentes primeiro | P04/P05, AC-04/05 |
| S01 Configurações | Tema, tamanho de fonte, densidade, idioma e histórico recente; fechar retorna à navegação | P08/P11, AC-08/11 |
| X01 Exceções | Arquivo vazio, nenhum resultado, carregamento, corrupção, recursos sem suporte, permissões e falha de tarefa | P05/P07/P15, AC-05/07/14 |

As «anotações de design» no cabeçalho do protótipo apresentam esses IDs e as regras atuais de estado. Resultados de permissão e espaço dependem do cenário; executar o protótipo com sucesso não aprova a aceitação técnica P09/P10/P12.

<a name="xdzip-interaction-decisions"></a>

## Decisões de interação

<a name="xdzip-selection-and-search"></a>

### Seleção e pesquisa

- Cada seleção corresponde a um ID estável de entrada, não ao número de uma linha ordenada. Alterar consulta ou diretório limpa a seleção para evitar mal-entendidos com seleção oculta.
- Selecionar tudo na lista atual e extrair o arquivo inteiro são conceitos distintos. Marcar uma pasta expande para arquivos descendentes sem duplicatas.
- Não envie consultas durante a composição IME; atualize após compositionend. Verificações de eventos simulados do navegador não substituem a aceitação IME nativa em macOS/Windows/Linux.
- Enter ou clique duplo em arquivo comum extrai uma cópia temporária e a abre pela associação do sistema; em diretório, entra nele. A entrada de propriedades permanece. HTML demonstra somente progresso e ocultação automática; não grava arquivos nem chama o sistema. Veja o [contrato de prévia](../engineering/content-preview.md) para a prévia automática de conteúdo Qt.

<a name="xdzip-extraction-and-conflicts"></a>

### Extração e conflitos

- Exibir o caminho final em tempo real; acrescentar um diretório de mesmo nome apenas uma vez. Caminhos de exemplo vazios ou comprovadamente sem permissão de escrita bloqueiam o início; repetir a pré-verificação após alterar o destino.
- Perguntar por padrão em conflitos; ignorar, renomear automaticamente e sobrescrever estão disponíveis. O texto próximo explica as consequências de sobrescrever.
- O foco inicial do painel de conflito é Ignorar; sobrescrever não é a ação padrão de Enter. Aplicar aos conflitos seguintes limita-se à tarefa; os dados de exemplo criam apenas um conflito.
- Renomeações aparecem no resumo final e nos detalhes dos arquivos. Itens ignorados não são falhas; arquivos de exemplo preexistentes não contam como gravações concluídas.

<a name="xdzip-tasks-and-results"></a>

### Tarefas e resultados

- Retornar à navegação não cancela a tarefa; a barra de status pode reabri-la.
- Solicitar cancelamento → parando, com novo cancelamento desativado → resumo de cancelamento. A extração mantém itens publicados; a criação de arquivo compactado não publica resultado intermediário.
- Disco cheio interrompe a tarefa e mantém sucessos. Tentar novamente seleciona apenas itens com falha ou inacabados e retorna à confirmação do plano.
- Mudanças na fonte exigem reabrir o arquivo compactado, sem reutilizar silenciosamente o índice antigo.
- Uma tarefa de criação conta um arquivo final; o cancelamento mostra zero arquivos criados e preserva claramente o antigo.

<a name="xdzip-archive-creation-and-capabilities"></a>

### Criação de arquivos e capacidades

O protótipo ilustra níveis ZIP, empacotamento TAR e opções de compressão GZIP. Ele verifica campos básicos de entrada, nome e destino e altera a extensão conforme o formato. As escolhas atuais do produto usam o [registro integrado de formatos](../engineering/format-registry.md), incluindo os perfis de criação [ISO](../engineering/iso-creation.md) e [SFX no Windows](../engineering/sfx-creation.md). Veja a [interação de senha](password-extraction.md) e o [contrato de volumes](../engineering/archive-opening.md) para as opções admitidas.

<a name="xdzip-visuals-and-adaptation"></a>

### Visual e adaptação

Manter temas cinza quente, azul cobalto e grafite. A altura normal da linha é 40 e a compacta 32. O destaque de texto no tema escuro é independente do preenchimento dos botões para preservar a legibilidade de texto pequeno. Em desktop estreito, detalhes se tornam uma sobreposição que pode ser fechada. A revisão móvel recolhe a barra lateral e colunas secundárias; painéis de operação se adaptam a telas estreitas e rolam.

Interfaces de operação são janelas independentes segundo a [especificação de janelas de plataforma](platform-windows.md). HTML as simula com dialog: janelas de decisão restringem o foco; progresso, resultados e configurações não são modais; fechar tenta restaurar o foco. Fechar uma janela de conflito cancela a tarefa em vez de ignorar silenciosamente; fechar progresso apenas oculta detalhes. Qt usa janelas reais independentes de nível superior.

<a name="xdzip-theme-selection"></a>

### Seleção de tema

A página de configurações organiza três botões de opção nativos, sistema, claro e escuro, usando `fieldset#setting-theme` e a legenda «Appearance theme». Cada cartão de tema usa o recurso compartilhado `theme-system`, `theme-light` ou `theme-dark` e associa seu rótulo e entrada. Os controles reais participam de cliques no cartão, alternância pelo teclado e foco. A seleção usa a marca de opção compartilhada e sincroniza borda e prévia do cartão, em vez de substituí-la por uma captura não interativa.

Isso descreve o comportamento de validação de design HTML. Configurações atuais ficam em memória e seguir o sistema só é resolvido na seleção. Mudanças do tema do sistema durante a execução, persistência entre inicializações e gravação imediata com tratamento de falhas nas configurações Qt do produto exigem verificação separada.

<a name="xdzip-verification-and-remaining-boundaries"></a>

## Verificação e limites restantes

O script de verificação no navegador é scripts/check-html-design.py. Ele verifica cliques reais, estatísticas, condições desativadas, acoplamento de formatos, recuperação, transbordamento móvel e operação offline em arquivo único. Relatórios acompanham o pacote exportado do protótipo; as verificações de navegador usam somente Chromium.

A entrega atual da janela principal inclui duas sessões independentes de arquivos de exemplo, lista/grade, ordenação por campo, árvore recursiva, caminhos hierárquicos, menus de contexto/suspensos/submenus e estados de botão. Veja o [contrato de componentes e interação](main-window-components.md) e o [método de verificação](../../../developers/qa/README.md). `scripts/check-html-controls.py` verifica isso com entrada real no navegador; a [página de revisão de componentes](../../../design/html/components.html) usa a mesma implementação da interface principal.

HTML verifica apenas estados de exemplo; não comprova agendamento real de tarefas, autorização, corridas de caminho, CRC ou acessibilidade da plataforma. Qt integra criação/extração reais, arrastar e soltar, seletores do sistema, configurações persistentes e sessões independentes em abas opcionais ou várias janelas. O escopo real de vários formatos, da indexação de um milhão de entradas em ZIP comum de arquivo único e da prévia limitada está, respectivamente, em [mecanismos de formato](../engineering/native-codecs.md), [indexação grande](../engineering/large-index.md) e [prévia de conteúdo](../engineering/content-preview.md); a aceitação é registrada nos issues correspondentes. As configurações HTML mantêm estado de demonstração apenas na página, não entre inicializações, e não comprovam acompanhamento nativo do tema do sistema nem aceitação de acessibilidade.

Esses limites não alteram os recursos pretendidos no design do software. «Concluído» no protótipo descreve apenas um estado de tarefa de demonstração e não comprova entrega de compressão ou extração reais.

<a name="xdzip-current-directory-queries-and-card-revisions"></a>

## Consultas do diretório atual e revisões de cartões

A entrada raiz é «Root» e lista somente arquivos e pastas diretos. A pesquisa encontra descendentes no escopo atual e mostra caminhos completos dentro do arquivo compactado. Listas e grades comuns mostram apenas o nome final; cópia e navegação preservam caminhos completos. O lado direito do caminho de navegação mostra a contagem de linhas atuais. O título da árvore é «Archive», centralizado verticalmente com o caminho; passar sobre a árvore não mostra dica flutuante.

O destaque ao passar cobre a linha inteira. Grades têm cerca de 16 pixels lógicos de preenchimento horizontal; uma caixa de 8 pixels no canto superior esquerdo aparece ao passar e permanece visível quando selecionada. Menus abertos por botões ficam a 4 unidades do botão e usam sombra suave. O botão para reabrir a prévia é quadrado de lado 28. Os parâmetros estão em layout-scale.json, directory-tree.json e controls.json; a verificação real segue o [método geral](../../../developers/qa/README.md).

<a name="xdzip-settings-window"></a>

## Janela de configurações

Configurações mantém títulos com ícones, cartões de prévia de tema, densidade, histórico recente e explicação de privacidade originais e adiciona tamanho de texto e idioma. Prévias de tema e fonte podem ser canceladas; Salvar e Restaurar padrões usam rascunhos. HTML mantém o estado de demonstração apenas na página atual. Na demonstração HTML, a seleção de idioma mostra uma configuração que entra em vigor na próxima inicialização do Qt; nem a página de revisão nem o conteúdo do arquivo de exemplo comprovam cobertura de tradução do produto. Para a troca de idioma, recursos incluídos e persistência atuais do Qt, veja [configurações e localização](settings-localization.md).
