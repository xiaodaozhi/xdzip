<a name="xdzip-ordinary-brotli-stream-contract"></a>

# Contrato de fluxos Brotli comuns

[English](../../engineering/brotli-streams.md) | [简体中文](../../zh-CN/engineering/brotli-streams.md) | [繁體中文](../../zh-TW/engineering/brotli-streams.md) | [日本語](../../ja-JP/engineering/brotli-streams.md) | [한국어](../../ko-KR/engineering/brotli-streams.md) | [Français](../../fr-FR/engineering/brotli-streams.md) | [Deutsch](../../de-DE/engineering/brotli-streams.md) | [Español](../../es-ES/engineering/brotli-streams.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/brotli-streams.md)

`xdzip-compression::BrotliReader` decodifica independentemente um único fluxo Brotli comum da RFC 7932. Recebe uma capacidade `Read` pertencente ao chamador e uma faixa compactada admitida; não tem autoridade sobre nomes, sistema de arquivos, rede, processos ou publicação de saída. O produto registra Brotli (`.br`) e TAR.BR (`.tar.br`) no [registro de formatos](format-registry.md). Ambos permitem navegação, visualização, verificação e extração de itens selecionados segundo as regras comuns do host para fonte, tarefa, cota, preparação e publicação. Arquivos TAR.BR existentes elegíveis podem ser editados por regravação com metablocos padrão não comprimidos. Nenhum perfil permite criar novos arquivos compactados ou volumes divididos. Consulte a [reescrita de arquivos TAR comprimidos](single-file-streams.md#xdzip-rewriting-compressed-tar-archives).

<a name="xdzip-wire-profile"></a>

## Perfil do fluxo

O leitor aceita declarações de janela padrão 10–24, metablocos comprimidos, não comprimidos e de metadados, códigos de prefixo canônicos simples e complexos, repetições encadeadas de comprimento de código, as três categorias de mudança de bloco, os quatro contextos literais, mapas de contexto RLE e MTF inverso, distâncias explícitas, recentes, diretas e de sufixo e cópias sobrepostas do histórico. Cópias do dicionário aplicam comprimentos normativos de palavra 4–24 e todas as 121 transformações de bytes, inclusive as que produzem zero bytes. Distâncias recentes persistem entre metablocos e são atualizadas apenas para referências de histórico elegíveis. Bytes brutos, bytes de dicionário e cópias do histórico atualizam o contexto literal.

A fonte é a [RFC 7932](https://www.rfc-editor.org/rfc/rfc7932.txt), SHA-256 `394abd1879e016f4eac89512c2acaf92f7b2d67feb8ab58b6b118c213be07146`. A [errata 5948](https://www.rfc-editor.org/errata/eid5948) verificada corrige um erro editorial na seção 3.5. A [RFC 9841](https://www.rfc-editor.org/rfc/rfc9841.html) acrescenta dicionários compartilhados, janelas grandes e contêiner separado; essas extensões ficam fora do perfil de fluxo comum. O padrão reservado de janela estendida é informado como não suportado. Dicionários externos nunca são procurados ou baixados.

Um único fluxo exato deve terminar com preenchimento zero válido e consumir sua faixa de bytes admitida. Campos de comprimento não canônicos, bits reservados de metadados, árvores inválidas, supersaturadas ou incompletas, símbolos repetidos, lacunas de mapa ou tipo, distâncias ou transformações inválidas e saída fora do bloco falham. O perfil rejeita concatenação, bytes adicionais e fluxos truncados. Brotli não tem soma de verificação de conteúdo, declaração do tamanho decodificado total, criptografia, caminho de membro ou carimbo de data e hora. A decodificação bem-sucedida comprova estrutura admitida, tamanhos declarados de metablocos e legibilidade; uma alteração que continue formando fluxo válido não pode ser detectada como corrupção.

<a name="xdzip-product-discovery-and-presentation"></a>

## Detecção e apresentação no produto

Brotli não tem assinatura mágica fixa. Assinaturas fortes da fonte, metadados TAR comuns válidos e detecção ZIP/SFX mantêm prioridade. No caminho restante sem cabeçalho, um prefixo Brotli limitado propõe uma composição comum ou TAR; uma varredura completa de metadados deve validar todo o fluxo antes de publicar o formato ou qualquer linha. Uma hipótese comum malsucedida pode prosseguir ao candidato LZMA Alone existente. Um cabeçalho TAR reconhecido após decodificação Brotli é determinante: falha na análise TAR nunca se torna sucesso como arquivo comum ou LZMA. Um marcador TAR bruto não validado nos bytes comprimidos só pode ser resolvido por admissão Brotli completa e válida; uma hipótese inválida preserva a falha TAR original. Erros de fonte, cancelamento e recursos são terminais.

Um único orçamento de leitura real de 24 GiB cobre testes dos candidatos Brotli/LZMA e leituras repetidas de metadados, sem devolução entre candidatos. Outras detecções de formatos fortes e envelopes de texto mantêm seus limites independentes. Fluxos Brotli vazios de um byte são admitidos; arquivos físicos vazios são inválidos. Sufixos não validam candidatos. Após validação completa, uma dica `.br` ou `.tar.br` do mesmo filtro resolve apenas uma saída ambígua de zeros de pelo menos 1024 bytes; outras dicas mantêm o padrão TAR vazio. Cabeçalhos TAR reais continuam prioritários sobre sufixos comuns.

Brotli comum apresenta um arquivo virtual regular com nome base seguro derivado do host, tamanho decodificado conhecido e tamanho físico compactado completo. TAR.BR preserva a identidade física das entradas TAR e tamanhos compactados desconhecidos dos filhos. Ambos mostram o método Brotli real e a observação de verificação estrutural, sem sugerir soma de verificação. A imagem genérica de arquivo compactado é reutilizada; a declaração MIME existente `application/x-brotli` fornece um candidato comum para o shell.

<a name="xdzip-resource-and-control-rules"></a>

## Recursos e controle

Antes de alocar o histórico, admitir `(1 << window_bits) + 2 MiB`. O segundo termo cobre de forma conservadora símbolos de prefixo limitados, vetores de bloco e contexto, buffer de entrada e estado fixo; cada contagem declarada também é limitada independentemente. Há no máximo 256 tipos de bloco ou árvores de contexto por categoria. Mapas literais têm no máximo 16384 entradas, mapas de distância 1024 e alfabetos de prefixo 704 símbolos. Dados normativos estáticos não concedem E/S externa.

Entrada compactada e saída acumulada têm cada uma limite rígido de 24 GiB, que o chamador pode reduzir. Análise real de bits, construção de prefixos, expansão de símbolos e repetições, movimento MTF de contexto e saída consomem um único orçamento acumulado de trabalho limitado a 96 GiB unidades; falhas nunca o devolvem. Declarações de saída de metablocos são admitidas antes da decodificação e nunca ampliam a cota do chamador. Cada leitura retorna no máximo 8192 bytes decodificados. `with_poll` chama uma função cooperativa sem autoridade em intervalos de trabalho limitados, inclusive durante cabeçalhos ou trabalho sem saída; o cancelamento não depende da necessidade de mais bytes da fonte. Erros da função, da fonte e de cota são terminais. Somente leituras interrompidas simples são repetidas; interrupções tipadas preservam o objeto de erro sem repetição por métodos auxiliares.

O chamador deve ler até EOF e manter a saída privada até o sucesso. Um erro posterior invalida os blocos anteriores; a primitiva nunca os publica. Um leitor que falhou não pode continuar. O controle cooperativo não interrompe uma leitura bloqueada do sistema operacional subjacente. Congelar a fonte e publicar o resultado final continuam sendo responsabilidades do host.

<a name="xdzip-normative-data-and-validation"></a>

## Dados normativos e validação

O dicionário estático e três tabelas de contexto são extraídos diretamente da RFC fixada e todos os comprimentos normativos e CRCs são verificadas. Profundidades de palavras e identificadores de transformação também são validados. A [origem e licença dos dados](../../../third_party/brotli-rfc7932/README.md) cobrem essas constantes padrão; nenhuma implementação Brotli de referência é vinculada. Manter o aviso BSD revisado da IETF no código e nos avisos distribuídos. As entradas de coleta de avisos desktop e CLI o coletam separadamente das dependências Cargo e dos runtimes de referência.

[Dados de teste próprios](../../../tests/fixtures/brotli/README.md) são gerados ou construídos independentemente e depois verificados byte a byte por um decodificador do autor fixado. Cobrem qualidades, janelas, saída real dependente de contexto, mudanças de bloco, parâmetros de distância, todas as transformações normativas e blocos de metadados e brutos. Cotas exatas, truncamento, preenchimento, entrada curta, erros tipados, sondagem de trabalho e mutações determinísticas complementam os casos de interoperabilidade. Referências ficam em `build/toolchain/` e evidências em `build/` ou CI. `FORMAT-BROTLI` nos [requisitos de aceitação](../../../developers/qa/acceptance.md) rege a integração do produto separadamente dos testes da primitiva.
