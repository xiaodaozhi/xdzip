# Explicit content preview

Preview is initiated when the user clicks “Preview content” in the properties card. Selecting an entry or opening an archive does not automatically decode content. The main window and independent properties viewer use the same component; passwords reuse the existing password window. Metadata details remain independently available.

## Capabilities and budgets

Only readable regular files with known sizes and no dangerous names are supported. The text-extension allowlist is txt, md, log, json, csv, tsv, toml, yaml, yml, ini, xml, rs, c, cpp, h, and py. Only strict UTF-8 or UTF-16 with BOM is accepted. Input and converted UTF-8 are each limited to 256 KiB; binary control bytes such as NUL are forbidden. Content is displayed in a PlainText control; tags, URLs, program text, and macros are not executed.

| Image type | Scope and actual semantics across three platforms |
| --- | --- |
| PNG, JPEG (jpg, jpeg, jfif, jpe) | Fully read one static image and apply available transformation metadata for orientation |
| GIF, WebP | Static image or first animation frame; animations do not play |
| BMP, ICO/CUR | Ordinary image or decoder-default image; CUR does not activate a system cursor |
| ICNS | PNG or JPEG2000 embedded images supported by an available system decoder, at most 256 blocks; unknown embedded types and legacy raw bitmap/RLE icons are explicitly rejected |
| TGA | Uncompressed true-color 16/24/32-bit images without color map or image ID, originating on the left; pad a missing legacy footer for the same layout; no RLE/indexed color |
| TIFF (tif, tiff) | First page only; pages are not automatically traversed |
| SVG | UTF-8 static vector rasterized after restricted parsing; see the subset below |
| HEIC/HEIF | macOS only, using the official Qt plugin with system ImageIO; explicitly rejected elsewhere |

AVIF, RAW, PSD, EXR, PDF, Office, and independent third-party decoder libraries are not added. File extension, Rust signature checks, and fixed Qt decoders jointly determine admission. Extra system-installed plugins do not become product capabilities, and other decoders are not automatically probed.

Encoded image content is limited to 16 MiB. Rust event queues and the Qt mailbox each have a 32 MiB limit; the command budget remains 8 MiB. Original width and height are each at most 16,384, with at most 33,554,432 total pixels. Source admission and output thumbnail are accounted separately: final output width and height are each at most 512, aspect ratio is preserved, and RGBA8 is at most 1 MiB. Photos such as 12 MP images can be read as thumbnails within budgets. Some decoders read the original before scaling, so thumbnail size does not prove internal usage of only 1 MiB. Available color profiles are converted to sRGB; unknown profiles retain decoder results. Professional color proofing is not promised.

The Qt asynchronous image provider receives only random tokens, not system paths or network URLs. At most 32 MiB of encoded content is retained globally and at most two decode tasks run in parallel. Canceled tasks still being reclaimed may each retain at most one 16 MiB input. Each task launches the fixed adjacent private `xdzip-image-preview` worker, without passing archive paths, passwords, destination handles, or application credentials. Worker output permits only a fixed-size header and RGBA8 whose width, height, and length strictly match, with no extra output. Failure, crash, invalid frames, timeout, and late results cannot be displayed.

The host imposes a five-second total deadline from launch and checks request validity every 50 ms. Closing, canceling, or changing entries terminates and reaps the worker. The worker limits QImage allocation to 128 MiB. Windows uses a Job with 512 MiB process commit memory and three seconds of user CPU. Unix uses three seconds of CPU, disables core dumps, and bounds file descriptors. Linux limits address space to 512 MiB and regular-file size to zero. macOS adds a 512 MiB address-space allowance beyond existing dyld virtual reservations; this is not an RSS hard limit. System ImageIO may use file-backed runtime resources, so each regular file is limited to 16 MiB; overages still terminate decoding. The application exports no archive-entry copy; system runtime resources and entry output are managed separately. This is crash and resource isolation for a private decoder, not a kernel permission sandbox for arbitrary third-party programs.

Each archive content read is limited to 256 MiB cumulative source reads, 64 MiB decoding and skipping work, and 256 MiB auxiliary algorithm work. Format dictionaries, workers, and method graphs remain subject to the separate budgets in the [format backend contract](native-codecs.md). A solid archive or sequential compressed stream may refuse preview of a small file if preceding decoding exceeds the work budget.

## Static SVG subset

SVG uses the same Qt Svg rendering path on all three desktop platforms. Before rendering, a streaming XML parser reconstructs supported static graphics, text, styles, internal definitions, gradients, clipping, masks, and restricted filters. Limits are 8,192 elements, 64 levels, 64 attributes per element, 1 MiB per attribute, and 16 MiB total output. Metadata does not participate in rendering. Style elements accept text only; combine XML text segments and CDATA before checking the complete CSS so segmentation cannot bypass external-reference restrictions.

Reject DTDs, entities, processing instructions, scripts, events, animation, foreignObject, image elements, font files, external styles, and external resource references. href and CSS url accept only simple fragment identities in the current document. xml:base, src, CSS imports, and escaped forms are forbidden. Embedded bitmaps and SVGZ are outside this subset. Unknown drawing elements or forbidden content fail the whole preview instead of being silently dropped and presented as complete. This supports ordinary static SVG without claiming all SVG/browser capabilities.

## Integrity, passwords, and lifecycle

Preview reuses frozen physical identity, source version, payload location, and existing decoders. Content must be completely decoded to EOF and complete the format's available integrity and authentication checks before text or encoded images reach preview. Formats such as TAR without per-file content checksums retain only their existing structure and readability guarantees. Disk-backed large indexes use bounded selected-payload reads without downloading all metadata to Qt.

Passwords bind only to the current request and use the existing zeroizing container; errors, authentication ambiguity, and cancellation follow existing categories, with at most ten attempts. Closing the password window cancels only preview and retains the usable directory. Closing preview, changing entries, hiding an archive page, or closing a session clears content; invalidated requests and decode results cannot restore old content. Failure and cancellation discard unverified partial content, do not write an entry copy, and do not launch a system application. Double-click temporary opening follows its [separate flow](archive-opening.md).

## Toolchain and acceptance

`desktop.py prepare --install-qt` adds the pinned Qt 6.8.2 qtimageformats to new or existing SDKs; doctor checks Svg and required plugins before building. Official sources, digests, notices, and SDK correspondence are included in shared configuration. Qt's bundled libwebp/libtiff and separately added format libraries are managed independently. `check-image-preview.py` verifies the actual worker, plugins, and minimal decoding in default tests, Windows/Linux deployment verification, and authorized Mac packaging. Mac explicitly includes offscreen and clears SDK search overrides before self-check; development SDK plugin directories cannot substitute for testing the actual runtime package.

Regression uses project-owned images covering each type, first animation frames, first TIFF pages, transparency, 12 MP photos, macOS HEIC and rejection on other platforms, SVG internal definitions and external-resource rejection, damaged input, pixel overages, invalid frames, timeouts, and cancellation. Real archive paths continue covering passwords, integrity, source identity, and UI invalidation cleanup. Layout follows the [visual standard](../visual/design-standard.md#主动内容预览); reuse unchanged layout evidence under [verification](../qa/README.md) and observe actual display of newly added images.
