<a name="xdzip-high-fidelity-states-and-implementation-contract"></a>

# Estados de alta fidelidade e contrato de implementação

[English](../../ux/hifi-state-spec.md) | [简体中文](../../zh-CN/ux/hifi-state-spec.md) | [繁體中文](../../zh-TW/ux/hifi-state-spec.md) | [日本語](../../ja-JP/ux/hifi-state-spec.md) | [한국어](../../ko-KR/ux/hifi-state-spec.md) | [Français](../../fr-FR/ux/hifi-state-spec.md) | [Deutsch](../../de-DE/ux/hifi-state-spec.md) | [Español](../../es-ES/ux/hifi-state-spec.md) | Português (Brasil) | [Русский](../../ru-RU/ux/hifi-state-spec.md)

Este guia descreve comportamento, foco e ações de fechamento das janelas de criação, extração e tarefas, com links para os [originais dos estados de operação](../../../design/hifi/README.md) e a [biblioteca compartilhada de recursos](../../../design/assets/README.md). Os identificadores da tabela se referem a pranchetas de design e não alteram o protocolo das tarefas.

<a name="xdzip-screens-and-task-semantics"></a>

## Telas e semântica das tarefas

| Estado / prancheta | Condição de entrada | Foco e ação padrão | Fechamento e retomada |
| --- | --- | --- | --- |
| C01 / 01 | Criação, nenhuma tarefa em execução | Primeiro botão de entrada ou nome do arquivo; validar entradas, nome, destino e capacidade antes de criar | Descartar o rascunho desta janela; nenhuma gravação |
| E02 / 02 | Um instantâneo da tarefa contém uma solicitação de conflito ativa | Ignorar, Renomear e Sobrescrever são decisões explícitas. Renomear é a ação principal. Aplicar aos conflitos seguintes começa desmarcado | Fechar dispensa a solicitação e deixa a tarefa aguardando. Reabri-la pela tarefa ou usar a ação Cancelar da tarefa para parar |
| T01 / 03, 08 | Plano aceito e evento de execução recebido | Voltar à navegação e cancelar são botões comuns; não há pausa inventada | Apenas ocultar a visualização; a entrada da tarefa permite reabri-la; conclusão em segundo plano não rouba o foco |
| T01 / verificação de integridade | Mesmo componente do progresso, arquivo compactado somente leitura e etapas de verificação | Voltar à navegação, cancelar a verificação e resultado real da tarefa; demonstração HTML e prévia do componente Qt continuam identificadas separadamente | Fechar não cancela; verificações reais entram na fila em segundo plano pela barra de ferramentas |
| R01 / 04 | A tarefa tem itens publicados e inacabados | Tentar novamente seleciona apenas itens inacabados e retorna à confirmação do planejamento | Fechar a visualização não exclui arquivos já confirmados |
| S01 / 05 | Abrir configurações e selecionar uma das seis abas | Todas as edições são acessíveis por teclado; alterações entram em vigor imediatamente após sucesso e endereços devem ser validados | Fechar mantém configurações aplicadas; falha preserva valores anteriores |
| R02 / 06 superior | Recebido o estado terminal único de sucesso | A extração pode abrir a pasta de destino por escolha explícita; a criação abre resultados compatíveis somente no aplicativo, nunca abre automaticamente uma pasta do sistema. Zero falhas usa estado neutro | Fechar detalhes sem repetir a notificação de conclusão |
| T02 / 06 meio | Cancelamento aceito, limpeza incompleta | Cancelamento repetido desativado; ainda é possível voltar à navegação | Pode ocultar; não pode mostrar “Cancelado” antes da hora |
| R03 / 06 inferior | Recebido o estado terminal de cancelamento | Manter itens publicados; se for permitido tentar novamente, replanejar o escopo inacabado | Fechar não desfaz itens publicados |
| C02 / 07 | A prancheta ilustra um arquivo de saída existente | Um conflito real de criação usa as regras de nome seguro e publicação da tarefa ativa | Dispensar a solicitação deixa a tarefa aguardando. Renomear, Substituir ou Cancelar explicitamente segue o resultado da tarefa |

