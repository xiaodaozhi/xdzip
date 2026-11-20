<a name="xdzip-xdzip-terminology-and-implementation-limits"></a>

# Termos e limites de implementação do XDZip

[English](../../design/glossary.md) | [简体中文](../../zh-CN/design/glossary.md) | [繁體中文](../../zh-TW/design/glossary.md) | [日本語](../../ja-JP/design/glossary.md) | [한국어](../../ko-KR/design/glossary.md) | [Français](../../fr-FR/design/glossary.md) | [Deutsch](../../de-DE/design/glossary.md) | [Español](../../es-ES/design/glossary.md) | Português (Brasil) | [Русский](../../ru-RU/design/glossary.md)

<a name="xdzip-1-terminology"></a>

## 1. Terminologia

| Termo | Definição |
| --- | --- |
| Arquivo compactado | Contêiner ou fluxo compactado compatível com sua origem autorizada. |
| Codec | Implementação que detecta, examina, lê ou cria um formato. |
| Sessão | Origem aberta, mecanismo, índice e contexto de navegação. Tarefas têm ciclo próprio. |
| Entrada | Registro real no arquivo compactado. Um diretório também pode ter um nó de exibição sintético. |
| SourceVersion | Impressão calculada na medida do possível de um arquivo ou volume admitido. |
| QuerySnapshot | Ordem de resultados atual e imutável, identificada por sessão, consulta, geração e revisão. |
| Geração | Substituição das solicitações da interface que impede resultados antigos de substituir resultados novos. |
| Operação preparada | Entrada de execução limitada com origem, seleção e opções pertinentes. |
| Tarefa | Ciclo de vida e resultado de uma operação aceita pela fila. |
| Autorização | Direito de acesso ao arquivo ou diretório mantido pelo host, distinto do texto do caminho. |
| Destino de saída | Interface de escrita com medição, cotas e controle transacional. |
| Processo ou thread de trabalho | Comandos e tarefas usam threads do host. Os processos de arquivos compactados e imagem são separados. |
| Gravação final | Publicação da saída temporária verificada pela operação autorizada de arquivo ou arquivo compactado. |
| Instantâneo da tarefa | Informações mais recentes de estado, progresso, solicitação ou resultado. Estados intermediários podem ser agrupados. |
| Capacidades | Interseção entre implementação, recursos do formato, plataforma e política. |
| Compactação sólida | Vários arquivos compartilham blocos compactados. A leitura seletiva pode exigir decodificar dados anteriores. |
| Primeiro quadro apresentado | Quadro realmente renderizado e apresentado. Um registro ui_ready não o substitui. |

<a name="xdzip-2-units-and-values"></a>

## 2. Unidades e valores

Memória e tamanho de arquivo usam KiB/MiB/GiB binários, com 1 MiB = 1024² bytes. Transporte e contabilidade mantêm bytes inteiros. O tempo usa ms ou s. A duração é medida com relógio monotônico. Registros de data preservam a representação de calendário e a precisão da origem quando disponíveis. A taxa é medida em MiB/s e informa se conta entrada compactada, dados decodificados ou saída.

100% de CPU de um núcleo quer dizer um núcleo lógico totalmente ocupado. Uma árvore de processos pode superar 100%. PSS, RSS, USS e ocupação física medem coisas diferentes. Tamanho do download compactado, tamanho instalado e tamanho do executável são registrados em separado.

<a name="xdzip-3-where-to-find-current-limits"></a>

## 3. Onde consultar os limites atuais

| Tópico | Referência de implementação |
| --- | --- |
| Solicitações, páginas e limite da caixa de mensagens | [Ponte Qt/Rust](qt-rust-bridge.md) |
| Admissão de tarefas, progresso e códigos de estado | [Agendamento](task-scheduling.md) |
| Limites de extração, autorização e publicação | [Segurança de arquivos](file-security.md) |
| Limites de metadados e índice ZIP em disco | [Índices de arquivos grandes](../engineering/large-index.md) |
| Limites de origens e volumes | [Abertura de arquivos compactados](../engineering/archive-opening.md) |
| Limites da prévia de texto e imagens | [Prévia do conteúdo](../engineering/content-preview.md) |
| Quadros, supervisão e limites de recursos | [Processo de formatos](../engineering/native-codecs.md) |
| Protocolo de extensão experimental | [Experimento de SDK](../engineering/sdk-trial.md) |
| Metas de desempenho | [Projeto de desempenho](performance.md) |

<a name="xdzip-4-versioning"></a>

## 4. Versões

A versão do aplicativo, a interface interna de compilação CXX, o protocolo externo e o esquema persistente têm versões independentes. A versão do produto não define sozinha a versão do protocolo do SDK. Veja o [projeto do SDK de formatos](codec-extension-sdk.md).
