# Verificar arquivos compactados e gerenciar tarefas

[English](../../user-guide/checks-and-tasks.md) | [简体中文](../../zh-CN/user-guide/checks-and-tasks.md) | [繁體中文](../../zh-TW/user-guide/checks-and-tasks.md) | [日本語](../../ja-JP/user-guide/checks-and-tasks.md) | [한국어](../../ko-KR/user-guide/checks-and-tasks.md) | [Français](../../fr-FR/user-guide/checks-and-tasks.md) | [Deutsch](../../de-DE/user-guide/checks-and-tasks.md) | [Español](../../es-ES/user-guide/checks-and-tasks.md) | Português (Brasil) | [Русский](../../ru-RU/user-guide/checks-and-tasks.md)

<a name="xdzip-test-an-archive"></a>

## Verificar um arquivo compactado

1. Abra o arquivo compactado e selecione **Verificar** na barra de ferramentas.
2. Digite a senha se ela for solicitada e aguarde o resultado. A tarefa verifica o conteúdo compatível sem gravar arquivos extraídos.
3. Em caso de falha, veja o motivo e a entrada afetada, se for exibida. Alguns formatos fornecem somas de verificação ou autenticação. Outros verificam a estrutura e se os dados podem ser lidos. Um resultado bem-sucedido cobre apenas as verificações previstas pelo formato. [Verificação de integridade](../engineering/archive-checking.md)

Alguns arquivos compactados não podem ser verificados. Por exemplo, um ZIP indexado em disco permite extrair arquivos selecionados dentro dos limites da tarefa. O NSIS expõe apenas metadados do instalador. Consulte [limites de arquivos grandes](../engineering/large-index.md) e [formatos compatíveis](../../../README.pt-BR.md#xdzip-format-support).

<a name="xdzip-progress-cancellation-and-results"></a>

## Progresso, cancelamento e resultados

Abra o menu de tarefas para voltar a uma tarefa em andamento ou ver resultados. Fechar a aba de um arquivo compactado não cancela a tarefa em segundo plano. Pode ser necessário digitar uma senha ou resolver um conflito para que a operação continue.

Selecione **Cancelar** na janela da tarefa para solicitar a interrupção e aguarde o estado final. Arquivos já gravados em definitivo podem permanecer no destino. Expanda os detalhes para distinguir itens concluídos, ignorados e não concluídos. Use as quantidades e os motivos exibidos para decidir o que tentar novamente. Apagar o histórico de tarefas concluídas remove as entradas do histórico e mantém os arquivos no disco. [Janela de tarefas](../ux/task-windows.md)
