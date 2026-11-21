<a name="xdzip-file-association-icons"></a>

# Ícones de associação de arquivos

[English](../../visual/file-association-icons.md) | [简体中文](../../zh-CN/visual/file-association-icons.md) | [繁體中文](../../zh-TW/visual/file-association-icons.md) | [日本語](../../ja-JP/visual/file-association-icons.md) | [한국어](../../ko-KR/visual/file-association-icons.md) | [Français](../../fr-FR/visual/file-association-icons.md) | [Deutsch](../../de-DE/visual/file-association-icons.md) | [Español](../../es-ES/visual/file-association-icons.md) | Português (Brasil) | [Русский](../../ru-RU/visual/file-association-icons.md)

Os ícones de arquivo do sistema reutilizam o documento de canto dobrado, selo de curvatura contínua, quatro faixas entrelaçadas e marca aberta em U do xdzip. A entrada de design é a [página de revisão de alta fidelidade](../../../design/hifi/sources/file-association-icons.html), os parâmetros vêm de [file-association-icons.json](../../../specs/visual/file-association-icons.json) e as fontes e contêineres de plataforma ficam em [file-associations](../../../design/assets/file-associations). Os usuários continuam escolhendo seus aplicativos padrão.

<a name="xdzip-shape-and-identification"></a>

## Forma e identificação

A geometria do papel e da marca reutiliza [brand_assets.py](../../../scripts/brand_assets.py). Os formatos compartilham papel, dobra, posição do selo e centro óptico; rótulos identificam formatos e cores auxiliam a visualização. TAR.GZ, TGZ e GZ formam uma família verde-azulada. A alternativa genérica usa cinza-azulado neutro e linhas de documento sem rótulo de formato.

Os rótulos seguem estritamente os glifos ZIP originais: altura 61, traço 14, extremidades quadradas, junções arredondadas, topo no Y=817 do original e cor #39465D. ZIP reutiliza diretamente o caminho completo do texto original; as outras letras seguem as mesmas medidas e regras de traço. Não são usados texto SVG, fontes do sistema, ícones de terceiros ou bitmaps incorporados. A largura máxima é 504. Preservar o rótulo TAR.GZ completo sem esticar glifos; apenas reduzir uniformemente e centralizar verticalmente quando exceder a largura disponível. A cor não é o único meio de identificação.

O original tem 1024 unidades. PNG/ICO genéricos mantêm o viewBox `32 32 960 960` com corpo de papel de aproximadamente 90% de altura; PNG/ICNS do macOS mantêm `-20 -20 1064 1064` com corpo de aproximadamente 81,2%. Isso continua a calibração óptica do projeto e não é apresentado como exigência do sistema. Os ícones usam telas transparentes sem papel específico de tema, fundo de desktop ou máscaras arredondadas adicionais.

Em 16 e 24 pixels, o selo é ampliado e o papel simplificado; em 32 pixels, o selo é um pouco menor. Esses três tamanhos omitem rótulos e sombras das imagens grandes, preservando a proporção original da marca. Nesses tamanhos, os ícones sozinhos não distinguem todos os formatos com segurança, especialmente a família GZIP da mesma cor; as colunas de nome e tipo permitem identificação precisa. Originais com rótulo são usados a partir de 48 pixels; rótulos longos são mais legíveis a partir de 64 pixels. Renderizar cada tamanho diretamente de vetores em vez de reduzir repetidamente um PNG de 1024.

Ícones de 24 unidades da interface e densidade da janela principal e dos menus continuam seguindo [layout-scale.json](../../../specs/visual/layout-scale.json). Esses recursos não acrescentam janela QML nem alteram dimensões lógicas da janela principal ou dos menus. Dimensões PNG das plataformas são dimensões de amostragem e não devem entrar na geometria lógica dos componentes QML.

<a name="xdzip-format-and-platform-mapping"></a>

## Mapeamento de formatos e plataformas

| ID do ícone | Rótulo | Extensão / uso | Cor |
| --- | --- | --- | --- |
| `zip` | ZIP | `.zip` | Azul da marca |
| `7z` | 7Z | `.7z` | Índigo |
| `rar` | RAR | `.rar` | Roxo |
| `tar` | TAR | `.tar` | Âmbar |
| `tar-gz` | TAR.GZ | Sufixo composto `.tar.gz` confirmado | Verde-azulado |
| `tgz` | TGZ | `.tgz` | Verde-azulado |
| `gz` | GZ | Candidato do sistema `.gz` | Verde-azulado |
| `cab` | CAB | `.cab` | Azul aço |
| `iso` | ISO | `.iso` | Azul lago |
| `udf` | UDF | `.udf` | Azul ciano |
| `archive` | Nenhum | Alternativa para arquivos reconhecidos sem ícone específico | Cinza-azulado neutro |

A tabela expressa intenção de design, não um registro uniforme entre plataformas. Quando possível, a correspondência prioriza `.tar.gz` completo antes de `.gz`; sistemas que reconhecem apenas a última extensão podem mostrar GZ para ambos. Não prometer que todas as plataformas distinguem sufixos compostos automaticamente. O ícone GZ indica uma extensão compartilhada pelo GZIP independente e pela camada externa TAR.GZ; a leitura real continua sujeita à validação do conteúdo e aos contratos de formato.

