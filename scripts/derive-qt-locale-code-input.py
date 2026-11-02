#!/usr/bin/env python3
"""Derive a limited Qt locale-generator input from an existing Qt source file."""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
ENTRY = re.compile(
    r'^\s*LanguageCodeEntry\s*\{\s*(\{[^{}]*\})\s*,\s*(\{[^{}]*\})\s*,\s*'
    r'(\{[^{}]*\})\s*,\s*(\{[^{}]*\})\s*\}\s*,\s*(?://[^\n]*)?$'
)
FIELD = re.compile(r"^\{\s*(?:'[a-z]'\s*(?:,\s*'[a-z]'\s*)*)?\}$")
TABLE = re.compile(
    r'^constexpr\s+std::array<LanguageCodeEntry,\s*(\d+)>\s+languageCodeList\s*\{\s*\n'
    r'(.*?)^\};\s*$', re.MULTILINE | re.DOTALL
)


def derived_table(qt_source):
    """Return a minimal, synthetic table for Qt's existing language lookup keys."""
    qt_source = qt_source.replace('\r\n', '\n')
    matches = list(TABLE.finditer(qt_source))
    if len(matches) != 1:
        raise ValueError('Expected exactly one Qt languageCodeList')
    declared = int(matches[0].group(1))
    records = {}
    entries = 0
    for line in matches[0].group(2).splitlines():
        if not line.strip():
            continue
        match = ENTRY.fullmatch(line)
        if not match or any(not FIELD.fullmatch(field) for field in match.groups()):
            raise ValueError('Unrecognized Qt language-code entry')
        part1, part2b, part2t, part3 = (
            ''.join(re.findall(r"'([a-z])'", field)) for field in match.groups()
        )
        if (len(part3) != 3 or len(part1) not in (0, 2) or
                not ((not part2b and not part2t) or
                     (len(part2b) == 3 and part2t == part3))):
            raise ValueError('Unexpected Qt language-code fields')
        key = part1 or part2b or part3
        row = (part3, part2b, part2t, part1, '')
        if key in records and records[key] != row:
            raise ValueError('Conflicting Qt language-code lookup key')
        records[key] = row
        entries += 1
    if not entries or entries != declared:
        raise ValueError('Qt language-code entry count differs')
    data = 'Id\tPart2b\tPart2t\tPart1\tUnused\n'
    data += ''.join('\t'.join(records[key]) + '\n' for key in sorted(records))
    return data.encode('utf-8'), entries, len(records)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qtbase', type=Path, required=True, help='Unpacked QtBase source directory')
    parser.add_argument('--output', type=Path, required=True, help='New output file under build/')
    args = parser.parse_args()
    source = args.qtbase/'src/corelib/text/qlocale_data_p.h'
    output = args.output.resolve()
    if not output.is_relative_to((ROOT/'build').resolve()) or output == (ROOT/'build').resolve():
        parser.error('Output must be a new file under this source tree\'s build/ directory')
    raw = source.read_bytes()
    table, entries, unique = derived_table(raw.decode('utf-8'))
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open('xb') as stream:
        stream.write(table)
    print(json.dumps({'qt_source_sha256': hashlib.sha256(raw).hexdigest(),
                      'derived_sha256': hashlib.sha256(table).hexdigest(),
                      'source_entries': entries, 'distinct_lookup_keys': unique,
                      'historical_input_verified': False, 'complete_iso_table': False}))


if __name__ == '__main__':
    main()
