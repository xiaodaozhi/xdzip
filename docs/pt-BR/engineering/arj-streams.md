<a name="xdzip-arj-compressed-streams-and-container-admission"></a>

# Fluxos compactados ARJ e admissão do contêiner

[English](../../engineering/arj-streams.md) | [简体中文](../../zh-CN/engineering/arj-streams.md) | [繁體中文（台灣）](../../zh-TW/engineering/arj-streams.md) | [日本語](../../ja-JP/engineering/arj-streams.md) | [한국어](../../ko-KR/engineering/arj-streams.md) | [Français](../../fr-FR/engineering/arj-streams.md) | [Deutsch](../../de-DE/engineering/arj-streams.md) | [Español](../../es-ES/engineering/arj-streams.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/arj-streams.md)

`ArjReader` fornece decodificação bruta limitada dos métodos ARJ 1–4. Store pertence ao contêiner, não a esse leitor. `xdzip-codec-arj` fornece estrutura física completa e entrega de membros verificada por CRC em fonte mantida pelo host. `ArjCodec` conecta perfis admitidos ao runtime, CLI e navegação, verificação e extração no desktop. O [registro de formatos](../../../specs/engineering/archive-formats.json) declara o limite somente de leitura; criação, regravação, senhas e volumes não estão disponíveis.

<a name="xdzip-host-admission-and-names"></a>

## Admissão do host e nomes

Uma proposta de cabeçalho principal de doze bytes nunca publica formato ou linhas. O runtime primeiro preserva a prioridade de contêineres fortes validados e ZIP/SFX e depois varre completamente os metadados ARJ propostos sob a contabilidade compartilhada de entrada de sondagens fracas. Um marcador TAR bruto não validado é desambiguado por admissão ARJ completa. Hipóteses inválidas não publicam nada; erros tipados do host, cancelamento e cotas continuam determinantes.

O adaptador preserva ordinais físicos de membros, bytes originais de método e nomes brutos. Nomes com indicador ANSI e nomes OEM têm grupos separados de codificação que compartilham limite de 1 MiB para amostragem de campos completos, sem seleção pela localidade ambiente. Decodificar o nome multibyte inteiro antes de interpretar separadores. Somente hosts semelhantes a DOS sem indicador de caminho portátil convertem barras invertidas em barras normais. Barras invertidas literais portáteis ou Unix continuam suspeitas. Diretórios explícitos recebem barra final; nomes suspeitos ou não resolvidos, métodos desconhecidos e perfis de objetos sem suporte permanecem visíveis sem autoridade sobre conteúdo. Nomes decodificados são limitados a 4.096 bytes e alocações mantidas de exibição/decodificação a 16 MiB, ou limites menores do chamador. Nenhum atributo é restaurado.

A abertura do conteúdo reprojeta e compara o conjunto completo de entradas congeladas antes da entrega. Cada registro escolhido deve corresponder exatamente à linha física; seleções sem suporte e qualquer falha de decodificação invalidam a instância de conteúdo. Verificações de identidade do host, pré-verificação, preparação privada, cancelamento e publicação final mantêm a semântica comum do runtime. Verificações `supported_content` decodificam integralmente todos os membros comuns admitidos e verificam tamanhos exatos e CRC-32 obrigatório. Linhas sem suporte impedem sucesso completo. Texto antigo é entregue como bytes do arquivo compactado cobertos pelo CRC, sem conversão de quebras de linha pelo host.

<a name="xdzip-sources-and-reference-boundary"></a>

## Fontes e limite das referências

