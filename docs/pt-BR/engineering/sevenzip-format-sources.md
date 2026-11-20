<a name="xdzip-7z-format-sources-and-independent-implementation"></a>

# Fontes do formato 7z e implementação independente

[English](../../engineering/sevenzip-format-sources.md) | [简体中文](../../zh-CN/engineering/sevenzip-format-sources.md) | [繁體中文](../../zh-TW/engineering/sevenzip-format-sources.md) | [日本語](../../ja-JP/engineering/sevenzip-format-sources.md) | [한국어](../../ko-KR/engineering/sevenzip-format-sources.md) | [Français](../../fr-FR/engineering/sevenzip-format-sources.md) | [Deutsch](../../de-DE/engineering/sevenzip-format-sources.md) | [Español](../../es-ES/engineering/sevenzip-format-sources.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/sevenzip-format-sources.md)

Este documento registra fontes reutilizáveis e limites de implementação. A aceitação da migração e as capacidades restantes são acompanhadas na [issue #50](https://github.com/xiaodaozhi/xdzip/issues/50); o roteamento Rust fixado e a remoção da dependência oficial do produto não comprovam automaticamente aceitação multiplataforma, manual ou de entrega. As regras gerais de substituição estão no [contrato de implementação independente](../../engineering/independent-rust-codecs.md).

<a name="xdzip-specifications-and-reference-identity"></a>

## Especificações e identidade da referência

Campos do contêiner, inteiros de tamanho variável, grafos de métodos, propriedades e mapeamento de arquivos seguem os documentos oficiais `DOC/7zFormat.txt` e `DOC/Methods.txt`. A referência fixada é o [arquivo oficial de código 7-Zip 26.05](https://github.com/ip7z/7zip/releases/download/26.05/7z2605-src.tar.xz), SHA-256 `0b6080b8c3e7a579161d2d495224c0e51c467576cb4c7048b0ab485ef44cc22e`. Os SHA-256 dos dois documentos são respectivamente `8ec976b73ca88710286be1e991e49dbab485d9c0bf2410a80fa87e657850f82b` e `b16ea5fdc50345fa2cb031f06fe410cea0b089f74a4138ac8141b9e2d816ffc7`.

O fluxo de bits LZMA também consulta o [pacote oficial de especificação](https://www.7-zip.org/a/lzma-specification.7z) de Igor Pavlov de 2015-06-14, SHA-256 do arquivo `4529aca392d6b9598f72086e7d136b1e05d2863bf7a757836d3fd4f49e623b58`; o texto da especificação tem SHA-256 `6a4d441c2fa1d8d37b0189c7147a4b5424d5cdb122a7fc1fbebd022864ed54fb`. São lidos apenas texto de formato, matemática da especificação e pseudocódigo; o decodificador de referência do pacote não é usado como entrada de portabilidade para Rust. Baixar documentos ou amostras não concede nova licença de redistribuição.

Deflate genérico segue RFC 1951 e compartilha uma implementação com ZIP, GZIP e MSZIP. Deflate64 seleciona explicitamente nessa implementação regras estendidas: histórico de 64 KiB, códigos de distância 30/31 e código de comprimento 285; Deflate/MSZIP comuns não mudam. A seção de Deflate aprimorado de [PKWARE APPNOTE](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT) é apenas uma visão geral. Diferenças específicas do fluxo são complementadas por análise factual da referência fixada e verificação independente de leitura; a visão geral não deve ser apresentada como especificação completa. Matemática AES-CBC e vetores independentes seguem [NIST SP 800-38A](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf). Propriedades 7z, derivação de senha UTF-16LE e preenchimento zero são regras adicionais do formato e não podem ser inferidas de ZIP AES ou PKCS#7. O manual BZip2 e o nome de formato PPMd não especificam completamente fluxos históricos, reinício do modelo ou término; essas implementações não podem ser descritas como possíveis apenas pela leitura dos manuais.

<a name="xdzip-reference-analysis-and-original-coding"></a>

## Análise de referência e código original

Uma pessoa analista lê seletivamente os grupos abaixo no arquivo original fixado para complementar campos omitidos, estados matemáticos, reinícios por capacidade, término e relações de erros. As transferências contêm apenas fatos e requisitos de identificação, sem código de referência, reescrita linha a linha, layouts de classes ou matrizes, decomposição de funções, fluxo de controle copiado ou comentários traduzidos. Os algoritmos Rust são organizados independentemente por implementadores que não leram as respectivas implementações de referência; a pessoa analista não implementa um algoritmo que analisou. Quem implementa o contêiner também não leu sua implementação oficial e programa independentemente a partir da documentação e dos fatos transferidos.

| Tema da análise | Escopo de referência no arquivo fixado | Distinção de licença |
| --- | --- | --- |
| LZMA1/LZMA2 | `C/LzmaDec.c/.h`, `C/Lzma2Dec.c/.h`, partes de range, chunk/reset e propriedades dos codificadores e wrappers C++ Lzma/Lzma2 | Esses arquivos C declaram domínio público; wrappers C++ usam os termos padrão do arquivo |
| Detalhes do contêiner | Atributos externos, herança de somas de subfluxos e propriedades de arquivos em `CPP/7zip/Archive/7z/7zIn.cpp` | LGPL-2.1-or-later padrão do arquivo |
| 7z AES | `CPP/7zip/Crypto/7zAes.cpp/.h`, partes de comprimento de bloco, senha, preenchimento e conclusão de MyAes, FilterCoder e wrappers do codec 7z | LGPL-2.1-or-later padrão; a implementação AES subjacente não é lida nem portada |
| BZip2 | BZip2Decoder, Const, Crc, distinguindo tratamento de erros em wrappers 7z comum e bz2 independente | LGPL-2.1-or-later padrão; não atribuir licença de outra implementação pelo nome do formato |
| PPMd7 | Estados matemáticos, capacidade virtual e reinícios em `C/Ppmd7.c/.h`, `Ppmd7Dec.c`, `Ppmd7Enc.c`; restrições de propriedades e término no wrapper C++ Ppmd | Esses arquivos C declaram domínio público; wrappers C++ usam os termos padrão. Layouts de contexto, alocador e fluxo de controle não são copiados |
| Deflate64 | Partes do modo de 64 bits de `CPP/7zip/Compress/DeflateConst.h` e DeflateDecoder, Deflate64Register e admissão de propriedades 7z | LGPL-2.1-or-later padrão; transferência cobre apenas diferenças de formato em relação ao Deflate comum |
| ISA, Delta, Swap | Semântica escalar em `C/Bra.c`, `Bra86.c`, `Bra.h`, `Delta.c`, `SwapBytes.c` e wrappers C++ de registro de métodos, propriedades e cauda | Esses arquivos C declaram domínio público; wrappers C++ usam separadamente os termos padrão. Estratégia SIMD não é regra de formato |
| BCJ2 | Quatro fluxos e estado matemático em `C/Bcj2.c/.h`; wrappers C++ Bcj2Coder de término do decodificador e tamanho | Arquivos C declaram domínio público; wrappers C++ usam os termos padrão. O corpo do codificador não é analisado nem copiado |

Manter com as evidências de desenvolvimento as identidades do arquivo fixado e de seus arquivos, o escopo efetivamente lido e os resumos factuais transferidos. Reavaliar ao atualizar a referência; números de linha antigos não comprovam o comportamento de nova versão. Constantes da especificação, identificadores e relações matemáticas podem ser usados para implementar o formato; otimizações internas e comportamento historicamente indefinido não concedem permissões ao produto. Declarações de domínio público não se estendem ao arquivo inteiro, documentação, wrappers C++ ou outras amostras.

Isso não afirma uma sala limpa estrita nem risco jurídico zero. Fontes de terceiros, entregas históricas com o backend oficial e ferramentas independentes de teste continuam sujeitas aos respectivos termos. O produto atual não compila o backend oficial; consulte [licenças](licensing.md) e [código correspondente](../../engineering/source-delivery.md).

<a name="xdzip-interoperability-boundaries"></a>

## Limites de interoperabilidade

Sucesso entre nosso gravador e leitor não substitui uma referência independente. Os testes verificam separadamente saídas conhecidas de compressores independentes, leitura oficial dos nossos fluxos compactados, consumo de toda a entrada, término, CRC, corrupção, cotas, cancelamento e erros de fonte. Verificação dos componentes e aceitação de host, processo e plataformas são registradas separadamente.

Tolerâncias históricas de compatibilidade exigem confirmação individual. Por exemplo, um nó BZip2 7z pode aceitar bits não usados diferentes de zero no último byte e exatamente quatro bytes repetidos ao fim de um bloco sem byte adicional de contagem, mas ainda exige CRC correto, tamanho declarado e término completo do fluxo. Isso não permite bytes adicionais nem fluxos concatenados. Bits do último byte e bytes adicionais não são o mesmo tipo de preenchimento.

PPMd7 distingue perfis de término para comprimento externo conhecido e desconhecido. Após a saída exata, um nó 7z de comprimento conhecido verifica estado terminal do range e faixa compactada sem consumir outro símbolo END; o leitor compartilhado para comprimento desconhecido exige END. Reinícios de capacidade do modelo definidos pelo formato e admissão de memória do host são distintos: memória insuficiente não deve provocar um reinício de formato fabricado. Entradas de comparação independentes incluem fluxos com reinício por capacidade gerados pelo codificador C oficial fixado. O gerador e o código original de referência são apenas ferramentas locais de teste e não dependências de produção.
