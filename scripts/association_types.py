"""Product association mapping. Format support is determined by the reader."""

# Windows resolves the final extension, so .tar.gz uses the gz entry.
WINDOWS_ICONS = {
    'zip': 'zip', 'tar': 'tar', 'tgz': 'tgz', 'gz': 'gz',
    '7z': '7z', 'rar': 'rar', 'cab': 'cab', 'iso': 'iso', 'udf': 'udf',
}
MACOS_ICONS = ('zip', 'tar', 'tar-gz', 'gz', '7z', 'rar', 'cab', 'iso', 'udf')


def windows_progid(extension: str) -> str:
    if extension not in WINDOWS_ICONS:
        raise ValueError('Unsupported Windows association')
    # Every visible candidate uses the launcher. The hidden shared ID remains
    # available separately for prior user choices and historical rollback.
    return 'xdzip.Archive.' + extension.upper()
