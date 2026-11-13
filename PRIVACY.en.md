# xdzip Privacy Policy

Effective date and last updated: November 13, 2026.

[简体中文](PRIVACY.md)

## Scope and contact

This policy explains how the official xdzip desktop application and command-line tools maintained by xiaodaozhi process information, including their Windows MSIX editions. Features may differ between versions and distribution channels.

For privacy questions, contact the maintainer through the [project issue tracker](https://github.com/xiaodaozhi/xdzip/issues). This is public: provide only what is needed to resolve the issue and do not include private files, passwords or full private paths. For security vulnerabilities or sensitive vulnerability details, use [private vulnerability reporting](https://github.com/xiaodaozhi/xdzip/security/advisories/new).

## Local files and application data

Basic compression, extraction and browsing work offline and do not require an xdzip account. The application reads files and directories you select and processes archive contents, names, paths, sizes, timestamps and other file attributes on your device as needed for your actions. These may contain personal information. xdzip does not automatically upload them to the developer or provide archive synchronization to developer-operated servers.

To remember preferences and support operations, xdzip processes the following locally:

| Data | Purpose and storage |
| --- | --- |
| Settings | Remember language, theme, text size, layout and applicable update network preferences. |
| Recent files | When enabled, remember paths of recently opened archives for convenient access, up to 20 entries. |
| Indexes and previews | Process archive listings, search results and previews you request in the current session. |
| Temporary files and task state | Support safe output, temporary opening, cancellation, cleanup and applicable task recovery and update handoff. |
| Version and error information | Explain versions, task status and failures in the interface, terminal or local diagnostics, with fields depending on the feature. |

When you copy file or version information, it is written to the system clipboard. When you open an archive entry with an associated application, xdzip creates a local temporary copy and passes it to that application, which can access the file. Network shares, cloud-synchronized folders, clipboard synchronization, system backups and other applications are governed by their respective settings and policies.

## Passwords and file protection

Passwords are used for the encryption or decryption operation you request. xdzip does not offer saved archive passwords, write passwords to application settings or logs, or upload them to the developer. The command-line tools receive passwords through hidden input or standard input, not plaintext command-line arguments or password environment variables.

The application processes passwords for as long as the operation needs them and clears the associated memory when no longer needed. Operating-system memory management and crash dumps are not entirely under the application's control, so it cannot guarantee immediate removal of every system copy.

Archive encryption does not mean that extracted files, temporary open copies or recent-file records are also encrypted. Choose output locations, device access permissions and archive passwords appropriate for the sensitivity of your files.

## Network connections and third-party services

xdzip does not automatically send telemetry, usage statistics or archive contents to the developer, track users for advertising, or sell or rent your files and local records. Network activity may include:

- User-initiated updates in ordinary editions: when you request an update check or download, applicable versions connect to GitHub's public release and download services. Providers can see your IP address, connection and request information, and the application version included in requests. Update requests do not include archive contents, paths, passwords, recent-file records or xdzip settings.
- Proxies and mirrors: if you choose a proxy or update mirror, requests pass through that service. A mirror receives the original GitHub download address. What the service can observe depends on how it operates.
- Microsoft Store editions: the standalone in-app updater is disabled when the application has Windows MSIX package identity. Microsoft Store manages updates for Store-delivered editions and may process installation, update and diagnostic information under its policies and your system settings. Sideloaded MSIX packages do not thereby gain an in-app automatic updater.
- Help, feedback and web links: following a link opens GitHub, Microsoft or another destination through your browser or operating system. Those sites may process account, request and browsing information.

Third-party services have their own privacy policies. See the [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement) and the [Microsoft Privacy Statement](https://privacy.microsoft.com/privacystatement). Third-party distributors are responsible for their modifications, additional services and data practices.

## Retention and deletion

Settings and recent-file records generally persist locally between launches. You can disable recent files or clear recent records in Settings. Disabling the feature stops new records; it does not automatically delete existing ones. Clearing records does not delete archive files.

In-memory archive indexes and previews are released when the relevant session closes or the content becomes invalid. Extracted files and newly created archives remain at your chosen destination and are not automatically deleted when xdzip closes.

Files temporarily opened in another application are kept in an application-managed local cache. Completed, unlocked copies are retained for at least seven days, and expired entries are cleaned during later temporary-open operations. Exiting the application does not guarantee immediate removal of these copies. Temporary recovery state, update downloads and receipts may also remain locally.

You can use your operating system's file and application-data controls to remove local data you no longer need. Whether uninstallation removes settings and caches depends on the installation method and operating system. Manage system backups, cloud copies and exported files separately.

## Feedback and diagnostics

xdzip does not automatically upload diagnostics to the maintainer. If you submit feedback, version information, terminal output, screenshots or files, the maintainer can see what you provide and use it for communication, troubleshooting and maintenance. Error output and materials you choose to provide may include file names, paths or personal information. Review them and remove unrelated sensitive content before sending.

Feedback in GitHub Issues is generally public; retention, editing and deletion also depend on GitHub's features and policies. Do not provide passwords, identity credentials or private archives for ordinary support. Prefer reproduction samples without personal information.

If you voluntarily give the maintainer your Microsoft account email to join a private Store test group, it is used to configure and manage test access and is submitted to Microsoft. You can request removal from the group through the contact channels above. Leaving the group does not automatically delete an application you have already downloaded or data retained by third-party platforms.

## Your choices and security

You choose whether to use recent files, request update checks, configure a proxy or mirror, open external applications or websites, or provide feedback. You can also manage file and directory access through your operating system. Denying access required for an operation may prevent it from completing.

The application uses operating-system permissions and applicable temporary-directory protections to limit local access. Device administrators, malware, other authorized applications and system or cloud services may still access relevant data. Protect your device, output files and backups accordingly.

xdzip does not require a birth date or other age information and does not collect children's information for advertising or profiling. Minors should have a guardian help manage files, passwords and public feedback.

## Policy updates

This policy will be updated when actual features or data practices change, with the date shown on this page. You can consult this page and the [project release notes](https://github.com/xiaodaozhi/xdzip/releases) for related information. Contact the maintainer through the channels at the start of this policy if you have questions.
