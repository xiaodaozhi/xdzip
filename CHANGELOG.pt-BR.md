# Histórico de alterações

[English](CHANGELOG.md) | [简体中文](CHANGELOG.zh-CN.md) | [繁體中文](CHANGELOG.zh-TW.md) | [日本語](CHANGELOG.ja-JP.md) | [한국어](CHANGELOG.ko-KR.md) | [Français](CHANGELOG.fr-FR.md) | [Deutsch](CHANGELOG.de-DE.md) | [Español](CHANGELOG.es-ES.md) | Português (Brasil) | [Русский](CHANGELOG.ru-RU.md)

<a id="unreleased"></a>

## Ainda não lançado

- Preparação de compilações nativas para macOS Intel e atualizações autenticadas por arquitetura para versões futuras.

- Adicionar um SDK experimental independente e somente para leitura, com processamento limitado de CBOR, E/S controlada, duas implementações independentes do protocolo e uma verificação de isolamento que recusa execuções inseguras. As extensões de terceiros continuam desativadas no produto.

## 0.3.1

- Colocar chinês simplificado e chinês tradicional juntos no topo da seleção manual de idioma, com os rótulos “中文 (简体)” e “中文 (繁体)”.
- Melhorar as categorias de formatos e a paginação da página inicial e corrigir o layout da barra de ferramentas e das janelas de operação.
- Corrigir o aviso prematuro de conclusão de consultas paginadas e preservar a origem dos arquivos extraídos no Windows.
- Atualizar as dependências da prévia de imagens.

Consulte as [notas da versão](release-notes/0.3.1.md) e a [versão no GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.1) para saber sobre plataformas, verificações e limitações.

## 0.3.0

- Oferecer dez idiomas na interface de desktop, nos menus do sistema e na documentação do usuário. É possível seguir o idioma do sistema ou fazer a seleção manualmente.
- Ampliar a edição para mais formatos de arquivo compactado. As entradas compatíveis podem ser adicionadas, substituídas, movidas, renomeadas ou excluídas.
- Carregar automaticamente a prévia dos arquivos, com atualização sob demanda. Melhorar as ações sobre arquivos compactados, as associações no macOS e o encaminhamento de janelas e arquivos no Ubuntu.

Consulte as [notas da versão](release-notes/0.3.0.md) e a [publicação no GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.0) para ver as plataformas compatíveis, o escopo da verificação e as limitações conhecidas.

## 0.2.1

- Ampliar a navegação e a extração para formatos como LHA/LZH, ARJ, ARC, DEB, ASAR, Zstandard e LZ4. Adicionar a criação de LZH, imagens de dados ISO9660/Joliet e ZIP autoextraível para Windows x64.
- Ampliar a prévia para imagens raster comuns e um perfil restrito de SVG estático. Usar o suporte do sistema a HEIC/HEIF no macOS e gerar miniaturas em um processo privado com recursos limitados. A especificação de prévia descreve os limites por formato.
- Indexar até um milhão de registros de um ZIP/ZIP64 comum em um único arquivo, com índice em disco construído em fluxo. Carregar resultados grandes sob demanda e manter a seleção entre páginas.
- Adicionar ao cartão de propriedades a prévia sob demanda de texto simples e imagens PNG/JPEG, dentro dos limites definidos. Compartilhar o tratamento de senha, integridade e cancelamento.

Consulte as [notas da versão](release-notes/0.2.1.md) e a [publicação no GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.2.1) para ver as plataformas compatíveis, o escopo da verificação.

## 0.2.0

- Oferecer atualizações nativas do XDZip instalado no macOS, com conexão direta ou por proxy HTTP CONNECT ou SOCKS5 personalizado.
- Fornecer pacotes CLI independentes para Windows, macOS e Linux, com comandos de arquivamento, resultados JSON, registro explícito de comandos e atualizações verificadas por manifestos assinados.
- Aplicar os patches oficiais do QtNetwork para CVE-2026-116337 e CVE-2026-105946. Preservar os patches completos, sua procedência e as instruções de substituição.
- Mover os metadados da CLI incluída no macOS para os recursos comuns do aplicativo, cobertos pela assinatura de integridade do pacote.

