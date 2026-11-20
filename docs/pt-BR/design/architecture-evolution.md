<a name="xdzip-architecture-evolution-and-current-boundaries"></a>

# Evolução da arquitetura e escopo atual

[English](../../design/architecture-evolution.md) | [简体中文](../../zh-CN/design/architecture-evolution.md) | [繁體中文](../../zh-TW/design/architecture-evolution.md) | [日本語](../../ja-JP/design/architecture-evolution.md) | [한국어](../../ko-KR/design/architecture-evolution.md) | [Français](../../fr-FR/design/architecture-evolution.md) | [Deutsch](../../de-DE/design/architecture-evolution.md) | [Español](../../es-ES/design/architecture-evolution.md) | Português (Brasil) | [Русский](../../ru-RU/design/architecture-evolution.md)

<a name="xdzip-from-framework-experiments-to-the-product"></a>

## Dos experimentos com frameworks ao produto

Os primeiros experimentos com frameworks usavam uma lista de arquivos compactados fictícia e tarefas simuladas. O produto adotou Qt Quick e uma ponte CXX tipada. Em seguida, implementou em Rust as operações reais de arquivos compactados, tarefas e arquivos. O JSON, a ABI C e os tempos medidos no protótipo pertencem a esses experimentos. As [decisões de arquitetura](architecture-decisions.md) registram os motivos das escolhas e as condições para revê-las.

O produto limita os modelos de linhas e verifica a identidade das consultas para evitar que um resultado antigo substitua a visualização nova. Ele acrescenta identidades físicas de entradas, capacidades explícitas por formato, permissões de arquivos, saída temporária e publicação verificada. A CLI independente usa o mesmo núcleo sem Qt. Veja a [arquitetura do sistema](system-architecture.md) e o [modelo de domínio](domain-data.md).

<a name="xdzip-current-experimental-and-proposed-work"></a>

## Recursos atuais, experimentos e propostas

| Área | Estado atual |
| --- | --- |
| Execução e indexação | Implementadas no host. Um ZIP/ZIP64 comum de arquivo único pode migrar para um índice privado em disco. Os limites das tarefas e dos metadados são distintos. [Projeto de desempenho](performance.md) |
| Isolamento por formato | O roteamento integrado usa Rust no processo ou o processo de formato incluído. O registro, o perfil e a admissão em execução determinam as operações disponíveis. [Adaptadores de formatos](codec-extension-sdk.md) |
| Extensões externas | O experimento independente XDP1 somente para leitura usa um protocolo com limites e requisitos por plataforma. Ele é executado em um host separado. Descobrir, instalar e ativar extensões no produto ainda são propostas. [Experimento do SDK](../engineering/sdk-trial.md) |
| Estado durável e concorrência | O agendador atual admite uma operação pesada de desktop por vez. Índices persistentes entre execuções, recuperação de tarefas e agendamento compartilhado geral ainda são opções de projeto. [Agendamento de tarefas](task-scheduling.md), [Publicação de arquivos](file-security.md) |

Consulte a [tabela de formatos](../../../README.pt-BR.md#xdzip-format-support) para saber o que o código atual oferece. Para uma compilação publicada, use as notas e os artefatos daquela versão. Implementação nativa, ferramentas experimentais e resultados medidos têm escopos próprios.
