#!/usr/bin/env python3
"""Adapt locally obtained iso-codes JSON into Qt's locale-generator input."""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def identifier(row, field, length, optional=False):
    value = row.get(field, '') if optional else row.get(field)
    if not isinstance(value, str) or (not (optional and value == '') and
                                      not re.fullmatch('[a-z]{%d}' % length, value)):
        raise ValueError('Invalid ' + field + ' in iso-codes data')
    return value


def convert(iso_639_2, iso_639_3):
    """Return a table with unambiguous Qt lookup keys, without names or metadata."""
    try:
        part2 = json.loads(iso_639_2)['639-2']
        part3 = json.loads(iso_639_3)['639-3']
    except (KeyError, TypeError, json.JSONDecodeError) as exc:
        raise ValueError('Invalid iso-codes JSON document') from exc
    if not isinstance(part2, list) or not isinstance(part3, list) or not part2 or not part3:
        raise ValueError('Both iso-codes tables must be nonempty lists')
    part2_by_code = {}
    reserved_seen = False
    for row in part2:
        if not isinstance(row, dict):
            raise ValueError('Invalid iso-codes record')
        # ISO 639-2 reserves a range, not a single language Qt can look up.
        if row.get('alpha_3') == 'qaa-qtz':
            if reserved_seen or row.get('alpha_2') or row.get('bibliographic'):
                raise ValueError('Invalid ISO 639-2 local-use range')
            reserved_seen = True
            continue
        code = identifier(row, 'alpha_3', 3)
        bibliographic = identifier(row, 'bibliographic', 3, optional=True) or code
        if code in part2_by_code:
            raise ValueError('Duplicate ISO 639-2 code: ' + code)
        part2_by_code[code] = bibliographic
    records = {}
    seen_part3 = set()
    for row in part3:
        if not isinstance(row, dict):
            raise ValueError('Invalid iso-codes record')
        code = identifier(row, 'alpha_3', 3)
        part1 = identifier(row, 'alpha_2', 2, optional=True)
        if code in seen_part3:
            raise ValueError('Duplicate ISO 639-3 code: ' + code)
        seen_part3.add(code)
        part2b = part2_by_code.get(code, '')
        part2t = code if part2b else ''
        key = part1 or part2b or code
        record = (code, part2b, part2t, part1, '')
        if key in records:
            raise ValueError('Conflicting Qt lookup key: ' + key)
        records[key] = record
    table = 'Id\tPart2b\tPart2t\tPart1\tUnused\n'
    table += ''.join('\t'.join(records[key]) + '\n' for key in sorted(records))
    return table.encode('utf-8'), len(part2), len(part3)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--iso-639-2', required=True, type=Path, help='Locally obtained iso_639-2.json')
    parser.add_argument('--iso-639-3', required=True, type=Path, help='Locally obtained iso_639-3.json')
    parser.add_argument('--output', required=True, type=Path, help='New output file under build/')
    args = parser.parse_args()
    output = args.output.resolve()
    build = (ROOT/'build').resolve()
    if output == build or not output.is_relative_to(build):
        parser.error('Output must be a new file under this source tree\'s build/ directory')
    raw2, raw3 = args.iso_639_2.read_bytes(), args.iso_639_3.read_bytes()
    table, count2, count3 = convert(raw2, raw3)
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open('xb') as stream:
        stream.write(table)
    print(json.dumps({'iso_639_2_sha256': hashlib.sha256(raw2).hexdigest(),
                      'iso_639_3_sha256': hashlib.sha256(raw3).hexdigest(),
                      'output_sha256': hashlib.sha256(table).hexdigest(),
                      'iso_639_2_records': count2, 'iso_639_3_records': count3,
                      'historical_input_verified': False}))


if __name__ == '__main__':
    main()