Consulte as [notas da versão](release-notes/0.2.0.md) e a [publicação no GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.2.0) para ver as plataformas compatíveis, o escopo da verificação.

## 0.1.3

- Impedir que o Finder trate como aplicativo danificado a pasta de extração padrão que herda o nome de um arquivo terminado em `.app`.

Consulte as [notas da versão](release-notes/0.1.3.md) e a [publicação no GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.3) para ver as plataformas compatíveis, o escopo da verificação.

## 0.1.2

- Preservar a lista de arquivos durante a troca de diretório, a busca e a ordenação para reduzir a cintilação.
- Melhorar a explicação e as etapas manuais de instalação quando a instalação automática estiver indisponível no macOS.
- Corrigir no macOS a extração de links simbólicos ZIP seguros e relativos ao diretório pai. Corrigir também os avisos do ambiente de execução do Qt.

Consulte as [notas da versão](release-notes/0.1.2.md) e a [publicação no GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.2) para ver as plataformas compatíveis, o escopo da verificação.

## 0.1.0

- Publicar a primeira versão estável, com instaladores nativos ou pacotes portáteis para Windows, macOS e Linux e atualizações iniciadas pelo usuário em instalações elegíveis.
- Melhorar a rolagem nas configurações, a acessibilidade do estado de atualização e o suporte ao proxy do sistema. Corrigir o comportamento nativo ao passar o ponteiro.
- Corrigir a coleta de avisos de terceiros e padronizar as expressões de licença na SBOM do código-fonte e no grafo de dependências do ambiente de execução Rust da versão candidata.

Consulte as [notas da versão](release-notes/0.1.0.md) e a [publicação no GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.0) para ver as plataformas compatíveis, as limitações conhecidas e os materiais de lançamento.

## 0.1.0-alpha.9

- Remover três DLLs do compilador gráfico do SDK do Windows do instalador e do pacote portátil. As atualizações limpam as cópias incluídas em versões anteriores.
- Melhorar o acesso às configurações e atualizações. As descrições de tipos de arquivo no Windows acompanham o idioma do aplicativo.
- Corrigir a minimização durante as transições de tela cheia no macOS e várias associações de arquivos. Melhorar a inicialização e o carregamento de fontes.
- Corrigir a limpeza ao fechar arquivos compactados, as verificações de destino da extração e os avisos de tarefas.
- Simplificar a CI do dia a dia e executar sob demanda as compilações completas do aplicativo e as verificações de lançamento. Ampliar os materiais de licenciamento do código-fonte e de terceiros.

As [notas da versão](release-notes/0.1.0-alpha.9.md) descrevem os limites de plataforma e licenciamento desta versão preliminar experimental.

## 0.1.0-alpha.8

- Adicionar à tela Sobre a consulta de atualizações iniciada pelo usuário. Instalações elegíveis podem ser atualizadas após verificar assinatura, versão e plataforma. A versão pública anterior, alpha.6, exige atualização manual.
- Usar fontes do sistema e fontes alternativas da plataforma para nomes de arquivos compactados, nomes de arquivos e caminhos. Os textos fixos da interface mantêm a fonte do produto. Exibir mensagens distintas para pastas vazias, arquivos compactados vazios e buscas sem resultados.
- Remover a biblioteca de OpenGL por software dos pacotes de desenvolvimento do Windows. Manter o Direct3D 11 como padrão e o WARP disponível para renderização por software.
- Corrigir a contagem duplicada de tarefas ativas quando um registro de estado chega antes da aceitação da tarefa. Preservar a aceitação, o cancelamento e a limpeza ao sair.
- Detectar codificações antigas comuns de nomes de arquivos de forma consistente na navegação, no deslocamento entre pastas, na extração, na abertura temporária, na exportação e na edição. Priorizar declarações Unicode, manter a validação dos nomes originais e salvar nomes reescritos em UTF-8. Nomes curtos e codificações mistas podem continuar ambíguos.
- Retirar os critérios antigos de lançamento e os modelos de relatório. Alinhar as regras de contribuição, o resumo de prontidão e os geradores de materiais. Registrar a procedência e o licenciamento na lista de verificação de lançamento.
- Usar notas revisadas específicas para cada versão, com verificação da versão e do número da compilação. Interromper o processo se o rascunho diferir do arquivo revisado para que uma nova execução não sobrescreva os limites de aceitação registrados.
- Atualizar a documentação sobre o lançamento dedicado do macOS, a leitura de volumes divididos e o histórico de alterações.

