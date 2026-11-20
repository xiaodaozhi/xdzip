<a name="xdzip-qt-rust-bridge-and-application-service"></a>

# Ponte Qt/Rust e serviço da aplicação

[English](../../design/qt-rust-bridge.md) | [简体中文](../../zh-CN/design/qt-rust-bridge.md) | [繁體中文](../../zh-TW/design/qt-rust-bridge.md) | [日本語](../../ja-JP/design/qt-rust-bridge.md) | [한국어](../../ko-KR/design/qt-rust-bridge.md) | [Français](../../fr-FR/design/qt-rust-bridge.md) | [Deutsch](../../de-DE/design/qt-rust-bridge.md) | [Español](../../es-ES/design/qt-rust-bridge.md) | Português (Brasil) | [Русский](../../ru-RU/design/qt-rust-bridge.md)

O desktop Qt chama o runtime Rust pela ponte `cxx` vinculada. Cada `BrowserController` possui uma `RuntimeConnection` com uma instância de serviço separada. A ponte transporta comandos tipados e resultados com propriedade transferida dentro do processo de desktop. A CLI usa diretamente o mesmo serviço Rust. [Arquitetura do sistema](../../design/system-architecture.md), [Sessões de desktop](../../engineering/application-sessions.md)

<a name="xdzip-handles-threads-and-ownership"></a>

## Handles, threads e propriedade

`new_runtime()` inicia as threads Rust de comandos e tarefas. O `Runtime` temporário entrega um `CommandClient` e um `EventReceiver`. A thread da interface usa o handle de comando. Uma thread receptora Qt dedicada possui o handle de eventos e bloqueia em `next_event()`. A thread de comandos Rust possui o arquivo compactado atual e seu índice. A thread de tarefas separada possui o trabalho preparado durante a execução.

CXX transporta tipos por valor e contêineres Rust proprietários. O código Qt receptor mantém ou converte esses valores enquanto detém sua propriedade. Nenhum handle de arquivo compactado ou arquivo nativo atravessa para Qt. Quando IDs ou contagens exatas de bytes chegam ao QML, o controlador usa strings para valores que precisam preservar todos os 64 bits. Os cabeçalhos gerados e a biblioteca estática são compilados junto com o host. Essa interface pertence ao build do produto. O processo de trabalho incluído usa um [protocolo de processo](../engineering/native-codecs.md) separado.

[Declarações e conversões da ponte](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-qt-bridge/src/lib.rs), [Conexão Qt](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/RuntimeConnection.cpp), [Serviço Rust](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/service.rs)

<a name="xdzip-request-query-and-task-identities"></a>

## Identidades de solicitações, consultas e tarefas

| Identidade | Finalidade |
| --- | --- |
| ID de solicitação | O serviço Rust atribui um ID na admissão, no escopo da conexão. `Submission` retorna `Accepted`, `Busy`, `Closed`, `Invalid` ou `Exhausted`. Recusas retornam ID zero. |
| ID de sessão | Identifica o instantâneo do arquivo aberto. Comandos que atuam nele devem corresponder à sessão. Novas aberturas recebem novo ID de sessão. |
| Identidade e deslocamento da consulta | Uma página é vinculada à sessão, ID da consulta, geração da interface, revisão e deslocamento. O controlador valida esses valores antes de aceitar linhas. |
| ID de entrada física ou navegação | Uma entrada física mantém sua identidade na ordenação e paginação. Diretórios sintetizados têm identidades próprias. Índice de linha, nome exibido ou intervalo de consulta nunca concede acesso a um caminho do sistema de arquivos. |
| IDs de tarefa e solicitação de resposta | Um ID de tarefa pertence à sua conexão. Uma solicitação identifica uma resposta pendente de conflito ou senha nessa tarefa. A entrada de tarefa da aplicação também conhece o controlador responsável. |
| Sequência de eventos | O receptor atribui sequência crescente por conexão ao retirar eventos. Qt descarta eventos cuja sequência não seja mais recente que a última consumida. Atualizações de tarefa compartilham essa sequência e usam o ID original da solicitação para correlação. |