“Usar novo nome” apenas propõe um nome; o início real ainda verifica se o destino existe. Verificar o nome não é a autorização final de gravação. A criação com substituição deve gerar e verificar completamente um arquivo temporário antes de verificar novamente as condições do destino para publicação. Um cancelamento recebido após o ponto de confirmação linearizado exibe a conclusão conforme o resultado do núcleo, sem reescrever sucesso como cancelamento.

<a name="xdzip-creation-capabilities-and-field-errors"></a>

## Recursos de criação e erros de campo

- Os quatro níveis ZIP correspondem a Store/Deflate 1/6/9, com 2 normal como padrão. TAR e TAR.GZ podem ser criados; mudar o formato sincroniza extensões e limpa a senha e a divisão em volumes. TAR desativa o nível; os níveis TAR.GZ 0/1/2/3 correspondem a GZIP 0/1/6/9. A mudança sincroniza a extensão sem sobrescrever destinos silenciosamente.
- Sem entrada, com nome vazio, destino inválido ou permissão ausente, Criar fica desativado e o motivo aparece ao lado do campo. O texto do erro participa da leitura de tela, em vez de mudar apenas a cor da borda.
- O núcleo trata entradas duplicadas e expansão de diretórios; a interface mostra apenas resumos da seleção. Ícones de imagem na criação indicam o tipo, não são prévias de imagem.
- ZIP e 7z expõem suas configurações compatíveis de senha e volumes. 7z também pode criptografar nomes de arquivos. O registro de formatos lista os recursos reais de gravação. [Criação ZIP](../engineering/zip-creation.md), [registro de formatos](../engineering/format-registry.md)

<a name="xdzip-task-data-and-exceptions"></a>

## Dados de tarefas e exceções

O exemplo A tem três arquivos: 8,4 + 6,2 + 12,8 = 27,4 MB. Com dois sucessos, 14,6 MB são salvos e o PDF restante tem 12,8 MB; contagens de itens bem-sucedidos, ignorados e inacabados não devem ser confundidas. O exemplo B é um arquivo compactado inteiro com 30 arquivos e 77,5 MB; 46,5 MB são 60%, não o mesmo momento do exemplo A.

Porcentagens derivam do total conhecido de bytes e da medição real. Total desconhecido usa progresso indeterminado sem inventar tempo restante. Estimar novamente a velocidade após mudança de fase; os 6,2 MB/s e aproximadamente 5 segundos ilustrados são apenas exemplo de uma fase estável. As contagens distinguem arquivos concluídos e em processamento, sem usar o número ordinal do item atual como contagem de sucessos.

Disco cheio: parar e manter itens publicados; replanejar após liberar espaço ou mudar o destino. Origem alterada: tentar novamente significa reabrir o arquivo compactado; índices antigos não podem ser reutilizados. Apenas cancelamento: itens inacabados usam âmbar ou cor neutra, sem relatar corrupção. Códigos de erro aparecem nos detalhes; telas comuns não mostram pilhas de chamadas nem termos internos da ponte.

<a name="xdzip-immediately-applied-settings"></a>

## Configurações aplicadas imediatamente

Tema, tamanho da fonte, densidade, idioma, modo de abertura e opções de privacidade são salvos e aplicados imediatamente após alterações bem-sucedidas. Limpar o histórico recente acontece imediatamente sem excluir os arquivos originais. Em caso de falha, as configurações anteriores são preservadas e uma mensagem aparece. Sobre e atualizações ficam na página Sobre das configurações; associações de arquivos exigem confirmação do sistema e refletem resultados reais. A página de rede salva endereços validados de proxy e espelho. Consulte o [contrato de configurações](settings-localization.md).

Os orçamentos de saída padrão do hospedeiro existente são 8 GiB por arquivo e 20 GiB por tarefa; a janela atual de configurações não permite editar cotas. Se limites forem adicionados no futuro, validar inteiros positivos e estouro, garantir que o limite por arquivo não exceda o da tarefa e aplicá-los apenas a novos planos. Não alterar a autorização imutável de tarefas ativas nem ignorar verificações de espaço.

