# Vérifier les archives et gérer les tâches

[English](../../user-guide/checks-and-tasks.md) | [简体中文](../../zh-CN/user-guide/checks-and-tasks.md) | [繁體中文（台灣）](../../zh-TW/user-guide/checks-and-tasks.md) | [日本語](../../ja-JP/user-guide/checks-and-tasks.md) | [한국어](../../ko-KR/user-guide/checks-and-tasks.md) | Français | [Deutsch](../../de-DE/user-guide/checks-and-tasks.md) | [Español](../../es-ES/user-guide/checks-and-tasks.md) | [Português (Brasil)](../../pt-BR/user-guide/checks-and-tasks.md) | [Русский](../../ru-RU/user-guide/checks-and-tasks.md)

<a name="xdzip-test-an-archive"></a>

## Vérifier une archive

1. Ouvrez l’archive et choisissez **Vérifier** dans la barre d’outils.
2. Saisissez le mot de passe si la boîte de dialogue le demande, puis attendez le résultat. La tâche vérifie le contenu pris en charge sans écrire de fichiers extraits.
3. En cas d’échec, consultez la raison et l’entrée concernée si elle est indiquée. Certains formats permettent une vérification par somme de contrôle ou une authentification. D’autres permettent uniquement de contrôler la structure et la lisibilité. Une vérification réussie couvre les contrôles prévus par le format. [Contrôles d’intégrité](../engineering/archive-checking.md)

Certaines archives ne peuvent pas être vérifiées. Par exemple, un ZIP indexé sur disque permet d’extraire des fichiers sélectionnés dans les limites de la tâche. Pour NSIS, seules les métadonnées du programme d’installation sont accessibles. Voir [les limites des grandes archives](../engineering/large-index.md) et [les formats pris en charge](../../../README.fr-FR.md#xdzip-format-support).

<a name="xdzip-progress-cancellation-and-results"></a>

## Progression, annulation et résultats

Ouvrez le menu des tâches pour revenir à une tâche en cours ou consulter ses résultats. Fermer un onglet d’archive n’annule pas sa tâche en arrière-plan. L’opération peut attendre une réponse à une demande de mot de passe ou à un conflit de noms.

Choisissez **Annuler** dans la fenêtre de la tâche pour demander l’arrêt et attendez l’état final. Les fichiers déjà enregistrés peuvent rester en place. Développez les détails du résultat pour distinguer les éléments terminés, ignorés et inachevés. Consultez les nombres et les raisons affichés avant de décider quoi réessayer. Effacer l’historique des tâches terminées retire les entrées de tâches et conserve les fichiers sur le disque. [Fenêtre de tâche](../ux/task-windows.md)
