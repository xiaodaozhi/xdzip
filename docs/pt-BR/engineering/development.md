<a name="xdzip-developer-preparation-guide"></a>

# Preparação do ambiente de desenvolvimento

[English](../../engineering/development.md) | [简体中文](../../zh-CN/engineering/development.md) | [繁體中文](../../zh-TW/engineering/development.md) | [日本語](../../ja-JP/engineering/development.md) | [한국어](../../ko-KR/engineering/development.md) | [Français](../../fr-FR/engineering/development.md) | [Deutsch](../../de-DE/engineering/development.md) | [Español](../../es-ES/engineering/development.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/development.md)

Use [Compilar e contribuir](../contributing/README.md) para escolher a compilação e as verificações adequadas. A [arquitetura](../design/system-architecture.md) e a [estrutura do repositório](repository-layout.md) mostram os limites de Qt, da execução em Rust e dos codecs.

<a name="xdzip-obtain-and-modify"></a>

## Obter e modificar o código

Comece pelo `main` atual e preserve as outras alterações no diretório de trabalho. A [política de contribuição](../../../CONTRIBUTING.md) define o escopo e a revisão. O [guia de entrega](../../../developers/engineering/repository-hygiene.md) trata da integração autorizada. Quando um contrato mudar, atualize suas edições em outros idiomas e os testes afetados. Use `specs/visual` para cores e geometria.

<a name="xdzip-documentation-builds"></a>

## Gerar e verificar a documentação

Mantenha um README na raiz e páginas correspondentes em docs para cada idioma de interface oferecido. As páginas padrão em inglês usam `README.md` e `docs/`. As traduções usam `README.<locale>.md` e `docs/<locale>/`. O manual de manutenção usa inglês dos EUA e chinês simplificado. Coloque a seleção de idioma abaixo do título. Mostre o idioma atual como texto e vincule as demais versões do mesmo assunto. Uma página em chinês simplificado aponta para a página de manutenção em chinês simplificado. As demais apontam para o inglês padrão. Ao adicionar ou mover documentos, atualize índices, links de entrada e âncoras.

As ferramentas de documentação exigem Python 3.10+, lxml, Pandoc e Graphviz. Recomenda-se instalar fontes CJK do sistema. Execute `python3 scripts/build-design-html.py` para gerar 16 documentos detalhados de projeto e uma edição em um único arquivo sob build/docs-html a partir do chinês em `docs/zh-CN/design/`. O pacote para download é gravado em dist/. Os outros guias técnicos e de colaboração estão em docs/README.md. A saída em HTML é a documentação. A interface do produto usa Qt Quick.

Execute `python3 scripts/check-docs.py` para verificar a cobertura de traduções, os links recíprocos de idioma, o idioma de destino dos links, as âncoras Markdown, JSON e os recursos de design. A verificação funciona offline. O gerador HTML confere a estrutura e os links das páginas que cria. Execute `python3 scripts/test-check-docs.py` ao mudar o verificador.

<a name="xdzip-product-toolchain-preparation"></a>

## Preparar as ferramentas do produto

No macOS, prepare Xcode Command Line Tools e Qt. No Windows, use MSVC Build Tools, Windows SDK e Qt compatível. No Linux, são necessários um compilador, o ambiente glibc do destino e as dependências Qt Wayland/X11. As versões de desenvolvimento são fixadas nos arquivos da raiz e na CI. Registre fontes e licenças na [tabela de ferramentas](toolchain.md). A [política de suporte](support-policy.md) informa sistemas e manutenção. Use a issue de revisão da versão para questões específicas de origem ou licenças.

O [aplicativo Qt de desenvolvimento](../../../apps/desktop/README.md) documenta os comandos para compilar e executar em uma cópia limpa, opções CMake, arquitetura e destino do macOS e limites dos testes sem tela. Após mudanças de compilação ou instalação, teste de novo caminhos com espaços e caracteres chineses. O desenvolvimento local dispensa identidade de assinatura paga. O [guia de compilação](../engineering/developer-build.md) descreve a assinatura ad hoc do DMG local e o limite da CI que exclui a implantação do aplicativo Mac.

<a name="xdzip-independent-engineering-probes"></a>

## Testes técnicos independentes

A [sonda de ferramentas](../../../tools/toolchain-probe/README.md) oferece uma compilação mínima e comandos CTest para uma janela Qt Quick autônoma. O README da sonda descreve versões e escopo. Os resultados ficam na [issue de validação inicial](https://github.com/xiaodaozhi/xdzip/issues/1).

A [sonda unificada CMake/Cargo da ponte](../../../tools/bridge-probe/README.md) oferece uma entrada de compilação na raiz, dependências fixadas e verificações de ciclo de vida. Veja a [justificativa para cxx](cxx-admission.md) para os registros de dependências.