<a name="xdzip-commands-in-the-current-bridge"></a>

## Comandos da ponte atual

| Comando ou método | Finalidade e resultado comum |
| --- | --- |
| `Open`, `OpenWithPassword` | Abrir e examinar a fonte selecionada. Uma resposta bem-sucedida inclui a primeira `Page` e dados limitados de navegação. Um diretório criptografado pode exigir senha antes de abrir uma sessão. |
| `Browse` | Iniciar consulta ou continuá-la com identidade e deslocamento correspondentes. Cada comando retorna no máximo uma `Page`, limitada a 128 linhas e 64 KiB de nomes. |
| `FreezeSelection`, `Preflight` | Resolver seletores atuais em IDs físicos ou de navegação próprios em `Selected`, ou resumir resultados de metadados em `Preflight`. O instantâneo e o escopo selecionado são revalidados para a operação real. |
| `Preview` | Resolver uma entrada física. Em caso de sucesso, retornar bytes `Preview` limitados e com propriedade, junto com a identidade da entrada e sessão. |
| `CheckTarget`, `InspectInputs` | Retornar a contagem restante em `TargetChecked` depois que o runtime abre e limpa um espaço de sondagem. A verificação de criação usa sessão zero. `Inputs` informa metadados superficiais na ordem de entrada, sem varrer diretórios recursivamente. Nenhuma resposta concede permissão para confirmar. |
| `Extract`, `Create`, `Check`, `OpenEntry` | Validar a intenção e colocar uma tarefa na fila. `TaskAccepted` contém o ID atribuído. O runtime e a camada de E/S executam a autorização do destino e a confirmação. |
| `Update`, `Move`, `Export`, `PublishExport`, `ExportManifest` | Edições de arquivos compactados e publicação de dados entram na fila de tarefas pesadas. Após admitir `Update` ou `Move`, o serviço descarta o instantâneo atual para impedir que solicitações posteriores usem os antigos IDs de entrada. `ExportManifest` retorna metadados relativos de transferência como resposta comum. |
| `Snapshot`, `CloseSession` | Retornar o ID de sessão do arquivo atual ou zero. Fechar a sessão identificada descarta seu instantâneo de navegação, enquanto entradas de tarefas mantidas separadamente podem continuar ativas. |
| `cancel`, `cancel_task`, `decide_task`, `respond_password`, `release_export`, `shutdown` | Métodos de controle fora da fila comum de comandos. O cancelamento de solicitação tem ID próprio e `Finished`; controles de tarefa atuam sobre a tarefa existente. |

O serviço Rust expõe operações adicionais específicas da CLI, como `Catalog` e `SelectMembers`. A ponte valida tamanho da entrada e valores de enumeração antes do envio. Por exemplo, caminhos de criação são limitados a 4 KiB cada e 4 MiB no total. O runtime valida a sessão e a semântica da operação, enquanto a camada de E/S abre arquivos e obtém capacidades de saída. [Contrato do serviço runtime](../engineering/product-runtime.md), [Contrato da CLI](../../engineering/command-line.md), [Autorização de arquivos](file-security.md)

<a name="xdzip-event-delivery-and-backpressure"></a>

## Entrega de eventos e contrapressão

Uma solicitação comum admitida produz no máximo um resultado de dados seguido de um `Finished` sob entrega normal. Um erro é representado por `Finished` com código de erro. O controlador espera a conclusão bem-sucedida antes de publicar a página ou prévia correspondente. Um resultado substituído é descartado enquanto sua solicitação ainda termina. O caminho de índices grandes mantém até 1.024 linhas no cache Qt e solicita páginas ausentes. Ele congela uma seleção entre páginas antes de apresentar a confirmação de extração. [Indexação de arquivos grandes](../engineering/large-index.md), [Prévia de conteúdo](../engineering/content-preview.md)

