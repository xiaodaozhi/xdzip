"""Product association mapping. Format support is determined by the reader."""

from archive_formats_generated import MACOS_ICONS, WINDOWS_ICONS


def windows_progid(extension: str) -> str:
    if extension not in WINDOWS_ICONS:
        raise ValueError('Unsupported Windows association')
    # Every visible candidate uses the launcher. The hidden shared ID remains
    # available separately for prior user choices and historical rollback.
    return 'xdzip.Archive.' + extension.upper()
