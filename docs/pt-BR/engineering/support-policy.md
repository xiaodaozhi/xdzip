<a name="xdzip-system-support-and-maintenance-policy"></a>
# Política de suporte e manutenção

[English](../../engineering/support-policy.md) | [简体中文](../../zh-CN/engineering/support-policy.md) | [繁體中文（台灣）](../../zh-TW/engineering/support-policy.md) | [日本語](../../ja-JP/engineering/support-policy.md) | [한국어](../../ko-KR/engineering/support-policy.md) | [Français](../../fr-FR/engineering/support-policy.md) | [Deutsch](../../de-DE/engineering/support-policy.md) | [Español](../../es-ES/engineering/support-policy.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/support-policy.md)

O responsável pelo projeto aprovou a política inicial em 2026-10-19. A versão `v0.1.0` foi lançada publicamente em 2026-11-06. Consulte a [página de versões](https://github.com/xiaodaozhi/xdzip/releases) para encontrar as versões atuais e os downloads. Esta política define o escopo do suporte. As verificações de versão mínima continuam valendo. Os resultados da CI e da validação pelo usuário se aplicam aos ambientes testados. O usuário deu por concluída a verificação manual.

<a name="xdzip-initial-support-scope"></a>
## Escopo inicial

| Plataforma | Escopo inicial | Verificações e limites |
| --- | --- | --- |
| macOS | macOS 14 ou posterior, Apple Silicon arm64 | Alvo de implantação 14.0, CI nativa e DMG de desenvolvimento local. Sem pacote para Intel ou Universal. O suporte a todas as versões futuras do sistema não é garantido. |
| Windows | Versões x86_64 do Windows 11 que a Microsoft ainda mantém | Compilação com MSVC, instalador por usuário e testes reais de instalação, atualização, reversão e desinstalação na CI. Os testes em Windows Server não abrangem todas as versões do Windows 11. Sem pacote ARM64. |
| Linux | Ubuntu 24.04 LTS x86_64, X11, glibc 2.39 ou posterior | Verificação do .deb atual e da execução isolada sem o SDK do Qt. Outras distribuições, versões anteriores da glibc, Wayland e ARM64 ficam fora do escopo. |

A versão mínima do macOS segue o alvo de implantação atual. A versão 13 citada em documentos de projeto anteriores não foi verificada. O suporte padrão do Windows 10 terminou em 2025-10-14. O Windows 10 está fora do compromisso inicial. As modalidades LTSC e ESU não são incluídas automaticamente. Consulte o [anúncio de ciclo de vida da Microsoft](https://learn.microsoft.com/en-us/lifecycle/announcements/windows-10-end-of-support). A lista de [plataformas compatíveis com o Qt 6.8](https://doc.qt.io/qt-6.8/supported-platforms.html) informa o escopo dessa dependência. O xdzip faz suas próprias verificações.

<a name="xdzip-maintenance"></a>
## Manutenção

Este projeto pessoal é mantido conforme os recursos disponíveis. Não oferece serviço pago, prazo fixo de resposta ou compromissos de segurança além do trabalho realizado. A versão estável pública mais recente tem prioridade. As compilações de desenvolvimento podem trazer mudanças de compatibilidade. As notas de versão listam os problemas conhecidos e as opções de reversão. Em caso de problema de segurança, os downloads afetados podem ser retirados. Em “Sobre o xdzip”, o usuário pode verificar se há atualizações e iniciar a instalação. O aplicativo não verifica atualizações em segundo plano nem as instala silenciosamente. O código atual também oferece um comando de atualização na CLI. O [contrato da linha de comando](command-line.md) descreve as instalações elegíveis e as diferenças em relação aos arquivos já publicados.

As versões candidatas continuam usando o Qt 6.8.2 e as versões fixadas de Rust e Cargo para permitir verificações reproduzíveis. Isso não garante que correções antigas estejam livres de vulnerabilidades nem que atualizações LTS gratuitas estejam disponíveis indefinidamente. Antes de cada lançamento público, o responsável pelo projeto consulta os avisos oficiais de segurança do Qt e as atualizações de código aberto disponíveis. A avaliação dá prioridade às versões públicas ainda mantidas. Se uma dependência tiver uma falha de segurança sem correção que afete o xdzip, o lançamento correspondente será suspenso. Atualizar o Qt exige revalidar o código-fonte, o licenciamento dos módulos, as interfaces privadas do Windows, a substituição dinâmica, as compilações nas três plataformas e o funcionamento real de menus e janelas. A contratação de manutenção comercial é opcional.

<a name="xdzip-approved-scope-and-changes"></a>
## Escopo aprovado e alterações

O escopo aprovado inclui as plataformas iniciais, a manutenção apenas da versão pública mais recente de acordo com os recursos disponíveis, a ausência de um prazo fixo de resposta e a identificação de compilações de desenvolvimento como experimentais. Os resultados de `v0.1.0` foram registrados na publicação e nas respectivas questões de validação. Esses resultados não oferecem uma garantia de desempenho para todas as versões dos sistemas. Novas arquiteturas, sistemas ou canais de distribuição devem ser implementados e verificados antes da atualização desta política. Consulte a [ADR-011](../design/architecture-decisions.md#xdzip-adr-011-initial-support-scope-and-personal-project-maintenance-policy).

Assinatura oficial, notarização e contas pagas deixaram de ser exigidas. Para relatar vulnerabilidades, consulte a [política de segurança](../../../SECURITY.md).
