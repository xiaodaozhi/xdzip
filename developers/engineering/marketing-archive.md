# Synchronize the latest marketing assets

English | [简体中文](../zh-CN/engineering/marketing-archive.md)

Store the latest completed export directly in the primary Google Drive account under `xdzip/assets/windows/`, `xdzip/assets/macos/`, or `xdzip/assets/linux/`. Keep the export's language and asset-category directories, filenames, capture provenance, `manifest.json`, and `SHA256SUMS.txt`. Do not upload ZIP archives or create version directories. A background-only export contains no real screenshots; a capture export may include only a subset of languages. See the [Windows capture guide](../market/windows/capture.md) and [macOS and Linux guide](platform-marketing.md).

The local export does not connect to Drive. To synchronize it, use the connected Google Drive app and the intended account. For each platform:

1. List the existing folder tree, including all pages. Save an inventory under `build/` as JSON with a `files` array of objects containing `path` (relative to the platform folder) and the observed Drive file `id`. Stop on duplicate names within a folder. Download the last completed platform `manifest.json` under `build/` if present. Never infer an ID from a name.
2. Create a plan with the actual local export directory, observed inventory and previous manifest:

   ```sh
   python3 scripts/plan-marketing-sync.py --platform macos --input build/macos-marketing --remote-inventory build/drive-inventory.json --previous-manifest build/previous-manifest.json > build/marketing-sync-plan.json
   ```

   For an empty platform folder omit `--previous-manifest`. The command validates the export against its SHA-256 inventory and refuses to replace files not identified by the previous manifest. It emits `create`, `update` and `trash` operations and required directories; it does not upload or delete anything.
3. Create missing directories, then execute data-file creates and in-place updates with the observed file IDs. Do not create a second file with the same name. After all data uploads succeed, move obsolete files listed in the previous manifest to Trash; leave unrelated files alone. Only then update `manifest.json` and `SHA256SUMS.txt` in that order. If anything fails, stop, relist the affected folders and resume from observed state rather than blindly replaying creates.
4. List every affected directory and compare names, file IDs and sizes with the export; read back the manifest and checksum text. Verify remote hashes when the connector exposes them. Drive updates multiple files separately, so a reader may see a partial update while it is running; publish the final manifest only after data operations finish. Do not claim that size checks prove remote byte hashes.

To sync a later capture set, ensure it is the intended complete current export: files recorded in the previous manifest but absent from the new one will be removed. Do not commit generated files or Drive inventory to Git. Run `python3 scripts/test-plan-marketing-sync.py` for the focused planning checks.
