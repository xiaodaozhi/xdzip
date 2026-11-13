# Host calendar and encoding services for format workers

Rust handles format fields, password compatibility rules, and validation. OS historical time zones, OEM/ANSI code pages, and local C encodings belong to the host environment and are exposed through optional `ReadSource` services. The format layer must not guess them from filename, HostOS, or current UTC offset. Services do not depend on Qt or grant path access or file-publication authority.

## Calendar projection

`Timestamp` explicitly distinguishes FILETIME 100 ns values from the 1601 epoch and unsigned Unix whole seconds from the 1970 epoch. `ReadSource::local_calendar` returns an optional `Calendar`. The default policy for regular files and memory sources is deterministic UTC; the product worker uses OS local time-zone rules through synchronous CXX callbacks. Windows uses system time-zone APIs and Unix uses `localtime_r`, both checking conversion results. An unavailable conversion returns no calendar; uninitialized fields must not be read.

Rust validates year/month/day and hour/minute/second before packing a DOS display value. 7z retains the existing UTC and 1980..2107 display range. RAR applies its [pinned compatibility rules](rar-format-sources.md) for internal modular arithmetic, time overrides, and platform differences before using the host calendar and historical year wraparound. Original precision, unset markers, and full physical headers remain in the private format index; display precision does not overwrite them. RAR's safe-candidate policy displays 0 for unprojectable times rather than copying undefined contents after unchecked OS failure in the reference implementation.

## Controlled encoding mappings

| Service | Input and output | Host boundary |
| --- | --- | --- |
| `oem_to_ansi_wide` | 512 UTF-16 units to 512 UTF-16 units, including NUL | Complete Windows OEM-to-ANSI round trip, preserving actual system mapping and substitution behavior |
| `wide_to_ansi` | At most 512 UTF-16 units including NUL to 512 bytes | Current Windows ANSI code page; do not invent a universal replacement character |
| `wide_to_local_prefix` | At most 128 wide scalars including NUL to 512 bytes | Current Unix C encoding; conversion failure clears and retries according to fixed compatibility policy while retaining the converted prefix |

These methods return lengths excluding NUL. Default implementations without host services return `None`; format callers may use only defined portable fallbacks or explicitly reject, never silently guess a code page. Product callbacks validate dimensions, NUL, and returned ranges; unavailable mappings return stable Unsupported.

The length parameter of Windows [CharToOemBuffW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-chartooembuffw) counts input wide characters and continues converting padding units after NUL; it is not destination-byte capacity. The host first uses `GetCPInfo(CP_OEMCP)` to admit code pages with at most four bytes per unit, then uses a 2,048-byte temporary destination to map all 512 units. Only afterward set byte 511 to NUL and project to ANSI. Reject conversion if this capacity prerequisite cannot be established. Do not receive 512 wide characters into a 512-byte destination or trim input first to evade capacity requirements. Actual OS probes include the complete 511-unit DBCS boundary and an independent direct WinAPI comparison.

512-unit admission, first NUL, OEM compensation conditions, 127-unit and byte truncation, legacy marker protocol, and explicit UTF8 on macOS are Rust format-compatibility policies, not C++ responsibilities. In particular, do not truncate a password before an OS mapping that originally precedes truncation, or arbitrarily change truncation within a DBCS sequence into character-boundary truncation.

## Ownership and failure

All slices are borrowed only during synchronous calls; C++ retains no pointers. Rust callers own the zeroization lifetime of secret inputs and outputs. Bounded C++ temporary wide/narrow buffers are erased on every return path. This does not guarantee erasure inside the OS or of all historical memory copies. Primitives do not log passwords or place them in process arguments or environment variables.

Readers check cancellation before services. Service results grant no extraction-output credit and do not change read, decoded, or written bytes. Failures preserve existing source, cancellation, and quota categories and must not be mistaken for password errors. Only the format layer determines the established password/integrity ambiguity from actual verifier state.
