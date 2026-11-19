# Project governance

English | [简体中文](GOVERNANCE.zh-CN.md)

Repository owner xiaodaozhi is responsible for project scope, release identity, licensing decisions and maintainer authorization. Reviews follow the affected area, such as the core, UI, platforms, security or release. Maintainer authority comes from the owner's actual delegation.

Changes to accepted architecture use an [architecture decision record](docs/design/architecture-decisions.md) with context, options, the decision, trade-offs and superseded decisions. Product scope changes update the relevant requirements. Resolve disagreements using reproducible evidence and documented safety constraints. The owner makes the final scope decision.

`main` is the reviewable baseline. Use PRs where useful or required by branch protection. The [working agreement](AGENTS.md) and [repository delivery guide](developers/engineering/repository-hygiene.md) define how authorized changes reach `main` and how to verify the landed revision and clean up.

Security-sensitive changes require evidence appropriate to their scope. Release decisions are specific to the candidate and use the [maintainer runbook](developers/engineering/release-runbook.md) to check verification, supply chain and platform evidence.
