# XDZip design guide

English | [简体中文](../zh-CN/design/index.md) | [繁體中文（台灣）](../zh-TW/design/index.md)

Start with the [system architecture](system-architecture.md) to see how the Qt Quick desktop and the independent Rust CLI reach the archive runtime. The design pages explain the current contracts, decisions and the scope of proposed work. Installation and everyday operations are covered in the [user manual](../user-guide/README.md). To build or change the project, see [Build and contribute](../contributing/README.md).

## Find a design topic

| Topic | What to read |
| --- | --- |
| Product behavior | [Product requirements](product-requirements.md) defines expected operations and their scope. [Requirements traceability](requirements-traceability.md) identifies corresponding verification. |
| Archive data | [Domain model](domain-data.md) describes sessions, source versions, physical entries, queries and operation input. |
| Desktop | [Qt Quick interface](qt-quick-ui.md) covers views, controller ownership and interaction. [Qt/Rust bridge](qt-rust-bridge.md) gives the actual commands, IDs, events and lifecycle. |
| Formats and extension boundaries | [Format adapters and the extension trial](codec-extension-sdk.md) explains the built-in registry, bundled worker and standalone XDP1 trial. |
| Output and background work | [File authorization and safe publication](file-security.md) covers staging, verification and commit. [Task scheduling](task-scheduling.md) explains admission, prompts, progress and cancellation. |
| Performance and quality | [Performance design](performance.md) explains budgets, measurements and targets. [Quality engineering](quality-engineering.md) maps changes to verification. |
| Delivery and decisions | [Build, deployment and release](build-release.md) covers platforms, packages and updates. [Architecture decisions](architecture-decisions.md) records the rationale, and [architecture evolution](architecture-evolution.md) explains how prototype choices led to the current product. |
| Terminology | [Terminology and implementation limits](glossary.md) explains vocabulary and links to the current limits for each subsystem. |

## Interpreting the references

Current behavior is described with implementation links. Design options and experimental tools have their own stated scope. The [engineering reference](../reference/README.md) leads to individual contracts. The [project README](../../README.md) and matching release notes describe supported distribution and delivered features. Timing, resource and native verification claims apply to the recorded build, platform and workload.
