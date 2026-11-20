# Architecture evolution and current boundaries

English | [简体中文](../zh-CN/design/architecture-evolution.md) | [繁體中文（台灣）](../zh-TW/design/architecture-evolution.md) | [日本語](../ja-JP/design/architecture-evolution.md) | [한국어](../ko-KR/design/architecture-evolution.md) | [Français](../fr-FR/design/architecture-evolution.md) | [Deutsch](../de-DE/design/architecture-evolution.md) | [Español](../es-ES/design/architecture-evolution.md) | [Português (Brasil)](../pt-BR/design/architecture-evolution.md) | [Русский](../ru-RU/design/architecture-evolution.md)

## From framework experiments to the product

Early framework experiments used a synthetic archive list and simulated tasks. The product adopted Qt Quick and a typed CXX bridge, then implemented real archive, task and file operations in Rust. Prototype JSON/C ABI and timing numbers belong to those experiments. The [architecture decision records](architecture-decisions.md) preserve the reasons for the choices and conditions for revisiting them.

The product uses bounded row models and checks query identity so old results cannot replace a new view. It adds physical entry identities, explicit format capabilities, file grants, staging and verified publication. The standalone CLI calls the same core without Qt. [System architecture](system-architecture.md), [Domain model](domain-data.md)

## Current, experimental and proposed work

| Area | Current state |
| --- | --- |
| Runtime and indexing | Implemented in the host. Ordinary single-file ZIP/ZIP64 can migrate to a private disk index. Task limits and metadata limits remain separate. [Performance design](performance.md) |
| Format isolation | Built-in format routing selects either in-process Rust or the bundled format worker. A supported operation is defined by its registry/profile and runtime admission. [Format adapters](codec-extension-sdk.md) |
| External extensions | The independent read-only XDP1 trial has a bounded protocol and platform gates. It runs through a separate host. Product extension discovery, installation and activation remain proposed. [SDK trial](../engineering/sdk-trial.md) |
| Long-lived state and concurrency | The current scheduler admits one heavy desktop operation at a time. Cross-launch persistent indexes and task recovery, and generalized shared scheduling, remain design options. [Task scheduling](task-scheduling.md), [File publication](file-security.md) |

For current source capabilities, see the [format support table](../../README.md#format-support). For a released build, use its release notes and artifacts. Native implementation, experimental tools and measured results have their own scope.
