<a name="xdzip-product-runtime-and-qt-lifecycle"></a>

# Runtime do produto e ciclo de vida do Qt

[English](../../engineering/product-runtime.md) | [简体中文](../../zh-CN/engineering/product-runtime.md) | [繁體中文](../../zh-TW/engineering/product-runtime.md) | [日本語](../../ja-JP/engineering/product-runtime.md) | [한국어](../../ko-KR/engineering/product-runtime.md) | [Français](../../fr-FR/engineering/product-runtime.md) | [Deutsch](../../de-DE/engineering/product-runtime.md) | [Español](../../es-ES/engineering/product-runtime.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/product-runtime.md)

Este guia explica como o aplicativo de desktop se conecta ao runtime Rust por comandos e eventos, incluindo contrapressão das filas, encerramento e builds por plataforma. Consulte [navegação somente para leitura](readonly-browser.md) e [extração segura](zip-safe-extraction.md) para os respectivos recursos.

<a name="xdzip-implemented-services"></a>

## Serviços implementados

`xdzip-runtime::service` implementa o serviço do produto. A interface obtém `CommandClient` uma única vez e uma thread receptora Qt dedicada obtém `EventReceiver` uma única vez. A thread de comandos Rust é dona do arquivo aberto da conexão e do instantâneo de navegação. Uma thread Rust de tarefas separada recebe trabalho preparado com a origem e as configurações necessárias e publica atualizações. Handles de arquivos compactados e pontos de entrada de varredura síncrona permanecem em Rust. Veja a [ponte Qt/Rust](../design/qt-rust-bridge.md) para a ABI e o [agendamento de tarefas](../design/task-scheduling.md) para propriedade e admissão.

Os comandos centrais incluem Open, Browse, Preflight, CloseSession e Snapshot; toda a navegação e pré-verificação reais existentes os usam. Cada solicitação comum aceita recebe um ID não reutilizado, retorna no máximo um evento de dados limitado e depois exatamente um Finished; erros retornam Finished diretamente. As páginas carregam identidade de sessão, solicitação e deslocamento. Cada RuntimeConnection tem uma sessão e solicitações pendentes da interface em série. Consultas também incluem ID completo da consulta, geração da interface e revisão do instantâneo; veja o [contrato de navegação ZIP](zip-readonly-browsing.md). Solicitações entre sessões retornam stale_session.

A admissão comum da interface usa try_lock sem aguardar espaço na fila e valida caminhos de 4 KiB, pesquisa de 1 KiB, 128 itens selecionados, IDs de diretório de 21 bytes e códigos de ordenação 0–12. Uma recusa não consome ID; Busy pode ser tentado novamente. O cancelamento tem canal de controle separado com até 128 itens e sua própria confirmação Finished; ela não garante que o alvo ainda não tenha terminado. O encerramento mantém um RequestId independente e idempotente e reserva o último ID disponível para funcionar mesmo se os IDs comuns acabarem ou a fila estiver cheia.

O controlador Qt mantém apenas um comando aguardando admissão. Busy temporário é tentado novamente no loop de eventos por um temporizador de execução única de 1 ms, por no máximo 32 tentativas; apenas saturação persistente pede nova tentativa. Isso não é prazo de conclusão. Cancelamento, encerramento ou desconexão retiram imediatamente comandos não aceitos; não há temporizador de repetição em repouso. Argumentos inválidos e IDs esgotados são informados separadamente, sem serem apresentados como ocupação. Open é desativado após desconexão.

<a name="xdzip-bounded-transport"></a>

## Transporte limitado

- Há no máximo 128 solicitações comuns pendentes; a capacidade só é liberada quando o receptor retira Finished. Dois espaços de evento são reservados por solicitação para que dados não desloquem estados terminais.
- Eventos de dados Rust têm orçamento acumulado de conteúdo de 32 MiB, incluindo imagens de até 16 MiB e seus contêineres. A publicação aguarda quando o orçamento é excedido; cancelamento ou encerramento desperta a espera e pode emitir um estado terminal cancelado. Um único evento grande demais retorna resource_limit.
- Comandos Rust mantêm limite acumulado de conteúdo de 8 MiB; excesso é rejeitado na admissão e o orçamento maior de eventos de imagem não o altera.
- A caixa Qt comporta até 256 eventos e 32 MiB de conteúdo. No máximo 128 espaços são de dados comuns; os demais são reservados para eventos terminais ou de controle. Ela não cria um callback Qt por evento Rust.
- Uma cadeia de notificação processa no máximo 32 eventos ou aproximadamente 4 ms por rodada e agenda outra se houver trabalho; o processamento de um único evento não é preemptível. Filas vazias deixam de agendar e o receptor espera em uma variável de condição.
- Page de Open também traz o índice completo e limitado de diretórios, até 32.768 nós e 4 MiB de nomes/prefixos, marcado para substituição por `replace_directories`; um índice vazio também substitui explicitamente. Browse não envia o índice novamente; `navigation_limited` indica explicitamente indexação reduzida. Rust contabiliza bytes de nomes e prefixos; a caixa Qt também conta strings de ID de nó e pai geradas pela ponte. Contêineres de nós e profundidade têm limites próprios. Aplicar página e modelo de diretórios juntos somente após o Finished bem-sucedido da solicitação correspondente.
- `total` de Page conta as linhas da lista atual; `archive_total` mantém separadamente o número de registros físicos do arquivo. A navegação comum lista arquivos e diretórios diretos do diretório atual; a busca retorna recursivamente correspondências físicas. Diretórios sem registro explícito usam `navigation_id` e `physical=false`; chaves de linha Qt são `dN`, IDs físicos originais ficam separados e nós virtuais não podem enviar Preflight. Os metadados mantêm caminhos completos; são exibidos o nome final na navegação e o caminho completo na busca, respectivamente.
- Orçamentos de strings não são RSS: contêineres, instantâneos e um evento em conversão exigem espaço adicional, mas entradas e páginas continuam limitadas. Lotes da ponte contêm no máximo 128 itens. Veja o [contrato de navegação ZIP](zip-readonly-browsing.md) para a base original de 100.000 entradas e a [indexação de arquivos grandes](large-index.md) para a expansão posterior de ZIP comuns de arquivo único.

