<a name="xdzip-requirements-and-verification-traceability"></a>

# Rastreabilidade de requisitos e verificações

[English](../../design/requirements-traceability.md) | [简体中文](../../zh-CN/design/requirements-traceability.md) | [繁體中文](../../zh-TW/design/requirements-traceability.md) | [日本語](../../ja-JP/design/requirements-traceability.md) | [한국어](../../ko-KR/design/requirements-traceability.md) | [Français](../../fr-FR/design/requirements-traceability.md) | [Deutsch](../../de-DE/design/requirements-traceability.md) | [Español](../../es-ES/design/requirements-traceability.md) | Português (Brasil) | [Русский](../../ru-RU/design/requirements-traceability.md)

Os [requisitos do produto](product-requirements.md) definem os identificadores e seu escopo atual. Cada linha indica o que verificar quando o comportamento muda. Resultados, plataformas e limites são registrados com a alteração correspondente. Os [casos de aceitação](../../../developers/qa/acceptance.md) indicam os pontos de entrada das verificações.

<a name="xdzip-requirement-matrix"></a>

## Matriz de requisitos

| Requisito | Referência de projeto | Foco da verificação |
| --- | --- | --- |
| P01 Explorar sem extrair | [Domínio](domain-data.md), [Desktop](qt-quick-ui.md) | T01 Abrir um arquivo compactado real e confirmar que a navegação não produz arquivos extraídos. |
| P02 Explorar, consultar e selecionar | [Domínio](domain-data.md), [Desempenho](performance.md) | T02 Identidade de entradas homônimas, consultas estáveis, páginas antigas, armazenamento de índices grandes e seleção entre páginas. |
| P03 Extrair o escopo pretendido | [Autorização de arquivos](file-security.md), [Tarefas](task-scheduling.md) | T03 Expansão de diretórios, duplicatas, renomeação, itens ignorados, sobrescrita e resultados exatos no disco. |
| P04 Criar um arquivo compactado | [Formatos](codec-extension-sdk.md), [Autorização de arquivos](file-security.md) | T04 Testes independentes de criação e leitura do perfil alterado, verificação temporária e proteção dos arquivos existentes. |
| P05 Respeitar as capacidades dos formatos | [Ponte](qt-rust-bridge.md), [Formatos](codec-extension-sdk.md) | T05 Registro, ações específicas de cada sessão e rejeição em execução de perfis sem suporte. |
| P06 Limitar tarefas e permitir cancelamento | [Ponte](qt-rust-bridge.md), [Tarefas](task-scheduling.md) | T06 Filas cheias, encaminhamento de decisões, concorrência entre cancelamento e confirmação e encerramento assíncrono. |
| P07 Proteger arquivos e recursos | [Autorização de arquivos](file-security.md), [Formatos](codec-extension-sdk.md) | T07 Verificações nativas aplicáveis de links e identidade, validação de caminhos, cotas e objetos protegidos. |
| P08 Oferecer um desktop acessível | [Desktop](qt-quick-ui.md) | T08 Comportamento dos modelos e controles, teclado nativo, IME, acessibilidade, escala e foco nas plataformas afetadas. |
| P09 Limitar e medir recursos | [Desempenho](performance.md) | T09 Compilação comparável, métricas da árvore de processos, inicialização, ordenação, vazão de saída ou tamanho do pacote conforme o caso. |
| P10 Manter Qt fora da lógica de formatos | [Arquitetura](../../design/system-architecture.md), [Formatos](codec-extension-sdk.md) | T10 Núcleo sem Qt, seleção explícita dos adaptadores e regressões específicas por formato. |
| P11 Proteger segredos e dados | [Autorização de arquivos](file-security.md), [Privacidade](../engineering/privacy.md) | T11 Uso limitado de senhas, diagnósticos sem dados sensíveis e acesso intencional à rede. |
| P12 Auditar a entrega | [Qualidade](quality-engineering.md), [Publicação](build-release.md) | T12 Entradas fixadas e identidade dos artefatos, inventário, licenças, confiança do pacote e verificações de instalação aplicáveis. |
| P13 Controlar extensões externas | [Formatos](codec-extension-sdk.md), [Experimento do SDK](../engineering/sdk-trial.md) | T13 Interoperabilidade e permissões nativas do experimento. A instalação de extensões no produto continua sendo uma opção de projeto. |
| P14 Limitar prévia e abertura | [Desktop](qt-quick-ui.md), [Autorização de arquivos](file-security.md) | T14 Prévia automática, conteúdo compatível, limites do processo e saídas temporárias privadas. |
| P15 Informar o resultado real | [Ponte](qt-rust-bridge.md), [Autorização de arquivos](file-security.md), [Tarefas](task-scheduling.md) | T15 Mudanças de origem ou destino, falhas e cancelamento após gravações confirmadas, limpeza e incerteza. |

<a name="xdzip-cross-cutting-verification"></a>

## Verificações transversais

| Conjunto | Alterações aplicáveis | Foco da verificação |
| --- | --- | --- |
| T16 Propriedade e ciclo de vida da ponte | Vida útil da ponte, conexão ou controlador | Identidade da solicitação atual, ativação do receptor, tarefas mantidas e destruição segura. |
| T17 Interoperabilidade e falhas de formato | Alterações de formato ou codec | Amostra válida independente quando aplicável, truncamento ou perfis sem suporte e saída limitada. |
| T18 Instalação, atualização e remoção | Alterações de plataforma ou canal de distribuição | Pacote real, dependências em ambiente limpo, identidade da instalação, encaminhamento da atualização e remoção ou recuperação compatível. |

<a name="xdzip-representative-workflows"></a>

## Fluxos representativos

1. Abrir e pesquisar um ZIP, fixar os resultados selecionados, extrair e comparar os bytes de saída. “Extrair tudo” continua abrangendo todo o arquivo compactado.
2. Extrair entradas homônimas ou conflitantes com uma política definida e verificar os nomes e estados resultantes.
3. Cancelar antes e durante a confirmação, verificar que os arquivos concluídos continuam informados e que arquivos compactados já verificados são preservados se a criação falhar antes da confirmação.
4. Alterar a origem ou o destino em um teste controlado e verificar o resultado real. Após uma interrupção anormal, examinar os itens restantes sem presumir repetição automática.
5. Provocar falha no processo de formato incluído ou no experimento separado do SDK conforme o contrato e verificar que dados incompletos nunca são publicados como sucesso.

As verificações estáticas de escrita, formato e links validam a documentação. As evidências com amostras, sem tela, automatizadas em ambiente nativo e manuais têm escopos próprios. [Engenharia de qualidade](quality-engineering.md), [Guia de verificação](../../../developers/qa/README.md)
