<a name="xdzip-domain-model-and-archive-data"></a>

# Modelo de domínio e dados de arquivos compactados

[English](../../design/domain-data.md) | [简体中文](../../zh-CN/design/domain-data.md) | [繁體中文（台灣）](../../zh-TW/design/domain-data.md) | [日本語](../../ja-JP/design/domain-data.md) | [한국어](../../ko-KR/design/domain-data.md) | [Français](../../fr-FR/design/domain-data.md) | [Deutsch](../../de-DE/design/domain-data.md) | [Español](../../es-ES/design/domain-data.md) | Português (Brasil) | [Русский](../../ru-RU/design/domain-data.md)

O runtime é dono da sessão do arquivo compactado, do índice concluído e da última consulta dessa sessão. O Qt mantém uma visualização do resultado e a CLI pode ler registros físicos por seu comando de catálogo separado. Ambos enviam operações ao mesmo serviço Rust. [Arquitetura do sistema](system-architecture.md), [Ponte Qt/Rust](qt-rust-bridge.md)

<a name="xdzip-identity-and-source-version"></a>

## Identidade e versão da fonte

| Identidade ou dado | Representação e escopo atuais |
| --- | --- |
| `SessionId` | Identidade de runtime diferente de zero para um arquivo compactado aberto. Cada nova abertura bem-sucedida cria uma sessão. Caminhos de arquivos são usados para entrada e roteamento da interface. |
| `SourceVersion` | O runtime registra identidade, comprimento e datas disponíveis do arquivo regular de cada parte admitida. Verifica novamente o objeto aberto e o caminho correspondente nos limites relevantes de leitura e escrita. A detecção de mudanças é feita pelo melhor esforço. |
| `EntryId` | Identifica um registro físico do arquivo compactado dentro de uma sessão. Registros com nomes iguais mantêm IDs distintos. |
| Identidade de navegação | Representa um diretório indexado. Um diretório virtual sintetizado a partir dos descendentes pode aparecer na árvore ou lista sem ID de entrada física. |
| `QueryKey` | Combina sessão, ID da consulta, geração e revisão de snapshot. O deslocamento indica uma posição nesse resultado. |
| Identidade de solicitação, tarefa e aviso | O serviço atribui números de solicitação e tarefa dentro de uma conexão. Um aviso pertence a uma tarefa específica. A entrada compartilhada de tarefas desktop usa a identidade da sessão do controlador junto com o número da tarefa. |

Os tipos de domínio exigem identidades de arquivo compactado e consulta diferentes de zero. Na ponte nativa, valores Rust tipados são representados em dados CXX gerados. O QML recebe strings opacas para IDs que precisam de todos os 64 bits. Um número de linha é apenas uma posição em uma consulta específica. [Identidades do domínio](../../../crates/xdzip-domain/src/identity.rs), [Implementação da fonte](../../../crates/xdzip-runtime/src/source.rs), [Sessões do aplicativo](../engineering/application-sessions.md)

<a name="xdzip-physical-entries-and-output-names"></a>

## Entradas físicas e nomes de saída

`xdzip-codec-api::Entry` contém formato, identidade física, representações do nome, informações de tamanho, indicadores de tipo de arquivo, dados de modificação e campos de método e criptografia específicos do formato. Bytes brutos do nome, grafia UTF-8 decodificada opcional, decisão de codificação e texto de exibição escapado são armazenados separadamente. Para saída, o runtime valida o nome efetivo pela política de caminhos portáveis e pelo destino autorizado. Um nome não resolvido pode permanecer visível enquanto a extração desse registro ou diretório é recusada.

O tamanho original tem um indicador separado `size_known`, distinguindo arquivos de tamanho zero conhecido de tamanhos desconhecidos. Alguns contêineres comprimidos não têm tamanho compactado por entrada. Os valores de método são interpretados conforme o formato. Dados de tempo também preservam convenções de formato, como hora DOS local no ZIP; tempo ausente continua desconhecido. `payload_supported()` determina se o registro atual pode ser decodificado. A saída também depende da operação escolhida, do estado da fonte, do nome e das regras do sistema de arquivos. [Campos de entrada](../../../crates/xdzip-codec-api/src/lib.rs), [Tratamento de nomes](../engineering/filename-encoding.md), [Autorização de arquivos](file-security.md)

<a name="xdzip-index-query-and-page-lifecycle"></a>

## Ciclo de índice, consulta e página

