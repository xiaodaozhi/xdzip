"""Keep the modified QtNetwork's source inputs and deployed-byte proof together."""
import json
from pathlib import Path
import shutil
import re
import subprocess

from qt_network_security import ROOT, configuration, digest, identity


def collect(prefix, notices):
    record = identity(prefix)
    config, source, _ = configuration()
    target = notices/'Qt-network-security'
    target.mkdir(parents=True, exist_ok=True)
    paths = []
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    for relative in ['patches/qt/README.md', 'scripts/qt-network-security.json'] + [p['file'] for p in config['patches']]:
        destination = target/Path(relative).name
        shutil.copyfile(ROOT/relative, destination)
        if destination.suffix == '.md':
            def absolute_link(match):
                path = (ROOT/relative).parent/match[1]
                return '](https://github.com/xiaodaozhi/xdzip/blob/'+revision+'/'+path.resolve().relative_to(ROOT).as_posix()+')'
            destination.write_text(re.sub(r'\]\(((?!https?://)[^)]+)\)', absolute_link, destination.read_text()), encoding='utf-8')
        paths.append(destination)
    receipt = target/'build-receipt.json'
    receipt.write_text(json.dumps(record, indent=2)+'\n', encoding='utf-8')
    paths.append(receipt)
    return paths, dict(name='QtNetwork', version=config['qt_version'], variant=config['variant'],
                       license='LGPL-3.0-only', source_build=True,
                       source=source, patches=config['patches'], network_sha256=record['network_sha256'],
                       receipt='Qt-network-security/build-receipt.json',
                       scope='Only QtNetwork rebuilt; QtCore and other SDK modules retain their existing provenance')


def verify_package(source):
    """Fail staging unless attribution binds deployed Network to the patched SDK.

    macOS deployment legitimately rewrites paths and seals. Its existing audited
    Mach-O section comparison supplies that link; a matching filename cannot.
    """
    target = source/'notices/Qt-network-security'
    record = json.loads((target/'build-receipt.json').read_bytes())
    config, upstream, inputs = configuration()
    if (record.get('variant') != config['variant'] or record.get('inputs_sha256') != inputs or
            record.get('source_sha256') != upstream['sha256'] or record.get('patches') != config['patches'] or
            record.get('verification', {}).get('status') != 'passed' or
            record['verification'].get('network_sha256') != record.get('network_sha256')):
        raise ValueError('Release requires the pinned verified QtNetwork security build')
    for patch in config['patches']:
        if digest(target/Path(patch['file']).name) != patch['sha256']:
            raise ValueError('Modified QtNetwork source patch missing or changed')
    attribution = json.loads((source/'runtime-attribution.json').read_bytes())
    audit = json.loads((source/'deployment-audit.json').read_bytes())
    import hashlib
    expected = hashlib.sha256(json.dumps(audit, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    if attribution.get('audit_sha256') != expected or attribution.get('network_security') != record:
        raise ValueError('Modified QtNetwork attribution differs from the audited package')
    candidates = [f for f in attribution['files'] if f.get('network_security_variant') == config['variant']]
    if not candidates or any(f['sdk_sha256'] != record['network_sha256'] or
                             f['evidence'] not in ('sdk-byte-identity', 'mach-o-sections-uuid-and-imports',
                                                   'deployment-copy-and-rewrite-record') for f in candidates):
        raise ValueError('Deployed QtNetwork is not bound to verified patched SDK bytes')
    audited = {('artifact/' if f in audit['files'] else '')+f['path']: f['sha256']
               for f in audit['files']+audit['external_files']}
    if any(audited.get(f['path']) != f['sha256'] for f in candidates):
        raise ValueError('Deployed QtNetwork attribution hash changed')
    return record
