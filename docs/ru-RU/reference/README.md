<a name="xdzip-product-design-and-technical-reference"></a>

# Проектирование продукта и технический справочник

[English](../../reference/README.md) | [简体中文](../../zh-CN/reference/README.md) | [繁體中文](../../zh-TW/reference/README.md) | [日本語](../../ja-JP/reference/README.md) | [한국어](../../ko-KR/reference/README.md) | [Français](../../fr-FR/reference/README.md) | [Deutsch](../../de-DE/reference/README.md) | [Español](../../es-ES/reference/README.md) | [Português (Brasil)](../../pt-BR/reference/README.md) | Русский

Пути выполнения настольного приложения и CLI описаны в [архитектуре системы](../../design/system-architecture.md). В разделах ниже указаны контракты для отдельных функций. Поддерживаемые операции собраны в [таблице форматов](../../../README.ru-RU.md#xdzip-format-support). Порядок работы и подготовки среды см. в [руководстве пользователя](../user-guide/README.md), [примерах CLI](../integration/README.md) и [руководстве участника](../contributing/README.md).

<a name="xdzip-architecture-and-interfaces"></a>

## Архитектура и интерфейсы

- [Обзор проектирования](../design/index.md)
- [Требования и границы операций](../design/product-requirements.md)
- [Модель предметной области, идентификаторы записей и версии источников](../design/domain-data.md)
- [Мост команд и событий Qt/Rust](../design/qt-rust-bridge.md)
- [Планирование задач, ход выполнения, отмена и завершение](../design/task-scheduling.md)
- [Бюджеты производительности и их проверка](../design/performance.md)
- [Термины и ограничения реализации](../design/glossary.md)
- [Прослеживаемость требований и проверок](../design/requirements-traceability.md)
- [Архитектурные решения](../design/architecture-decisions.md)
- [Развитие от прототипов до приложения и среды Rust](../design/architecture-evolution.md)
- [Структура репозитория и зависимости](../engineering/repository-layout.md)
- [Контракты интерфейсов и проверки](../engineering/contracts.md)
- [Среда выполнения продукта и жизненный цикл Qt](../engineering/product-runtime.md)
- [Сеансы и завершение приложения](../../engineering/application-sessions.md)
- [Причины выбора моста cxx](../engineering/cxx-admission.md)
- [Службы календаря и кодировки хоста для процессов форматов](../engineering/host-platform-services.md)

<a name="xdzip-reading-and-writing-archives"></a>

## Чтение и запись архивов

- [Файловые разрешения, подготовка и безопасная запись](../design/file-security.md)
- [Контракт просмотра в режиме чтения](../../engineering/readonly-browser.md)
- [Просмотр ZIP в режиме чтения](../engineering/zip-readonly-browsing.md)
- [Индексы больших архивов и просмотр по запросу](../engineering/large-index.md)
- [Чтение томов и временное открытие](../engineering/archive-opening.md)
- [Адаптивная кодировка имён файлов](../engineering/filename-encoding.md)
- [Предпросмотр содержимого](../engineering/content-preview.md)
- [Предварительная проверка метаданных перед распаковкой](../engineering/extraction-preflight.md)
- [Безопасная распаковка ZIP](../../engineering/zip-safe-extraction.md)
- [Проверка целостности архивов](../engineering/archive-checking.md)
- [Открытие записей, перетаскивание и изменение архивов](../../engineering/archive-transfers.md)
- [Пароли архивов и открытие зашифрованных каталогов](../engineering/password-extraction.md)
- [Контракт создания ZIP](../../engineering/zip-creation.md)
- [Создание ISO9660 и Joliet](../engineering/iso-creation.md)
- [Встроенные самораспаковывающиеся ZIP для Windows](../engineering/sfx-creation.md)

<a name="xdzip-format-support-and-implementation"></a>

## Поддержка и реализация форматов

- [Встроенные возможности, профили создания и охват проверок](../engineering/format-registry.md)
- [Адаптеры форматов и проект испытания внешнего SDK](../design/codec-extension-sdk.md)
- [Испытание внешнего SDK в режиме чтения и область выполнения](../engineering/sdk-trial.md)
- [Контракт изолированного процесса форматов](../engineering/native-codecs.md)
- [Контракт собственных форматов и алгоритмов Rust](../../engineering/independent-rust-codecs.md)
- [Учёт вспомогательной работы декодирования](../engineering/auxiliary-decoding-work.md)
- [Допуск зависимости метаданных ZIP](../engineering/zip-metadata-admission.md)
- [Допуск зависимостей распаковки и вывода ZIP](../engineering/zip-extraction-admission.md)
- [Источники формата 7z и независимая реализация](../engineering/sevenzip-format-sources.md)
- [Основы независимой реализации RAR и границы совместимости](../../engineering/rar-format-sources.md)
- [Источники формата CAB и основы реализации](../../engineering/cab-format-sources.md)
- [Контракт реализации TAR/GZIP](../engineering/tar-gzip.md)
- [Потоки одного файла и пакеты на основе ZIP](../engineering/single-file-streams.md)
- [Контракт потоков LZ4](../engineering/lz4-streams.md)
- [Контракт потоков Zstandard](../engineering/zstandard-streams.md)
- [Контракт Unix compress Z](../engineering/compress-streams.md)
- [Контракт обычных потоков Brotli](../engineering/brotli-streams.md)
- [Структура данных и алгоритмы LHA](../../engineering/lha-containers.md)
- [Контейнеры PMA и методы PMarc](../engineering/pma-streams.md)
- [Сжатые потоки и допуск контейнеров ARJ](../engineering/arj-streams.md)
- [Контейнеры ARC и профили сжатых потоков](../engineering/arc-streams.md)
- [Контейнеры BH и поддерживаемый профиль BlakHole](../engineering/bh-containers.md)
- [Источники и лицензионные условия ACE, ALZ и EGG](../engineering/ace-alz-egg-format-sources.md)
- [Оболочки AES Crypt в режиме чтения](../engineering/aescrypt-envelopes.md)
- [Контракт контейнеров DEB](../engineering/deb-containers.md)
- [Контракт контейнеров ASAR](../engineering/asar-containers.md)
- [Текстовые оболочки UU, UUE и XXE](../engineering/text-envelopes.md)
- [Источники и лицензии WIM, MSI и форматов установщиков](../engineering/wim-msi-installer-format-sources.md)
- [Источники и лицензии образов дисков и других архивных форматов](../engineering/disk-image-archive-format-sources.md)
- [Перечень форматов Bandizip и возможность поддержки в xdzip](../engineering/bandizip-format-assessment.md)

<a name="xdzip-desktop-interaction-and-visual-design"></a>

## Взаимодействие и оформление приложения

- [Представления Qt Quick и обязанности контроллеров](../design/qt-quick-ui.md)
- [Спецификации оформления приложения](../../visual/README.md)
- [Стандарт оформления интерфейса](../../visual/design-standard.md)
- [Процесс проектирования и корректировки интерфейса](../visual/design-workflow.md)
- [Значки ассоциаций файлов](../visual/file-association-icons.md)
- [Проверка визуального соответствия HTML](../visual/html-fidelity.md)
- [Дополнительные требования к взаимодействию и текстам](../ux/interaction-spec.md)
- [Функции, компоненты и взаимодействие главного окна](../../ux/main-window-components.md)
- [Проектирование и интеграция дерева каталогов](../../ux/directory-tree.md)
- [Страница приветствия главного окна](../ux/welcome.md)
- [Загрузка на странице приветствия и завершение создания](../ux/welcome-opening.md)
- [Ввод пароля при распаковке](../ux/password-extraction.md)
- [Окна задач и проверка целостности](../ux/task-windows.md)
- [Окно настроек и язык интерфейса](../ux/settings-localization.md)
- [Термины интерфейса на упрощённом китайском](../ux/chinese-terminology.md)
- [Системные меню и область уведомлений](../ux/desktop-integration.md)
- [Окна и заголовки по платформам](../ux/platform-windows.md)
- [Состояния макетов высокой точности и контракт реализации](../ux/hifi-state-spec.md)
- [Прототип взаимодействия v0.1 и соответствие функций](../ux/prototype-spec.md)

<a name="xdzip-platform-integration-releases-and-trust"></a>

## Интеграция, выпуски и доверие

- [Установщики и ассоциации файлов](../../engineering/installers-associations.md)
- [Поддержка систем и политика сопровождения](../engineering/support-policy.md)
- [Действия файлового менеджера и отдельный хост задач](../engineering/shell-actions.md)
- [Обновления по инициативе пользователя](../../engineering/software-update.md)
- [Жизненный цикл данных и диагностика](../engineering/privacy.md)
- [Версия продукта и идентичность сборки](../engineering/versioning.md)
- [Протокол GitHub Release v1](../engineering/release-protocol.md)
- [Локальный DMG для разработки под macOS](../../engineering/macos-deployment.md)
- [Автономный каталог среды Linux и его проверка](../engineering/linux-deployment.md)
- [Проверка зависимостей развёртывания и SBOM](../engineering/deployment-audit.md)
- [Указание авторства и лицензии фактически используемых компонентов](../engineering/runtime-attribution.md)
- [Происхождение и уведомления нативных компонентов](../engineering/native-runtime-materials.md)
- [Проверка соответствия Qt SDK и исходников](../engineering/qt-sdk-correspondence.md)
- [Лицензии инструментов разработки и среды Windows](../engineering/windows-licensing.md)
- [Выбор лицензии и сторонние материалы](../engineering/licensing.md)
- [Допуск зависимостей и модулей форматов](../../engineering/dependencies.md)
- [Соответствующие исходники и материалы для пересборки](../../engineering/source-delivery.md)
- [Пересборка данных ICU 73.2 из CLDR](../engineering/icu-cldr-rebuild.md)

<a name="xdzip-build-and-cli-reference"></a>

## Справочник по сборке и CLI

- [Проект сборки, развёртывания и выпуска](../design/build-release.md)
- [Соглашения по коду и уровни проверок](../design/quality-engineering.md)
- [Подготовка среды разработки](../engineering/development.md)
- [Сборки для разработки и проверки платформ](../../engineering/developer-build.md)
- [Закреплённые версии инструментов и платформ](../engineering/toolchain.md)
- [Команды CLI, JSON, установка и квитанции обновлений](../engineering/command-line.md)
