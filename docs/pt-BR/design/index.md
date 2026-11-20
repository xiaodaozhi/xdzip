<a name="xdzip-xdzip-design-guide"></a>

# Guia de projeto do XDZip

[English](../../design/index.md) | [简体中文](../../zh-CN/design/index.md) | [繁體中文](../../zh-TW/design/index.md) | [日本語](../../ja-JP/design/index.md) | [한국어](../../ko-KR/design/index.md) | [Français](../../fr-FR/design/index.md) | [Deutsch](../../de-DE/design/index.md) | [Español](../../es-ES/design/index.md) | Português (Brasil) | [Русский](../../ru-RU/design/index.md)

Comece pela [arquitetura do sistema](../../design/system-architecture.md) para entender como o desktop Qt Quick e a CLI independente em Rust acessam o núcleo de arquivos compactados. As páginas de projeto explicam os contratos atuais, as decisões e o escopo das propostas. O [manual do usuário](../user-guide/README.md) cobre instalação e operações cotidianas. Para compilar ou alterar o projeto, veja [Compilar e contribuir](../contributing/README.md).

<a name="xdzip-find-a-design-topic"></a>

## Como encontrar um tópico de projeto

| Tópico | Leituras |
| --- | --- |
| Comportamento do produto | Os [requisitos do produto](product-requirements.md) definem as operações esperadas e seu escopo. A [rastreabilidade dos requisitos](requirements-traceability.md) identifica as verificações correspondentes. |
| Dados de arquivos compactados | O [modelo de domínio](domain-data.md) descreve sessões, versões da origem, entradas físicas, consultas e entradas das operações. |
| Desktop | A [interface Qt Quick](qt-quick-ui.md) explica visualizações, propriedade dos controladores e interações. A [ponte Qt/Rust](qt-rust-bridge.md) detalha comandos, identificadores, eventos e ciclos de vida reais. |
| Formatos e extensões | Os [adaptadores de formato e o experimento de extensão](codec-extension-sdk.md) explicam o registro integrado, o processo incluído e o experimento XDP1 independente. |
| Saída e tarefas em segundo plano | A [autorização e publicação segura](file-security.md) cobre preparação, verificação e gravação final. O [agendamento de tarefas](task-scheduling.md) detalha admissão, solicitações, progresso e cancelamento. |
| Desempenho e qualidade | O [projeto de desempenho](performance.md) explica orçamentos, medições e metas. A [engenharia de qualidade](quality-engineering.md) relaciona mudanças a verificações. |
| Entrega e decisões | [Compilação, implantação e lançamento](build-release.md) cobre plataformas, pacotes e atualizações. As [decisões](architecture-decisions.md) registram motivos, e a [evolução da arquitetura](architecture-evolution.md) mostra o caminho do protótipo ao produto. |
| Terminologia | A [terminologia e os limites](glossary.md) explica o vocabulário e aponta os limites de cada subsistema. |

<a name="xdzip-interpreting-the-references"></a>

## Entenda as referências

O comportamento atual é documentado com links para a implementação. As opções de projeto e ferramentas experimentais informam seus escopos. A [referência técnica](../reference/README.md) leva a cada contrato. O [README do projeto](../../../README.pt-BR.md) e as notas da versão descrevem as distribuições aceitas e os recursos entregues. Resultados de tempo, recursos e verificações nativas se aplicam à compilação, plataforma e carga registradas.
