<a name="xdzip-read-only-zip-browsing-contract"></a>

# Contrato de navegação ZIP somente para leitura

[English](../../engineering/zip-readonly-browsing.md) | [简体中文](../../zh-CN/engineering/zip-readonly-browsing.md) | [繁體中文（台灣）](../../zh-TW/engineering/zip-readonly-browsing.md) | [日本語](../../ja-JP/engineering/zip-readonly-browsing.md) | [한국어](../../ko-KR/engineering/zip-readonly-browsing.md) | [Français](../../fr-FR/engineering/zip-readonly-browsing.md) | [Deutsch](../../de-DE/engineering/zip-readonly-browsing.md) | [Español](../../es-ES/engineering/zip-readonly-browsing.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/zip-readonly-browsing.md)

O contrato define as entradas diretamente na raiz, a busca em subpastas, a identidade física estável e a ordenação que exibe pastas primeiro. A navegação ZIP lê metadados sem decodificar o conteúdo das entradas nem verificar o CRC. Nenhum arquivo é extraído. ZIPs comuns de grande porte podem usar um [índice privado de metadados em disco](large-index.md).

Consulte a [indexação de arquivos compactados grandes](large-index.md) para saber como funcionam o índice atual de ZIPs comuns grandes e o modelo Qt sob demanda. Esta página mantém o limite inicial de 100.000 entradas e os requisitos de identidade das consultas. A ampliação da capacidade mantém os testes de erros, cancelamento e identidade.

<a name="xdzip-browsing-scope-and-limits"></a>

## Escopo e limites da navegação

- Metadados ZIP/ZIP64 de até 100.000 registros físicos. O [host de volumes divididos](archive-opening.md) converte os volumes antes da leitura. O codec continua rejeitando os metadados brutos de vários discos.
- Até 4 KiB por nome, 16 MiB no total de nomes brutos, 16 MiB no total de nomes de exibição decodificados e escapados, 64 MiB de leitura total e 256 KiB de buffer de varredura. Tamanhos declarados de 64 bits continuam sendo inteiros exatos.
- O índice de diretórios tem até 32.768 nós, 64 níveis e 4 MiB de nomes e prefixos no total. Se algum limite for excedido, descarte o índice de navegação inteiro e informe a perda de recursos. A árvore incompleta não é exibida.
- O índice de entradas diretas e os resultados são reutilizados. Os lotes de uma mesma consulta não voltam a filtrar ou ordenar os dados. Os nomes de cada lote estão limitados a 64 KiB. A interface mostra o diretório atual completo sem botões de paginação.
- Cada página valida a sessão, o ID da consulta, a geração da interface, a revisão do estado capturado e o deslocamento. Alterações na consulta cancelam o trabalho anterior e mantêm a solicitação mais recente. Respostas desatualizadas não substituem resultados novos.
- O tamanho da origem, a data de modificação, o dispositivo e o inode no Unix, e o número do volume e o índice do arquivo no Windows são verificados antes e depois da varredura, durante consultas e na verificação prévia. Em caso de `source_changed`, as consultas do Qt e a verificação prévia limpam a lista, a seleção, a árvore de diretórios e os detalhes invalidados. Reabra o arquivo compactado para iniciar uma nova sessão. A detecção é feita na medida do possível e não oferece um snapshot atômico do sistema de arquivos nem autorização para gravação segura.

<a name="xdzip-acceptance-exit-criteria"></a>

## Critérios de aceitação

Um ZIP64 gerado com 100.000 entradas deve concluir a varredura, a ordenação de todos os resultados, a recuperação, a navegação em pastas, a exibição da lista completa do Qt e a seleção. Cada lote compartilha a mesma identidade de consulta. Verifique ZIP64 danificado, volumes divididos, limites de nomes, exibição e leitura, cancelamento, respostas desatualizadas e alterações ou substituições da origem. Registre a duração, os recursos, a fila de eventos e o número real de delegates. Quando builds de três plataformas ou pacotes de desenvolvimento para Apple Silicon forem verificados sob demanda, registre os resultados separadamente.

Filtre as execuções de CI do repositório pelo head_sha e pelo evento real. As verificações de rotina são executadas em pushes e PRs. As verificações completas do Foundation e do Qt são iniciadas manualmente sob demanda. Registre os resultados e as verificações pendentes na issue correspondente. Uma lista vazia para outro evento não indica a situação da verificação.
