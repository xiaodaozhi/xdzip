# Solução de problemas do XDZip

[English](../../user-guide/troubleshooting.md) | [简体中文](../../zh-CN/user-guide/troubleshooting.md) | [繁體中文（台灣）](../../zh-TW/user-guide/troubleshooting.md) | [日本語](../../ja-JP/user-guide/troubleshooting.md) | [한국어](../../ko-KR/user-guide/troubleshooting.md) | [Français](../../fr-FR/user-guide/troubleshooting.md) | [Deutsch](../../de-DE/user-guide/troubleshooting.md) | [Español](../../es-ES/user-guide/troubleshooting.md) | Português (Brasil) | [Русский](../../ru-RU/user-guide/troubleshooting.md)

| O que aconteceu | O que fazer |
| --- | --- |
| Um arquivo compactado não abre | Confira os [formatos compatíveis](../../../README.pt-BR.md#xdzip-format-support), o método de compactação ou criptografia e todos os volumes. Se não puder acessar a origem, copie o arquivo e todas as partes para uma pasta local acessível, mantendo os nomes, e reabra. |
| Os nomes parecem incorretos ou não podem ser gravados | Confira se pode recriar o arquivo compactado com nomes em UTF-8. O XDZip detecta algumas codificações antigas, com limites para nomes curtos ou mistos. Extraia apenas entradas que o plano possa gravar com segurança. [Tratamento de nomes](../engineering/filename-encoding.md) |
| O arquivo tem propriedades, mas não é possível visualizar | Confira o tipo, o tamanho, a plataforma e a codificação. A visualização tem limites próprios e é independente da extração. [Limites da visualização](../engineering/content-preview.md) |
| O destino foi recusado ou o resultado está incompleto | Escolha uma pasta privada com permissões de gravação. Confira conflitos de nomes ou entradas incompatíveis e veja o resultado antes de tentar novamente. [Extrair arquivos](extract.md) |
| O arquivo foi substituído durante o trabalho | Abra a nova cópia e faça uma nova seleção. O XDZip confere a origem de novo antes da gravação. |
| O Explorer, Finder ou Arquivos não exibe a ação esperada | Confira o canal de instalação e se a extensão está ativa. No Windows 11, abra o menu clássico com **Mostrar mais opções**. No macOS, ative a extensão nas configurações do sistema ou experimente o serviço registrado. Você sempre pode abrir o arquivo primeiro no XDZip. [Extração pelo gerenciador de arquivos](extract.md#xdzip-from-the-file-manager) |
| A verificação ou modificação está desativada para um ZIP grande | Se o XDZip usar um índice em disco, selecione apenas os arquivos comuns de que precisa e extraia dentro dos limites da tarefa. [Limites de arquivos grandes](../engineering/large-index.md) |
| A atualização ou instalação falha | Confira a versão e o canal. Verifique as somas dos downloads manuais e siga as instruções de instalação correspondentes. [Recuperação de atualização](settings-and-updates.md) |

Ao relatar um problema, informe a versão e a compilação em **Sobre o xdzip**, seu sistema e arquitetura, formato e operação, além dos passos com os resultados esperados e obtidos. Se puder, use um exemplo pequeno que seja seu. Remova senhas e caminhos privados antes de compartilhar. [Relatar um problema](https://github.com/xiaodaozhi/xdzip/issues/new/choose)
