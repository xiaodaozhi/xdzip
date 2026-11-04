"""Validate the language-subtag portion of an IANA registry snapshot."""

from datetime import date
import re


def parse_languages(iana_text):
    """Return the file date and explicit 2/3-letter language subtags.

    The private-use range is validated but not expanded into registered tags.
    A registered language subtag does not by itself identify ISO 639-2 fields.
    """
    blocks = re.split(r'(?m)^%%\s*$', iana_text.replace('\r\n', '\n').lstrip('\ufeff'))
    header = blocks[0].strip()
    if not re.fullmatch(r'File-Date: \d{4}-\d{2}-\d{2}', header):
        raise ValueError('Invalid IANA registry header')
    try:
        file_date = date.fromisoformat(header.removeprefix('File-Date: ')).isoformat()
    except ValueError as exc:
        raise ValueError('Invalid IANA registry date') from exc
    tags = set()
    ranges = set()
    for block in blocks[1:]:
        fields = {}
        for line in block.splitlines():
            if ': ' in line and not line.startswith(' '):
                key, value = line.split(': ', 1)
                if key in ('Type', 'Subtag'):
                    fields.setdefault(key, []).append(value)
        if len(fields.get('Type', [])) > 1:
            raise ValueError('Invalid IANA record type')
        if fields.get('Type') != ['language']:
            continue
        if len(fields.get('Subtag', [])) != 1:
            raise ValueError('Invalid IANA language record')
        tag = fields['Subtag'][0]
        if tag == 'qaa..qtz':
            if tag in ranges:
                raise ValueError('Duplicate IANA private-use range')
            ranges.add(tag)
            continue
        if not re.fullmatch('[a-z]{2,3}', tag) or tag in tags:
            raise ValueError('Invalid or duplicate IANA language subtag')
        tags.add(tag)
    if not tags:
        raise ValueError('No IANA language subtags')
    return file_date, tags
