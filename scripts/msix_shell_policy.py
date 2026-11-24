"""Manifest/channel contract shared by MSIX construction and verification."""
import re

COM = 'http://schemas.microsoft.com/appx/manifest/com/windows10'
D4 = 'http://schemas.microsoft.com/appx/manifest/desktop/windows10/4'
D5 = 'http://schemas.microsoft.com/appx/manifest/desktop/windows10/5'
D9 = 'http://schemas.microsoft.com/appx/manifest/desktop/windows10/9'
CLASSIC_STORE = 'D1915E65-8E17-4B3C-BB61-4E34A1551A25'
CLASSIC_SIDELOAD = '635731B5-F385-42E3-984D-894217682DAA'
MODERN = '50F4499A-05FD-497B-9F72-C39041672093'


def marker(channel):
    if channel not in ('store', 'sideload'):
        raise ValueError('Unknown MSIX channel')
    return ('xdzip-msix-v1\n' + channel + '\n').encode('ascii')


def verify_manifest(root, channel, shell_library='xdzip-shell.dll'):
    marker(channel)
    if not re.fullmatch(r'(xdzip-shell\.dll|shell/xdzip-shell-[a-f0-9]{16}\.dll)', shell_library):
        raise ValueError('Invalid MSIX shell library path')
    expected = {MODERN} if channel == 'store' else {CLASSIC_SIDELOAD}
    classes = list(root.iter('{' + COM + '}Class'))
    if (len(classes) != len(expected) or {item.get('Id') for item in classes} != expected or
            any(item.get('Path') != shell_library.replace('/', '\\') or item.get('ThreadingModel') != 'STA' for item in classes)):
        raise ValueError('MSIX COM classes differ from channel policy')
    legacy = list(root.iter('{' + D9 + '}ExtensionHandler'))
    if channel == 'sideload' and (len(legacy) != 2 or {item.get('Type') for item in legacy} != {'*', 'Directory'} or
            any(item.get('Clsid') != CLASSIC_SIDELOAD for item in legacy)):
        raise ValueError('MSIX classic context menu differs from channel policy')
    if channel == 'store' and (legacy or any(item.get('Category') == 'windows.fileExplorerClassicContextMenuHandler' for item in root.iter())):
        raise ValueError('Store MSIX must not register a classic handler')
    modern = list(root.iter('{' + D5 + '}Verb'))
    contexts = list(root.iter('{' + D5 + '}ItemType'))
    if channel == 'store':
        if (len(modern) != 2 or any(item.get('Clsid') != MODERN for item in modern) or
                len(contexts) != 2 or {item.get('Type') for item in contexts} != {'*', 'Directory'}):
            raise ValueError('MSIX modern context menu differs from channel policy')
    elif modern or contexts:
        raise ValueError('Sideload MSIX must not register a modern context menu')
