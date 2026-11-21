# Compilações de desenvolvimento e verificação por plataforma

[English](../../engineering/developer-build.md) | [简体中文](../../zh-CN/engineering/developer-build.md) | [繁體中文](../../zh-TW/engineering/developer-build.md) | [日本語](../../ja-JP/engineering/developer-build.md) | [Français](../../fr-FR/engineering/developer-build.md) | [Español](../../es-ES/engineering/developer-build.md) | Português (Brasil)

Esta página é a entrada unificada para preparar, compilar, testar, empacotar e verificar ambientes nas três plataformas. A coordenação compartilhada é implementada em [scripts/desktop.py](../../../scripts/desktop.py). Os documentos de plataforma complementam layouts de implantação, comportamento das ferramentas e limites do sistema, sem manter comandos rotineiros separados.

A CLI independente usa `python3 scripts/cli.py build`, `test` e `package` sem Qt, X11 ou sessão gráfica. A compilação Linux também exige `libssl-dev pkg-config`; a execução usa OpenSSL e certificados do sistema. Compilações do desktop criam automaticamente um diretório privado de CLI. Consulte o [contrato da CLI](command-line.md) para comandos, instalação e limites de atualização confiável.

Compilações de desenvolvimento continuam experimentais e não são instaladores Release oficiais. Consulte o [README](../../../README.pt-BR.md) da raiz para capacidades e limites e a [política de suporte](support-policy.md) para o escopo atendido. A revisão histórica de `v0.1.0` está em [#4](https://github.com/xiaodaozhi/xdzip/issues/4) e [#53](https://github.com/xiaodaozhi/xdzip/issues/53); versões futuras exigem novos registros de revisão de seus candidatos específicos.

<a id="environment-and-entry-points"></a>

## Ambiente e pontos de entrada

Instale primeiro Python 3.10 ou posterior, Git, CMake 3.24 ou posterior, Ninja, rustup e o compilador/SDK da plataforma. A versão Rust vem de rust-toolchain.toml na raiz; Qt é fixado em 6.8.2. Consulte a [política de ferramentas](toolchain.md) para regras de dependências e atualização. Execute comandos na raiz do repositório. No Windows, substitua `python3` dos exemplos por um `python` disponível e use um ambiente de desenvolvimento MSVC x64.

## Diretórios locais de ferramentas

Ao carregar o ambiente PowerShell do Visual Studio, preserve os valores Unicode de `USERPROFILE`, `LOCALAPPDATA` e `APPDATA` e restaure-os se o carregador alterar a codificação. Qt usa esses locais para caminhos padrão de cache e configuração; valores corrompidos podem causar falhas no sistema de arquivos mesmo com o compilador funcionando.

`prepare` compila e verifica no SDK existente um QtNetwork com patches de segurança para CVE-2026-116337 e CVE-2026-105946; a biblioteca reconstruída não substitui outros módulos de execução. Também são necessários Perl, make (nmake no Windows) e a ferramenta de linha de comando OpenSSL. Ubuntu exige `zlib1g-dev libpcre2-dev libicu-dev libglib2.0-dev libzstd-dev libkrb5-dev`. Código, declarações geradas e caches de compilação ficam sob `build/`, reutilizando por padrão `build/toolchain/qt-network-security/<platform>`. Se os digests de entrada e da biblioteca real não coincidirem, compilação e empacotamento falham imediatamente; execute `prepare` novamente. As [notas de manutenção do patch](../../../patches/qt/README.md) descrevem origem, ABI, funcionamento TLS e recuperação.

Dependências locais da entrada unificada usam somente `build/toolchain/`, ignorado pelo Git e não incluído em commits:

| Conteúdo | Diretório fixo | Observações |
| --- | --- | --- |
| Qt SDK 6.8.2 | `build/toolchain/Qt/6.8.2/<platform-subdirectory>` | `<platform-subdirectory>` é `macos`, `msvc2022_64` ou `gcc_64`; qmllint, qmake, configuração CMake e runtimes Qt vêm daqui |
| Ambiente de ferramentas Python | `build/toolchain/desktop-tools` | Ambiente virtual fixo para aqtinstall e scripts de empacotamento e verificação |
| Preparação de downloads Qt | `build/toolchain/qt-install-tool` | Apenas downloads e entradas temporárias do instalador Qt |

`scripts/desktop.py prepare --install-qt`, `doctor`, `build` e `test` compartilham esses locais por padrão. Não crie cópias separadas `.cache/Qt`, `.cache/desktop-tools` ou semelhantes para cada pipeline de desenvolvimento. Informe `--qt-prefix`, `--qt-root` e `--tools-dir` explicitamente apenas para isolar versões Qt, plataformas ou builds experimentais distintos. A CI pode apontá-los explicitamente para diretórios temporários do runner, mas caminhos alternativos de CI não mudam os padrões locais. CMake, Ninja, rustup, Rust e o compilador C++ da plataforma pertencem às ferramentas do host e são instalados ou chamados conforme os requisitos de verificação prévia, não copiados pelo diretório Qt.

Após um ciclo de desenvolvimento e a limpeza de branches e worktrees, remova os diretórios e arquivos temporários criados pelo ciclo sob `build/`. Mantenha ferramentas compartilhadas reutilizáveis como `build/toolchain/`; remova builds, diagnósticos e preparação de downloads específicos do ciclo. Consulte [gestão de conteúdo e histórico do repositório](../../../developers/engineering/repository-hygiene.md#研发周期工件收尾) para regras e alternativa manual.

| Plataforma | Ambiente do sistema a preparar primeiro | Subdiretório Qt SDK |
| --- | --- | --- |
| macOS | Terminal nativo Apple Silicon, Xcode Command Line Tools e SDK macOS válido | macos |
| Windows | Ferramentas C++ MSVC 2022 x64 e Windows SDK, Developer PowerShell ou linha de comando x64 | msvc2022_64 |
| Ubuntu 24.04 | Compilador C++ x86_64, pacotes de desenvolvimento OpenGL/EGL/xkbcommon e bibliotecas de execução X11 | gcc_64 |

No macOS, `xcode-select --install` prepara as ferramentas de linha de comando. Os pacotes básicos de desenvolvimento Ubuntu incluem `libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev libx11-dev libxext-dev libnautilus-extension-dev pkg-config`; o empacotamento também precisa de patchelf, binutils (readelf e strip) e ferramentas de pacotes Debian, e a verificação isolada de execução precisa de Docker acessível. Instalar o Visual Studio no Windows sem entrar em um ambiente de desenvolvimento pode deixar cl, link, rc ou dumpbin indisponíveis; a verificação prévia identifica as ferramentas realmente ausentes. A entrada compartilhada não instala automaticamente compiladores de sistema, SDKs ou pacotes que exijam privilégios de administrador.

Desenvolvedores Windows podem preparar suas ferramentas pelo [caminho gratuito de licença Community](windows-licensing.md), lendo e aceitando os termos da versão. Apenas Build Tools ou VC_redist não comprova elegibilidade. Uso empresarial e redistribuição associada em runners padrão hospedados pelo GitHub têm autorização pública separada; mantenedores não precisam comprar Enterprise para builds hospedados ou instalar localmente um IDE Community, e essa autorização não se aplica a máquinas autogerenciadas. Ferramentas 2019 existentes não precisam ser desinstaladas para revisão de licença; a compatibilidade de alternativas locais com o SDK 2022 de lançamento atual deve ser verificada separadamente.

Todos os comandos compartilham estas opções:

| Opção | Padrão e finalidade |
| --- | --- |
| `--qt-prefix` | Primeiro `XDZIP_QT_PREFIX`; caso contrário, `build/toolchain/Qt/6.8.2/<platform-subdirectory>`. Aponta para o prefixo do SDK, não para bin |
| `--build-dir` | `build/desktop`, diretório CMake de build e teste |
| `--output-dir` | `build/package`, diretório de artefatos entregáveis e saída de verificação |
| `--tools-dir` | `build/toolchain/desktop-tools`, ferramentas Python isoladas e materiais de preparação |
| `--configuration` | `Release`; outra configuração de desenvolvimento pode ser selecionada explicitamente |

build-dir e output-dir devem ser subdiretórios independentes dentro de build/ do repositório para respeitar os limites de saída de empacotamento e verificação. Mantenha as mesmas opções entre as etapas de uma execução. `--qt-prefix` explícito prevalece sobre a variável de ambiente; não é necessário alterar CMAKE_PREFIX_PATH global. A entrada prepara os ambientes de busca de Qt e ferramentas para subprocessos. Inicializações normais do desktop não devem herdar variáveis de teste fora da tela. Saídas geradas ficam somente em build/, target/, dist/ ou caches ignorados e não são incluídas em commits.

## Execução por etapas

As etapas podem ser executadas independentemente e não repetem implicitamente preparação, build ou teste anteriores. Pré-requisitos ausentes produzem erros; execute a etapa correspondente e tente novamente. Cada etapa verifica o ambiente necessário antes de trabalhar; `doctor` antecipa problemas sem compilar ou empacotar.

Builds protegidos no Windows coordenam processos dentro do console existente. Consoles distintos podem executar em paralelo; builds que compartilham um console existente executam sequencialmente. Durante um build, outro código não deve desconectar ou reconectar processos protegidos desse console. Um console temporário criado por processo que não possui um é para esse processo e seus filhos; processos externos não devem se conectar. Scripts antigos usam um bloqueio alternativo comum para consoles sem handle de janela e não conseguem coordenar com scripts novos; não inicie build de versão nova no mesmo console sem janela até terminar o build antigo. Se listas de processos não puderem ser determinadas, pare sem mudar a página de código.

Para builds não interativos, um wrapper de console independente pode ser usado explicitamente com stdin, stdout e stderr totalmente redirecionados. Exemplo em PowerShell:

```powershell
New-Item -ItemType Directory -Force build\isolated-logs | Out-Null
cmd /c "python scripts\windows_detached_console.py --timeout 7200 -- python scripts\desktop.py build <NUL >build\isolated-logs\desktop.log 2>&1"
$LASTEXITCODE
```

O wrapper cria um console oculto independente por comando e encerra filhos no timeout ou saída do supervisor; ausência de sinais de conclusão causa falha. Builds paralelos ainda exigem valores `--build-dir` separados; o wrapper não bloqueia diretórios de build. As verificações existentes de dependências de desktop-build e `windows_ninja_safety.py` continuam, mas encapsular outros comandos não as adiciona automaticamente. Esta entrada é apenas para comandos não interativos; prompts, entrada de senha e interação com pseudoconsole não são suportados.

`prepare` verifica o inventário atual de fontes de formato do produto. Contêineres ZIP, TAR/GZIP, 7z, RAR, CAB e ópticos usam Rust independente; a lista oficial de fontes do produto está vazia e os motores oficiais UnRAR/7-Zip não são baixados ou compilados. Referências independentes de teste são preparadas a partir de suas próprias fontes fixadas e separadas das dependências de execução do produto. Avisos originais de fonte para mapeamentos RAR limitados permanecem no empacotamento e nos materiais de código correspondente; consulte o [contrato de licença](licensing.md). `build` usa somente entradas preparadas e não baixa versões flutuantes. Para testes apenas de Rust, compile primeiro `xdzip-native-codec`, aponte `XDZIP_NATIVE_CODEC` para esse processo e execute testes do workspace. O fluxo de backend nativo também executa `cargo test -p xdzip-runtime --locked native_backend_tests -- --ignored`; tanto `test` unificado quanto Foundation CI exigem isso. Estado ignored não é uma verificação de formato aprovada. O [contrato de backend nativo](native-codecs.md) descreve fontes, autorização de fluxos e limites de licença.

| Comando | Finalidade e pré-requisitos |
| --- | --- |
| `doctor --for-stage build` | Verifica PATH, Rust fixado, Qt, compilador, SDK e versões; por padrão verifica o ambiente de build |
| `prepare` | Prepara ferramentas e dependências compartilhadas; download/instalação de Qt, Rust e materiais de empacotamento exige opções explícitas abaixo |
| `build` | Configura e compila produto real, testes e diagnósticos; exige ambiente preparado e permite alterações sem commit |
| `test` | Executa por padrão testes do workspace Rust, qmllint e CTest; exige build concluído da mesma configuração e permite alterações sem commit |
| `package` | Implanta runtime, coleta materiais de licença e origem, audita implantação e produz artefatos selecionados; exige código limpo, recibos de build e teste do mesmo commit e ferramentas de empacotamento |
| `verify` | Verifica runtime ou pacote existente sem recompilar, instalar o produto ou alterar associações padrão; exige o artefato e ambiente de verificação da plataforma |

A verificação prévia distingue ferramentas obrigatórias ausentes, versões incompatíveis e ferramentas desnecessárias à etapa atual; ausência de Docker ou Inno Setup não bloqueia builds comuns. A verificação de empacotamento testa de fato a chamada Rust/Cargo e os metadados das dependências bloqueadas para evitar descobrir problemas de PATH ou cache apenas na coleta de licenças. Comandos que falham e limites de verificação permanecem na saída local/CI; conclusão parcial não é informada como aprovada.

Prepare primeiro os toolchains Qt e Rust fixados:

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt --qt-root build/toolchain/Qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

`--install-rust` usa o rustup instalado para preparar o toolchain fixado pelo repositório; não instala rustup. `--install-qt` baixa explicitamente o SDK Qt fixado para `build/toolchain/Qt` por padrão. Um SDK existente não precisa ser baixado novamente; passe seu caminho a cada etapa:

```sh
python3 scripts/desktop.py doctor --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
python3 scripts/desktop.py build --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
```

Após compilar, prepare explicitamente as entradas públicas de teste para o lote completo `native_backend_tests`. Fixtures CAB, ISO/UDF e RAR básicas são mantidas no código do projeto sob `tests/fixtures/cab-iso/` e `tests/fixtures/native-archives/`. Testes 7z criam seus próprios arquivos com o processo Rust recém-compilado. Esse lote de execução não requer ferramentas oficiais 7-Zip/UnRAR ou imagens montadas; outras referências de interoperabilidade Foundation seguem o [contrato de backend nativo](native-codecs.md).

Para obter pela primeira vez as entradas públicas fixadas de RAR PPM, senha antiga e outras, execute explicitamente:

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs
```

Com cache fixado existente verificado por tamanho e SHA-256, use explicitamente o modo offline, substituindo o caminho e mantendo a estrutura do subdiretório `junrar/` do manifesto:

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs --archives /path/to/fixed-rar-cache --offline
```

Fontes e avisos fixados estão em `scripts/rar-rust-fixtures.json` e seu inventário público de compatibilidade. A saída `sources.json.environment` usa caminhos relativos à raiz do código. Os materiais atuais de código correspondente não contêm esses sete arquivos públicos de cache; `--offline` do Cargo não os fornece. O preparador não exporta variáveis de ambiente para a shell. Antes do `test` completo, inicie um processo filho com a seguinte passagem explícita:

```sh
python3 -c "
import json, os, pathlib, subprocess, sys
root = pathlib.Path.cwd()
worker = pathlib.Path(sys.argv[1]).resolve(strict=True)
prepared = json.loads((root/'build/native-test-inputs/sources.json').read_text(encoding='utf-8'))
environment = os.environ.copy()
environment.update({name: str(root/value) for name, value in prepared['environment'].items()})
environment.update(XDZIP_NATIVE_CODEC=str(worker), XDZIP_NATIVE_CODEC_WORKER=str(worker), XDZIP_CAB_BACKEND='rust')
environment['XDZIP_RAR_FIXTURE_PASSWORD'] = 'password'
subprocess.run([sys.executable, 'scripts/desktop.py', 'test', *sys.argv[2:]], env=environment, check=True)
" build/desktop/tools/native-codec/xdzip-native-codec
```

No Windows, acrescente `.exe` ao caminho do processo. Com SDK, build-dir ou tools-dir personalizados, acrescente as mesmas opções após o caminho, por exemplo `--qt-prefix /path/to/Qt/6.8.2/gcc_64`, e ajuste o caminho ao diretório de build real. `XDZIP_CAB_BACKEND=rust` seleciona a verificação Rust CAB fixada; `password` é um valor público existente de fixture passado apenas ao filho de teste. Essa passagem preserva testes padrão do workspace e o lote ignored completo; não baixa entradas nem compila implicitamente o processo.

`test` habilita fora da tela/software apenas para filhos e não contamina comandos posteriores do desktop. Em mudanças rotineiras, recompile e verifique conforme o impacto. A verificação apenas do núcleo Rust pode executar `cargo test --locked` no crate afetado sem Qt. `test --skip-rust` executa qmllint e Qt CTest, mas pula o workspace Rust e os testes completos de backend nativo; recibos marcam Rust como `not-run`. Quando uma conclusão sobre Rust for necessária, execute-o separadamente e registre os resultados reais.

Os testes do navegador podem ser executados diretamente ou em grupos pela entrada existente da suíte. Cada processo cria uma raiz temporária curta e única sob `build/test-tmp/` do projeto. Na saída, após destruir os objetos de teste e antes de destruir o aplicativo Qt, restaura TEMP, TMP e TMPDIR e limpa a raiz. Arquivos de configuração padrão também são isolados. Unix verifica o comprimento em bytes da raiz temporária mais o nome IPC existente contra a capacidade real de sun_path. Caminho longo demais falha explicitamente na pré-condição da fixture; encurte o caminho do checkout em vez de sair do projeto ou mudar a IPC do produto. Processos auxiliares de um grupo herdam a raiz temporária do pai para preservar a identidade do diretório IPC de instância única, usam sondas privadas prévias e terminam antes da limpeza do pai. Grupos distintos não compartilham raízes. Filhos de reinício de idioma e modo de abertura que recebem explicitamente configuração de teste continuam compartilhando-a, sem usar nova configuração vazia. Antes de executar, verifique de fato criação de arquivos filhos, bloqueios, preparação, commits atômicos e leitura da configuração. Falha informa `Fixture precondition` e saída diferente de zero; não pule testes ou enfraqueça asserções de ocultação de janela. Existência do diretório e QTemporaryDir::isValid sozinhos não provam possibilidade de escrita, e permissões do sistema não são alteradas para fazer fixtures passar.

Caminhos de configuração de teste usam salvamento e restauração limitados de `tests/test_environment`. Windows mantém valores de caracteres largos e distingue entradas não definidas de entradas explicitamente vazias no bloco de ambiente do sistema; Unix preserva bytes originais. O setter CRT do MSVC exclui valores vazios, então o bloco nativo é restaurado separadamente. Verifique presença de valor vazio pelo processValue do auxiliar ou QProcessEnvironment, sem prometer a distinção vazio/nulo do CRT de Qt. Esse auxiliar é restrito à thread principal de testes sequenciais: configure antes de criar os motores/processos relevantes e restaure após destruição/junção. Não serve para código do produto ou mudanças concorrentes de ambiente. Caracteres ASCII, chineses e não representáveis em ANSI exigem verificar arquivos reais no caminho original, filhos de restauração de idioma e preservação de janela e rascunho em falhas. Resultados automáticos ficam separados das operações reais de configuração por pessoas.

O produto real usa o nome `xdzip` em todas as plataformas. Entradas padrão de build:

| Plataforma | Ponto de entrada |
| --- | --- |
| macOS | `build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip` |
| Windows | `build/desktop/apps/desktop/xdzip.exe` |
| Linux | `build/desktop/apps/desktop/xdzip` |

O exemplo de componente `xdzip-ui-preview` não serve para aceitação com arquivos reais. Ajuste caminhos ao mudar build-dir. A identidade real do build registra versão, canal, commit e estado com alterações. Consulte o [aplicativo Qt de desenvolvimento](../../../apps/desktop/README.md) para módulos e depuração CMake de baixo nível.

## Geração de fontes da interface

`prepare` instala FontTools/Brotli fixados no ambiente compartilhado e mantém em cache as fontes de interface upstream verificadas por hash. Cada build do desktop coleta automaticamente os catálogos de idiomas suportados e o texto fixo QML/C++ e gera os subconjuntos de fontes sob `<build-directory>/generated/fonts/`. Acrescentar caracteres não exige commit de binário de fonte. Após a preparação, builds funcionam offline; caches ausentes ou corrompidos falham explicitamente. Builds CMake diretos devem selecionar esse ambiente por `Python3_EXECUTABLE`.

Para prévia somente HTML, exportação offline, importação manual de fontes e verificações de fonte, consulte o [guia de fontes compartilhadas](../../../design/assets/fonts/README.en.md). O empacotamento valida o manifesto de fontes do build correspondente sem regenerá-lo. O texto fixo da interface e a seleção de fonte do sistema para dados de usuário mantêm seus papéis existentes.

## Empacotamento e verificação

`test` e `package` verificam recibos de build, impressões digitais do código, configuração, identidade do SDK e arquivos reais do produto. Recompile primeiro após mudanças de código ou configuração. Faça commit do código entregável antes de empacotar e compile e teste a partir desse commit limpo. package não deve contornar verificações ignorando alterações, mudando a identidade do artefato ou reutilizando relatórios antigos. Materiais de licença, metadados Rust e versão real da CLI devem corresponder ao código. Ferramentas de plataforma verificam restrições de uso e sobrescrita de diretórios de pacote existentes. Prefira um output-dir novo para candidatos recompilados, evitando mistura com arquivos antigos.

No Windows/Linux, prepare ferramentas de empacotamento e execute as etapas unificadas:

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py build
python3 scripts/desktop.py test
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

`prepare --packaging` prepara dependências Python de validação isoladas, textos de licença fixados, caches de esquemas e materiais de plataforma; Windows inclui CRT e Inno Setup fixados. Desenvolvedores ainda instalam ferramentas de sistema primeiro. `package --kind runtime` prepara somente o diretório de execução implantado e materiais de auditoria. Depois pode-se executar `package --kind installer` no mesmo output-dir para criar instalador do runtime verificado; Linux também cria tar.gz que preserva permissões de execução. `--kind all` é o padrão e faz ambos em um comando. Empacotamento Windows usa Release ou RelWithDebInfo, sem implantar bibliotecas Qt Debug. Windows fornece diretório implantado e instalador por usuário; Linux fornece diretório de execução, tar.gz e .deb. A verificação Windows exige executável de teste do navegador da mesma configuração e entradas Qt Test/fora da tela. A verificação Linux exige m2-flat.zip gerado pelos testes de build. Arquivos de teste não entram no diretório do produto. verify `--kind runtime` verifica o runtime; o padrão `--kind all` verifica os entregáveis selecionados. A verificação ordinária não instala, atualiza, reverte ou altera associações de arquivos do usuário.

Um DMG macOS exige build principal selecionado separadamente que possa se registrar como candidato a manipulador de documentos, evitando que cópias normais de desenvolvimento ou teste entrem em “Abrir com”:

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py build --build-dir build/desktop-dmg --macos-handler-rank Alternate
python3 scripts/desktop.py test --build-dir build/desktop-dmg
python3 scripts/desktop.py doctor --for-stage package --build-dir build/desktop-dmg
python3 scripts/desktop.py package --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
python3 scripts/desktop.py verify --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
```

O rank padrão do manipulador no build macOS comum é `None` e não muda padrões do usuário; selecione `Alternate` apenas para aplicativo principal duradouro ou entrada de DMG. O pacote local macOS aceita somente `all`, produzindo DMG de desenvolvimento arm64 com assinatura ad-hoc de integridade necessária. verify verifica Cocoa/Metal após realocação e extração real. CI comum não deve executar pacote Mac nem implantar, assinar, comprimir ou enviar pacote de aplicativo; o [protocolo de lançamento](release-protocol.md) descreve a exceção do Release dedicado. Consulte o [guia de implantação Mac](macos-deployment.md) para limites completos e manutenção de registro de cópias.

Consulte [implantação Linux](linux-deployment.md), [DMG Mac](macos-deployment.md) e [instalação e associações](installers-associations.md) para layouts e ferramentas subjacentes. Instaladores Windows/Linux não são formalmente assinados; macOS não tem assinatura Developer ID nem notarização. Diagnostique problemas de instalação/execução com o produto real e a verificação pertinente da plataforma; execução sob o SDK não comprova verificação do pacote independente.

## Relação com CI

Antes de commit ou push, execute builds ou verificações locais adequados ao conteúdo proposto. Mudanças de documentação executam `python3 scripts/check-docs.py` e verificam saídas geradas afetadas. Mudanças Rust verificam formatação e ao menos compilam ou testam os crates afetados. Testes com backends isolados de formato preparam o processo real e as fixtures necessárias conforme descrito acima. Mudanças C++, de ponte e QML compilam o produto afetado com `python3 scripts/desktop.py build` e executam testes pertinentes. Nem toda mudança exige toda a interoperabilidade de formatos ou dez repetições completas da suíte do navegador.

Ao adicionar ou alterar uma janela QML, execute `python3 scripts/check-window-contracts.py --self-test`, `python3 scripts/check-operation-components.py` e regressões afetadas de layout bilíngue com vários tamanhos de fonte. Após um build, `python3 scripts/desktop.py test --skip-rust` executa as verificações completas de componentes Qt e CTest. Como alternativa, após configurar caminhos do SDK Qt e ambiente fora da tela, execute casos pertinentes por entradas existentes como `ctest --test-dir build/desktop -R qt_task_components --output-on-failure`. Mudanças de botões de opção/caixas de seleção também executam regressões QAccessibleActionInterface apropriadas, como `accessibleRadioActionsCommitSelection` e `accessibleCheckboxActionsCommitIntent`.

A [CI rotineira](../../../.github/workflows/docs.yml) executa apenas verificações estáticas leves e não substitui builds locais do produto. [Foundation](../../../.github/workflows/foundation.yml) e o [workflow Qt](../../../.github/workflows/qt-components.yml) são executados manualmente sob demanda. Qt usa o mesmo desktop.py para builds, testes Qt e empacotamento e verificação de execução Windows/Linux. [ci-desktop.ps1](../../../scripts/ci-desktop.ps1) mantém verificações sob demanda de repetição, visual, interoperabilidade e compatibilidade de instalação e não é obrigatório na verificação local rotineira.

Ciclos históricos de produto completo, instalação real com atualização/rollback e associações executam apenas em hosts CI Windows/Linux descartáveis ou contêineres isolados; a entrada unificada não os torna padrão do verify local comum. O job macOS no workflow Qt sob demanda usa auditoria de implantação em modo SDK e verificações Cocoa de inicialização/abertura de arquivo, mantendo apenas logs e JSON. Consulte a [política de CI](../../../developers/engineering/ci-policy.md) para verificações e limites de evidência completos.

## Limites de arquivos e tarefas

Abrir permite apenas navegar; não extrai automaticamente. Arquivos regulares de mesmo nome oferecem perguntar, pular, renomear e substituir explicitamente. Publique um arquivo inteiro apenas após verificar e sincronizar o conteúdo. Cancelar antes do commit ou falha de escrita preserva o arquivo antigo; entradas extraídas já confirmadas permanecem. Fechar a janela de tarefa apenas oculta detalhes. A saída usa a decisão sobre tarefas ativas e a barreira assíncrona de desligamento.

ZIP pode restaurar links simbólicos relativos e bits executáveis em condições POSIX restritas, mas não bits de privilégio, atributos estendidos ou datas originais de modificação. Isso não comprova instalação de pacote de aplicativo ou verificação de assinatura. Origem e destino devem cumprir o [contrato de extração segura](zip-safe-extraction.md). Staging após falha é apenas informado e mantido para inspeção; conteúdo desconhecido não é recuperado ou excluído automaticamente.

## Microsoft Store MSIX

Para envio à Store, siga o [procedimento do mantenedor](../../../developers/market/windows/build.md).

<a id="windows-msix-实验构建"></a>

## Build experimental de MSIX Windows

MSIX reutiliza o build Windows Release do desktop, testes e diretório de execução auditado do mesmo commit. Conclua `prepare --packaging`, `build` e `test` como acima e execute com as mesmas opções:

```powershell
python scripts/desktop.py package --kind runtime
python scripts/desktop.py package --kind msix --msix-name xdzip.Development --msix-publisher 'CN=xdzip development' --msix-publisher-display-name xdzip
python scripts/desktop.py verify --kind msix
```

A identidade de exemplo é apenas para desenvolvimento local. Nome de pacote, publicador e nome de exibição de uma identidade Store formal devem coincidir com o Partner Center. A saída `build/package/xdzip-windows-x64.msix` é um pacote experimental sem assinatura, não automaticamente distribuível por sideload ou certificado pela Store. Sideload normal exige assinatura confiável pelo dispositivo que corresponda ao publicador. O modo restrito de desenvolvimento sem assinatura do Windows 11 exige ainda um OID de publicador específico e, para conteúdo executável, geralmente direitos de administrador; a identidade de exemplo não atende a esse modo. Os scripts não criam certificados, assinam, instalam ou enviam à Store. O mínimo técnico é Windows 10 22H2 x64 e não amplia os compromissos de plataformas suportadas. A versão `1.0.<product build>.0` é separada da versão exibida; a política de versões da Store deve ser revista antes de esgotar números de build.

O build exige código limpo, recibos correspondentes de build/teste, diretório de execução completamente auditado e MakeAppx e MakePRI do Windows SDK. Os scripts reverificam o runtime e seus marcadores de atualização, associação e menu, geram manifesto e recursos de ícones em staging separado, comparam cada arquivo empacotado e registram hashes. Não sobrescrevem pacotes ou staging existentes. O manifesto declara os tipos de extensão existentes no registro de formatos, uma entrada desktop de confiança total e classes de menu específicas do canal.

Ícones do aplicativo incluem variantes de escala e tamanhos exatos de destino nas formas padrão, sem placa para fundo escuro e sem placa para fundo claro. MakePRI indexa esses recursos em `resources.pri`, permitindo que Windows selecione o tamanho desejado sem placa de sistema. Associações declaram logotipo próprio por formato e reutilizam quadros PNG de 16–256 pixels de cada ICO Windows existente sem mudar arte, tamanho visível, margens ou posição. A política está em [msix-icons.json](../../../specs/visual/msix-icons.json). Esses recursos pertencem ao pacote instalado; logotipos da ficha Partner Center são separados. `verify --kind msix` compara novos recibos de ícones com o pacote real, incluindo dimensões, variantes completas, imagens canônicas de formato e presença de índice de recursos. Alterações chegam aos usuários instalados somente após compilar e entregar um pacote atualizado.

O exemplo usa `--msix-channel sideload` por padrão, registrando apenas o menu clássico do Windows 11. Para enviar à Store use `package --kind msix --msix-channel store` com a identidade exata do Partner Center. Isso declara menus moderno e clássico; a entrada moderna permanece oculta até o pacote ter realmente uma assinatura de sistema Store. Marcadores de canal são adicionados no empacotamento e não exigem recompilar o runtime auditado. Mantenha saídas em diretórios separados. Consulte a [política de canais de menu](shell-actions.md#limite-do-pacote-windows). Licenças, instalação real, associações, desinstalação, assinatura Store e aceitação real no sistema de destino são separadas do sucesso do empacotamento.

O manifesto MSIX declara os dez idiomas do registro compartilhado, correspondentes aos catálogos internos; recibos registram e verificam a lista. Esse fluxo MakeAppx não converte automaticamente recursos de tradução Qt em declarações de idioma do pacote. “Idiomas suportados pelo pacote” no Partner Center reflete o pacote enviado; após corrigir o código é necessário recompilar e substituir o pacote no rascunho para atualizá-lo. “Idiomas adicionais da ficha Store” são configurados separadamente e não substituem declarações do manifesto. Consulte a [documentação Microsoft de idioma do manifesto](https://learn.microsoft.com/windows/apps/design/globalizing/manage-language-and-region).

## Evidência e nova verificação

Selecione verificações afetadas conforme [verificação geral](../../../developers/qa/README.md) e [verificação de desktop sob demanda](../../../developers/qa/private-test-manual.md); trabalho rotineiro não exige formulário de aceitação manual. Recibos de build e teste são `desktop-build.json` e `desktop-test.json` sob build-dir. Recibos de preparação, empacotamento e verificação são `<stage>.json` sob output-dir e vinculam evidências subjacentes. Recibos automáticos registram etapas e fontes reais. Renderização fora da tela/software, verificações Cocoa nativas e verificações manuais realizadas são registradas separadamente e não substituem umas às outras. A aceitação existente permanece em [#5](https://github.com/xiaodaozhi/xdzip/issues/5) e não é repetida apenas porque um inventário de candidatos foi regenerado.

A entrega de artefatos Windows/Linux inclui código do mesmo commit e avisos de licença; Actions mantém artefatos por 14 dias. Após expirar, reconstrua usando identidades e entradas fixadas. deployment-audit.json e inventários CycloneDX por fonte/arquivo ajudam a verificar origem, mas podem ser incompletos e não comprovam aprovação de licença. Consulte [entrega de código](source-delivery.md) e o [contrato de verificação automatizada](../../../developers/engineering/candidate-verification.md) para reconstrução de código e substituição de Qt.

Builds, capturas, medições, relatórios e pacotes permanecem apenas como artefatos locais ignorados ou de CI. Não descreva formulários pendentes gerados, verificações fora da tela ou SBOMs parciais como aprovações de ambiente limpo e de todas as condições de lançamento.

## Framework de atualização macOS

Builds nativos usam `prepare-sparkle.py` para preparar `build/toolchain/Sparkle/2.10.0-xdzip-proxy.2`. A biblioteca principal do framework é compilada do código upstream fixado e do patch de proxy; auxiliares de instalação, recursos e ferramentas de assinatura mantêm origem do SDK oficial. Consulte o [contrato de dependências](dependencies.md#macos-更新提供者). Recibos de build válidos podem ser reutilizados; um SDK oficial antigo não substitui o framework derivado com interfaces de proxy. A CI pode selecionar explicitamente um diretório temporário de SDK.

`python3 scripts/prepare-sparkle.py --output build/toolchain/Sparkle/2.10.0-xdzip-proxy.2` usa Clang nativo do Xcode Command Line Tools ou Xcode existente e não precisa de SDK Qt separado. Arquivos originais do SDK e código são mantidos em cache em `build/source-cache/`; a operação offline pode receber `--archive` e `--source-archive` separadamente. O build não executa scripts do código baixado, altera proxies do sistema, acessa chaves de assinatura de produção ou coleta arquivos de entrega de código do projeto.

`check-macos-proxy.py` verifica por padrão apenas o downloader próprio via loopback. `--system-conflicts` é restrito a runners Apple Silicon hospedados descartáveis em workflow dedicado. Um auxiliar restrito prepara, restaura e verifica as configurações de proxy do sistema e hosts de domínios de teste fixos. Não adicione essa opção localmente.
