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
3. Use a full-field CSV for bulk imports. Pilot new import behavior on a small number of locales, then verify intended changes and existing fields before expanding it. A previous import with only caption rows cleared product features and keywords and left the captions unwritten. Restore and recheck any lost data before submitting.
4. Save in batches and record completed locales, failures and remaining work. Confirm that each save succeeds before moving to the next batch. After a timeout or interruption, read the actual state before correcting and resuming, so captions are not assigned to the wrong images.
5. Once all target locales have been reviewed, check the package version, validation status and publishing choice, then submit for certification. A listing marked Complete by the portal does not establish that the images, captions and other materials required for this submission have been reviewed. Confirm that the page enters certification after submission; report publishing progress according to its actual state after certification succeeds and publishing begins.

For manual publishing, select **Publish now** after certification. Microsoft signs and distributes the package, and the installed Store build can then be checked on Windows.

## Bulk listing updates

The [locale map](../../../specs/marketing/windows-store-locales.json) records the 337 observed listings by source language. Reconcile any difference against the draft export and the Partner Center language selector before changing the map. The generator also checks the source languages against the MSIX manifest and the marketing specification.

Export the entire draft to a UTF-8 CSV and keep that file for recovery. Use the [listing generator](../../../scripts/prepare-windows-store-listings.py) with the verified MSIX. It preserves all populated field rows, default values and properties it does not own. Confirm that the marketing specification has the intended release notes, or supply `--release-notes` with a JSON object keyed by the ten source locales. `--materials` optionally verifies the source artwork in a reviewed export against its manifest. The plan records the changes and input hashes under `build/`.

```sh
python scripts/prepare-windows-store-listings.py prepare --baseline build/store-before.csv --package build/xdzip-windows-x64.msix --output build/store-listing-update
python scripts/prepare-windows-store-listings.py verify --baseline build/store-before.csv --intended build/store-listing-update/listing.csv --actual build/store-after.csv
```

Import the generated `listing.csv` into the draft. The default updates text and captions using the screenshot URLs in the export. To update only captions, add `--images-only`. This option still includes all field rows in the CSV. To pilot an update in a few locales, add `--only` with their locale codes. If the artwork changes, upload the four reviewed promotional images for each affected listing, check the images and their order, then export again before generating the text and caption update. Use a fresh export as the baseline for each operation and resolve any locale mismatch or missing image URLs before generating the CSV. Keep the output CSV and JSON in the local work directory.

The [three-locale draft pilot](https://github.com/xiaodaozhi/xdzip/issues/440) confirmed that the full-field CSV preserved existing listing data and saved a changed caption. With `--reuse-source-urls`, Partner Center saved four new URLs for `fr-ca`, which served the same images as `fr-fr`. The exported URLs for `zh-hant-hk` and `zh-hans-sg` stayed the same. Keep URL reuse limited to draft experiments. Verify the actual images and their order when testing this option, since the exported URLs alone do not show which images are displayed.

Re-export after the import. The verification command compares all fields in the draft, including locales omitted from the upload. A discrepancy in image URL, caption, previously populated field, or locale blocks expansion and certification. Read the reported fields in Partner Center and restore affected values from the saved export if needed. [Microsoft's MSIX import and export guide](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/import-and-export-store-listings) explains how to reuse hosted assets and how empty cells affect existing content.

## Evaluate the submission API

The Windows Store submission API requires a [Microsoft Entra application associated with Partner Center](https://learn.microsoft.com/en-us/windows/uwp/monetize/create-and-manage-submissions-using-windows-store-services#how-to-associate-an-azure-ad-application-with-your-partner-center-account) with the Manager role. For an individual developer account without an associated tenant, confirm eligibility with [Microsoft Store developer support](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account#need-help-contact-us) before setting up a tenant or application. The CSV workflow can be used while API access is being evaluated.

Once access is available, load `PARTNER_CENTER_TENANT_ID`, `PARTNER_CENTER_CLIENT_ID` and `PARTNER_CENTER_CLIENT_SECRET` from protected storage. Read an existing submission and compare the response with a CSV export from the same submission. To check a response already obtained with the official CLI, use `--submission-json` instead of `--product-id`.

```sh
python scripts/prepare-windows-store-listings.py api-report --export build/store-published.csv --product-id STORE_PRODUCT_ID --submission-id SUBMISSION_ID
```

The report checks region codes, text and screenshot descriptions in order, and records whether the API reports `runFullTrust`. Before adopting API publication, use a separate draft created and edited via the API to test writing and reading back the locale set, image content and captions, and the restricted capability justification. Reuse an already verified MSIX. If a Partner Center draft is open, limit the API evaluation to reads until that draft is resolved. The top-level `msstore publish` command can replace a pending draft. [Microsoft’s submission API guide](https://learn.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions) and [CLI commands](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/commands) describe the available operations.
