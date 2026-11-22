<a name="xdzip-split-volume-reading-and-temporary-opening"></a>

# Leitura de volumes e abertura temporária

[English](../../engineering/archive-opening.md) | [简体中文](../../zh-CN/engineering/archive-opening.md) | [繁體中文](../../zh-TW/engineering/archive-opening.md) | [日本語](../../ja-JP/engineering/archive-opening.md) | [한국어](../../ko-KR/engineering/archive-opening.md) | [Français](../../fr-FR/engineering/archive-opening.md) | [Deutsch](../../de-DE/engineering/archive-opening.md) | [Español](../../es-ES/engineering/archive-opening.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/archive-opening.md)

A seleção no desktop, as solicitações do gerenciador de arquivos e a CLI usam as mesmas regras Rust para abrir fontes. O host resolve conjuntos de volumes permitidos, registra a identidade da fonte e examina os bytes autorizados. O mecanismo escolhido fornece [metadados de navegação](readonly-browser.md), às vezes pelo [processo de formato isolado](native-codecs.md). Abrir explicitamente uma entrada cria uma cópia temporária verificada para o aplicativo associado. Use uma [verificação de integridade](archive-checking.md) separada para conferir todo o arquivo compactado.

Se um cabeçalho que parece ser TAR falhar na verificação completa da estrutura, a detecção usa a mesma ordem de candidatos e o mesmo limite de leitura compartilhado de uma origem não identificada. O LZMA independente só é testado por último, quando nenhum cabeçalho TAR nem ZIP autoextraível válido foi identificado.

<a name="xdzip-split-zip"></a>

No Windows, a extração e a abertura temporária preservam a zona de origem do arquivo compactado (Mark of the Web). Em arquivos divididos, vale a zona mais alta entre os volumes autorizados. O XDZip grava a zona no arquivo temporário privado antes de publicá-lo. Se o destino não puder armazenar a marca necessária, o arquivo não será publicado.

## ZIP dividido

A camada de leitura aceita fluxos contínuos numerados como `name.zip.001`, `.002` e volumes PKZIP como `name.z01`, `.z02` e o último `name.zip`. Qualquer volume pode ser usado para abrir o conjunto. Os volumes devem estar na mesma pasta, com nomes consecutivos e no máximo 1.024 volumes; volumes ausentes, links e arquivos especiais são rejeitados. As extensões servem apenas para descobrir conjuntos adjacentes. O contêiner real é identificado pelos bytes da fonte e examinado integralmente pelo mecanismo.

Quando o número de disco clássico ZIP64 é 0xFFFF, o total limitado de volumes no locator adjacente é usado primeiro para descobrir o conjunto real. O valor de preenchimento não significa que faltam 65.535 volumes. Essa descoberta não substitui a validação completa dos registros finais e do diretório.

O host Rust mantém identidade, comprimento e horário de modificação de cada volume. A entrada lógica lê entre volumes sob demanda sem unir o arquivo inteiro. Deslocamentos PKZIP relativos ao disco são convertidos em uma visão limitada do diretório central em um único volume. O codec existente continua verificando diretório, cabeçalhos locais, CRC, autenticação por senha e cotas de extração. Todos os volumes de origem são verificados novamente a cada uso e antes da conclusão; qualquer alteração interrompe a tarefa. A identidade de todos é protegida contra sobrescrita pela saída.

A normalização PKZIP processa apenas registros centrais admitidos e não aceita o registro final de assinatura digital do diretório central. Essa rejeição não significa que uma verificação de assinatura falhou. Um arquivo comum de volume único pode ler esse registro dentro dos limites, mas não verifica sua assinatura criptográfica e recusa modificações para evitar perda ou dano de metadados desconhecidos durante o reposicionamento.

A aceitação cobre volumes numerados, PKZIP, ZIP64, cabeçalhos e conteúdo distribuídos entre volumes, volumes ausentes ou não consecutivos, substituição da fonte, números de disco inválidos, nomes chineses e regressões em arquivos comuns de volume único.

<a name="xdzip-7z-and-rar-volumes"></a>

## Volumes 7z e RAR

Volumes 7z numerados formam fluxos contínuos como name.7z.001 e .002. RAR usa name.partN.rar ou nomes antigos como name.rar e name.r00; os limites físicos de cada volume são preservados ao entregá-los ao leitor RAR isolado em Rust. Ambos localizam o conjunto na mesma pasta a partir de qualquer parte existente. Os limites são 1.024 volumes e 100.000 entradas na busca do diretório. Volumes ausentes, números conflitantes nos nomes, links e pontos de nova análise são explicitamente rejeitados.

Rust mantém identidade, comprimento e horários do handle de cada volume e informa os limites autorizados ao adaptador nativo por ReadSource::volume_lengths(). O processo nativo só pode solicitar ao host a leitura de faixas autorizadas; a troca de volume não aceita caminhos arbitrários do host fornecidos pelo arquivo. Todos os volumes participam da nova verificação da fonte e da proteção contra sobrescrita. Uma alteração no primeiro volume ou em qualquer volume posterior interrompe a tarefa.

A criptografia de nomes 7z e RAR pode exigir senha na abertura. Se Open retornar password_required, nenhum índice será publicado. Qt reutiliza a janela de senha e reenvia o arquivo originalmente selecionado e o segredo limitado com OpenWithPassword; um valor de verificação incorreto retorna wrong_password. Após o sucesso, somente a sessão e suas tarefas admitidas mantêm a senha. Fechar a sessão libera sua cópia; ela não é gravada nas configurações nem nos registros de recuperação.

