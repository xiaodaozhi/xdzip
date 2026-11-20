<a name="xdzip-file-authorization-and-safe-publication"></a>

# Autorização de arquivos e publicação segura

[English](../../design/file-security.md) | [简体中文](../../zh-CN/design/file-security.md) | [繁體中文（台灣）](../../zh-TW/design/file-security.md) | [日本語](../../ja-JP/design/file-security.md) | [한국어](../../ko-KR/design/file-security.md) | [Français](../../fr-FR/design/file-security.md) | [Deutsch](../../de-DE/design/file-security.md) | [Español](../../es-ES/design/file-security.md) | Português (Brasil) | [Русский](../../ru-RU/design/file-security.md)

O host controla quais arquivos podem ser abertos e onde a saída verificada pode ser publicada. Um backend recebe leitores e coletores limitados, inclusive quando executado no processo de arquivo compactado isolado. Nomes, tamanhos declarados, somas de verificação e conteúdos do arquivo compactado são dados de entrada. As verificações de arquivo do host se aplicam independentemente das realizadas pelo formato. [Arquitetura do sistema](system-architecture.md), [Protocolo do processo de arquivo](../engineering/native-codecs.md)

<a name="xdzip-authorizing-the-source-and-destination"></a>

## Autorizar fonte e destino

- **Fonte do arquivo compactado.** `Source::open` abre o caminho escolhido por `xdzip-io::open_regular_readonly` e verifica o tipo do arquivo aberto. No Unix a abertura de classificação não bloqueia, para que um FIFO não possa bloqueá-la. Um caminho escolhido pelo usuário pode resolver um link para arquivo comum. A descoberta de volumes divididos é limitada e específica do formato. O host congela as partes vizinhas admitidas e verifica suas identidades e versões. Um codec só pode acessar os fluxos fornecidos. [Implementação da fonte](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/source.rs), [Tratamento de volumes](../engineering/archive-opening.md)
- **Entradas de criação.** `InputTree` percorre as entradas selecionadas com metadados e identidades de arquivos limitados. Leituras abrem membros dessa árvore verificando suas versões. O diretório privado de preparação atual é excluído da própria varredura de entrada. Uma alteração detectada interrompe o trabalho. Verificações de identidade, tamanho e horário detectam mudanças por melhor esforço, não como instantâneo completo do conteúdo. [Árvore de entrada](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-io/src/input.rs), [Criação de arquivo compactado](../engineering/zip-creation.md)
- **Destino.** `Grant` mantém o objeto diretório escolhido e sua identidade. A extração também passa as identidades de todos os volumes de origem como arquivos protegidos. Criação e edição verificam as identidades das entradas congeladas ou do arquivo original em seus próprios caminhos de tarefa. Subpastas e preparação são acessadas pelo diretório mantido e identidades dos pais são verificadas novamente antes da publicação. `CheckTarget` e a pré-verificação de extração podem fornecer informações úteis antes de iniciar a tarefa. A autorização de escrita é novamente verificada durante a operação. [Autorizações de destino](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-io/src/target.rs), [Pré-verificação de extração](../engineering/extraction-preflight.md)

<a name="xdzip-name-mapping-platform-protection-and-links"></a>

## Mapeamento de nomes, proteção por plataforma e links

1. **Validar a grafia relativa ao arquivo compactado.** O runtime resolve a seleção congelada, remove o prefixo da subárvore escolhida quando aplicável e usa `PortablePath` para aceitar componentes UTF-8 relativos. Caminhos absolutos ou de unidade, caracteres de controle, separadores ambíguos, componentes pai, nomes reservados de dispositivo e espaços ou pontos finais são rejeitados. Uma grafia validada é apenas entrada para a capacidade de destino.
2. **Planejar colisões antes de criar saída.** A extração compara aliases comuns de caixa e Unicode usando chaves NFD em minúsculas. Grafias ambíguas de diretório e colisões de arquivo/diretório fazem a preparação falhar. Aliases de arquivos comuns são mapeados a uma grafia comum para aplicar a política de conflito. A publicação nativa exclusiva ainda decide se o nome solicitado está disponível no sistema de arquivos real. Grafia exibida e identidade física da entrada permanecem separadas.
3. **Percorrer a árvore autorizada.** A saída desce de uma capacidade de diretório mantida, abrindo diretórios filhos sem seguir links. Verifica novamente a raiz autorizada, identidades dos pais e a folha esperada para sobrescrita. Um destino substituído ou link inseguro é recusado. Uma última verificação antes da renomeação no mesmo volume reduz corridas. Renomeação comum não garante comparação e troca atômica do conteúdo contra processo com autoridade da mesma conta.

