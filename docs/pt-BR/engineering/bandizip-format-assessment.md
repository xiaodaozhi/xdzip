<a name="xdzip-bandizip-format-inventory-and-xdzip-feasibility"></a>

# Inventário de formatos do Bandizip e viabilidade no xdzip

[English](../../engineering/bandizip-format-assessment.md) | [简体中文](../../zh-CN/engineering/bandizip-format-assessment.md) | [繁體中文（台灣）](../../zh-TW/engineering/bandizip-format-assessment.md) | [日本語](../../ja-JP/engineering/bandizip-format-assessment.md) | [한국어](../../ko-KR/engineering/bandizip-format-assessment.md) | [Français](../../fr-FR/engineering/bandizip-format-assessment.md) | [Deutsch](../../de-DE/engineering/bandizip-format-assessment.md) | [Español](../../es-ES/engineering/bandizip-format-assessment.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/bandizip-format-assessment.md)

Esta avaliação compara os formatos listados pelo Bandizip ao escopo de implementação do xdzip e identifica recursos que exigem verificação de fontes, amostras e testes automatizados. O inventário vem da página do produto Bandizip e das páginas de ajuda de formatos, consultadas em 2026-11-15:

- A [página do produto Bandizip](https://www.bandisoft.com/bandizip/) lista: `7Z`, `ACE`, `AES`, `ALZ`, `ARJ`, `BH`, `BIN`, `BR`, `BZ`, `BZ2`, `CAB`, `Compound(MSI)`, `DAA(1.0)`, `DEB`, `EGG`, `GZ`, `IMG`, `ISO`, `ISZ`, `LHA`, `LZ`, `LZH`, `LZMA`, `PMA`, `RAR`, `RAR5`, `SFX(EXE)`, `TAR`, `TBZ/TBZ2`, `TGZ`, `TLZ`, `TXZ`, `UDF`, `WIM`, `XPI`, `XZ`, `Z`, `ZIP`, `ZIPX`, `ZPAQ`, `PEA`, `UU`, `UUE`, `XXE`, `ASAR`, `ZSTD`, `NSIS` e `LZ4`.
- A mesma página informa que o Bandizip cria `ZIP`, `7Z`, `ZIPX`, `EXE (SFX)`, `TAR`, `TGZ`, `LZH`, `ISO`, `GZ` e `XZ`; a lista mais longa é principalmente de extração.
- A [ajuda do Bandizip sobre ALZ e EGG](https://www.bandisoft.com/bandizip/help/how-to-open-alz-and-egg-files/) descreve Deflate, Deflate ofuscado, BZip2 e ZipCrypto de ALZ, e Deflate, BZip2, LZMA, AZO, criptografia AES/LEA, blocos sólidos e arquivos divididos de EGG.
- A [ajuda do Bandizip sobre ZPAQ](https://www.bandisoft.com/bandizip/help/what-is-a-zpaq/) descreve ZPAQ como formato de arquivo com diário próprio e lista os métodos de compressão que o Bandizip consegue processar.
- O [guia de fontes de imagens de disco e outros formatos de arquivo](disk-image-archive-format-sources.md) registra as verificações de fontes públicas, licenças, corpus e sala limpa para DAA, ISZ, ZPAQ, PEA, BOND e J2J. BOND e J2J não estão confirmados como formatos oficiais atuais de arquivos do Bandizip e permanecem apenas como pistas históricas.

<a name="xdzip-current-xdzip-disposition"></a>

## Situação atual no xdzip

A tabela usa estes significados. **Admitido** significa que o registro atual, runtime, CLI e percurso no desktop têm evidências automatizadas para o perfil indicado. **Alias/perfil** significa que a extensão é admitida por um contêiner existente somente após validação do conteúdo. **Candidato** significa que o sufixo pode ser uma entrada futura útil, mas nenhum decodificador é afirmado. **Bloqueado** significa que o formato não pode ser admitido responsavelmente antes de existir uma especificação pública do formato, testável de forma independente, e uma implementação compatível ou algoritmo próprio. **Excluído** significa que o projeto decidiu explicitamente não dar suporte ao formato; ele não recebe registro nem detector até que essa decisão de produto seja revista.

| Nome no Bandizip | Status no xdzip | Escopo atual defensável ou motivo |
| --- | --- | --- |
| 7Z | Admitido | Backend 7z independente em Rust, incluindo os perfis declarados de senha, sólido e volumes divididos. |
| ACE | Excluído | Decisão de produto: abandonar ACE permanentemente; nenhum decodificador, detector, sufixo, MIME ou associação será adicionado. |
| AES | Admitido | AES Crypt v0–v3, perfil de senha e HMAC; é o envelope `.aes`, não arquivos criptografados arbitrários. |
| ALZ | Admitido | Store e Deflate bruto comuns de volume único e somente leitura, com tamanhos exatos e CRC-32; BZip2 modificado, ZipCrypto e volumes divididos ficam fora do perfil. |
| ARJ | Admitido | Store e métodos 1–4 somente leitura com cabeçalhos completos, tamanhos e CRC-32 dos membros. |
| BH | Admitido | Membros comuns `BH 05 07` somente leitura, método 8 Deflate bruto, tamanhos exatos e CRC-32. Store, criptografia, volumes e outros métodos são excluídos. |
| BIN | Candidato | Extensão ambígua entre disco, executável e dados. Detectar apenas um formato interno validado; nunca registrar BIN genérico como decodificador de arquivos compactados. |
| BR | Admitido | Fluxo Brotli limitado e perfis TAR.BR. |
| BZ / BZ2 | Admitido | Fluxos BZip2 de arquivo único e aliases TAR.BZ2 declarados. |
| CAB | Admitido | Backend CAB independente em Rust com Store, MSZip, Quantum, LZX e conjuntos de gabinetes autorizados. |
| Compound(MSI) | Admitido | Estrutura CFB, nomes de armazenamento/fluxo e cópia limitada de fluxos brutos somente leitura para conteúdo `.msi` validado. Tabelas MSI, interpretação de CAB incorporado, transformações, ações personalizadas e instalação ficam fora da capacidade. |
| DAA(1.0) | Excluído | Permanentemente sem suporte por decisão de produto; nenhum detector, codec, registro, leitor ou gravador é admitido. |
| DEB | Admitido | Envelope `ar` comum, visualizações control/data e membros compactados admitidos. |
| EGG | Admitido | Store e Deflate bruto comuns de volume único, não criptografados, não sólidos e somente leitura, com tamanhos exatos de bloco e CRC-32; AZO, BZip2, LZMA, criptografia, divisão e outros perfis ficam fora. |
| GZ | Admitido | Membros GZIP, incluindo fluxos concatenados e TAR.GZ. |
| IMG | Candidato | Sufixo ambíguo de imagem de disco. Admissão de ISO/UDF pelo conteúdo não implica suporte genérico a IMG. |
| ISO | Admitido | Leitura ISO9660/Joliet/Rock Ridge e criação de imagens de dados comuns ISO9660 + Joliet. |
| ISZ | Excluído | Decisão de produto: abandonar ISZ; nenhum detector, registro ou tratamento de chaves será adicionado. |
| LHA / LZH | Admitido | Níveis LHA 0–3, Store, LH1–LH7, LZS/LZ5 e criação LH5. |
| LZ | Admitido | Lzip v1 e TAR.LZ. |
| LZMA | Admitido | LZMA Alone e TAR.LZMA. |
| PMA | Admitido | Perfis PM0/PM1/PM2 somente leitura com estrutura CP/M e verificações CRC. |
| RAR / RAR5 | Admitido | Backend RAR independente em Rust, incluindo formatos antigos, RAR5, extensões RAR7, senhas, perfis sólidos e divididos dentro do escopo declarado. |
| SFX(EXE) | Alias/perfil | Prefixos ZIP SFX são validados pelo leitor ZIP. Extração ou execução genérica de executáveis nunca é autorizada. A criação ZIP Store/Deflate própria para Windows x64 está disponível pelo perfil explícito `.sfx.exe`, sem execução automática. |
| TAR | Admitido | TAR e perfis filtrados TAR.GZ/TAR.BZ2/TAR.LZMA/TAR.LZ/TAR.XZ/TAR.Z/TAR.BR/TAR.LZ4/TAR.ZST conforme o registro. |
| TBZ / TBZ2 | Alias/perfil | Aliases TAR filtrados por BZip2. |
| TGZ | Alias/perfil | Alias TAR filtrado por GZIP. |
| TLZ | Alias/perfil | Alias TAR filtrado por LZMA. |
| TXZ | Alias/perfil | Alias TAR filtrado por XZ. |
| UDF | Admitido | Imagens UDF somente leitura, incluindo regras de namespace para imagens híbridas. |
| WIM | Excluído | Decisão de produto: não seguir com leitor, modelo de recursos e corpus específicos de imagens necessários para WIM. |
| XPI | Alias/perfil | Conteúdo ZIP sob sufixo XPI; a validação do conteúdo permanece determinante. |
| XZ | Admitido | XZ e TAR.XZ com LZMA2, verificações declaradas e filtros admitidos. |
| Z | Admitido | LZW do Unix compress e aliases TAR.Z/TAZ. |
| ZIP | Admitido | ZIP/ZIP64, AES-256 e perfis compatíveis de volumes, métodos e regravação. |
| ZIPX | Alias/perfil | ZIPX é admitido para métodos ZIP validados como XZ; não é uma implementação de contêiner separada. A criação ZIPX não está disponível atualmente. |
| ZPAQ | Excluído | Decisão de produto: não implementar a máquina virtual ZPAQL necessária para esse formato sólido com diário. |
| PEA | Admitido | Perfil somente leitura PEA 1.x de volume único, UTF-8, sem criptografia, com arquivos e diretórios armazenados PCOMPRESS0; verificações, criptografia, compressão, volumes, criação e alteração ficam fora. |
| UU / UUE | Admitido | Envelopes UU/UUE limitados com validação de membros físicos e linhas. |
| XXE | Admitido | Perfil de envelope XXE limitado. |
| ASAR | Admitido | Metadados ordenados, intervalos armazenados e verificações SHA-256 declaradas. |
| ZSTD | Admitido | Quadros Zstandard padrão e perfis TAR.ZST/TZST. |
| NSIS | Admitido | Somente metadados estáticos `firstheader`. A detecção por conteúdo valida um overlay PE limitado; `.exe` não é entrada de arrastar e soltar ou associação shell, e não é permitida execução de PE, scripts, plugins ou dados. |
| LZ4 | Admitido | Quadros LZ4 padrão e TAR.LZ4. |

Portanto, o inventário não justifica adicionar processadores genéricos para `BIN`, `IMG` ou `SFX(EXE)`. Esses nomes podem descrever dados executáveis, de disco ou de instalador, e um sufixo sozinho não estabelece capacidade de arquivo compactado. NSIS tem um leitor mais restrito de metadados estáticos validados pelo conteúdo; ele nunca concede autoridade de execução ou sobre os dados e sua entrada `.exe` fica oculta em arrastar e soltar e associações shell. Compound(MSI) agora tem um leitor CFB validado pelo conteúdo; o sufixo `.msi` continua sem conceder autoridade de execução.

Consulte as [fontes de imagens de disco e formatos de arquivo](disk-image-archive-format-sources.md) para DAA, ISZ, ZPAQ, PEA, BOND e J2J. PEA está disponível no perfil conservador somente leitura documentado. ACE, DAA, ISZ e ZPAQ são excluídos por decisão de produto. BOND e J2J são excluídos porque sua identidade como formatos de arquivo Bandizip não foi confirmada.

<a name="xdzip-delivery-plan"></a>

## Plano de entrega

1. **Controle de inventário e registro.** Manter uma entrada legível por máquina por perfil admitido em `specs/engineering/archive-formats.json`. Gerar consumidores Rust, C++, JavaScript e de associações e executar ambas as verificações do registro. Cada novo formato recebe contrato e parágrafo de aceitação correspondente.
2. **Contrato do formato antes do código.** Fixar revisões de fontes públicas e SHA-256; documentar campos de cabeçalho, codificação de nomes, término físico, métodos, somas de verificação, identidade de duplicatas e limites de recursos. Uma ferramenta de referência só é evidência quando produz amostras próprias completas; falha ou uma única amostra Store é registrada como limitação.
3. **Testes de algoritmo bruto.** Implementar o leitor limitado independentemente do Qt. Cobrir dados vazios e de limite, cada método admitido, orçamentos exatos de entrada/saída/trabalho/memória, cancelamento, E/S tipada, alteração de origem, truncamento, bytes finais, CRC e estado terminal após erro. Métodos desconhecidos permanecem visíveis sem autoridade sobre os dados.
4. **Admissão do contêiner e produto.** Adicionar crate de codec, detecção fraca/forte no runtime, list/test/extract da CLI, rótulos de desktop e amostras de associação. Verificar metadados congelados e seleção de duplicatas físicas e executar as verificações automáticas relevantes de Qt e layout. Não é necessária aceitação visual manual para esta parte de pesquisa e formatos.
5. **Fila de prioridades.** Estão admitidos o subconjunto conservador ALZ Store/Deflate bruto, EGG sem criptografia e não sólido Store/Deflate bruto, metadados CFB/Compound(MSI), perfil conservador armazenado PEA e metadados estáticos NSIS `firstheader`. ACE, DAA, WIM, ISZ, ZPAQ, BOND e J2J são permanentemente excluídos por decisão de produto. A criação ZIPX continua fora dos gravadores atuais. O [gravador SFX próprio](sfx-creation.md) é implementado separadamente e não decorre do suporte ao alias em leitura.
6. **Limite de publicação.** Manter formatos candidatos fora da tabela pública de capacidades até que as evidências de codec, runtime, CLI e desktop estejam completas. Registrar métodos sem suporte e limitações de ferramentas de referência na issue, sem retornar silenciosamente a Store ou a outro contêiner.

O subconjunto BH atual segue esse plano: somente leitura, sem caminho de senha ou volumes, com leitor Deflate limitado independente, CRC-32 obrigatório e término físico exato. O subconjunto Compound(MSI) segue o mesmo limite: validação CFB FAT/DIFAT, árvore de diretórios, mini-stream e ciclos independente do Qt, e cópia de fluxos brutos somente após a correspondência do instantâneo imutável de metadados. O subconjunto NSIS para no registro fixo de metadados firstheader e retorna não suportado para acesso aos dados. Nenhum comportamento de instalador MSI ou NSIS é executado.
