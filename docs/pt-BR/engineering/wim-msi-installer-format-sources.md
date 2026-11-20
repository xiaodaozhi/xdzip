<a name="xdzip-wim-msi-and-installer-format-sources-and-licenses"></a>

# Fontes e licenças de WIM, MSI e formatos de instaladores

[English](../../engineering/wim-msi-installer-format-sources.md) | [简体中文](../../zh-CN/engineering/wim-msi-installer-format-sources.md) | [繁體中文](../../zh-TW/engineering/wim-msi-installer-format-sources.md) | [日本語](../../ja-JP/engineering/wim-msi-installer-format-sources.md) | [한국어](../../ko-KR/engineering/wim-msi-installer-format-sources.md) | [Français](../../fr-FR/engineering/wim-msi-installer-format-sources.md) | [Deutsch](../../de-DE/engineering/wim-msi-installer-format-sources.md) | [Español](../../es-ES/engineering/wim-msi-installer-format-sources.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/wim-msi-installer-format-sources.md)

Esta página descreve referências técnicas, restrições de licença e requisitos para oferecer suporte a WIM, MSI e formatos de instaladores. As referências permanecem em `build/toolchain/references` ou apontam para especificações públicas; nenhuma é dependência do produto.

<a name="xdzip-wim"></a>

## WIM

- Implementação de referência: `ebiggers/wimlib`, commit `cd5e231c348c255ae5088873b5a66ee0eb96fa07`.
- O repositório oferece GPLv3+ para o projeto e uma opção LGPL-2.1+ condicionada para a biblioteca. A opção LGPL depende de que componentes de terceiros, como `libntfs-3g`, não imponham os termos da GPL.
- WIM é um contêiner de imagens, não um arquivo compactado comum. Um leitor sujeito a limites precisa tratar tabelas de recursos, fluxos de metadados, recursos em blocos, compressão Xpress/LZX/LZMS, tabelas de integridade, metadados de links físicos e seleção de imagem, sem montar a imagem nem restaurar permissões de segurança.

A implementação de referência é útil para pesquisar o formato e as amostras, mas não pode ser vinculada ao xdzip sob a política atual que exclui dependências de execução de terceiros. WIM está **excluído por decisão do produto**: o projeto não assumirá o leitor específico para imagens, o modelo de amplificação de descompressão, os recursos de integridade, a semântica de links físicos e o conjunto completo de amostras necessários para suportá-lo. Nenhuma entrada de registro, MIME, ambiente de execução ou CLI é permitida sem uma revisão explícita dessa decisão.

<a name="xdzip-compound-file-and-msi"></a>

## Compound File e MSI

- Microsoft [MS-CFB] é a principal especificação pública: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b . A página publica revisões em PDF/DOCX e um aviso de propriedade intelectual que permite implementar a tecnologia descrita; as licenças de patentes continuam sujeitas aos programas aplicáveis da Microsoft.
- Antes de interpretar dados específicos de MSI, um leitor CFB precisa tratar a alocação de setores e FAT/DIFAT, a travessia da árvore rubro-negra de diretórios, a alocação de mini-streams, os limites de tamanho dos fluxos e a detecção de ciclos.

Um leitor independente de metadados CFB é tecnicamente viável e já foi implementado na parte somente leitura `xdzip-codec-cfb`. Antes de expor linhas de armazenamentos e fluxos, ele valida os cabeçalhos CFB 3.0/4.0, incluindo as bases de setor de 512 e 4096 bytes, cadeias FAT/DIFAT, árvore de diretórios, alocação de mini-streams, limites de tamanho dos fluxos e ciclos. A cópia de fluxos brutos só está disponível após a correspondência com o retrato imutável dos metadados. Tabelas MSI, cabinets incorporados, ações personalizadas e transformações permanecem fora do contrato: não há execução do instalador, interpretação de ações personalizadas, instalação de pacotes ou gravação de volta. A extensão `.msi` não concede autorização de execução. As amostras e as evidências de execução são automatizadas nos testes do codec e do ambiente de execução.

<a name="xdzip-nsis-and-other-sfx"></a>

## NSIS e outros SFX

- Código de referência do NSIS: `kichik/nsis`, commit `a96303eae3115221471bb127fa916a13d8bf9ecd`.
- Seu arquivo `COPYRIGHT` registra zlib/libpng para a maior parte do código e o compressor zlib, bzip2 para o módulo bzip2 e Common Public License 1.0 para o módulo LZMA.
- Arquivos NSIS são instaladores executáveis com scripts e conteúdo incorporado. Um analisador deve permanecer fora do limite de execução, limitar os intervalos de cabeçalho e conteúdo e manter os métodos de compressão separados do comportamento do instalador.
- O código público define o registro fixo `firstheader`: `flags`, `0xDEADBEEF`, as palavras de assinatura `NullsoftInstallSystem`, `length_of_header` e `length_of_all_following_data`. Um leitor estático pode localizar e validar esse registro sem abrir o programa inicial PE, mas ainda precisa de um modelo independente dos blocos de cabeçalho compactados e de instaladores próprios completos.

NSIS está **admitido apenas para metadados estáticos**. O codec examina somente registros alinhados a 512 bytes em uma área adicional PE limitada, aceita os dois valores mágicos `firstheader` registrados, limita sinalizadores e comprimentos e exige que o intervalo posterior declarado caiba na entrada física. Em seguida, publica uma linha de metadados `NSIS/firstheader`. Ele não descompacta o cabeçalho do instalador nem expõe bytes do conteúdo. A entrada de registro `.exe` intencionalmente não é candidata a arrastar e soltar nem a associação do sistema; a detecção pelo conteúdo é determinante. O analisador é independente do Qt e das APIs de execução de instaladores. Ele nunca executa o programa inicial PE, script, página personalizada, plug-in ou conteúdo extraído. Outros formatos SFX só permanecem como aliases quando o arquivo compactado interno passa na validação de conteúdo.

<a name="xdzip-admission-gates"></a>

## Requisitos de admissão

1. Fixar as revisões de código e os hashes dos textos de licença antes de usar uma ferramenta de referência.
2. Criar amostras completas e redistribuíveis para cada contêiner e método de compressão admitido; uma amostra apenas com cabeçalho é insuficiente.
3. Manter a análise de CFB/WIM/NSIS independente do Qt e das APIs de instalação, montagem, registro ou execução de scripts.
4. Adicionar entradas de ambiente de execução, CLI e desktop somente após passar nos testes de identidade física exata, entradas duplicadas, limites de ciclos, limites cumulativos de entrada, trabalho e saída, cancelamento, alteração da origem e falha terminal.
5. Manter os componentes de referência GPL/LGPL/CPL em build/toolchain, a menos que uma decisão separada de dependência e licença altere explicitamente a política do produto.
