# Store review notes

English | [简体中文](../../zh-CN/market/windows/reviewer-notes.md)

The following text is intended for the restricted-capability explanation and certification notes for the MSIX candidate identified in [the listing](listing.md). It does not represent approval of the capability or a submitted certification request.

## runFullTrust explanation

```text
xdzip is a packaged x64 desktop archive manager using Qt Quick and Rust. Its application entry point is Windows.FullTrustApplication. The runFullTrust capability is needed to run this existing desktop application and use ordinary desktop file APIs to browse, create and extract archives selected by the user.

The application reads selected archives and writes files to a destination chosen by the user. It also starts bundled worker processes for supported archive formats and bounded content previews. Temporary files support staging, extraction and user-requested opening in the system's associated application.

The application does not require an administrator account or request elevation for normal archive operations. It does not install a service or driver. The Store build declares separate modern and classic File Explorer menu classes. Modern commands are enabled only when the registered package has a Microsoft Store signature; an unsigned or self-signed review copy keeps the modern entry hidden. Its separate in-app updater is disabled when running with MSIX package identity; Store-distributed updates are managed by Microsoft Store.

Basic archive operations work locally and do not require an xdzip account. Archive contents, paths and passwords are not automatically uploaded to the developer. The public privacy policy is linked in the Store listing.
```

## Certification notes

```text
No xdzip sign-in, subscription, test credentials or external archive program is required for basic use.

To review browsing and preview, open a ZIP containing ordinary text and PNG files. Select a file and show its properties to view the supported content preview. Open another archive to test tabbed browsing and switch between list and grid views.

To review creation, choose New, add ordinary files and select ZIP. Choose a destination owned and writable by the test account, then create the archive. ZIP, TAR, TAR.GZ and 7z creation are available in this candidate. RAR, CAB and UDF are read-only. ISO images can be created, but existing ISO images cannot be modified. Encryption and split-archive support depend on the supported format and profile.

To review extraction, select files in an archive, choose Extract selected and select a writable destination. Progress and cancellation are available. Opening a file through the system creates a temporary local copy; changes to that copy are not automatically written back to the archive.

To review system integration after Store delivery, open Settings, File associations and use the Windows default-app settings button. Right-click owned test files in File Explorer and check both the modern menu and Show more options. Menu enablement and language follow the app settings. The app does not change default applications automatically.

The listing targets Windows 11 x64. Screenshots use owned demonstration content. The application receives Store-distributed updates through Microsoft Store.
```

The capability is declared by [build-windows-msix.py](../../../scripts/build-windows-msix.py). These notes describe the candidate's existing desktop behavior, including manifest-based shell integration, without additional services or elevation. Pricing, age ratings and other submission questionnaires must be completed in their own Partner Center sections.