Criptografia e divisão em volumes são estados do arquivo compactado, não inferidos dos ícones de extensão; não desenhar cadeados ou marcas de sucesso. Volumes ZIP, 7z e RAR reconhecidos podem reutilizar sua família no aplicativo sem registrar `.001`, `.rNN`, `.zNN` ou `.img` genéricos. Ícones ISO/UDF não implicam montagem ou execução. Uma extensão correspondente não comprova conteúdo válido.

O instalador comum do Windows usa ProgIDs e ICOs por extensão. MSIX declara candidatos “Abrir com” com PNGs derivados dos quadros ICO correspondentes. Ambos seguem o mapeamento gerado, que pode reutilizar imagens entre sufixos. O macOS usa ICNS por tipo de documento, incluindo TAR comprimido e GZIP independente. Linux usa ícones MIME do sistema. O aplicativo padrão escolhido e os caches de ícones afetam a exibição real. Consulte [instalação e associações](../engineering/installers-associations.md).

MSIX declara um `uap:Logo` separado para cada associação de formato. Seus PNGs de tamanho alvo reutilizam byte a byte os quadros ICO canônicos do Windows, preservando simplificação em tamanhos pequenos, rótulos nos grandes, limites visíveis e centralização. Logos de aplicativo são recursos separados com formas padrão e sem placa para ambos os temas. [msix-icons.json](../../../specs/visual/msix-icons.json) define tamanhos e variantes do pacote; MakePRI os indexa antes do empacotamento MakeAppx. A arte prevista do documento não inclui placa do sistema nem alternativa com logo do aplicativo.

A entrada de aplicativo em aplicativos padrão e “Abrir com” do Windows usa o [ICO da marca](../../../design/assets/brand/xdzip.ico), incorporado ao executável principal e ao inicializador de associações. Ícones de documento por formato não substituem a identidade do aplicativo. O sistema deve mostrar uma única entrada opcional xdzip com o ícone da marca; o comando compartilhado antigo mantém compatibilidade com escolhas existentes. Isso reutiliza recursos e não altera a geometria lógica da janela principal, menus ou QML. Ícones nativos de aplicativo, documentos e número de candidatos são aceitos separadamente; uma caixa genérica de aplicativo do sistema não deve ser classificada como problema de ícone de documento.

<a name="xdzip-generation-and-delivery"></a>

## Geração e entrega

Executar na raiz do repositório:

```sh
python scripts/build-association-icons.py
python scripts/build-design-assets.py --metadata-only
python scripts/build-association-icons.py --check --check-runtime
# Exportar artefatos locais com Pillow, Playwright e Chromium existentes.
python scripts/build-association-icons.py --export build/file-association-icons
python scripts/build-association-icons.py --check --export build/file-association-icons
# Após copiar as exportações ICO/ICNS desejadas para a pasta canônica:
python scripts/build-design-assets.py --metadata-only --refresh-association-icons
```

`XDZIP_CHROMIUM` pode indicar um caminho existente de Chromium/Edge. Originais SVG, fontes pequenas, especificação e página de revisão são versionados; as fontes estão registradas no [manifest.json](../../../design/assets/manifest.json) principal. A entrada de geração é [build-association-icons.py](../../../scripts/build-association-icons.py), com composições em [association_icon_assets.py](../../../scripts/association_icon_assets.py).

Os artefatos exportados contêm PNGs genéricos e macOS de 16 a 1024 pixels por formato, além de ICO e ICNS. ICO e ICNS do produto, inventariados pelo registro de formatos, ficam junto às fontes vetoriais em `file-associations/`; contêineres ZIP preservam os bytes originais anteriores à migração e os demais vêm das exportações correspondentes. Compilações comuns leem esses contêineres diretamente sem navegador. PNGs temporários, exportações completas, folhas de contato e comprovantes permanecem em `build/` ou `dist/`.

<a name="xdzip-verification-and-acceptance"></a>

## Verificação e aceitação

Verificações de geração conferem se os SVGs correspondem à especificação e não dependem de fontes ou imagens externas. Verificações de plataforma cobrem tela e RGBA, cantos transparentes, altura visível e centralização do corpo, resumos vetoriais e concordância dos pixels decodificados de cada representação ICO/ICNS. Os limites do corpo são medidos com alfa ≥ 128, usando a tolerância da marca de 1,25 pixels físicos.

A revisão visual cobre fundos claros, escuros e selecionados, tamanhos de 16 a 256 pixels e renderização real com DPR normal e alto. A página de revisão usa pixels CSS; capturas do navegador não devem ser identificadas como aceitação nativa de Explorer, Finder ou Linux. A integração posterior deve verificar nomes reais, sufixos compostos, caches de ícones, antes e depois da escolha padrão e exibição após instalação e desinstalação. Registrar estado e escopo real em issues do GitHub em vez de acumular relatórios aqui.
