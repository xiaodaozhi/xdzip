<a name="xdzip-main-window-welcome-page"></a>

# Página de boas-vindas da janela principal

[English](../../ux/welcome.md) | [简体中文](../../zh-CN/ux/welcome.md) | [繁體中文](../../zh-TW/ux/welcome.md) | [日本語](../../ja-JP/ux/welcome.md) | [한국어](../../ko-KR/ux/welcome.md) | [Français](../../fr-FR/ux/welcome.md) | [Deutsch](../../de-DE/ux/welcome.md) | [Español](../../es-ES/ux/welcome.md) | Português (Brasil) | [Русский](../../ru-RU/ux/welcome.md)

O design de alta fidelidade está confirmado. Qt implementa abertura e criação reais; a aceitação manual de desktop é registrada separadamente. Veja o design dos estados de boas-vindas e [welcome-reference.json](../../../specs/visual/welcome-reference.json). Cores e densidade da janela principal mantêm tokens e layout-scale existentes sem alterar padrões globais. Quatro PNGs são exportações de originais desenhados por código, não capturas do produto em execução.

<a name="xdzip-information-and-layout"></a>

## Informação e layout

O título da página inicial é «打包与解压，都简单一点。» em chinês e «Pack and unpack, a little simpler» em inglês; o catálogo brasileiro usa «Compacte e extraia com facilidade». O texto de apoio descreve navegar primeiro, extrair quando necessário e compactar para compartilhar.

A área de cliente padrão é 1024×622, mínima 832×544; área de título 48, cápsula 32 e barra inferior 32. O grupo de conteúdo de largura 608 é centralizado apenas horizontalmente e começa a 28 pixels lógicos fixos abaixo do topo disponível do corpo, sem centralização vertical. O título mede 30, ou 28 em janela estreita; descrição 13, título de ação 16 e descrição de cartão 12. Dois cartões de ação medem 296 de largura por 124 de altura, com espaço 16, preenchimento 20 e raio 8. São cartões de entrada da página de boas-vindas, não substitutos dos botões compartilhados de janelas de operação.

A barra de título reutiliza integralmente os componentes existentes da janela principal, mantendo marca, aba inicial, adicionar aba e configurações de aparência; boas-vindas não altera layout, dimensões, fontes ou ícones. O corpo não exibe árvore de arquivos vazia, cabeçalho de tabela, pesquisa ou barra de extração; o layout de navegação aparece após abrir um arquivo compactado. A ação principal é «Abrir arquivo compactado», a secundária «Criar arquivo compactado»; a área de soltar é seguida dos tipos suportados. Cada categoria mostra a contagem de extensões registradas. «Arquivos compactados» começa aberta. Ativar outra categoria troca a prévia; ativar a mesma a recolhe. Cor de destaque e peso marcam a categoria ativa. A prévia mostra extensões em uma única linha de etiquetas com preenchimento leve e contorno, usando a fonte normal da interface. Reticências indicam transbordamento; passar sobre a linha mostra os sufixos da categoria selecionada. «Associações de arquivos» abre os controles existentes nas configurações. O aviso de processamento local da barra inferior descreve onde ocorre o processamento, não garante segurança. A contagem de tarefas usa o histórico real; zero no design representa histórico vazio no primeiro uso.

Não mostrar arquivos recentes de exemplo, tamanhos fictícios ou capacidades de formatos ausentes do produto. Semáforos na área de título indicam apenas a posição da decoração macOS; a implementação usa recursos de janela da plataforma. Windows e Linux mantêm decorações próprias sem copiar os semáforos.

<a name="xdzip-interaction-design"></a>

## Design de interação

