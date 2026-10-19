#!/usr/bin/env python3
"""Check the actual PE subsystem of the product and diagnostic executables."""
import argparse
import struct
from pathlib import Path


def subsystem(path):
    data = Path(path).read_bytes()
    assert data[:2] == b'MZ', 'Missing DOS signature'
    pe = struct.unpack_from('<I', data, 0x3c)[0]
    assert data[pe:pe + 4] == b'PE\0\0', 'Missing PE signature'
    optional = pe + 24
    assert struct.unpack_from('<H', data, optional)[0] in (0x10b, 0x20b), 'Unsupported PE optional header'
    return struct.unpack_from('<H', data, optional + 68)[0]


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--gui', required=True)
    parser.add_argument('--console', required=True)
    args = parser.parse_args()
    assert subsystem(args.gui) == 2, 'Product must use IMAGE_SUBSYSTEM_WINDOWS_GUI'
    assert subsystem(args.console) == 3, 'Diagnostics must retain IMAGE_SUBSYSTEM_WINDOWS_CUI'
    print('Windows PE subsystem: GUI product, console diagnostics passed')
