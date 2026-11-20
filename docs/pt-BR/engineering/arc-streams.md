<a name="xdzip-arc-containers-and-compressed-stream-profiles"></a>

# Contêineres ARC e perfis de fluxos compactados

[English](../../engineering/arc-streams.md) | [简体中文](../../zh-CN/engineering/arc-streams.md) | [繁體中文](../../zh-TW/engineering/arc-streams.md) | [日本語](../../ja-JP/engineering/arc-streams.md) | [한국어](../../ko-KR/engineering/arc-streams.md) | [Français](../../fr-FR/engineering/arc-streams.md) | [Deutsch](../../de-DE/engineering/arc-streams.md) | [Español](../../es-ES/engineering/arc-streams.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/arc-streams.md)

`ArcReader` implementa decodificação bruta limitada dos métodos ARC 3–9. Os métodos 1 e 2 são perfis Store do contêiner. `xdzip-codec-arc` e `ArcCodec` conectam estrutura completa e CRC-16 obrigatório ao runtime, CLI e navegação, verificação e extração no desktop. O [registro de formatos](../../../specs/engineering/archive-formats.json) declara esse escopo somente de leitura; criação, modificação, senhas e volumes não estão disponíveis.

<a name="xdzip-container-and-host-admission"></a>

## Admissão do contêiner e do host

O perfil admitido usa estrutura ARC 5.x comum; metadados adicionais ARC 6.x e extensões Spark não são alegados. Cada membro começa com `0x1a` e seu byte de método original. O método 1 usa 25 bytes; os demais métodos diferentes de zero usam o cabeçalho comum de 29 bytes. O campo de nome de treze bytes deve conter nome não vazio terminado em NUL. Todos os inteiros são little endian: comprimento compactado em 15, data/hora DOS em 19/21, CRC-16 obrigatório em 23 e comprimento decodificado comum em 25. O Store antigo herda o comprimento compactado. Ambos os métodos Store exigem comprimentos iguais. Intervalos compactados devem caber, o próximo marcador é obrigatório e o terminador exato de dois bytes `1a00` deve coincidir com o EOF físico. Não se admitem ressincronização, busca de prefixo, preenchimento ou arquivos vizinhos.

O contêiner preserva todas as duplicatas físicas e métodos desconhecidos. Leituras de metadados têm limite rígido de 64 MiB, até 100000 registros, nomes brutos limitados pelo chamador e limite de entrada física de 24 GiB. SHA-256 vincula cada byte de cabeçalho e o terminador, inclusive preenchimento de nome ignorado, ao reabrir um `Index` exato. Hashes de cabeçalho não autenticam conteúdo. Cada leitor tem contabilidades acumuladas de 24 GiB de entrada real e de bytes decodificados e 96 GiB de trabalho, reserva o trabalho em bytes do CRC por tabela antes da decodificação e encerra em qualquer falha sem reembolso de orçamento ou publicação de índice parcial.

O host mantém a prioridade de contêineres fortes e ZIP/SFX e valida integralmente propostas ARC fracas sob a contabilidade compartilhada de sondagem. Nomes são decodificados como campos completos antes da conversão de barras invertidas DOS, usando a decisão compartilhada e limitada de codificação do arquivo. Bytes originais e IDs de método permanecem disponíveis. Nomes suspeitos/não resolvidos e métodos desconhecidos permanecem visíveis sem autoridade sobre conteúdo. ARC não tem perfil admitido de diretório, link, restauração de modo ou objeto externo. O XOR antigo não tem indicador de criptografia; validação de fluxo, tamanho e CRC continua obrigatória, sem adivinhação de senha ou alternativa. Esse perfil não declara registro MIME dedicado no Linux.

