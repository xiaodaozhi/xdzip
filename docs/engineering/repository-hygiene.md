# Repository content and history management

The source repository retains durable inputs needed to understand, build, and maintain the product. Issues manage task progress and acceptance conclusions; local or CI artifacts manage execution output. “Generated” alone is not a reason to commit.

## Content ownership

| Content | Location | Git rule |
| --- | --- | --- |
| Product source, test code, project-owned minimal regression fixtures | apps, crates, tools, tests, resources | Retain; fixtures need explicit purposes and redistribution basis |
| Design sources, runtime assets, fonts, parameters, necessary provenance inventories | design, specs | Retain as maintainable inputs, not execution evidence |
| Design, architecture decisions, interface contracts, build methods, support policy, general test methods | docs | Retain and update current rules; do not accumulate daily or per-batch execution conclusions |
| Task scope, progress, acceptance summaries, approvals, blockers, follow-up ownership | GitHub Issues, Milestones | Cite exact commits and Actions runs; separate completed work packages from new defects |
| Screenshots, measurements, logs, stage reports, generated reading editions, builds, development packages | build, target, dist, Actions artifacts | Do not commit; follow evidence retention periods |
| Approved release packages, notices, SBOMs, checksums | GitHub Release attachments | Deliver after explicit publication authorization; do not commit large release artifacts to source; do not provide corresponding-source archives |

`docs/` holds only Markdown and necessary image assets. Machine-readable visual parameters, terminology, and dependency inventories are centralized in `specs/` and read by relevant generation/check entry points. `scripts/repository_contents.py` checks Git index directory boundaries.

Use the corresponding-source collector for local review and rebuild verification only when the user separately and explicitly requests source collection. Routine release and acceptance do not run it; it is not a GitHub Release or Actions download attachment. See the [release protocol](release-protocol.md).

`design/html` is an interactive high-fidelity design; `docs/html` is a Markdown-derived reading edition, serving a different purpose. The latter is generated under `build/docs-html/` by default; the former's source and necessary assets remain maintained. Historical design originals and durable dependency provenance materials are not categorically deleted merely because they are binary or generated.

## Routine checks

Before adding documentation, decide whether it will still guide developers after the task. If it answers only what ran, how many passed, or who remains responsible for one execution, record conclusions in an issue with necessary summaries and evidence links; extract reusable methods into specifications. Blank report templates may be retained, but completed instances are not committed.

`check-docs.py` checks content boundaries against the Git index to prevent force-staging generated output. Documentation changes also check local links, JSON, generated design output, and relevant tests. Renaming large files does not bypass rules: explain necessary product assets; put others in artifacts or release attachments. Records contain no credentials, complete private paths, or user files.

When maintaining capabilities, resource limits, window dimensions, or build flows, check README and engineering entry points against code, pinned configuration, and authoritative specifications, clearly distinguishing implementation from future design. Existing manual acceptance is not reset by generating pending templates; work-package completion does not replace release licensing review. After removing execution reports, update links to actual issues/artifacts or general verification methods rather than continuing to label method pages as a particular report. Prefer references to authoritative configuration for constants instead of duplicating sizes or capability tables.

## Branch handoff for issue fixes

When starting or resuming work, inspect the issue body and latest progress, related open/merged PRs, branches, and remote state, then compare current code to identify completed and remaining scope. Continue from usable fixes rather than reimplementing because context was interrupted, devices changed, or commit SHAs differ. Before verification, inspect applicable existing builds, artifacts, and checks. Continue authorized work without introducing separate approvals or fixed forms for these checks.

Each work package has one primary executor and delivery branch. Reuse an existing PR where needed; related issues may share an entry point but record scope and results separately. For cross-device parallel work, record necessary roles, delivery entry points, and handoff status in issues, for example one party fixes/integrates while another verifies a platform. If verification reveals overlapping changes are needed, synchronize progress first to avoid duplicate implementations. Issue comments are not cross-device locks; if duplication occurs, stop overlapping work and compare/integrate existing results.

