# Contributing to XDZip

English | [简体中文](CONTRIBUTING.zh-CN.md)

For build commands, source entry points and verification, start with [Build and contribute](docs/contributing/README.md). Original contributions are licensed under the project's [Apache License 2.0](LICENSE). Confirm that you have permission to share your submission, and preserve third-party notices and provenance.

## Workflow

1. Find or open an issue and record the scope and acceptance criteria. Significant architecture changes need an architecture decision record.
2. Start from current `main` and use a short-lived `coding/` branch where appropriate. Preserve other changes in the checkout.
3. Build or verify the affected paths locally before submitting. In a PR, describe the change, the actual checks and any outstanding requirements.
4. Update both language versions of affected documentation, and keep only redistributable test inputs in Git.
5. Follow repository protection and the agreed delivery scope. Maintainer-owned work continues through integration, remote verification and cleanup once its required checks pass. See [repository delivery](developers/engineering/repository-hygiene.md).

Keep each commit focused and use a descriptive subject such as `docs: clarify archive support`, `feat: add an option`, or `fix: preserve selection`.

## Code and tests

Follow the [quality engineering guide](docs/design/quality-engineering.md). Explain ownership, invariants and failure boundaries where useful. For performance changes, record the workload and before/after measurements. Choose verification by scope under the [CI policy](developers/engineering/ci-policy.md) and the [verification guide](developers/qa/README.md).

## Reporting problems

Use the repository issue template and an owned, minimal reproduction. Remove personal information before sharing it. Report vulnerabilities privately under the [security policy](SECURITY.md). Participation follows the [code of conduct](CODE_OF_CONDUCT.md).