`TaskAccepted` e `Finished` pertencem à entrega comum de solicitações. `TaskUpdate` usa espaços pendentes separados por tarefa e carrega o ID original de solicitação, ID da tarefa e ID da sessão de origem. Uma atualização pode chegar antes de `TaskAccepted` ou depois de `Finished`. O controlador correlaciona uma tarefa antecipada à solicitação original e encaminha atualizações posteriores para essa tarefa. Uma atualização posterior nunca conclui novamente a solicitação comum. Os espaços pendentes mantêm o instantâneo mais recente de uma tarefa. O progresso pode ser agrupado e o instantâneo terminal persiste até ser retirado pelo receptor. Solicitações de resposta aparecem nos instantâneos enquanto a tarefa aguarda resposta. [Estado das tarefas e cancelamento](task-scheduling.md)

- **Rust.** Por conexão, a admissão comum permite 128 solicitações pendentes e orçamento de 8 MiB para bytes de comandos na fila. O orçamento de conteúdo dos eventos comuns é 32 MiB. Solicitações aceitas reservam espaço para resultado e conclusão. Confirmações de cancelamento de solicitação têm fila limitada própria e o encerramento não precisa de espaço nela.
- **Qt.** A caixa mantém até 256 eventos, até 128 eventos de conteúdo e 32 MiB de conteúdo contabilizado. Se a capacidade acabar, o receptor espera a interface esvaziá-la. Uma única cadeia de notificação na fila processa até 32 eventos ou 4 ms por rodada. A notificação vai para a thread da interface e os eventos mantêm seus dados próprios enquanto aguardam.

<a name="xdzip-prompts-failures-and-shutdown"></a>

## Solicitações de resposta, falhas e encerramento

- Decisões de conflito usam `decide_task`. Senhas de tarefas usam `respond_password` e um ID distinto de solicitação de senha. Ambos validam a solicitação atual ainda sem resposta. A resposta de senha entra em armazenamento Rust zerado ao ser liberado e nunca aparece em resultado ou evento de progresso. `OpenWithPassword`, `Preview` e criação criptografada levam segredos de entrada específicos da operação que a ponte move para os respectivos tipos de senha Rust. [Tratamento de senhas](../engineering/password-extraction.md)
- Códigos estáveis descrevem resultados de admissão, solicitação e tarefa. Exemplos são `stale_session`, `stale_query`, `source_changed`, `unsafe_target`, `target_changed`, `disk_full` e `password_required`. O controlador traduz códigos para exibição. Uma alteração da fonte invalida o instantâneo de navegação relevante. `RuntimeFault` e desconexões inesperadas invalidam trabalho pendente da interface com `runtime_unavailable`; a perda de conexão pode impedir a chegada dos eventos terminais originais.
- `shutdown` fecha a admissão e cancela o trabalho aceito, incluindo tarefas aguardando entrada. O receptor Qt continua retirando eventos enquanto as threads Rust terminam. O receptor Rust espera sua conclusão antes de produzir `ShutdownReady`; depois a ponte produz `End`. Qt informa desconexão limpa após esvaziar a caixa. Se a conexão for destruída antes da hora, fecha a caixa e desperta o receptor sem aguardá-lo na thread da interface. [Encerramento de tarefas](task-scheduling.md), [Ciclo de vida do runtime](../engineering/product-runtime.md)

Para mudanças nessa fronteira, a [sonda da ponte](https://github.com/xiaodaozhi/xdzip/blob/main/tools/bridge-probe/README.md), os testes do serviço Rust e os [testes do controlador de desktop](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/tests/browser_test.cpp) cobrem camadas diferentes. Use o [guia de build](../../engineering/developer-build.md) para localizar os alvos relevantes.