O runtime abre a fonte e conclui sua varredura limitada antes de publicar o arquivo compactado e a primeira página. A varredura pode ser cancelada. Cauda danificada, senha ausente ou mudança da fonte impede a publicação de índice parcial. A navegação tem orçamento separado. Se a árvore de diretórios o exceder, ainda pode haver navegação na raiz e busca física. ZIP/ZIP64 comum de arquivo único pode conter até um milhão de registros pelo caminho privado em disco. Outros formatos e caminhos divididos mantêm seus próprios limites.

`Archive::query_page` aceita diretório, texto, modo de ordenação, deslocamento e identidade esperada da consulta. Uma nova consulta começa no deslocamento zero com uma nova geração. O runtime constrói uma ordem completa de resultados e mantém uma consulta atual imutável por arquivo compactado. As páginas resolvem identidades das linhas a partir dessa ordem. Buscas e ordenações mantêm as identidades originais dos registros. Cada modo agrupa diretórios antes de arquivos, desempata deterministicamente por identidade e coloca tamanhos desconhecidos após os conhecidos dentro de cada grupo.

O tipo genérico de domínio `PageRequest` valida até 256 linhas. O transporte do runtime desktop solicita até 128 linhas com orçamento de nomes de 64 KiB. O Qt confirma resultados pequenos integralmente e mantém no máximo 1.024 linhas residentes para resultados grandes. Respostas com sessão, consulta, geração, revisão ou deslocamento divergentes não podem substituir os dados atuais. [Implementação de consulta](../../../crates/xdzip-runtime/src/query.rs), [Listagem e ordenação](../../../crates/xdzip-runtime/src/listing.rs), [Índice de arquivos grandes](../engineering/large-index.md)

<a name="xdzip-selections-and-operation-input"></a>

## Seleções e entrada de operações

O modelo Qt armazena por ID as seleções de resultados carregados e, em resultados grandes, como intervalos limitados no resultado atual. Selecionar tudo cobre esse resultado. Uma tarefa pode distinguir o arquivo compactado inteiro de um conjunto explicitamente selecionado de resultados de busca. O Rust resolve seletores entre páginas pela chave completa da consulta usando `FreezeSelection` para confirmação de extração. A expansão de diretórios resolve descendentes reais e elimina IDs físicos duplicados. Exceder o limite da operação retorna erro. A CLI resolve caminhos de membros e IDs físicos por `SelectMembers`.

Extração, verificação e edição preparam trabalhos específicos com a fonte e os metadados retidos necessários. A criação recebe opções, depois varre e valida uma `InputTree` separada. A pré-verificação de metadados pode descrever problemas potenciais sem dar autoridade de saída a uma tarefa. O runtime verifica novamente as entradas e obtém a autorização pertinente antes de escrever. Quando um trabalho passa a possuir sua seleção, mudanças na interface não alteram seu escopo. [Modelo desktop](../../../apps/desktop/BrowserController.cpp), [Comandos do runtime](../../../crates/xdzip-runtime/src/service.rs), [Agendamento de tarefas](task-scheduling.md)

<a name="xdzip-results-and-data-lifetime"></a>

## Resultados e vida útil dos dados

| Dado | Vida útil |
| --- | --- |
| Arquivo compactado, consulta e índice em disco atuais | Pertencem à sessão do serviço. A substituição da consulta ou o fechamento da sessão libera dados privados associados, sujeitos aos recursos retidos independentemente pelos trabalhos. O índice de metadados em disco é reconstruído ao reabrir o arquivo compactado. |
| Snapshots de tarefas | O runtime publica snapshots limitados com estado real, contagens de bytes e itens, estado de limpeza e lista limitada de detalhes. O desktop mantém resultados disponíveis pelo controlador proprietário durante a sessão do aplicativo. |
| Preferências do usuário | Gerenciadas separadamente de índices e resultados de tarefas. Retenção e privacidade são documentadas nos contratos de configurações e privacidade. |
| Áreas de trabalho e diários | Preparação privada de saída e registros relativos de publicação apoiam limpeza e inspeção. Um reinício não repete uma gravação incompleta do arquivo compactado. |

[Campos de snapshot](../../../crates/xdzip-runtime/src/extraction/mod.rs), [Publicação de arquivos](file-security.md), [Configurações e localização](../ux/settings-localization.md), [Ciclo de dados e privacidade](../engineering/privacy.md)

<a name="xdzip-design-options-outside-the-current-implementation"></a>

## Opções de design fora da implementação atual

Publicação incremental de índices pesquisáveis, um plano reutilizável com revisão e token de início próprios e histórico persistente paginado de tarefas são opções de design. Os comandos atuais preparam e enfileiram seus próprios trabalhos limitados, mantêm uma consulta atual e retornam snapshots de tarefas limitados. A recuperação persistente também exigiria um design separado de validação da fonte e da saída.
