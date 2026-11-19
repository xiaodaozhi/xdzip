$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
if ($IsLinux) {
    python scripts/verify-linux-installer.py --packages build/installer-tests/linux-packages --fixture build/desktop/apps/desktop/m2-flat.zip --formats build/installer-tests/association-fixtures --output build/review/linux-install-lifecycle.json
} elseif ($IsWindows) {
    pwsh -NoProfile -File scripts/check-windows-install.ps1 -Packages build/installer-tests/windows-packages -Fixture build/desktop/apps/desktop/m2-flat.zip -Formats build/installer-tests/association-fixtures -Output build/review/windows-install-lifecycle.json
} else {
    throw 'Installer lifecycle applies to Windows and Linux'
}
