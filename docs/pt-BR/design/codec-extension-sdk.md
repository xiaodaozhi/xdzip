<a name="xdzip-format-adapters-and-the-extension-trial"></a>

# Adaptadores de formato e teste de extensões

[English](../../design/codec-extension-sdk.md) | [简体中文](../../zh-CN/design/codec-extension-sdk.md) | [繁體中文](../../zh-TW/design/codec-extension-sdk.md) | [日本語](../../ja-JP/design/codec-extension-sdk.md) | [한국어](../../ko-KR/design/codec-extension-sdk.md) | [Français](../../fr-FR/design/codec-extension-sdk.md) | [Deutsch](../../de-DE/design/codec-extension-sdk.md) | [Español](../../es-ES/design/codec-extension-sdk.md) | Português (Brasil) | [Русский](../../ru-RU/design/codec-extension-sdk.md)

<a name="xdzip-three-execution-paths"></a>

## Três caminhos de execução

| Caminho | Implementação e disponibilidade |
| --- | --- |
| Adaptadores integrados no host | Desktop e CLI usam o mesmo registro fixo de formatos e despacho de execução. Implementações Rust de ZIP, TAR, filtros de fluxo e contêineres suportados como DEB e ASAR são chamadas no processo do host. |
| Processo de arquivos compactados incluído | `xdzip-codec-native` no host inicia `xdzip-native-codec`. Sua camada de protocolo C++ chama os backends Rust de 7z, RAR, CAB e mídias ópticas por `xdzip-codec-bridge`. O mesmo caminho trata criação e regravação 7z aplicáveis. A criação ISO usa seu gravador óptico no processo. |
| Teste de SDK externo | `xdzip-plugin-host` e `xdzip-plugin-protocol` formam um experimento XDP1 separado e explícito. Compilações de produto desktop e CLI não descobrem, instalam nem executam extensões de terceiros. |

Os dois primeiros caminhos são componentes integrados do produto. O processo incluído é separado, com protocolo privado e supervisão de recursos. O teste externo tem validação de pacote, negociação de versão e requisitos de permissão por plataforma próprios. [Arquitetura do sistema](system-architecture.md), [Contrato do processo de arquivos compactados](../engineering/native-codecs.md), [Contrato de teste do SDK](../engineering/sdk-trial.md)

<a name="xdzip-product-routing-and-operation-capabilities"></a>

## Roteamento do produto e capacidades de operação

