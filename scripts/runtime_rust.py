"""Separate normal dependency candidates from build/dev/proc-macro tooling."""


def runtime_candidates(metadata, references, host):
    packages = {p['id']:p for p in metadata['packages']}
    nodes = {n['id']:n for n in metadata['resolve']['nodes']}
    root_names = ['xdzip-qt-bridge', 'xdzip-codec-bridge', 'xdzip-cli']
    if '-windows-' in host or '-linux-' in host:
        root_names.append('xdzip-update-helper')
    elif not host.endswith('-apple-darwin'):
        raise ValueError('Unsupported runtime candidate host: '+host)
    roots = {}
    for name in root_names:
        matches = [p['id'] for p in metadata['packages'] if p['name']==name and p['source'] is None]
        if len(matches)!=1 or matches[0] not in nodes or matches[0] not in references:
            raise ValueError('Expected one resolved workspace runtime root: '+name)
        roots[name] = matches[0]
    selected=set();pending=list(roots.values())
    while pending:
        key=pending.pop()
        if key in selected:continue
        if any('proc-macro' in target['kind'] for target in packages[key]['targets']):continue
        selected.add(key)
        pending.extend(d['pkg'] for d in nodes[key]['deps'] if any(k['kind'] is None for k in d['dep_kinds']))
    return {'root':references[roots['xdzip-qt-bridge']],
            'roots':{name:references[key] for name,key in roots.items()},
            'packages':sorted(references[k] for k in selected),
            'evidence':'Host-filtered normal dependency graph rooted at the desktop bridge, independent CLI, codec worker bridge and applicable update helper; excludes build/dev/proc-macro edges.',
            'limit':'Candidate runtime inputs, not proof of final retained machine code; linker dead stripping and native/compiler runtime require separate review.'}
