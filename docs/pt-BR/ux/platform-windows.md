<a name="xdzip-platform-windows-and-title-bars"></a>

# Janelas de plataforma e barras de título

[English](../../ux/platform-windows.md) | [简体中文](../../zh-CN/ux/platform-windows.md) | [繁體中文](../../zh-TW/ux/platform-windows.md) | [日本語](../../ja-JP/ux/platform-windows.md) | [한국어](../../ko-KR/ux/platform-windows.md) | [Français](../../fr-FR/ux/platform-windows.md) | [Deutsch](../../de-DE/ux/platform-windows.md) | [Español](../../es-ES/ux/platform-windows.md) | Português (Brasil) | [Русский](../../ru-RU/ux/platform-windows.md)

As interfaces de operação do xdzip usam janelas independentes de nível superior, não folhas anexadas à janela principal ou painéis sobrepostos desenhados dentro dela. Uma janela independente não implica processo separado nem entrada na barra de tarefas para cada janela auxiliar. Menus suspensos e dicas continuam como Popups leves e não são janelas de operação.

<a name="xdzip-window-contract"></a>

## Contrato das janelas

| Janela | Relação com a janela principal | Fechar / Escape | Minimizar / maximizar |
| --- | --- | --- | --- |
| Formulário de extração ou criação | `OperationWindow` não modal com proprietária transitória quando aberta pela navegação | Fechar o rascunho ou a confirmação sem enviar uma nova tarefa | Desativado / desativado |
| Aviso de conflito de tarefa | Aviso não modal pertencente ao hospedeiro da janela de operação. A tarefa aguarda uma decisão | Fechar o aviso. Reabri-lo pela tarefa ou cancelar explicitamente a tarefa ali | Desativado / desativado |
| Aviso de senha, saída ou reinício | Modal para o aplicativo. O seletor nativo de arquivos também é modal para o aplicativo e tem uma janela chamadora transitória | Senha, saída e reinício têm ações explícitas próprias de fechamento ou confirmação | Conforme os recursos do diálogo nativo |
| Progresso e resultado da tarefa | `TaskWindow` não modal. A navegação pode continuar se não houver outra janela modal ativa | Fechar os detalhes e reabri-los pela entrada da tarefa. Operações independentes mantêm acesso às tarefas ativas | Minimizar / desativado |
| Configurações e Sobre | Não modal. Uma entrada na bandeja pode abri-la sem uma janela de navegação proprietária | Fechar a visualização. Configurações já salvas permanecem ativas | Desativado / desativado |

Cancelar uma tarefa é um comando explícito independente de fechar sua janela de progresso. A conclusão em segundo plano não toma o foco da janela em primeiro plano; atualizam-se os detalhes abertos ou a entrada da tarefa. A saída do aplicativo continua seguindo a política de saída de tarefas; fechar a janela principal não pode significar diretamente fechar as janelas auxiliares.

<a name="xdzip-platform-appearance"></a>

## Aparência por plataforma

- macOS: usar barra de título decorada pelo sistema com controles vermelho, amarelo e verde no canto superior esquerdo; não substituir controles nativos por pontos coloridos personalizados. Janelas de operação de tamanho fixo mantêm o botão vermelho de fechar e desativam minimizar e ampliar quando indisponíveis; janelas de progresso podem ser minimizadas. A disponibilidade real depende dos recursos de janela AppKit/Qt e é aceita em dispositivo real. A janela principal mantém redimensionamento, minimização e tela cheia normais.
- Windows usa uma barra de título principal estendida à área cliente com botões QML, moldura nativa redimensionável e sombra DWM. Arrastar, redimensionar pelas bordas e encaixar usam recursos da plataforma. Janelas de operação independentes usam decorações nativas. Consulte os [projetos principais de Windows e Ubuntu](../../../design/hifi/sources/main.html?platform=windows).
- Ubuntu X11 usa controles integrados à direita e contorno arredondado no estado normal. Sessões com composição e definição da região de entrada também exibem uma sombra suave que permite a passagem de cliques. Janelas maximizadas e em tela cheia têm bordas retas e ocupam a área disponível. O sistema de janelas cuida do movimento e do redimensionamento. Janelas de operação independentes mantêm decorações do sistema. Outros backends fora do caminho X11 personalizado usam decorações normais. Wayland está fora do escopo suportado. Consulte o [projeto principal do Ubuntu](../../../design/hifi/sources/main.html?platform=linux).

