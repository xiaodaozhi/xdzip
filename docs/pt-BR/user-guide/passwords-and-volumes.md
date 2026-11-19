# Senhas e volumes divididos

[English](../../user-guide/passwords-and-volumes.md) | [简体中文](../../zh-CN/user-guide/passwords-and-volumes.md) | [繁體中文（台灣）](../../zh-TW/user-guide/passwords-and-volumes.md) | [日本語](../../ja-JP/user-guide/passwords-and-volumes.md) | [한국어](../../ko-KR/user-guide/passwords-and-volumes.md) | [Français](../../fr-FR/user-guide/passwords-and-volumes.md) | [Deutsch](../../de-DE/user-guide/passwords-and-volumes.md) | [Español](../../es-ES/user-guide/passwords-and-volumes.md) | Português (Brasil) | [Русский](../../ru-RU/user-guide/passwords-and-volumes.md)

<a name="xdzip-passwords"></a>

## Senhas

AES Crypt e arquivos compactados com nomes criptografados, como 7z com criptografia de nomes, exigem uma senha para mostrar a lista de arquivos. Outros podem pedir a senha durante a extração, verificação ou visualização. Mantenha os espaços e respeite as maiúsculas e minúsculas ao digitar a senha. ZIP AES-256 é aceito nos perfis compatíveis. ZIP AES-128/192 e ZipCrypto tradicional não são aceitos. Se a senha estiver correta e ainda assim houver falha, confira se a criptografia é compatível e se o arquivo está danificado. [Criptografia compatível](../engineering/password-extraction.md)

Para criar um arquivo compactado protegido por senha, escolha um formato com criptografia. ZIP AES-256 protege o conteúdo e deixa os nomes visíveis. Se precisar proteger também a lista de arquivos, escolha 7z e ative a criptografia de nomes. Mantenha a senha em um local separado do arquivo compactado.

<a name="xdzip-split-volumes"></a>

## Volumes divididos

1. Coloque todos os volumes do mesmo arquivo compactado na mesma pasta e mantenha os nomes originais.
2. Abra um volume compatível. O XDZip procura as demais partes na mesma pasta. Os volumes numerados de ZIP e 7z, o PKZIP e os conjuntos RAR e CAB compatíveis seguem regras de nomes diferentes.
3. Se o conjunto estiver incompleto, obtenha as partes que faltam e reabra o arquivo. Juntar ou renomear partes manualmente pode torná-las ilegíveis.

Para criar um arquivo dividido, escolha ZIP ou 7z e defina o tamanho do volume na janela de criação. Compartilhe o conjunto completo. Dividir o arquivo compactado não o torna editável no XDZip. [Nomes de volumes e perfis compatíveis](../engineering/archive-opening.md)
