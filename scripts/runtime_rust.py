"""Separate normal dependency candidates from build/dev/proc-macro tooling."""


def runtime_candidates(metadata, references):
    packages = {p['id']:p for p in metadata['packages']}
    nodes = {n['id']:n for n in metadata['resolve']['nodes']}
    roots = [p['id'] for p in metadata['packages'] if p['name']=='xdzip-qt-bridge' and p['source'] is None]
    if len(roots)!=1:
        raise ValueError('Expected one workspace Rust product bridge')
    selected=set();pending=list(roots)
    while pending:
        key=pending.pop()
        if key in selected:continue
        if any('proc-macro' in target['kind'] for target in packages[key]['targets']):continue
        selected.add(key)
        pending.extend(d['pkg'] for d in nodes[key]['deps'] if any(k['kind'] is None for k in d['dep_kinds']))
    return {'root':references[roots[0]],'packages':sorted(references[k] for k in selected),
            'evidence':'Host-filtered normal dependency graph rooted at the static product bridge; excludes build/dev/proc-macro edges.',
            'limit':'Candidate runtime inputs, not proof of final retained machine code; linker dead stripping and native/compiler runtime require separate review.'}
