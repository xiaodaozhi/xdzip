<a name="xdzip-qt-sdk-and-source-correspondence-verification"></a>

# Verificação de correspondência entre o SDK Qt e o código-fonte

[English](../../engineering/qt-sdk-correspondence.md) | [简体中文](../../zh-CN/engineering/qt-sdk-correspondence.md) | [繁體中文（台灣）](../../zh-TW/engineering/qt-sdk-correspondence.md) | [日本語](../../ja-JP/engineering/qt-sdk-correspondence.md) | [한국어](../../ko-KR/engineering/qt-sdk-correspondence.md) | [Français](../../fr-FR/engineering/qt-sdk-correspondence.md) | [Deutsch](../../de-DE/engineering/qt-sdk-correspondence.md) | [Español](../../es-ES/engineering/qt-sdk-correspondence.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/qt-sdk-correspondence.md)

A verificação de correspondência usa os bytes reais dos arquivos oficiais; números de versão iguais não comprovam equivalência da compilação do SDK. `scripts/qt_sdk_correspondence.py` associa os arquivos-fonte fixados, os SPDX de código-fonte fornecidos pelo SDK, as declarações de compilação, a configuração instalada e os patches conhecidos aplicados por quem usa o SDK, gravando a saída em build/. Ele não modifica o SDK nem aprova opções de licença.

```sh
python3 scripts/qt_sdk_correspondence.py \
  --qt-prefix build/toolchain/Qt/6.8.2/macos \
  --output build/review/qt-sdk-correspondence.json
```

Por padrão, lê build/source-cache offline e falha se estiver ausente. Apenas --fetch explicitamente informado baixa e verifica arquivos oficiais do [manifesto fixado](../../../scripts/corresponding-sources.json). Os fluxos Qt sob demanda para as três plataformas executam --fetch após instalar o SDK fixado e guardam o JSON com os demais registros de verificação. No Mac continuam sendo enviados apenas registros, sem implantar, assinar ou empacotar aplicativos. Quando o coletor de entrega de fontes recebe --qt-prefix, inclui esse registro, patches e o manifesto fixado nos hashes da entrega; consulte [entrega de fontes](source-delivery.md).

<a name="xdzip-verification-layers"></a>

## Camadas de verificação

1. O SHA-256 do arquivo deve corresponder ao valor fixado; a raiz do arquivo, os caminhos dos membros e o marcador `.tag` fixado também devem corresponder. Registre separadamente o tipo de objeto Git do marcador. Se o marcador for uma árvore, recalcule primeiro o hash do objeto commit original fixado e verifique seu campo tree. Leia os arquivos individualmente sem extrair o código-fonte original.
2. Se o SDK fornecer o .source.spdx de um módulo, compare cada SHA-1 declarado com o conteúdo do arquivo. Registre a quantidade correspondente, ambos os hashes das diferenças, os arquivos declarados pelo SDK ausentes do arquivo e a quantidade de arquivos não cobertos. SHA-1 verifica apenas a declaração original do projeto; o arquivo completo é vinculado separadamente pelo SHA-256 fixado. Declarações ausentes, diferenças e arquivos não cobertos não são registrados como aprovados.
3. Guarde os hashes SPDX de compilação dos módulos do SDK, as declarações do compilador e os argumentos de configure. Leia também os modules/*.json, qconfig*.pri e cabeçalhos de configuração QT_FEATURE instalados, preservando conteúdo e hashes. Substitua na saída os diretórios absolutos do ambiente de compilação original por marcadores; os hashes dos arquivos vinculam a evidência original. A configuração não é todo o cache CMake, o ambiente de compilação ou os materiais da cadeia de ferramentas. Campos de arquitetura padrão do SDK não representam todas as arquiteturas das bibliotecas reais e não podem classificar incorretamente bibliotecas universais como x86_64.
4. O [patch AGL](../../../scripts/qt-patches/macos-no-agl.patch) altera apenas lib/cmake/Qt6/FindWrapOpenGL.cmake do SDK para evitar a vinculação ao AGL antigo quando este projeto usa Qt. A ferramenta lê o original do arquivo QtBase fixado, aplica o diff unificado, verifica os SHA-256 do patch, original e resultado e depois confere o arquivo real do SDK. Aceita apenas a versão original ou a versão com patch conhecida. Para CRLF do SDK Windows, compara somente após normalizar finais de linha, preservando SHA-256 dos bytes reais, SHA-256 LF e marcador de conversão. Outras alterações desconhecidas falham. Isso não altera o código de máquina do Qt em execução nem comprova que o Qt foi recompilado. O histórico completo de modificações do SDK ainda precisa ser verificado.

Os registros de correspondência usam o esquema 2 e preservam separadamente archive_commit fixado, archive_marker real do arquivo, archive_marker_type e commit_tree_verified. O `.tag` do Qt Declarative 6.8.2 é a árvore Git `0a8139c574fe150aba92d5f2484a8f3fdb774977`; a tag de lançamento corresponde ao commit `75534f3e7fff24ed7ccb364e2ed9950a73da879f`. Um objeto commit original em base64 fixado no repositório permite recalcular offline o hash do commit e validar seu campo tree. Os demais marcadores de módulos atualmente fixados são commits. A relação da árvore estabelece apenas a relação entre esses dois objetos Git, não que o arquivo contenha toda a árvore ou que o SDK tenha compilação equivalente.

Os source_commits de qt-license-sources.json identificam apenas commits fixados para obtenção de textos de licença e são armazenados separadamente dos marcadores de arquivo e commits de lançamento. Não podem ser apresentados como commits de compilação do SDK. Materiais históricos do esquema 1 colocavam o marcador de árvore do Qt Declarative em archive_commit; os artefatos publicados permanecem inalterados. Explicações posteriores devem corrigir o tipo de objeto e continuar verificando os textos de licença com o SHA-256 original.

<a name="xdzip-interpreting-results-and-release-boundaries"></a>

## Interpretação dos resultados e limites de lançamento

O derivado de segurança QtNetwork registra `network_security` separadamente, com comprovantes que vinculam patches, ferramenta de compilação, digests do QtCore original e do QtNetwork real, recursos e verificação real do transporte. Ao ler os comprovantes, a ferramenta verifica novamente os bytes reais do SDK. Essa evidência vale apenas para a biblioteca modificada; não reescreve o SPDX original como prova do derivado nem altera os fatos não cobertos dos outros módulos. Consulte as [notas de patches Qt](../../../patches/qt/README.md) para entradas e manutenção.

A existência de SPDX de fontes por módulo, as contagens de correspondências, ausências e itens não cobertos, a configuração e o estado dos patches seguem o JSON gerado na execução atual. Não generalize observações de um SDK para outro. Sem declarações por arquivo, não conclua que a correspondência foi aprovada; preserve as lacunas específicas.

O status do relatório é sempre evidence-collected-not-build-equivalence. A [compilação real das fontes Qt e seu carregamento ou substituição](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement) segue fluxo separado, mas não comprova a reprodução byte a byte do SDK original. Até que os materiais completos da cadeia de ferramentas e do SDK do sistema, os arquivos não cobertos ou gerados, a procedência em execução e o cumprimento de licenças estejam totalmente esclarecidos, não afirme que todas as obrigações de código-fonte correspondente foram cumpridas nem que as compilações são reproduzíveis byte a byte. Permanecem o status incompleto da [atribuição em execução](runtime-attribution.md), as escolhas de licença e as revisões de execução por plataforma.
