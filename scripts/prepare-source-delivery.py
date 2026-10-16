#!/usr/bin/env python3
"""Collect pinned Qt source archives, exact project source and locked Rust vendor inputs."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import urllib.request
from deployment_audit import digest, within

ROOT = Path(__file__).resolve().parents[1]


def archive_source(item, cache):
    target = cache/item['file']
    if not target.exists():
        temporary = target.with_suffix(target.suffix+'.partial')
        try:
            with urllib.request.urlopen(item['url'], timeout=60) as source, temporary.open('wb') as output:
                shutil.copyfileobj(source, output)
            if digest(temporary) != item['sha256']:
                raise ValueError('Pinned Qt source checksum mismatch')
            temporary.replace(target)
        finally:
            temporary.unlink(missing_ok=True)
    if digest(target) != item['sha256']:
        raise ValueError('Cached Qt source checksum mismatch')
    # Inspect only; never unpack third-party archive paths into the workspace.
    with tarfile.open(target, 'r:xz') as archive:
        names = archive.getnames()
        prefix = item['file'].removesuffix('.tar.xz')
        if not names or any(not (name == prefix or name.startswith(prefix+'/')) or
                            '..' in Path(name).parts for name in names):
            raise ValueError('Unexpected source archive layout')
        if 'source_commit' in item:
            tag = archive.getmember(prefix+'/.tag')
            if not tag.isfile() or archive.extractfile(tag).read().decode('ascii').strip() != item['source_commit']:
                raise ValueError('Pinned source commit marker mismatch')
    return target


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--cache', type=Path, default=ROOT/'build/source-cache')
    parser.add_argument('--qt-prefix', type=Path, help='Also collect installed SDK correspondence evidence')
    args = parser.parse_args()
    output = args.output.resolve()
    if not within(output, ROOT/'build') or output.exists():
        parser.error('Use a new output directory under build/')
    if subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT).strip():
        parser.error('Source delivery requires clean committed inputs')
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    config = json.loads((ROOT/'scripts/corresponding-sources.json').read_text())
    args.cache.mkdir(parents=True, exist_ok=True)
    with ThreadPoolExecutor(max_workers=4) as pool:
        sources = list(pool.map(lambda item: archive_source(item, args.cache), config['archives']))
    output.mkdir(parents=True)
    materials = []
    for source, item in zip(sources, config['archives']):
        shutil.copy2(source, output/source.name)
        materials.append(dict(item))
    shutil.copy2(ROOT/'scripts/corresponding-sources.json', output/'corresponding-sources.json')
    patch_dir = output/'qt-patches'
    patch_dir.mkdir()
    for item in config.get('sdk_patches', []):
        patch = ROOT/item['patch']
        if digest(patch) != item['patch_sha256']:
            raise ValueError('Pinned SDK patch checksum mismatch')
        shutil.copy2(patch, patch_dir/patch.name)
    if args.qt_prefix:
        from qt_sdk_correspondence import collect
        evidence = collect(args.qt_prefix.resolve(), args.cache.resolve())
        (output/'qt-sdk-correspondence.json').write_text(json.dumps(evidence, indent=2)+'\n')
    project = output/'xdzip-source.tar'
    subprocess.run(['git', 'archive', '--format=tar', '--output', str(project), commit], cwd=ROOT, check=True)
    # --locked binds checksummed registry sources to Cargo.lock, including build
    # dependencies; vendor does not run third-party build scripts.
    vendor = output/'vendor'
    result = subprocess.run(['cargo', 'vendor', '--locked', '--versioned-dirs', str(vendor)],
                            cwd=ROOT, capture_output=True, text=True, check=True)
    config_dir = output/'cargo-config.toml'
    config_dir.write_text(result.stdout.replace(str(vendor), 'vendor'), encoding='utf-8')
    vendor_archive = output/'rust-vendor.tar.gz'
    with tarfile.open(vendor_archive, 'w:gz') as archive:
        archive.add(vendor, arcname='vendor')
    shutil.rmtree(vendor)
    instructions = '''# 离线重建与 Qt 动态库替换

本目录是可复核的源码输入，不是许可审批或已重建全部依赖的声明。

1. 将 xdzip-source.tar 和 rust-vendor.tar.gz 解到同一个工作目录，将 cargo-config.toml
   复制为该目录 .cargo/config.toml。保持 Cargo.lock，不执行 cargo update。
2. 按仓库固定的 Rust、CMake、编译器和 Qt 6.8.2 工具链构建。Qt 源码包保留原始归档，
   包含相应模块的第三方源码和通知；构建 Qt 时需匹配 SDK 的 configure/features/补丁。
   qt-patches/ 保留本地 SDK 消费端 CMake 补丁，按 corresponding-sources.json\n   的前后哈希核对；不要将消费端补丁当作 Qt 二进制源码变更。可选\n   qt-sdk-correspondence.json 保存实际 SDK 配置、来源文件核对及缺口。\n   现有 SDK 与上游源码逐项等价性、平台构建配置及再分发库仍须审查，不能假设一致。
3. 从源码工作目录运行 cargo test --workspace --locked --offline。
4. 将 QT_PREFIX 指向重建的兼容 Qt SDK，运行：
   cmake -S . -B build/relink -G Ninja -DCMAKE_BUILD_TYPE=Release \\
     -DCMAKE_PREFIX_PATH="$QT_PREFIX" -DXDZIP_BUILD_DESKTOP_PREVIEW=ON
   cmake --build build/relink --parallel 3
   ctest --test-dir build/relink --output-on-failure
   macOS 另加 -DCMAKE_OSX_ARCHITECTURES=arm64 -DCMAKE_OSX_DEPLOYMENT_TARGET=14.0。
5. 应用动态链接 Qt，用户可重建后重新部署兼容的修改版 Qt。macOS 对替换后的所有
   Mach-O、framework 和应用重新做 ad-hoc 签名，否则 ARM64 可能拒绝加载。
   不要求用户拥有本项目的 Developer ID 私钥。勿关闭 SIP 或系统安全保护。

该流程不宣称任意 ABI 不兼容的 Qt 都可替换。独立安全审查、许可分支选择、完整
平台对应源码与配置核对、修改版 Qt 的行为验证仍在 WP-12 准入中单独登记。
'''
    (output/'REBUILD.md').write_text(instructions, encoding='utf-8')
    files = [{'file': p.relative_to(output).as_posix(), 'sha256': digest(p), 'bytes': p.stat().st_size}
             for p in sorted(output.rglob('*')) if p.is_file()]
    report = {'schema_version': 1, 'revision': commit, 'qt_version': config['qt_version'],
              'source_delivery': 'collected-not-compliance-verified', 'materials': materials, 'files': files,
              'limits': ['Pinned Qt module archives and locked Cargo vendor inputs, not a proof of SDK build equivalence.',
                         'Exact linked components, platform runtimes, license selection and independent review remain pending.']}
    (output/'source-delivery.json').write_text(json.dumps(report, indent=2)+'\n')
    print('Collected project source, locked Rust vendor inputs and four verified Qt source archives')


if __name__ == '__main__':
    main()
