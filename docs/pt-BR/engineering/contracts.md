<a name="xdzip-interface-contracts-and-checks"></a>

# Contratos de interfaces e verificações

[English](../../engineering/contracts.md) | [简体中文](../../zh-CN/engineering/contracts.md) | [繁體中文（台灣）](../../zh-TW/engineering/contracts.md) | [日本語](../../ja-JP/engineering/contracts.md) | [한국어](../../ko-KR/engineering/contracts.md) | [Français](../../fr-FR/engineering/contracts.md) | [Deutsch](../../de-DE/engineering/contracts.md) | [Español](../../es-ES/engineering/contracts.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/contracts.md)

As interfaces de referência estão descritas no [domínio e dados](../design/domain-data.md), na [ponte Qt/Rust](../design/qt-rust-bridge.md), no [projeto de extensões de formato](../design/codec-extension-sdk.md) e no [agendamento](../design/task-scheduling.md). Esta página define as verificações antes de fixar interfaces. A ponte interna, os dois identificadores de recursos e seu ciclo de vida já estão implementados. Veja os limites no [contrato de execução](product-runtime.md). As gerações de consultas, verificações de versão da origem e tarefas de criação e extração também estão implementadas. Veja [navegação](readonly-browser.md), [extração segura](zip-safe-extraction.md) e [criação](zip-creation.md). O protocolo de processos de terceiros tem um [experimento independente de SDK somente para leitura](sdk-trial.md). Ele ainda não foi lançado como SDK estável nem integrado ao produto. A tabela lista os requisitos de integração.

| Interface | O que confirmar antes da implementação | Falhas a testar |
| --- | --- | --- |
| Interface → ponte | Propriedade dos identificadores, RequestId, envio sem bloqueio, rejeição durante o encerramento | Fila cheia, encerramento repetido, objeto destruído |
| Ponte → interface | session/query/generation/revision, páginas em lotes, sequência de eventos | Respostas antigas descartadas, falha ao reduzir inteiro, estado terminal preservado |
| Execução → codec | Capacidades em vez de extensões, leitor de entrada, destino de saída, token de cancelamento | Recursos sem suporte, erros de CRC, leitura ou gravação incompleta |
| Execução → E/S segura | Raiz autorizada, mapeamento, preparação, autorização e recibo de gravação final | Corridas em links, disco cheio, destino de substituição alterado |
| Extensão → host | Versão da negociação, limites de quadros, capacidades, ciclo de vida da sessão | Quadros inválidos ou grandes demais, tempo esgotado, reordenação, recusa de cancelamento |

Cada interface define validade de entrada, propriedade, threads, orçamentos, categorias de erro, idempotência e limites de cancelamento. Um valor desconhecido é diferente de zero. TaskId e EntryId têm papéis distintos. A seleção fica vinculada às entradas e independe dos números de linha atuais.

CLI e interface gráfica compartilham serviços Rust. Catalog retorna no máximo 128 entradas físicas de metadados por página. SelectMembers resolve caminhos efetivos completos e EntryId na mesma sessão imutável e envia as tarefas existentes. Identificadores de lista pertencem àquela varredura e deixam de valer após mudanças no arquivo compactado. submit_wait espera apenas pelo breve bloqueio de admissão. Capacidade da fila e regras de rejeição no encerramento continuam iguais. A interface gráfica não expõe a codificação QML desses comandos internos. A CLI não grava arquivos com o texto de exibição escapado.

Metadados fixos do host usam open_regular_nofollow com autorização de diretório e verificação de tamanho e identidade. Links simbólicos e pontos de nova análise no componente final são recusados. A leitura de arquivos compactados selecionados pelo usuário conserva suas regras de aliases de caminho. Solicitações da CLI para o desktop enviam um diretório limitado pela extensão opcional `cli_protocol=1` de instância única do mesmo usuário. O protocolo antigo de abertura permanece na versão 1. Instâncias sem a extensão não confirmam a entrega pela CLI. Veja o [contrato da CLI](command-line.md) para resultados, códigos de saída e limites de instalação.

Contratos internos podem mudar com a implementação e devem ser atualizados no mesmo commit. O protocolo para terceiros só é considerado v1 estável após duas implementações independentes interoperarem e passarem nos testes de compatibilidade. Versões desconhecidas são rejeitadas explicitamente ou negociadas para uma versão compatível.
