# Build and submit the Microsoft Store MSIX

English | [简体中文](../../zh-CN/market/windows/build.md)

Set `XDZIP_STORE_MSIX_NAME`, `XDZIP_STORE_MSIX_PUBLISHER` and `XDZIP_STORE_MSIX_PUBLISHER_DISPLAY_NAME` in the GitHub repository's **Settings → Secrets and variables → Actions → Variables**, using the values from **Product identity** in Partner Center. The workflow stops before building if any value is missing.

For a Store submission, first commit the increased `build` in `VERSION.json` and the synchronized version files. On GitHub, open [Windows Store MSIX](../../../.github/workflows/windows-store-msix.yml), select **Run workflow** on `main`, and enter the full commit SHA from `main`. Download the `windows-store-msix-<package-version>-<attempt>` artifact. It contains the unsigned `xdzip-windows-x64.msix` and receipts for the build, tests, runtime verification and package inspection.

Check the commit, package version and SHA-256 in `package-msix.json` and `windows-msix.json` before uploading the MSIX to Partner Center. Confirm that the package version is higher than the published Store version. Upload the package, wait for validation to finish, and then update the Store listings.

See the [Store materials guide](README.md) for listing fields and artwork.

When working in Partner Center, prefer the browser extension plugin for submissions, listing fields and material uploads. Use the computer use plugin when the extension does not support the required operation.

## Listing review and certification

1. Use the [locale rules and materials guide](README.md) to list the locales to update and their text and artwork sources. Confirm that the materials match this version; existing images may be retained when still accurate. Use the four 1920 × 1080 combined promotional images for each language in the desktop screenshots section.
2. For each target locale, check the description, release notes, product features, short description, keywords and other fields required for the submission. Check the count, language, order and corresponding captions of the four images. When reusing materials across a language family, confirm the mappings for Simplified Chinese, Traditional Chinese and other languages.
3. Before a bulk import, test it on a small number of locales. Check both the intended fields and existing fields after import before expanding it. A previous captions-only CSV import did not populate captions and cleared product features and keywords; do not assume omitted columns remain unchanged. Restore and recheck any lost data before submitting.
4. Save in batches and record completed locales, failures and remaining work. Confirm that each save succeeds before moving to the next batch. After a timeout or interruption, read the actual state before correcting and resuming, so captions are not assigned to the wrong images.
5. Once all target locales have been reviewed, check the package version, validation status and publishing choice, then submit for certification. A listing marked Complete by the portal does not establish that the images, captions and other materials required for this submission have been reviewed. Confirm that the page enters certification after submission; report publishing progress according to its actual state after certification succeeds and publishing begins.

For manual publishing, select **Publish now** after certification. Microsoft signs and distributes the package, and the installed Store build can then be checked on Windows.
