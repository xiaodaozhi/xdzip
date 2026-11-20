<a name="xdzip-performance-design-and-measurement"></a>

# Projeto e medição de desempenho

[English](../../design/performance.md) | [简体中文](../../zh-CN/design/performance.md) | [繁體中文（台灣）](../../zh-TW/design/performance.md) | [日本語](../../ja-JP/design/performance.md) | [한국어](../../ko-KR/design/performance.md) | [Français](../../fr-FR/design/performance.md) | [Deutsch](../../de-DE/design/performance.md) | [Español](../../es-ES/design/performance.md) | Português (Brasil) | [Русский](../../ru-RU/design/performance.md)

O XDZip limita separadamente memória e trabalho de indexação, transporte, execução de tarefas e visualização de imagens. Os limites de código abaixo são regras de implementação. Tempos, medições de memória e vazão dependem da compilação e da carga. Os objetivos de referência adiante são metas de engenharia.

<a name="xdzip-limits-and-mechanisms-in-the-current-code"></a>

## Limites e mecanismos no código atual

| Área | Comportamento e custo atuais |
| --- | --- |
| ZIP/ZIP64 comum de arquivo único | O analisador em fluxo admite até um milhão de registros físicos, sujeito a orçamentos de nomes, diretórios e leitura. Quando metadados ativos excedem 64 MiB ou um novo registro excederia 100.000 entradas residentes, os registros vão para armazenamento privado em disco. O arquivo de metadados é limitado a 512 MiB, com tabela de posições separada. A navegação varre metadados em vez de decodificar todos os dados. |
| Consultas em memória e disco | O runtime constrói uma vez a ordem imutável do resultado atual. O caminho em disco descarrega sequências de ordenação de até 8 MiB e as mescla em ordem de tokens. Armazenamento de nomes e ordenação é limitado. O cache contém a consulta atual, não uma visualização integral renderizada ou histórico indefinido de consultas. |
| Navegação e transferência Qt | A árvore de diretórios tem orçamento de 32.768 nós, 64 níveis e 4 MiB de nomes. Se for excedido, o runtime pode manter navegação na raiz e busca. A ponte envia até 128 linhas e 64 KiB de nomes por página. Visualizações grandes mantêm até 1.024 linhas residentes e contagem lógica de todo o resultado. |
| Comandos, tarefas e gravações | As threads de metadados e tarefas usam filas limitadas separadas. Uma tarefa desktop pesada é admitida por vez. Trabalho de decodificação, bytes de saída, confirmação de arquivos e recursos de processos têm contabilizações distintas. Formatos sólidos e em fluxo podem ler ou decodificar dados anteriores antes do item selecionado. |
| Visualização e processos nativos | Texto e imagens codificadas têm limites de bytes separados. Dimensões e número de pixels, execução do processo e tamanho de miniaturas são limitados independentemente. Processos de formato têm limites próprios de protocolo e processo. |

[Índice em disco](../../../crates/xdzip-runtime/src/disk_index.rs), [Índice de arquivos grandes](../engineering/large-index.md), [Limites da ponte Qt/Rust](qt-rust-bridge.md), [Admissão de tarefas](task-scheduling.md), [Orçamentos de saída](file-security.md), [Orçamentos de visualização](../engineering/content-preview.md)

O custo de memória inclui Qt, fontes, renderização e modelos, metadados, tabela de posições e buffers de consulta, estado de decodificadores, processos filhos e buffers de E/S. O gatilho de migração de 64 MiB é um limite de metadados. A memória da árvore de processos também inclui tudo fora desse índice. Maior admissão de metadados não amplia cotas de extração nem habilita teste ou edição do arquivo inteiro em sessões com índice em disco. Escrita do índice privado e preparação de extração têm custos de disco separados.

<a name="xdzip-measuring-the-current-implementation"></a>

## Medir a implementação atual

| Entrada | O que mede ou verifica |
| --- | --- |
| `scripts/probe-large-index.py` e `xdzip-m2-probe` | Geram um ZIP64 próprio, exercitam o analisador real, ordenação e consultas paginadas e relatam fases e armazenamento separadamente. Incluir duplicados, nomes longos e estrutura de diretórios e identificar se o caminho em disco foi alcançado. |
| `scripts/measure-desktop.py` e `xdzip-process-metrics` | Medem árvore de processos desktop, inicialização e navegação. Com opção de extração, executam extração real e verificam independentemente os bytes de saída. Registrar renderizador, identidade da compilação e carga exata. |
| `scripts/run-performance-suite.py` | Coleta observações Release de inicialização, ociosidade estável e navegação e compara extração com uma sonda de núcleo fixada em dados próprios. O relatório registra identidades da compilação e dados. A comparação é observacional; a aprovação da máquina de referência é condição separada. |
| `scripts/check-runtime-resources.py` | Verifica o recibo de recursos e fontes de runtime da sonda. Ele comprova identidade e cobertura para aquela sonda sem medir latência interativa. |

