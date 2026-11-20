# Criar e modificar arquivos compactados

[English](../../user-guide/create-and-modify.md) | [简体中文](../../zh-CN/user-guide/create-and-modify.md) | [繁體中文](../../zh-TW/user-guide/create-and-modify.md) | [日本語](../../ja-JP/user-guide/create-and-modify.md) | [한국어](../../ko-KR/user-guide/create-and-modify.md) | [Français](../../fr-FR/user-guide/create-and-modify.md) | [Deutsch](../../de-DE/user-guide/create-and-modify.md) | [Español](../../es-ES/user-guide/create-and-modify.md) | Português (Brasil) | [Русский](../../ru-RU/user-guide/create-and-modify.md)

<a name="xdzip-create-an-archive"></a>

## Criar um arquivo compactado

1. Abra a janela de criação na tela inicial ou com **Novo**. Adicione arquivos ou pastas e confira a lista de entrada.
2. Defina o nome e o destino. Escolha o formato e uma de suas opções de compactação. ZIP é uma opção comum para compartilhar arquivos. Consulte a [tabela de formatos](../../../README.pt-BR.md#xdzip-format-support) para ver as opções de criação. Um fluxo de arquivo único, como GZIP, aceita um arquivo comum.
3. ZIP e 7z permitem criptografia e volumes divididos quando as opções escolhidas oferecem esses recursos. Selecione ISO para criar uma imagem de disco de dados ou use o perfil SFX do Windows x64 para criar um ZIP autoextraível. Inicie a operação e confira no resultado quais arquivos foram salvos. [Senhas e volumes](passwords-and-volumes.md)
4. Se precisar cancelar ou tentar novamente, confira o resultado primeiro. O arquivo compactado é gravado em local temporário antes de publicar o resultado concluído.

<a name="xdzip-add-replace-or-move-files"></a>

## Adicionar, substituir ou mover arquivos

Abra um ZIP/ZIPX, 7z, LHA/LZH, TAR ou TAR compactado compatível e acesse a pasta de destino dentro dele. Arraste arquivos ou pastas locais para a lista ou a grade, depois confira e confirme a inclusão. Entre as opções de conflito está substituir arquivos comuns compatíveis dentro do arquivo compactado. Para mover uma entrada, arraste-a para outra pasta do mesmo arquivo compactado e confirme. Arrastar para fora da janela exporta uma cópia. [Como arrastar e soltar em cada plataforma](../../engineering/archive-transfers.md)

A criação e a inclusão ignoram arquivos `.DS_Store` e pastas `__MACOSX` e `_MACOSX`. As opções de modificação dependem do formato, do conteúdo e da origem. O fluxo normal de modificação não aceita arquivos criptografados, divididos ou indexados em disco. Quando não for possível editar, extraia o conteúdo em uma nova pasta e crie outro arquivo compactado. [Regras de modificação e arraste](../../engineering/archive-transfers.md)

<a name="xdzip-rename-or-delete-entries"></a>

## Renomear ou excluir entradas

Em uma pasta comum de um arquivo compactado editável, clique com o botão direito em um arquivo ou pasta na lista ou grade. Se o item já estiver selecionado, a seleção atual será mantida. Caso contrário, apenas o item clicado será selecionado.

- **Renomear** está disponível para um único item. Digite o novo nome na mesma pasta. Ao renomear uma pasta, todos os caminhos abaixo dela são atualizados. Se o nome já estiver em uso, escolha outro.
- **Excluir** aceita um ou mais itens selecionados. Confira o nome ou a quantidade antes de confirmar. Ao excluir uma pasta, todo o conteúdo dela também é excluído. Se você excluir todos os itens da raiz, o arquivo compactado continuará disponível, vazio. Use uma extensão TAR composta, como `.tar.xz`, se o arquivo vazio for interpretado como fluxo compactado independente.

Para editar um item encontrado na busca, abra primeiro a pasta que o contém. [Regras de alteração de arquivos compactados](../../engineering/archive-transfers.md)