| Plataforma | Verificações dos objetos de saída |
| --- | --- |
| Linux e macOS | O proprietário deve ser o usuário efetivo atual. Permissões de escrita para grupo e outros são recusadas e um arquivo comum existente deve ter um link físico. macOS também verifica a ACL nativa, inclusive permissões de modificação herdadas. |
| Windows | Verificar pontos de nova análise, proprietário, ID do arquivo, contagem de links físicos para arquivos comuns e DACL dos objetos mantidos. Regras efetivas e somente de herança devem limitar modificações a entidades confiáveis. Compartilhamento existente somente para leitura pode permanecer. |
| Texto claro privado ou área temporária | Verificações adicionais de confidencialidade excluem leitores não relacionados. No Windows um diretório de cache privado criado exclusivamente pode receber DACL protegida. Destinos comuns escolhidos pelo usuário mantêm sua própria política de compartilhamento de leitura. |

A saída de links simbólicos ZIP limita-se a um grafo Unix admitido. O runtime valida o grafo selecionado e folhas que devem estar ausentes antes de publicar links, confirmando primeiro destinos comuns. A tarefa nunca segue um link criado pelo arquivo compactado para gravar conteúdo. Outros perfis não suportados de links ou arquivos especiais falham explicitamente. Modos POSIX de arquivos comuns podem restaurar bits seguros de leitura/execução na preparação, além de leitura/gravação do proprietário. Bits de privilégio e escrita externa são excluídos. A extração não restaura horários após a confirmação.

[Analisador de caminhos portáveis](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-domain/src/path.rs), [Planejamento de extração](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/extraction/plan.rs), [Verificações de objetos privados](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-io/src/private.rs), [Detalhes de links e plataformas](../engineering/zip-safe-extraction.md)

<a name="xdzip-staging-verification-and-commit"></a>

## Preparação, verificação e confirmação

Uma autorização de destino cria espaço de trabalho privado exclusivo no mesmo volume. O host contabiliza a saída do codec no arquivo de preparação. O término da decodificação, tamanho esperado e verificações de integridade ou autenticação fornecidas pelo formato devem passar antes da publicação. `Stage::finish` descarrega e sincroniza o arquivo e mantém sua versão verificada. O host verifica novamente preparação e destino na confirmação. Sincronização dos dados e visibilidade atômica de um arquivo não comprovam durabilidade dos metadados do diretório após queda de energia.

| Operação | Semântica de publicação |
| --- | --- |
| Extrair ou publicar promessa de arquivo | Publicar cada arquivo elegível após verificação. Sem sobrescrita, criar exclusivamente um link físico no mesmo volume e depois remover o link privado de preparação. Com sobrescrita, a extração exige o arquivo comum seguro esperado e o substitui por renomeação no mesmo volume sem truncá-lo antes. Publicação de promessa de arquivo sempre ocorre sem sobrescrita. A ausência da operação necessária no sistema de arquivos ou alteração do destino faz a escrita afetada falhar. Não há reversão global dos arquivos já publicados ou diretórios criados. |
| Criar arquivo compactado único | Codificar na preparação, reabrir e verificar entradas e conteúdo esperados do perfil e publicar o arquivo completo uma vez. Um conflito de nome pode escolher outro nome ou permitir substituição elegível. Uma entrada protegida não pode se tornar alvo de sobrescrita. |
| Criar volumes divididos | Verificar o conteúdo preparado, preparar cada parte e publicar o novo conjunto plano sem sobrescrever volumes existentes. As partes ficam visíveis individualmente. Em falha parcial, a reversão tenta remover apenas objetos recém-publicados cujas identidades ainda correspondam. Uma reversão malsucedida pode deixar resultado incerto. |
| Modificar arquivo existente | `ArchiveUpdate` vincula o diretório pai autorizado e a versão esperada do arquivo original. Reabrir e verificar o arquivo reescrito e substituir o original uma vez por essa autoridade. Extração comum não pode usar esse caminho especial para sobrescrever a origem. |

