<a name="xdzip-adaptive-filename-encoding-contract"></a>

# Contrato de codificação adaptativa de nomes de arquivos

[English](../../engineering/filename-encoding.md) | [简体中文](../../zh-CN/engineering/filename-encoding.md) | [繁體中文（台灣）](../../zh-TW/engineering/filename-encoding.md) | [日本語](../../ja-JP/engineering/filename-encoding.md) | [한국어](../../ko-KR/engineering/filename-encoding.md) | [Français](../../fr-FR/engineering/filename-encoding.md) | [Deutsch](../../de-DE/engineering/filename-encoding.md) | [Español](../../es-ES/engineering/filename-encoding.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/filename-encoding.md)

<a name="xdzip-resolution-and-identity"></a>

## Resolução e identidade

A decodificação ocorre na camada de formatos Rust independente do Qt. Ela não usa páginas de código do sistema host, idioma da interface, suposições sobre o sistema de origem, conteúdo dos arquivos ou serviços de rede. Em ZIP, raw_name preserva os bytes originais do campo de nome. TAR preserva o nome bruto selecionado por PAX/GNU após remover o `./` inicial conforme as regras existentes. O EntryId físico é independente dos nomes. decoded_name armazena a grafia UTF-8 que exige transcodificação ou declaração adicional de codificação; name_encoding mantém a origem da resolução e name_bytes fornece o nome efetivo congelado. display_name é apenas exibição escapada e não pode ser caminho de saída.

ZIP resolve nomes nesta ordem:

1. Quando a marca UTF-8 está definida, exigir UTF-8 válido no campo original; não reparar declaração corrompida com codificação antiga.
2. Caso contrário, ler Info-ZIP Unicode Path 0x7075 e aceitar versão 1 com CRC32 correspondente ao nome original e UTF-8 completo e válido. Ignorar versões desconhecidas e CRCs divergentes conforme a especificação. Marcar para revisão UTF-8 inválido com CRC correspondente, campos válidos duplicados ou estrutura danificada.
3. Sem extensão utilizável, se todos os nomes não declarados do grupo forem UTF-8 válido, aceitar UTF-8 não marcado.
4. Caso contrário, aplicar detecção estatística em todo o arquivo compactado aos nomes não ASCII não declarados, incluindo GBK/GB18030, Big5, Shift-JIS, EUC-JP, coreano e outros conjuntos suportados. A hipótese ocidental padrão do ZIP recorre ao padrão histórico CP437. Quando GBK for detectado, usar o decodificador estrito GB18030 compatível.

Cabeçalhos TAR comuns, nomes longos GNU e caminhos PAX explicitamente marcados hdrcharset=BINARY usam a mesma estratégia para nomes não declarados. Caminhos PAX comuns exigem UTF-8 válido; a detecção não deve esconder declarações corrompidas. Valores explícitos de hdrcharset diferentes de UTF-8 e BINARY não são suportados. TAR não herda o padrão CP437 do ZIP. TAR.GZ usa a mesma resolução de nomes do TAR.

