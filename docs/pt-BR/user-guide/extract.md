# Extrair todos os arquivos ou apenas a seleção

[English](../../user-guide/extract.md) | [简体中文](../../zh-CN/user-guide/extract.md) | [繁體中文](../../zh-TW/user-guide/extract.md) | [日本語](../../ja-JP/user-guide/extract.md) | [한국어](../../ko-KR/user-guide/extract.md) | [Français](../../fr-FR/user-guide/extract.md) | [Deutsch](../../de-DE/user-guide/extract.md) | [Español](../../es-ES/user-guide/extract.md) | Português (Brasil) | [Русский](../../ru-RU/user-guide/extract.md)

<a name="xdzip-from-the-archive-window"></a>

## Pela janela do arquivo compactado

1. Abra o arquivo compactado. O botão principal da barra de ferramentas extrai os itens selecionados ou o arquivo inteiro se não houver seleção. Use a seta ao lado do botão para escolher **Extrair tudo** ou **Extrair seleção**.
2. Confirme os itens e escolha um destino em que você possa gravar. Se a opção estiver disponível, você pode criar uma pasta com o nome do arquivo compactado.
3. Decida como resolver conflitos. **Perguntar o que fazer** pausa a extração para você decidir. **Ignorar arquivos existentes** preserva o arquivo existente. **Renomear e manter as duas versões** salva o novo arquivo separadamente. **Sobrescrever arquivos existentes** substitui um arquivo existente quando permitido, após a verificação.
4. Inicie a extração e confira o resultado. Em caso de falha ou cancelamento, os arquivos extraídos antes da interrupção podem permanecer no destino. Consulte os detalhes para saber o que falta antes de tentar novamente. [Verificações, tarefas e resultados](checks-and-tasks.md)

O menu também oferece **Verificar metadados** para seleções de até 128 entradas reais do arquivo compactado. A verificação aponta problemas como nomes inseguros, métodos incompatíveis e limites declarados. Selecione entradas específicas para obter o relatório. Uma pasta implícita exibida apenas para navegação ainda pode ser extraída se o formato aceitar essa operação. A verificação prévia não lê os dados nem autoriza a gravação. [Verificação prévia e limites da extração](../engineering/extraction-preflight.md)

Arquivos ZIP grandes com índice em disco permitem extrair arquivos comuns selecionados dentro dos limites da tarefa. Se o arquivo compactado inteiro passar do limite, selecione um grupo menor. Se a origem mudar ou for trocada, abra-a novamente e refaça a seleção. [Limites de arquivos grandes](../engineering/large-index.md)

<a name="xdzip-from-the-file-manager"></a>

## Pelo gerenciador de arquivos

No Windows 11, use **Mostrar mais opções** para abrir o menu clássico. A edição da Microsoft Store também pode mostrar ações no menu moderno. No Ubuntu, use a extensão do Nautilus. No macOS, ative a extensão do Finder nos ajustes do sistema ou use o serviço registrado no Finder. Se o menu não estiver disponível, abra o arquivo compactado no xdzip e faça a extração pela janela do aplicativo.

No menu do gerenciador, você pode extrair na pasta atual de cada arquivo compactado, criar uma pasta separada para cada um, definir a pasta automaticamente ou escolher o destino. O modo automático só dispensa uma pasta externa se o resultado validado já contiver exatamente uma pasta no nível superior. O processamento em lote exibe o resultado de cada arquivo compactado e continua se houver uma falha individual. Cancelar impede os próximos itens e solicita a interrupção do trabalho em andamento. [Ações e configurações do gerenciador de arquivos](../engineering/shell-actions.md)
