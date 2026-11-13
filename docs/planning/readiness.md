# Development and release readiness review

This page defines repeatable review requirements. Completion records, manual conclusions and outstanding work for specific stages belong in [Issues](https://github.com/xiaodaozhi/xdzip/issues); see the [work packages](backlog.md) for the roadmap. Do not append dated status reports here.

## Development baseline

- The technical approach, module boundaries, interfaces and format capabilities agree with the [design specifications](../design/index.md).
- Toolchain, dependencies and provenance are pinned according to lockfiles and the [toolchain policy](../engineering/toolchain.md).
- Rust, C++, Qt/QML and FFI changes receive CI and regression checks appropriate to their scope, with actual limitations recorded.
- Visual changes follow the [design workflow](../visual/design-workflow.md) first; actual native visual acceptance is recorded separately from functional tests.
- Source, tests and design inputs can be recovered from the current repository; generated evidence belongs only in local or Actions artifacts.

## Work-package exit

Each work package defines dependencies, deliverables, acceptance criteria, exclusions and actual evidence in its issue. Track unfinished defects, resource limits and license reviews separately; do not automatically close them because CI passed once. Record owner decisions that adjust exit boundaries without describing unperformed reviews as passed. Historical WP-12 closure is in [#5](https://github.com/xiaodaozhi/xdzip/issues/5).

## Public release

Follow the [release permission review](../qa/release-review.md) and [release runbook](../engineering/release-runbook.md), checking actual packages, source, provenance and licenses, notices, installation and rollback, and support policy. Development completion or manual acceptance does not mean release approval; unverified items retain their actual status. Formal signing and paid accounts are not mandatory prerequisites for this project.
