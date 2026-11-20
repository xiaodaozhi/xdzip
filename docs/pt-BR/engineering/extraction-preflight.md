<a name="xdzip-extraction-preflight-metadata-phase"></a>

# Pré-verificação de extração: fase de metadados

[English](../../engineering/extraction-preflight.md) | [简体中文](../../zh-CN/engineering/extraction-preflight.md) | [繁體中文（台灣）](../../zh-TW/engineering/extraction-preflight.md) | [日本語](../../ja-JP/engineering/extraction-preflight.md) | [한국어](../../ko-KR/engineering/extraction-preflight.md) | [Français](../../fr-FR/engineering/extraction-preflight.md) | [Deutsch](../../de-DE/engineering/extraction-preflight.md) | [Español](../../es-ES/engineering/extraction-preflight.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/extraction-preflight.md)

Este módulo fornece regras de pré-verificação de metadados. Ele não tem capacidade de autorizar destinos, decodificar, gravar, preparar temporariamente ou confirmar resultados.

<a name="xdzip-implementation-and-boundaries"></a>

## Implementação e limites

`xdzip-extract-plan` depende apenas de interfaces de domínio e metadados, sem Qt, analisadores de formato ou dependências extras de terceiros. A entrada é um retrato de sessão, um conjunto de EntryIds físicos e cancelamento; a saída é apenas um relatório limitado, não caminhos utilizáveis para gravação nem um indicador de aprovação de segurança. `xdzip-runtime` mantém o retrato; o Qt o chama pela ponte da thread de trabalho.

A seleção deve ser não vazia, é limitada a 128 IDs físicos e todos devem pertencer ao retrato atual. Selecionar um diretório expande todos os descendentes usando os limites de barra nos nomes originais, independentemente da pesquisa, paginação ou escopo do diretório atual. Selecionar diretório e filho conta o filho uma vez. Registros físicos com mesmo nome não são mesclados pelo caminho. Estatísticas de diretório contam apenas registros explícitos e não criam objetos de diretório sintéticos.

A pré-verificação usa o serviço de comandos e eventos, com admissão limitada e cancelamento independente. O ambiente verifica a identidade da origem aberta antes e depois da inspeção. Para ZIP indexado em disco, carrega do índice os registros físicos selecionados e descendentes necessários. Alterar seleção, consulta ou sessão limpa o relatório. Os resultados são estimativas dos metadados registrados. A pré-verificação nunca decodifica conteúdo de membros, reserva caminhos de destino ou autoriza gravações.

<a name="xdzip-rules-performed"></a>

## Regras executadas

- Nomes devem ser resolvidos como UTF-8, com no máximo 4 KiB, 64 componentes e 255 bytes por componente; nomes originais não são reparados nem reescritos.
- Rejeitar caminhos absolutos, componentes vazios, `.`, `..`, barras invertidas, dois-pontos, caracteres de controle e controle bidirecional, caracteres inválidos no Windows e espaços ou pontos finais em componentes.
- Rejeitar nomes comuns de dispositivos Windows, inclusive com extensão e dígitos COM/LPT sobrescritos, aplicando de forma conservadora um subconjunto portátil multiplataforma.
- Informar avisos de metadados existentes e usar os indicadores de capacidade de conteúdo e senha do registro. Um registro criptografado pode informar requisito de senha suportado ou criptografia não suportada. Um registro com tamanho conhecido e conteúdo não suportado também informa problema de método. A pré-verificação usa metadados e capacidades projetados pelo leitor; não descriptografa, autentica nem decodifica o conteúdo.
- Verificar nomes exatamente duplicados, colisões por comparação Unicode em minúsculas, aliases de diretórios pais com diferenças de maiúsculas e conflitos arquivo/diretório nas duas ordens de inserção. Um diretório pai implícito pode ser complementado por um explícito do mesmo nome; registros explícitos duplicados de diretório continuam em conflito.
- Informar tamanhos declarados por arquivo acima de 8 GiB, totais acima de 20 GiB e estouro de inteiro. Para registros ZIP, taxa de compressão declarada acima de 1000:1 solicita revisão. Tamanhos de diretórios não contam como bytes de saída.

