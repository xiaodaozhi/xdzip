# Data lifecycle and diagnostics

For user-facing privacy information, see the [Privacy Policy](../../PRIVACY.en.md) and its [Chinese version](../../PRIVACY.md). This page retains engineering implementation contracts and does not treat planned diagnostic designs as delivered features.

By default there is no telemetry, background update installation, or archive-content upload. Basic compression and extraction work offline. Product settings, user content, and diagnostic evidence are stored separately.

| Data | Lifetime | Cleanup and limits |
| --- | --- | --- |
| Settings and themes | Across launches, versioned schema | Save failures can restore defaults; retain a diagnosable copy of corrupt files without reading them as code |
| Recent files | Optional, local only | Separate recent.json holds at most 20 entries with a 64 KiB read limit, deduplication, and user-only read/write. Disabling stops additions; clearing removes records only; paths do not enter diagnostic uploads |
| Memory indexes | Session lifetime | Released on close; not cached across launches by default |
| Staged output/transaction logs | During task and recovery | Clean only items whose ownership can be established; do not delete published files |
| Preview cache | User initiated and bounded | Clear at session end; invalidation cannot keep displaying old content |
| Passwords | Short-lived | No disk or logs; absolute erasure of OS-internal copies is not promised |
| Update checks and downloads | User initiated only | Fixed public GitHub APIs and CDN, without login credentials, archive content, or paths. Servers can see IP, connection information, and application version |
| Update transaction receipts | Local, across installation restarts | Separate update-state.ini beside settings stores transaction and target identity; private cache retains failed materials. Only fixed files are read and receipt content is not executed |
| Diagnostics | Locally bounded, explicitly exported by user | Version/anonymous ID/error code; no content or full paths by default |

Diagnostic records require capacity limits and retention policies configured centrally in the implementation. Show fields to be included before export so users can inspect them before uploading. Crash recovery handles only controlled metadata and does not execute archive content or restore arbitrary-path writes.

See the [software update contract](software-update.md) for networking, signatures, installation handoff, and recovery boundaries.
