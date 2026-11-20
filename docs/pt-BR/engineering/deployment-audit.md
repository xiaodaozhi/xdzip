<a name="xdzip-deployment-dependency-verification-and-sbom"></a>

# Verificação de dependências implantadas e SBOM

[English](../../engineering/deployment-audit.md) | [简体中文](../../zh-CN/engineering/deployment-audit.md) | [繁體中文](../../zh-TW/engineering/deployment-audit.md) | [日本語](../../ja-JP/engineering/deployment-audit.md) | [한국어](../../ko-KR/engineering/deployment-audit.md) | [Français](../../fr-FR/engineering/deployment-audit.md) | [Deutsch](../../de-DE/engineering/deployment-audit.md) | [Español](../../es-ES/engineering/deployment-audit.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/deployment-audit.md)

A verificação de implantação gera por padrão materiais de revisão de desenvolvimento explicitamente marcados como `incomplete`; ela não comprova automaticamente prontidão para lançamento. O Release oficial coleta separadamente materiais do mesmo candidato e os entrega com o pacote após revisão do processo de lançamento. O formato usa o [esquema oficial CycloneDX 1.6](https://github.com/CycloneDX/specification/blob/55343ba19dee1785acf1ce9191540d5fd7b590db/schema/bom-1.6.schema.json). O commit original e o SHA-256 de três arquivos de esquema estão fixados na [configuração](../../../scripts/sbom-schemas.json). A validação baixa e analisa apenas esses recursos, armazenando-os em build/; não baixa esquemas de URLs arbitrárias em um SBOM.

| Artefato | Fonte dos dados e finalidade | O que não pode comprovar |
| --- | --- | --- |
| source-build.cdx.json | Grafo Cargo de dependências do espaço de trabalho resolvido para o host, incluindo ferramentas de desenvolvimento e compilação; módulos Qt declarados; hashes das fontes de fontes tipográficas e marca | Que um pacote Rust esteja necessariamente vinculado estaticamente ao produto ou que todos os plugins Qt e terceiros incorporados estejam identificados |
| deployment-audit.json | SHA-256 dos arquivos sob a raiz real indicada, importações binárias nativas, dependências SDK, do sistema ou não resolvidas e identidade real da compilação CLI | Todos os carregamentos dinâmicos de QML ou plugins por nome, instalação limpa, ABI do sistema, assinaturas ou conformidade de licença |
| artifact-files.cdx.json | Artefatos reais e arquivos SDK externos observados, relações conhecidas de importações nativas, vinculados ao inventário de fontes e à versão, commit e estado dirty do executável real | Identidade completa dos componentes implantados, atribuição de vinculação estática, licenças de bibliotecas incorporadas ao Qt ou SBOM formal de produto concluído |
| Registros opcionais de desktop | Verificações de desktop executadas e limitações conforme a mudança e o ambiente disponível | Ações não executadas, outras plataformas ou aprovação de lançamento |

Registros de fontes não incluem IDs opacos do Cargo baseados em caminhos nem inventam purls do crates.io para pacotes do espaço de trabalho. Caminhos de artefatos são relativos à raiz indicada ou ao SDK Qt. Outras dependências externas são identificadas por hash de conteúdo e nome de arquivo, sem caminhos absolutos privados do SDK. Fontes tipográficas usam OFL-1.1; fontes originais de marca preservam Apache-2.0 da biblioteca de ativos. Declarar módulos Qt não permite inferir licenças de todos os componentes realmente implantados.

<a name="xdzip-automated-verification"></a>

## Verificação automatizada

`deployment_audit.py` verifica somente compilações confiáveis do projeto explicitamente indicadas, não arquivos compactados de usuários. macOS usa otool para importações e RPATH; Windows usa dumpbin do MSVC para importações DLL comuns e atrasadas; Linux usa ldd, que não deve ser usado em executáveis de origem desconhecida. Um link simbólico de arquivo que escape da raiz indicada do artefato faz a verificação falhar. Relatórios devem ser gravados fora da raiz para não incluir sua própria saída no inventário.

- `--mode sdk` permite resolução para um SDK Qt externo e mostra explicitamente que a compilação não é independente. Importações não resolvidas continuam falhando. A recursão cobre o artefato e o SDK Qt declarado; outras bibliotecas externas são registradas apenas por hash e contagem não percorrida, sem serem consideradas verificadas como fechamento.
- `--mode deployed` proíbe dependências externas não pertencentes ao sistema e importações não resolvidas. No Windows, importações VC++ como MSVCP e VCRUNTIME devem vir do pacote; o System32 da máquina de desenvolvimento não pode ocultar arquivos ausentes. UCRT e contratos de API do sistema continuam fornecidos pelo sistema. Continue os testes nativos básicos de inicialização com os caminhos de pesquisa de desenvolvimento Qt removidos.
- Importações não resolvidas, divergências de versão, commit ou estado dirty entre fonte e binário real, erro de hash ou formato do esquema oficial e referências de componentes duplicadas ou pendentes bloqueiam a verificação correspondente.
- O fechamento de importações nativas não substitui cobertura de plugins carregados por nome nem verificações em máquina limpa. Aprovar importações de um diretório Windows implantado não é aceitação do instalador.

O fluxo Qt sob demanda para três plataformas mantém dois arquivos cdx.json na raiz e deployment-audit.json sem gerar automaticamente formulários manuais vazios. macOS lê apenas o .app compilado localmente e o SDK; verificações comuns não executam macdeployqt nem assinam, comprimem ou enviam pacotes de aplicativos. Linux fornece Qt/QML/plugins por um [diretório de runtime independente](linux-deployment.md), usando verificação deployed e contêiner sem SDK; o tar.gz baixado preserva bits executáveis. Atualmente só a base Ubuntu 24.04, x86_64, X11 é declarada, não um instalador Linux universal.

Para artefatos Windows confiáveis do projeto, extraia o instalador separadamente e compare seu diretório de aplicativo com o ZIP portátil do mesmo candidato usando `python scripts/check-windows-package-payload.py --installer-app <extracted-app-directory> --portable <portable-package.zip> --output build/review/payload-comparison.json`. A verificação compara hashes por arquivo, manifesto de instalação e DLLs gráficas do SDK que não devem ser incluídas. O caminho de saída deve ser novo. Essa comparação estática não executa o instalador ou uma atualização e não substitui a verificação real de instalação.

Exemplo local no macOS após recompilar a partir de fontes consistentes:

```sh
python3 scripts/collect-notices.py build/review/wp12/notices
python3 scripts/deployment_audit.py \
  --root build/ui/apps/desktop/xdzip.app \
  --app build/ui/apps/desktop/xdzip.app/Contents/MacOS/xdzip \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --mode sdk \
  --output build/review/wp12/deployment-audit.json
python3 scripts/sbom.py \
  --audit build/review/wp12/deployment-audit.json \
  --source build/review/wp12/notices/source-build.cdx.json \
  --output build/review/wp12/artifact-files.cdx.json
```

Depois de instalar jsonschema==4.23.0 em um ambiente isolado de verificação, use o Python desse ambiente para executar `scripts/validate-sbom.py` com os dois cdx.json acima. Consulte a [documentação jsonschema 4.23](https://python-jsonschema.readthedocs.io/en/v4.23.0/referencing/) sobre uso do registro. Ferramentas de verificação não são implantadas no produto.

<a name="xdzip-attribution-results-and-verification-limits"></a>

## Resultados de atribuição e limites de verificação

A [atribuição dos componentes reais](runtime-attribution.md) está integrada: o SPDX do SDK associa arquivos reais, módulos Qt e terceiros incorporados; textos oficiais de licença com hash fixado são entregues; diferenças de hash SDK/original e atribuições não confirmadas são registradas. Candidatos Rust comuns de execução são separados de ferramentas de compilação, desenvolvimento e proc-macro. Novas relações de componentes vinculam a auditoria real e são incorporadas ao SBOM do artefato, que continua explicitamente incompleto.

Na árvore instalada, associe cada DLL, framework e plugin entregue a seu componente original, versão, licença e código correspondente. Revise separadamente componentes incorporados ao Qt, atribuição de vinculação estática Rust e procedência da cadeia de ferramentas. Use o [guia de revisão de lançamento](../../../developers/qa/release-review.md) para decisões sobre candidatos.
