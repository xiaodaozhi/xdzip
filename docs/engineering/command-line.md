# Command-line runtime contract

English | [简体中文](../zh-CN/engineering/command-line.md) | [繁體中文](../zh-TW/engineering/command-line.md) | [Français](../fr-FR/engineering/command-line.md) | [Русский](../ru-RU/engineering/command-line.md)

The CLI is an independent Rust host that calls format and secure I/O services through the same `xdzip-runtime`; it does not depend on Qt, QML, or a display session. 7z, RAR, CAB, ISO, and UDF continue to use bundled isolated workers and do not search PATH for alternative extraction tools. Platform and architecture follow the [support policy](support-policy.md). The standalone Linux CLI does not require X11; running requires system OpenSSL 3 and certificates, while building additionally requires libssl-dev and pkg-config.

Starting with `v0.2.0`, official releases provide standalone CLI packages on three platforms, and desktop packages include a private CLI. Earlier release assets do not change retroactively. See the [releases](https://github.com/xiaodaozhi/xdzip/releases) for downloads and the sections below and [release protocol](release-protocol.md) for source build, installation, and update boundaries. Implementation and acceptance progress are tracked in [#254](https://github.com/xiaodaozhi/xdzip/issues/254).

## Usage and output

```sh
xdzip --help
xdzip version
xdzip list archive.zip --json
xdzip extract archive.zip docs/ -C output
xdzip extract archive.zip --entry-id 3 -C output
xdzip create backup.7z docs/ --level normal
xdzip create data.iso docs/
xdzip create portable.sfx.exe docs/ --level normal
xdzip create private.zip docs/ --encrypt --password-stdin
xdzip add backup.zip notes.txt --into docs
xdzip test backup.zip
xdzip update --check
xdzip update --download --output downloads
xdzip update --yes
xdzip update --status
xdzip gui archive.zip
```

With no arguments or with `--help`, xdzip shows help. The `version`, `--version` and `--version-json` forms report identity from `VERSION.json` and the build's Git state. `--version-json` emits a standalone object with product, version, build, revision, dirty, component, channel, configuration, distribution, platform and architecture. For help and messages, use `--language zh-CN|en-US`. The default checks `LC_ALL`, `LC_MESSAGES` and `LANG` in that order, and uses Chinese when the selected value starts with `zh`. Public identifiers and error codes are English. `l`, `x`, `c`, `a` and `t` alias `list`, `extract`, `create`, `add` and `test`. Use `--` to pass literal file arguments.

All business commands accept `--json`. stdout emits exactly one JSON object with fixed fields schema_version (1), command, status, error_code, data, summary, and warnings. Unknown sizes use null, including children sharing a compressed TAR stream; raw DEB ar members retain their exact packed sizes. List data includes format and entries; each retains id, path, display_name, size, packed_size, method, directory, link, payload_supported, encrypted, and warning. path is the frozen effective name; invalid or unresolved names are null. display_name is for escaped display only. Task summaries include actual read, decoded, verified, committed, skipped, directory, and volume counts, retaining cleanup warnings and uncertain outcomes.

Progress, errors, and interactive prompts go to stderr. JSON, nonterminal, or `--non-interactive` calls do not prompt. When a password or conflict decision is missing, cancel the waiting task, wait for cleanup, and return interaction_required. Exit codes are 0 (success or explicit handoff), 1 (failure, partial completion, or cleanup warning), 2 (invalid arguments), 3 (missing interaction, authorization, or installation conditions), and 130 (confirmed cancellation). Explicit skip is an executed policy; skipped counts remain separately reported. If cancellation is not accepted and commit has finished, retain the actual completion result.

<a id="归档行为"></a>

## Archive behavior

`list` returns physical records and preserves duplicates. `extract` accepts full, exact and case-sensitive effective member paths using forward slashes. Directories expand recursively and retain hierarchy below the root. A duplicate path returns `ambiguous_member`. Pass the numeric ID returned by `list`, starting at 1, using `--entry-id`. Repeat the flag for multiple IDs. Explicit paths or IDs that cannot be resolved fail before destination directory creation. Wildcards, reading an archive from stdin and writing extracted contents to stdout are unsupported.

Either `--limit` or `--offset` enables pagination. Offset starts at zero and must be at most 1,000,000. With `--limit` alone the offset is zero. With `--offset` alone the limit is 128. Valid limits are 1–4,096. JSON `data` includes `offset`, `total` and nullable `next_offset`. `summary.entries` counts rows actually returned. Ordinary single-file ZIP/ZIP64 can index a million entries, while unpaged listing rejects totals over 100,000. Both modes budget 16 MiB for serialized JSON rows plus rendered text. Exceeding a limit returns `resource_limit` without partial rows. Page boundaries do not change physical IDs. See [large archive indexing](large-index.md).

extract defaults to the current directory. If `-C` names a missing directory, obtain authorization component by component from the nearest existing safe parent rather than unconditionally creating directories recursively. Destination, source identity, links, Unicode/case collisions, staging, verification, and commit continue to follow [safe extraction](zip-safe-extraction.md) and [filename encoding](filename-encoding.md). Name conflicts default to Ask; `--conflict ask|skip|rename|overwrite` selects a policy explicitly. `--allow-high-ratio` confirms only the high compression ratio and does not remove hard resource limits.

`create` chooses its profile from the output [suffix](format-registry.md) and requires the destination parent to exist. `--level store|fast|normal|best` is validated against the selected profile. Omit the level or choose `store` for plain TAR and ISO. LHA (`.lzh` or `.lha`) uses LH5 levels 1, 6 and 9 for `fast`, `normal` and `best`. ZIP and 7z support `--encrypt` and `--volume-size`; 7z also supports `--encrypt-names` and `--solid` subject to its profile. Volume sizes accept positive integer bytes or the `KiB`, `MiB` and `GiB` suffixes. Arguments unsupported by the selected profile fail before writing.

Passwords are passed only through hidden terminal input or one UTF-8 line from `--password-stdin`, removing line endings without trimming the password itself. Interactive encrypted creation requires confirmation twice. Reject plaintext password arguments; do not read password environment variables or write passwords to settings or logs. Length and zeroization follow the [password contract](password-extraction.md) and [creation contract](zip-creation.md).

`add` submits a host update and defaults to the archive root. `--into` selects an existing safe directory by archive-relative name. Eligible inputs can add or replace entries. Internal moves are a desktop operation. The CLI shares the [archive modification](archive-transfers.md) checks for eligible formats and records, input protection, temporary rebuild and final replacement. `test` checks the full archive without producing member files. JSON `data.check_kind` reports the strength of a completed check, as defined in [archive checking](archive-checking.md).

Each CLI operation owns an independent runtime and does not join desktop browsing sessions. Ctrl+C/termination signals request cooperative cancellation and wait for terminal state, cleanup, and runtime join; previously committed files remain. Shared-service submit_wait serially acquires only a short state lock and does not wait for queue capacity or change GUI nonblocking admission. Catalog pages contain at most 128 physical entries. SelectMembers count and name budgets retain existing limits; all requests bind immutable session identity.

<a id="构建与命令安装"></a>

## Build and command installation

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
python3 scripts/cli.py package --output build/package/cli
build/cli/bin/xdzip --help
```

This entry point uses the repository-pinned Rust, CMake, platform C++ toolchain, and existing isolated worker without requiring Qt. Windows packaging also requires the existing prepare-windows-runtime.py to prepare the pinned app-local VC++ runtime with its original license. Qt's unified build entry point also builds the CLI into a private cli directory without renaming the GUI executable, application icon, or association entry points.

The bundled macOS CLI executable is in `Contents/Helpers/cli/`. Its ordinary context JSON is in `Contents/Resources/xdzip-cli-context.json`, sealed by the application resource signature. The CLI finds this resource only in the fixed Contents/Helpers/cli layout and continues to verify schema, desktop kind, and the fixed GUI relative path. Missing markers are treated as portable identity; invalid markers are rejected. Repeated builds remove obsolete generated markers under Helpers; do not relax signature verification to address a layout error.

New desktop bundles also declare `worker_relative` in this context and share the desktop format worker. Only fixed platform-relative paths are accepted: the adjacent desktop worker on Windows, `Contents/MacOS/xdzip-native-codec` on macOS, and the build-adjacent or deployed `bin/` worker on Linux. A missing, linked or unexpected worker rejects the context without searching PATH or another copy. Legacy contexts without this field and standalone/managed CLI packages retain sibling-worker lookup. Explicit developer worker overrides retain precedence. Verify deployed lookup with `scripts/check-cli.py --default-worker`; ordinary test overrides alone do not establish it.

After unpacking a standalone package, run bin/xdzip, adding .exe on Windows. Run `bin/xdzip setup` to explicitly install a user-level stable entry point and versioned runtime directory. The Unix default entry point is ~/.local/bin; on Windows it is under xdzip/cli/bin in user LocalAppData. Management directories use platform user-data locations. --bin-dir and --install-dir explicitly override locations. Users add the directory to PATH themselves; the tool does not modify shell startup files. Existing entry points or registration files are never overwritten.

Managed identity requires a valid installation.json. A directory merely named versions without a registration file is still portable.

When removing a running Windows launcher, move the owned copy whose hash still matches into the private .xdzip-retired subdirectory of the same command directory. Registration is immediately removed and the result records retained_launcher. A later explicit setup reaps only copies in bounded receipts that still match their hashes and have exited. Modified files remain and are not overwritten. The CLI retained in the versioned directory can run setup again to restore registration.

The distribution field distinguishes source from release identity. An ordinary source build, even clean and built in Release configuration, supports check and download only. Automatic replacement requires a distribution identity marked by the dedicated Release pipeline, a valid managed installation, and an authenticated target signature together.

The desktop installer's Windows PATH option is unchecked by default and rejects conflicts with other xdzip commands. Mac/Linux may run setup from the desktop package's private CLI; it registers only the CLI inside the desktop installation and does not copy out a separately upgradeable core. `setup --remove` removes only hash-verified entry points and registration files owned by this tool. Version directories and recovery materials remain; user inputs and runtime directories that may still be in use are not deleted.

The stable launcher does not use shell escaping. On Unix it execs the actual CLI; on Windows it waits for the console child and forwards cancellation and exit status. The current runtime directory is selected by an atomic current.json pointer. Initial installation, new-version self-check, or file-commit failure must not produce an installed-success result.

## Updates, desktop handoff, and recovery

update targets the current installation. A standalone CLI updates only the CLI; a CLI bundled in the desktop package updates the entire desktop edition. --check queries, --download downloads and verifies, default operation installs after confirmation, and --status checks receipts and actual identity. --yes confirms only this update and does not replace system authorization. Portable, dirty, source, or unknown-identity builds allow only check and download. Standalone automatic replacement requires a managed, clean Release installation.

The CLI uses fixed public GitHub Releases, channel, and monotonic build rules. Three-platform CLI assets use separate cli-release.json and cli-release.json.sig; the embedded public key verifies raw bytes before size and SHA-256 checks. Missing signatures or untrusted keys provide only manual acquisition information; an invalid signature fails explicitly. The six-package desktop release.json does not include CLI entries, and older clients continue using the original protocol.

Networking uses system certificates and reads standard proxy environment variables by default. --proxy selects a credential-free HTTP/SOCKS5 proxy; --no-proxy connects directly; --mirror selects an HTTPS prefix without credentials, query, or fragment. Original GitHub URLs, redirect hosts, response sizes, timeouts, and download budgets remain bounded. Network, TLS, offline, rate-limit, and no-compatible-new-version outcomes are reported separately; there is no automatic polling or retry.

Standalone updates stage in a user-owned private directory, safely extract through the same archive runtime, verify the complete fixed file inventory and each file digest, check product, component, local platform/architecture, and signed-manifest channel, and run the new version's --version-json self-check before switching the version pointer. Transactions are mutually exclusive and receipts bind target identity. Failure preserves the old pointer or explicitly enters recovery; do not fabricate automatic rollback.

Desktop check and download query only the main program's non-GUI installation-identity entry point; they do not create QApplication or consume installation receipts. Installation intent passes through the existing same-user single-instance channel, accepting only a bounded request directory and candidate version, not arbitrary commands, packages, or trust roots. Request/result files reuse Rust native directory authorization and safe commits. The desktop controller independently rechecks and matches the candidate, reusing About, installer, Sparkle, and the all-task join barrier. Desktop CLI tasks also hold installation-occupancy protection; installation cannot destroy them.

A terminal receiving accepted reports only handed_off. Actual installation and recovery depend on desktop receipts and actual running identity; downloading or handing off does not mean installed. Non-interactive mode rejects desktop installation requiring GUI or system authorization. See the [update contract](software-update.md) for installation security and native acceptance boundaries. Automated, native, and manual acceptance remain distinct.

The `.iso` profile creates uncompressed ISO9660/Joliet data images at level 0. Omit `--level` or use `--level store`. The Windows x64 `.sfx.exe` profile wraps Store/Deflate ZIP using the bundled sibling `xdzip-sfx.exe`. It rejects encryption, solid mode and volumes. Neither supports editing an existing output. See [ISO creation](iso-creation.md) and [owned SFX creation](sfx-creation.md).
