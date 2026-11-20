<a name="xdzip-external-read-only-sdk-trial"></a>

# Teste externo de SDK somente leitura

[English](../../engineering/sdk-trial.md) | [简体中文](../../zh-CN/engineering/sdk-trial.md) | [繁體中文（台灣）](../../zh-TW/engineering/sdk-trial.md) | [日本語](../../ja-JP/engineering/sdk-trial.md) | [한국어](../../ko-KR/engineering/sdk-trial.md) | [Français](../../fr-FR/engineering/sdk-trial.md) | [Deutsch](../../de-DE/engineering/sdk-trial.md) | [Español](../../es-ES/engineering/sdk-trial.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/sdk-trial.md)

`xdzip-plugin-protocol` e `xdzip-plugin-host` implementam o teste independente somente leitura descrito no [design de extensão de formatos](../design/codec-extension-sdk.md). Ele fica fora da descoberta de formatos do desktop e da CLI, instalação, empacotamento e inicialização automática. O usuário executa explicitamente o comando de teste e seleciona pacote e origem; extensões de terceiros continuam desativadas no produto. Somente `xdp1` é negociado, sem criação, senhas, volumes, links, interface gráfica ou permissões arbitrárias de caminhos do hospedeiro. Isso não é um ecossistema completo de plugins nem uma publicação de SDK.

<a name="xdzip-reproducible-entry-points"></a>

## Pontos de entrada reproduzíveis

```sh
python3 scripts/sdk-trial.py test
python3 scripts/sdk-trial.py example --output-dir build/sdk-example
target/debug/xdzip-plugin-host scan build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp
mkdir build/sdk-example/output
target/debug/xdzip-plugin-host read build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp 0 build/sdk-example/output
```

No Windows use `python` e `.exe`. `test` compila o programa nativo de referência e executa regressões Rust de enquadramento, interoperabilidade Python independente, protocolo malicioso, cancelamento, alteração de origem, conflito e permissões do kernel. Python é iniciado somente pelo ponto de conformidade confiável compilado para testes; o hospedeiro distribuído não aceita interpretador, argumentos, shell ou opção de execução sem sandbox. O exemplo gera pacote próprio, manifesto e XDP1 somente em `build/`; não aciona publicação nem download de terceiros. Logs omitem caminhos do espaço de trabalho e do diretório pessoal; o JSON vincula commit, estado modificado e hashes dos binários. Ambos são artefatos locais ou de CI. A mesma matriz nativa Foundation chama esse ponto; o [fluxo separado de teste SDK](../../../.github/workflows/sdk-trial.yml) resolve primeiro a compatibilidade de sandbox por plataforma sem repetir pipelines não relacionados de formatos e Qt. O fluxo Foundation também pode chamar o teste reutilizável com `sdk_only=true`; isso pula o controle Foundation completo e não comprova resultado Foundation completo. No Windows verifica-se a recusa de execução.

<a name="xdzip-package-admission"></a>

## Admissão do pacote

O manifesto JSON rigoroso usa `deny_unknown_fields`. Os campos são `manifest_version=1`, `plugin_id`, `version`, `author`, `protocol_majors=[1]`, `targets`, um `entrypoint` dentro do pacote, `sha256` hexadecimal minúsculo, `formats=["xdp1"]`, `permissions`, `resources`, `license_files` e uma `source_url` HTTPS. O manifesto de referência gerado por `example` acima é uma entrada executável para o esquema atual.

As `permissions` são exatamente `broker_source_read=true`, `broker_output_write=true`, `network=false`; os `resources` são exatamente `memory_bytes=536870912`, `threads=1`. O manifesto tem limite de 32 KiB, o executável nativo de 64 MiB e são permitidos até quatro arquivos de licença, cada um de até 128 KiB. Pontos de entrada e arquivos de licença não podem ser absolutos, conter travessia para diretório pai ou escapar por links ou pontos de reanálise. Cada membro é aberto componente por componente sem seguir links por um diretório cap-std mantido e não é reaberto pela string original após a verificação. O hospedeiro copia do arquivo regular mantido, verifica contagem real de bytes, SHA-256 e versão do arquivo antes e depois da cópia e executa apenas a cópia temporária privada, preservada até o encerramento do processo e da IPC. O hash declarado comprova consistência do conteúdo do pacote selecionado; não autentica o publicador nem comprova que o código seja seguro.

<a name="xdzip-wire-protocol-and-state"></a>

## Protocolo e estado

Um subconjunto CBOR próprio e limitado segue [RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html) sem implementação CBOR de terceiros. O par Rust de referência reutiliza o crate de enquadramento; o [par Python](../../../examples/sdk/reference.py) implementa codificação, decodificação e análise XDP1 independentemente, sem importar Rust ou código de protocolo gerado. A interoperabilidade cobre vários blocos de conteúdo real e o digest do hospedeiro.

