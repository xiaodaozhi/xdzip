#!/usr/bin/env python3
"""Generate bounded CycloneDX 1.6 inventories; never claim complete deployment."""
import argparse
import hashlib
import json
from pathlib import Path
from urllib.parse import quote
import uuid

ROOT = Path(__file__).resolve().parents[1]


def properties(**values):
    return [{'name':'xdzip:'+name, 'value':str(value)} for name,value in values.items()]


def base(version, revision, scope):
    return {'$schema':'http://cyclonedx.org/schema/bom-1.6.schema.json',
            'bomFormat':'CycloneDX', 'specVersion':'1.6', 'version':1,
            'serialNumber':'urn:uuid:'+str(uuid.uuid4()),
            'metadata':{'component':{'type':'application', 'bom-ref':'xdzip-product', 'name':'xdzip',
                                     'version':version, 'licenses':[{'license':{'id':'Apache-2.0'}}]},
                        'properties':properties(revision=revision, scope=scope, readiness='experimental; release review pending')},
            'compositions':[{'aggregate':'incomplete', 'assemblies':['xdzip-product']}],
            'components':[]}


def source_bom(inventory):
    version = inventory['display_version']
    bom = base(version, inventory['revision'], 'host-resolved source build; includes tools and development dependencies')
    for package in inventory['rust_packages']:
        component = {'type':'library', 'bom-ref':package['bom_ref'], 'name':package['name'],
                     'version':package['version'],
                     'properties':properties(evidence='Cargo host-resolved graph; not proof of linked runtime inclusion')}
        if package.get('source_kind') == 'registry':
            component['purl'] = f'pkg:cargo/{quote(package["name"],safe="")}@{quote(package["version"],safe="")}'
        if package['license']:
            component['licenses'] = [{'expression':package['license']}]
        bom['components'].append(component)
    bom['metadata']['lifecycles'] = [{'phase':'build'}]
    bom['dependencies'] = inventory['rust_dependencies']
    for module in inventory['qt_declared_product_modules']:
        bom['components'].append({'type':'library', 'bom-ref':'declared-qt-'+module,
            'name':'Qt '+module, 'version':inventory['qt_version'],
            'properties':properties(evidence='declared CMake module; transitive and plugin licenses require deployment review')})
    for asset in inventory['assets']:
        item = {'type':'file', 'bom-ref':'asset:'+asset['path'], 'name':asset['path'],
                'hashes':[{'alg':'SHA-256','content':asset['sha256']} ]}
        if asset.get('license'):
            item['licenses'] = [{'license':{'id':asset['license']}}]
        bom['components'].append(item)
    bom['metadata']['properties'] += properties(dirty=str(inventory['dirty']).lower(), build=inventory['version_source']['build'])
    return bom


def artifact_bom(audit, source):
    if audit['status'] != 'passed':
        raise ValueError('Cannot generate a passing artifact inventory from failed imports')
    identity = audit['identity']
    props = {p['name']:p['value'] for p in source['metadata']['properties']}
    if (source['metadata']['component']['version'] != identity['version'] or
        not props['xdzip:revision'].startswith(identity['revision']) or
        props['xdzip:dirty'] != str(identity['dirty']).lower() or
        props['xdzip:build'] != str(identity['build'])):
        raise ValueError('Source and binary identities differ; rebuild and regenerate evidence')
    bom = base(identity['version'], props['xdzip:revision'],
               'actual artifact files and observed native imports; component attribution incomplete')
    bom['metadata']['properties'] += properties(platform=audit['platform'], mode=audit['mode'], build=identity['build'], dirty=str(identity['dirty']).lower(),
        source_bom_canonical_sha256=hashlib.sha256(json.dumps(source,sort_keys=True,separators=(',',':')).encode()).hexdigest(),
        sdk_independent_imports=str(audit['sdk_independent_imports']).lower(),
        limitation='Static Rust runtime attribution, Qt bundled third parties and dynamic plugin coverage still require review')
    bom['metadata']['lifecycles'] = [{'phase':'post-build'}]
    refs = set()
    for file in audit['files'] + audit['external_files']:
        prefix = '' if file in audit['external_files'] else 'artifact/'
        ref = prefix+file['path']
        refs.add(ref)
        bom['components'].append({'type':'file', 'bom-ref':ref, 'name':ref,
                                 'hashes':[{'alg':'SHA-256','content':file['sha256']} ]})
    dependencies = {}
    for edge in audit['binary_imports']:
        if edge['from'] in refs and edge.get('to') in refs:
            dependencies.setdefault(edge['from'],set()).add(edge['to'])
    bom['dependencies'] = [{'ref':ref,'dependsOn':sorted(deps)} for ref,deps in sorted(dependencies.items())]
    return bom


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--audit',type=Path,required=True)
    parser.add_argument('--source',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    report = artifact_bom(json.loads(args.audit.read_text()),json.loads(args.source.read_text()))
    report['metadata']['properties'] += properties(source_bom_file_sha256=hashlib.sha256(args.source.read_bytes()).hexdigest())
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n')
    print(f'Artifact CycloneDX: {len(report["components"])} hashed files, completeness=incomplete')

if __name__ == '__main__':
    main()