## 0.1.0-alpha.6

- Limpar estados antigos do ponteiro após ocultar ou minimizar janelas, incluindo o lampejo vermelho do botão de fechar ao restaurar pela bandeja do Windows. Compartilhar a limpeza do ponteiro entre as janelas de navegação e de operação.
- Navegar e extrair volumes ZIP numerados e PKZIP. A leitura entre volumes, os deslocamentos ZIP64, as senhas e as verificações de integridade seguem o fluxo comum de extração. Todos os volumes são protegidos contra alterações na origem e sobrescrita.
- Abrir entradas comuns pelo sistema a partir de uma cópia temporária privada. Ocultar o progresso após a aceitação pelo sistema e manter falhas para nova tentativa. As cópias seguem uma política de retenção de sete dias e suas edições ficam fora do arquivo compactado.
- Corrigir a persistência do modo de abrir arquivos compactados. Adicionar rolagem quando as abas não couberem, uma lista de abas e reordenação por arraste. Usar peso normal nas abas inativas.
- Aceitar nomes longos do GNU em TAR de código-fonte do Git. Fixar os erros da tela inicial no topo do layout. Fechar uma janela limpa suas abas e mantém as tarefas em segundo plano. Atualizar o texto da contagem de tarefas na bandeja.

## 0.1.0-alpha.1

