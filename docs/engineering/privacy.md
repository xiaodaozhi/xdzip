# Data lifecycle and diagnostics

English | [简体中文](../zh-CN/engineering/privacy.md)

The [Privacy Policy](../../PRIVACY.md) explains what data xdzip handles and which choices are available to users. The table below describes where the implementation keeps that data and when it is released. Browsing, compression and extraction work offline. xdzip does not automatically send telemetry or archive contents to the maintainer.

| Data | Lifetime | Storage and cleanup |
| --- | --- | --- |
| Settings and themes | Across launches, in a versioned schema | Changes use atomic replacement. If a settings update fails, it preserves the previous settings and the interface reports the failure. |
| Previous default apps on macOS | Local, across launches | When you make xdzip the default, a separate settings file records the previous app's bundle ID for that content type. Restoring or choosing a replacement clears the record for that type. |
| Recent files | Optional, local only | The private `recent.json` file stores up to 20 distinct entries with a 64 KiB read limit. Disabling the feature stops new entries. Clearing it deletes the history without touching the original files. |
| Archive indexes | For the open session | Small indexes live in memory. Large ZIP indexes use private disk files for metadata and query order. Normal close, query replacement, failure and cancellation clean their known files. A later creation can reclaim unused crash leftovers under the identity and lease rules. Indexes are never reused across launches. See [large archive indexes](large-index.md). |
| Staged output and recovery state | During tasks and recovery | Recovery acts only on controlled metadata and cleans only items whose ownership can be established. It does not execute archive contents or restore writes to arbitrary paths. |
| In-app previews | User initiated and bounded | Changing the preview context or ending the session clears the current result. Late results cannot restore invalidated content. |
| Passwords | For the operation that needs them | The app does not save passwords to settings or logs. The operating system may hold additional memory copies beyond the app's control. |
| Standalone update checks and downloads | User initiated in applicable editions | Requests use public GitHub release and download services. A configured proxy can handle requests, and an optional HTTPS mirror receives the original download URL. Providers can see connection information and the application version included in requests. Windows MSIX disables the standalone updater. |
| Update transaction receipts | Local, across installation restarts | A separate `update-state.ini` beside settings holds transaction and target identity. Failed downloads may remain in the private update cache. The updater reads only fixed receipt files and does not execute receipt contents. |
| Status and diagnostics | Interface, terminal or local records, depending on the feature | The app does not automatically upload diagnostics. Copying version or file information writes to the system clipboard. Check filenames, paths and other private information before sharing an error, log or screenshot. |

See the [software update contract](software-update.md) for networking, signatures, installation handoff and recovery.
