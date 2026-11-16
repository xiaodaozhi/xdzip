#!/usr/bin/env python3
"""Extract RFC 7932 wire data, never reference implementation algorithms."""
import argparse
import ast
import hashlib
from pathlib import Path
import re
import zlib

ROOT = Path(__file__).resolve().parents[1]
RFC_SHA256 = '394abd1879e016f4eac89512c2acaf92f7b2d67feb8ab58b6b118c213be07146'


def rust_bytes(value):
    return 'b"' + ''.join(chr(b) if 32 <= b < 127 and b not in [34, 92] else f'\\x{b:02x}' for b in value) + '"'


def render(data):
    if hashlib.sha256(data).hexdigest() != RFC_SHA256:
        raise ValueError('Unexpected RFC 7932 identity')
    text = data.decode('ascii')
    appendix = text[text.index('Appendix A.  Static Dictionary Data'):text.index('Appendix B.  List of Word Transformations')]
    dictionary = bytes.fromhex(''.join(re.findall(r'^      ([0-9a-f]{20,})\s*$', appendix, flags=re.M)))
    if len(dictionary) != 122784 or zlib.crc32(dictionary) != 0x5136cb04:
        raise ValueError('Invalid normative dictionary length or CRC')
    result = ['// Generated from RFC 7932 sections 7.1 and appendices A/B.',
              '// Copyright (c) 2016 IETF Trust and the persons identified as authors of the code. All rights reserved.',
              '// Redistribution and use in source and binary forms, with or without modification, is permitted',
              '// pursuant to, and subject to the license terms contained in, the Revised BSD License set forth',
              '// in Section 4.c of the IETF Trust Legal Provisions Relating to IETF Documents.',
              '// https://trustee.ietf.org/license-info. See third_party/brotli-rfc7932/LICENSE.txt.',
              '#[rustfmt::skip]', 'pub(super) const DICTIONARY: &[u8; 122784] = include_bytes!("../../../../third_party/brotli-rfc7932/dictionary.bin");']
    for name, crc in [('Lut0', 0x8e91efb7), ('Lut1', 0xd01a32f4), ('Lut2', 0x0dd7a0d6)]:
        found = re.search(name + r' :=\s*([0-9,\s]+)', text)
        values = [int(n) for n in re.findall(r'\d+', found[1])]
        if len(values) != 256 or zlib.crc32(bytes(values)) != crc:
            raise ValueError('Invalid normative context table: ' + name)
        result += ['#[rustfmt::skip]', f'pub(super) const {name.upper()}: [u8;256] = [']
        result.extend('    ' + ', '.join(map(str, values[at:at+16])) + ',' for at in range(0, 256, 16))
        result.append('];')
    found = re.search(r'NDBITS :=\s*([0-9,\s]+)', text)
    values = [int(n) for n in re.findall(r'\d+', found[1])]
    if len(values) != 25 or sum(n * (1 << values[n]) for n in range(4,25)) != len(dictionary):
        raise ValueError('Invalid normative word depths')
    result += ['#[rustfmt::skip]', 'pub(super) const DEPTHS: [u8;25] = [' + ', '.join(map(str, values)) + '];']
    transforms = text[text.index('Appendix B.  List of Word Transformations'):text.index('Appendix C.  Computing CRC-32 Check Values')]
    pattern = r'^\s+(\d+)\s+("(?:\\.|[^"\\])*")\s+(Identity|FermentFirst|FermentAll|OmitFirst[1-9]|OmitLast[1-9])\s+("(?:\\.|[^"\\])*")\s*$'
    rows = re.findall(pattern, transforms, flags=re.M)
    if [int(row[0]) for row in rows] != list(range(121)):
        raise ValueError('Invalid normative transform identifiers')
    result += ['#[rustfmt::skip]', 'pub(super) const TRANSFORMS: [(&[u8], u8, &[u8]);121] = [']
    for _, prefix, op, suffix in rows:
        op = {'Identity': 0, 'FermentFirst': 1, 'FermentAll': 2}.get(op, int(op[-1])+2 if op.startswith('OmitFirst') else int(op[-1])+11 if op.startswith('OmitLast') else 0)
        result.append(f'    ({rust_bytes(ast.literal_eval("b"+prefix))}, {op}, {rust_bytes(ast.literal_eval("b"+suffix))}),')
    result.append('];')
    return dictionary, ('\n'.join(result)+'\n').encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rfc', type=Path, required=True)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    dictionary, table = render(args.rfc.read_bytes())
    for path, data in [(ROOT/'third_party/brotli-rfc7932/dictionary.bin', dictionary), (ROOT/'crates/xdzip-compression/src/brotli/data.rs', table)]:
        if args.check:
            if path.read_bytes() != data:
                raise ValueError('Stale normative data: ' + path.relative_to(ROOT).as_posix())
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
    print('Normative dictionary, 3 context tables, word depths and 121 transformations verified')


if __name__ == '__main__':
    main()