- Introduzir o epoch 1 do Debian para que os pacotes alpha atualizem os pacotes dev anteriores na ordenação de versões do sistema. Preservar a identidade dos arquivos de teste históricos.
- Preservar os estados normal, maximizado e de tela cheia durante diálogos, restauração pela bandeja e abertura externa de arquivos.
- Compartilhar um processo e uma bandeja por sessão de login. Encaminhar pedidos de ativação e de abertura em lote durante a inicialização, rejeitar solicitações expiradas e recuperar o estado residual após falhas.
- Adicionar abas ou janelas separadas com controladores de arquivos compactados e estados de navegação independentes. Manter tarefas e resultados após fechar uma página, encaminhá-los pelo identificador de sessão e tarefa e serializar a admissão de tarefas pesadas.
- Separar o número da prévia do número da compilação. Preparar seis tipos de pacote para as plataformas, manifestos imutáveis, somas de verificação, materiais de código-fonte e licença e procedimentos de publicação. As atualizações automáticas ainda estavam pendentes e o licenciamento do lançamento era acompanhado separadamente.
- Manter todas as entradas comuns selecionadas explicitamente para a criação, inclusive um destino existente ou um volume anterior. Preparar a saída em uma área privada e tratar conflitos na gravação final. Preservar um destino também usado como entrada e oferecer outro nome ou o cancelamento.
- Atualizar no mesmo lugar os itens do menu de tarefas pelo ID da tarefa para evitar cintilação ao mudar o progresso. Preservar o ponteiro, o destaque e o foco do teclado, alterando apenas os itens que entram ou saem do histórico.
- Reduzir as DMGs de desenvolvimento do macOS a binários Qt arm64, ao estilo Basic do Qt Quick e às dependências compartilhadas necessárias. Registrar arquitetura, estilo e tamanho e verificar o pacote final. Preservar o aplicativo e o SDK originais, com assinatura de integridade ad hoc local.
- No Windows, incluir apenas o estilo Qt Quick Basic fixado, remover os plug-ins de depuração QML e os metadados de tipos de desenvolvimento, e manter o estilo nativo e as alternativas gráficas. Corrigir o carregamento das traduções de sistema do Qt e incluir o mesmo runtime VC++ fixado no pacote portátil e no instalador por usuário. As auditorias rejeitam o uso do runtime da máquina hospedeira.
- Padronizar o nome do executável principal como xdzip em Windows, macOS e Linux. Alinhar atalhos, recursos, CI, recompilações e documentação. Manter a compatibilidade dos testes de atualização com compilações anteriores.
- Adicionar a extração de ZIP AES-256 de volume único com AE-1/AE-2 e Store/Deflate. Pedir a senha na tarefa e oferecer novas tentativas e cancelamento. Gravar os arquivos após a autenticação. Atualizar os projetos visuais, os textos, a cobertura de protocolo e as verificações de interoperabilidade.
- Adicionar limpeza seletiva das associações de “Abrir com” para cópias de desenvolvimento no macOS. Remover os registros novos após a verificação ou uma falha, mantendo o aplicativo principal, os arquivos e os padrões do sistema. Adicionar testes de limites e de limpeza após falhas.
- Concluir o trabalho de desenvolvimento e aceitação com a aprovação do responsável. Registrar dez verificações automatizadas, oito verificações manuais e a política de suporte da versão candidata congelada. Acompanhar procedência e licenciamento na lista de lançamento e nas issues. O lançamento formal ainda estava pendente.
- Adicionar a barra de menus nativa do macOS e os menus da bandeja nas três plataformas. Compartilhar comandos, foco de edição e estado de tarefas. Por padrão o aplicativo permanece na bandeja ao fechar a última janela. Isso pode ser desativado nas configurações. Ao sair ou reiniciar, manter a confirmação das tarefas e a limpeza. Adicionar textos bilíngues, ícones pequenos de modelo, projetos e testes de ciclo de vida.
- Unificar o versionamento em VERSION.json, nos metadados nativos e na tela Sobre, com textos bilíngues e cópia das informações de compilação. Continuar reunindo evidências de desempenho de todo o processo e de licenciamento, separando as verificações de desktop, máquina limpa e licenças.
- Alinhar dimensões da janela principal Qt, painéis, listas, textos, ícones, abas e prévia às medições do HTML e dos projetos congelados. Compartilhar as fontes e adicionar medições independentes de CI e capturas de ZIP reais. Permitir fechar abas e selecionar tudo na página atual, preservando estados desativados e datas desconhecidas.
- Concluir os menus comuns, dicas, estados dos botões e a navegação por caminho. O HTML recebe menus de contexto, suspensos e aninhados, abas de exemplo, grade, ordenação, seleção, toque e tarefas de verificação demonstrativas. O Qt recebe menus reais de seleção, cópia, visualização e diretório em modo somente leitura. Verificar 22 novas interações, 20 fluxos existentes e cinco grupos Qt locais. Manter as ações indisponíveis desativadas e as demonstrações identificadas.
- Adicionar ao HTML e ao Qt linhas de conexão da árvore de diretórios, contadores fixos, indicadores de ancestrais recolhidos e espaços compartilhados para ícones. Preservar o projeto principal anterior. Acompanhar separadamente avisos de limites, indicadores temporários de carregamento e acessibilidade manual.
- Entregar três pranchetas da árvore de diretórios, 26 recortes nos pixels originais, indicadores claros e escuros, especificações de leiaute e interação e HTML em arquivo único com recursos. A integração com HTML e Qt estava pendente. Preservar os originais e as fotos.
- Mover os projetos de alta fidelidade para design/hifi, com visões gerais das janelas principais e de operação, propostas antigas e hashes. Alinhar fontes dos recursos e links. O projeto principal escuro correspondente ainda estava pendente.
- Adicionar uma árvore de diretórios com limites, pastas implícitas, ordenação de todos os resultados e cabeçalhos de coluna operáveis por teclado. Paginar após ordenar e preservar os IDs dos registros duplicados. Adicionar limites de nós, profundidade e bytes, mensagens alternativas, testes de interação e 54 verificações visuais.
- Adicionar a base do runtime com dois handles cxx, comandos e eventos limitados, controles independentes de cancelamento e encerramento, fila Qt e barreiras de encerramento. Verificar a CI do produto em três plataformas, dez ciclos de vida por plataforma e a inicialização e saída nativas após a implantação no macOS e Windows. Oferecer uma compilação de desenvolvimento para Apple Silicon e um guia de verificação pontual.
- Repetir de forma assíncrona e limitada tentativas após breves disputas pelo bloqueio de admissão. Desativar comandos quando a conexão for perdida, informar argumentos inválidos e proteger a destruição durante o tratamento de eventos.
- Adicionar uma verificação prévia de extração restrita aos metadados e com limites. Expandir diretórios e verificar caminhos, conflitos e tamanhos, com uma janela nativa de resultados. A autoridade para gravar permanece com a extração.
- Aperfeiçoar o navegador em três painéis, com temas claro e escuro, detalhes, escopo por diretório, busca e seleção múltipla por ID físico. Adicionar testes de mouse e de filtragem.
- Adicionar navegação real de ZIP somente leitura, com adaptação Rust independente, varredura limitada, nomes duplicados e chineses, páginas de 128 registros e cancelamento e fechamento assíncronos. Registrar a adoção do rawzip e as capturas do Qt.
- Completar o progresso de tarefas do Qt com cartões de caminho, métricas em três colunas, progresso em gradiente e rodapé compartilhado conforme o projeto. Manter a reorganização com texto grande e os rótulos de demonstração. Capturar os estados claro, escuro e com texto a 200%.
- Estabelecer invariantes de identidade e paginação do domínio, prévias independentes das janelas de tarefas Qt, dicas compartilhadas e parâmetros de tema gerados. Adicionar verificações de leiaute, fechamento e CI sem tela.
- Adicionar diagnóstico independente da ponte Rust/cxx, compilações unificadas de CMake e Cargo, dependências fixadas e CI nativa. Verificar no Linux propriedade dos dados, erros, UTF-8 e encerramento.
- Adicionar diagnóstico independente do conjunto de ferramentas Qt Quick e registrar a execução sem tela no Linux e as compilações em caminhos com caracteres chineses e espaços. As verificações de três plataformas e cxx ainda estavam pendentes.
- Adicionar oito pranchetas de alta fidelidade e recursos para criação, conflitos, progresso, resultados, configurações e comparações de plataformas. Definir os fluxos de salvar, cancelar e recuperar.
- Entregar 24 recortes nos pixels originais, 27 tipos de ícones vetoriais claros e escuros e PNGs transparentes em várias densidades, com notas de origem e precisão.
- Alinhar ícones de tipos de arquivo, a geometria das listas e a coluna de tipos. Definir janelas independentes, barras de título e o fechamento nas três plataformas.
- Adicionar um protótipo HTML offline para navegação, seleção, extração, criação, tarefas e falhas. Vincular o protótipo aos requisitos e às verificações.
- Adotar Apache-2.0, adicionar a licença completa e atualizar os documentos de contribuição e lançamento.

<a id="pre-development-baseline-2026-10-11"></a>

### Referência anterior ao desenvolvimento (2026-10-11)

- Registrar o projeto de software, a rastreabilidade dos requisitos e as decisões de arquitetura.
- Escolher Qt Quick com uma camada C++ leve e Rust.
- Estabelecer a referência visual e parâmetros de tema para as exibições clara, escura e de extração.
- Adicionar documentação de contribuição, governança, segurança, engenharia, aceitação, riscos e pacotes de trabalho.
- O código do produto e as publicações de binários ainda estavam pendentes.
