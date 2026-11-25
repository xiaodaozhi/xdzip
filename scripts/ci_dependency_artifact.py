"""Verify and download an exact dependency artifact without forwarding the API token."""
import json
import os
import shutil
from urllib.error import HTTPError, URLError
from urllib.parse import quote, urlencode, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener, urlopen
import zipfile

from qt_network_security import digest


def api(path, *, redirect=False, token=None):
    url = os.environ['GITHUB_API_URL'] + '/repos/' + os.environ['GITHUB_REPOSITORY'] + path
    request = Request(url, headers={'Authorization': 'Bearer ' + (token or os.environ['GITHUB_TOKEN']),
                                    'Accept': 'application/vnd.github+json'})
    if not redirect:
        with urlopen(request, timeout=15) as response:
            return json.load(response)

    class NoRedirect(HTTPRedirectHandler):
        def redirect_request(self, *_args, **_kwargs):
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


def download_archive(request, artifact, output, member, maximum):
    archive_path = output.with_suffix('.download')
    temporary = output.with_suffix('.partial')
    output.parent.mkdir(parents=True, exist_ok=True)
    try:
        location = request('/actions/artifacts/' + str(int(artifact['id'])) + '/zip', redirect=True)
        with urlopen(location, timeout=30) as response, archive_path.open('wb') as archive:
            remaining = maximum
            while block := response.read(1024 * 1024):
                remaining -= len(block)
                if remaining < 0:
                    raise ValueError('Actions archive exceeds the dependency reuse budget')
                archive.write(block)
        if artifact.get('digest') != 'sha256:' + digest(archive_path):
            raise ValueError('Actions archive digest differs')
        with zipfile.ZipFile(archive_path) as archive:
            if archive.namelist() != [member] or not 0 < archive.getinfo(member).file_size <= maximum:
                raise ValueError('Unexpected Actions archive inventory')
            with archive.open(member) as source, temporary.open('wb') as target:
                shutil.copyfileobj(source, target)
        os.replace(temporary, output)
    finally:
        archive_path.unlink(missing_ok=True)
        temporary.unlink(missing_ok=True)


def release_producer(request, artifact, repo, workflow, job_name, upload_step):
    producer = artifact['workflow_run']
    if producer['repository_id'] != repo['id'] or producer['head_repository_id'] != repo['id']:
        return False
    run_path = '/actions/runs/' + str(int(producer['id']))
    run = request(run_path)
    if not (run['id'] == producer['id'] and run['workflow_id'] == workflow['id']
            and run['path'] == '.github/workflows/release.yml' and run['head_sha'] == producer['head_sha']
            and run['repository']['id'] == repo['id'] and run['head_repository']['id'] == repo['id']
            and run['event'] == 'workflow_dispatch' and run['status'] == 'completed'
            and run['conclusion'] in ('success', 'failure')):
        return False
    # A preflight's immutable source may be admitted once that exact commit is on main.
    revision = run['head_sha']
    if len(revision) != 40 or any(c not in '0123456789abcdef' for c in revision):
        return False
    comparison = request('/compare/' + revision + '...main?per_page=1')
    if comparison['merge_base_commit']['sha'] != revision or comparison['status'] not in ('ahead', 'identical'):
        return False
    attempt = str(int(run['run_attempt']))
    jobs = request(run_path+'/attempts/'+attempt+'/jobs?per_page=50')['jobs']
    return any(job['name'] == job_name and job['conclusion'] == 'success' and
               any(step['name'] == upload_step and step['conclusion'] == 'success' for step in job['steps'])
               for job in jobs)


def fetch_release_bundle(request, name, output, runner, upload_step, maximum):
    repo = request('')
    if repo['default_branch'] != 'main':
        raise ValueError('Dependency producers require main as the default branch')
    workflow = request('/actions/workflows/' + quote('release.yml'))
    artifacts = request('/actions/artifacts?' + urlencode({'name': name, 'per_page': 10}))['artifacts']
    for artifact in sorted(artifacts, key=lambda item: item['id'], reverse=True):
        if (artifact['name'] == name and not artifact['expired'] and 0 < artifact['size_in_bytes'] <= maximum and
                release_producer(request, artifact, repo, workflow, f'Native dependencies ({runner})', upload_step)):
            download_archive(request, artifact, output, output.name, maximum)
            return {'status': 'downloaded', 'run_id': artifact['workflow_run']['id'], 'artifact_id': artifact['id']}
    return {'status': 'miss'}