As regras seguem [ZIP APPNOTE 6.3.10](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT), [POSIX pax](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/pax.html) e o [padrão WHATWG Encoding](https://encoding.spec.whatwg.org/). Um campo Unicode Path pode estar ausente do cabeçalho local, mas um valor válido fornecido deve corresponder ao nome efetivo congelado. A verificação dos dados continua conferindo nome bruto, cabeçalhos local e central, CRC, tamanho, autenticação de senha e término do fluxo.

CAB preserva bytes originais do nome e atributo UTF-8. UTF-8 explícito é validado estritamente; os demais nomes reutilizam detecção limitada em todo o arquivo sem páginas de código do sistema. Barras invertidas do CAB são separadores de diretório do formato, mas só são normalizadas para barras após a decodificação completa, evitando dividir o byte final multibyte 0x5C do Shift-JIS. Os bytes brutos permanecem iguais e caminhos efetivos passam pelas mesmas verificações de segurança e colisão. Nomes ISO não Joliet preservam bytes brutos e usam a mesma resolução limitada, evitando conversão OEM do host para nomes Rock Ridge. Joliet/UDF usam nomes Unicode fornecidos pelo processador e os validam estritamente; caracteres de substituição com perda não são tratados como nomes originais.

<a name="xdzip-detection-boundaries-and-resources"></a>

## Limites e recursos da detecção

O detector lê no máximo 1 MiB de nomes completos não declarados e não UTF-8, separando registros por quebras de linha para impedir que sequências multibyte truncadas se combinem entre nomes. Declarações Unicode explícitas e extensões inválidas não são amostras. Quando a amostra contém nomes não UTF-8, aplicar consistentemente o resultado global aos nomes não declarados, evitando que fragmentos GBK coincidentemente válidos em UTF-8 dividam um diretório em dois nomes. Apenas quando a decodificação estrita de um nome individual falhar pode-se tentar UTF-8 válido ou resultado local para esse nome. A detecção ISO-2022-JP é desativada; estados de escape em nomes não são interpretados. A decodificação estrita não insere caracteres de substituição, remove BOM, converte barras, normaliza Unicode nem repara caminhos.

Nomes brutos somam no máximo 16 MiB. Um único nome bruto ou decodificado tem no máximo 4,096 bytes, e nomes decodificados mais exibição escapada somam no máximo 16 MiB. Alocações por entrada e acumuladas da extensão Unicode ZIP são verificadas durante a varredura, não após o fim do arquivo. Resolução, detecção e conversão individual verificam cancelamento. Dados ZIP mantêm no máximo mais 16 MiB para a tabela de verificação de nomes efetivos; decodificação posterior não pode alterar nomes congelados.

Detecção estatística não recupera de forma única nomes curtos arbitrários ou codificações antigas misturadas, nem identifica texto já corrompido e salvo como Unicode válido por outro programa. Unicode explicitamente declarado e codificação antiga podem coexistir no mesmo arquivo; várias codificações não declaradas são detectadas pelo melhor esforço. Atualmente não há seletor manual de codificação. Nomes que não podem ser decodificados estritamente, possuem declarações corrompidas ou caminhos anormais continuam exibidos de forma escapada e marcados para revisão, sem permitir extração, abertura temporária, exportação ou alteração. Não se promete identificar corretamente todos os arquivos históricos. Se houver nomes não resolvidos, sua associação real a diretórios pode ser desconhecida; por isso a extração por expansão de diretório é rejeitada para evitar omitir itens silenciosamente de um resultado bem-sucedido. Arquivos regulares resolvidos e selecionados explicitamente ainda podem ser extraídos individualmente.

<a name="xdzip-end-to-end-use-and-saving"></a>

## Uso completo e salvamento

Árvore de diretórios, índice de filhos diretos, busca recursiva, expansão da seleção de diretório, pré-verificação, extração, abertura temporária, exportação externa e alteração do arquivo usam o mesmo nome efetivo. Após decodificar, ainda verificar PortablePath, aliases Unicode NFD ou de maiúsculas e minúsculas, nomes duplicados e autorização de saída. O byte final multibyte 0x5C do Shift-JIS não é separador de diretório. Exceto pelos separadores de formato CAB e LHA, barras invertidas realmente decodificadas continuam sendo rejeitadas.

Após o planejamento, adicionar e mover em ZIP, TAR e TAR.GZ comuns declara novamente como UTF-8 os nomes de codificação antiga, extensão Unicode e UTF-8 não marcado, mantendo posições originais e semântica de pular no plano lógico de movimentação. ZIP preserva os dados compactados e TAR usa caminhos PAX UTF-8 explícitos. Após gravar, varrer novamente, verificar cada nome e conteúdo e executar a substituição existente no mesmo volume. Se a regravação TAR remover uma declaração BINARY de um caminho PAX e isso alterar a interpretação de outros textos PAX não UTF-8, falhar explicitamente e preservar o arquivo original. Arquivos criptografados e divididos usam os mesmos nomes para leitura e extração; restrições de adição e movimentação permanecem iguais.

<a name="xdzip-reproducible-acceptance"></a>

## Aceitação reproduzível

Usar dados de teste próprios de bytes brutos e arquivos compactados codificados independentemente com a biblioteca padrão Python, cobrindo UTF-8 não marcado, GBK, GB18030 de quatro bytes, Big5, Shift-JIS com 0x5C, coreano, russo, ZIP CP437 e Unicode Path. Verificar nomes completos, bytes originais, diretórios, busca, seleção, pré-verificação, bytes de saída reais, abertura temporária, exportação, adição e movimentação em arquivos comuns, AES e volumes divididos. Verificar também declarações danificadas, prioridade de extensões, rejeição de caminhos, colisões Unicode ou de maiúsculas e minúsculas, cancelamento e orçamentos.

scripts/check-filename-encoding.py extrai ZIP/TAR/TAR.GZ gerados pelo Python por meio de uma sonda real do serviço. Python também lê independentemente arquivos regravados pelo Rust para verificar marcas UTF-8/PAX, nomes e conteúdos. Os dados de teste vêm apenas de constantes do código, não de arquivos de usuários. Resultados reais e aceitação manual pendente são registrados em issues e artefatos build/CI, não como aprovação neste contrato.

LHA preserva os limites originais dos campos dirname/filename e os separadores brutos `0xff` de dirname. A detecção limitada compartilhada amostra cada campo completo separadamente. Decodificar estritamente cada componente antes de interpretar barras invertidas decodificadas e compor `/`; os limites originais impedem empréstimo de bytes multibyte entre campos. Expansão de diretório e saída usam o mesmo nome efetivo. Consulte o [contrato LHA](lha-containers.md).