Comparação Unicode em minúsculas não é conversão completa de caixa nem equivalência NFC/NFD do sistema de arquivos. Qualquer componente não ASCII acrescenta um aviso de que equivalências e colisões ainda precisam ser verificadas no sistema de arquivos de destino. Mesmo um relatório sem achados não autoriza saída.

<a name="xdzip-resources-and-failures"></a>

## Recursos e falhas

Verificar no máximo 100.000 registros do retrato; nomes selecionados ou expandidos totalizam no máximo 4 MiB. Chaves de comparação e grafias originais no índice de colisões totalizam no máximo 8 MiB, sem sobrecarga do contêiner. A profundidade de percurso é limitada e o cancelamento é verificado em cada registro externo. Relatórios contam todos os achados, mas no máximo 64 problemas representativos atravessam a ponte, com apenas IDs físicos e motivos estáveis, sem texto arbitrário de erro do arquivo compactado.

Seleção vazia, IDs externos, limites do índice ou cancelamento retornam erros, não relatórios parcialmente bem-sucedidos. Totais com estouro exibem “Indisponível” em vez de serem truncados para zero. Achados básicos são apenas informados; não sobrescrevem, renomeiam, ignoram ou pulam automaticamente.

Tamanhos descompactados desconhecidos informam o motivo estável `unknown_size`. `declared_bytes=None` significa total desconhecido e é diferente de `Some(0)` para arquivo vazio conhecido. Valores desconhecidos não inventam achados de tamanho total ou método desconhecido; estouro real ainda produz achado de limite total. Tarefas de saída rejeitam separadamente registros de tamanho desconhecido antes de decodificar, preparar ou interagir com senha.

<a name="xdzip-desktop-entry-point"></a>

## Entrada de desktop

Selecione entradas e clique em “Verificar metadados antes de extrair…”. Os resultados usam uma `OperationWindow` superior independente e não modal, com a janela principal como proprietária transitória e barra de título nativa preservada; o aplicativo não desenha os botões de janela do macOS. Fechar os resultados fecha apenas essa janela. Fechar a janela principal usa cancelamento cooperativo e não espera trabalhadores na thread gráfica.

<a name="xdzip-boundary-with-actual-extraction"></a>

## Limite com a extração real

A [extração segura real](zip-safe-extraction.md) implementa independentemente contabilização de saída, verificação CRC e estrutural, autorização de destino, novas verificações de caminho por plataforma, conflitos, preparação e confirmações de arquivos. Ela não se limita às 128 seleções da pré-verificação e suporta planos para 100.000 registros físicos e seleções de diretórios virtuais. Um relatório prévio não é reutilizado como permissão de gravação. Ao iniciar uma tarefa, é necessário obter novamente autorização de destino, revalidar origens e verificar conteúdo real conforme a [especificação de segurança de arquivos](../design/file-security.md). Falhas nativas e defeitos conhecidos são acompanhados pelos contratos e issues correspondentes.

<a name="xdzip-window-and-result-presentation"></a>

## Janela e apresentação dos resultados

A pré-verificação usa uma janela de operação compartilhada de 491 pixels lógicos de largura, corpo adaptável ao conteúdo, altura máxima da área cliente de 600, lista longa de problemas rolável e rodapé fixo acessível. Os limites de autorização das fases de dados e metadados não mudam. Consulte os [parâmetros de pré-verificação](../../../specs/visual/preflight-reference.json) para parâmetros e fonte de design.

A posição da explicação foi revisada em 2026-10-15: mover “Este relatório verifica apenas metadados…” para o fim do corpo rolável com texto explicativo de 12 pontos. O rodapé não contém mais a explicação; mantém somente a ação Voltar e tem a mesma altura de um rodapé compartilhado apenas de ações no mesmo tamanho de fonte, independentemente do comprimento do relatório.