Cancelamento antes da barreira de publicação impede a confirmação atual. Publicação já em andamento pode terminar e informar o resultado real. Arquivos de extração confirmados anteriormente permanecem visíveis. O instantâneo terminal pode informar dados confirmados junto com `cancelled`, `failed` ou aviso de limpeza. Uma publicação completa bem-sucedida do arquivo compactado mantém o sucesso mesmo com cancelamento tardio. [Cancelamento de tarefa](task-scheduling.md), [Modificação de arquivo compactado](../engineering/archive-transfers.md), [Extração segura](../engineering/zip-safe-extraction.md)

<a name="xdzip-output-budgets-and-integrity-scope"></a>

## Orçamentos de saída e escopo de integridade

A extração limita saída declarada e real separadamente: 8 GiB por arquivo comum e 20 GiB por tarefa. Uma verificação de proporção elevada pode recusar a solicitação na admissão em formatos suportados. Para ZIP, proporção declarada acima de 1.000 para um em entrada maior que 1 MiB exige permissão explícita. A permissão mantém os limites rígidos. Decodificadores sólidos ou sequenciais podem trabalhar em dados não selecionados. Esse trabalho, leituras ou tentativas repetidas e trabalho auxiliar são contabilizados separadamente dos bytes publicados.

Navegação de metadados, admissão de tarefas, memória do processo e prévia têm orçamentos próprios. Um grande índice ZIP em disco não aumenta limites de extração ou escrita. Verificações de integridade dependem do formato. Por exemplo, estrutura e legibilidade ISO/UDF não fornecem soma por arquivo que não existe. [Índices grandes](../engineering/large-index.md), [Verificação de integridade](../engineering/archive-checking.md), [Contabilidade auxiliar de decodificação](../engineering/auxiliary-decoding-work.md)

<a name="xdzip-preview-temporary-opening-and-secrets"></a>

## Prévia, abertura temporária e segredos

- **Prévia.** Selecionar um arquivo suportado inicia uma prévia no cartão de propriedades visível. Rust lê e verifica a entrada admitida inteira dentro dos orçamentos. Texto é limitado a 256 KiB. Imagens codificadas são limitadas a 16 MiB, com limites separados para dimensões e pixels originais. O processo privado de imagem recebe bytes e retorna uma miniatura limitada. Ele não recebe nome do arquivo compactado, senha ou handle de destino. Formatos e restrições atuais estão no [contrato de prévia](../engineering/content-preview.md).
- **Abertura temporária.** Uma entrada comum é extraída para o cache privado sob as regras normais de verificação da tarefa. O desktop entrega o arquivo verificado à associação do sistema como URL local. A aceitação não comprova que o programa externo terminou de ler. Em operação posterior após o período de retenção, a limpeza verifica identidade do cache e bloqueios. Edições da cópia são independentes do arquivo original. [Abertura temporária](../engineering/archive-opening.md), [Transferências de arquivos](../engineering/archive-transfers.md)
- **Segredos.** Senhas de abertura e extração permanecem em contêineres específicos da operação zerados ao liberar durante a vida da sessão ou tarefa. Não entram em instantâneos de tarefa nem no diário de preparação. As regras de armazenamento privado, retenção e divulgação de dados do usuário estão no [contrato de privacidade](../engineering/privacy.md) e no [contrato de senha](../engineering/password-extraction.md).

<a name="xdzip-journals-leftovers-and-verification"></a>

## Diários, restos e verificação

Diários de preparação registram contexto da tarefa e nomes relativos de saída na preparação e confirmação. A limpeza normal verifica propriedade e identidades e remove apenas objetos de preparação conhecidos. Diários privados e objetos restantes podem fornecer evidências após término anormal. O runtime conta possíveis restos ao verificar um destino. Ele não usa um diário descoberto para repetir gravações ou apagar conteúdo desconhecido. A expiração do cache de saída temporária privada tem verificações próprias de identidade e bloqueio.

Para mudanças nessa fronteira, verifique publicação e limpeza além dos códigos de erro. Os [testes de E/S](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-io/src/tests.rs), [testes de extração](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/extraction/tests.rs) e o [contrato de extração segura](../engineering/zip-safe-extraction.md) cobrem camadas distintas. Mudanças no comportamento nativo exigem evidência específica da plataforma afetada.
