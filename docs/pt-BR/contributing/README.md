# Compilar e contribuir com o XDZip

[English](../../contributing/README.md) | [简体中文](../../zh-CN/contributing/README.md) | [繁體中文（台灣）](../../zh-TW/contributing/README.md) | [日本語](../../ja-JP/contributing/README.md) | [한국어](../../ko-KR/contributing/README.md) | [Français](../../fr-FR/contributing/README.md) | [Deutsch](../../de-DE/contributing/README.md) | [Español](../../es-ES/contributing/README.md) | Português (Brasil) | [Русский](../../ru-RU/contributing/README.md)

Leia as [regras de contribuição](../../../CONTRIBUTING.md) e encontre ou abra uma issue para a mudança. Descreva o comportamento esperado, as plataformas afetadas e um exemplo pequeno e verificável. Use arquivos de teste do projeto ou que possam ser redistribuídos. A [estrutura do repositório](../engineering/repository-layout.md) mostra onde fica cada parte da implementação.

<a name="xdzip-build-the-part-you-need"></a>

## Compilar a parte necessária

Execute os comandos na raiz do repositório. No Windows, use `python` em um ambiente de desenvolvimento MSVC x64. No macOS, use um terminal Apple Silicon nativo e as ferramentas de linha de comando do Xcode. No Ubuntu, siga a lista de dependências da plataforma. O [guia de compilação](../engineering/developer-build.md#xdzip-environment-and-entry-points) detalha os requisitos e as versões de Rust e Qt fixadas pelo repositório.

Para a primeira compilação do desktop:

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

As opções de preparação usam o rustup existente para instalar as ferramentas fixadas e baixam o SDK do Qt fixado quando necessário. Reutilize as dependências compartilhadas de `build/toolchain/` se já estiverem prontas. Com um SDK do Qt compatível instalado em outro local, passe o mesmo `--qt-prefix` às etapas aplicáveis. Quando a mudança precisar dos testes de desktop, execute `python3 scripts/desktop.py test` com uma compilação concluída da mesma configuração. Consulte a [preparação de entradas de teste](../engineering/developer-build.md) para as suítes completas do programa auxiliar.

Para a CLI independente, compile e teste a CLI gerada e seu programa auxiliar:

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
```

A saída padrão é `build/cli`. A compilação usa as ferramentas fixadas de Rust, CMake e as ferramentas C++ da plataforma. No Ubuntu, também exige `libssl-dev` e `pkg-config`. A [especificação de compilação da CLI](../engineering/command-line.md#xdzip-build-and-command-installation) detalha as plataformas.

<a name="xdzip-verify-the-affected-behavior"></a>

## Verificar o comportamento afetado

Reutilize uma compilação adequada, quando disponível. Para uma mudança apenas na documentação, comece pelas verificações abaixo e confira a saída dos geradores afetados. Faça uma compilação completa do desktop quando a mudança ou as verificações necessárias exigirem.

```sh
python3 scripts/check-docs.py
git diff --check
```

| Mudança | Código e especificação | O que executar ou verificar |
| --- | --- | --- |
| Documentação e exemplos | Atualize as traduções, os links e as âncoras de seção. O README raiz e `docs/` têm dez versões de idioma. Os guias de `developers/` usam inglês dos Estados Unidos e chinês simplificado. Use a especificação correspondente e confira os comandos alterados com um pequeno arquivo de teste próprio. | Execute a verificação de documentação acima. Abra os links de seção alterados e confira a saída gerada quando um gerador usar a página. |
| CLI | Comece em `crates/xdzip-cli` e siga o ambiente Rust compartilhado. [Especificação da CLI](../engineering/command-line.md) | Execute as verificações da CLI independente na sua compilação. Se comandos ou saídas mudarem, confira stdout, stderr, o código de saída e os arquivos gravados, incluindo um caso de falha aplicável. |
| Domínio Rust, formatos ou tarefas de arquivo | Comece em `xdzip-domain`, no codec correspondente, em `xdzip-runtime` ou em `xdzip-io`. Para um comando do desktop, siga `BrowserController` e `RuntimeConnection`. [Registro de formatos](../engineering/format-registry.md), [execução](../engineering/product-runtime.md), [segurança de arquivos](../design/file-security.md) | Execute `cargo fmt --all -- --check` e os testes dos crates afetados. Verifique o fluxo real e as falhas, cancelamentos ou gravações definitivas pertinentes. Se usar o programa auxiliar nativo de formatos, execute os [testes desse programa](../../../developers/qa/runtime-worker-tests.md). Quando o registro mudar, regenere com `python3 scripts/generate-format-registry.py`, confira com `--check` e execute `python3 scripts/test-format-registry.py`. |
| Janelas QML, controles ou localização | Comece em `apps/desktop/ui`, `resources/i18n` e `specs/visual`. Antes de mudar uma janela, leia a [especificação visual](../visual/design-standard.md) e o [processo de design](../visual/design-workflow.md) e registre seu contrato no [registro de janelas](../../../specs/visual/window-registry.json). | Para janelas, execute `python3 scripts/check-window-contracts.py --self-test`, `python3 scripts/check-operation-components.py` e os testes bilíngues de layout afetados com vários tamanhos de texto. Para localização, execute `python3 scripts/check-ui-i18n.py`. Quando exigido, confira a interação ou aparência nativa na plataforma relevante. |
| Instalação, integração ou atualizações | Consulte o [guia de compilação e implantação](../engineering/developer-build.md), a [especificação de associações](../engineering/installers-associations.md) e a de [atualizações](../engineering/software-update.md). | Confira a instalação ou o registro afetado, a identidade do pacote real, os erros e a recuperação. Use a [política de CI](../../../developers/engineering/ci-policy.md) para localizar o fluxo da plataforma. |

<a name="xdzip-describe-the-evidence"></a>

## Descrever as verificações

Na PR, informe a mudança de comportamento, os comandos e plataformas testados e as verificações restantes. Identifique os resultados sem tela, no desktop nativo e da instalação. O [guia de verificação](../../../developers/qa/README.md) explica o alcance de cada um. As especificações dos subsistemas estão na [referência técnica](../reference/README.md), e os procedimentos de publicação, no [manual de manutenção](../../../developers/README.md).