A [fonte do registro de formatos](https://github.com/xiaodaozhi/xdzip/blob/main/specs/engineering/archive-formats.json) declara extensões, tipos de contêiner e filtro, operações suportadas, métodos e opções de criação. O gerador sincroniza o registro Rust, listas do desktop e artefatos de design. Na leitura, o ambiente de execução examina os bytes da origem e valida o formato antes de escolher um backend. Na criação, o nome de saída solicitado seleciona um perfil, e o ambiente verifica as opções contra ele. A lista de extensões ajuda na descoberta e nas dicas da interface. Detecção e varredura bem-sucedidas determinam o formato aberto.

Leitura, prévia, extração, verificação do arquivo inteiro, criação e regravação têm restrições separadas. Uma entrada listada pode estar visível sem método de conteúdo suportado. Senhas, volumes e registros especiais afetam a admissão; o ambiente também trata diferenças por formato, como verificações estruturais ISO/UDF e perfis de regravação não suportados. [Roteamento de execução](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/formats.rs), [Contrato do registro de formatos](../engineering/format-registry.md), [Verificação de arquivos compactados](../engineering/archive-checking.md)

<a name="xdzip-internal-rust-adapter-interfaces"></a>

## Interfaces internas dos adaptadores Rust

| Interface | Responsabilidade |
| --- | --- |
| `ReadSource` e `MetadataCodec` | Ler ou posicionar uma origem autorizada pelo host, incluindo metadados de volume aplicáveis, e produzir registros de formato limitados. |
| `PayloadCodec`, `PayloadReader` e `DecodedSink` | Resolver conteúdo e decodificar um registro físico elegível para um destino do host. Distinguir bytes comuns, trabalho de decodificação ignorado e trabalho auxiliar quando o backend os informar. |
| `CreationCodec`, `EncodeInput` e `EncodeOutput` | Codificar a partir de metadados de entrada fixados e fluxos indexados. Gravar em destino do host, com gravações aleatórias limitadas quando o gravador precisar. |
| `RewriteCodec` | Regravar arquivos suportados a partir das entradas originais preservadas, alterações e adições autorizadas em um novo arquivo preparado temporariamente. |

O host controla caminhos, revalidação da origem, cancelamento, senhas, cotas e toda publicação final. Um codec recebe capacidades pelas interfaces necessárias à operação. O processo de arquivos compactados solicita intervalos permitidos da origem e entrega registros ou blocos de saída limitados. O host valida o estado do protocolo e cotas de bytes antes de agir. Falha do processo ou resposta inválida não pode autorizar publicação. A fronteira do processo isola falhas e recursos. Restrições de permissão do núcleo para código externo arbitrário são requisito separado do teste. [Contratos de metadados](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/lib.rs), [Contratos de conteúdo](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/payload.rs), [Contratos de codificação e regravação](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/encode.rs), [Autorização de arquivos](file-security.md)

Esses traits Rust são interfaces internas de código-fonte. Um adaptador de produto é integrado ao espaço de trabalho e ao roteamento explícito de execução. Suas operações permitidas precisam de testes separados de arquivos válidos, recursos não suportados, limites de recursos, corrupção e comportamento de saída. Use os [contratos de codecs Rust independentes](../../engineering/independent-rust-codecs.md) e o [guia de desenvolvimento](../contributing/README.md).

<a name="xdzip-implemented-external-sdk-trial"></a>

## Teste implementado do SDK externo

O teste aceita um manifesto de pacote nativo selecionado explicitamente e uma origem escolhida pelo host. Ele permite varrer uma amostra XDP1 própria e ler uma entrada comum. A admissão valida o manifesto versão 1, o alvo executável e seu hash, caminhos internos do pacote, perfil fixo de permissões e limites. O host executa uma cópia privada verificada. O hash declarado comprova consistência do pacote; autenticar o publicador exigiria política de confiança separada.

O protocolo de teste versão 1 usa CBOR limitado com prefixo de comprimento e verifica identidade e sequência das operações. O host concede leituras da origem e um destino de saída por tokens opacos. A outra parte recebe intervalos de bytes e crédito de saída de um bloco. O host mede e verifica o conteúdo, confere conclusão do protocolo e estado da origem e publica sem sobrescrever somente após a saída limpa da outra parte. A amostra não expõe à outra parte senhas do arquivo, volumes divididos, criação, atualizações, links ou caminhos do host.

O teste tem requisitos nativos explícitos de permissão no macOS e Linux, com requisitos por plataforma e recusa em caso de falha definidos em seu contrato. O Windows recusa a execução. Testes nativos de interoperabilidade e negação exercitam esses limites independentemente do desktop. Consulte o [contrato de execução e protocolo do teste](../engineering/sdk-trial.md) para esquema, orçamentos e entradas de teste reais. [Host de teste](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-host/src/lib.rs), [Implementação de enquadramento](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-protocol/src/lib.rs)

<a name="xdzip-design-options-outside-the-current-implementation"></a>

## Opções de design fora da implementação atual

Um gerenciador de extensões no produto, confiança no publicador e política de atualização, compatibilidade além do protocolo de teste e capacidades externas de escrita ou senha continuam em design. A admissão ao produto exigiria verificar restrições de permissão, interoperabilidade e comportamento diante de falhas para cada plataforma e capacidade suportada. O processo incluído e o experimento XDP1 não ativam essa funcionalidade. [Decisões de arquitetura](architecture-decisions.md)
