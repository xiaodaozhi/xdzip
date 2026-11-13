# Risks and decisions

This register defines risks and controls that require ongoing assessment. Actual status, owners, deadlines and closure evidence belong in the corresponding issue; do not append stage reports here.

| ID | Risk / decision | Responsible role | Closure evidence / gate |
| --- | --- | --- | --- |
| D01 (closed) | Apache-2.0 for original content | Repository owner | License policy established 2026-10-11; root LICENSE and licensing documentation synchronized |
| D02 | Release toolchain freeze and OS support | Core/platform | Check provenance, maintenance channels and actual target environments against [toolchain](../engineering/toolchain.md), [support policy](../engineering/support-policy.md) and [release review](../qa/release-review.md) |
| D03 | ZIP/TAR backend versions | Format maintainer | Interoperability, cancellation, licensing and malformed input; WP-05/11 |
| D04 (closed by user decision) | Distribution | Owner/release | GitHub Releases without formal signing decided 2026-10-19; no paid account, certificate or notarization required. Retain necessary ad-hoc integrity signing and artifact verification; actual uploads still require explicit authorization |
| D05 | Applicability of performance results | Performance maintainer | Record hardware, system, backend, workload and budget for each measurement; observations on a shared runner or a single development machine are not guarantees for all devices |
| D06 | Vulnerability and community feedback channels | Owner | Receive and handle actual feedback under the [security policy](../../SECURITY.md); do not put sensitive content in public issues |
| R01 | cxx and Qt shutdown races | Core/Qt | Close, full-queue and post-destruction event tests; WP-03 |
| R02 | Inconsistent path/link semantics across three platforms | Platform security | Native race tests and no changes outside output; WP-07 |
| R03 | Complete Qt size exceeds budget | Release/performance | Measure with all runtime libraries; remove unused dependencies rather than prematurely static-linking |
| R04 | Plugin sandbox not yet viable | Extension maintainer | Platform authorization-boundary suite; remain disabled until passed |
| R05 | Designs mistaken for full first-release functionality | Product/Qt | First release uses details instead of preview, with explicit capability wording; WP-04/12 |

Risks may be closed only on evidence. Reduce capability scope when security or licensing gates cannot be met; complete documentation alone does not mean a risk is resolved.
