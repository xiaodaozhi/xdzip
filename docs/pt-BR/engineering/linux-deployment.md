<a name="xdzip-linux-standalone-runtime-directory-and-verification"></a>

# Diretório independente de execução Linux e verificação

[English](../../engineering/linux-deployment.md) | [简体中文](../../zh-CN/engineering/linux-deployment.md) | [繁體中文](../../zh-TW/engineering/linux-deployment.md) | [日本語](../../ja-JP/engineering/linux-deployment.md) | [한국어](../../ko-KR/engineering/linux-deployment.md) | [Français](../../fr-FR/engineering/linux-deployment.md) | [Deutsch](../../de-DE/engineering/linux-deployment.md) | [Español](../../es-ES/engineering/linux-deployment.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/linux-deployment.md)

A base atual de implantação para desenvolvimento é Ubuntu 24.04, x86_64 e X11. O objetivo é executar o produto atual sem o SDK Qt de desenvolvimento; isso não promete outras distribuições, glibc mais antiga, Wayland ou suporte completo a instalador. A CI do macOS mantém compilação, testes e verificação de inicialização nativos. DMGs locais de desenvolvimento seguem o [guia de implantação Mac](macos-deployment.md) e não são afetados por este processo.

<a name="xdzip-layout-and-generation"></a>

## Estrutura e geração

`scripts/deploy_linux.py` aceita apenas arquivos ELF confiáveis pertencentes ao projeto e um SDK Qt 6.8.2 explicitamente informado, criando um novo diretório build/ ou dist/:

- `xdzip` é o inicializador, usando bibliotecas, plugins e QML locais sem herdar caminhos de busca do SDK de desenvolvimento.
- `bin/xdzip` é o produto; o `qt.conf` adjacente usa caminhos Qt relativos.
- `lib/` contém bibliotecas compartilhadas Qt/SDK realmente importadas; `qml/` mantém o estilo Basic Controls e os submódulos de implementação necessários. Outros estilos de Controls, NativeStyle, Effects, variantes de diálogo específicas de estilo, arquivos do designer e metadados de tipos QML são removidos antes da coleta de dependências nativas, para que suas bibliotecas exclusivas não sejam implantadas.
- `plugins/` contém xcb, verificação fora da tela, imagens, ícones, entrada de plataforma, integração gráfica xcb e o necessário `tls/libqopensslbackend.so`; a implantação falha sem o backend OpenSSL. Plugins de plataforma EGLFS, framebuffer e Wayland não são descritos como entrega verificada.
- `translations/` mantém traduções de controles padrão Qt; `share/notices/` entrega avisos atuais e materiais de dependências de fonte.
- `share/deployment.json` registra QML, dependências de execução do sistema, bibliotecas SDK copiadas e limites; README.md fornece instruções de uso.

