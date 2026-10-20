# scripts/publish-release.py

"""Publish only a complete verified immutable prerelease, then download it again."""
import argparse
import json
from pathlib import Path
import runpy
import subprocess
from release_protocol import sha256, validate

ROOT = Path(__file__).resolve().parents[1]


def gh(*args, capture=False):
    return subprocess.run(['gh', *map(str, args)], check=True, text=True, capture_output=capture).stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path)
    parser.add_argument('--tag', required=True)
    parser.add_argument('--repository', required=True)
    parser.add_argument('--draft-only', action='store_true')
    args = parser.parse_args()
    version = runpy.run_path(str(ROOT/'scripts/sync-version.py'))['version_info']()
    if args.tag != 'v'+version['display_version'] or version['channel'] == 'stable':
        raise ValueError('This workflow only publishes the exact configured prerelease tag')
    work = ROOT/'build/release-publish'
    work.mkdir(parents=True, exist_ok=True)
    notes = work/'notes.md'
    notes.write_text(f'''xdzip {version['display_version']}，首个实验性跨平台预发布。

- 修复弹窗激活使主窗口退出最大化的问题。
- 每个登录会话共用一个进程和托盘，支持在新标签页或新窗口打开归档。
- 关闭页面后任务继续运行，明确退出时执行取消、清理与异步关闭。
- 提供 Windows x86_64 便携 ZIP 和安装器，macOS 14+ arm64 app ZIP 和 DMG，Ubuntu 24.04 x86_64 X11 运行包和 deb。

Windows 包包含应用目录内 VC++ 运行库及图形兼容库。macOS 仅 ad-hoc 完整性签名，未经 Developer ID 签名和 Apple 公证。Linux 要求 Ubuntu 24.04、glibc 2.39、X11。

`release.json` 是更新协议 v1，按语义版本、允许渠道、平台、架构及包类型选择资产。预发布列表须分页获取，不能依赖 `/releases/latest`。每个资产以 SHA-256 验证；本版本尚未实现自动更新器。

本版本仍属实验性，独立安全审查、独立许可审查及完整桌面人工验收尚未完成，跟踪见 #4。自动化记录不等同于人工视觉验收或签名发行。对应源码、通知、许可、SBOM 与重建材料见附件。

Experimental prerelease. Independent security/licensing review and full manual desktop acceptance remain pending. macOS uses ad-hoc integrity signing only, without Developer ID or notarization. No automatic updater is included.
''', encoding='utf-8')
    query = subprocess.run(['gh', 'release', 'view', args.tag, '--repo', args.repository, '--json', 'isDraft,assets'],
                           capture_output=True, text=True)
    if query.returncode:
        gh('release', 'create', args.tag, '--repo', args.repository, '--verify-tag', '--draft', '--prerelease',
           '--title', 'xdzip '+version['display_version'], '--notes-file', notes)
        state = dict(isDraft=True, assets=[])
    else:
        state = json.loads(query.stdout)
    if not state['isDraft']:
        raise ValueError('Public release is immutable; publish a new version for revisions')
    if args.draft_only: return
    if not args.directory: raise ValueError('Missing complete asset directory')
    directory = args.directory.resolve()
    manifest = validate(json.loads((directory/'release.json').read_text(encoding='utf-8')), directory)
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
            gh('release', 'edit', args.tag, '--repo', args.repository, '--draft=false', '--prerelease', '--latest=false', '--notes-file', notes)
    print('Published and downloaded all assets with matching hashes: '+args.tag)


if __name__ == '__main__': main()
