# Contrato de execução da linha de comando

[English](../../engineering/command-line.md) | [简体中文](../../zh-CN/engineering/command-line.md) | [繁體中文](../../zh-TW/engineering/command-line.md) | [日本語](../../ja-JP/engineering/command-line.md) | [한국어](../../ko-KR/engineering/command-line.md) | [Français](../../fr-FR/engineering/command-line.md) | [Deutsch](../../de-DE/engineering/command-line.md) | [Español](../../es-ES/engineering/command-line.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/command-line.md)

O CLI é um hospedeiro Rust independente que chama serviços de formatos e E/S segura pelo mesmo `xdzip-runtime`; ele não depende de Qt, QML ou sessão gráfica. 7z, RAR, CAB, ISO e UDF continuam usando processos isolados fornecidos e não procuram ferramentas alternativas de extração no PATH. Plataformas e arquiteturas seguem a [política de suporte](support-policy.md). O CLI independente no Linux não requer X11; a execução exige OpenSSL 3 e certificados do sistema, e a compilação também exige libssl-dev e pkg-config.

Desde `v0.2.0`, os lançamentos oficiais fornecem pacotes CLI independentes para três plataformas e os pacotes de desktop incluem um CLI privado. Os arquivos de versões anteriores não mudam retroativamente. Consulte os [lançamentos](https://github.com/xiaodaozhi/xdzip/releases) para download e as seções abaixo e o [protocolo de lançamento](release-protocol.md) para os limites de compilação, instalação e atualização. O progresso de implementação e aceitação é acompanhado em [#254](https://github.com/xiaodaozhi/xdzip/issues/254).

## Uso e saída

```sh
xdzip --help
xdzip version
xdzip list archive.zip --json
xdzip extract archive.zip docs/ -C output
xdzip extract archive.zip --entry-id 3 -C output
xdzip create backup.7z docs/ --level normal
xdzip create data.iso docs/
xdzip create portable.sfx.exe docs/ --level normal
xdzip create private.zip docs/ --encrypt --password-stdin
xdzip add backup.zip notes.txt --into docs
xdzip test backup.zip
xdzip update --check
xdzip update --download --output downloads
xdzip update --yes
xdzip update --status
xdzip gui archive.zip
```

Sem argumentos ou com `--help`, xdzip mostra a ajuda. As formas `version`, `--version` e `--version-json` informam a identidade de `VERSION.json` e o estado Git da compilação. `--version-json` emite um objeto independente com product, version, build, revision, dirty, component, channel, configuration, distribution, platform e architecture. Para ajuda e mensagens use `--language zh-CN|en-US`. Por padrão são consultados `LC_ALL`, `LC_MESSAGES` e `LANG` nessa ordem; se o valor escolhido começar com `zh`, usa-se chinês. Identificadores públicos e códigos de erro são em inglês. `l`, `x`, `c`, `a` e `t` são atalhos de `list`, `extract`, `create`, `add` e `test`. Use `--` para passar argumentos de arquivo literais.

Todos os comandos de operação aceitam `--json`. stdout emite exatamente um objeto JSON com os campos fixos schema_version (1), command, status, error_code, data, summary e warnings. Tamanhos desconhecidos são null, inclusive nos elementos que compartilham um fluxo TAR compactado; membros ar brutos de DEB mantêm seus tamanhos compactados exatos. Os dados da lista incluem formato e entradas, cada uma com id, path, display_name, size, packed_size, method, directory, link, payload_supported, encrypted e warning. path é o nome efetivo congelado; nomes inválidos ou não resolvidos são null. display_name serve apenas à exibição escapada. Resumos de tarefa incluem as contagens reais de leitura, decodificação, verificação, confirmação, ignorados, diretórios e volumes, mantendo avisos de limpeza e resultados incertos.

Progresso, erros e solicitações interativas vão para stderr. Chamadas JSON, sem terminal ou com `--non-interactive` não perguntam. Se faltar senha ou decisão de conflito, a tarefa em espera é cancelada, aguarda-se a limpeza e retorna-se interaction_required. Códigos de saída: 0 para sucesso ou transferência explícita, 1 para falha, conclusão parcial ou aviso de limpeza, 2 para argumentos inválidos, 3 para interação, autorização ou condição de instalação ausente, e 130 para cancelamento confirmado. Ignorar explicitamente é uma política executada; a contagem de ignorados é informada separadamente. Se o cancelamento não for aceito e a confirmação tiver terminado, preserva-se o resultado real.

<a name="xdzip-archive-behavior"></a>

## Comportamento dos arquivos compactados

`list` retorna registros físicos e preserva duplicatas. `extract` aceita caminhos efetivos completos e exatos dos membros, diferenciando maiúsculas e minúsculas e usando barras. Diretórios são expandidos recursivamente e mantêm a hierarquia abaixo da raiz. Um caminho duplicado retorna `ambiguous_member`. Passe o ID numérico retornado por `list`, a partir de 1, com `--entry-id`; repita a opção para vários IDs. Caminhos ou IDs explícitos que não possam ser resolvidos falham antes da criação do diretório de destino. Não há suporte a curingas, leitura do arquivo compactado por stdin ou saída do conteúdo extraído para stdout.

`--limit` ou `--offset` ativa a paginação. O deslocamento começa em zero e não pode exceder 1.000.000. Com apenas `--limit`, o deslocamento é zero; com apenas `--offset`, o limite é 128. Os limites válidos são 1–4.096. O `data` JSON contém `offset`, `total` e `next_offset`, que pode ser null. `summary.entries` conta linhas efetivamente retornadas. Um ZIP/ZIP64 comum de arquivo único pode indexar um milhão de entradas, enquanto a lista sem páginas recusa mais de 100.000. Ambos os modos têm orçamento de 16 MiB para linhas JSON serializadas e texto renderizado. O excesso retorna `resource_limit` sem linhas parciais. Limites de página não alteram IDs físicos. Consulte [indexação de arquivos grandes](large-index.md).

extract usa por padrão o diretório atual. Se `-C` indicar um diretório ausente, a autorização é obtida componente por componente a partir do ancestral seguro existente mais próximo, em vez de criar diretórios recursivamente sem condições. Destino, identidade da origem, links, colisões Unicode ou de maiúsculas, preparação, verificação e confirmação seguem a [extração segura](zip-safe-extraction.md) e a [codificação de nomes](filename-encoding.md). A política de conflito padrão é Ask; `--conflict ask|skip|rename|overwrite` a seleciona explicitamente. `--allow-high-ratio` confirma apenas a alta taxa de compactação e não remove os limites rígidos de recursos.

`create` escolhe o perfil pelo [sufixo](format-registry.md) de saída e exige que o diretório pai exista. `--level store|fast|normal|best` é validado contra o perfil selecionado. Para TAR simples e ISO, omita o nível ou escolha `store`. LHA (`.lzh` ou `.lha`) usa níveis LH5 1, 6 e 9 para `fast`, `normal` e `best`. ZIP e 7z aceitam `--encrypt` e `--volume-size`; 7z também aceita `--encrypt-names` e `--solid` conforme seu perfil. Tamanhos de volume aceitam bytes inteiros positivos ou sufixos `KiB`, `MiB` e `GiB`. Argumentos não suportados pelo perfil falham antes da escrita.

Senhas são passadas somente por entrada oculta do terminal ou uma linha UTF-8 de `--password-stdin`; finais de linha são removidos sem aparar a senha. A criação criptografada interativa exige confirmação duas vezes. Argumentos com senha em texto simples são recusados; senhas não são lidas de variáveis de ambiente nem gravadas em configurações ou registros. Comprimento e zeragem seguem o [contrato de senhas](password-extraction.md) e o [contrato de criação](../../engineering/zip-creation.md).

`add` envia uma atualização ao hospedeiro e usa a raiz do arquivo compactado por padrão. `--into` seleciona um diretório seguro existente pelo nome relativo ao arquivo. Entradas admissíveis podem adicionar ou substituir itens. Movimentações internas são uma operação do desktop. O CLI compartilha as verificações de [modificação de arquivos](../../engineering/archive-transfers.md) para formatos e registros elegíveis, proteção de entradas, reconstrução temporária e substituição final. `test` verifica todo o arquivo sem produzir arquivos de membros. `data.check_kind` no JSON informa a força de uma verificação concluída conforme [verificação de arquivos](archive-checking.md).

Cada operação CLI tem ambiente de execução próprio e não participa das sessões de navegação do desktop. Ctrl+C e sinais de término solicitam cancelamento cooperativo e aguardam estado final, limpeza e término do ambiente; arquivos já confirmados permanecem. O `submit_wait` do serviço compartilhado adquire sequencialmente apenas um breve bloqueio de estado, sem esperar capacidade da fila ou alterar a admissão não bloqueante da GUI. Páginas de catálogo contêm até 128 entradas físicas. Continuam os limites existentes de quantidade e nomes de SelectMembers; todas as solicitações vinculam uma identidade imutável de sessão.

<a name="xdzip-build-and-command-installation"></a>

## Compilação e instalação do comando

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
python3 scripts/cli.py package --output build/package/cli
build/cli/bin/xdzip --help
```

Essa entrada usa Rust, CMake e a ferramenta C++ da plataforma fixados pelo repositório e o processo isolado existente, sem exigir Qt. O empacotamento Windows também exige o `prepare-windows-runtime.py` existente para preparar o runtime VC++ local fixado com sua licença original. A entrada unificada de compilação Qt também compila o CLI em um diretório cli privado sem renomear o executável GUI, ícone do aplicativo ou entradas de associação.

O executável CLI incluído no macOS fica em `Contents/Helpers/cli/`. Seu contexto JSON comum fica em `Contents/Resources/xdzip-cli-context.json`, selado pela assinatura de recursos do aplicativo. O CLI só encontra esse recurso no layout fixo Contents/Helpers/cli e verifica esquema, tipo desktop e caminho relativo fixo da GUI. Marcadores ausentes são tratados como identidade portátil; marcadores inválidos são recusados. Compilações repetidas removem marcadores gerados obsoletos sob Helpers; não enfraqueça a verificação de assinatura para resolver erro de layout.

Novos pacotes de desktop também declaram `worker_relative` nesse contexto e compartilham o processo de formatos do desktop. Apenas caminhos relativos fixos por plataforma são aceitos: processo do desktop adjacente no Windows, `Contents/MacOS/xdzip-native-codec` no macOS e processo adjacente à compilação ou implantado em `bin/` no Linux. Um processo ausente, ligado por link ou inesperado rejeita o contexto sem procurar no PATH ou em outra cópia. Contextos antigos sem esse campo e pacotes CLI independentes ou gerenciados mantêm a busca pelo processo vizinho. Substituições explícitas do desenvolvedor mantêm prioridade. Verifique a busca implantada com `scripts/check-cli.py --default-worker`; substituições comuns de teste não a comprovam.

Após descompactar um pacote independente, execute bin/xdzip, acrescentando .exe no Windows. `bin/xdzip setup` instala explicitamente uma entrada estável por usuário e um diretório de runtime versionado. No Unix a entrada padrão é ~/.local/bin; no Windows fica em xdzip/cli/bin no LocalAppData do usuário. Diretórios de gerenciamento usam os locais de dados de usuário da plataforma. `--bin-dir` e `--install-dir` substituem locais explicitamente. O usuário adiciona o diretório ao PATH; a ferramenta não modifica arquivos de inicialização do shell. Entradas ou arquivos de registro existentes nunca são sobrescritos.

A identidade gerenciada exige um installation.json válido. Um diretório chamado versions sem arquivo de registro ainda é portátil.

Ao remover um lançador Windows em execução, a cópia própria cujo hash ainda confere é movida para o subdiretório privado .xdzip-retired no mesmo diretório de comandos. O registro é removido imediatamente e o resultado registra retained_launcher. Um setup explícito posterior limpa somente cópias em recibos limitados cujos hashes ainda conferem e cujos processos terminaram. Arquivos modificados permanecem e não são sobrescritos. O CLI mantido no diretório versionado pode executar setup novamente para restaurar o registro.

O campo distribution distingue identidade de código-fonte e de lançamento. Uma compilação comum do código-fonte, mesmo limpa e em configuração Release, permite apenas verificar e baixar. A substituição automática exige simultaneamente identidade distribution marcada pelo pipeline Release dedicado, instalação gerenciada válida e assinatura de destino autenticada.

A opção PATH do instalador desktop Windows vem desmarcada e recusa conflitos com outros comandos xdzip. Em Mac/Linux é possível executar setup pelo CLI privado do pacote desktop; ele registra apenas o CLI dentro da instalação desktop e não copia um núcleo atualizável separadamente. `setup --remove` remove somente entradas e arquivos de registro próprios da ferramenta verificados por hash. Diretórios versionados e materiais de recuperação permanecem; entradas do usuário e diretórios de runtime possivelmente em uso não são excluídos.

O lançador estável não usa escape de shell. No Unix executa o CLI real com exec; no Windows aguarda o processo de console filho e encaminha cancelamento e código de saída. Um ponteiro atômico current.json seleciona o diretório de runtime atual. Falha da instalação inicial, da autoverificação da nova versão ou da confirmação dos arquivos não pode produzir resultado de instalação bem-sucedida.

## Atualizações, transferência ao desktop e recuperação

update atua sobre a instalação atual. Um CLI independente atualiza somente o CLI; o CLI incluído no pacote desktop atualiza toda a edição desktop. `--check` consulta, `--download` baixa e verifica, a operação padrão instala após confirmação e `--status` verifica recibos e identidade real. `--yes` confirma apenas essa atualização e não substitui autorização do sistema. Compilações portáteis, modificadas, de código-fonte ou de identidade desconhecida permitem somente verificar e baixar. A substituição automática independente exige instalação Release gerenciada e limpa.

O CLI usa GitHub Releases públicos fixos, canal e regras de número de compilação monotônico. Os arquivos CLI das três plataformas usam cli-release.json e cli-release.json.sig separados; a chave pública incorporada verifica os bytes brutos antes das verificações de tamanho e SHA-256. Assinatura ausente ou chave não confiável fornecem apenas informações para aquisição manual; assinatura inválida falha explicitamente. O release.json dos seis pacotes desktop não inclui entradas CLI e clientes antigos continuam usando o protocolo original.

A rede usa certificados do sistema e variáveis de ambiente padrão de proxy por padrão. `--proxy` seleciona um proxy HTTP/SOCKS5 sem credenciais; `--no-proxy` conecta diretamente; `--mirror` seleciona prefixo HTTPS sem credenciais, consulta ou fragmento. URLs originais do GitHub, hosts de redirecionamento, tamanhos de resposta, tempos limite e orçamentos de download continuam limitados. Resultados de rede, TLS, modo offline, limite de taxa e ausência de nova versão compatível são informados separadamente; não há consulta ou repetição automáticas.

Atualizações independentes são preparadas em diretório privado do usuário, extraídas com segurança pelo mesmo runtime de arquivos, verificam o inventário fixo completo e o hash de cada arquivo, produto, componente, plataforma e arquitetura locais e canal do manifesto assinado, e executam a autoverificação `--version-json` da nova versão antes de trocar o ponteiro. As transações são mutuamente exclusivas e os recibos vinculam a identidade do alvo. Uma falha preserva o ponteiro anterior ou inicia recuperação explicitamente; não se inventa reversão automática.

A verificação e o download desktop consultam apenas a entrada não gráfica de identidade de instalação do programa principal; não criam QApplication nem consomem recibos de instalação. A intenção de instalar passa pelo canal existente de instância única do mesmo usuário, que aceita somente diretório de solicitação limitado e versão candidata, não comandos, pacotes ou raízes de confiança arbitrários. Arquivos de solicitação e resultado reutilizam a autorização nativa de diretórios do Rust e confirmações seguras. O controlador desktop verifica e compara o candidato novamente e de forma independente, reutilizando Sobre, instalador, Sparkle e a barreira de conclusão de todas as tarefas. Tarefas CLI desktop também mantêm proteção de ocupação da instalação; a instalação não pode destruí-las.

O terminal que recebe a transferência aceita informa apenas handed_off. Instalação e recuperação efetivas dependem dos recibos do desktop e da identidade realmente em execução; baixar ou transferir não significa instalado. O modo não interativo recusa instalação desktop que exige GUI ou autorização do sistema. Consulte o [contrato de atualização](../../engineering/software-update.md) para limites de segurança de instalação e aceitação nativa. Aceitações automatizada, nativa e manual continuam distintas.

O perfil `.iso` cria imagens de dados ISO9660/Joliet sem compressão no nível 0. Omita `--level` ou use `--level store`. O perfil Windows x64 `.sfx.exe` encapsula ZIP Store/Deflate com o `xdzip-sfx.exe` vizinho fornecido. Ele recusa criptografia, modo sólido e volumes. Nenhum dos dois permite editar uma saída existente. Consulte [criação ISO](iso-creation.md) e [criação de SFX próprio](sfx-creation.md).