- A página pode aparecer na primeira inicialização, após fechar a última aba de arquivo ou após falha de abertura sem arquivo navegável. Tarefas em segundo plano continuam e não são reiniciadas.
- Abrir chama o seletor de arquivos do sistema; cancelar mantém a página de boas-vindas. Sucesso muda para a aba real do arquivo; falha mostra erro recuperável e não inventa uma lista bem-sucedida.
- Criar abre a janela de criação existente; cancelar mantém a página. Atalhos candidatos são ⌘O/⌘N no macOS e Ctrl+O/Ctrl+N no Windows/Linux. Verifique os atalhos reais antes da implementação; o design não afirma que foram integrados.
- Arrastar uma seleção admitida destaca a área com borda de destaque de 2 pixels lógicos e preenchimento de seleção. Sair restaura o estado normal. Veja [Integração Qt real](#xdzip-real-qt-integration) para a abertura efetiva.
- O cartão inteiro é clicável. A ordem de teclado após a barra de título é abrir, criar e tarefas. O foco de teclado não acrescenta contorno e segue a regra geral de perda de foco da janela. Passar e pressionar usam accentHover e accentPressed compartilhados, não apenas opacidade.
- Durante a abertura, o cartão substitui o título por indicador ocupado e «Abrindo…» e impede temporariamente aberturas repetidas e soltura; a aparência desativada usa accentDisabled e accentDisabledText. A possibilidade de executar a janela de criação simultaneamente segue regras do host; a página não altera a semântica das tarefas. Ocupado, desativado e erro são contratos de implementação não retratados nos quatro originais estáticos.
- Menus de configurações e tarefas mantêm largura 240, fonte 13 e altura mínima de linha 32, proporção 1:1 com fonte de corpo 13; a página não cria estilo de menu próprio. Fonte de cartão 16 é título de entrada, não referência de comparação de densidade de menu.

<a name="xdzip-verification-and-boundaries"></a>

## Verificação e limites

Cobrir temas claro/escuro, janela mínima, estados de arrastar e proporções de menu/barra de título, verificando DPR 1, 1,25 e 2 e zoom real da página no navegador. Captura reduzida não substitui layout e escala CSS não substitui zoom do navegador. Resultados, imagens e relatórios geométricos pertencem apenas a artefatos locais ou de CI; o estado é registrado no issue correspondente.

Qt usa regressões da página de boas-vindas e do navegador e abre arquivos pela entrada real do produto. Verificações automatizadas não substituem a validação nativa de arrastar e soltar, leitor de tela ou IME.

<a name="xdzip-real-qt-integration"></a>

## Integração Qt real

WelcomePage e WelcomeAction expressam apenas layout e intenção. A página aparece na inicialização e após fechar um arquivo; abertura bem-sucedida entra na sessão apropriada. Um arquivo vazio continua sendo estado válido de navegação. O carregamento inicial desativa abertura repetida e criação, mostra «Abrindo…» e permite cancelar o carregamento. Falha mantém página e erro real para nova seleção. Fechar rascunho de criação não cria tarefa. Listas de tarefas em segundo plano não são reiniciadas.

Abrir e atalhos de sistema reutilizam chooseArchive; criação reutiliza CreateWindow. O seletor nativo continua sendo janela do sistema e seu cancelamento não cria sessão. Qt encaminha pelo modo salvo de aba/janela, reutiliza preferencialmente uma página inicial vazia e ativa sessões existentes para arquivos duplicados.

Com o gerenciador de sessões de desktop, a página pode abrir até 128 arquivos locais em uma soltura. Cada item deve corresponder a um sufixo gerado para arrastar e soltar ou a um padrão suportado de nomes de volumes divididos. A entrada de desenvolvimento de sessão única aceita um item. Diretórios e URLs não locais são rejeitados. Essas verificações escolhem uma rota de abertura; o runtime Rust ainda identifica o formato real e valida a fonte. Use **Abrir arquivo compactado** para formatos com entrada apenas por abertura explícita. Veja o [registro de formatos](../engineering/format-registry.md).

Dimensões vêm de welcome-reference.json para Theme por generate-qml-theme.py. Texto maior amplia cartões e corpo; o corpo rola e traz automaticamente entradas com foco de teclado para a área visível. Áreas de status e título crescem independentemente sem Item.scale. Qt usa ícones PNG compartilhados, com o gráfico existente de traço escuro/claro para a ação principal; recursos de execução não são recortados dos originais.

As entradas de automação são tst_welcome.qml, welcomeOpensRealArchivesAndReturns em browser_test.cpp e scripts/check-qt-welcome.py. Workflows Qt sob demanda em três plataformas verificam abertura real, fechamento, recuperação de falha e 36 itens geométricos em DPR 1, 1,25 e 2, além de largura de menu 240 e fonte 13. Soltura nativa, seletor de arquivos, leitor de tela e IME são verificados conforme necessário; resultados fora de tela não são apresentados como operações de desktop não realizadas.

A página de boas-vindas altera apenas o conteúdo e reutiliza o mesmo componente Qt ArchiveTabs. `screens/welcome-titlebar-*.png` preserva o projeto da barra de título. A regressão compara a geometria da barra, abas e botão de aparência antes e depois de abrir um arquivo.

Estado de carregamento: a entrada de cancelamento da abertura inicial foi movida para uma sobreposição flutuante do corpo. Progresso desconhecido usa barra ativa sem porcentagens simuladas; veja o [contrato da sobreposição de carregamento](welcome-opening.md). Os bitmaps originais são mantidos; o design dinâmico oferece `opening=1`.

Erros e avisos aparecem abaixo das indicações de formato com a cor de perigo e o ícone de aviso compartilhados. Mostrar, quebrar, limpar ou substituir um erro não altera as coordenadas superiores do título e cartões; conteúdo longo rola no corpo. Reabrir a janela principal após fechá-la para a bandeja mostra somente uma página inicial e não cancela tarefas em segundo plano. PNGs antigos mantêm seus bytes históricos e não devem substituir o contrato de posicionamento superior pelo antigo layout centralizado.