O scanner usa [qmlimportscanner](https://doc.qt.io/qt-6.8/qtqml-tooling-qmlimportscanner.html) do Qt fixado. Xdzip.Settings e Xdzip.Native são módulos integrados registrados; bibliotecas estáticas de desenvolvimento não são copiadas. Dependências comuns ausentes, caminhos QML fora do SDK, conflitos de bibliotecas de mesmo nome e bibliotecas externas desconhecidas causam falha. A coleta recursiva de importações nativas copia apenas bibliotecas do SDK, não glibc ou drivers gráficos do hospedeiro; dependências do sistema são listadas pelo pacote Debian proprietário real.

A poda valida a árvore QML copiada, plugins Basic necessários, importações fortes mantidas e layouts de estilo conhecidos antes da exclusão. Links inesperados, estilos ou seletores de diálogo desconhecidos e dependências fortes de módulos omitidos causam falha. Diálogos padrão, métodos de entrada, acessibilidade, plugins de imagem, TLS e traduções do sistema mantêm suas regras. `share/deployment.json` registra arquivos QML removidos e contagens de bytes. O `cli/` privado compartilha `bin/xdzip-native-codec` por seu caminho de contexto fixo; pacotes CLI independentes continuam autossuficientes.

HTTPS também exige `libssl3t64` e `ca-certificates` do Ubuntu. Verificar a propriedade dos pacotes de sistema de `libssl.so.3` e `libcrypto.so.3` e um armazenamento de confiança CA do sistema não vazio. O backend OpenSSL do Qt pode carregar essas bibliotecas em execução, portanto ldd sozinho não comprova dependências completas. O manifesto de implantação registra versões dos pacotes binários instalados, pacotes fonte e suas versões. Instaladores declaram ambas as dependências; o SBOM de arquivos reais as registra como pacotes de sistema necessários sem incluir bibliotecas do sistema no aplicativo.

patchelf define RPATH `$ORIGIN` relativo em cada ELF, calculando a distância relativa de cada plugin até lib/. O redirecionamento de caminhos Qt segue as [regras oficiais de qt.conf](https://doc.qt.io/qt-6.8/qt-conf.html), permitindo mover o diretório inteiro. As bibliotecas continuam ligadas dinamicamente e substituíveis; não há restrição à substituição compatível do Qt. Consulte [implantação Qt no Linux](https://doc.qt.io/qt-6.8/linux-deployment.html) para limites gerais.

Após reescrever o RPATH relativo, a implantação remove tabelas de símbolos nativas desnecessárias e verifica que cada entrada DT_NEEDED e RPATH/RUNPATH permaneceu inalterada. Registra versão da ferramenta, hashes e contagens de bytes antes e depois para cada ELF em `symbol_stripping`; entradas do SDK e binários de compilação originais não são alterados. Para uma cópia de diagnóstico, o script de implantação subjacente aceita `--keep-symbols`. A atribuição final do SDK e hashes do aplicativo descrevem os arquivos transformados. Isso reduz o tamanho do pacote, não comprova melhoria medida de memória ou vazão.

Use por etapas o [ponto de entrada unificado de compilação](../engineering/developer-build.md) para geração e verificação de rotina. Conclua primeiro compilação, testes e preparação de empacotamento na mesma configuração e execute:

```sh
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

O diretório de compilação padrão é build/desktop, a saída build/package e o SDK Qt `build/toolchain/Qt/6.8.2/gcc_64`; passe as opções compartilhadas em cada etapa para outros locais. `package --kind runtime` gera o diretório de execução e os materiais de auditoria; depois `package --kind installer` com o mesmo output-dir gera tar.gz e .deb. O padrão `all` executa ambos em ordem. Os scripts subjacentes [deploy_linux.py](../../../scripts/deploy_linux.py), [package_linux.py](../../../scripts/package_linux.py) e [verify_linux_runtime.py](../../../scripts/verify_linux_runtime.py) mantêm a lógica de plataforma; consulte seus `--help` para resolver problemas.

A implantação executa somente em Linux x86_64 nativo e não sobrescreve diretórios existentes. A verificação prévia do empacotamento confere qmlimportscanner, ldd, patchelf, dpkg-query e dpkg-deb necessário ao instalador; essas ferramentas de compilação não entram no produto. A verificação de execução em contêiner também exige Docker e um daemon acessível; compilações são limitadas à imagem Ubuntu fixada e aos pacotes de sistema registrados. A ausência do Docker não bloqueia compilação e testes comuns, mas verify falha explicitamente; uma verificação de isolamento não executada não é registrada como aprovada. verify comum não instala .deb no hospedeiro nem altera associações de arquivos; o ciclo de instalação continua sendo verificado em CI descartável.

<a name="xdzip-experimental-strict-snap"></a>

## Snap estrito experimental

O Snap `core24` com confinamento estrito é uma distribuição experimental separada. Um aplicativo confinado não herda a configuração XKB do host em `$SNAP/usr/share/X11/xkb`. Quando o inicializador define `QT_XKB_CONFIG_ROOT` para esse caminho, o Snap deve incluir toda a árvore XKB da entrada de compilação Ubuntu fixada. Sem ela, o plugin Qt Wayland pode carregar e se conectar ao compositor, mas falha ao criar o contexto de teclado por um erro no caminho de inclusão de `xkbcommon`. Mantenha o aviso de copyright de `xkb-data` no pacote e registre no recibo do candidato a versão do pacote, o hash determinístico do diretório, o número de arquivos e o hash do aviso. Verifique o Snap final após a extração e execute separadamente os testes de inicialização fora da tela e em Wayland. Essas verificações não substituem a instalação e a interação em uma sessão real de desktop com Snap estrito.

<a name="xdzip-automated-evidence-without-the-sdk"></a>

## Evidências automatizadas sem o SDK

O contêiner usa Ubuntu 24.04 fixado por digest e instala dependências reais de execução do sistema e suporte de teste Xvfb, fontes e localidades, mas não Qt, Rust ou compilador, verificando novamente sua ausência. Não monta o SDK de desenvolvimento nem toda a árvore de fontes, apenas o diretório do produto somente leitura, um ZIP separado de 100.000 entradas pertencente ao projeto e os scripts de verificação. A verificação de execução de arquivos compactados desativa rede e escalada de privilégios; somente o diretório temporário de configuração permite gravação.

Monte o aplicativo em um novo caminho contendo espaços. Após limpar variáveis de busca de desenvolvimento, use o inicializador para verificar:

1. Execução real de `--version`.
2. Inicialização e fechamento seguros de uma janela QML real sob X11.
3. A sonda nativa do produto conclui primeiro quadro, leitura de 100.000 entradas, cancelamento de solicitação ativa e saída; contagens e resultados de cancelamento atendem ao contrato.
4. Após limpar variáveis de desenvolvimento do hospedeiro, verificar todos os ELFs empacotados em modo implantado sem recurso ao SDK. O SBOM de arquivos reais mantém a mesma identidade de versão e commit e passa pelo esquema oficial.
5. Habilitar rede em um novo processo da mesma imagem sem SDK, executar `--update-probe` do produto com `XDZIP_SETTINGS_PATH` isolado e realmente solicitar GitHub por HTTPS. Deve informar backend OpenSSL disponível e concluir a verificação de atualização; preservar `.https.json` e `.https.log` separadamente. Falhas de rede, certificado ou TLS bloqueiam a verificação.

A sonda HTTPS registra separadamente backend OpenSSL, arquivo CA do sistema e resultados reais da solicitação ao GitHub. Uma solicitação real bem-sucedida, sozinha, não comprova verificação de assinatura nem instalação nativa de atualização. A análise de assinatura e os fluxos de atualização sem SDK são verificados pelo [fluxo separado de regressão de atualização](../../../.github/workflows/update-regression.yml); essas evidências não substituem umas às outras.

Preservar logs brutos e relatórios estruturados de cada tentativa como `.https.attempt-N.log` e `.https.attempt-N.json`, além de `.https.log` para a última tentativa. Relatórios ausentes ou inválidos, tempo esgotado e saídas do processo mantêm status de falha. Tentar novamente somente quando o controlador do produto confirmar limitação de taxa da lista anônima de lançamentos do GitHub e status, fase da solicitação, cabeçalhos de resposta e diagnósticos estiverem completos. Aguardar o prazo do servidor ou usar espera progressiva limitada, com no máximo três tentativas e 3.700 segundos de espera total, como a sonda de aceitação do macOS. Prazo superior ao orçamento restante, espera interrompida ou erro TLS, de rede ou outro falha imediatamente. A sonda deve concluir uma solicitação real de atualização para passar. Registrar cada decisão e duração real de espera em `.https-waits.json`.

Antes de reutilizar um caminho de saída, mover relatório e logs anteriores para um diretório `.history` com o nome do relatório no mesmo diretório pai, com subdiretório separado por execução. Se uma nova execução falhar no meio, o relatório de sucesso anterior não permanece no caminho de saída atual, evitando confusão; artefatos históricos continuam verificáveis separadamente.

linux-runtime.json registra eventos reais e SHA-256 do executável. A renderização por software Xvfb/Qt Quick comprova apenas o fluxo de execução, não GPU física, janelas de candidatos em chinês, leitores de tela reais ou instalação completa no desktop.

<a name="xdzip-download-delivery-and-remaining-gates"></a>

## Entrega para download e condições restantes

A CI gera xdzip-linux-x86_64.tar.gz e registra SHA-256 e hashes do executável. tar.gz preserva permissões de execução, evitando sua perda na extração ZIP do Actions. Caminhos do arquivo, UID/GID e carimbos de data são normalizados sem diretórios privados. Após baixar, extrair e executar `./xdzip` dentro do diretório; não mover apenas bin/xdzip. O diretório linux/ bruto não é enviado novamente.

O diretório de execução `build/package`, tar.gz e .deb gerados aqui são entregas experimentais de desenvolvimento e não instalam nem alteram associações padrão automaticamente. Os tar.gz e .deb oficiais do Ubuntu para Release são gerados pelo fluxo Release dedicado e fornecidos conforme o [protocolo de publicação](release-protocol.md). O escopo de plataformas segue a [política de suporte](support-policy.md); outras distribuições e desktops, Wayland e ampliações relacionadas exigem issues separadas. A aceitação existente do desktop permanece em [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5). Para alterações posteriores, verificar novamente os itens afetados com IME e leitor de tela reais e instalação, atualização e desinstalação limpas; reconstruir um diretório por si só não exige repetir a aceitação existente. A revisão histórica de componentes de `v0.1.0` está em [#4](https://github.com/xiaodaozhi/xdzip/issues/4) e [#53](https://github.com/xiaodaozhi/xdzip/issues/53); executar o diretório com sucesso não completa automaticamente as condições de futuras publicações.

Consulte [instalação e associações](installers-associations.md) para instalação, atualização, reversão e desinstalação de Ubuntu .deb, bloqueios de execução e automação MIME do desktop. Passar no contêiner de execução não substitui verificações reais do desktop nem revisão de licenças de publicação.
