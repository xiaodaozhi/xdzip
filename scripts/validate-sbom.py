#!/usr/bin/env python3
"""Validate against hash-pinned official CycloneDX schemas plus local graph checks.

Schemas are cached under build/, never mixed into product assets or deployed files.
Requires jsonschema==4.23.0 in the isolated validation environment.
"""
import argparse
import hashlib
import json
from pathlib import Path
import urllib.request
from jsonschema import Draft7Validator
from referencing import Registry, Resource

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('boms',nargs='+',type=Path)
parser.add_argument('--schemas',type=Path,default=ROOT/'build/review/sbom-schemas')
args = parser.parse_args()
args.schemas.mkdir(parents=True,exist_ok=True)
store = {}
for item in json.loads((ROOT/'scripts/sbom-schemas.json').read_text())['files']:
    target = args.schemas/item['name']
    if not target.exists():
        data = urllib.request.urlopen(item['url'],timeout=30).read()
        if hashlib.sha256(data).hexdigest() != item['sha256']:
            raise SystemExit('Official schema download hash mismatch')
        target.write_bytes(data)
    data = target.read_bytes()
    if hashlib.sha256(data).hexdigest() != item['sha256']:
        raise SystemExit('Cached official schema hash mismatch')
    schema = json.loads(data)
    for key in [schema.get('$id'), 'http://cyclonedx.org/schema/'+item['name'], 'https://cyclonedx.org/schema/'+item['name']]:
        if key:
            store[key] = schema
schema = store['http://cyclonedx.org/schema/bom-1.6.schema.json']
# Only the three pinned in-memory resources can resolve schema references.
registry = Registry().with_resources((uri, Resource.from_contents(value)) for uri,value in store.items())
validator = Draft7Validator(schema,registry=registry)
for path in args.boms:
    bom = json.loads(path.read_text())
    errors = sorted(validator.iter_errors(bom),key=lambda e:str(list(e.path)))
    if errors:
        raise SystemExit('CycloneDX schema error: '+str(list(errors[0].path))+' '+errors[0].message)
    refs = [c['bom-ref'] for c in [bom['metadata']['component']]+bom.get('components',[])]
    if len(set(refs)) != len(refs):
        raise SystemExit('Duplicate component reference')
    for edge in bom.get('dependencies',[]):
        if not {edge['ref'], *edge.get('dependsOn',[])} <= set(refs):
            raise SystemExit('Dangling dependency reference')
    if not bom.get('compositions') or any(c['aggregate']!='incomplete' for c in bom['compositions']):
        raise SystemExit('Current review inventories must explicitly declare incomplete coverage')
    print('CycloneDX 1.6 official schema and references passed:',path.name)
