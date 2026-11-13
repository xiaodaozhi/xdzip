# Command-line runtime contract

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
xdzip create private.zip docs/ --encrypt --password-stdin
xdzip add backup.zip notes.txt --into docs
xdzip test backup.zip
xdzip update --check
xdzip update --download --output downloads
xdzip update --yes
xdzip update --status
xdzip gui archive.zip
```

No arguments displays help. `version`, `--version`, and `--version-json` use VERSION.json and build-time Git identity. Existing version strings and JSON product, version, build, revision, and dirty fields are retained; the CLI additionally identifies component, channel, configuration, distribution, platform, and architecture. `--language zh-CN|en-US` chooses help and result language, defaulting from desktop locale environment variables. Public identifiers and error codes are English. `l`, `x`, `c`, `a`, and `t` alias list, extract, create, add, and test, respectively. Arguments after `--` are literal file arguments.

All business commands accept `--json`. stdout emits exactly one JSON object with fixed fields schema_version (1), command, status, error_code, data, summary, and warnings. Unknown sizes use null. List data includes format and entries; each retains id, path, display_name, size, packed_size, method, directory, link, payload_supported, encrypted, and warning. path is the frozen effective name; invalid or unresolved names are null. display_name is for escaped display only. Task summaries include actual read, decoded, verified, committed, skipped, directory, and volume counts, retaining cleanup warnings and uncertain outcomes.

Progress, errors, and interactive prompts go to stderr. JSON, nonterminal, or `--non-interactive` calls do not prompt. When a password or conflict decision is missing, cancel the waiting task, wait for cleanup, and return interaction_required. Exit codes are 0 (success or explicit handoff), 1 (failure, partial completion, or cleanup warning), 2 (invalid arguments), 3 (missing interaction, authorization, or installation conditions), and 130 (confirmed cancellation). Explicit skip is an executed policy; skipped counts remain separately reported. If cancellation is not accepted and commit has finished, retain the actual completion result.

<a id="归档行为"></a>

## Archive behavior

list returns physical records without merging duplicate names. extract member arguments are complete, exact, case-sensitive effective archive paths using forward slashes. Directories expand recursively and retain hierarchy below the root. Exact duplicate names return ambiguous_member; use --entry-id to select a physical record. Any unmatched member, foreign ID, or ambiguity fails before creating a destination directory. The first version does not interpret wildcards or read archives from stdin or send extracted content to stdout.

List large archives using `list archive.zip --offset 999872 --limit 128 --json`. Offset is zero-based and limit is 1–4096; specifying offset alone defaults to 128. JSON data also returns offset, total, and nullable next_offset; summary.entries is the number returned. Regular single-file ZIP/ZIP64 supports indexing one million entries. Without pagination, listing is limited to 100,000 records and 16 MiB total serialized JSON rows and text; overages return resource_limit without emitting a prefix that could be mistaken for a complete list. Pagination does not change physical IDs; see [large indexing](large-index.md).

extract defaults to the current directory. If `-C` names a missing directory, obtain authorization component by component from the nearest existing safe parent rather than unconditionally creating directories recursively. Destination, source identity, links, Unicode/case collisions, staging, verification, and commit continue to follow [safe extraction](m3-extraction.md) and [filename encoding](filename-encoding.md). Name conflicts default to Ask; `--conflict ask|skip|rename|overwrite` selects a policy explicitly. `--allow-high-ratio` confirms only the high compression ratio and does not remove hard resource limits.

create selects an existing backend from .zip, .tar, .tar.gz, .tgz, or .7z; the destination parent must already exist. `--level store|fast|normal|best` maps to the existing four levels, defaulting to normal; TAR only packs files. `--encrypt` applies only to ZIP/7z; 7z additionally supports --encrypt-names and --solid. --volume-size accepts positive integer bytes and KiB, MiB, GiB suffixes; split volumes apply only to ZIP/7z. Reject format-inapplicable arguments explicitly.

Passwords are passed only through hidden terminal input or one UTF-8 line from `--password-stdin`, removing line endings without trimming the password itself. Interactive encrypted creation requires confirmation twice. Reject plaintext password arguments; do not read password environment variables or write passwords to settings or logs. Length and zeroization follow the [password contract](password-extraction.md) and [creation contract](m4-creation.md).

add defaults to the archive root; --into selects only an existing safe directory. Encrypted or split archives, RAR, CAB, ISO, UDF, and other archives that do not meet editing conditions remain rejected. Regular archive rebuild/replacement, the 7z working-set limit, macOS metadata filtering, and input protection reuse [archive modification](archive-transfers.md). test checks the entire archive without output files. TAR, ISO, and UDF guarantee only structure and readability; other formats guarantee only checks they actually provide. Unsupported capabilities are not described as passed.

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
