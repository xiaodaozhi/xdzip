# Build and submit the Microsoft Store MSIX

English | [简体中文](../../zh-CN/market/windows/build.md)

Set `XDZIP_STORE_MSIX_NAME`, `XDZIP_STORE_MSIX_PUBLISHER` and `XDZIP_STORE_MSIX_PUBLISHER_DISPLAY_NAME` in the GitHub repository's **Settings → Secrets and variables → Actions → Variables**, using the values from **Product identity** in Partner Center. The workflow stops before building if any value is missing.

For a Store submission, first commit the increased `build` in `VERSION.json` and the synchronized version files. On GitHub, open [Windows Store MSIX](../../../.github/workflows/windows-store-msix.yml), select **Run workflow** on `main`, and enter the full commit SHA from `main`. Download the `windows-store-msix-<package-version>-<attempt>` artifact. It contains the unsigned `xdzip-windows-x64.msix` and receipts for the build, tests, runtime verification and package inspection.

Check the commit, package version and SHA-256 in `package-msix.json` and `windows-msix.json` before uploading the MSIX to Partner Center. Confirm that the package version is higher than the published Store version. Update the Store listings and submit for certification. In a manual publishing submission, select **Publish now** after certification. Microsoft signs and distributes the package, and the installed Store build can then be checked on Windows.

See the [Store materials guide](README.md) for listing fields and artwork.
