<a name="xdzip-additional-interaction-and-copy-specification"></a>

# Especificação complementar de interação e textos

[English](../../ux/interaction-spec.md) | [简体中文](../../zh-CN/ux/interaction-spec.md) | [繁體中文（台灣）](../../zh-TW/ux/interaction-spec.md) | [日本語](../../ja-JP/ux/interaction-spec.md) | [한국어](../../ko-KR/ux/interaction-spec.md) | [Français](../../fr-FR/ux/interaction-spec.md) | [Deutsch](../../de-DE/ux/interaction-spec.md) | [Español](../../es-ES/ux/interaction-spec.md) | Português (Brasil) | [Русский](../../ru-RU/ux/interaction-spec.md)

A [especificação de estados](hifi-state-spec.md) completa o projeto da janela principal e da confirmação de extração com criação, conflitos, tarefas, resultados, configurações e comparações de plataformas. O [contrato da tela inicial](welcome-opening.md) descreve a abertura e o progresso. Os desenhos e parâmetros atuais seguem o [registro de janelas](../../../specs/visual/window-registry.json). Os contratos técnicos e de interação indicam os limites da implementação e da verificação visual nativa.

| Situação | Apresentação e ação | Continuação |
| --- | --- | --- |
| Primeira execução | Abrir ou criar arquivo compactado. Permitir soltar arquivos locais. | Evitar examinar todos os discos ou associar todos os formatos automaticamente. |
| Leitura do diretório | Quantidade lida e progresso indeterminado. Total desconhecido. | Permitir fechar a sessão. Exibir resultados parciais como incompletos. |
| Arquivo compactado vazio | “Este arquivo compactado está vazio”. Manter as propriedades. | Abrir outro. Desativar extração e explicar o motivo. |
| Sem resultados | Mostrar termo e alcance da busca. | Limpar a busca. Indicar que não há resultados sem apresentar o arquivo compactado como vazio. |
| Recurso sem suporte | Mostrar o motivo específico de formato, algoritmo ou criptografia. | Conservar metadados legíveis e interromper saídas parciais inseguras. |
| Criação | Entradas, formato, destino, nível e pré-verificação. | Mostrar erros perto dos campos. Preservar por padrão os arquivos existentes. |
| Conflito de nomes | Mostrar os dois itens, Ignorar, Renomear e Sobrescrever, e o alcance das decisões seguintes. | Fechar o aviso mantém a tarefa aguardando. Reabra ou cancele pela tarefa. |
| Tarefa concluída | Contagem de êxitos, itens ignorados, falhas e saídas gravadas. | A extração abre o destino quando solicitado. A criação pode abrir um arquivo compatível único ou o primeiro volume no aplicativo. Uma nova tentativa exige replanejamento. |
| Falha parcial | Preservar arquivos gravados e explicar falhas. | Manter os arquivos do usuário já escritos no destino. |
| Sair do aplicativo | Número de tarefas ativas. Esperar ou cancelar e sair. | O núcleo confirma a limpeza. A interface continua acessível durante o encerramento. |
| Configurações | Tema do sistema, claro ou escuro, densidade, tamanho, idioma, histórico e área de notificação. | Aplicar imediatamente após salvar. Em caso de erro, preservar os valores anteriores. A edição de cotas não é uma configuração atual. |
| Extensões de formato | O produto usa formatos integrados. Um [experimento de SDK externo](../engineering/sdk-trial.md) testa execução somente para leitura à parte. | O [registro de formatos](../engineering/format-registry.md) indica as operações disponíveis. |

O botão de extração acompanha a seleção. O menu oferece os dois escopos conforme os recursos disponíveis. A busca abrange o diretório atual e os descendentes. Selecionar um único arquivo compatível exibe automaticamente a [prévia de texto ou imagem](../engineering/content-preview.md) no painel de propriedades visível. Quando faltar largura para o painel, ele pode abrir em uma janela separada. Fontes maiores ampliam o conteúdo e a rolagem mantém os controles de confirmação acessíveis.

O texto de erro informa o que aconteceu e o que o usuário pode fazer. Os códigos ficam nos detalhes. Evite pilhas de chamadas ou termos de IPC para o público geral. Exiba nomes perigosos como texto seguro e trate mensagens de extensões sem interpretar HTML. Atalhos, IME, foco e leitores de tela seguem o [projeto Qt Quick](../design/qt-quick-ui.md).
