"""Prepare Qt Network from a trusted update-regression run or current source."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
from urllib.error import URLError
from urllib.parse import quote, urlencode, urlsplit
from urllib.request import Request, urlopen
import zipfile

from qt_network_bundle import BUNDLE_NAME, MAX_LIBRARY_BYTES, inputs, key
from qt_network_security import RECEIPT, ROOT, digest, prepare
from windows_ninja_safety import utf8_console

WORKFLOW = 'update-regression.yml'
WORK = ROOT/'build/toolchain/qt-network-update-ci'
SOURCE_CACHE = ROOT/'build/source-cache'
MAX_ARCHIVE_BYTES = MAX_LIBRARY_BYTES + 2 * 1024 * 1024
PRODUCER_JOBS = {('win32', 'x86_64'): 'windows', ('darwin', 'arm64'): 'macos',
                 ('darwin', 'x86_64'): 'macos_intel', ('linux', 'x86_64'): 'linux'}
FAILURES = (OSError, ValueError, KeyError, TypeError, AttributeError, zipfile.BadZipFile,
            subprocess.SubprocessError, URLError)


def api(path, *, redirect=False):
    url = os.environ['GITHUB_API_URL'] + '/repos/' + os.environ['GITHUB_REPOSITORY'] + path
    request = Request(url, headers={'Authorization': 'Bearer ' + os.environ['GITHUB_TOKEN'],
                                    'Accept': 'application/vnd.github+json'})
    if redirect:
        from urllib.error import HTTPError
        from urllib.request import HTTPRedirectHandler, build_opener

        class NoRedirect(HTTPRedirectHandler):
            def redirect_request(self, *args, **kwargs):
                return None

        try:
            with build_opener(NoRedirect()).open(request, timeout=15):
                raise ValueError('Expected GitHub archive redirect')
        except HTTPError as error:
            if error.code != 302:
                raise
            location = error.headers['Location']
            if not location or urlsplit(location).scheme != 'https':
                raise ValueError('Invalid GitHub archive redirect')
            return location
    with urlopen(request, timeout=15) as response:
        return json.load(response)


def trusted_artifact(artifact, name, workflow_id, repo_id, platform_job):
    if artifact['name'] != name or artifact['expired'] or not 0 < artifact['size_in_bytes'] <= MAX_ARCHIVE_BYTES:
        return False
    producer = artifact['workflow_run']
    if (producer['repository_id'] != repo_id or producer['head_repository_id'] != repo_id
            or producer['head_branch'] != 'main'):
        return False
    run_url = '/actions/runs/' + str(int(producer['id']))
    run = api(run_url)
    if not (run['id'] == producer['id'] and run['workflow_id'] == workflow_id
            and run['path'] == '.github/workflows/' + WORKFLOW and run['head_sha'] == producer['head_sha']
            and run['repository']['id'] == repo_id and run['head_repository']['id'] == repo_id
            and run['head_branch'] == 'main' and run['event'] == 'workflow_dispatch'
            and run['status'] == 'completed'):
        return False
    if run['conclusion'] == 'success':
        return True
    if run['conclusion'] != 'failure':
        return False
    # A failure on another architecture does not invalidate this producer job.
    attempt = str(int(run['run_attempt']))
    jobs = api(run_url+'/attempts/'+attempt+'/jobs?per_page=20')['jobs']
    return any(job['name'] == platform_job and job['conclusion'] == 'success'
               and any(step['name'] == 'Preserve the verified Qt Network dependency' and step['conclusion'] == 'success'
                       for step in job['steps']) for job in jobs)


def fetch(identity, output):
    """GitHub authenticates the producer and archive digest before any extraction."""
    name = key(identity)
    platform_job = PRODUCER_JOBS[identity['os'], identity['arch']]
    repo = api('')
    if repo['default_branch'] != 'main':
        raise ValueError('Qt Network producers require main as the default branch')
    workflow = api('/actions/workflows/' + quote(WORKFLOW))
    query = urlencode({'name': name, 'per_page': 10})
    artifacts = api('/actions/artifacts?' + query)['artifacts']
    for artifact in sorted(artifacts, key=lambda item: item['id'], reverse=True):
        if not trusted_artifact(artifact, name, workflow['id'], repo['id'], platform_job):
            continue
        archive_path = output.with_suffix('.download')
        temporary = output.with_suffix('.partial')
        output.parent.mkdir(parents=True, exist_ok=True)
        try:
            location = api('/actions/artifacts/' + str(int(artifact['id'])) + '/zip', redirect=True)
            # The temporary storage URL receives no repository token.
            with urlopen(location, timeout=30) as response, archive_path.open('wb') as archive:
                remaining = MAX_ARCHIVE_BYTES
                while block := response.read(1024 * 1024):
                    remaining -= len(block)
                    if remaining < 0:
                        raise ValueError('Actions archive exceeds the Qt Network reuse budget')
                    archive.write(block)
            if artifact.get('digest') != 'sha256:' + digest(archive_path):
                raise ValueError('Actions archive digest differs')
            with zipfile.ZipFile(archive_path) as archive:
                if archive.namelist() != [BUNDLE_NAME] or archive.getinfo(BUNDLE_NAME).file_size > MAX_ARCHIVE_BYTES:
                    raise ValueError('Unexpected Actions archive inventory')
                with archive.open(BUNDLE_NAME) as source, temporary.open('wb') as target:
                    shutil.copyfileobj(source, target)
            os.replace(temporary, output)
            return {'status': 'downloaded', 'run_id': artifact['workflow_run']['id'], 'artifact_id': artifact['id']}
        finally:
            archive_path.unlink(missing_ok=True)
            temporary.unlink(missing_ok=True)
    return {'status': 'miss'}


def download(identity, output, mode):
    started = time.monotonic()
    output.unlink(missing_ok=True)
    if mode == 'cold':
        result = {'status': 'forced-cold'}
    else:
        try:
            result = fetch(identity, output)
        except FAILURES as error:
            result = {'status': 'fetch-rejected', 'error_type': type(error).__name__}
    return dict(result, seconds=round(time.monotonic()-started, 2))


def use_bundle(prefix, bundle, output, manifest, *, publish=False):
    export = output/BUNDLE_NAME if publish else None
    if (prefix/RECEIPT).exists():
        raise ValueError('Update CI requires an unmodified SDK')
    if bundle.is_file():
        try:
            record = prepare(prefix, WORK/'restored', SOURCE_CACHE, bundle=bundle, bundle_identity=manifest)
            return 'hit', record
        except FAILURES as error:
            # Native installation rolls back before we use the clean source path.
            print(json.dumps({'status': 'bundle-rejected', 'error_type': type(error).__name__}), flush=True)
            if inputs(prefix) != manifest or (prefix/RECEIPT).exists():
                raise ValueError('Qt Network SDK rollback did not restore its original inputs') from error
            status = 'rejected-cold'
    else:
        status = 'cold'
    record = prepare(prefix, WORK/'source', SOURCE_CACHE, export=export, bundle_identity=manifest)
    return status, record


def write_record(path, record):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(record, sort_keys=True, indent=2) + '\n', encoding='utf-8')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['inputs', 'fetch', 'prepare'])
    parser.add_argument('--qt-prefix', type=Path)
    parser.add_argument('--inputs', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--incoming', type=Path)
    parser.add_argument('--mode', choices=['auto', 'cold'], default='auto')
    parser.add_argument('--publish', action='store_true')
    args = parser.parse_args()
    if args.publish and args.command != 'prepare':
        parser.error('--publish only applies to preparation')
    if ((args.command != 'fetch' and args.qt_prefix is None)
            or (args.command == 'fetch' and args.inputs is None)):
        parser.error('Provide --qt-prefix for SDK operations or --inputs for fetching')
    if args.command != 'inputs' and args.output is None:
        parser.error('Provide --output for fetching or preparation')
    started = time.monotonic()
    if args.command == 'inputs':
        print(json.dumps(inputs(args.qt_prefix)), flush=True)
        return
    if args.command == 'fetch':
        manifest = json.loads(args.inputs.read_bytes())
        result = download(manifest, args.output/BUNDLE_NAME, args.mode)
        write_record(args.output/'fetch.json', result)
        print(json.dumps(result), flush=True)
        return
    token = os.environ.pop('GITHUB_TOKEN', None)
    manifest = inputs(args.qt_prefix)
    if args.incoming is None:
        incoming = args.output/'incoming'
        if token:
            os.environ['GITHUB_TOKEN'] = token
        try:
            downloaded = download(manifest, incoming/BUNDLE_NAME, args.mode)
        finally:
            os.environ.pop('GITHUB_TOKEN', None)
    else:
        incoming = args.incoming
        report = incoming/'fetch.json'
        downloaded = json.loads(report.read_bytes()) if report.is_file() else {'status': 'miss'}
    preparing = time.monotonic()
    status, record = use_bundle(args.qt_prefix, incoming/BUNDLE_NAME, args.output, manifest, publish=args.publish)
    metrics = {'schema_version': 1, 'status': status, 'fetch': downloaded, 'key': key(manifest),
               'seconds': round(time.monotonic()-started, 2), 'prepare_seconds': round(time.monotonic()-preparing, 2),
               'network_sha256': record['network_sha256']}
    write_record(args.output/'qt-network-prepare.json', metrics)
    if 'GITHUB_OUTPUT' in os.environ:
        with Path(os.environ['GITHUB_OUTPUT']).open('a', encoding='utf-8') as output:
            if (args.output/BUNDLE_NAME).is_file():
                output.write('bundle=' + metrics['key'] + '\n')
    print(json.dumps(metrics), flush=True)


if __name__ == '__main__':
    with utf8_console():
        main()
