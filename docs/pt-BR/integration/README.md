# Usar o XDZip em scripts

[English](../../integration/README.md) | [简体中文](../../zh-CN/integration/README.md) | [繁體中文（台灣）](../../zh-TW/integration/README.md) | [日本語](../../ja-JP/integration/README.md) | [한국어](../../ko-KR/integration/README.md) | [Français](../../fr-FR/integration/README.md) | [Deutsch](../../de-DE/integration/README.md) | [Español](../../es-ES/integration/README.md) | Português (Brasil) | [Русский](../../ru-RU/integration/README.md)

A CLI independente funciona sem Qt ou sessão de desktop. Escolha um pacote `xdzip-cli-` para seu sistema nas [versões do GitHub](https://github.com/xiaodaozhi/xdzip/releases). Descompacte o pacote inteiro e deixe o programa auxiliar ao lado do executável. No diretório extraído, execute `./bin/xdzip --help` no macOS ou Linux, ou `.\bin\xdzip.exe --help` no PowerShell.

Os exemplos usam `xdzip` no PATH. Execute `setup` com a CLI extraída para registrar um inicializador para seu usuário. Se necessário, adicione o diretório do comando ao PATH. Use `xdzip --version-json` para confirmar que o programa encontrado informa `"component":"cli"`. O inicializador de desktop e a CLI podem ter o mesmo nome de comando. Se houver conflito, use o caminho explícito da CLI extraída. [Instalar e remover o comando](../engineering/command-line.md#xdzip-build-and-command-installation)

<a name="xdzip-create-and-check"></a>

## Criar e verificar

Em um diretório de trabalho novo, privado e gravável, crie uma pasta chamada `docs` contendo `hello.txt`. Coloque um segundo arquivo, `notes.txt`, ao lado da pasta. Use a sequência abaixo para criar um ZIP, adicionar o segundo arquivo à pasta `docs/` existente no ZIP e verificar o resultado.

```sh
xdzip create sample.zip docs --level normal --json
xdzip add sample.zip notes.txt --into docs --conflict skip --json
xdzip test sample.zip --json
```

Escolha a extensão e as opções de criação na [tabela de formatos](../../../README.pt-BR.md#xdzip-format-support). Mantenha a saída fora da árvore de entrada e use uma pasta pai existente. `add` exige um arquivo compactado existente e compatível. Use `--into` para incluir arquivos em uma pasta existente do arquivo compactado. Sem essa opção, a inclusão ocorre na raiz. `test` não grava arquivos extraídos. Em caso de êxito, `data.check_kind` indica o alcance da verificação. Consulte o [registro de formatos](../engineering/format-registry.md) e as [verificações de integridade](../engineering/archive-checking.md).

<a name="xdzip-list-select-and-extract"></a>

## Listar, selecionar e extrair

Continue com o arquivo acima no mesmo diretório de trabalho. Use um novo destino ou escolha explicitamente como tratar arquivos existentes.

```sh
xdzip list sample.zip --limit 2 --json
xdzip extract sample.zip docs/hello.txt -C output --conflict skip --json
```

Os caminhos dos membros são exatos, distinguem maiúsculas de minúsculas e usam barras normais. Se vários registros tiverem o mesmo caminho, selecione pelo `id` retornado por `list`. Os IDs começam em 1. Por exemplo, substitua `docs/hello.txt` por `--entry-id 2` no segundo comando se a lista atribuir o ID 2 ao arquivo. Repita `--entry-id` para mais membros. Use `path` para localizar membros e `display_name` para exibi-los. Gere nova lista após alterar o arquivo compactado.

A primeira página começa no deslocamento 0. Passe `data.next_offset` a `--offset` com o mesmo `--limit` para ler a próxima página. Pare quando `next_offset` for null. O deslocamento é a posição de uma linha, contada a partir de zero. O ID identifica um registro físico. Cada comando abre o arquivo novamente. Mantenha a origem inalterada entre chamadas paginadas. [Limites de seleção e listagem](../engineering/command-line.md#xdzip-archive-behavior)

<a name="xdzip-run-without-prompts"></a>

## Executar sem perguntas

`--json` desativa as perguntas interativas. Quando uma tarefa precisar terminar sem supervisão, indique uma política de conflitos. Para um comando que aceita senhas, como `extract`, `test` ou `create` com criptografia, use `--password-stdin` e envie uma linha UTF-8 com a senha por um canal privado de entrada padrão. A criação criptografada também exige `--encrypt`. Se faltar uma decisão ou senha obrigatória, a CLI retorna `interaction_required`. [Opções de senha e formato](../engineering/command-line.md#xdzip-archive-behavior)

<a name="xdzip-interpret-results"></a>

## Interpretar os resultados

Para operações em arquivos compactados com `--json`, interprete o objeto UTF-8 em stdout. Leia `schema_version`, `status`, `error_code`, `data`, `summary` e `warnings`. Mantenha stderr disponível para diagnóstico. `--version-json` retorna um objeto separado com a identidade da compilação.

| Código de saída | Tratamento |
| --- | --- |
| 0 | Confira o resultado e o total de arquivos ignorados. Em comandos que encaminham operações ao desktop, `handed_off` confirma a entrega. |
| 1 | Consulte o erro, os avisos e o resumo para verificar falha, conclusão parcial ou avisos de limpeza. |
| 2 | Corrija o comando ou os argumentos. |
| 3 | Confira o requisito informado de interação, autorização, plataforma ou instalação. |
| 130 | O cancelamento foi confirmado. Confira se há arquivos já gravados em definitivo. |

Valores desconhecidos podem ser null. Uma falha ou um cancelamento pode preservar arquivos já gravados. Antes de tentar novamente, confira `summary.committed_files`, quando disponível. A [referência da CLI](../engineering/command-line.md) descreve o encaminhamento ao desktop, os comprovantes de atualização e o formato completo da saída.