When a new or split branch/PR is actually needed, explain why, organize it on existing work, and compare code, tests, documentation, and applicable evidence to confirm complete migration before closing superseded entry points. Different commit hashes alone do not imply different functionality. Synchronize and assess concurrent remote changes before proceeding so other devices' progress is not overwritten. Update related records on handoff, scope, or status changes only; logs and reports keep conclusions and necessary evidence links without repeatedly pasting full output.

For source, test, or product-behavior fixes with explicit issue scope, work first on a temporary branch with the `coding/` prefix. Branch names contain the issue number or recognizable scope. Do not leave product fixes awaiting handoff only as uncommitted changes on shared `main`. A branch may cover related issues in one work package, but issues separately record their scopes, verification, and commit relationships.

Before reporting a fix complete, perform local builds and verification appropriate to the change, commit and push source, tests, and necessary design inputs to the temporary branch, then append commit SHA, branch name, actual verification, and unfinished items to the issue, creating a reviewable, recoverable remote handoff. Pushing is not merging, release approval, or a platform manual-acceptance pass.

After branch handoff, integrate the fix into `main` under project convention or merge through a Pull Request. Clean temporary branches and worktrees under the cycle cleanup rules below. Keep issues open when platform acceptance remains; a pushed branch does not replace acceptance conclusions.

<a id="研发周期工件收尾"></a>

## Development-cycle artifact cleanup

`build/` holds only local/CI builds, tests, diagnostics, tool caches, and evidence; it is not source input. At each development-cycle start, record newly created `build/` paths in the issue or work record. On completion:

1. Merge commits and confirm remote branch status.
2. Delete merged local and remote branches and remove unused Git worktrees; directories still referenced by `git worktree list` must not be cleaned.
3. Remove this cycle's dedicated `build/` temporary directories and files, retaining receipts still needed for reverification, pinned local toolchains, and build directories explicitly reused next cycle.
4. Run `git status --short` again to ensure temporary output, logs, and credentials were not left in the source directory.

Preview first, then perform explicitly authorized deletion:

```sh
python3 scripts/cleanup-build.py --path build/<cycle-directory>
python3 scripts/cleanup-build.py --path build/<cycle-directory> --apply --yes
```

For directories not cleaned at cycle end, manually scan by age as a fallback. Scans select only named temporary pipeline directories not used by active worktrees and not protected shared directories; default is preview:

```sh
python3 scripts/cleanup-build.py --stale-days 30
python3 scripts/cleanup-build.py --stale-days 30 --apply --yes
```

Protected directories include `build/desktop`, `build/package`, `build/review`, `build/toolchain`, and `build/test-tmp`. The script refuses to clean these and parents of active worktrees. Use explicit `--path` for temporary directories automatic scans cannot identify. Do not use `rm -rf build`, `Remove-Item -Recurse build`, or equivalent whole-directory deletion. GitHub-hosted runners are temporary and manual Actions runs cannot clean a developer's local `build/`; the local script is this fallback's entry point.

## History cleanup

Rewrite history only with explicit owner authorization, as an exception to the shared-main no-force-push policy; routine commits still prohibit force pushes. First synchronize remote, save the original tip, create a recoverable Git bundle of all refs and a latest file-tree backup, then organize current content. Verify object hashes and modes file by file for source, tests, resources, and design inputs and check the allowlist of changes; passing builds do not prove file completeness.

Use an independent clone to check the complete new main tree, reachable objects, documentation, and required checks. Remote updates require `--force-with-lease` using the exact old tip. If concurrent remote commits appear, stop overwriting, back up, and integrate first. Do not use unconditional `--force` or delete other unauthorized branches incidentally.

Record the new starting point, old tip, backup digests, content migration destinations, and verification conclusions in the maintenance issue; do not recommit cleanup reports to docs. Retain original identities for historical evidence rather than globally replacing old SHAs with new ones. Other branches, PR refs, server caches, and existing clones may still reference old objects; do not claim a main rewrite physically erased all remote copies.

## Migrating existing clones

Prefer recloning in another directory, first retaining the old clone and uncommitted work. Do not merge old main back into new main or removed history returns. For unmerged work, back up, replay individual changes or apply patches, and reverify. Do not directly push old branches over the new starting point.

Reclaim local old objects or reflogs separately and only after confirming backups are recoverable. Backups are not new public branches, tags, or Release attachments and should not be reuploaded to the cleaned repository.