<a name="xdzip-window-layout-and-asset-mapping"></a>

## Mapeamento de janelas, layout e recursos

Todas as janelas de operação seguem a [especificação de janelas por plataforma](platform-windows.md). Barras de título nativas do macOS não usam botões de semáforo recortados. A camada de plataforma verifica posição e estados desativados dos botões de título do Windows/Linux, menus do sistema e movimentação. Controles à direita em imagens de comparação do Linux não são uma promessa fixada no código para todos os ambientes de desktop.

As dimensões atuais seguem [layout-scale.json](../../../specs/visual/layout-scale.json), o [registro de janelas](../../../specs/visual/window-registry.json) e o [padrão de design](../../visual/design-standard.md), substituindo sugestões iniciais grandes demais. Criação, extração, conflito, progresso e resultados compartilham largura 491; configurações usa 612 como diferença registrada para sua finalidade. Título 20, corpo 13, botões de rodapé 13 e texto de apoio 12. A altura segue o conteúdo e o limite de cada janela; o corpo rola. Janelas de operação com rodapé o mantêm fixo e acessível; configurações não tem rodapé. Configurações oferece tamanhos de fonte 100%/125%/150%; não deduzir dimensões de componentes do escalonamento global ou dos pixels físicos de capturas.

| Uso | Recursos adicionados | Partes compartilhadas / dinâmicas |
| --- | --- | --- |
| Título de criação, adicionar entradas | archive-add, image-file, remove | folder, file-pdf, new |
| Conflito, falha parcial | document-warning, status-warning, info | file-pdf; cartões de comparação e texto desenhados por QML |
| Progresso em execução | document-download, image-file, info | Barras de progresso e números desenhados dinamicamente |
| Sucesso, ignorar, cancelar, parar | status-success, status-skipped, status-stopped, spinner, clock | Controle Qt gira spinner |
| Configurações | settings-tile, theme-system | Prévias de tema, alternâncias e entradas numéricas são componentes; settings é um ícone pequeno |

Os recursos ficam em `design/assets/`. `reference/` mantém recortes nos pixels originais para comparar a aparência; `svg/` e `png/` são recursos transparentes de execução e não devem ser chamados de camadas originais. Todos os SVGs usam telas de 24×24; PNGs são exportados em 24×24, 48×48 e 72×72, mantendo o mesmo tamanho lógico de exibição. Listas HTML usam um espaço de 48×48 nas coordenadas de origem; Qt usa aproximadamente 31,75×39,69 pixels lógicos. Ícones de PDF, texto, código e Markdown compartilham centro e altura visível de 22 pixels, mantendo larguras de contorno distintas; pastas são centralizadas na mesma tela. SVGs escuros são adaptações de recursos; o comportamento escuro completo ainda exige verificações reais de Qt e contraste.

<a name="xdzip-directory-tree-addition"></a>

## Adição da árvore de diretórios

A navegação B01 adiciona a [especificação B da árvore de diretórios](directory-tree.md), com três pranchetas formais e 26 recortes compartilhados. Setas de nós dependem de subdiretórios; expansão, diretório atual e foco do teclado são independentes. Carregamento, arquivos compactados vazios, arquivos na raiz, limites de navegação, falhas e caminhos atuais ocultos são especificados separadamente; o contrato de domínio somente leitura existente não muda. HTML e Qt integram a navegação principal, linhas de conexão e indicadores de ramo atual oculto; esqueletos e ícones específicos de limites continuam como recursos de design. Consulte a especificação da árvore para o escopo da integração.

<a name="xdzip-changes-and-revalidation"></a>

## Alterações e nova validação

Os principais estados foram usados em HTML e Qt. Alterações posteriores ainda verificam os designs originais e parâmetros revisados, o mapeamento de controles para funções, os componentes compartilhados e contratos de janela, além de validar as plataformas afetadas. Designs ou verificações estáticas por si só não comprovam funcionamento nativo nem acessibilidade. Registrar conclusões reais na issue correspondente; esta lista não reinicia a aceitação histórica.