Cada janela de operação tem título, borda, sombra e barra de título arrastável próprios; a janela principal atrás dela não é escurecida nem desfocada como uma página web. Na primeira abertura ela é centralizada na área disponível da tela da proprietária. Posições restauradas devem ser limitadas às telas visíveis. Barras de ferramentas e conteúdo não devem cobrir a barra de título.

<a name="xdzip-qt-quick-implementation-constraints"></a>

## Restrições da implementação Qt Quick

`OperationWindow` usa uma `Window` independente com `transientParent` opcional e `Qt.NonModal` por padrão. `PasswordWindow` e `OperationConfirmationWindow` usam explicitamente `Qt.ApplicationModal`. O seletor nativo usa a janela realmente chamadora como proprietária transitória e restaura sua ativação apenas enquanto ela estiver visível. Consulte o [registro de janelas](../../../specs/visual/window-registry.json) e as [janelas de tarefas](task-windows.md).

Barras de título personalizadas em Windows/Linux exigem recursos reais da plataforma para mover, redimensionar e abrir o menu do sistema, sem simular arraste alterando posições QML manualmente. `WindowController` gerencia ciclo de vida, propriedade, solicitações de fechamento e reabertura de visualizações de tarefa. O núcleo é dono das tarefas; destruir uma visualização não pode destruir uma tarefa ativa. Atualizações de estado de janelas entre threads usam somente eventos de ponte definidos.

<a name="xdzip-prototype-evidence-and-limits"></a>

## Evidências e limites do protótipo

O protótipo HTML simula aparência de janelas independentes, arraste da barra de título, interações bloqueantes ou não e comportamento de fechamento. O seletor superior “Platform” compara macOS, Windows e Linux e não é uma configuração do produto. O `dialog` do navegador permanece em uma página e não oferece janelas nativas reais, botões do sistema, minimização do sistema, barra de tarefas ou comportamento em múltiplas telas. A minimização de uma janela de tarefa é simulada ocultando os detalhes.

Os controles de plataforma da janela principal do protótipo ilustram a aparência. Ao voltar da confirmação de sobrescrita para a criação, o formulário de exemplo ainda é redefinido; a implementação do produto deve preservar o rascunho. Esses limites não comprovam uma aceitação nativa concluída.

<a name="xdzip-real-device-acceptance-criteria"></a>

## Critérios de aceitação em dispositivo real

1. Cada janela de operação é uma janela independente da plataforma corretamente associada à janela principal, sem folha ou sobreposição de página web.
2. A ordem dos botões coloridos ou à direita, seu estado desativado e os pedidos de fechamento correspondem aos recursos da plataforma e da janela; títulos e botões do sistema não se sobrepõem em diferentes tamanhos de fonte e escalas.
3. Arraste, limites de tela, múltiplas telas, escalas 125%/150%/200% e temas claro e escuro funcionam corretamente.
4. Fechar os detalhes deixa a tarefa em execução e sua entrada permite reabri-los. Fechar uma janela de conflito deixa a tarefa aguardando até reabrir o aviso ou cancelar explicitamente a tarefa.
5. Modalidade da proprietária, restauração de foco, navegação por teclado, títulos para leitores de tela e atalhos do sistema funcionam; janelas de arquivos compactados não relacionadas não são bloqueadas desnecessariamente.
6. Verificar separadamente o menu e encaixe do Windows e o Linux X11; Wayland está fora do suporte inicial. As conclusões Qt existentes em dispositivo real estão em [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5). Esta lista serve para verificar novamente itens afetados e não reinicia a aceitação já concluída.
