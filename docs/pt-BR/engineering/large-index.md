<a name="xdzip-large-archive-indexing-and-on-demand-browsing"></a>

# Indexação de arquivos compactados grandes e navegação sob demanda

[English](../../engineering/large-index.md) | [简体中文](../../zh-CN/engineering/large-index.md) | [繁體中文](../../zh-TW/engineering/large-index.md) | [日本語](../../ja-JP/engineering/large-index.md) | [한국어](../../ko-KR/engineering/large-index.md) | [Français](../../fr-FR/engineering/large-index.md) | [Deutsch](../../de-DE/engineering/large-index.md) | [Español](../../es-ES/engineering/large-index.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/large-index.md)

O limite de capacidade do código para ZIP/ZIP64 comum de arquivo único é de um milhão de registros físicos. Outros formatos e caminhos com volumes divididos mantêm as admissões existentes de 100.000 entradas; o nome do formato não significa que todas as combinações tenham limites ampliados. Diretórios, nomes duplicados e registros que exigem revisão contam como entradas físicas; diretórios implícitos de navegação acrescentam apenas linhas de exibição.

<a name="xdzip-scanning-and-storage"></a>

## Varredura e armazenamento

ZIP examina cada registro com o analisador ZIP comum. Primeiro conclui uma amostragem limitada de nomes em codificação antiga em todo o arquivo e depois executa uma segunda passagem com a mesma decisão de codificação. As duas passagens leem juntas no máximo 512 MiB; o diretório central tem no máximo 256 MiB, os nomes brutos somam no máximo 128 MiB e os nomes decodificados e exibidos juntos no máximo 128 MiB. Cada nome bruto ou decodificado continua limitado a 4 KiB. Cauda danificada, cancelamento ou mudança da fonte causa falha de todo o resultado sem publicar índice parcial.

Ao atingir 64 MiB de metadados ativos ou ultrapassar 100.000 entradas, migrar os registros individualmente para armazenamento privado em disco e liberar nomes e registros existentes sem manter dois índices completos ao mesmo tempo. A tabela de posições físicas contém no máximo um milhão de deslocamentos u64. Os metadados em disco ocupam no máximo 512 MiB; nomes e conteúdos de arquivos não entram em mensagens de diagnóstico. A árvore de diretórios continua limitada a 32.768 nós, 64 níveis e 4 MiB de nomes. Se exceder o limite, não publicar a árvore; navegação na raiz e busca física continuam disponíveis.

Arquivos temporários ficam em um diretório `xdzip-index-*` de nome aleatório sob o diretório temporário do hospedeiro, legível e gravável apenas pelo usuário atual e operado por identificadores mantidos de diretório e arquivo comum. Fechamento normal, substituição da consulta, falha e cancelamento limpam arquivos conhecidos e diretórios vazios com identidade verificada. Restos de falhas são identificados por marcador de 16 bytes correspondente à identidade aleatória do diretório e por uma concessão exclusiva. Uma criação posterior faz varreduras limitadas e limpa apenas arquivos conhecidos sem uso há mais de sete dias. Links, identidades divergentes e objetos desconhecidos são preservados. Não excluir recursivamente conteúdo desconhecido nem restaurar ou reutilizar índices entre execuções.

No Windows, o diretório recém-criado deve passar primeiro pelas verificações comuns de proprietário e permissão de modificação antes de proteger sua DACL para armazenamento confidencial. Permissões herdadas de modificação para entidades não relacionadas causam rejeição antes da gravação da concessão ou dos metadados; o runtime não usa silenciosamente um local menos privado. Para verificação isolada, apontar apenas `TMP` e `TEMP` do processo de teste para um diretório próprio novo cuja ACL permita modificação somente ao usuário atual, SYSTEM e Administrators. Não alterar ACLs dos diretórios reais de usuários nem afrouxar verificações de permissão. Verificar separadamente alocação, consultas e mesclagem reais em disco, concessões ativas, limpeza de arquivos conhecidos e rejeição de permissões de modificação de entidades não relacionadas.

<a name="xdzip-queries-and-selection"></a>

## Consultas e seleção

