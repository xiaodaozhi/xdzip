# scripts/publish-release.py

"""Publish a complete verified immutable release, then download it again."""
import argparse
import json
from pathlib import Path
import re
import runpy
import subprocess
from release_protocol import reject_corresponding_source_archives, sha256, validate

ROOT = Path(__file__).resolve().parents[1]


def load_notes(path, version):
    source = path.read_text(encoding='utf-8')
    metadata = re.findall(r'^<!-- xdzip-release (\{[^\n]+\}) -->$', source, re.M)
    if len(metadata) != 1:
        raise ValueError('Release notes require one xdzip-release version/build header')
    identity = json.loads(metadata[0])
    if (set(identity) != {'version', 'build'} or type(identity['build']) is not int or
            identity['version'] != version['display_version'] or identity['build'] != version['build']):
        raise ValueError('Release notes version/build differs from VERSION.json')
    body = re.sub(r'\A(?:<!--[ \t]+[^\n]+[ \t]+-->[ \t]*\n\s*)+', '', source).strip()
    heading = '# xdzip ' + version['display_version']
    if not body.startswith(heading + '\n') or not body[len(heading):].strip():
        raise ValueError('Release notes require a matching heading and reviewed content')
    return body + '\n'


def comparable_notes(body):
    return body.replace('\r\n', '\n').rstrip()


def gh(*args, capture=False):
    return subprocess.run(['gh', *map(str, args)], check=True, text=True, encoding='utf-8', capture_output=capture).stdout


def require_release_state(state, tag, *, draft, prerelease):
    if (not isinstance(state, dict) or state.get('tagName') != tag or
            state.get('isDraft') is not draft or state.get('isPrerelease') is not prerelease):
        phase = 'draft' if draft else 'public'
        channel = 'prerelease' if prerelease else 'stable release'
        raise ValueError(f'{phase} release state for {tag} must match the expected {channel}; '
                         'a public release may already exist and requires owner review')


def read_release_state(tag, repository):
    return json.loads(gh('release', 'view', tag, '--repo', repository, '--json',
                         'tagName,isDraft,isPrerelease,assets,body', capture=True))


def require_latest_stable_release(tag, repository):
    releases = json.loads(gh('release', 'list', '--repo', repository, '--limit', '100',
                             '--exclude-drafts', '--exclude-pre-releases',
                             '--json', 'tagName,isLatest,isPrerelease', capture=True))
    if not any(isinstance(release, dict) and release.get('tagName') == tag and
               release.get('isLatest') is True and release.get('isPrerelease') is False
               for release in releases):
        raise ValueError(f'Stable release {tag} is not marked as the latest public release')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path)
    parser.add_argument('--tag')
    parser.add_argument('--repository')
    parser.add_argument('--notes-file', type=Path, help='Reviewed version-specific notes; defaults to release-notes/<version>.md')
    parser.add_argument('--check-notes', action='store_true', help='Validate notes without contacting GitHub')
    parser.add_argument('--draft-only', action='store_true')
    args = parser.parse_args()
    version = runpy.run_path(str(ROOT/'scripts/sync-version.py'))['version_info']()
    if args.tag and args.tag != 'v'+version['display_version']:
        raise ValueError('This workflow only publishes the exact configured version tag')
    if version['channel'] not in {'alpha', 'beta', 'rc', 'stable'}:
        raise ValueError('Development channel builds cannot be published')
    prerelease = version['channel'] != 'stable'
    reviewed = load_notes(args.notes_file or ROOT/'release-notes'/(version['display_version']+'.md'), version)
    if args.check_notes:
        print('Version-specific release notes verified: '+version['display_version'])
        return
    if not args.tag or not args.repository:
        parser.error('--tag and --repository are required for publishing')
    if args.directory:
        reject_corresponding_source_archives(args.directory.resolve().iterdir())
    work = ROOT/'build/release-publish'
    work.mkdir(parents=True, exist_ok=True)
    notes = work/'notes.md'
    notes.write_text(reviewed, encoding='utf-8')
    query = subprocess.run(['gh', 'release', 'view', args.tag, '--repo', args.repository, '--json',
                            'tagName,isDraft,isPrerelease,assets,body'],
                           capture_output=True, text=True, encoding='utf-8')
    if query.returncode:
        create_args = ['release', 'create', args.tag, '--repo', args.repository, '--verify-tag', '--draft']
        if prerelease:
            create_args.append('--prerelease')
        create_args.extend(['--title', 'xdzip '+version['display_version'], '--notes-file', notes])
        gh(*create_args)
        state = read_release_state(args.tag, args.repository)
    else:
        state = json.loads(query.stdout)
    if isinstance(state, dict) and state.get('isDraft') is False:
        raise ValueError('Public release is immutable; publish a new version for revisions')
    require_release_state(state, args.tag, draft=True, prerelease=prerelease)
    if comparable_notes(state['body']) != comparable_notes(reviewed):
        raise ValueError('Draft notes differ from the reviewed file; reconcile them explicitly before publishing')
    if args.draft_only: return
    if not args.directory: raise ValueError('Missing complete asset directory')
    directory = args.directory.resolve()
    manifest = validate(json.loads((directory/'release.json').read_text(encoding='utf-8')), directory)
    from release_updates import verify_release_updates, verify_cli_updates
    verify_release_updates(directory, manifest)
    verify_cli_updates(directory, manifest)
    if manifest['tag'] != args.tag or manifest['repository'] != args.repository:
        raise ValueError('Release identity mismatch')
    expected = {}
    for line in (directory/'SHA256SUMS').read_text().splitlines():
        digest, name = line.split('  ', 1)
        path = directory/name
        if path.parent != directory or not path.is_file() or sha256(path) != digest:
            raise ValueError('Invalid complete release checksum set')
        expected[name] = digest
    if set(expected) != {path.name for path in directory.iterdir() if path.is_file() and path.name != 'SHA256SUMS'}:
        raise ValueError('Checksums do not cover every attachment')
    # Reruns may replace private draft assets. Public assets are never replaced.
    for asset in state['assets']:
        gh('release', 'delete-asset', args.tag, asset['name'], '--repo', args.repository, '--yes')
    files = sorted(path for path in directory.iterdir() if path.is_file())
    gh('release', 'upload', args.tag, *files, '--repo', args.repository)
    for phase in ['draft-download', 'public-download']:
        destination = work/phase
        if destination.exists(): raise ValueError('Use a fresh publishing workspace for a retry')
        destination.mkdir()
        gh('release', 'download', args.tag, '--repo', args.repository, '--dir', destination)
        for name, digest in expected.items():
            if sha256(destination/name) != digest: raise ValueError('Downloaded attachment hash mismatch')
        validate(json.loads((destination/'release.json').read_text()), destination)
        if phase == 'draft-download':
            publish_args = ['release', 'edit', args.tag, '--repo', args.repository, '--draft=false',
                            '--notes-file', notes]
            if prerelease:
                publish_args.extend(['--prerelease', '--latest=false'])
            else:
                publish_args.append('--latest')
            gh(*publish_args)
        require_release_state(read_release_state(args.tag, args.repository), args.tag,
                              draft=False, prerelease=prerelease)
    if not prerelease:
        require_latest_stable_release(args.tag, args.repository)
    print('Published and downloaded all assets with matching hashes: '+args.tag)


if __name__ == '__main__': main()