[Sonda de índice](../../../scripts/probe-large-index.py), [Medição desktop](../../../scripts/measure-desktop.py), [Suíte de desempenho](../../../scripts/run-performance-suite.py), [Sonda de recursos](../../../scripts/check-runtime-resources.py), [Coleta de evidências](../../../developers/engineering/candidate-verification.md#performance-and-resource-evidence)

Para comparar, registrar commit e configuração, sistema e arquitetura, renderizador, hardware, número de execuções, dados de teste, condições de dispositivo e cache. Separar inícios frios e repetidos. Quando a medição permitir, relatar separadamente primeira interação, varredura do arquivo, ordenação da consulta, transferência da página e renderização visível. Distinguir resposta ao cancelamento de parada e limpeza reais. Separar bytes de entrada, decodificados e confirmados ao relatar vazão.

A ferramenta da árvore de processos identifica separadamente PSS no Linux, espaço físico no macOS e comprometimento privado no Windows, e também informa RSS ou conjunto de trabalho quando disponível. Comparar a mesma métrica em condições comparáveis. Relatar separadamente tamanhos de download e de instalação de pacotes reais. Tempos fora de tela e execuções de protótipo descrevem apenas suas próprias condições.

<a name="xdzip-workloads-for-comparative-tests"></a>

## Cargas para testes comparativos

| Carga | Medições |
| --- | --- |
| B01/B08 Janela vazia, ociosidade e mudança de tema | Tempos de inicialização e quadros, CPU ociosa, memória retida e renderizador. |
| B02 100.000 e um milhão de registros ZIP | Varredura de metadados, migração, ordenação, obtenção de páginas e custos separados de memória e disco. |
| B03/B04 Um arquivo comum grande ou muitos pequenos | Vazão, cancelamento, trabalho no destino, sincronização e limpeza. Operações de arquivo podem dominar nos arquivos pequenos. |
| B05/B06 Entradas muito comprimidas ou profundas no fluxo | Tratamento de cota e trabalho adicional de decodificação antes da entrada selecionada. |
| B07 Trabalho pesado pendente durante a navegação | Latência de busca e páginas enquanto uma operação pesada executa e outra espera. |
| B09 Processos incluídos e pacote completo | Uso da árvore de processos, custo do protocolo, tamanho baixado e instalado. |

São cenários comparativos. Escolher a carga adequada à mudança e usar dados de teste próprios com tamanhos e hashes registrados.

<a name="xdzip-reference-targets-and-planned-research"></a>

## Objetivos de referência e pesquisa planejada

O projeto existente usa uma referência Apple Silicon com 16 GiB de memória, SSD local, tela de 60 Hz, fonte do sistema, GPU real e compilação Release. Os objetivos são candidatos para medições comparáveis. Eles não afirmam resultados da versão mais recente ou de outra plataforma.

| Cenário | Meta de projeto |
| --- | --- |
| Inicialização repetida de janela vazia | P95 de no máximo 500 ms até um quadro interativo, com pelo menos 30 execuções |
| Abrir e consultar 100.000 registros ZIP em cache | P95 de no máximo 1 s para preparar metadados e 80 ms da consulta indexada até ficar visível |
| Salto de página em cache e escalonamento GUI | P95 de no máximo 50 ms para salto visível e 8 ms de bloqueio contínuo da thread GUI |
| Memória | Meta para árvore de processos de até 150 MiB vazia e 256 MiB ao navegar em 100.000 entradas sem visualização ou dicionário de decodificação |
| CPU ociosa | Média de no máximo 0,5% de um núcleo lógico após estabilização |
| Cancelamento | P95 de no máximo 50 ms para resposta e 250 ms para parada cooperativa local em fluxo, medidos separadamente |
| Extração e distribuição | Comparar GUI e núcleo com carga e E/S idênticas, com meta de perda de vazão de até 10%. Metas Mac de arquitetura única são no máximo 100 MiB baixados e 250 MiB instalados. |

Extração paralela por bloco, reutilização persistente de índices e escalonador compartilhado de tokens de CPU permanecem opções de projeto. Analisar a rota atual de tarefas seriais e os caminhos de memória e disco antes de mudar concorrência ou armazenamento. Manter verificação equivalente de bytes, cancelamento e publicação segura em qualquer comparação. [Evolução da arquitetura](architecture-evolution.md), [Publicação de arquivos](file-security.md)
