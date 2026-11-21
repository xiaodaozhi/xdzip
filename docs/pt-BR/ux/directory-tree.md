# Árvore de diretórios B: design, recursos e integração

[English](../../ux/directory-tree.md) | [简体中文](../../zh-CN/ux/directory-tree.md) | [繁體中文](../../zh-TW/ux/directory-tree.md) | [日本語](../../ja-JP/ux/directory-tree.md) | [한국어](../../ko-KR/ux/directory-tree.md) | [Français](../../fr-FR/ux/directory-tree.md) | [Deutsch](../../de-DE/ux/directory-tree.md) | [Español](../../es-ES/ux/directory-tree.md) | Português (Brasil) | [Русский](../../ru-RU/ux/directory-tree.md)

Estado: **a interação principal de diretórios está integrada à navegação somente leitura do Qt; os recursos dos estados de limite aguardam integração**. A interação segue o [contrato de navegação somente leitura](../engineering/readonly-browser.md). A navegação normal usa filhos diretos, a busca cobre recursivamente o escopo atual e a autorização de saída é tratada separadamente da leitura.

As três pranchas atuais são [expansão multinível](../../../design/hifi/screens/directory-tree-capsule.png), [estados de nós e interação](../../../design/hifi/screens/directory-tree-states.png) e [limites de navegação e estados vazios](../../../design/hifi/screens/directory-tree-boundaries.png). A galeria compartilhada mostra as pranchas completas e a [biblioteca de recursos](../../../design/assets/index.html#directory-tree) fornece os recortes individuais. A expansão multinível é exportada da fonte editável, com a barra de título sincronizada com as abas em cápsula atuais. As listas superior e central da prancha fornecem apenas contexto e não redefinem barra de título, barra de ferramentas ou tabela de arquivos. O [original multinível histórico](../../../design/hifi/screens/directory-tree.png) permanece inalterado como fonte de 26 recortes e referência independente de medição.

As pranchas são ampliadas e as escalas dos cartões diferem. Texto, raios e dimensões em pixels das imagens não podem definir diretamente o tamanho dos controles; os [parâmetros de componentes](../../../specs/visual/directory-tree.json) registram explicitamente as dimensões lógicas implementadas e o manifesto de recursos registra separadamente as coordenadas físicas dos recortes. A implementação usa largura lógica 1024 e uma transformação compartilhada do canvas de origem; não amplie novamente todo o novo conjunto de componentes.

O Qt usa diretamente os valores lógicos abaixo. A referência de 1548 pixels corresponde a 1024 pixels lógicos conforme layout-scale.json. O DPR determina apenas a densidade de amostragem e o alinhamento das linhas, sem alterar as dimensões lógicas.

## Estrutura da árvore e contagens

O índice completo fornece diretórios explícitos e implícitos. Uma folha significa **sem subdiretórios**, não sem arquivos. Se Glacier Lake contiver um subdiretório, ele terá sua própria seta de expansão, independentemente da seta do pai.

```text
Root                               ← current directory
▼ 01 Photos                     12
  ▼ Glacier Lake                 6
    ▸ Lakeside Trail             3  ← deeper directory, not expanded
      Snow Mountain View          3  ← files only, retains empty arrow slot
  ▸ Mountain Sunrise             4
    Other Photos                 2  ← no subdirectories
▸ 02 Documents                   8
  03 Maps                        4  ← no subdirectories
```

O arquivo de exemplo contém 12 fotos, 8 documentos, 4 mapas e 6 arquivos na raiz; a lista raiz mostra três diretórios e seis arquivos diretos sem expandir os descendentes. Os descendentes de ramos recolhidos também são listados integralmente no arquivo de parâmetros para que folhas não recebam setas falsas. O diretório vazio explícito de outra prancha é um exemplo de estado separado, não acrescentado a este exemplo de 30 entradas.

Os números dos diretórios representam todos os **registros físicos do arquivo** descendentes abaixo do prefixo, incluindo nomes duplicados, mas excluindo o registro do diretório idêntico ao prefixo. Eles não são contagens de subdiretórios e não mudam com a busca por nome. As linhas da árvore não exibem contagens. O resumo do arquivo mantém a contagem completa de registros físicos, enquanto o lado direito do título da lista mostra o número de linhas do diretório atual; as definições são diferentes. Listas comuns exibem o nome final; resultados de busca exibem o caminho completo dentro do arquivo.

As setas dependem apenas dos subdiretórios conhecidos: para a direita quando o pai está recolhido e para baixo quando expandido. Nós que contêm apenas arquivos ou estão vazios não têm seta, mas os pares mantêm os mesmos espaços de seta e ícone e os mesmos inícios de texto. Durante o carregamento do índice completo, não adivinhe setas, contagens ou estrutura por nó nem adicione estados assíncronos de carregamento por nó.

## Layout, fontes e geometria dinâmica

| Item | Tamanho lógico / comportamento normal |
| --- | --- |
| Largura padrão da barra lateral | 193.16; o limite estrutural da janela principal arredonda para 193 |
| Altura da linha | Mínimo 32; cresce conforme a fórmula dos parâmetros com fontes maiores |
| Recuo por nível | 12.8; independente do tipo de nó |
| Área de toque / espaço gráfico da seta | Pelo menos 19.2×19.2 / 14.4×14.4; folhas mantêm o mesmo espaço |
| Espaço do ícone de pasta | 16×16; o SVG compartilhado de 24 unidades escala proporcionalmente, substituindo o espaço 18 da árvore atual |
| Espaço entre ícone e texto / margem final | 6.4 / 16; sem contagem visível |
| Texto principal | 11.2; Qt arredonda para 11 e usa fonte da mesma origem |
| Contagem de registros | Mantida nos dados e na semântica acessível; não aparece nas linhas e não reserva coluna |
| Diretório atual | Fundo azul-claro, cantos direitos de 6.4, borda esquerda reta e indicador de largura 2.4 com altura total |
| Foco do teclado | Sem contorno ou barra de foco extra e sem mudar a ocupação da linha; o indicador do diretório atual permanece |
| Linhas de hierarquia | Largura 1; hierarquia discreta em relação ao fundo |
| Marcador do diretório atual oculto | Ponto de 4.8 unidades em espaço transparente de 19.2 unidades, compartilhando o centro da linha do nome |

As linhas de conexão são desenhadas dinamicamente a partir da cadeia de ancestrais visível e das marcas de último irmão. As linhas verticais dos pais cruzam subárvores expandidas até o centro do último irmão; a última conexão termina em L e não avança para o próximo grupo de nível superior. O centro X de cada linha vertical alinha-se à ponta X da seta descendente do pai, inclusive com fontes maiores de acessibilidade. As linhas internas terminam de forma independente; uma imagem genérica de linha não pode cobrir todas as linhas. Alinhe segmentos ao DPR de destino; texto, setas e pastas compartilham o centro vertical da linha.

A região da hierarquia rola horizontalmente. A largura usa apenas nomes reais, recuo e margem final de 16 dos nós visíveis atualmente expandidos. Os nomes permanecem completos sem sobreposição de números. Recalcule a largura após recolher para que nós profundos ocultos não deixem espaço vazio; o indicador esquerdo fica fixo à borda da área visível. Ao passar o cursor sobre uma linha não aparece dica flutuante; caminhos completos estão disponíveis nas descrições acessíveis e nos detalhes. A rolagem vertical ocorre apenas dentro da área de diretórios; as barras não envolvem a borda da janela principal.

Todos os diretórios usam o mesmo recurso de pasta. Pontos de ramo, ícones preenchidos de informação e aviso e indicadores segmentados de carregamento têm SVG claro/escuro e PNG 1×/2×/3×; linhas, quadros de foco, fundos, esqueletos, texto e mensagens são componentes dinâmicos. Os parâmetros escuros reutilizam o tema global com cores adicionais de linha e marcador; **o original escuro completo da árvore ainda está pendente** e a exportação de ícones escuros não substitui a prancha inteira. Conclusões existentes de renderização e aceitação no desktop são registradas separadamente nas issues e nos artefatos correspondentes.

Registre a consistência de interação entre Qt conforme o [método de verificação](../../../developers/qa/README.md). Pranchas e recortes preservam o histórico do design original; contagens de linha, foco e cantos atuais seguem esta página e a especificação lógica.

## Interações e estados independentes

Expansão, diretório atual, passagem do cursor e foco do teclado são armazenados independentemente e podem coexistir. O diretório atual também é diferente da seleção por caixas dos arquivos à direita.

A raiz é uma entrada navegável separadamente, sempre expandida, que não participa da expansão ou do recolhimento comum dos nós. Quando clique direito, tecla Menu ou Shift+F10 abre seu menu, “Recolher subdiretórios” permanece desativado sem linha de explicação nem dica visível. A descrição acessível é “根目录始终展开” / “Root is always expanded”; a ausência de uma linha comum na árvore não significa ausência de subdiretórios. A expansão ou o recolhimento de pais comuns depende dos subdiretórios reais; folhas comuns mantêm a descrição acessível “This node has no subdirectories”. Os motivos são apenas acessíveis, não dicas visíveis. Abrir o menu ou clicar em itens desativados não deve mudar o diretório, a seleção de arquivos ou a expansão da árvore.

| Ação | Resultado |
| --- | --- |
| Clicar na seta | Expandir/recolher apenas esse nó; consulta de diretório, seleção de arquivos e detalhes permanecem iguais |
| Clicar no nome ou Enter | Entrar usando o ID de diretório da sessão atual; após substituição bem-sucedida da consulta, redefinir consulta do diretório atual, seleção e detalhes conforme o contrato existente |
| Cima / baixo | Mover o foco entre linhas visíveis de diretório sem trocar automaticamente a lista de arquivos |
| Direita | Expandir um pai recolhido e, se já expandido, ir ao primeiro filho; folhas permanecem no lugar |
| Esquerda | Recolher um pai expandido; caso contrário, mover para o pai. Nós de nível superior permanecem no lugar |
| Recolher um ancestral do diretório atual | Manter diretório atual e lista de arquivos; o foco oculto retorna ao ancestral visível mais próximo |
| Nova sessão de arquivo | Limpar IDs de nós antigos, expansão, foco e pontos de ramo; não reutilizar estado de outro arquivo |

Por exemplo, quando o caminho é `01 Photos / Glacier Lake`, recolher `01 Photos` mantém Glacier Lake à direita. A linha Photos mostra apenas um ponto azul no fim, sem receber o fundo de diretório atual de Glacier Lake; a raiz também não se torna atual. O ponto azul fica no **ancestral recolhido visível mais próximo**; cada diretório atual oculto tem um marcador de ramo. Ao expandir novamente, o fundo atual de Glacier Lake é restaurado.

A árvore inteira é uma única entrada de navegação por teclado. Os nós mantêm semântica acessível para ID estável, nome, nível, expansão, posição e contagem de registros. Folhas não se anunciam como expansíveis; pastas, linhas e pontos decorativos não são anunciados de modo redundante. Caminhos atuais ocultos têm descrições acessíveis e não dependem apenas de azul. Teclas de seta e cliques nas setas não devem se propagar para a ativação da linha.

## Cobertura de estados e recuperação

| Estado | Árvore de diretórios | Lista de arquivos / recuperação |
| --- | --- | --- |
| Leitura | Mensagem de carregamento, contagem desconhecida; esqueleto é recurso futuro; navegação desativada | Seguir o estado global de carregamento; substituir a árvore uma vez após a publicação do índice completo |
| Arquivo vazio | Linha raiz sem número; “Este arquivo está vazio” | Lista vazia; outro arquivo ainda pode ser aberto |
| Apenas arquivos na raiz | Linha raiz sem número; “Nenhum subdiretório” | Mostrar seis arquivos normalmente; não transformar a página inteira em estado vazio |
| Limite do índice de diretórios excedido | Não publicar meia árvore; mostrar “O diretório excede o limite de exibição” | Raiz, busca e navegação de diretórios explícitos elegíveis continuam dentro dos recursos atuais |
| Falha de leitura | Atenuar navegação e mostrar erro explícito | Reabrir arquivo; não interpretar falha como zero ou diretório vazio |
| Nó com subdiretórios | Seta independente em cada nível | Expandir não entra nesse diretório |
| Folha não vazia / vazia | Contagem mantida nos dados; nenhuma mostra contagem de linha ou seta | Nome continua ativável; diretório vazio pode ser navegado |
| Nome longo / aninhamento profundo | Nome completo, caminho acessível, rolagem horizontal e largura baseada em nós visíveis | Filtragem de caminhos permanece consistente |
| Fonte 150% (200% é apenas caso histórico de estresse) | Espaços e linhas crescem; linhas são recentradas | Nomes, setas e foco não são cortados |
| Diretório atual oculto | Ancestral visível mais próximo mostra ponto de ramo | Lista mantém caminho atual; expandir novamente restaura a linha atual |

Limites de navegação e falha global de leitura são estados diferentes; motivos, ações disponíveis e fontes de contagem não podem compartilhar um modelo de “diretório vazio”. Os limites existentes de 32.768 nós, 64 níveis e 4 MiB de nomes de diretórios vêm do contrato somente leitura; o design não os aumenta nem contorna.

## Entrega de recursos e integração real

Os 26 recortes em pixels originais ficam em `design/assets/reference/tree-*.png`, cada um com origem, SHA-256, `x,y,width,height` e finalidade. `component_reference_coverage` aponta individualmente para vetores compartilhados ou componentes dinâmicos. Não descreva imagens estáticas de linhas inteiras como controles interativos nem remova fundos brancos dos recortes para fabricar transparência.

| Parte | Recurso / desenho | Integração atual |
| --- | --- | --- |
| Pasta, setas direita/baixo | `folder`, `chevron-right`, `chevron-down` dos recursos compartilhados | Recursos compartilhados existentes disponíveis; ícones e dimensões B usados por delegados Qt |
| Carregamento, informação, aviso | Novos `tree-spinner`, `tree-info`, `tree-warning` | Parte de oito SVG claros/escuros e 24 PNG transparentes; mantenha as formas preenchidas e segmentadas das pranchas em vez de ícones de contorno existentes |
| Ramo atual oculto | Novo `tree-active-branch` | Seis PNG transparentes para dois temas entregues; Qt usam o recurso para marcar o ancestral recolhido visível mais próximo |
| Linhas, fundo atual, foco, esqueleto, dicas | Desenho dinâmico com dimensões e temas do arquivo de parâmetros | Estilo e lógica especificados; nenhum segundo conjunto de controles bitmap entregue |
| Pranchas oficiais | `hifi/screens/directory-tree*.png` | Preservar bytes dos originais confirmados para comparação do design |

Qt integra linhas de conexão, pontos de ramo, espaços de pasta de 16 unidades e menus de contexto de diretório. Qt usa o índice real limitado. Recolher o ancestral atual não envia consulta. Esqueletos de carregamento e ícones dedicados de informação e erro permanecem como recursos de design; a prancha escura completa está pendente. A integração de ícones não comprova aceitação de leitores de tela ou HiDPI em cada plataforma; conclusões manuais existentes seguem [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5).

## Aceitação após integração

1. Capture a região real de diretórios com a mesma largura lógica 1024, configuração de fonte, DPR e dados de estado e compare com a região correspondente da prancha confirmada e com a especificação. Registre a relação de escala e preserve separadamente julgamento visual e verificações estruturais.
2. Alinhe os inícios de seta, ícone e nome em cada nível, sem coluna numérica de linha. Ícones, texto, pontos e setas compartilham os centros das linhas. O desvio geométrico visado é de no máximo um pixel físico; registre a plataforma e a fonte substituta real em caso de diferenças.
3. Expanda pelo menos três níveis, incluindo um ramo mais profundo recolhido e uma folha não vazia. Avalie a continuidade e o término das linhas de conexão até os últimos irmãos e através das subárvores expandidas.
4. Verifique estados atual, passagem do cursor e foco separadamente e juntos. Cliques nas setas não enviam consulta de diretório nem limpam a seleção de arquivos; a ativação por Enter segue a regra de redefinição após consulta bem-sucedida.
5. Com Glacier Lake atual, recolha Photos: o caminho da lista permanece, a raiz não é selecionada e Photos mostra apenas um ponto de ramo. Expanda novamente para restaurar a linha atual. A navegação por teclado continua possível quando o foco está oculto.
6. Verifique arquivos vazios, arquivos na raiz, diretórios vazios explícitos, falha de leitura e limite de diretórios com um índice real limitado; verifique separadamente contagens, desativação e ações de recuperação.
7. Com nomes longos, fontes 100%/125%/150% e o caso histórico de estresse 200%, janela mínima e profundidade máxima permitida do índice, os nomes permanecem totalmente visíveis, a rolagem horizontal afeta apenas a hierarquia e os caminhos completos são acessíveis. A rolagem da árvore não move o resumo do arquivo nem as estatísticas inferiores.
8. Conclua a aceitação da renderização real clara/escura, densidade normal/Retina/alta do Windows, teclado e leitor de tela. Aprovação em verificações de imagem ou geração de recursos não substitui essas verificações em execução.

Conforme o [método de verificação](../../../developers/qa/README.md), verifique separadamente originais, recortes, recursos e comportamento real de mouse, teclado, menus e árvore. Marque explicitamente acessibilidade e diferenças de plataforma não testadas.