Quadros são `u32 LE length + CBOR`; o comprimento é verificado antes da alocação. Limites: quadro 1 MiB, bytes/text individuais 256 KiB, profundidade 16, arrays 128, mapas 32 e nós decodificados 8192. São aceitos apenas UInt, bytes, texto UTF-8 estrito, arrays, mapas com chaves UInt e bool. Inteiros que não usam a forma mais curta, chaves duplicadas, comprimentos indefinidos, inteiros negativos, tags, ponto flutuante e dados finais são rejeitados. As chaves do mapa de envelope `1..5` são kind, request, operation, sequence e payload. A sequência começa em 1 e aumenta consecutivamente em cada direção; request aumenta a cada chamada e operation é uma identidade aleatória não nula do hospedeiro. Cada resposta deve corresponder exatamente à identidade atual. Chaves payload desconhecidas `<128` são rejeitadas; `>=128` são campos opcionais ignoráveis.

| kind | Mensagem | Chaves numéricas do payload |
| --- | --- | --- |
| 1 | Hello | 1 major mínimo, 2 major máximo, 3 ID do pacote, 4 `xdp1` |
| 2 | Welcome | 1 major, 2 limite de bloco, 3 crédito de um bloco, 4 orçamento total de leitura, 5 orçamento de saída |
| 10 | Probe | 1 token de origem, 2 comprimento da origem |
| 11 | Open | 1 token de origem |
| 12 | Scan | 1 token de sessão |
| 13 | ReadEntry | 1 sessão, 2 entrada, 3 destino, 4 quantidade de créditos 1, 5 orçamento de saída, 6 token de crédito do primeiro bloco |
| 14 | Close | 1 sessão |
| 20, 21 | ReadAt, ReadData | 1 token de origem, 2 deslocamento, 3 comprimento ou bytes |
| 30 | EntryBatch | 1 sessão, 2 entradas; por registro: 1 token, 2 caminho bruto, 3 tipo 0 arquivo / 1 diretório, 4 tamanho conhecido |
| 31 | ScanFinished | 1 contagem real de registros |
| 40 | WriteData | 1 destino, 2 sequência de blocos a partir de 0, 3 bytes, 4 token de crédito atual |
| 41 | FinishOutput | 1 destino, 2 contagem declarada de saída, 3 bytes SHA-256 |
| 42 | WriteAck | 1 destino, 2 sequência do próximo bloco, 3 novo token de crédito aleatório |
| 50, 51, 52 | Cancel, CancelAck, Shutdown | Mapa vazio |
| 61 | OperationFinished | 1 resultado, 0 sucesso / 1 cancelado; Probe também tem 2 formato, Open também tem 2 sessão |

Outros kinds são rejeitados pela máquina de estados deste teste mesmo que o enquadramento os reconheça. Hello deve chegar primeiro e ambos selecionam major 1; a identidade do handshake é zero, as chamadas posteriores são diferentes de zero. ScanFinished ainda deve ser seguido por um único estado terminal bem-sucedido. Apenas uma operação executa por vez e não há novas tentativas da extensão em segundo plano. O crédito atual de um bloco é mais estrito que o máximo de quatro no design; um token imprevisível é renovado após a aceitação real de cada bloco de saída e não pode ser antecipado ou reutilizado.

<a name="xdzip-broker-budgets-and-publication"></a>

## Intermediário, orçamentos e publicação

Tokens de origem, destinos e créditos por bloco são valores aleatórios de 128 bits e não podem ser reutilizados entre operações. O par não recebe caminhos do arquivo compactado ou do destino. O hospedeiro retorna no máximo 256 KiB por leitura dentro do intervalo de origem autorizado; estouro de deslocamento, leitura após EOF, comprimento zero e leituras acumuladas acima de 64 MiB falham. Ele verifica o `FileVersion` do objeto mantido e do caminho correspondente antes e depois de cada leitura e antes da publicação final. Isso reutiliza a impressão digital existente de identidade, tempo e comprimento e não afirma detectar alterações concorrentes arbitrárias por um atacante da mesma conta que restaure todos os metadados.

Os metadados são limitados a 1000 registros físicos e 256 KiB no total de caminhos e tokens, com tokens de registro únicos. O hospedeiro valida rigorosamente caminhos brutos segundo PortablePath; nomes duplicados mantêm identidades distintas. Uma entrada regular é explicitamente selecionada por vez, com tamanho declarado de até 4 MiB; diretórios não podem ser saída de arquivo. O hospedeiro mantém o Grant de destino, Workspace privado e Stage, mede bytes reais e calcula SHA-256 sem fornecer ao par um identificador de arquivo ou autoridade de publicação.

