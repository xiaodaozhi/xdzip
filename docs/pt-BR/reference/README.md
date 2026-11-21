<a name="xdzip-product-design-and-technical-reference"></a>

# Projeto do produto e referência técnica

[English](../../reference/README.md) | [简体中文](../../zh-CN/reference/README.md) | [繁體中文](../../zh-TW/reference/README.md) | [日本語](../../ja-JP/reference/README.md) | [한국어](../../ko-KR/reference/README.md) | [Français](../../fr-FR/reference/README.md) | [Deutsch](../../de-DE/reference/README.md) | [Español](../../es-ES/reference/README.md) | Português (Brasil) | [Русский](../../ru-RU/reference/README.md)

Comece pela [arquitetura do sistema](../design/system-architecture.md) para conhecer os caminhos de execução do desktop e da CLI. As seções a seguir apontam o contrato que define cada comportamento. Veja a [tabela de formatos](../../../README.pt-BR.md#xdzip-format-support) para consultar as operações aceitas. Para instruções de uso ou preparação do ambiente, consulte o [manual do usuário](../user-guide/README.md), os [exemplos de CLI](../integration/README.md) e o [guia de contribuição](../contributing/README.md).

<a name="xdzip-architecture-and-interfaces"></a>

## Arquitetura e interfaces

- [Visão geral do projeto](../design/index.md)
- [Requisitos e limites das operações](../design/product-requirements.md)
- [Modelo de domínio, identidade das entradas e versões da origem](../design/domain-data.md)
- [Ponte de comandos e eventos Qt/Rust](../design/qt-rust-bridge.md)
- [Agendamento, progresso, cancelamento e encerramento](../design/task-scheduling.md)
- [Orçamentos de desempenho e verificação](../design/performance.md)
- [Termos e limites de implementação](../design/glossary.md)
- [Rastreabilidade de requisitos e verificações](../design/requirements-traceability.md)
- [Decisões de arquitetura](../design/architecture-decisions.md)
- [Evolução dos protótipos para o desktop e a execução em Rust](../design/architecture-evolution.md)
- [Estrutura do repositório e dependências](../engineering/repository-layout.md)
- [Contratos de interfaces e verificações](../engineering/contracts.md)
- [Execução do produto e ciclo de vida do Qt](../engineering/product-runtime.md)
- [Sessões e encerramento do aplicativo](../engineering/application-sessions.md)
- [Justificativa para a ponte cxx](../engineering/cxx-admission.md)
- [Serviços de calendário e codificação do host](../engineering/host-platform-services.md)

<a name="xdzip-reading-and-writing-archives"></a>

## Leitura e gravação de arquivos compactados

- [Autorização, preparação e gravação segura](../design/file-security.md)
- [Navegação somente para leitura](../engineering/readonly-browser.md)
- [Navegação ZIP somente para leitura](../engineering/zip-readonly-browsing.md)
- [Índices grandes e navegação sob demanda](../engineering/large-index.md)
- [Leitura de volumes divididos e abertura temporária](../engineering/archive-opening.md)
- [Codificação adaptativa de nomes de arquivos](../engineering/filename-encoding.md)
- [Prévia do conteúdo](../engineering/content-preview.md)
- [Verificação prévia dos metadados para extração](../engineering/extraction-preflight.md)
- [Extração segura de ZIP](../engineering/zip-safe-extraction.md)
- [Verificação de integridade](../engineering/archive-checking.md)
- [Abertura de entradas, arrastar e soltar e alteração](../engineering/archive-transfers.md)
- [Senhas e abertura de diretórios criptografados](../engineering/password-extraction.md)
- [Contrato de criação ZIP](../engineering/zip-creation.md)
- [Criação ISO9660 e Joliet](../engineering/iso-creation.md)
- [Arquivos ZIP autoextraíveis integrados para Windows](../engineering/sfx-creation.md)

<a name="xdzip-format-support-and-implementation"></a>

## Suporte e implementação de formatos

- [Recursos integrados, perfis de criação e alcance das verificações](../engineering/format-registry.md)
- [Adaptadores de formato e projeto do experimento de SDK externo](../design/codec-extension-sdk.md)
- [Experimento de SDK externo somente para leitura e escopo de execução](../engineering/sdk-trial.md)
- [Contrato de processo isolado de formatos](../engineering/native-codecs.md)
- [Formatos e algoritmos próprios em Rust](../engineering/independent-rust-codecs.md)
- [Contabilização do trabalho auxiliar de decodificação](../engineering/auxiliary-decoding-work.md)
- [Admissão da dependência de metadados ZIP](../engineering/zip-metadata-admission.md)
- [Admissão de dependências de extração e saída ZIP](../engineering/zip-extraction-admission.md)
- [Fontes do formato 7z e implementação independente](../engineering/sevenzip-format-sources.md)
- [Bases da implementação independente RAR e limites de compatibilidade](../engineering/rar-format-sources.md)
- [Fontes do formato CAB e bases da implementação](../engineering/cab-format-sources.md)
- [Contrato de implementação TAR/GZIP](../engineering/tar-gzip.md)
- [Fluxos de um único arquivo e pacotes derivados de ZIP](../engineering/single-file-streams.md)
- [Contrato de fluxos LZ4](../engineering/lz4-streams.md)
- [Contrato de fluxos Zstandard](../engineering/zstandard-streams.md)
- [Contrato de Unix compress Z](../engineering/compress-streams.md)
- [Contrato de fluxos Brotli comuns](../engineering/brotli-streams.md)
- [Estrutura e algoritmos LHA](../engineering/lha-containers.md)
- [Contêineres PMA e métodos PMarc](../engineering/pma-streams.md)
- [Fluxos compactados e admissão de contêineres ARJ](../engineering/arj-streams.md)
- [Contêineres ARC e perfis de fluxos compactados](../engineering/arc-streams.md)
- [Contêineres BH e perfil BlakHole aceito](../engineering/bh-containers.md)
- [Fontes e requisitos de licença de ACE, ALZ e EGG](../engineering/ace-alz-egg-format-sources.md)
- [Invólucros AES Crypt somente para leitura](../engineering/aescrypt-envelopes.md)
- [Contrato de contêineres DEB](../engineering/deb-containers.md)
- [Contrato de contêineres ASAR](../engineering/asar-containers.md)
- [Invólucros de texto UU, UUE e XXE](../engineering/text-envelopes.md)
- [Fontes e licenças de WIM, MSI e instaladores](../engineering/wim-msi-installer-format-sources.md)
- [Fontes e licenças de imagens de disco e outros formatos](../engineering/disk-image-archive-format-sources.md)
- [Inventário de formatos do Bandizip e viabilidade no xdzip](../engineering/bandizip-format-assessment.md)

<a name="xdzip-desktop-interaction-and-visual-design"></a>

## Interação e design do desktop

- [Visualizações Qt Quick e responsabilidades dos controladores](../design/qt-quick-ui.md)
- [Especificações visuais do desktop](../visual/README.md)
- [Padrão de design da interface](../../visual/design-standard.md)
- [Fluxo de design e correção da interface](../visual/design-workflow.md)
- [Ícones de associações de arquivos](../visual/file-association-icons.md)
- [Revisão de fidelidade visual HTML](../visual/html-fidelity.md)
- [Especificação complementar de interação e textos](../ux/interaction-spec.md)
- [Funções, componentes e interação da janela principal](../ux/main-window-components.md)
- [Design e integração da árvore de diretórios](../ux/directory-tree.md)
- [Página inicial da janela principal](../ux/welcome.md)
- [Camada de carregamento da página inicial e conclusão da criação](../ux/welcome-opening.md)
- [Entrada de senha na extração](../ux/password-extraction.md)
- [Janelas de tarefas e verificação de integridade](../ux/task-windows.md)
- [Janela de configurações e idioma da interface](../ux/settings-localization.md)
- [Terminologia da interface em chinês simplificado](../ux/chinese-terminology.md)
- [Menus do sistema e área de notificação](../ux/desktop-integration.md)
- [Janelas e barras de título por plataforma](../ux/platform-windows.md)
- [Estados de alta fidelidade e contrato de implementação](../ux/hifi-state-spec.md)
- [Protótipo de interação v0.1 e mapeamento de recursos](../ux/prototype-spec.md)

<a name="xdzip-platform-integration-releases-and-trust"></a>

## Integração, lançamentos e confiança

- [Instaladores e associações de arquivos](../engineering/installers-associations.md)
- [Suporte a sistemas e política de manutenção](../engineering/support-policy.md)
- [Ações do gerenciador de arquivos e host de tarefas independente](../engineering/shell-actions.md)
- [Atualização iniciada pelo usuário](../engineering/software-update.md)
- [Ciclo de vida dos dados e diagnóstico](../engineering/privacy.md)
- [Versão do produto e identidade da compilação](../engineering/versioning.md)
- [Protocolo GitHub Release v1](../engineering/release-protocol.md)
- [DMG de desenvolvimento local para macOS](../engineering/macos-deployment.md)
- [Diretório de execução Linux independente e verificação](../engineering/linux-deployment.md)
- [Verificação de dependências da implantação e SBOM](../engineering/deployment-audit.md)
- [Atribuições e licenças dos componentes utilizados](../engineering/runtime-attribution.md)
- [Origem e avisos dos componentes nativos](../engineering/native-runtime-materials.md)
- [Correspondência entre Qt SDK e código-fonte](../engineering/qt-sdk-correspondence.md)
- [Licenças de ferramentas de desenvolvimento e execução no Windows](../engineering/windows-licensing.md)
- [Decisão de licença e materiais de terceiros](../engineering/licensing.md)
- [Admissão de dependências e mecanismos de formato](../engineering/dependencies.md)
- [Código-fonte correspondente e materiais de recompilação](../engineering/source-delivery.md)
- [Reconstruir dados ICU 73.2 a partir do CLDR](../engineering/icu-cldr-rebuild.md)

<a name="xdzip-build-and-cli-reference"></a>

## Referência de build e CLI

- [Projeto de compilação, implantação e lançamento](../design/build-release.md)
- [Convenções de código e camadas de verificação](../design/quality-engineering.md)
- [Preparação do ambiente de desenvolvimento](../engineering/development.md)
- [Compilações de desenvolvimento e verificação por plataforma](../engineering/developer-build.md)
- [Versões fixadas de ferramentas e plataformas](../engineering/toolchain.md)
- [Comandos CLI, JSON, instalação e comprovantes de atualização](../engineering/command-line.md)
