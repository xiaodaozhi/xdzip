<a name="xdzip-lz4-stream-contract"></a>

# Contrato de fluxos LZ4

[English](../../engineering/lz4-streams.md) | [简体中文](../../zh-CN/engineering/lz4-streams.md) | [繁體中文（台灣）](../../zh-TW/engineering/lz4-streams.md) | [日本語](../../ja-JP/engineering/lz4-streams.md) | [한국어](../../ko-KR/engineering/lz4-streams.md) | [Français](../../fr-FR/engineering/lz4-streams.md) | [Deutsch](../../de-DE/engineering/lz4-streams.md) | [Español](../../es-ES/engineering/lz4-streams.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/lz4-streams.md)

O perfil de quadros padrão segue as especificações do autor para o [formato de quadros 1.6.5](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Frame_format.md), o [formato de blocos de 2026-10-26](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Block_format.md) e o [XXH32](https://github.com/Cyan4973/xxHash/blob/v0.8.3/doc/xxhash_spec.md). Os algoritmos do produto são implementações independentes em Rust. As ferramentas de referência e as evidências de execução geradas ficam no `build/` local. Apenas padrões de entrada reproduzíveis do projeto são versionados. Veja [fluxos de um único arquivo](single-file-streams.md) para as permissões do host e a publicação.

<a name="xdzip-admission-and-framing"></a>

## Admissão e quadros

Quadros padrão da versão 1 validam bits reservados, valores BD de 4 a 7, o XXH32 do cabeçalho, o tamanho opcional de conteúdo, todos os XXH32 declarados de blocos e conteúdo e o marcador final. Os dados dos blocos respeitam o máximo declarado de 64 KiB a 4 MiB. Blocos independentes zeram o histórico. Blocos vinculados retêm somente os 64 KiB anteriores. Blocos sem compactação, inclusive de tamanho zero, passam pelas mesmas verificações e limites. Perfis com dicionário externo e quadros antigos ficam fora do escopo. Declarar um dicionário nunca autoriza ler outro arquivo.

Quadros padrão concatenados e quadros ignoráveis intercalados são limitados pela entrada física, pelas cotas da tarefa, pelo cancelamento e pelo máximo de 100.000 quadros ou blocos. Um arquivo autônomo válido contém ao menos um quadro de dados padrão. Dados finais arbitrários ou uma entrada composta só de quadros ignoráveis causam falha. A detecção examina até 8 MiB da entrada compactada real antes de produzir um prefixo de conteúdo de 512 bytes. Uma identificação inequívoca ou a detecção de TAR danificado impede a troca de formato após uma falha. A varredura completa valida todos os quadros antes de publicar o tamanho apurado e a entrada virtual.

<a name="xdzip-block-processing-and-creation"></a>

## Processamento de blocos e criação

Os comprimentos usam aritmética verificada. Uma correspondência exige deslocamento diferente de zero dentro do histórico. Os bytes sobrepostos são copiados em ordem crescente, dentro do limite de saída declarado. A última sequência contém apenas literais. Seu semibyte de correspondência não usado é ignorado. Um fluxo com correspondências conserva no mínimo cinco literais finais e sua última correspondência começa pelo menos doze bytes decodificados antes do fim do bloco. A leitura gera partes limitadas e não é retomada após uma falha. O histórico é transferido somente após validar o fim exato do bloco. E/S do host, cancelamento, estouro de cotas e perfis incompatíveis mantêm categorias de erro distintas.

Um gravador admitido deve gerar correspondências LZ4 reais, usar blocos brutos apenas quando compactar não reduz o tamanho e incluir somas de verificação de blocos e conteúdo. Os níveis limitam o trabalho de busca de correspondências e devem produzir conteúdo com vários blocos decodificável de forma independente. Buffers de quadros e correspondências são admitidos antes da alocação. A criação usa tamanhos de entrada exatos e congelados, saída temporária no host, verificação completa e conferência final da origem e da saída. LZ4 comum recebe um único arquivo de entrada comum. Uma composição TAR só pode ser declarada quando estiver coberta pelo leitor e pelo gravador TAR compartilhados e pelas verificações automatizadas correspondentes. Registrar uma extensão não habilita regravação, criptografia ou volumes em nenhum dos dois perfis.

A verificação usa `format_declared_checks` porque os quadros podem omitir somas de verificação do conteúdo. Um quadro sem soma de verificação não pode ter seu conteúdo declarado verificado. Limites de recursos, aceitação real de plataformas, validade de formato e autorização de publicação são critérios separados.
