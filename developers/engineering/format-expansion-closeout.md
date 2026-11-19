# Format expansion closeout and handoff

English | [简体中文](../zh-CN/engineering/format-expansion-closeout.md)

This guide explains how to assess and deliver new archive formats by checking implementation scope, environment requirements and verification evidence separately. A format name, a generated registry row or a successful self-read does not by itself establish support.

## 1. Freeze the scope ledger

At the start and at the final review, record the issue, delivery branch and remote tip. For every candidate, use one explicit state:

| State | Meaning | Required evidence |
| --- | --- | --- |
| Admitted | The product exposes the declared capability | Codec, runtime, registry, documentation and targeted automated checks |
| Excluded | Product decision removes the format from this work package | Decision recorded in the issue and durable scope document |
| Blocked | Work cannot proceed until a named external gate clears | Gate, owner or required input, and the exact missing evidence |
| Deferred | Work is intentionally left for a later package | Follow-up issue or explicit later milestone |

Do not promote a candidate because a suffix, a reference tool, a parser stub or a partial sample exists. Keep payload, creation, modification, execution and installation capabilities separate.

## 2. Reconcile the working tree before making claims

Run all of the following and inspect the output, rather than relying on the short status flag alone:

```text
git status --short --branch
git diff --name-status
git diff --cached --name-status
git diff --check
```

For every path marked modified, inspect the actual diff. If the textual diff is empty, compare the working-tree object with `git hash-object` and the committed object with `git rev-parse HEAD:<path>`. Resolve line-ending or stat-only noise before reporting an uncommitted change. Preserve real unrelated edits, but list their exact paths and never describe a status-only artifact as pending implementation.

Before committing, compare the staged paths with the issue scope. Inspect the result with `git show --stat --summary HEAD`, then verify the landed revision on remote `main` after integration. If a remote handoff branch is part of the required delivery, check its remote revision as well.

## 3. Preflight the real environment

Check prerequisites before running a long batch:

- the product-owned native worker exists at the path used by the test and deployment entry point;
- `XDZIP_NATIVE_CODEC` and `XDZIP_NATIVE_CODEC_WORKER` are set only when the command requires the real worker, and the selected binary is identified by path and digest;
- the compiler and platform tools used by the command are available in the same shell, including `dumpbin` for Windows PE dependency checks;
- temporary directories used by native workers are private and writable by the test identity;
- reference tools and fixtures are present, pinned and distinguished from product binaries.

Record each prerequisite as `PASS`, `IGNORED`, `BLOCKED` or `NOT RUN`. A missing optional fixture is not a pass, and a test that was not executed through the real worker must not be reported as a worker-path pass.

## 4. Run the smallest complete validation matrix

Use the matrix appropriate to the changed scope, retaining the command and result in the issue or local evidence:

1. codec unit and malformed-input tests;
2. runtime targeted tests through the actual worker;
3. runtime full-library tests with the worker configured;
4. registry generation, format-inventory and source-gate checks;
5. documentation links and generated-output checks;
6. CLI and deployment audits with the platform dependency tools;
7. cross-platform or manual acceptance only where the issue or contract requires it.

Separate product regressions from missing tools, missing fixtures, ACL failures and unavailable platforms. Do not turn `ignored`, `skipped`, `not run` or `environment failure` into a passing count.

## 5. Close the handoff in one order

Complete the implementation and required checks. Commit the scoped source, tests and durable documentation. Follow the [repository delivery agreement](repository-hygiene.md) to integrate and push `main`, verify the landed revision on the remote, and clean temporary branches, worktrees and cycle artifacts. Record the actual revision, final scope, evidence and exclusions before closing the issue.

When the user specifies a local, branch or PR handoff, or a required external check blocks integration, record the recoverable commit and remaining work at that boundary. A pushed delivery branch remains a checkpoint.

Issue comments and reports must not include credentials, complete local paths, lock-screen state or raw temporary-environment details. Formal release approval remains a separate decision.

## 6. Resume after interruption

When a session is resumed, reread the issue's latest comment, branch and remote tip, then compare the current tree with the last delivered commit. Reuse existing commits and evidence. Do not restart implementation because a different process, shell or model forgot the previous handoff. If a claimed result cannot be reproduced, downgrade the claim to `NOT RUN` or `BLOCKED` until the exact command is rerun.
