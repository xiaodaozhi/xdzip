<a name="xdzip-actual-component-attribution-and-license-materials"></a>

# Atribuição dos componentes reais e materiais de licença

[English](../../engineering/runtime-attribution.md) | [简体中文](../../zh-CN/engineering/runtime-attribution.md) | [繁體中文（台灣）](../../zh-TW/engineering/runtime-attribution.md) | [日本語](../../ja-JP/engineering/runtime-attribution.md) | [한국어](../../ko-KR/engineering/runtime-attribution.md) | [Français](../../fr-FR/engineering/runtime-attribution.md) | [Deutsch](../../de-DE/engineering/runtime-attribution.md) | [Español](../../es-ES/engineering/runtime-attribution.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/runtime-attribution.md)

Este processo complementa os SBOMs por arquivo ao associar bibliotecas e plugins Qt reais às declarações SPDX do SDK. Sua saída é evidência revisável, não seleção automática de licença nem aprovação de conformidade. A documentação necessária para uma publicação oficial continua incompleta.

<a name="xdzip-inputs-and-sources"></a>

## Entradas e fontes

Desde o Qt 6.8, são fornecidos [inventários SPDX](https://doc.qt.io/qt-6.8/licenses-used-in-qt.html) correspondentes à compilação. As ferramentas leem somente declarações de execução como qtbase, qtdeclarative, qtsvg e qttranslations em sbom/ do SDK Qt 6.8.2 fixado, excluindo ferramentas de compilação e teste qttools. Validam-se a versão SPDX, o nome do módulo e a versão do Qt; preservam-se o SHA-256, o namespace e as referências estáveis a pacotes do documento original. A saída mantém apenas campos necessários, sem copiar comentários de compilação upstream que contenham caminhos de computadores de desenvolvimento.

`scripts/qt-license-sources.json` fixa commits oficiais das fontes Qt e do SPDX license-list-data, URLs dos textos de licença e SHA-256. Downloads ficam em cache sob build/; qualquer divergência de hash no cache ou download causa falha. URLs SPDX arbitrárias não são seguidas para baixar materiais. Além dos textos padrão em LICENSES/, incluem-se avisos originais dos algoritmos de redimensionamento suave do Qt GUI e de SVG, bem como textos LGPL-2.1 referenciados por declarações de dependências do sistema. Textos LicenseRef e direitos autorais upstream permanecem no JSON das declarações.

Os avisos de execução do Qt também incluem a fonte editável fixada da Public Suffix List usada pelo Qt Network, o texto MPL-2.0 e instruções de obtenção. O coletor verifica o blob upstream, SHA-256 e bytes da licença. A entrega de fontes verifica separadamente se essa entrada reproduz duas formas de dados gerados no arquivo Qt; consulte os [materiais de fonte correspondentes](source-delivery.md). Isso não substitui as verificações de fonte e substituição de outros módulos Qt.

Os avisos MIME do Tika no Qt Core preservam as atribuições relevantes da Apache Software Foundation e do PRONOM dos Arquivos Nacionais britânicos, os textos Apache-2.0 e OGL-UK-3.0 e o NOTICE upstream completo fixado. O NOTICE completo também descreve outros componentes upstream do Tika; isso não implica que sejam distribuídos com este produto. Consulte os [materiais de fonte correspondentes](source-delivery.md) para os dados MIME originais e a verificação da conversão Qt.

<a name="xdzip-file-and-component-association"></a>

## Associação de arquivos e componentes

1. Recalcular os hashes dos arquivos dentro da raiz de verificação e rejeitar arquivos modificados após deployment-audit.json.
2. O modo SDK do macOS usa caminhos relativos do SDK Qt realmente resolvidos e identidade de bytes, sem afirmar implantação independente.
3. A implantação Linux registra o caminho e SHA-256 originais do SDK e o caminho e SHA-256 do artefato após patchelf. Verificações posteriores conferem registros nos dois sentidos e rejeitam provas de cópia antigas ou alteradas.
4. O Windows prefere identidade de bytes do arquivo do SDK. Candidatos com nome único e bytes diferentes são marcados apenas como filename-only-unverified-rewrite e registrados como lacunas, não como fontes verificadas.
5. Usar SPDX CONTAINS para encontrar os proprietários dos arquivos e resolver DEPENDS_ON recursivamente, incluindo referências entre repositórios Qt e terceiros incorporados. Referências de pacotes ausentes continuam como lacunas. Não incluir incorretamente todas as ferramentas na execução por um CONTAINS de todo o repositório.
6. QML e traduções não nativos sem declarações por arquivo são associados somente ao escopo do repositório de origem correspondente; suas licenças individuais não são inferidas. Bibliotecas nativas do SDK sem registros de arquivo SPDX, arquivos nativos fora do escopo Qt, licenças NOASSERTION e textos ausentes são registrados separadamente.

Os bytes reais do SDK podem diferir do SHA-1 do SPDX de compilação upstream, por exemplo por processamento posterior de instalação ou assinatura. As ferramentas preservam valores esperados, comparações e lacunas sem inferir causas. Associação dos bytes do SDK, registros de reescrita da implantação e hashes originais upstream são três tipos distintos de evidência.

O empacotamento dedicado do macOS gera associações implantadas enquanto o aplicativo final assinado existe, relacionando entradas do SDK por bytes exatos dos recursos ou por seções arm64, UUID e versões de importação, e adiciona o relatório ao SBOM dos arquivos reais. A comparação exclui assinaturas, layout dos comandos de carregamento e LINKEDIT. Consulte [origem e avisos das bibliotecas nativas de execução](native-runtime-materials.md) para o escopo e os avisos específicos. O modo SDK comum não estabelece, por isso, conclusões sobre a implantação.

Para plugins movidos de diretórios QML para `Contents/PlugIns/quick/`, a associação da implantação pode usar links simbólicos QML auditados no pacote para localizar uma única entrada do SDK. Verificar novamente os links reais, destinos auditados e hashes dos arquivos ao gerar e consumir os registros e executar a mesma comparação de bytes ou Mach-O. Origem ausente permanece não verificada; várias fontes diferentes ou links alterados são rejeitados. Correspondência apenas pelo nome não reforça a atribuição nem elimina diferenças de hash do SDK upstream ou lacunas de revisão de licenças.

O Rust preserva o grafo completo de compilação do hospedeiro e gera candidatos de execução seguindo dependências comuns a partir de `xdzip-qt-bridge` do desktop, `xdzip-codec-bridge` do processo de formatos e do `xdzip-update-helper` real no Windows/Linux; o macOS não tem uma raiz separada de assistente de atualização. Ferramentas de compilação, desenvolvimento e proc-macro e suas dependências são excluídas. São possíveis entradas de execução, não prova de que o código de máquina de cada pacote permaneceu após a ligação; eliminação pelo linker, C/C++ nativo e bibliotecas de execução do compilador precisam de revisão adicional.

Alternativas simples antigas de licença Cargo separadas por barras tornam-se SPDX `OR` no CycloneDX de fontes, preservando o original upstream em `xdzip:cargo_license_raw`. Expressões com barras e precedência ambígua são rejeitadas. Inventários e avisos originais de compilação não são reescritos; `AND`, `OR` e `WITH` padrão permanecem inalterados. Isso normaliza o formato, não escolhe uma alternativa de licença nem aprova a publicação.

<a name="xdzip-execution-and-delivery"></a>

## Execução e entrega

A variante derivada conhecida do QtNetwork com patches adiciona `network_security_variant` aos registros de arquivo; os relatórios também preservam comprovantes completos de compilação verificados novamente. Diferenças de hash do SDK upstream continuam visíveis; a revisão de publicação usa fontes fixadas, patches completos, comprovantes e evidências de transformação da implantação para explicá-las. A preparação para publicação exige que a atribuição final do Network esteja ligada a esses bytes; nome de arquivo correspondente não basta. Os SBOMs de fontes e artefatos identificam a variante separadamente sem converter o estado global incompleto em aprovação de licença.

```sh
python3 scripts/prepare-runtime-notices.py --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --output build/review/notices/Qt-runtime
python3 scripts/runtime_attribution.py --audit build/review/deployment-audit.json \
  --root build/review/linux --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --deployment build/review/linux-deployment.json --output build/review/runtime-attribution.json
python3 scripts/sbom.py --audit build/review/deployment-audit.json \
  --source build/review/source-build.cdx.json --attribution build/review/runtime-attribution.json \
  --output build/review/artifact-files.cdx.json
```

O Linux fornece registros de reescrita da implantação; as demais plataformas omitem --deployment. Gravar relatórios fora da raiz de verificação para evitar recursão de autoinventário. Fluxos Qt sob demanda e Release preparam materiais de licença antes da implantação e do hash final. Windows e Linux colocam avisos nos ambientes reais de execução; o tar.gz do Linux também os entrega. No macOS, Qt sob demanda lê o SDK e registra somente logs e JSON. Um fluxo Release dedicado e explicitamente autorizado implanta, assina ad hoc, empacota e verifica separadamente os materiais reais segundo o [protocolo de publicação](release-protocol.md), preservando escopo implantado e hashes separados. Registros do SDK não substituem a atribuição do pacote final do aplicativo.

runtime-attribution.json inclui arquivos, componentes, relações, declarações upstream, direitos autorais, fontes dos textos e lacunas. artifact-files.cdx.json adiciona os mesmos componentes e relações conhecidas de propriedade, vinculando o relatório aos mesmos hashes da auditoria real e à identidade de compilação. As verificações do esquema oficial CycloneDX e das referências continuam; compositions permanece incompleto.

As verificações padrão permitem lacunas de revisão de desenvolvimento registradas explicitamente, mas nunca as marcam silenciosamente como aprovadas. Erros de identidade, caminho, bytes, versão, registro de cópia ou hash de texto de licença falham imediatamente. --require-complete bloqueia a publicação e atualmente falha se a atribuição ou os materiais de publicação estiverem incompletos. Os testes cobrem arquivos reescritos, registros de cópia antigos, escape de caminho, versões erradas, omissões entre documentos, SBOMs parciais e exclusão de ferramentas Rust.

<a name="xdzip-unresolved-release-gates"></a>

## Condições de publicação ainda não resolvidas

Um [coletor de entrega de fontes](source-delivery.md) preserva o código real do projeto, Rust vendor, quatro arquivos de módulos Qt de execução, dois de módulos de compilação e instruções de reconstrução e substituição dinâmica. Ele não comprova equivalência de compilação SDK/upstream por item. Cada publicação ainda deve revisar atribuição exata de ligação estática, bibliotecas do sistema e redistribuíveis, licenças individuais de todos os recursos incorporados, escolhas de alternativas de licenças de código aberto e cumprimento. A alteração ou substituição de metadados na [verificação local de implantação Mac](macos-deployment.md) e a [compilação, carregamento e substituição reais do Qt a partir das fontes](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement) são processos distintos com evidências separadas; nenhum comprova equivalência de compilação byte a byte para SDKs arbitrários. Inventários de fontes e campos SPDX não substituem obrigações de licença. A origem e as licenças do candidato histórico `v0.1.0` estão em [#4](https://github.com/xiaodaozhi/xdzip/issues/4) e [#53](https://github.com/xiaodaozhi/xdzip/issues/53); candidatos posteriores exigem revisão separada, sem identidade de assinatura paga como condição. A aceitação existente do desktop permanece em [#5](https://github.com/xiaodaozhi/xdzip/issues/5).

A [verificação de correspondência do SDK Qt](qt-sdk-correspondence.md) registra também commits de arquivos fixados, hashes por arquivo em declarações de fonte do SDK, configuração instalada e o patch conhecido do consumidor AGL. Correspondências no escopo declarado não comprovam equivalência completa de compilação do SDK; os bloqueios de publicação existentes permanecem.

O encerramento automatizado adiciona mapas reais do linker e membros de bibliotecas estáticas, hashes de recursos de execução, versões confiáveis de dependências nativas e análise precisamente reversível de diferenças binárias; consulte o [contrato completo](../../../developers/engineering/candidate-verification.md). Marcar uma transformação como explicada apenas quando os hashes dos bytes reconstruídos coincidirem exatamente com as declarações upstream. Não apagar diferenças originais do SDK nem adivinhar o histórico não documentado do publicador.
