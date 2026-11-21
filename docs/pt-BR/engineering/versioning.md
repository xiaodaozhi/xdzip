<a name="xdzip-product-version-and-build-identity"></a>

# Versão do produto e identidade da compilação

[English](../../engineering/versioning.md) | [简体中文](../../zh-CN/engineering/versioning.md) | [繁體中文](../../zh-TW/engineering/versioning.md) | [日本語](../../ja-JP/engineering/versioning.md) | [한국어](../../ko-KR/engineering/versioning.md) | [Français](../../fr-FR/engineering/versioning.md) | [Deutsch](../../de-DE/engineering/versioning.md) | [Español](../../es-ES/engineering/versioning.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/versioning.md)

O campo opcional `prerelease_number` separa a sequência de pré-lançamento do número de compilação. Por exemplo, alpha.1 pode usar a compilação 14 enquanto o número de compilação continua subindo em alpha.2. Sem esse campo, vale a interpretação histórica `channel.build`. O [protocolo de lançamento](release-protocol.md) define manifestos e nomes de arquivos.

[VERSION.json](../../../VERSION.json) é a única fonte mantida da versão, do canal e do número de compilação. A compilação lê os valores atuais. Veja o [histórico de alterações](../../../CHANGELOG.md) para as versões e as [issues](https://github.com/xiaodaozhi/xdzip/issues) para entregas e verificações. Aumentar a versão não aprova um lançamento.

As verificações de compatibilidade usam fontes históricas fixas junto da compilação atual para testar atualização, reversão e configurações. O [script de preparação do produto anterior](../../../scripts/prepare-previous-product.py) mantém a referência completa. O [script de compatibilidade das configurações](../../../scripts/check-settings-compat.py) mantém a referência de configurações. Versões e commits atuais não são repetidos aqui. O esquema das configurações e a versão do produto são gerenciados separadamente.

- `version` segue `major.minor.patch` sem zeros à esquerda. Mudanças incompatíveis, recursos compatíveis e correções usam os componentes correspondentes. Os limites de compatibilidade também precisam ser explicados em 0.x.
- `channel` aceita development, alpha, beta, rc e stable. Versões não estáveis aparecem como `version-channel-abbreviation.build`, com development abreviado para dev. Versões estáveis exibem a versão base.
- `build` é um inteiro de 1 a 65535. Incremente-o para uma nova compilação de entrega externa. A compilação local não incrementa o valor automaticamente e a alteração não cria automaticamente uma tag nem um lançamento. Para recursos Windows, cada componente numérico também está limitado a 65535.
- CMake lê o JSON diretamente. `python3 scripts/sync-version.py` sincroniza as versões derivadas do workspace Cargo. A CI usa `--check`. Pacotes Rust internos herdam a versão do workspace. Atualize Cargo.lock depois de sincronizar e execute `cargo check --workspace --locked`.
- O recurso numérico Windows de quatro partes usa versão base e número de compilação. A versão textual mantém o canal. No macOS, `CFBundleShortVersionString` contém a versão base, `CFBundleVersion` a compilação e a versão longa conserva o canal. Siga a [documentação da Apple](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleversion). Use o formato numérico de cada plataforma sem inserir commits Git nos campos numéricos do sistema.
- Em Windows, `FileDescription` é `xdzip Archive Manager` em locais como o Gerenciador de Tarefas. `ProductName` continua `xdzip`. O nome do executável e a identidade das associações independem do nome exibido. Os testes leem os recursos do EXE real.
- A compilação de desktop gera um hash Git curto e um indicador de alterações não registradas. Sem Git o valor é desconhecido. Nenhum caminho da máquina, nome de usuário ou registro de hora instável é incluído. O cabeçalho só muda quando seu conteúdo muda. Recompilar após o commit gera a identidade limpa correta.

As versões do módulo QML, protocolo da ponte e esquema das configurações são independentes da versão do produto.

`xdzip --version`, Configurações → Sobre e o aplicativo de prévia leem a mesma informação. A CLI processa argumentos de versão antes de criar a interface gráfica. Sobre mostra versão, compilação, commit, versão Qt efetiva, sistema e arquitetura. Canais de desenvolvimento e alterações ainda não registradas são identificados. O JSON copiado não contém arquivos pessoais, caminhos, nomes de máquina ou credenciais. O botão do projeto abre o navegador por ação do usuário.

O ícone grande de Sobre usa o PNG de 512 px da plataforma.

A página Sobre usa as dimensões lógicas de Configurações sem rodapé. Oferece os [idiomas de interface incluídos](../ux/settings-localization.md), temas claro e escuro e tamanhos 100%, 125% e 150%. Veja a tela Sobre. Valores nativos de Qt e commit ficam como desconhecidos se indisponíveis.

As verificações usam `scripts/check-build-version.py` para CLI e recursos nativos, `scripts/test-version.py`, `BrowserTest::versionAboutWindow`. Capturas e medições ficam em `build/review/about/`. Fontes, metadados do sistema e versões em execução precisam coincidir. O número de versão não comprova assinatura, aceitação manual ou compatibilidade.
