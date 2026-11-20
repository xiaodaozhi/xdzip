# xdzip: registros de decisões de arquitetura

[English](../../design/architecture-decisions.md) | [简体中文](../../zh-CN/design/architecture-decisions.md) | [繁體中文](../../zh-TW/design/architecture-decisions.md) | [日本語](../../ja-JP/design/architecture-decisions.md) | [한국어](../../ko-KR/design/architecture-decisions.md) | [Français](../../fr-FR/design/architecture-decisions.md) | [Deutsch](../../de-DE/design/architecture-decisions.md) | [Español](../../es-ES/design/architecture-decisions.md) | Português (Brasil) | [Русский](../../ru-RU/design/architecture-decisions.md)

Os primeiros registros são de 11 de outubro de 2026. Os posteriores informam suas próprias datas. Uma decisão aceita está adotada. As notas sobre a implementação atual relacionam cada decisão aos contratos mantidos e às evidências de capacidade. Ao substituir uma decisão, preserve o registro original e acrescente outro.

## ADR-001: Qt Quick como única interface do produto

**Status: aceita.**

**Implementação atual.** O aplicativo de desktop usa Qt Quick/QML com adaptadores C++ enxutos. O executável SFX do Windows com caixas de diálogo do sistema é a exceção limitada descrita na ADR-013. [Interface de desktop](qt-quick-ui.md)

Contexto: o produto precisa de uma interface de desktop moderna, experiência consistente e baixo consumo de recursos. Foram concluídos protótipos Slint, Tauri e Qt Quick com o mesmo núcleo. Considerando os recursos da interface, a consistência e os dados disponíveis de renderização por software no Linux, o projeto escolheu Qt Quick para a etapa de produto; as demais plataformas ainda precisam ser verificadas.

Decisão: Qt Quick/QML e Qt Quick Controls 2 com uma adaptação C++ enxuta. Slint e Tauri permanecem como experimentos históricos e não são distribuídos no produto.

Custos: ambiente de execução Qt, módulos QML, compilações em várias linguagens e trabalho de licenciamento. Entrada, acessibilidade, GPU e implantação em Mac e Windows exigem testes adicionais. O executável de 0,80 MiB do protótipo não é um instalador Qt completo.

Condição de reavaliação: o Qt não atende aos requisitos de funcionalidade, licença ou manutenção de segurança nas plataformas de destino e continua fora dos orçamentos essenciais após otimização efetiva. Flutuações comuns de desempenho não provocam outra seleção de framework.

## ADR-002: o Rust é responsável por toda a lógica de arquivos compactados

**Status: aceita.**

**Implementação atual.** O ambiente Rust gerencia o estado dos arquivos compactados e as tarefas. A seleção de arquivos expressa uma intenção da interface; a camada de E/S do host autoriza o acesso ao sistema de arquivos. [Arquitetura do sistema](../../design/system-architecture.md)

Decisão: domínio, índices, planos, tarefas, cotas, políticas de caminhos e registro de formatos ficam no Rust. O Qt representa o estado, autoriza a seleção de arquivos do sistema e transmite as decisões do usuário. Os testes de lógica não precisam do Qt.

Alternativas: conectar libarchive ou 7zip diretamente ao C++/QML ou repetir a lógica de controle em cada interface. Isso acopla o núcleo às interfaces de cada plataforma e impede uma verificação consistente de segurança e cancelamento. O Rust não elimina riscos das bibliotecas nativas; bibliotecas relevantes podem ser isoladas em processos de trabalho.

Condição de reavaliação: se uma API de plataforma não puder ser implementada de forma confiável em Rust, permitir uma pequena adaptação em C++ sem transferir a responsabilidade pela lógica.

## ADR-003: ponte cxx gerada na compilação

**Status: aceita.**

**Implementação atual.** O produto compila e utiliza a ponte tipada com handles separados para comandos e eventos. O receptor de eventos e o ciclo de encerramento são definidos no [contrato da ponte](qt-rust-bridge.md).

Decisão: usar no produto uma ponte cxx com segurança de tipos, compilada a partir do mesmo commit. Não manter uma ABI C escrita à mão a longo prazo nem serializar JSON nos caminhos críticos. A ponte não contém lógica de produto em Qt.

Alternativas: manter a ABI C experimental, gerar QObject com CXX-Qt ou conectar a interface e todo o núcleo por IPC local. O cxx preserva limites explícitos de modelos e threads Qt sem acrescentar inicialmente um framework gerador de QObject. Em troca, é preciso manter a camada QObject enxuta e a coordenação CMake/Cargo.

