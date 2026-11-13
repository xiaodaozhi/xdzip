# Adaptive filename encoding contract

## Resolution and identity

Decoding occurs in the Qt-independent Rust format layer. It does not use host system code pages, UI language, source-system guesses, file contents, or network services. ZIP raw_name retains original name-field bytes. TAR retains the raw name selected from PAX/GNU with leading `./` removed under existing rules. Physical EntryId is independent of names. decoded_name stores UTF-8 spelling requiring transcoding or an additional encoding declaration; name_encoding retains resolution origin; name_bytes provides the frozen effective name. display_name is escaped display only and cannot be an output path.

ZIP resolves names in this order:

1. When the UTF-8 flag is set, require valid UTF-8 in the original field; do not repair a corrupt declaration using a legacy encoding.
2. Otherwise read Info-ZIP Unicode Path 0x7075 and accept version 1 with a matching original-name CRC32 and complete valid UTF-8. Ignore unknown versions and CRC mismatches as specified. Matching but invalid UTF-8, duplicate valid fields, or damaged field structure are marked for review.
3. Without a usable extension, if the entire undeclared group has valid UTF-8 names, accept unmarked UTF-8.
4. Otherwise apply archive-wide statistical detection to undeclared non-ASCII names, covering GBK/GB18030, Big5, Shift-JIS, EUC-JP, Korean, and other detector-supported character sets. ZIP's Western default guess falls back to its historical CP437 default. When GBK is detected, use the compatible strict GB18030 decoder.

TAR ordinary headers, GNU long names, and PAX paths explicitly marked hdrcharset=BINARY use the same undeclared-name strategy. Ordinary PAX paths require valid UTF-8; detection must not conceal corrupt declarations. Explicit hdrcharset values other than UTF-8 and BINARY are unsupported. TAR does not inherit ZIP's CP437 default. TAR.GZ uses the same name resolution as TAR.

Rules follow [ZIP APPNOTE 6.3.10](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT), [POSIX pax](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/pax.html), and the [WHATWG Encoding Standard](https://encoding.spec.whatwg.org/). A Unicode Path field may be absent from the local header, but a supplied valid value must match the frozen effective name. Payload checking still verifies raw name, local/central headers, CRC, size, password authentication, and stream termination.

CAB retains original name bytes and the UTF-8 attribute. Explicit UTF-8 is strictly validated; other names reuse bounded archive-wide detection without system code pages. CAB backslashes are format directory separators but are normalized to forward slashes only after complete decoding, preventing Shift-JIS multibyte trail byte 0x5C from being split. Raw bytes remain unchanged, and effective paths still pass the same safety and collision checks. Non-Joliet ISO names retain raw bytes and use the same bounded resolution, avoiding host OEM conversion of Rock Ridge names. Joliet/UDF use processor-provided Unicode names and validate them strictly; lossy replacement characters are not treated as original names.

## Detection boundaries and resources

The detector reads at most 1 MiB of complete undeclared non-UTF-8 names, separating records with newlines so truncated multibyte sequences cannot combine across names. Explicit Unicode declarations and invalid extensions are not samples. Whenever a sample contains non-UTF-8 names, consistently apply the archive-level result to undeclared names, preventing coincidentally valid UTF-8 fragments in GBK from splitting one directory into two names. Only when strict decoding of an individual name fails may valid UTF-8 or a local result for that name be tried. ISO-2022-JP guessing is disabled; escape states in names are not interpreted. Strict decoding inserts no replacement characters, removes no BOM, converts no slashes, performs no Unicode normalization, and repairs no paths.

Raw names total at most 16 MiB. A single raw/decoded name is at most 4,096 bytes, and decoded names plus escaped display total at most 16 MiB. ZIP Unicode extension per-entry and cumulative allocations are checked during scanning rather than after the archive ends. Resolution, detection, and per-entry conversion check cancellation. ZIP payloads retain at most another 16 MiB effective-name verification table; later decode cannot change frozen names.

Statistical detection cannot uniquely recover arbitrary short names or mixed legacy encodings, nor identify mojibake already saved as valid Unicode by other software. Explicitly declared Unicode and legacy encoding may coexist in one archive; multiple undeclared encodings are detected on a best-effort basis. There is currently no manual encoding selector. Names that cannot be strictly decoded, have corrupt declarations, or have abnormal paths remain escaped for display and marked for review without granting extraction, temporary opening, export, or modification. Correct identification of every historical archive is not promised. If an archive has unresolved names, their actual directory membership may be unknown, so directory expansion extraction is rejected to avoid silently omitting items from a successful result. Explicitly selected resolved regular files can still be extracted individually.

## End-to-end use and saving

The directory tree, direct-child index, recursive search, directory selection expansion, preflight, extraction, temporary opening, external export, and archive modification use the same effective name. After decoding, still perform PortablePath, Unicode NFD/case alias, duplicate-name, and output-authorization checks. Shift-JIS multibyte trail byte 0x5C is not a directory separator. Except for CAB's format separator, actual decoded backslashes remain rejected.

After planning, additions and moves in regular ZIP, TAR, and TAR.GZ redeclare legacy-encoded, Unicode-extension, and unmarked UTF-8 names as UTF-8 while retaining original locations and skip semantics in the logical move plan. ZIP preserves compressed payloads; TAR uses explicit UTF-8 PAX paths. Rescan after writing, verify each name and content, then perform the existing same-volume replacement. When TAR rewriting removes a BINARY declaration for a PAX path, explicitly fail and preserve the original archive if doing so would change interpretation of other non-UTF-8 PAX text. Encrypted and split archives use the same names for reading/extraction; add/move restrictions remain unchanged.

## Reproducible acceptance

Use project-owned raw-byte fixtures and archives independently encoded with Python's standard library, covering unmarked UTF-8, GBK, four-byte GB18030, Big5, Shift-JIS containing 0x5C, Korean, Russian, ZIP CP437, and Unicode Path. Verify complete names, original bytes, directories/search/selection, preflight, actual output bytes, temporary opening, export, regular archive add/move, AES, and split volumes. Also check damaged declarations, extension priority, path rejection, Unicode/case collisions, cancellation, and budgets.

scripts/check-filename-encoding.py extracts Python-generated ZIP/TAR/TAR.GZ through an actual service probe. Python also independently reads Rust-rewritten archives to check UTF-8 flags/PAX, names, and contents. Test data comes only from source constants, not user archives. Actual results and remaining manual acceptance are recorded in issues and build/CI artifacts, not as pass status in this contract.
