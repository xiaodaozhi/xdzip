# Architecture evolution and current boundaries

English | [简体中文](../zh-CN/design/architecture-evolution.md)

## From prototypes to the product

Early experiments compared interface frameworks with a synthetic archive list and simulated tasks. Qt Quick was selected for the product. The production application uses a typed cxx bridge and real archive and file operations. The prototype's JSON/C ABI and fixtures explain the technology choice and have no supported integration contract. [Architecture decisions](architecture-decisions.md)

The product keeps the useful design principles of bounded model caches, request generation checks and accessible controls. The production implementation adds real entry identities, format capability checks, secure file authorization and verified output transactions. The [system architecture](system-architecture.md) follows the current code.

## What is available and what is experimental

| Area | Current boundary |
| --- | --- |
| Desktop and CLI | Qt Quick desktop plus a separate Rust CLI sharing runtime behavior. |
| Large archives | Memory and disk indexes live in the runtime. The large ZIP index has separate limits for selected extraction and whole-archive actions. |
| Formats | Built-in routes follow the [format registry](../engineering/format-registry.md). A reference decoder or format specification alone does not grant support. |
| Extensions | The [read-only SDK trial](../engineering/sdk-trial.md) exercises a separate protocol and host. Current product builds disable third-party extensions. |

For current source capabilities, see the [format support table](../../README.md#format-support). For a released build, see the matching release notes.
