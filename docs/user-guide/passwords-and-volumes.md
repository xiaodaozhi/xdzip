# Passwords and split volumes

English | [简体中文](../zh-CN/user-guide/passwords-and-volumes.md) | [繁體中文（台灣）](../zh-TW/user-guide/passwords-and-volumes.md)

## Passwords

AES Crypt and archives with encrypted filenames, such as 7z with name encryption, need a password before showing the file list. Others can prompt during extraction, testing or preview. Preserve spaces and letter case when entering the password. ZIP AES-256 is supported in admitted profiles. ZIP AES-128/192 and traditional ZipCrypto are unsupported. If the password is correct but the archive still fails, check encryption support and whether the file is damaged. [Supported encryption](../engineering/password-extraction.md)

When creating a password-protected archive, choose a format that offers encryption. ZIP AES-256 protects file contents and leaves filenames visible. If the file list must also be protected, choose 7z and enable filename encryption. Store the password separately from the archive.

## Split volumes

1. Put all volumes from the same archive in one directory and keep their original names.
2. Open a supported volume. XDZip finds the other parts in the same directory. Numbered ZIP and 7z, PKZIP, and supported RAR and CAB sets follow different naming rules.
3. If the set is incomplete, obtain the missing parts and reopen the archive. Joining or renaming parts by hand can make them unreadable.

To create a split archive, choose ZIP or 7z and set the volume size in the creation window. Share the complete set. Splitting an archive does not make it editable in XDZip. [Volume naming and supported profiles](../engineering/archive-opening.md)