Limites: o cxx não garante compatibilidade binária dinâmica entre quaisquer versões de Rust e Qt e não gera um protocolo de formatos de terceiros.

Condição de reavaliação: a verificação da compilação de desktop ou do ciclo de vida suportado falha, ou a camada de adaptação cresce bastante. Avaliar CXX-Qt ou uma ABI C explicitamente versionada em uma ADR, sem mudar silenciosamente.

## ADR-004: separar o SDK de código integrado do protocolo de processos externos

**Status: limite de projeto aceito. O protocolo externo é experimental.**

**Implementação atual.** Os formatos do produto são registrados explicitamente e integrados. O processo de formatos distribuído com o produto usa seu próprio protocolo. A implementação CBOR externa é um teste independente somente de leitura. [Adaptadores de formato e teste](codec-extension-sdk.md)

Decisão: usar interfaces de código Rust para formatos integrados e um protocolo separado e limitado de processos para extensões externas. O host mantém a autoridade sobre E/S.

Motivo: a ABI de bibliotecas dinâmicas Rust é instável e plugins nativos arbitrários dentro do processo prejudicam o isolamento de falhas e a gestão de recursos. Wasm não é prioridade como ambiente geral de formatos devido aos custos de migração, threads, acesso por posição e verificação de desempenho dos codecs nativos existentes; seu uso futuro não está descartado.

Custos: protocolo, intermediário, sandbox e sobrecarga de IPC. O protocolo só será congelado depois da interoperabilidade de duas implementações independentes.

## ADR-005: o host controla centralmente as gravações reais

**Status: aceita.**

**Implementação atual.** O ambiente prepara as operações e `xdzip-io` controla autorizações, preparação temporária e confirmação. Os adaptadores de formato trabalham com leitores e destinos de saída. [Autorização de arquivos](file-security.md)

Decisão: os mecanismos escrevem somente em `OutputSink` autorizados. O host controla caminhos reais, conflitos, preparação temporária e publicação. Não chamar diretamente a extração geral de uma biblioteca para diretórios arbitrários, a menos que suas adaptações tenham sido integralmente verificadas como restritas pelo host.

Motivo: implementar proteção de caminhos separadamente para cada formato pode produzir inconsistências; verificar strings não impede links ou condições de corrida. O custo é implementar e testar falhas das camadas seguras de arquivos em três plataformas.

Limites: o host não pode provar que a saída de um plugin malicioso está semanticamente correta nem impedir um processo malicioso que já controle o mesmo usuário. A confiança e as permissões de plugins devem permanecer explícitas.

## ADR-006: publicar arquivos extraídos individualmente e novos arquivos compactados por inteiro

**Status: aceita.**

**Implementação atual.** A extração publica os arquivos individualmente. A criação e a edição verificam os arquivos compactados temporários antes de substituir ou publicar. A criação em volumes publica um conjunto verificado parte por parte e tenta uma reversão limitada após falha parcial. [Semântica de publicação](file-security.md)

Decisão: cada arquivo extraído é preparado e verificado antes da publicação; uma tarefa com falha pode manter resultados parciais. Novos arquivos compactados são escritos por completo antes da publicação. Sobrescrever não é o padrão.

Motivo: a mesclagem em um diretório existente não oferece facilmente uma transação realmente atômica de todo o diretório em todas as plataformas. Fingir que toda a tarefa será revertida poderia apagar alterações posteriores do usuário. São necessários resultados e registros de recuperação detalhados e uma explicação clara do cancelamento.

Condição de reavaliação: um novo modo para publicar um diretório inteiramente novo pode ser implementado separadamente em condições restritas, sem alterar a semântica de mesclagem em diretórios existentes.

## ADR-007: atores e grupos de trabalho limitados com envio de eventos

**Status: aceita.**

**Implementação atual.** Cada conexão usa threads separados para comandos e tarefas; o desktop admite tarefas pesadas em série, e as filas Rust e a caixa de mensagens Qt são limitadas. Um grupo generalizado de tokens de recursos continua sendo uma opção de projeto. [Agendamento de tarefas](task-scheduling.md), [ponte Qt/Rust](qt-rust-bridge.md)

Decisão: um agendador leve decide o estado, grupos limitados executam o trabalho e o Qt possui um thread separado para receber eventos. Agrupar atualizações de progresso, entregar estados finais e reservar admissão de controle independente para cancelamento e encerramento.

