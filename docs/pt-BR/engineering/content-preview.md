<a name="xdzip-content-preview"></a>

# Visualização de conteúdo

[English](../../engineering/content-preview.md) | [简体中文](../../zh-CN/engineering/content-preview.md) | [繁體中文（台灣）](../../zh-TW/engineering/content-preview.md) | [日本語](../../ja-JP/engineering/content-preview.md) | [한국어](../../ko-KR/engineering/content-preview.md) | [Français](../../fr-FR/engineering/content-preview.md) | [Deutsch](../../de-DE/engineering/content-preview.md) | [Español](../../es-ES/engineering/content-preview.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/content-preview.md)

Quando o cartão de propriedades está visível, selecionar um arquivo suportado inicia a visualização após a seleção se estabilizar. O ícone de atualização aparece no canto superior direito quando o ponteiro está sobre a visualização ou o botão recebe foco do teclado; ele recarrega o arquivo selecionado. O cartão informa falhas de leitura ou decodificação e mostra uma mensagem para seleção múltipla. A janela principal e o visualizador independente de propriedades compartilham esses comportamentos e a janela de senha existente. Os metadados permanecem disponíveis separadamente.

<a name="xdzip-capabilities-and-budgets"></a>

## Capacidades e orçamentos

Apenas arquivos regulares legíveis, com tamanho conhecido e sem nomes perigosos são suportados. A lista de extensões de texto permitidas é txt, md, log, json, csv, tsv, toml, yaml, yml, ini, xml, rs, c, cpp, h e py. Aceita-se apenas UTF-8 estrito ou UTF-16 com BOM. Entrada e UTF-8 convertido são limitados cada um a 256 KiB; bytes de controle binários como NUL são proibidos. O conteúdo aparece em um controle PlainText; tags, URLs, texto de programa e macros não são executados.

| Tipo de imagem | Escopo e comportamento real nas três plataformas |
| --- | --- |
| PNG, JPEG (jpg, jpeg, jfif, jpe) | Ler integralmente uma imagem estática e aplicar metadados de transformação disponíveis para orientação |
| GIF, WebP | Imagem estática ou primeiro quadro de animação; animações não são reproduzidas |
| BMP, ICO/CUR | Imagem comum ou imagem padrão do decodificador; CUR não ativa cursor do sistema |
| ICNS | Imagens PNG ou JPEG2000 incorporadas suportadas por decodificador de sistema disponível, até 256 blocos; tipos incorporados desconhecidos e ícones antigos de bitmap bruto/RLE são explicitamente rejeitados |
| TGA | Imagens true-color não comprimidas de 16/24/32 bits, sem mapa de cores nem ID de imagem, com origem à esquerda; completar rodapé legado ausente para o mesmo layout; sem RLE ou cor indexada |
| TIFF (tif, tiff) | Apenas primeira página; páginas não são percorridas automaticamente |
| SVG | Vetor estático UTF-8 rasterizado após análise restrita; veja o subconjunto abaixo |
| HEIC/HEIF | Apenas macOS, usando o plugin oficial Qt com ImageIO do sistema; explicitamente rejeitado nas demais plataformas |

AVIF, RAW, PSD, EXR, PDF, Office e bibliotecas independentes de decodificadores de terceiros não são adicionados. Extensão, verificações Rust de assinatura de formato e decodificadores Qt fixados determinam juntos a admissão. Plugins adicionais instalados no sistema não se tornam recursos do produto e outros decodificadores não são sondados automaticamente.

O conteúdo de imagem codificado é limitado a 16 MiB. As filas de eventos Rust e a caixa Qt têm limite de 32 MiB cada; o orçamento de comandos permanece 8 MiB. Largura e altura originais têm no máximo 16.384 cada, com no máximo 33.554.432 pixels no total. Admissão da fonte e miniatura de saída são contabilizadas separadamente: largura e altura finais de no máximo 512 cada, proporção preservada e RGBA8 de no máximo 1 MiB. Fotos como imagens de 12 MP podem ser lidas como miniaturas dentro dos orçamentos. Alguns decodificadores leem o original antes de reduzir, portanto o tamanho da miniatura não comprova uso interno de apenas 1 MiB. Perfis de cor disponíveis são convertidos para sRGB; perfis desconhecidos mantêm o resultado do decodificador. Não se promete prova profissional de cores.

O provedor assíncrono de imagens Qt recebe apenas tokens aleatórios, não caminhos de sistema nem URLs de rede. No máximo 32 MiB de conteúdo codificado são mantidos globalmente e até duas tarefas de decodificação executam em paralelo. Cada tarefa cancelada ainda em liberação pode manter no máximo uma entrada de 16 MiB. Cada tarefa inicia o processo privado adjacente fixado `xdzip-image-preview`, sem passar caminhos do arquivo compactado, senhas, identificadores de destino ou credenciais do aplicativo. A saída permite apenas um cabeçalho de tamanho fixo e RGBA8 com largura, altura e comprimento estritamente correspondentes, sem saída adicional. Falhas, quedas, quadros inválidos, tempo esgotado e resultados tardios não podem ser exibidos.

