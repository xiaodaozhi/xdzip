<a name="xdzip-xdzip-product-requirements"></a>

# Requisitos do produto XDZip

[English](../../design/product-requirements.md) | [简体中文](../../zh-CN/design/product-requirements.md) | [繁體中文](../../zh-TW/design/product-requirements.md) | [日本語](../../ja-JP/design/product-requirements.md) | [한국어](../../ko-KR/design/product-requirements.md) | [Français](../../fr-FR/design/product-requirements.md) | [Deutsch](../../de-DE/design/product-requirements.md) | [Español](../../es-ES/design/product-requirements.md) | Português (Brasil) | [Русский](../../ru-RU/design/product-requirements.md)

O XDZip permite explorar arquivos compactados como pastas, trabalhar com o conteúdo selecionado e criar ou editar arquivos quando o formato permite. O desktop e a [CLI autônoma](../engineering/command-line.md) compartilham o ambiente de execução. O [README do projeto](../../../README.pt-BR.md) descreve os recursos e distribuições atuais. Os requisitos abaixo definem o comportamento a preservar e verificar.

<a name="xdzip-requirements-and-implementation"></a>

## Requisitos e implementação

| ID | Requisito | Comportamento ou escopo atual |
| --- | --- | --- |
| P01 | Explorar sem extrair | A abertura identifica a origem e conclui uma análise limitada antes de publicar a primeira página. Decodificar o conteúdo de uma entrada exige uma ação separada. |
| P02 | Navegar, buscar, ordenar e selecionar | Estão implementados IDs físicos estáveis, identidades de navegação separadas, consultas paginadas e seleção limitada. ZIP/ZIP64 comum de arquivo único tem um índice de metadados maior e limites de tarefa separados. |
| P03 | Extrair o escopo pretendido | O ambiente de execução fixa o escopo, valida os nomes e autoriza o destino. Trata conflitos e informa arquivos confirmados, itens ignorados, erros e cancelamento. |
| P04 | Criar arquivos compactados a partir de arquivos e pastas | As opções são validadas pelo registro de criação. A saída é preparada e verificada antes da publicação. Os formatos e opções disponíveis seguem o registro. |
| P05 | Respeitar as capacidades dos formatos | A interface combina o descritor de formato e o estado da sessão. O ambiente de execução valida cada solicitação. Leitura, prévia, verificação, criação, senhas e volumes têm limites próprios. |
| P06 | Limitar tarefas e permitir cancelamento | Metadados e tarefas pesadas usam filas separadas. Operações pesadas do desktop são executadas uma por vez. Uma tarefa aceita pode continuar após o fechamento da aba. |
| P07 | Limitar gravações às permissões e aos limites verificados | O host controla nomes, links, identidade, preparação, saída real e confirmação. O formato verifica a estrutura e a integridade aplicáveis. |
| P08 | Oferecer controles de desktop consistentes e acessíveis | QML e C++ oferecem controles compartilhados, navegação por teclado, foco e acessibilidade. IME, leitores de tela e exibição nativos exigem verificações próprias. |
| P09 | Limitar e medir recursos | O código aplica orçamentos independentes. Metas de desempenho e medições são acompanhadas separadamente. |
| P10 | Manter Qt fora da lógica de formatos | Traits Rust, roteamento explícito do registro e interfaces de processos dedicados permitem implementações integradas. Desktop e CLI usam o mesmo núcleo. |
| P11 | Proteger segredos e dados | Operações de arquivos compactados funcionam offline. Senhas ficam em memória de escopo limitado que pode ser zerada. Privacidade, diagnósticos e acesso explícito a atualizações têm contratos próprios. |
| P12 | Auditar a entrega | Usar canais de pacote definidos, entradas fixadas, manifestos, somas de verificação, licenças e testes de plataforma. Os resultados valem para a compilação e distribuição efetivamente verificadas. |
| P13 | Permitir extensões de terceiros apenas com confiança e permissões verificadas | O experimento XDP1 somente para leitura é implementado separadamente. Instalação e ativação de extensões no produto ainda estão em projeto. |
| P14 | Limitar prévia e abertura | Com o painel de propriedades visível, selecionar um arquivo compatível inicia uma prévia limitada de texto ou imagem. Abrir explicitamente uma entrada comum compatível usa uma cópia privada verificada e a associação do sistema. |
| P15 | Informar com precisão falhas e resultados incertos | Erros estáveis e resultados limitados cobrem mudanças de origem, falhas de disco, entradas ignoradas e limpeza. Recuperação automática após reinício continua sendo opção de projeto. |

<a name="xdzip-workflows-and-scope"></a>

## Fluxos e escopo

- **Abrir e examinar.** A solicitação chega ao coordenador de sessões. Uma análise bem-sucedida publica uma sessão. Busca e ordenação criam consultas novas. Clique duplo abre uma pasta ou um arquivo compatível por meio de cópia temporária. Com o painel de propriedades visível, selecionar um arquivo compatível inicia a prévia. [Navegação e prévia](../user-guide/browse.md), [Abertura temporária](../engineering/archive-opening.md)
- **Extrair.** “Extrair tudo” abrange o arquivo compactado inteiro. Para extrair apenas correspondências, selecione-as na busca e confira o escopo. A verificação prévia dos metadados informa problemas antes da autorização de gravação. A confirmação por arquivo preserva os arquivos concluídos após falha ou cancelamento posterior. [Extração](../../engineering/zip-safe-extraction.md), [Agendamento de tarefas](task-scheduling.md)
- **Criar, modificar ou verificar.** Criação e edições permitidas verificam a saída temporária antes da publicação. Verificar analisa um arquivo compatível conforme seu formato sem receber autorização de saída. As operações dependem do formato, da entrada e do perfil de origem. [Registro de formatos](../engineering/format-registry.md), [Transferências](../../engineering/archive-transfers.md), [Verificação](../engineering/archive-checking.md)

<a name="xdzip-planned-scope-and-verification"></a>

## Escopo planejado e verificação

Gerenciamento de extensões externas, recuperação persistente de tarefas e novos perfis de formato exigem projeto, implementação e verificação próprios. O número de versão por si só não comprova cobertura de formatos ou plataformas. Sincronização de arquivos compactados na nuvem, montagem de imagens de disco e um serviço de análise de malware ficam fora do produto. A [matriz de rastreabilidade](requirements-traceability.md) relaciona requisitos e verificações. A [política de suporte](../engineering/support-policy.md) e as notas de versão descrevem as compilações distribuídas.
