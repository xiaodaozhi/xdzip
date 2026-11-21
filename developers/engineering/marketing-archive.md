# Package marketing assets

English | [简体中文](../zh-CN/engineering/marketing-archive.md)

Package each completed Windows, macOS or Linux marketing export separately. The export can contain backgrounds only, or screenshots and composites for the languages actually captured. See the [Windows workflow](../market/windows/capture.md) and [macOS and Linux workflow](platform-marketing.md) for generation and capture.

```sh
python3 scripts/package-marketing-assets.py --platform windows --input build/windows-marketing
python3 scripts/package-marketing-assets.py --platform macos --input build/macos-marketing
python3 scripts/package-marketing-assets.py --platform linux --input build/linux-marketing
```

Use the actual generated directory for each `--input`; Windows screenshot exports from `package-windows-store-materials.py` are also supported. Packages default to `build/marketing-packages/`; `--output-dir` selects another directory under `build/`, `dist/`, or outside the repository. The exporter preserves the original language directories, screenshots, backgrounds, illustrations, copy, capture provenance, manifest and checksum file.

The filename is `xdzip-<platform>-marketing-<backgrounds|captures>-<manifest-hash>.zip`. `backgrounds` indicates that the export has no recorded captures; `captures` means at least one language has captures, not necessarily all ten. The hash identifies the export manifest. Existing package names are never overwritten. The command checks the platform, inventory and per-file SHA-256 before writing, then checks the ZIP entries, checksums and CRC after writing. It prints the package SHA-256 for comparison when archiving or transferring.

Archive the resulting ZIPs in `xdzip/assets` in the designated Google Drive account. With multiple connected accounts, use the account designated as primary. The packaging command only creates local packages; ask Codex to upload the ZIPs through the connected Google Drive app, or upload them using Drive. Check the account, folder, filenames and sizes after upload. Do not overwrite a different existing file. The local package digest is printed for comparison when the upload interface exposes a checksum. Do not represent background-only packages as real screenshots or submit generated exports to Git.

Run `python3 scripts/test-package-marketing-assets.py` for the focused packaging checks.
