<a name="xdzip-coding-conventions-and-quality-engineering"></a>

# Convenções de código e engenharia de qualidade

[English](../../design/quality-engineering.md) | [简体中文](../../zh-CN/design/quality-engineering.md) | [繁體中文（台灣）](../../zh-TW/design/quality-engineering.md) | [日本語](../../ja-JP/design/quality-engineering.md) | [한국어](../../ko-KR/design/quality-engineering.md) | [Français](../../fr-FR/design/quality-engineering.md) | [Deutsch](../../de-DE/design/quality-engineering.md) | [Español](../../es-ES/design/quality-engineering.md) | Português (Brasil) | [Русский](../../ru-RU/design/quality-engineering.md)

<a name="xdzip-code-and-ownership"></a>

## Código e propriedade

- **Rust.** Siga as ferramentas, os lints e o arquivo de bloqueio do workspace. Valide comprimentos, deslocamentos e conversões de inteiros externos. Preserve tamanhos desconhecidos e erros tipados. Documente precondições de unsafe e FFI, propriedade, duração e falhas. Mantenha seções críticas curtas e faça E/S de arquivos compactados fora delas.
- **C++ e Qt.** Use C++17, RAII e regras claras de propriedade de QObject. Altere modelos na thread gráfica, passe valores de propriedade definida entre threads e vincule notificações em fila a um receptor ativo. Mantenha o encerramento assíncrono quando o receptor ainda depende da interface para esvaziar eventos. Rust continua responsável pelas políticas de arquivos compactados e arquivos. [Ponte Qt/Rust](qt-rust-bridge.md)
- **QML.** Expresse estado e intenção por meio de controladores. Reutilize controles comuns e recursos visuais e de texto gerados. Mantenha E/S e interpretação dos formatos fora dos delegates. Mudanças de janelas seguem o registro, o comportamento nativo e os padrões visuais. [Interface Qt Quick](qt-quick-ui.md), [Fluxo visual](../visual/design-workflow.md)

Documente as invariantes e o que permanece após falhas. No código de arquivos, explique o que a autorização de gravação final protege, quais identidades são verificadas e o que pode já estar visível após um erro. [Publicação de arquivos](file-security.md)

<a name="xdzip-verification-layers"></a>

## Camadas de verificação

| Alteração | Evidência aplicável |
| --- | --- |
| Domínio, consultas e serviços | Testes Rust de identidade, limites, nomes duplicados, ordem, respostas antigas, entrega limitada e cancelamento. |
| Operação de formato | Dados de teste próprios, testes de ida e volta independentes quando aplicáveis e casos de rejeição, corrupção, perfis incompatíveis e cotas. |
| Arquivos nativos e tarefas | Identidades e bytes realmente gravados, corridas de cancelamento e gravação final, proteção da saída existente e limpeza na plataforma afetada. |
| Desktop, Qt e acessibilidade | Testes de modelos e controladores, verificações QML e de controles compartilhados, interação nativa quando pertinente e evidência visual ou de tecnologia assistiva. |
| Desempenho e entrega | Cargas ligadas ao commit, métricas do renderizador e dos recursos, ou identidade de compilação por canal, resultados de instalação e atualização e comprovantes de dependências. |

O [guia de verificação](../../../developers/qa/README.md) determina os controles necessários. Conformidade do protocolo, testes de dados ou sem tela, automação nativa e observação manual comprovam aspectos diferentes. Consulte o [guia de compilação](../engineering/developer-build.md) e a [matriz de rastreabilidade](requirements-traceability.md) para pontos de teste existentes e limites dos entregáveis.

<a name="xdzip-existing-ci-and-additional-techniques"></a>

## CI e técnicas adicionais

A [CI de rotina](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml) verifica documentação, recursos gerados, contratos de layout, formatação e estrutura de workflows. Verificações nativas mais pesadas de Rust, formatos e Qt são disparadas separadamente em Foundation e Qt. Tarefas próprias preparam artefatos de lançamento. A [política de CI](../../../developers/engineering/ci-policy.md) descreve os gatilhos e as condições. Os caminhos e plataformas afetados definem quais verificações locais e remotas executar.

Use fuzzing, sanitizadores, verificação de modelos e testes de desempenho quando cobrirem a mudança. Escolha dados de teste limitados e redistribuíveis e guarde os resultados à parte. Para relatos sensíveis, siga a [política de segurança](../../../SECURITY.md). Planos e exemplos não são execuções agendadas nem resultados de aceitação.

<a name="xdzip-maintaining-the-design"></a>

## Manutenção do projeto

Se a implementação mudar identidade, comando, capacidade, tarefa ou limite de gravação, atualize o projeto, a referência técnica e os links de aceitação correspondentes. Mantenha todas as edições de idioma alinhadas. As [decisões de arquitetura](architecture-decisions.md) registram motivos, e o [projeto de desempenho](performance.md) define como medir. Registre resultados e limitações reais junto da mudança avaliada.
