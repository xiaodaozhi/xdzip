<a name="xdzip-repository-layout-and-dependencies"></a>

# Organização do repositório e dependências

[English](../../engineering/repository-layout.md) | [简体中文](../../zh-CN/engineering/repository-layout.md) | [繁體中文](../../zh-TW/engineering/repository-layout.md) | [日本語](../../ja-JP/engineering/repository-layout.md) | [한국어](../../ko-KR/engineering/repository-layout.md) | [Français](../../fr-FR/engineering/repository-layout.md) | [Deutsch](../../de-DE/engineering/repository-layout.md) | [Español](../../es-ES/engineering/repository-layout.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/repository-layout.md)

Os arquivos Cargo.toml e CMakeLists.txt da raiz definem os módulos atuais do produto. A tabela descreve os diretórios existentes. As divisões previstas na arquitetura ainda são propostas.

| Diretório | Responsabilidades e dependências |
| --- | --- |
| apps/desktop | Qt/QML, controladores C++ leves, modelos, configurações e interação com o sistema. Acessa o núcleo pela ponte. |
| crates/xdzip-cli | Argumentos, terminal, JSON, instalação gerenciada e host de atualização sem Qt. Depende diretamente do ambiente de execução, da E/S segura e do núcleo de atualização. Delega a interpretação dos formatos. |
| crates/xdzip-domain | Tipos puros e invariantes. Sem dependência de Qt ou codecs concretos. |
| crates/xdzip-codec-api | Contratos de capacidades, entrada emprestada, saída codificada e cancelamento. Depende do domínio. |
| crates/xdzip-compression | Algoritmos e encapsulamento compartilhados para Deflate/Deflate64/GZIP, LZMA/LZMA2, BZip2, PPMd7, MSZIP, LZX e Quantum. Sem dependências de terceiros em produção nem autoridade sobre caminhos ou publicação. Veja o [contrato](../../engineering/independent-rust-codecs.md). |
| crates/xdzip-crypto | Hashes SHA/HMAC e BLAKE independentes de formato, com memória fixa e estado sensível apagável. Reutiliza primitivas admitidas. Outros módulos interpretam senhas e decidem sobre a publicação. |
| crates/xdzip-codec-zip, xdzip-codec-tar | Implementações ZIP e TAR/GZIP sem caminhos de saída arbitrários. |
| crates/xdzip-codec-lha, xdzip-codec-arj, xdzip-codec-arc, xdzip-codec-bh | Perfis independentes de estrutura e entradas para [LHA](../../engineering/lha-containers.md)/[PMA](pma-streams.md), [ARJ](arj-streams.md), [ARC](arc-streams.md) e [BH](bh-containers.md), sem autoridade sobre o sistema de arquivos. O registro de formatos determina a disponibilidade no produto. |
| crates/xdzip-codec-7z | Contêiner 7z próprio, grafos de métodos, filtros, composição AES, criação em fluxo e regravação limitada. Não controla caminhos de arquivos. |
| crates/xdzip-codec-cab | Contêiner CAB próprio somente para leitura com volumes autorizados pelo host e algoritmos compartilhados. Preserva nomes brutos, declarações UTF-8 e registros comuns de nomes duplicados. A descoberta de volumes e a publicação cabem ao host. Veja o [contrato](../../engineering/independent-rust-codecs.md#cab只读规格) e as [fontes](../../engineering/cab-format-sources.md). |
| crates/xdzip-codec-optical | Leitores ISO9660/Joliet/Rock Ridge/UDF e gravação ISO9660/Joliet para novas imagens de dados. Usa fluxos sem controlar caminhos. |
| crates/xdzip-codec-bridge | Ponte CXX leve entre o processo isolado e os codecs em Rust de 7z, RAR, CAB e mídias ópticas. Fluxos autorizados, limites de recursos e cancelamento. |
| crates/xdzip-codec-native, tools/native-codec | Processo isolado para módulos em Rust de 7z/RAR/mídias ópticas/CAB. A camada leve de C++ fornece apenas o protocolo e os serviços de calendário e codificação do sistema. O receptor nativo verifica a saída já contabilizada e o trabalho auxiliar de forma independente. Veja o [contrato de E/S](native-codecs.md). |
| crates/xdzip-extract-plan | Verificação prévia de metadados e relatórios limitados, sem autorização do destino nem permissão de gravação. |
| crates/xdzip-io | Árvores de entrada, autorização e identidade nativas, preparação segura e confirmação de gravação. |
| crates/xdzip-runtime | Sessões, índices limitados em memória ou disco, consultas, seleção, prévias limitadas, tarefas de criação e extração e agendamento. |
| crates/xdzip-qt-bridge | Ponte de tipos cxx, dois identificadores e ciclo de vida. Depende do ambiente de execução. |
| crates/xdzip-plugin-protocol | CBOR e enquadramento de mensagens com limites e apenas a biblioteca padrão. Sem autoridade sobre caminhos, processos ou Qt. |
| crates/xdzip-plugin-host, examples/sdk | Experimento independente de SDK somente para leitura, E/S controlada, requisitos de isolamento nativo e contraparte Python independente. Não faz parte da descoberta de formatos nem das dependências de execução do produto. |
| tools | Sondas independentes da ponte, formatos, E/S e contagem de processos. A interface gráfica tem seus próprios pontos de entrada. |
| scripts, .github | Geração, verificações, ferramentas de implantação, CI e modelos de colaboração. |
| release-notes | Notas revisáveis de cada versão e metadados de versão e compilação. Relatórios de execução e pacotes gerados ficam em outros diretórios. |
| design/hifi, design/html, design/assets | Originais de alta fidelidade, projetos interativos e recursos de entrada do ambiente de execução. |
| docs | Projetos duradouros em Markdown, contratos técnicos, métodos de verificação reutilizáveis e imagens necessárias. Os relatórios de execução ficam em outros diretórios. |
| specs | Inventários de aspectos visuais, terminologia e dependências legíveis por máquina. Atendem aos recursos do produto, geradores e verificações. |
| third_party | Fontes de terceiros e materiais de licença mantidos com o código. |

Os módulos independentes xdzip-index e xdzip-platform e as ferramentas completas de conformidade para formatos externos fazem parte da arquitetura futura. Hoje, a indexação está no ambiente de execução, e as operações de arquivos por plataforma estão em io e nos adaptadores de desktop. O módulo plugin-host existente serve ao [experimento de SDK somente para leitura](sdk-trial.md). A maior parte dos testes fica junto aos módulos Rust, em apps/desktop/tests e nas verificações de scripts. Módulos vazios sem implementação não são criados.

O produto encaminha CAB sempre para Rust. Os componentes oficiais de CAB e as variantes LZX/Quantum específicas de CAB ficam fora da compilação. As chamadas continuam usando o mesmo processo isolado, sem novos executáveis do produto, opções de teste ou alternativa automática em caso de falha. codec-api define a contabilização antecipada de quadros completos. Os contêineres mantêm créditos por quadro, e os receptores nativos e o ambiente de execução verificam seus próprios créditos e limites de saída de forma independente. Veja o [contrato interno](../../engineering/independent-rust-codecs.md#cab只读规格).

Consulte a [arquitetura do sistema](../../design/system-architecture.md) para entender as dependências. Evite dependências inversas do domínio para o ambiente de execução, notificações diretas de codecs para QML e decisões sobre os textos da interface nas camadas de plataforma. Guarde as ligações geradas, amostras, relatórios e resultados de compilação em build/, target/ ou dist/ fora dos diretórios de código. [Instalação e associações](../../engineering/installers-associations.md#兼容基线的长期输入) descreve as entradas de verificação de compatibilidade histórica que devem ser preservadas.

`tools/image-preview/` é um processo interno QtGui/QtSvg para imagens estáticas. Recebe conteúdo verificado com limites e devolve dados RGBA limitados. Não mantém caminhos dos arquivos compactados nem dos destinos. Veja as plataformas e o tratamento de falhas no [contrato de prévias](content-preview.md).