A abertura de conteúdo reprojeta e compara entradas congeladas; IDs selecionados identificam linhas físicas e devem corresponder exatamente. Pré-verificação do runtime, identidade congelada da fonte, preparação privada, cancelamento, proteção de arquivos antigos e publicação final continuam determinantes. A verificação completa `supported_content` decodifica de fato cada membro comum admitido, valida tamanho exato e CRC-16 e falha em linhas sem suporte. Uma seleção física suportada ainda pode ser extraída ao lado de um vizinho sem suporte.

<a name="xdzip-sources-and-references"></a>

## Fontes e referências

A adaptação Unix original do ARC mantida por Howard Chu e Anibal Monsalve Salazar está fixada no [commit `70f0b93516d0ee222db5cf1241aad0db1b721c0d`](https://github.com/ani6al/arc/tree/70f0b93516d0ee222db5cf1241aad0db1b721c0d). Seu `Arcinfo` registra a descrição da estrutura ARC 5.12 de Robert Freed. Observações limitadas do formato usam `arcunp.c`, `arcusq.c` e `arclzw.c`. `LICENSE` e `COPYING` dos autores registram licença GPL. Algoritmos de referência são compilados sem mudanças apenas em build/toolchain com adaptador de teste próprio. Não são compilados nem implantados no produto.

A comparação independente usa nomarch 1.4 de Russell Marks, GPL versão 2 ou posterior. Seu pacote fonte original está disponível no [repositório fonte Debian](https://deb.debian.org/debian/pool/main/n/nomarch/), SHA-256 `fe20da34e0d3ba0cf6388701f44ac22224cf65130ddbc5fcbc27bc4949a6e1ad`, 22411 bytes, correspondente ao descritor de fonte do mantenedor. Não se afirma uma sala limpa estrita. O código do produto expressa campos do formato com primitivas próprias de bits, entrada, trabalho e alocação. O [registro de amostras próprias](../../../tests/fixtures/arc/README.md) distingue saída do codificador nativo, entradas de formato elaboradas, comparações independentes de bytes completos e limites das referências.

<a name="xdzip-method-scope"></a>

## Escopo dos métodos

| Método | Perfil do formato | Etapa posterior |
| --- | --- | --- |
| 1 | Estrutura obsoleta de 25 bytes com comprimento original implícito | Store |
| 2 | Estrutura comum de 29 bytes | Store |
| 3 | Bytes com sequências DLE `0x90` e DLE escapado | Nenhuma |
| 4 | Árvore squeeze com filhos assinados explícitos, códigos de bit menos significativo primeiro e símbolo EOF 256 | RLE do método 3 |
| 5 | Códigos fixos de 12 bits com bit mais significativo primeiro, dicionário hash original | Nenhuma |
| 6 | Mesmo dicionário hash original | RLE do método 3 |
| 7 | Códigos fixos de 12 bits, hash melhorado | RLE do método 3 |
| 8 | Byte inicial de largura máxima 12, LZW agrupado de 9–12 bits com bit menos significativo primeiro | RLE do método 3 |
| 9 | Sem cabeçalho de largura, LZW agrupado de 9–13 bits com bit menos significativo primeiro | Nenhuma |

Métodos desconhecidos, extensões Spark, senhas e estrutura SFX exigem admissão separada do contêiner. Um nome de algoritmo compartilhado não autoriza ordem de bits, dicionário, alinhamento, reinício ou etapa RLE de outro método.

<a name="xdzip-rle-and-squeeze-trees"></a>

## RLE e árvores squeeze

Um byte normal é emitido e passa a ser o histórico de repetição. DLE seguido de zero emite um DLE literal sem alterar o histórico. Uma contagem diferente de zero emite `count - 1` cópias do byte normal anterior; contagem sem histórico é inválida. O escape deve estar completo e o tamanho decodificado exato limita cada repetição. O estado RLE sobrevive a strings do dicionário e blocos de saída do chamador.

Squeeze começa com contagem de nós little endian, no máximo 256, seguida por dois filhos assinados de 16 bits por nó. Filhos não negativos designam nós internos; valores negativos codificam o símbolo `-child - 1`, de 0 a 256. A raiz é zero; a forma antiga sem nós tem filhos EOF implícitos. O grafo completo deve ser árvore enraizada sem ciclo, filho interno compartilhado, nó interno inalcançável ou aresta fora do intervalo, e EOF deve ser alcançável. Percorrer bits menos significativos primeiro até EOF. A entrada compactada deve terminar no mesmo byte desse terminador; bits finais não usados são ignorados. Decodificar até o tamanho declarado não substitui a verificação EOF.

<a name="xdzip-fixed-dictionaries"></a>

## Dicionários fixos

O dicionário de 4096 posições insere inicialmente sequências de bytes 0–255 com predecessor `0xffff`. Códigos identificam posições físicas de hash. O hash antigo usa os doze bits centrais do quadrado de `((prefix + suffix) mod 65536) | 0x0800`. O método 7 multiplica a soma de 16 bits por 15073 e toma os doze bits baixos. Essas constantes definem identificadores históricos do formato.

Colisões seguem a cadeia de posições existente, começam a busca 101 posições após o último elemento e depois pesquisam linearmente com retorno circular. Cada sondagem e expansão de prefixo é limitada e contabilizada. Cada nova string é a anterior mais o primeiro byte da atual. Um código ainda ausente só é admitido para a posição KwKwK exata recém-inserida. Dicionários cheios param de crescer. Códigos de doze bits colocam o meio byte alto primeiro entre bytes, diferente dos perfis LSB posteriores. O último meio byte não usado é ignorado; bytes de código ausentes e bytes completos extras são inválidos.

<a name="xdzip-dynamic-dictionaries"></a>

## Dicionários dinâmicos

O primeiro código deve ser um byte. Código 256 é CLEAR e 257 inicia adições sequenciais ao dicionário. A largura cresce nos limiares do dicionário até o máximo do método. Códigos são agrupados de oito; mudança de largura ou CLEAR termina o grupo antigo, consome seus bits de alinhamento não usados e inicia novo grupo. Os bits de alinhamento não são especificados. CLEAR reinicia a próxima posição e largura, exige literal seguinte e não pode tomar string anterior à reinicialização. Não são fornecidos código EOF ou bytes sintéticos.

Crescimento do dicionário, KwKwK, operação na largura máxima e reinicializações adaptativas são separados da etapa RLE. Entradas nativas de referência com LZW forçado estabelecem cada largura e reinicializações reais. Entradas CLEAR elaboradas cobrem independentemente cada uma das oito posições do grupo antigo.

<a name="xdzip-budgets-and-failures"></a>

## Orçamentos e falhas

Comprimentos compactado e decodificado têm cada um limite rígido de 24 GiB e o trabalho limite de 96 GiB. Limites menores do chamador prevalecem. A admissão completa de memória privada é 16 KiB para método 3, 32 KiB para método 4, 64 KiB para métodos 5–7 e 96 KiB para métodos 8–9. Esses limites incluem buffer de entrada de 8192 bytes, dicionários, expansão inversa, validação de grafo e estado.

Inicialização do dicionário, bits consumidos, arestas da árvore, sondagens hash, expansão de prefixo, entrega de pilha e trabalho de saída/repetição são contabilizados. `work_remaining()` inclui trabalho do construtor para uma contabilidade acumulada do contêiner. A sondagem cooperativa cobre trabalho dependente e independente da entrada. Leituras de saída são de até 8192 bytes. Entrada curta, erros tipados do host, esgotamento de cota, cancelamento, códigos/árvores inválidos, dados finais e divergência de tamanho decodificado encerram a instância; o primeiro erro mantém sua carga e leituras posteriores não retomam. Leitores concluídos retornam EOF estável sem gastar mais trabalho.

O CRC-16 obrigatório do conteúdo e a estrutura física completa pertencem ao contêiner ARC. Sucesso bruto sozinho não autentica conteúdo nem concede autoridade sobre sistema de arquivos, caminho, processo, arquivo vizinho ou publicação.
