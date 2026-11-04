#!/usr/bin/env python3
"""Adapt independently obtained IANA and Library of Congress data for Qt."""
import argparse
import hashlib
import json
from pathlib import Path
import re

from iana_language_registry import parse_languages

ROOT = Path(__file__).resolve().parents[1]


def convert(iana_text, loc_text):
    """Return Qt lookup rows and omitted codes; never treat these as historical inputs."""
    file_date, tags = parse_languages(iana_text)

    records = {}
    used = set()
    seen_b, seen_t, seen_a = set(), set(), set()
    reserved = False
    unmatched = []
    count = 0
    for line in loc_text.lstrip('\ufeff').splitlines():
        fields = line.split('|')
        if len(fields) != 5:
            raise ValueError('Invalid Library of Congress record')
        b, t, a = fields[:3]
        count += 1
        if b == 'qaa-qtz':
            if reserved or t or a:
                raise ValueError('Invalid Library of Congress private-use range')
            reserved = True
            continue
        t = t or b
        if (not re.fullmatch('[a-z]{3}', b) or not re.fullmatch('[a-z]{3}', t) or
                (a and not re.fullmatch('[a-z]{2}', a)) or b in seen_b or t in seen_t or
                (a and a in seen_a)):
            raise ValueError('Invalid or duplicate Library of Congress code')
        seen_b.add(b); seen_t.add(t)
        if a:
            seen_a.add(a)
        if a not in tags and t not in tags:
            unmatched.append(t)
            continue
        if a and a not in tags:
            raise ValueError('Library of Congress two-letter code absent from IANA')
        row = (t, b, t, a, '')
        key = a or b or t
        if key in records and records[key] != row:
            raise ValueError('Conflicting Qt lookup key')
        records[key] = row
        used.add(t)
    if not count:
        raise ValueError('No Library of Congress codes')
    for tag in sorted(tags):
        if len(tag) != 3 or tag in used:
            continue
        row = (tag, '', '', '', '')
        if tag in records and records[tag] != row:
            raise ValueError('Conflicting Qt lookup key')
        records[tag] = row
    table = 'Id\tPart2b\tPart2t\tPart1\tUnused\n'
    table += ''.join('\t'.join(records[key]) + '\n' for key in sorted(records))
    return table.encode('utf-8'), {
        'iana_file_date': file_date, 'iana_language_subtags': len(tags),
        'loc_rows': count, 'output_rows': len(records), 'loc_unmatched': sorted(unmatched),
        'iana_two_letter_without_loc': sorted(tag for tag in tags if len(tag) == 2 and tag not in seen_a),
        'historical_input_verified': False, 'complete_corresponding_source_verified': False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--iana', required=True, type=Path, help='Locally obtained IANA registry')
    parser.add_argument('--loc', required=True, type=Path, help='Locally obtained ISO 639-2 text')
    parser.add_argument('--output', required=True, type=Path, help='New output file under build/')
    args = parser.parse_args()
    output = args.output.resolve()
    build = (ROOT/'build').resolve()
    if output == build or not output.is_relative_to(build):
        parser.error('Output must be a new file under this source tree\'s build/ directory')
    raw_iana, raw_loc = args.iana.read_bytes(), args.loc.read_bytes()
    table, report = convert(raw_iana.decode('utf-8'), raw_loc.decode('utf-8'))
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open('xb') as stream:
        stream.write(table)
    report.update(iana_sha256=hashlib.sha256(raw_iana).hexdigest(),
                  loc_sha256=hashlib.sha256(raw_loc).hexdigest(),
                  output_sha256=hashlib.sha256(table).hexdigest())
    print(json.dumps(report))


if __name__ == '__main__':
    main()