Alternativas: chamadas síncronas da interface, um thread por arquivo, Tokio ou `spawn_blocking` sem limites, ou consulta contínua. O trabalho com arquivos compactados usa principalmente CPU e E/S síncronas; não criar tarefas ilimitadas apenas porque uma operação é chamada de assíncrona. Futuras E/S reais de rede podem usar um ambiente assíncrono localmente.

Custos: controle de pressão, encerramento e reservas de recursos explícitos. Benefícios: baixo custo quando ocioso e uma interface que as tarefas não conseguem sobrecarregar.

## ADR-008: formatos limitados na primeira versão conforme suas capacidades

**Status: aceita.**

**Implementação atual.** Os formatos iniciais são contexto histórico. O registro mantido e a admissão em execução definem as capacidades atuais do código, incluindo perfis adicionais de leitura e criação e restrições próprias de cada operação. [Registro de formatos](../engineering/format-registry.md)

Decisão: a v0.1 usa ZIP Store/Deflate e TAR/GZIP; criptografia, volumes, RAR, 7z e algoritmos adicionais são verificados individualmente. A interface não deduz suporte completo pela extensão.

Motivo: o nome de um formato não descreve todos os seus recursos reais. A tabela inicial de formatos é menor, mas honesta e verificável pelos usuários.

## ADR-009: separar pequenos dados do produto do conteúdo do usuário

**Status: aceita.**

**Implementação atual.** Configurações, índices e registros têm durações diferentes. Os índices em disco são privados e limitados à sessão. A reutilização persistente de índices e a retomada de tarefas incompletas continuam sendo opções de projeto. [Modelo de domínio](domain-data.md), [publicação de arquivos](file-security.md)

Decisão: configurações, índices temporários, registros de transações e manifestos de extensões têm ciclos de vida separados. Não há telemetria por padrão, senhas persistentes ou cache de índices de arquivos compactados entre inicializações.

Custos: vários esquemas e regras de limpeza. Reconsiderar índices entre inicializações exige projetar identidade e invalidação das origens, privacidade e capacidade.

## ADR-010: distribuição completa e licenças são requisitos de arquitetura

**Status: aceita.**

**Implementação atual.** Os binários do GitHub e os MSIX da Microsoft Store seguem canais de empacotamento documentados e distintos. Revisar as dependências realmente implantadas, a integridade dos artefatos e os materiais de licença de cada canal. [Compilação e publicação](build-release.md)

Decisão: implantar Qt dinamicamente por padrão e revisar as licenças dos módulos efetivamente usados. O tamanho da instalação inclui os ambientes de execução. Assinatura, SBOM e obrigações de substituição ou fornecimento de código-fonte entram nas verificações de publicação.

Alternativas: comparar somente executáveis ou adicionar `LICENSE` pouco antes da publicação. Nenhuma prova que o produto é pequeno ou pode ser distribuído legalmente.

Custos: manutenção contínua de implantação e licenças. Compilação estática do Qt ou distribuição em loja exige uma ADR e uma revisão de licenças separadas; uma PR de redução de tamanho não pode alterá-las incidentalmente.

<a name="xdzip-adr-011-initial-support-scope-and-personal-project-maintenance-policy"></a>

## ADR-011: escopo inicial e política de manutenção de projeto pessoal

**Status: aceita em 19 de outubro de 2026, com aprovação explícita do proprietário do projeto.**

**Implementação atual.** A [política de compatibilidade](../engineering/support-policy.md) e o [contrato de atualização](../../engineering/software-update.md) definem o suporte e as atualizações atuais. A cláusula de atualização manual descreve a v0.1.

Decisão: oferecer suporte inicial a macOS 14 ou posterior em Apple Silicon arm64, versões mantidas do Windows 11 x86_64 e Ubuntu 24.04 LTS x86_64 com X11 e glibc 2.39 ou posterior. Priorizar somente a versão pública mais recente, tratar os comentários conforme os recursos disponíveis sem prazo fixo de resposta, usar atualizações manuais na v0.1 e manter as prévias de desenvolvimento como experimentais. Veja a [política de compatibilidade](../engineering/support-policy.md).

Motivo: compilações nativas, instaladores e dependências disponíveis delimitam o escopo e reduzem promessas não verificadas de sistemas e arquiteturas de projetos anteriores à capacidade de manutenção de um projeto pessoal de código aberto. Não há promessa para Mac Intel, Windows 10, Windows/Linux ARM64, outras distribuições ou Wayland. A ampliação exige implementação e verificação reais antes da mudança.

