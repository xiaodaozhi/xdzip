# Release licensing review

This page defines reusable provenance and redistribution review methods for each candidate; it does not record progress or approvals for a particular candidate. Historical `v0.1.0` review and release closeout are recorded in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) and [#53](https://github.com/xiaodaozhi/xdzip/issues/53), both closed. Future candidates require a new release issue for the specific version. Completing work packages, passing automated checks and approving a formal release are separate conclusions.

## Candidate and inputs

Bind review to an explicit version, full commit SHA, source package, actual installers for each platform and SHA-256. Provide same-commit SBOM, provenance and notices, offline rebuild and applicable Qt source and replacement verification. Select performance and desktop checks according to actual changes, recording results and uncovered scope. Reassess affected items if source tree or packages change; do not mix binaries and historical reports from different commits. Machine report `automatic_complete` and `release_ready` values are not rewritten because an issue closes or documentation is reorganized.

## Review items

| ID | Scope | Required conclusion |
| --- | --- | --- |
| RR-01 | License provenance and redistribution obligations | Individually check provenance, applicable licenses, notices, corresponding source and replacement materials for actually delivered Qt, Rust, native libraries, fonts and icons. After repairing missing materials, rebuild the actual candidate and checksums. |

The maintainer consolidates and tracks results. When resources or conditions are unavailable, honestly record unperformed work and limitations in the issue; do not automatically treat them as passed or waived, and do not require purchasing review services or a paid signing identity. Valid existing manual acceptance does not expire merely because materials are reorganized.

## Records and handoff

Licensing checks follow [runtime attribution](../engineering/runtime-attribution.md), [corresponding source](../engineering/source-delivery.md) and [SDK correspondence verification](../engineering/qt-sdk-correspondence.md). Matching hashes do not grant redistribution rights or reproduce upstream build history. Adding notices does not mean old packages contained them. Never rewrite delivered packages or their original checksums.

Each review conclusion records its ID, candidate, actual scope and limitations. Record gaps in provenance claims, applicable licenses, notices and corresponding source separately; file hashes or automated checks do not substitute for missing content.

## Release decision

System scope and maintenance follow the [support policy](../engineering/support-policy.md). Before actual release, review dependency security, unresolved defects, notices, source, artifacts and hashes; handle findings and known limitations according to the [release runbook](../engineering/release-runbook.md). Do not create a Release or tag or publicly upload installers without the owner's approval of the specific results.

Paid developer accounts, signing certificates, Developer ID and notarization are not required. macOS retains necessary ad-hoc integrity signing and Windows uses an unsigned installer. This does not approve an incomplete licensing review.
