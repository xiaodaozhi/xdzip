<a name="xdzip-qt-quick-desktop-interface"></a>

# Interface de desktop com Qt Quick

[English](../../design/qt-quick-ui.md) | [简体中文](../../zh-CN/design/qt-quick-ui.md) | [繁體中文](../../zh-TW/design/qt-quick-ui.md) | [日本語](../../ja-JP/design/qt-quick-ui.md) | [한국어](../../ko-KR/design/qt-quick-ui.md) | [Français](../../fr-FR/design/qt-quick-ui.md) | [Deutsch](../../de-DE/design/qt-quick-ui.md) | [Español](../../es-ES/design/qt-quick-ui.md) | Português (Brasil) | [Русский](../../ru-RU/design/qt-quick-ui.md)

O desktop usa Qt Quick para apresentação e interação. Os controladores C++ encaminham as intenções do usuário e mantêm o estado da visualização. O ambiente de execução Rust interpreta arquivos compactados e controla as regras de tarefas e as permissões sobre arquivos. A [arquitetura do sistema](../../design/system-architecture.md) e o [contrato da ponte](qt-rust-bridge.md) descrevem as chamadas abaixo.

<a name="xdzip-windows-sessions-and-models"></a>

## Janelas, sessões e modelos

| Componente | Responsabilidade atual |
| --- | --- |
| `Browser.qml` e `SessionPage.qml` | Tela inicial, abas de arquivos compactados, controles de caminho e busca, navegação de pastas, lista ou grade, propriedades e ações. A página apresenta o estado atual do controlador e chama comandos. |
| `ApplicationSessions` | Encaminha pedidos de abertura a sessões de navegação independentes e coordena abas ou janelas, o acesso compartilhado às tarefas e a saída. O modo de abertura vale para pedidos futuros. Fechar uma página libera a visualização, e o controlador de uma tarefa pode continuar ativo. |
| `BrowserController` e `ArchiveTableModel` | Mantêm seleção, estado da lista e identidade do resultado. Resultados pequenos são aplicados como um modelo completo. Resultados grandes expõem um total lógico de linhas com até 1.024 linhas residentes e carregam as páginas ausentes. Linhas não carregadas ficam indisponíveis para ações. |
| `TaskListModel` e janelas de operações | Exibem instantâneos limitados das tarefas, progresso, resultados e a decisão pendente. A identidade composta do desktop encaminha comandos ao controlador responsável. |
| Configurações e adaptadores da plataforma | Mantêm aparência, idioma, tamanho do texto e comportamento do aplicativo. Os adaptadores nativos tratam área de notificação, ativação, seletores de arquivo, associações e encaminhamento de transferências de acordo com o sistema operacional. |

[Entrada de desktop](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/browser_main.cpp), [Página de sessão](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/ui/SessionPage.qml), [Controlador e modelos](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/BrowserController.h), [Sessões do aplicativo](../../engineering/application-sessions.md), [Configurações e idiomas](../ux/settings-localization.md)

<a name="xdzip-browsing-and-operation-flow"></a>

## Navegação e fluxo de operações

A abertura publica um instantâneo completo dos metadados. Cada nova solicitação de busca, ordenação ou navegação tem sua própria identidade de resultado. Uma página antiga não pode substituir o modelo atual. Durante uma consulta, a visualização mostra o resultado aplicado enquanto o controlador o mantiver válido. O transporte do núcleo envia até 128 linhas e 64 KiB de nomes por lote. [Dados de domínio](domain-data.md), [Indexação de arquivos compactados grandes](../engineering/large-index.md)

Abra um diretório para navegar até ele. Um clique duplo em uma entrada de arquivo comum compatível ou a ação equivalente inicia a abertura temporária. A visualização de propriedades oferece uma ação de prévia separada. As solicitações de conflito e senha preservam a identidade da operação atual. O acesso às tarefas mostra progresso e resultados mesmo após fechar a página de navegação. [Navegação](../user-guide/browse.md), [Abertura temporária](../engineering/archive-opening.md), [Prévia de conteúdo](../engineering/content-preview.md), [Janelas de tarefas](../ux/task-windows.md)

Soltar arquivos pode abrir arquivos compactados ou fornecer entradas para criação, dependendo do alvo. Movimentações internas permitidas e o arraste para fora usam preparação de tarefas e transferência conforme a plataforma. A plataforma determina as possibilidades de transferência, incluindo o conteúdo assíncrono e as gravações no destino. [Transferências de arquivos compactados](../../engineering/archive-transfers.md)

<a name="xdzip-shared-controls-and-native-boundaries"></a>

## Controles compartilhados e limites nativos

As telas de operações reutilizam `OperationButton`, `OperationFooterButton`, `OperationFrame`, `OperationSelect` e `OperationSegmentedGroup`. Componentes compartilhados definem medidas e interação. Os controles de seleção comuns encaminham ações de mouse, teclado e acessibilidade para o mesmo modelo de estado. Texto de interface e dados de arquivo seguem as regras de fontes e localização do [padrão de design](../../visual/design-standard.md). Os nomes dentro de arquivos compactados permanecem como dados e são exibidos de forma segura.

As janelas de operações independentes usam o registro de janelas e as regras de modalidade e fechamento de cada janela. O macOS usa controles nativos da barra de título. Windows e Linux usam seus adaptadores de sistema. O [contrato de janelas da plataforma](../ux/platform-windows.md) define a aceitação das janelas reais. O [contrato de integração de desktop](../ux/desktop-integration.md) descreve a área de notificação e a ativação.

<a name="xdzip-design-and-verification-scope"></a>

## Escopo de design e verificação

Código-fonte e testes descrevem o comportamento do produto. O HTML de alta fidelidade ilustra o layout e estados simulados para revisão. Alterações no foco nativo, nos menus, no IME, em ações de acessibilidade, na escala da tela e nos gerenciadores de arquivos exigem evidências próprias nas plataformas afetadas. Novas configurações de desktop e janelas de operações têm sua própria implementação e aceitação. [Fluxo visual](../visual/design-workflow.md), [Engenharia de qualidade](quality-engineering.md)