O host impõe prazo total de cinco segundos desde o início e verifica a validade da solicitação a cada 50 ms. Fechar, cancelar ou mudar de entrada encerra e recolhe o processo. O processo limita alocações QImage a 128 MiB. Windows usa um Job com 512 MiB de memória comprometida do processo e três segundos de CPU de usuário. Unix usa três segundos de CPU, desativa core dumps e limita descritores de arquivo. Linux limita espaço de endereços a 512 MiB e tamanho de arquivos regulares a zero. macOS acrescenta 512 MiB de espaço de endereços além das reservas virtuais dyld existentes; não é um limite rígido de RSS. O ImageIO do sistema pode usar recursos de runtime apoiados em arquivos, portanto cada arquivo regular é limitado a 16 MiB e excessos ainda encerram a decodificação. O aplicativo não exporta cópia de entrada do arquivo compactado; recursos do sistema e saída da entrada são gerenciados separadamente. Isso isola falhas e recursos de um decodificador privado, não é uma caixa de permissões do kernel para programas arbitrários de terceiros.

Cada leitura de conteúdo do arquivo compactado é limitada cumulativamente a 256 MiB de leituras da fonte, 64 MiB de trabalho de decodificação e salto e 256 MiB de trabalho algorítmico auxiliar. Dicionários, processos e grafos de métodos continuam sujeitos aos orçamentos separados do [contrato do backend de formatos](native-codecs.md). Um arquivo sólido ou fluxo comprimido sequencial pode recusar visualização de arquivo pequeno se a decodificação anterior exceder o orçamento de trabalho.

<a name="xdzip-static-svg-subset"></a>

## Subconjunto SVG estático

SVG usa o mesmo caminho de renderização Qt Svg nas três plataformas desktop. Antes da renderização, um analisador XML em fluxo reconstrói gráficos estáticos, texto, estilos, definições internas, gradientes, recortes, máscaras e filtros restritos suportados. Os limites são 8.192 elementos, 64 níveis, 64 atributos por elemento, 1 MiB por atributo e 16 MiB de saída total. Metadados não participam da renderização. Elementos de estilo aceitam apenas texto; combinar segmentos de texto XML e CDATA antes de verificar todo o CSS para que a segmentação não contorne restrições de referências externas.

Rejeitar DTDs, entidades, instruções de processamento, scripts, eventos, animação, foreignObject, elementos image, arquivos de fontes, estilos externos e referências a recursos externos. href e CSS url aceitam apenas identidades de fragmento simples no documento atual. xml:base, src, importações CSS e formas escapadas são proibidos. Bitmaps incorporados e SVGZ estão fora do subconjunto. Elementos de desenho desconhecidos ou conteúdo proibido fazem toda a visualização falhar em vez de serem descartados silenciosamente e apresentados como completos. Isso suporta SVG estático comum sem afirmar todas as capacidades SVG ou de navegador.

<a name="xdzip-integrity-passwords-and-lifecycle"></a>

## Integridade, senhas e ciclo de vida

A visualização reutiliza identidade física congelada, versão da fonte, localização dos dados e decodificadores existentes. O conteúdo deve ser totalmente decodificado até EOF e concluir as verificações disponíveis de integridade e autenticação do formato antes de texto ou imagens codificadas chegarem à visualização. Formatos como TAR sem somas de conteúdo por arquivo mantêm apenas as garantias existentes de estrutura e legibilidade. Índices grandes em disco usam leituras limitadas de dados selecionados sem carregar todos os metadados no Qt.

Senhas são vinculadas apenas à solicitação atual e usam o contêiner existente que zera os dados; erros, ambiguidade de autenticação e cancelamento seguem categorias existentes, com até dez tentativas. Fechar a janela de senha cancela apenas a visualização e mantém o diretório utilizável. Abrir, navegar e outros comandos de leitura em primeiro plano têm prioridade sobre uma leitura de visualização pendente. Fechar a visualização, mudar de entrada, ocultar uma página de arquivo compactado ou fechar uma sessão limpa o conteúdo; solicitações invalidadas e resultados de decodificação não podem restaurar conteúdo antigo. Falha e cancelamento descartam conteúdo parcial não verificado, não gravam cópia da entrada nem iniciam aplicativo do sistema. A abertura temporária por clique duplo segue seu [fluxo separado](archive-opening.md).

<a name="xdzip-toolchain-and-acceptance"></a>

## Ferramentas e aceitação

`desktop.py prepare --install-qt` adiciona qtimageformats do Qt 6.8.2 fixado a SDKs novos ou existentes; doctor verifica Svg e plugins necessários antes da compilação. Fontes oficiais, hashes, avisos e correspondência do SDK estão na configuração compartilhada. libwebp/libtiff incluídos pelo Qt e bibliotecas de formatos adicionadas separadamente são gerenciados independentemente. `check-image-preview.py` verifica o processo real, plugins e decodificação mínima em testes padrão, verificação de implantação Windows/Linux e empacotamento Mac autorizado. Mac inclui offscreen explicitamente e limpa substituições de busca do SDK antes da autoverificação; pastas de plugins do SDK de desenvolvimento não substituem testes do pacote de runtime real.

Regressões usam imagens próprias cobrindo cada tipo, primeiros quadros de animação, primeiras páginas TIFF, transparência, fotos de 12 MP, HEIC no macOS e rejeição em outras plataformas, definições internas SVG e rejeição de recursos externos, entrada danificada, excesso de pixels, quadros inválidos, tempo esgotado e cancelamento. Caminhos reais de arquivos compactados continuam cobrindo senhas, integridade, identidade da fonte e limpeza após invalidação da interface. O layout segue o [padrão visual](../visual/design-standard.md#xdzip-content-preview); reutilizar evidências de layout inalteradas conforme a [verificação](../../../developers/qa/README.md) e observar a exibição real das imagens adicionadas.
