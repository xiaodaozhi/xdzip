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
        if 'rust_runtime_candidates' in inventory:
            component['properties'] += properties(runtime_candidate=str(package['bom_ref'] in inventory['rust_runtime_candidates']['packages']).lower())
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
    for dependency in inventory.get('native_dependencies', []):
        bom['components'].append({'type':'library', 'bom-ref':'declared-native-'+dependency['name'],
            'name':dependency['name'], 'version':dependency['version'],
            'licenses':[{'expression':dependency['license']}],
            'externalReferences':[{'type':'distribution', 'url':dependency['sdk']['url']},
                                  {'type':'vcs', 'url':dependency['source']['url']}],
            'properties':properties(evidence='pinned native SDK declaration; deployed files require separate attribution',
                                    sdk_sha256=dependency['sdk']['sha256'], source_sha256=dependency['source']['sha256'])})
    for asset in inventory['assets']:
        item = {'type':'file', 'bom-ref':'asset:'+asset['path'], 'name':asset['path'],
                'hashes':[{'alg':'SHA-256','content':asset['sha256']} ]}
        if asset.get('license'):
            item['licenses'] = [{'license':{'id':asset['license']}}]
        bom['components'].append(item)
    bom['metadata']['properties'] += properties(dirty=str(inventory['dirty']).lower(), build=inventory['version_source']['build'])
    return bom


def artifact_bom(audit, source, attribution=None):
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
    for package in audit.get('system_runtime_requirements',[]):
        ref='system-runtime:'+package['package']
        bom['components'].append({'type':'library','bom-ref':ref,'name':package['package'],'version':package['version'],
            'purl':'pkg:deb/ubuntu/'+quote(package['package'],safe='')+'@'+quote(package['version'],safe=''),
            'properties':properties(evidence=package['evidence'],source_package=package['source_package'],
                source_version=package['source_version'],reason=package['reason'],scope='Required OS package; not bundled into application')})
    if audit.get('system_runtime_requirements'):
        bom['dependencies'].append({'ref':'xdzip-product','dependsOn':['system-runtime:'+p['package'] for p in audit['system_runtime_requirements']]})
    if attribution is not None:
        expected=hashlib.sha256(json.dumps(audit,sort_keys=True,separators=(',',':')).encode()).hexdigest()
        if attribution['audit_sha256']!=expected or attribution['identity']!=identity:
            raise ValueError('Attribution and audited artifact identities differ')
        bom['metadata']['properties'] += properties(attribution_completeness='incomplete', attribution_gaps=len(attribution['gaps']),
            attribution_canonical_sha256=hashlib.sha256(json.dumps(attribution,sort_keys=True,separators=(',',':')).encode()).hexdigest())
        for p in attribution['components']:
            c={'type':'library','bom-ref':'runtime:'+p['ref'],'name':p['name'],
                'properties':properties(evidence='SDK upstream SPDX; not a licensing decision', upstream_ref=p['ref'],
                    license_declared=p['license_declared'], license_concluded_by_upstream=p['license_concluded_by_upstream'])}
            if p.get('version'):c['version']=p['version']
            if p['copyright']!='NOASSERTION':c['copyright']=p['copyright']
            if p['source'].startswith(('https://','http://','git://')):
                c['externalReferences']=[{'type':'vcs','url':p['source']}]
            if p['license_concluded_by_upstream'] not in {'NOASSERTION','NONE',''}:
                c['licenses']=[{'expression':p['license_concluded_by_upstream']}]
            bom['components'].append(c)
        for edge in attribution['dependencies']:
            bom['dependencies'].append({'ref':'runtime:'+edge['ref'],'dependsOn':['runtime:'+r for r in edge['dependsOn']]})
        owners={f['path']:['runtime:'+r for r in f['owners']] for f in attribution['files'] if f['path'] in refs and f['owners']}
        combined={d['ref']:set(d['dependsOn']) for d in bom['dependencies']}
        for ref,values in owners.items():combined.setdefault(ref,set()).update(values)
        bom['dependencies']=[{'ref':ref,'dependsOn':sorted(values)} for ref,values in sorted(combined.items())]
    return bom


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--audit',type=Path,required=True)
    parser.add_argument('--source',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--attribution',type=Path)
    args = parser.parse_args()
    report = artifact_bom(json.loads(args.audit.read_text()),json.loads(args.source.read_text()),
        json.loads(args.attribution.read_text()) if args.attribution else None)
    report['metadata']['properties'] += properties(source_bom_file_sha256=hashlib.sha256(args.source.read_bytes()).hexdigest())
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n')
    print(f'Artifact CycloneDX: {sum(c["type"]=="file" for c in report["components"])} hashed files, {sum(c["type"]=="library" for c in report["components"])} attributed components, completeness=incomplete')

if __name__ == '__main__':
    main()