A E/S segura existente só pode publicar sem sobrescrever após todos os blocos, contagem e digest de FinishOutput, o único estado terminal, Close, saída normal após Shutdown e EOF limpo, identidade da origem e verificações finais de cancelamento terem sucesso. Qualquer anomalia, estado terminal duplicado, falha ou tempo esgotado revoga a saída primeiro; a destruição remove o stage antes de limpar o workspace. Destinos existentes permanecem inalterados. A análise e os digests esperados continuam vindo da extensão; o cálculo do hospedeiro não comprova que a saída de uma extensão maliciosa seja fiel ao arquivo compactado.

O handshake é limitado a 2 segundos e cada chamada de teste a 5 segundos no total, sem o progresso reiniciar o prazo. A CPU por processo é limitada a 3 segundos, descritores de arquivo a 32; saída direta em arquivo é proibida e core dumps são desativados. Filas IPC recebem até quatro quadros e enviam um, e gravações em pipe têm limite de 2 segundos. stderr é descartado diretamente com orçamento zero; conteúdo não confiável não é encaminhado. O cancelamento revoga o destino imediatamente, concede 2 segundos de cooperação e depois encerra e recolhe o processo e os pipes; Ack não autoriza publicação. Esses orçamentos restringem um pequeno teste, não prometem desempenho para arquivos grandes comuns.

<a name="xdzip-platform-gates"></a>

## Condições por plataforma

| Plataforma | Política de teste e comportamento em falhas |
| --- | --- |
| macOS | A política deny-default de `/usr/bin/sandbox-exec` fixado permite a cópia executável privada exata, bibliotecas do sistema e cache dyld, leituras do carregador na raiz e metadados de `/var`. Arquivos do usuário, gravações, rede e fork são negados; o ambiente é limpo e somente stdio é herdado. Rlimits nativos e supervisão RSS a cada 20 ms encerram acima de 512 MiB ou em falha de medição. A RSS é amostrada, portanto picos breves entre amostras podem ultrapassar o limite; essa interface não é prometida para futura distribuição oficial. |
| Linux x86_64 / aarch64 | Landlock ABI ≥3 trata todos os direitos de arquivo v3, sem regras de gravação e com leitura somente do executável e bibliotecas do carregador do sistema. `no_new_privs` e uma lista de chamadas seccomp verificada por arquitetura incluem readv/writev necessários aos pipes padrão do Rust e negam socket, fork/clone, ptrace, io_uring e outros. Aplicam-se limite de CPU de 3 segundos, limite de espaço de endereçamento de 512 MiB e outros rlimits. Falta de regras, ABI, seccomp ou chamadas necessárias causa falha antes de exec. Somente a CI nativa comprova resultados reais dessa plataforma. |
| Windows e outras plataformas | A execução é recusada incondicionalmente. AppContainer, Job e lista permitida de identificadores não foram implementados e verificados; um subprocesso comum não substitui o sandbox. |

As escolhas de API Linux seguem a [documentação Landlock do kernel](https://docs.kernel.org/userspace-api/landlock.html) e UAPI; trabalhos futuros no Windows seguem o [isolamento AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation). Código unsafe nativo do hospedeiro fica restrito ao módulo de sandbox e documenta estruturas, pré-condições das chamadas e comportamento em falhas; pre-exec não aloca nem registra logs. O par de referência adiciona sondas controladas fork/_exit/waitpid somente no modo explícito de auditoria; o filho não executa limpeza Rust nem lógica de formato. As regressões de plataforma usam primeiro marcadores próprios legíveis e graváveis, um listener localhost próprio e self-exec como controles positivos; depois verificam a negação pelo kernel de leitura, gravação, rede, fork direto e self-exec conforme o estado real dos marcadores, arquivos e conexões. Plataformas não testadas não são marcadas como aprovadas e a execução de terceiros no desktop permanece desativada.

<a name="xdzip-owned-xdp1-example-format"></a>

## Formato de exemplo XDP1 próprio

XDP1 é somente uma amostra de interoperabilidade e não substitui qualquer formato real de arquivo compactado. Os quatro bytes `XDP1` são seguidos de uma contagem de registros `u32 LE`. Cada registro contém uma contagem de bytes do nome UTF-8 `u16 LE`, uma contagem de bytes do conteúdo `u32 LE`, nome e conteúdo sem compressão. São permitidos até 1000 registros sem dados finais; nomes e conteúdo são lidos dentro dos orçamentos do hospedeiro. Tokens de entrada são ordinais físicos LE de quatro bytes, sem deduplicação por caminho. Modos de falha Python são selecionados explicitamente apenas em testes de conformidade para verificar acesso não autorizado, sequência, replay, identidade duplicada, caminhos, créditos, digests, falhas, cancelamento e estados terminais duplicados.
