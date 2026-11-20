<a name="xdzip-targzip-implementation-contract"></a>

# Contrato de implementação de TAR/GZIP

[English](../../engineering/tar-gzip.md) | [简体中文](../../zh-CN/engineering/tar-gzip.md) | [繁體中文](../../zh-TW/engineering/tar-gzip.md) | [日本語](../../ja-JP/engineering/tar-gzip.md) | [한국어](../../ko-KR/engineering/tar-gzip.md) | [Français](../../fr-FR/engineering/tar-gzip.md) | [Deutsch](../../de-DE/engineering/tar-gzip.md) | [Español](../../es-ES/engineering/tar-gzip.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/tar-gzip.md)

Este contrato define os limites de navegação, criação e extração segura de TAR/TAR.GZ. Resultados por plataforma e aceitação manual são registrados em issues ou artefatos de CI. Ele preserva autorização de entrada, preparação temporária, conflitos, cancelamento e confirmação do arquivo inteiro da [extração segura](zip-safe-extraction.md) e da [criação ZIP](zip-creation.md).

<a name="xdzip-scope-and-capabilities"></a>

## Escopo e capacidades

São suportadas navegação, criação e extração segura de TAR e TAR.GZ. TAR é um contêiner e GZIP um filtro de compressão; a leitura detecta o conteúdo em vez de escolher um backend pela extensão. O comportamento ZIP permanece compatível. Expansão aninhada automática, criptografia e TAR dividido não são habilitados. GZIP comum de um único arquivo usa o [contrato de fluxo](single-file-streams.md) separado.

Cabeçalhos TAR, números, PAX, codificação em fluxo e regravação usam uma camada independente de registros Rust. Enquadramento GZIP, CRC-32 e Deflate LZ77/Huffman usam o módulo compartilhado `xdzip-compression`. `tar` e `flate2` são apenas referências para testes de desenvolvimento e não entram nas dependências de produção desses crates. Consulte o [contrato de formatos próprios](../../engineering/independent-rust-codecs.md) para especificações, fontes e limites.

TAR aceita arquivos regulares e diretórios e analisa caminhos PAX e nomes longos GNU. Links simbólicos e físicos, dispositivos, FIFOs, arquivos esparsos e tipos desconhecidos não podem ser extraídos. Todos os nomes de saída continuam passando por PortablePath e pela pré-verificação de conflitos do host; bibliotecas de formato não recebem caminhos nem autoridade para desempacotar. Permissões de diretório, proprietários e modos especiais não são restaurados. Datas são metadados de exibição sem garantia de restauração.

<a name="xdzip-bounded-streaming"></a>

## Processamento em fluxo limitado

Codificações antigas de nomes, nomes longos GNU e declarações PAX UTF-8/BINARY são resolvidos uniformemente pelo [contrato de codificação](filename-encoding.md). Índices só são publicados após varredura completa; contagens e totais são desconhecidos durante a varredura. Metadados mantêm limites de 100.000 entradas, 4.096 bytes por nome e orçamentos cumulativos de nomes. A entrada codificada do fluxo e o trabalho de decodificação têm cada um no máximo 24 GiB; um cabeçalho de extensão GNU/PAX tem no máximo 64 KiB, com até quatro cabeçalhos por entrada. PAX global aceita somente comentários sem semântica de arquivo, para compatibilidade com git archive. Eles são integralmente verificados pela extensão declarada do registro e descartados, sem herança pelas entradas. Campos globais path, size, mtime, linkpath, sparse e desconhecidos continuam explicitamente rejeitados; links longos GNU e variantes esparsas não são suportados. Comentários globais também contam para os quatro cabeçalhos por entrada e para os orçamentos de bytes do fluxo e não tornam válidas extensões locais incompletas. O prefixo comum `./` em nomes TAR é normalizado; marcadores raiz não são entradas de saída. Leituras e saltos sequenciais também verificam cancelamento e orçamentos de bytes; posicionamento não contorna verificações de fluxo corrompido. GZIP deve chegar ao EOF e verificar CRC e ISIZE. Vários membros formam apenas um fluxo contínuo de bytes TAR. Dados finais não nulos, truncamento e corrupção falham. Índices parciais não verificados são rejeitados.

TAR.GZ não possui tamanhos compactados por entrada. A admissão por alta taxa compara o total declarado selecionado com o tamanho completo da origem fixada. Acima de 1000:1 e 1 MiB ainda é necessária autorização explícita, que não ignora cotas de trabalho de varredura ou saída. A pré-verificação de metadados não inventa taxas por entrada.

A extração processa entradas selecionadas em ordem física. Pular conteúdo não selecionado é contabilizado e cancelável; leitura aleatória não é prometida. PayloadReader recebe apenas a origem e o destino do host. O host continua verificando versões da origem nos limites de varredura, decodificação e confirmação. A confirmação por entrada e a semântica de falha da criação inteira seguem o contrato existente de E/S segura.

<a name="xdzip-creation-and-interface"></a>

## Criação e interface

A janela de criação habilita os segmentos TAR e TAR.GZ existentes sem mudar a densidade. A troca de formato atualiza as extensões. TAR desabilita o nível de compressão; ambos os formatos TAR desabilitam e limpam senha e valores de volume dividido. TAR.GZ usa níveis GZIP 0/1/6/9. O tamanho compactado por entrada é desconhecido e aparece como traço, não 0 bytes. Rust deriva uniformemente a pasta de destino nomeada pelo arquivo compactado a partir do nome da origem e a fornece pela ponte limitada para prévia. TAR.GZ remove toda a extensão composta sem diferenciar maiúsculas; um nome-base vazio usa archive. A interface não deriva outro destino; PortablePath e a raiz autorizada o validam novamente. As etiquetas de formato da janela principal se adaptam ao formato real e à largura do texto. Datas TAR aparecem em UTC e permanecem desconhecidas fora do intervalo de exibição existente. A criação grava explicitamente caminhos PAX UTF-8 para Unicode ou nomes longos e tamanho PAX para arquivos grandes, evitando suposições de codificação local de leitores de referência. Fixe a árvore de entrada, grave em fluxo, finalize TAR/GZIP, verifique novamente e decodifique e valide cada entrada; publique o conjunto somente após sucesso. A renomeação por conflito preserva toda a extensão composta.

<a name="xdzip-verification-and-limits"></a>

## Verificação e limites

Cubra interoperabilidade bidirecional com tar do sistema/Python tarfile, caminhos longos, Unicode, diretórios vazios, duplicatas, PAX/GNU, entradas especiais, escape de caminho, truncamento, CRC, limites de recursos, cancelamento e proteção de arquivos anteriores. Criação, reabertura e extração reais no Qt reutilizam janelas existentes de tarefas e conflitos. Evidências reais são geradas apenas em build/ ou artefatos de CI.

Escolha as reverificações de desktop e acessibilidade afetadas conforme a [verificação geral](../../../developers/qa/README.md); automação nativa e verificações fora da tela não substituem verificações manuais. Resultados e limitações reais vão para a issue correspondente. A aceitação de desktop existente está em [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5); bloqueios pontuais de permissão ou ambiente não permanecem neste contrato. A revisão histórica de licença de `v0.1.0` está em [#4](https://github.com/xiaodaozhi/xdzip/issues/4) e [#53](https://github.com/xiaodaozhi/xdzip/issues/53); futuros candidatos precisam de novas issues de lançamento.

A entrada de criação da CLI aceita .tgz como alias do mesmo formato de .tar.gz, com a mesma codificação GZIP, verificação e confirmação. GZIP comum sem TAR usa o codec separado de arquivo único.