Eventos de controle e comuns são retirados alternadamente para que confirmações contínuas de cancelamento não impeçam estados terminais comuns. Números de sequência crescentes são atribuídos na entrega. Qt descarta sequências antigas; confirmações de controle e request_ids obsoletos não modificam a visualização atual.

Envelopes `Task`/`TaskUpdate` no canal independente de tarefas mantêm o request_id original de criação da tarefa. Registros de correlação permanecem nos espaços limitados de controle de tarefas e são liberados na entrega do estado terminal da tarefa. Eles podem preceder os dados de aceitação ou vir após o Finished da solicitação. Não consomem um segundo espaço de dados comuns nem liberam ou concluem novamente a solicitação; vinculam corretamente os marcadores de preparação ao TaskId real e à sessão de origem. Esse ID não é um ID de solicitação de cancelamento; o controle de tarefas continua usando TaskId.

<a name="xdzip-shutdown-destruction-and-failure"></a>

## Encerramento, destruição e falha

Fechar um arquivo ou dispensar uma janela pode deixar o aplicativo e as tarefas ativos na bandeja. Ao sair ou reiniciar, o coordenador interrompe admissões, solicita cancelamento e esvazia eventos enquanto Rust libera sessões. O receptor aguarda as duas threads Rust antes de retornar `ShutdownReady`. O desktop espera o receptor Qt terminar e a caixa esvaziar antes de sair. A thread da interface nunca espera threads que estejam aguardando trabalho de arquivo ou entrega de eventos.

O controle de cancelamento e encerramento usa seções críticas curtas com operações de estado limitadas, independentes da capacidade da fila de dados. Cancelamento cooperativo não interrompe uma leitura bloqueada pelo sistema operacional e não garante saída em tempo real rígido. As solicitações somente de leitura aqui não produzem transações de saída. Confirmações de criação/extração, limpeza temporária e resultados de saída seguem separadamente os contratos de [criação](zip-creation.md) e [extração segura](zip-safe-extraction.md).

Se um QObject for destruído antes da hora, feche a caixa, desperte o receptor e interrompa o serviço. O trabalho em segundo plano mantém apenas filas compartilhadas e o handle do receptor Rust, não ponteiros para o controlador. Qt cancela automaticamente notificações de objetos destruídos. QPointer também protege a vida útil se um callback de consumo destruir a conexão de forma síncrona. Ao abandonar a recepção, não aguarde threads no destrutor; o trabalho termina cooperativamente. Apenas um ShutdownReady normal comprova que a espera terminou.

Se um manipulador de leitura do processo de trabalho encontrar um panic que permite desenrolar a pilha, a fronteira o captura, descarta todo o serviço, envia RuntimeFault e encerra solicitações incompletas como runtime_unavailable; a sessão não é reutilizada. Isso define apenas falha de leitura e não promete recuperar transações de criação ou extração parcialmente concluídas. Itens confirmados, resultados incertos e limpeza de gravações seguem seus contratos. Falha de alocação irrecuperável, aborto do processo ou bloqueios internos envenenados não garantem entrega terminal. Em EOF inesperado ou falha da conexão, Qt encerra uniformemente estados pendentes para evitar ocupação permanente.

<a name="xdzip-verification-and-build"></a>

## Verificação e build

Veja a [verificação geral](../../../developers/qa/README.md). Testes do núcleo Rust não exigem Qt; casos Qt de janela, modelo, estresse e encerramento são compilados e executados nativamente nas três plataformas. A CI do macOS verifica arm64 e inicialização Cocoa no ambiente SDK sem implantar ou enviar apps; DMGs de desenvolvimento locais podem ser produzidos mediante pedido explícito. Windows/Linux implantam Qt incluído e verificam execução sem SDK; veja o [guia de build](../engineering/developer-build.md). Essas verificações não são aprovação de lançamento oficial.

Consultas iniciais alocam uma identidade imutável de resultado; lotes posteriores devem corresponder a todos os campos e parâmetros de QueryKey. Qt cancela solicitações antigas e agrupa a intenção mais recente mantendo verificações de sequência e ID. Um lote também limita nomes a 64 KiB e não filtra nem ordena novamente. A aceitação histórica permanece limitada aos recursos disponíveis na época.

A camada de aplicação pode criar várias conexões independentes enquanto o runtime de domínio isola sessões por conexão. Veja o [contrato de sessões](application-sessions.md) para identidade global de tarefas e admissão serial de tarefas pesadas.