Busca e ordenação produzem uma ordem completa e imutável de linhas. As chaves de ordenação são gravadas em sequências privadas de mesclagem de no máximo 8 MiB cada, com no máximo 128 sequências e 512 MiB de armazenamento temporário de ordenação. Consultas finais armazenam apenas tokens de linha, sem outra cópia completa dos nomes. As comparações preservam os inteiros originais, regras existentes de nome e tipo e identidade física estável. Todo modo de ordenação coloca diretórios explícitos e implícitos antes dos arquivos; a direção vale dentro de cada grupo. Tamanhos desconhecidos ficam após os conhecidos no respectivo grupo; a ordem decrescente não altera a ordem de identidade dos duplicados. O cancelamento é verificado durante a varredura e em torno das sequências e mesclagem, sem alterar as relações de ordem no comparador.

A ponte transfere no máximo 128 linhas e 64 KiB de nomes por lote. Consultas de até 1.024 linhas mantêm o método existente de confirmação completa. Modelos Qt maiores mantêm a contagem lógica total e carregam apenas posições visíveis, com no máximo 1.024 linhas residentes em cache. Linhas não carregadas não podem enviar ações de entrada. Cada página adiada é vinculada à sessão, consulta, geração, revisão e deslocamento solicitado; páginas obsoletas não podem entrar em novos resultados.

A seleção entre páginas usa até 4.096 intervalos limitados na consulta atual imutável. Selecionar tudo cobre toda a consulta atual; sair do cache não altera a seleção. Antes da confirmação da extração, Rust congela os intervalos em identidades físicas ou de navegação, mantidas mesmo se a consulta mudar. Seleções acima do limite de 100.000 entradas da tarefa são rejeitadas explicitamente, não reduzidas em silêncio. O tamanho total de seleções não residentes ainda não é mostrado. Uma cópia em massa para a área de transferência além do conteúdo residente retorna limite de recursos em vez de copiar silenciosamente uma lista incompleta.

<a name="xdzip-task-boundaries"></a>

## Limites das tarefas

Arquivos comuns selecionados de um índice em disco podem ser extraídos após revalidação completa da fonte, planejamento seguro, verificação da entrada inteira e confirmação autorizada existente. Um plano congelado continua limitado a 100.000 entradas e 64 MiB de metadados; os limites de arquivos e bytes efetivamente decodificados seguem o [contrato de extração](../../engineering/zip-safe-extraction.md). A expansão de diretórios não pode ignorar nomes não resolvidos; links selecionados são rejeitados nessa configuração. O leitor ZIP verifica novamente os deslocamentos físicos de todo o diretório central, mantém apenas as posições dos dados selecionados e preserva as verificações de sobreposição, cabeçalho local, tamanho, CRC, autenticação e cancelamento.

Arquivos compactados que usam índice em disco ainda não suportam modificação ou teste do arquivo inteiro; uma projeção vazia em memória não pode ser tratada como arquivo completo. Os limites existentes para criação, outros formatos, volumes divididos e regravação permanecem iguais. Consulte separadamente a [visualização de conteúdo](content-preview.md). A listagem física e a seleção CLI reutilizam o mesmo índice; a [paginação explícita](../../engineering/command-line.md#archive-behavior) obtém toda a faixa física dos arquivos grandes. Respostas sem paginação mantêm os limites de 100.000 entradas e 16 MiB de dados serializados ou texto e falham por completo quando excedidos.

<a name="xdzip-verification"></a>

## Verificação

`scripts/probe-large-index.py` gera um ZIP64 real em `build/` com nomes chineses, duplicados e diretórios aninhados e chama o `xdzip-m2-probe` real. A sonda lê cada lote de consulta e registra separadamente varredura, ordenação, recuperação, bytes em disco e métricas de memória disponíveis na plataforma. O gerador de amostras não constrói na memória uma matriz de um milhão de nomes. Escopo, falha, cancelamento, limpeza, páginas obsoletas e saída real dos arquivos selecionados seguem os [requisitos de aceitação](../../../developers/qa/acceptance.md); resultados permanecem em issues, localmente ou em artefatos CI. As medições comprovam apenas a máquina e a carga reais, não garantem desempenho universal.