O [projeto ARJ de código aberto](https://arj.sourceforge.net/) fornece a [fonte 3.10.22](https://arj.sourceforge.net/files/arj-3.10.22.tar.gz), SHA-256 `589e4c9bccc8669e7b6d8d6fcd64e01f6a2c21fe10aad56a83304ecc3b96a7db`, e seu [pacote Win32 3.10](https://arj.sourceforge.net/files/arjw_310.exe), SHA-256 `e5cc5d8f82c5f08246e1c0a8c5be77e5f796c2dd30f987e6bbb27a2c1df1e371`. A fonte inclui o texto GNU GPL versão 2. Observações limitadas do formato usam `defines.h`, `decode.c` e `arj_arcv.c`; a referência nativa permanece sem modificações e sob seus termos originais, apenas em build/toolchain.

A comparação independente usa o [7-Zip](https://www.7-zip.org/) oficial, com observações limitadas do seu [manipulador ARJ](https://github.com/ip7z/7zip/blob/main/CPP/7zip/Archive/ArjHandler.cpp), blob Git `da890f279d35c00bc61d332ce55f4d96ecd6d373`. Não se afirma uma sala limpa estrita. A implementação Rust reutiliza primitivas próprias de bits, prefixos e LZ e expressa diretamente os campos unários rápidos. Fontes de referência, binários e mecanismos de compressão não entram nas dependências do produto. Identidades exatas de componentes e entradas próprias são registradas com as [amostras](../../../tests/fixtures/arj/README.md).

<a name="xdzip-static-methods-13"></a>

## Métodos estáticos 1–3

Os métodos 1–3 compartilham o modelo estático Huffman/LZ, embora codificadores nativos possam escolher buscas diferentes. Suas contagens de comandos de bloco de 16 bits, alfabeto de comprimentos de 19 símbolos, alfabeto literal/comprimento de 510 símbolos, alfabeto de posições de 17 símbolos e contagem de posições de cinco bits reutilizam a implementação limitada de prefixos LZH por um perfil ARJ explícito. Não adquirem campos antigos de árvore LH3 nem identificador público de método LHA. Contagens de bloco zero não são admitidas.

Símbolos literais são 0–255; os demais resultam em comprimentos `symbol - 253`, de 3 a 256. Símbolo de posição zero dá deslocamento zero; caso contrário, o deslocamento é `2^(symbol - 1)` mais a mesma quantidade de bits baixos. A distância real é deslocamento mais um, no máximo 26624, e deve referenciar bytes já emitidos. Um anel de 32 KiB implementa o histórico lógico limitado; sua alocação maior não amplia a admissão. Espaços iniciais de LHA permanecem separados e não podem inicializar a saída ARJ. Retorno circular do dicionário e cópias sobrepostas mantêm semântica sequencial por byte. Árvores canônicas completas, campos de símbolo único, sequências de zeros e profundidade de código de 16 bits mantêm a validação e contabilidade de trabalho do prefixo compartilhado.

<a name="xdzip-fast-method-4"></a>

## Método rápido 4

O fluxo usa o bit mais significativo primeiro. Um campo de comprimento conta de 0 a 7 bits iniciais um, com parada em zero exceto no máximo. Ler essa quantidade de bits baixos e somar `2^count - 1`. Zero indica literal de oito bits; valor diferente de zero indica cópia do valor mais dois, de 3 a 256 bytes. O ponteiro conta de forma semelhante de 0 a 4 bits iniciais um, com largura baixa `9 + count` e base `512 * (2^count - 1)`. Deslocamento mais um dá distância máxima 15872. Cada referência deve apontar para histórico já emitido, usando anel de 16 KiB com sobreposição e retorno circular; reparo de histórico inicial não é permitido.

<a name="xdzip-limits-and-failures"></a>

## Limites e falhas

Perfis estáticos admitem 32 KiB de histórico mais 64 KiB para entrada/tabelas/área temporária. Decodificação rápida admite 16 KiB de histórico mais 32 KiB para entrada/estado/área temporária. Tamanhos compactado e decodificado têm cada um limite rígido de 24 GiB, trabalho de 96 GiB; orçamentos menores do chamador prevalecem. Leituras de saída têm até 8192 bytes. Operações de prefixo, trabalho de histórico e cópias independentes da fonte mantêm sondagem cooperativa. `work_remaining()` apoia a contabilidade cumulativa de trabalho real do contêiner.

Comprimentos declarados não podem ser excedidos. Bytes compactados lógicos devem terminar exatamente; preenchimento final menor que um byte é ignorado, bytes completos adicionais ou entrada ausente falham e nenhum byte zero EOF é fabricado. Apenas `Interrupted` sem carga adicional é tentado novamente. Erros tipados preservam a primeira carga e encerram; qualquer erro de fonte, dados malformados, cota ou cancelamento invalida o leitor. Leitores brutos não têm autoridade sobre caminho, credenciais, execução, publicação no sistema de arquivos ou CRC.

<a name="xdzip-bounded-physical-containers"></a>

## Contêineres físicos limitados

`inspect` valida arquivo compactado no deslocamento zero, largura do primeiro cabeçalho principal de 30/34, larguras de arquivo 30/34/46 e blocos-base completos de até 2600 bytes. Nomes e comentários terminam exatamente, o campo de posição do nome não autoriza truncamento e todos os CRC-32 de base e estendidos devem passar. Cada registro admite até 256 blocos estendidos com comprimentos de 16 bits. Atributos opacos de arquivo são validados e incluídos na identidade de metadados, sem aplicação ou execução. O terminador exato `60 ea 00 00` deve encerrar a fonte física; não se infere prefixo SFX, preenchimento ou cauda de recuperação.

Leituras de metadados são limitadas a 64 MiB, entradas físicas a 100000, nomes brutos a 4096 bytes cada e 16 MiB no total, mantendo limites menores do chamador. Nenhuma falha posterior publica índice parcial. SHA-256 cobre todos os metadados físicos e terminador, inclusive comentários/modificadores/atributos ignorados; `Reader::open` varre novamente e compara isso, cada registro e o comprimento físico antes da saída. Erros da fonte permanecem terminais em vez de se tornarem hipóteses de dados malformados. Horários de host Unix usam apenas o calendário opcional do host; projeções ausentes tornam-se zero e erros de callback encerram.

O perfil de membro admite versões exigidas até 11, hosts DOS/Unix/OS2/Win95/WinNT, métodos 0–4 binários ou texto antigo sem criptografia e diretórios Store vazios. Métodos sem suporte, versões futuras, tipos especiais, modos especiais Unix e campos de criptografia/continuação/capítulo permanecem explícitos e não podem ser decodificados. Perfis principais de volume/proteção/segurança/nome duplo/criptografia/capítulo e layouts de arquivo com nome duplo não são admitidos. Modos e atributos nunca autorizam permissões ou criação de arquivos especiais. Texto antigo é entregue como bytes cobertos pelo tamanho e CRC do arquivo, como no 7-Zip; ARJ Win32 nativo expande LF para CRLF nesse perfil. Não há conversão de quebra de linha dependente do host.

Um leitor mantém 24 GiB de entrada real acumulada, inclusive nova varredura de metadados, 24 GiB de saída decodificada e 96 GiB de trabalho. O trabalho de passos de bits CRC da saída declarada é reservado antes de emprestar o restante ao decodificador com estado. Membros bem-sucedidos contabilizam exatamente essa reserva mais o trabalho real de construção/modelo/Store; falha encerra sem reembolso. A contabilidade do coletor do chamador inclui construção e não começa após a inicialização. Todas as leituras continuam limitadas a 8192 bytes, instantâneos e índices físicos são verificados e qualquer falha de conteúdo/CRC/coletor/cancelamento/cota invalida a instância.

<a name="xdzip-required-container-and-host-admission"></a>

## Admissão exigida de contêiner e host

Cabeçalhos principal e de arquivo ARJ, layouts limitados do primeiro cabeçalho, CRC-32, cadeias de cabeçalhos estendidos, intervalos compactados exatos e marcador final de comprimento zero devem ser validados antes de publicar índice completo. IDs físicos preservam registros duplicados. Nomes, páginas de código, separadores, diretórios, tipos de texto antigo ou binário, objetos especiais e cada método desconhecido exigem projeção e admissão de conteúdo explícitas; um marcador reconhecido não autoriza conteúdo.

Cada CRC de arquivo declarado deve passar após decodificação completa. Cabeçalhos e campos opcionais de segurança do arquivo não implicam autenticidade criptográfica. Senhas, métodos criptografados, volumes, entradas continuadas, capítulos, caudas de recuperação/segurança e envoltórias autoextraíveis exigem perfis e evidências separados. Nenhum script, comando automático, atributo externo, caminho vizinho ou indicador executável concede autoridade. Congelamento da fonte, extração selecionada, orçamentos acumulados de entrada/trabalho/saída, preparação privada, cancelamento, proteção de arquivos antigos e publicação final seguem o [contrato de extração](zip-safe-extraction.md).

O gerador de amostras cria somente entradas binárias próprias, registra métodos solicitados e reais e lê independentemente a saída completa com ARJ nativo e 7-Zip. Uma alternativa nativa Store permanece Store. Ele normaliza apenas horários não criptografados, modificador não usado derivado do tempo e CRCs de cabeçalho correspondentes; bytes compactados e CRCs de conteúdo não mudam. Entradas nativas vazias produzem membros Store, não evidência de fluxo compactado vazio codificado. A admissão do produto também exige automação real de runtime/CLI/Qt e de candidatos/layouts aplicáveis conforme o [escopo de QA](../../../developers/qa/README.md).
