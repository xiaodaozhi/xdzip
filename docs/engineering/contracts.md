# Pre-implementation interface contract review

Authoritative interfaces are in design/03, 05, 06, and 08. This page defines freeze checks rather than duplicating signatures. The M1 internal bridge, dual handles, and lifecycle are implemented; see the [M1 runtime contract](m1-runtime.md) for boundaries. Query generations, source-version checks, and creation/extraction tasks have also been implemented; see the [browser contract](readonly-browser.md), [safe extraction](m3-extraction.md), and [creation contract](m4-creation.md). The third-party process protocol has an independent [read-only SDK trial](sdk-trial.md), but is not released as a stable SDK or integrated into the product. The table defines integration gates.

| Boundary | Confirm before work | Failure behavior to test |
| --- | --- | --- |
| UI → bridge | Handle ownership, RequestId, nonblocking submission, rejection during shutdown | Full queue, repeated shutdown, destroyed object |
| bridge → UI | session/query/generation/revision, batched pages, event sequence | Stale responses dropped, integer narrowing failure, terminal state retained |
| runtime → codec | Capabilities rather than extensions, input reader, output sink, cancellation token | Unsupported features, CRC errors, short reads/writes |
| runtime → secure I/O | Authorized root, mapping, staging, commit permit, commit receipt | Link races, full disk, changed overwrite destination |
| plugin → host | Handshake version, frame limits, capabilities, session lifecycle | Malformed/oversized frames, timeout, reordering, refusal to cancel |

Each interface explains input validity, ownership, threading, resource budgets, error categories, idempotency, and cancellation boundaries. Unknown values are separate from zero. TaskId and EntryId are not interchangeable. Selection identity binds entries rather than current row numbers.

CLI and GUI share Rust services. Catalog returns at most 128 physical metadata entries per page. SelectMembers resolves complete effective paths and EntryIds in the same immutable session, then submits existing tasks. List IDs belong only to that scan and are not durable identities after archive changes. submit_wait waits only for the short admission lock; queue-capacity and shutdown rejection rules remain unchanged. The GUI does not expose QML encoding for these internal commands, and the CLI does not write files using escaped display text.

Fixed host metadata uses open_regular_nofollow to reject leaf symbolic links and reparse points, combined with directory authorization and size/identity checks. Existing user-selected archive reading retains its own path-alias semantics. Desktop CLI requests send a bounded request directory through the optional same-user single-instance `cli_protocol=1` extension. The old open protocol remains version 1; an instance without the extension cannot acknowledge CLI handoff. See the [CLI contract](command-line.md) for results, exit codes, and installation boundaries.

M1 freezes the first minimal internal slice; internal contracts may change with implementation in the same commit. A third-party wire protocol is called v1 stable only after two independent implementations interoperate and pass the compatibility suite. Unknown version numbers must be explicitly rejected or downgraded under negotiation rules.
