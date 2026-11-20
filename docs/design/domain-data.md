# Domain model and archive data

English | [简体中文](../zh-CN/design/domain-data.md) | [繁體中文（台灣）](../zh-TW/design/domain-data.md) | [日本語](../ja-JP/design/domain-data.md) | [한국어](../ko-KR/design/domain-data.md) | [Français](../fr-FR/design/domain-data.md) | [Deutsch](../de-DE/design/domain-data.md) | [Español](../es-ES/design/domain-data.md) | [Português (Brasil)](../pt-BR/design/domain-data.md) | [Русский](../ru-RU/design/domain-data.md)

The runtime owns the archive session, the completed index and the latest query for that session. Qt keeps a view of the result, and the CLI can read physical records through its separate catalog command. Both submit operations to the same Rust service. [System architecture](system-architecture.md), [Qt/Rust bridge](qt-rust-bridge.md)

## Identity and source version

| Identity or data | Current representation and scope |
| --- | --- |
| `SessionId` | A nonzero runtime identity for an opened archive. A successful new open creates a new session. File paths are used for input and UI routing. |
| `SourceVersion` | The runtime records the regular file's identity, length and available timestamps for each admitted part. It checks the open object and the corresponding path again at relevant read and write boundaries. This detects changes on a best-effort basis. |
| `EntryId` | Identifies a physical archive record within a session. Records with identical names keep distinct IDs. |
| Navigation identity | Represents an indexed directory. A virtual directory synthesized from descendants can appear in the tree or listing without a physical entry ID. |
| `QueryKey` | Combines session, query ID, generation and snapshot revision. Offset addresses a position in this result. |
| Request, task and prompt identity | The service assigns request and task numbers within a connection. A prompt belongs to a particular task. The shared desktop task entry uses the controller's session identity together with the task number. |

The domain types enforce nonzero archive and query identities. At the native bridge, typed Rust values are represented in generated CXX data. QML receives opaque strings for IDs that require all 64 bits. A row number is only a position in a specific query. [Domain identities](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-domain/src/identity.rs), [Source implementation](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/source.rs), [Application sessions](../engineering/application-sessions.md)

## Physical entries and output names

`xdzip-codec-api::Entry` contains the format, physical identity, name representations, size information, file-kind flags, modification information and format-specific method and encryption fields. Raw name bytes, an optional decoded UTF-8 spelling, the encoding decision and escaped display text are stored separately. For output, the runtime validates the effective name through the portable-path policy and the authorized destination. An unresolved name can remain visible while extraction of that record or directory is refused.

The original-size field carries a separate `size_known` flag, so known zero-length files are distinguishable from unknown sizes. Some compressed containers lack a packed size per entry. Method values are interpreted by format. Time data also retains format conventions, such as local DOS time for ZIP, and missing time remains unknown. `payload_supported()` determines whether the current record can be decoded. Output additionally depends on the selected operation, source state, name and filesystem rules. [Entry fields](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/lib.rs), [Filename handling](../engineering/filename-encoding.md), [File authorization](file-security.md)

## Index, query and page lifecycle

The runtime opens the source and completes its bounded scan before publishing the archive and first page. The scan can be cancelled. A bad tail, missing password or source change leaves the operation without a partial published index. Navigation has a separate budget. If the directory tree exceeds it, the runtime can still support root browsing and physical searches. Ordinary single-file ZIP/ZIP64 can hold up to one million records through the private disk path. Other formats and split paths keep their own limits.

`Archive::query_page` accepts directory, text, sort mode, offset and the expected query identity. A new query starts at offset zero with a new generation. The runtime builds a complete result order and keeps one current immutable query per archive. Pages resolve row identities from that order. Searches and sorting retain the archive records in their original identities. Each sort mode groups directories before files, uses deterministic identity tie-breaking, and orders unknown sizes after known ones within a group.

The generic domain `PageRequest` validates up to 256 rows. Desktop runtime transport requests up to 128 rows with a 64 KiB name budget. Qt commits small results in full and keeps at most 1,024 resident rows for a large result. Responses with mismatched session, query, generation, revision or offset cannot replace current data. [Query implementation](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/query.rs), [Listing and sorting](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/listing.rs), [Large archive indexing](../engineering/large-index.md)

## Selections and operation input

The Qt model stores loaded-result selections by ID and large-result selections as bounded ranges within the current result. Select All covers that result. A task can distinguish the entire archive from an explicitly selected set of search results. Rust resolves cross-page selectors against the full query key through `FreezeSelection` for extraction confirmation. Directory expansion resolves actual descendants and deduplicates physical IDs. Exceeding the operation's limit returns an error. The CLI resolves member paths and physical IDs through `SelectMembers`.

Extraction, checking and editing prepare operation-specific jobs with the source and retained metadata they need. Creation receives options, then scans and validates a separate `InputTree`. Metadata preflight can describe potential issues without giving a task output authority. The runtime rechecks inputs and obtains the relevant grant before writing. Once a job owns its selection, changes in the UI do not change its scope. [Desktop model](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/BrowserController.cpp), [Runtime commands](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/service.rs), [Task scheduling](task-scheduling.md)

## Results and data lifetime

| Data | Lifetime |
| --- | --- |
| Current archive, query and disk index | Belong to the service session. Query replacement or session closure releases the associated private data, subject to the resources independently retained by jobs. The disk metadata index is rebuilt when the archive is reopened. |
| Task snapshots | The runtime publishes bounded snapshots with actual status, byte and item counts, cleanup state and a bounded detail list. The desktop keeps task results available through the owning controller for the application session. |
| User preferences | Managed separately from archive indexes and task results. Data retention and privacy are documented in the settings and privacy contracts. |
| Workspaces and journals | Private output staging and relative publication records support cleanup and inspection. A restart does not replay an unfinished archive write. |

[Snapshot fields](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/extraction/mod.rs), [File publication](file-security.md), [Settings and localization](../ux/settings-localization.md), [Data lifecycle and privacy](../engineering/privacy.md)

## Design options outside the current implementation

Incremental publication of searchable indexes, a reusable plan with its own revision and start token, and persistent paged task history are design options. Current commands prepare and enqueue their own bounded jobs, keep one current query and return bounded task snapshots. Persistent recovery would also require a separate source and output validation design.