Limite: uma distribuição pessoal de código aberto no GitHub não exige contas pagas de desenvolvedor, certificados de assinatura, Developer ID ou notarização. O macOS mantém a assinatura ad hoc necessária para integridade. Essa política não substitui a verificação da origem das licenças e dos pacotes nem pressupõe compatibilidade com todas as versões futuras dos sistemas.

## ADR-012: migrar cada formato para algoritmos Rust independentes

**Status: aceita em 25 de outubro de 2026, com aprovação explícita do proprietário para implementação. Substitui a escolha de longo prazo de mecanismos de formato de terceiros; não significa que a migração esteja concluída.**

**Implementação atual.** O roteamento do produto usa implementações de formato Rust. As leituras aplicáveis de 7z, RAR, CAB e mídias ópticas usam o processo de trabalho fornecido. A matriz atual inclui um gravador ISO dentro do processo. Confira cada operação no [registro de formatos](../engineering/format-registry.md) e no [projeto dos adaptadores](codec-extension-sdk.md).

Decisão: usar implementações Rust independentes para analisar formatos e para os algoritmos de compressão e descompressão; bibliotecas gerais de criptografia, codificação de caracteres e infraestrutura do sistema são permitidas. Definir primeiro as especificações de comportamento a partir das especificações oficiais e depois implementar independentemente; não traduzir o código de referência linha por linha nem copiá-lo com nomes alterados. Desenvolvedores que viram o código não afirmam ter realizado desenvolvimento estrito em sala limpa. Fontes insuficientes e restrições de licença exigem revisão separada, especialmente a proibição de desenvolver um codificador RAR.

O plano previa aceitar e substituir ZIP, TAR, TAR.GZ, 7z, RAR, CAB, ISO e UDF individualmente conforme as capacidades da época. Manter a implementação atual dos formatos ainda não aceitos; não usar fallback silencioso nem ampliar o escopo somente de leitura de RAR, CAB, ISO ou UDF. Reutilizar algoritmos compartilhados de compressão. Nomes, identidades, E/S autorizadas, recursos, cancelamento e publicações continuam sob o host Rust; o isolamento do processo de trabalho não desaparece com a mudança para Rust.

Custos: manter independentemente compatibilidade, segurança, desempenho, fuzzing e algoritmos a longo prazo. Referências de terceiros podem ser usadas em testes, mas formatos migrados devem remover os mecanismos correspondentes das dependências e dos arquivos entregues. O [contrato dos mecanismos próprios](../../engineering/independent-rust-codecs.md) descreve fontes e regras de interface; o progresso e a aceitação efetiva são acompanhados em [#45](https://github.com/xiaodaozhi/xdzip/issues/45).

<a name="xdzip-adr-013-standalone-windows-sfx-host"></a>

## ADR-013: executável SFX independente para Windows

**Status: aceita em 16 de novembro de 2026 para o primeiro perfil SFX autorizado pelo proprietário.**

**Implementação atual.** O [contrato SFX](../engineering/sfx-creation.md) define o perfil Windows e o ponto de entrada independente Rust com diálogos do sistema.

Contexto: um arquivo autoextraível gerado para Windows x64 deve funcionar sem xdzip ou Qt instalados. Distribuir outro aplicativo Qt completo em cada arquivo acrescentaria um ambiente de implantação desnecessário.

Decisão: o desktop mantém Qt Quick e reutiliza sua janela de criação. O executável SFX independente do projeto usa apenas diálogos de pasta e tarefa do Windows, com fontes, DPI, tamanho e acessibilidade controlados pelo sistema. ZIP, verificação, autorização de saída, conflitos, progresso, cancelamento e limpeza usam o núcleo Rust existente. É uma exceção limitada para o executável de distribuição em relação à ADR-001, não outro framework de desktop nem uma nova biblioteca de controles personalizados.

Custos: manter e testar automaticamente um executável Windows adicional, um envelope versionado, vínculo CRT estático, auditoria de dependências sem Qt e um host de diálogos nativos. Ele nunca executa o conteúdo extraído. A assinatura do executável gerado é separada da assinatura de um programa fixo.

Condição de reavaliação: qualquer solicitação de instaladores, execução automática de conteúdo, outras plataformas executáveis ou interface SFX personalizada exige escopo e revisão de arquitetura próprios.
