<a name="xdzip-build-deployment-and-release-architecture"></a>

# Arquitetura de compilação, implantação e publicação

[English](../../design/build-release.md) | [简体中文](../../zh-CN/design/build-release.md) | [繁體中文](../../zh-TW/design/build-release.md) | [日本語](../../ja-JP/design/build-release.md) | [한국어](../../ko-KR/design/build-release.md) | [Français](../../fr-FR/design/build-release.md) | [Deutsch](../../de-DE/design/build-release.md) | [Español](../../es-ES/design/build-release.md) | Português (Brasil) | [Русский](../../ru-RU/design/build-release.md)

<a name="xdzip-build-and-package-inputs"></a>

## Entradas de compilação e empacotamento

O CMake compila o aplicativo de desktop Qt, os adaptadores C++ e os auxiliares nativos. O Cargo compila o ambiente de execução Rust e gera a ponte CXX vinculada ao aplicativo. A CLI autônoma tem seu próprio ponto de entrada de compilação e funciona sem Qt. `scripts/desktop.py` e `scripts/cli.py` coordenam a compilação e os testes. O código fixa o Rust 1.99.0 e o SDK Qt 6.8.2. Ao comparar resultados, mantenha juntos a identidade da compilação, a arquitetura de destino, os arquivos de bloqueio e a origem das ferramentas. [Cadeia de ferramentas](../engineering/toolchain.md), [Guia de compilação](../../engineering/developer-build.md)

O pacote de desktop implanta as bibliotecas Qt, os módulos QML, os recursos da plataforma e os auxiliares incluídos que ele exige. O processo interno de imagens usa Qt para decodificar imagens e exibir SVGs estáticos. A distribuição da CLI inclui o processo auxiliar de arquivos compactados. Os scripts e os contratos de plataforma mantêm os perfis de compilação, as listas de arquivos permitidos nos pacotes e as verificações de implantação. Examine o pacote completo para saber quais dependências são instaladas e qual é seu tamanho total. [Auditoria de implantação e dependências](../engineering/deployment-audit.md), [Distribuição da CLI](../engineering/command-line.md)

<a name="xdzip-current-distribution-channels"></a>

## Canais de distribuição atuais

| Destino | Pacote e limite de confiança |
| --- | --- |
| macOS Apple Silicon | O GitHub fornece um DMG e um ZIP com o pacote do aplicativo. A versão mínima é o macOS 14. O pacote usa assinatura ad hoc de integridade. Consulte as instruções da plataforma para instalar e atualizar. |
| Windows 11 x86_64 | O GitHub fornece um instalador por usuário sem assinatura e um ZIP portátil. A Microsoft Store gerencia sua própria identidade MSIX, a assinatura e o canal de atualização. |
| Ubuntu 24.04 LTS x86_64 | O GitHub fornece um arquivo compactado do diretório de execução e um DEB. A configuração mínima inclui X11 e glibc 2.39 ou posterior. |
| CLI autônoma | Publicada com identidades próprias de pacote e atualização. A CLI incluída na instalação de desktop segue as regras de integração e transferência do próprio pacote. |

A [política de suporte](../engineering/support-policy.md) define os sistemas operacionais e as arquiteturas atendidos. Consulte o [README do projeto](../../../README.pt-BR.md) e as notas da versão correspondente para ver os arquivos publicados. O [guia de instalação](../../engineering/installers-associations.md) descreve os limites de associação e integração.

<a name="xdzip-routine-checks-native-builds-and-release"></a>

## Verificações rotineiras, compilações nativas e publicação

O [fluxo de CI rotineiro](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml) executa verificações leves. As tarefas nativas Foundation e Qt usam fluxos manuais separados. A CI comum no Apple Silicon compila, testa e verifica a inicialização, sem disponibilizar um aplicativo para download. O fluxo dedicado de publicação tem etapas autorizadas próprias de implantação, assinatura, empacotamento e publicação. O [protocolo de publicação](../engineering/release-protocol.md) e a [política de CI](../../../developers/engineering/ci-policy.md) definem as condições e evidências exigidas.

Os manifestos de publicação, as somas de verificação, os inventários, a SBOM e os documentos de licença descrevem os artefatos entregues. O fluxo de publicação não gera nem envia ZIPs do código-fonte do projeto. Os instantâneos de tags do GitHub seguem um mecanismo próprio. As obrigações dependem dos componentes entregues, do [registro de licenças](../engineering/licensing.md) e da [política de entrega de código-fonte](../../engineering/source-delivery.md).

<a name="xdzip-update-and-future-distribution"></a>

## Atualizações e distribuição futura

A seleção e a verificação de atualizações são independentes das tarefas com arquivos compactados. As instalações qualificadas seguem o contrato de atualização para metadados de confiança, arquivos verificados, identidade da instalação e encaminhamento quando há tarefas ativas. Windows e Linux usam o auxiliar aplicável, o macOS usa o Sparkle e a Microsoft Store mantém sua edição. Configurações portáteis e de desenvolvimento têm limites próprios. [Atualizações de software](../engineering/software-update.md)

Expandir para outras arquiteturas, versões mínimas de sistemas, formatos de pacote ou lojas exige evidências próprias de compilação e aceitação. Os [registros de decisões de arquitetura](architecture-decisions.md) mantêm as decisões anteriores de distribuição e as condições para revê-las.