<a name="xdzip-cab-volumes"></a>

## Volumes CAB

Os nomes dos volumes CAB vêm dos cabeçalhos e podem ser quaisquer nomes de arquivo válidos; as regras de concatenação de bytes dos volumes numerados não se aplicam. A referência ao próximo avança um volume por vez. Uma referência lógica ao anterior pode apontar para um volume mais antigo onde começou um arquivo continuado, em vez do imediatamente anterior; amostras existentes também podem usar referências anteriores adjacentes. A partir do volume selecionado, Rust segue as referências anteriores para trás e depois descobre o conjunto completo na mesma pasta em ordem direta. Depois de validar a identidade do conjunto, a ordem consecutiva e as identidades, fixa o comprimento e o nome original de cada volume. O decodificador verifica todos os fragmentos de arquivo; um volume anterior não adjacente deve ser o início real de um arquivo continuado. Os limites são 1.024 volumes e 100.000 entradas descobertas na mesma pasta. São rejeitados volumes ausentes ou incorretos, ciclos, identidades duplicadas, caminhos absolutos, separadores de diretório, escapes para o diretório pai, links e pontos de nova análise. Um subconjunto aberto não é publicado como arquivo completo.

Os callbacks de troca de volume do processo só podem escolher nomes desse conjunto autorizado; cada volume fornece seu próprio fluxo limitado. Todos participam da nova verificação da fonte antes de tarefas e conclusões de arquivos, e da proteção contra sobrescrita. Avisos do manipulador nativo ou resultados de volume ausente não podem se tornar um índice bem-sucedido. Arquivos CAB de um ou vários volumes são somente para leitura.

<a name="xdzip-double-clicking-regular-files"></a>

## Clique duplo em arquivos comuns

Um clique duplo em um diretório continua entrando nele. Em um arquivo comum, ele envia uma tarefa de extração em segundo plano contendo apenas essa entrada física para um diretório de nome aleatório no cache privado do usuário. Links e objetos especiais não podem ser abertos por esse caminho. Senha, CRC, autenticação, cancelamento e admissão serial de tarefas pesadas da aplicação continuam valendo; o usuário não precisa escolher um destino.

Após a saída ser concluída, a associação do sistema é chamada por uma URL de arquivo local, sem interpretador de comandos. A janela de progresso correspondente só é ocultada automaticamente se o sistema aceitar a solicitação de abertura. Falha de extração, cancelamento ou recusa do sistema mantém a janela visível com erro legível. A aceitação pelo sistema não significa que o aplicativo externo terminou de ler; o arquivo temporário não pode ser apagado ao enviar a solicitação.

Diretórios temporários usam nomes aleatórios imprevisíveis de 128 bits, com conteúdo e marcador de identidade separados. O diretório de saída permite acesso somente ao usuário atual e a entidades de sistema confiáveis. No Windows, uma DACL protegida é definida pelo handle mantido apenas em um diretório vazio criado exclusivamente para essa operação, e verificada novamente antes de gravar dados em claro; permissões compartilhadas de um diretório escolhido pelo usuário não são alteradas. POSIX usa 0700 e verifica ACLs herdadas do macOS. Se não for possível garantir as permissões, a saída é recusada; veja a [API Win32 de DACL](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setsecurityinfo).

Não há limpeza enquanto um bloqueio estiver ativo. A saída concluída é mantida por pelo menos sete dias; depois, diretórios expirados e desbloqueados são limpos em uma abertura temporária posterior. O cache mantém no máximo 256 entradas e recusa explicitamente ao atingir o limite; cada varredura é limitada a 4.096 entradas. A limpeza valida permissões privadas, marcadores e nomes e não percorre links. Alterações em uma cópia temporária não são gravadas de volta no arquivo compactado.

A limpeza se limita ao cache privado criado pela aplicação; ela não percorre nem apaga diretórios de extração do usuário. A aceitação cobre ocultação automática em caso de sucesso, janelas mantidas em falhas, cancelamento, novas tentativas de senha, nomes duplicados, Unicode e caracteres especiais, encaminhamento de tarefas após fechar a página, segurança do diretório temporário e política de limpeza ao sair.

A admissão somente para leitura de [Brotli/TAR.BR](brotli-streams.md) valida integralmente candidatos sem cabeçalho antes de publicar linhas, compartilha a contabilidade de leituras dos candidatos Brotli/LZMA, preserva a prioridade de fontes fortes e da descoberta TAR/ZIP válida e nunca rebaixa um TAR decodificado danificado a fluxo comum.

Propostas [LHA/LZH](lha-containers.md) exigem validação completa e limitada de cabeçalhos e nomes antes da publicação, inclusive para um arquivo vazio de um único zero. Propostas fracas LHA, Brotli e LZMA compartilham uma contabilidade de leituras reais. A descoberta de contêineres fortes válidos mantém a prioridade; a admissão LHA completa pode resolver a ambiguidade de um marcador TAR bruto inválido. Não são admitidos perfis de senha, volumes ou reparo.

[PMA](pma-streams.md) usa a mesma contabilidade compartilhada de sondagens fracas e a admissão completa de metadados, com um terminador zero exato e preenchimento CP/M opcional e limitado. Conteúdo PM0/PM1/PM2 de nível 0 é suportado; métodos desconhecidos permanecem visíveis, mas sem suporte. Um contêiner CP/M vazio com preenchimento identifica PMA; um terminador zero de um byte indistinguível preserva a escolha anterior de LZH. Não são admitidos perfis de criação, senha, volumes, modificação ou reparo.
