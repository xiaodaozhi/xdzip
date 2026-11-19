# Instalar o XDZip e abrir um arquivo compactado

[English](../../user-guide/install.md) | [简体中文](../../zh-CN/user-guide/install.md) | [繁體中文（台灣）](../../zh-TW/user-guide/install.md) | [日本語](../../ja-JP/user-guide/install.md) | [한국어](../../ko-KR/user-guide/install.md) | [Français](../../fr-FR/user-guide/install.md) | [Deutsch](../../de-DE/user-guide/install.md) | [Español](../../es-ES/user-guide/install.md) | Português (Brasil) | [Русский](../../ru-RU/user-guide/install.md)

Baixe o pacote para seu sistema na [última versão estável do GitHub](https://github.com/xiaodaozhi/xdzip/releases/latest) ou use a Microsoft Store no Windows. A página do GitHub informa o nome e a versão do arquivo e fornece `SHA256SUMS` para verificar downloads diretos. Consulte os requisitos na [política de compatibilidade](../engineering/support-policy.md).

| Sistema | Instalação ou inicialização |
| --- | --- |
| Windows 11 x64 | Procure **XDZip** na Microsoft Store ou execute o arquivo `-setup.exe` do GitHub para instalar na sua conta de usuário. O instalador do GitHub não é assinado. Para usar o ZIP portátil, extraia todo o pacote e inicie `xdzip.exe` na pasta extraída. |
| macOS 14 ou posterior, Apple Silicon | Abra o DMG e arraste `xdzip.app` para **Aplicativos**. No caso de um `.app.zip`, extraia-o primeiro e mova o aplicativo para Aplicativos. Abra a cópia em Aplicativos. As compilações do GitHub têm assinatura de integridade ad hoc e não são notarizadas. Se o macOS exibir um aviso, verifique a origem do download e a soma de verificação. |
| Ubuntu 24.04 LTS x64, X11, glibc 2.39 ou posterior | Instale o pacote `.deb` pelo gerenciador de pacotes. Para usar o `.tar.gz` portátil, extraia todos os arquivos e execute `./xdzip` na pasta de execução extraída. Mantenha o inicializador junto aos arquivos de execução. |

A versão para desktop não oferece suporte a Windows ARM64, Macs com Intel, Linux ARM64 ou Wayland. A ferramenta independente de linha de comando para Linux funciona sem X11 e exige OpenSSL 3 e os certificados do sistema. Escolha um arquivo cujo nome comece com `xdzip-cli-` e siga o [guia rápido da linha de comando](../integration/README.md).

<a name="xdzip-open-an-archive"></a>

## Abrir um arquivo compactado

Use **Abrir arquivo compactado** no XDZip ou arraste o arquivo para a tela inicial. No gerenciador de arquivos, use **Abrir com** quando o pacote instalado estiver registrado. Para tornar o XDZip o aplicativo padrão de um tipo de arquivo, selecione-o nas configurações do sistema. A edição da Microsoft Store oferece **Definir aplicativos padrão no Windows** na tela de associações de arquivos. Os aplicativos padrão e as ações de compactação do menu de contexto têm configurações diferentes. Consulte [extração pelo gerenciador de arquivos](extract.md#xdzip-from-the-file-manager) e [instalação e associações](../engineering/installers-associations.md).

Em seguida, [navegue pelos arquivos e visualize seu conteúdo](browse.md).
