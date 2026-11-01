#!/usr/bin/env python3
"""Collect pinned Qt source archives, exact project source and locked Rust vendor inputs."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import gzip
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tarfile
import urllib.request
from deployment_audit import digest, within
from native_runtime_materials import configuration, fetch
from sparkle_bundle import collect_sources as collect_sparkle_sources
from rar_mapping_materials import collect as collect_rar_mapping_materials
from qt_source_identity import commit_object_path, source_identity
from vendor_notice_materials import collect as collect_vendor_notice_materials
from qt_psl_source import collect as collect_qt_psl_source
from qt_tika_materials import collect as collect_qt_tika_source

ROOT = Path(__file__).resolve().parents[1]
ISO639_DIRECTORY = 'tools/cldr-code/src/main/resources/org/unicode/cldr/util/data/'
ISO639_TABLES = frozenset(ISO639_DIRECTORY + name for name in (
    'iso-639-3.tab', 'iso-639-3_Name_Index.tab', 'iso-639-3-macrolanguages.tab',
    'iso-639-3_Retirements.tab', 'iso-639-3-version.tab'))


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
            identity = source_identity(item)
            tag = archive.getmember(prefix+'/.tag')
            if not tag.isfile() or archive.extractfile(tag).read().decode('ascii').strip() != identity['archive_marker']:
                raise ValueError('Pinned source commit or tree marker mismatch')
    return target


def collect_source_identity_proofs(output, items):
    for item in items:
        if item.get('archive_marker_type') != 'tree':
            continue
        source_identity(item)
        proof = commit_object_path(item)
        destination = output / item['commit_object']
        parents = [output.joinpath(*Path(item['commit_object']).parts[:index])
                   for index in range(1, len(Path(item['commit_object']).parts) + 1)]
        if any(part.is_symlink() for part in parents) or not within(destination, output):
            raise ValueError('Git commit object proof destination escapes output')
        destination.parent.mkdir(parents=True, exist_ok=True)
        if destination.exists() or destination.is_symlink():
            raise ValueError('Duplicate Git commit object proof destination')
        shutil.copy2(proof, destination)


def collect_native_sources(output, cache):
    # Verify and inspect the pinned archive without extracting third-party paths.
    config = configuration()
    license_item, source_item = config['icu'], config['icu_source']
    license_file = fetch(license_item, cache)
    source_file = fetch(source_item, cache)
    with tarfile.open(source_file, 'r:gz') as archive:
        members = archive.getmembers()
        if not members or any('\\' in member.name or
                              PurePosixPath(member.name).is_absolute() or
                              '..' in PurePosixPath(member.name).parts or
                              not PurePosixPath(member.name).parts or
                              PurePosixPath(member.name).parts[0] != 'icu'
                              for member in members):
            raise ValueError('Unexpected ICU source archive layout')

        def read_member(name):
            matching = [member for member in members if member.name == name]
            if len(matching) != 1 or not matching[0].isfile() or matching[0].size > 1024 * 1024:
                raise ValueError('Missing or ambiguous ICU source identity file')
            return archive.extractfile(matching[0]).read()
        if read_member('icu/LICENSE') != license_file.read_bytes():
            raise ValueError('ICU source license differs from pinned notice')
        version_header = read_member('icu/source/common/unicode/uvernum.h').decode('utf-8')
        versions = re.findall(r'^#define U_ICU_VERSION "([^"]+)"\s*$', version_header, re.MULTILINE)
        if versions != [source_item['version']] or source_item['version'] != license_item['version']:
            raise ValueError('ICU source version mismatch')
    for path in (license_file, source_file):
        shutil.copy2(path, output/path.name)
    shutil.copy2(ROOT/'scripts/native-runtime-sources.json', output/'native-runtime-sources.json')
    return [dict(license_item), dict(source_item)]


def collect_icu_data_sources(output, cache):
    """Keep editable ICU/CLDR inputs as well as the pre-generated ICU archive."""
    items = configuration()['icu_data_sources']
    if len(items) != 2 or {item['module'] for item in items} != {'icu-data-source', 'cldr-data-source'}:
        raise ValueError('Expected the pinned ICU and CLDR data sources')
    sources = []
    for item in items:
        source = fetch(item, cache)
        root = item['archive_root']
        if PurePosixPath(root).name != root or '\\' in root or not item['anchors']:
            raise ValueError('Unexpected ICU data source archive root')
        with tarfile.open(source, 'r:gz') as archive:
            members = archive.getmembers()
            if not members or any('\\' in member.name or
                                  not PurePosixPath(member.name).parts or
                                  PurePosixPath(member.name).is_absolute() or
                                  '..' in PurePosixPath(member.name).parts or
                                  PurePosixPath(member.name).parts[0] != root or
                                  not (member.isfile() or member.isdir())
                                  for member in members):
                raise ValueError('Unexpected ICU data source archive layout')
            names = [member.name for member in members]
            if len(names) != len(set(names)):
                raise ValueError('Duplicate ICU data source archive member')
            for relative, expected in item['anchors'].items():
                if not PurePosixPath(relative).parts or '\\' in relative or PurePosixPath(relative).is_absolute() or '..' in PurePosixPath(relative).parts:
                    raise ValueError('Unexpected ICU data source anchor')
                found = [member for member in members if member.name == root+'/'+relative]
                if len(found) != 1 or not found[0].isfile() or found[0].size > 8 * 1024 * 1024:
                    raise ValueError('Missing or ambiguous ICU data source anchor')
                actual = hashlib.sha256(archive.extractfile(found[0]).read()).hexdigest()
                if actual != expected:
                    raise ValueError('ICU data source anchor checksum mismatch')
            if item['module'] == 'cldr-data-source':
                iso_names = {member.name.removeprefix(root+'/') for member in members
                             if member.isfile() and 'iso-639-3' in member.name.lower()}
                if iso_names != ISO639_TABLES:
                    raise ValueError('Unexpected ISO 639-3 files in CLDR source archive')
        sources.append(source)
    records = []
    for item, source in zip(items, sources):
        record = dict(item)
        if item['module'] == 'cldr-data-source':
            if not source.name.endswith('.tar.gz'):
                raise ValueError('Unexpected CLDR source archive filename')
            name = source.name.removesuffix('.tar.gz') + '-without-iso6393.tar.gz'
            target = output/name
            root = item['archive_root']
            filter_source_tar(source, target, lambda path: path.removeprefix(root+'/') not in ISO639_TABLES)
            record['upstream_url'] = record.pop('url')
            if 'source_tree' in record:
                record['upstream_source_tree'] = record.pop('source_tree')
            record.update(file=name, sha256=digest(target), upstream_file=item['file'],
                          upstream_archive_sha256=item['sha256'], complete_upstream_tree=False,
                          omitted_files=sorted(ISO639_TABLES))
        else:
            shutil.copy2(source, output/source.name)
        records.append(record)
    return records


def filter_source_tar(source, target, include):
    """Create a labelled subset without extracting any archive paths."""
    temporary = target.with_suffix(target.suffix+'.partial')
    if target.exists() or target.is_symlink() or temporary.exists() or temporary.is_symlink():
        raise ValueError('Source subset destination already exists')
    try:
        with tarfile.open(source, 'r:gz') as original, temporary.open('xb') as raw:
            with gzip.GzipFile(filename='', mode='wb', fileobj=raw, mtime=0) as compressed:
                with tarfile.open(fileobj=compressed, mode='w') as subset:
                    for member in original:
                        if not include(member.name):
                            continue
                        if member.isfile():
                            with original.extractfile(member) as data:
                                subset.addfile(member, data)
                        elif member.isdir():
                            subset.addfile(member)
                        else:
                            raise ValueError('Unexpected source subset member type')
        temporary.replace(target)
    finally:
        temporary.unlink(missing_ok=True)


def collect_qt_cldr_source(output, qtbase, cache):
    """Deliver the pinned CLDR common tree required by Qt's locale generator."""
    config_path = ROOT/'scripts/qt-cldr-sources.json'
    config = json.loads(config_path.read_text(encoding='utf-8'))
    item = config['cldr']
    if (config['schema_version'] != 1 or qtbase.name != config['qtbase_archive'] or
            digest(qtbase) != config['qtbase_archive_sha256']):
        raise ValueError('Qt base source identity mismatch for CLDR')
    if (not re.fullmatch(r'[0-9]+', item['version']) or
            not re.fullmatch(r'[0-9a-f]{40}', item['source_commit']) or
            not re.fullmatch(r'[0-9a-f]{40}', item['common_tree']) or
            item['archive_root'] != 'cldr-'+item['source_commit']):
        raise ValueError('Unexpected Qt CLDR source identity')
    prefix = qtbase.name.removesuffix('.tar.xz')
    with tarfile.open(qtbase, 'r:xz') as archive:
        for relative in ('src/corelib/text/qlocale_data_p.h',
                         'src/corelib/time/qtimezoneprivate_data_p.h'):
            found = [member for member in archive.getmembers() if member.name == prefix+'/'+relative]
            if len(found) != 1 or not found[0].isfile() or found[0].size > 32 * 1024 * 1024:
                raise ValueError('Missing or ambiguous Qt CLDR generated file')
            marker = ('Common Locale Data Repository v'+item['version']).encode('ascii')
            if marker not in archive.extractfile(found[0]).read():
                raise ValueError('Qt CLDR generated version mismatch')

    upstream = fetch(item, cache)
    root = item['archive_root']
    with tarfile.open(upstream, 'r:gz') as archive:
        members = archive.getmembers()
        names = [member.name for member in members]
        if (not members or len(names) != len(set(names)) or
                any('\\' in name or not PurePosixPath(name).parts or
                    PurePosixPath(name).is_absolute() or '..' in PurePosixPath(name).parts or
                    PurePosixPath(name).parts[0] != root or not (member.isfile() or member.isdir())
                    for name, member in zip(names, members))):
            raise ValueError('Unexpected Qt CLDR source archive layout')
        required = {'LICENSE', 'common/main/root.xml', 'common/main/en.xml',
                    'common/supplemental/windowsZones.xml', 'common/bcp47/timezone.xml'}
        if set(item['anchors']) != required:
            raise ValueError('Unexpected Qt CLDR source anchors')
        for relative, expected in item['anchors'].items():
            found = [member for member in members if member.name == root+'/'+relative]
            if len(found) != 1 or not found[0].isfile() or found[0].size > 8 * 1024 * 1024:
                raise ValueError('Missing or ambiguous Qt CLDR source anchor')
            if hashlib.sha256(archive.extractfile(found[0]).read()).hexdigest() != expected:
                raise ValueError('Qt CLDR source anchor checksum mismatch')
        if any(member.isfile() and member.name.startswith(root+'/common/') and
               'iso-639-3' in member.name.lower() for member in members):
            raise ValueError('Unexpected ISO 639-3 table in Qt CLDR common tree')
    name = 'cldr-'+item['version']+'-common-only.tar.gz'
    target = output/name
    filter_source_tar(upstream, target, lambda path: path in (root, root+'/LICENSE', root+'/common') or
                      path.startswith(root+'/common/'))
    shutil.copy2(config_path, output/'qt-cldr-sources.json')
    return {'kind': 'qt-cldr-common-source', 'file': name, 'sha256': digest(target),
            'version': item['version'], 'scope': 'common and LICENSE only',
            'upstream_url': item['url'], 'upstream_archive_sha256': item['sha256'],
            'source_commit': item['source_commit'], 'upstream_common_tree': item['common_tree'],
            'iso_639_3_table_included': False}


def write_vendor_config(output):
    """Ask Cargo for a relative output path and reject unexpected source mappings."""
    result = subprocess.run(['cargo', 'vendor', '--locked', '--versioned-dirs',
                             '--manifest-path', str(ROOT/'Cargo.toml'), 'vendor'],
                            cwd=output, capture_output=True, text=True, check=True)
    lines = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    expected = ['[source.crates-io]', 'replace-with = "vendored-sources"',
                '[source.vendored-sources]', 'directory = "vendor"']
    if lines != expected:
        raise ValueError('Unexpected or non-relocatable Cargo vendor configuration')
    (output/'cargo-config.toml').write_text(result.stdout, encoding='utf-8', newline='\n')


def rebuild_instructions():
    """Keep offline product rebuild and separately prepared test data distinct."""
    return '''# 离线重建与 Qt 动态库替换

本目录是可复核的源码输入，不是许可审批或已重建全部依赖的声明。

1. 将 xdzip-source.tar 和 rust-vendor.tar.gz 解到同一个工作目录，将 cargo-config.toml
   复制为该目录 .cargo/config.toml。保持 Cargo.lock，不执行 cargo update。
   rust-vendor-notices/ 按包列出固定上游的补充许可原文与来源，不修改 vendor 内容。
   English: rust-vendor-notices/ indexes pinned upstream notices by source package
   without altering the vendored files or approving license compliance.
2. rust-src-1.99.0.tar.xz 是固定 Rust 标准库原始源码组件，保留上游归档与通知。\n   按仓库固定的 Rust、CMake、编译器和 Qt 6.8.2 工具链构建。Qt 源码包保留原始归档，
   包含相应模块的第三方源码和通知；构建 Qt 时需匹配 SDK 的 configure/features/补丁。
   qt-patches/ 保留本地 SDK 消费端 CMake 补丁，按 corresponding-sources.json\n   的前后哈希核对；不要将消费端补丁当作 Qt 二进制源码变更。可选\n   qt-sdk-correspondence.json 保存实际 SDK 配置、来源文件核对及缺口。\n   现有 SDK 与上游源码逐项等价性、平台构建配置及再分发库仍须审查，不能假设一致。
   scripts/qt-source-identity/ 保留固定 Git 提交对象输入，用于离线验证源码标记与提交的关系；\n   它不证明归档覆盖整个 Git 树或 SDK 由该树重建。
   qt-public-suffix-list/ 包含 Qt 固定的可编辑列表和 MPL-2.0 原文；收集时用 Qt 归档内
   生成器重现并核对 C++ 和测试二进制数据，不据此声称整个 SDK 的构建等价。
   English: qt-public-suffix-list/ includes the pinned editable input and MPL-2.0 text.
   Both generated forms are checked against the Qt source archive during collection.
   cldr-46-common-only.tar.gz 保留 Qt 区域数据生成所需 CLDR 46 的 common 数据和许可；
   来源提交和原归档摘要见 qt-cldr-sources.json。Qt 的 qlocalexml2cpp.py 还需要
   单独的 ISO 639-3 代码表，该表未在本交付中提供。试验用候选表的来源和摘要见
   qt-cldr-sources.json；其是否为 Qt 历史输入未经确认。获取前须核对上游条款：
   https://iso639-3.sil.org/code_tables/download_tables
   English: The CLDR 46 subset contains common data and its license; Qt's locale
   generator also needs a separate ISO 639-3 table, not included in this delivery.
   qt-cldr-sources.json records a tested candidate, not a verified historical Qt input;
   check its upstream terms before obtaining it at the URL above.
3. 从源码工作目录离线核验当前 codec 构建材料：
   python3 scripts/prepare-native-codecs.py --offline --archives <交付目录>/native-codecs
   当前独立 Rust 后端的格式源码清单为空，此步骤不下载或编译官方格式引擎。
   English: The independent Rust codec source list is empty; this step does not
   download or compile official format engines.
   rar-wire-mappings/ 另保留有限 RAR 映射的参考来源和原始完整许可，不含官方引擎。
   English: rar-wire-mappings/ separately preserves finite-mapping provenance and
   the exact original notice, without adding an official engine or legal clearance.
4. 将 QT_PREFIX 指向重建的兼容 Qt SDK，运行：
   cmake -S . -B build/relink -G Ninja -DCMAKE_BUILD_TYPE=Release \\
     -DCMAKE_PREFIX_PATH="$QT_PREFIX" -DXDZIP_BUILD_DESKTOP_PREVIEW=ON
   cmake --build build/relink --parallel 3
   ctest --test-dir build/relink --output-on-failure
   macOS 另加 -DCMAKE_OSX_ARCHITECTURES=arm64 -DCMAKE_OSX_DEPLOYMENT_TARGET=14.0。
   Mac 重建前从交付目录的 Sparkle-2.10.0.tar.xz 离线准备 SDK：
   python3 scripts/prepare-sparkle.py --archive <交付目录>/Sparkle-2.10.0.tar.xz \\
     --output build/sparkle-sdk
   CMake 另传 -DXDZIP_SPARKLE_SDK="$PWD/build/sparkle-sdk"。Sparkle 原始源码归档
   与固定 SDK 均随本目录交付，不将 SDK 分片核对冒充上游源码重建证明。
5. 应用动态链接 Qt，用户可重建后重新部署兼容的修改版 Qt。macOS 对替换后的所有
   Mach-O、framework 和应用重新做 ad-hoc 签名，否则 ARM64 可能拒绝加载。
   不要求用户拥有本项目的 Developer ID 私钥。勿关闭 SIP 或系统安全保护。
6. Linux ICU 的完整许可、ICU4C 73.2 上游源码归档、固定 ICU 可编辑数据源码树及
   CLDR 43.1 源码子集随目录提供。CLDR 子集排除了五个 ISO 639-3 文件；须按
   其上游来源和条款另行取得，不能将子集称为完整源码树。版本与哈希见清单；
   版本一致不证明供应商库由这些材料无补丁构建。
   当前 Windows 开发包不交付软件 OpenGL 库。历史包按原有材料和条款处理。
   English: The Linux ICU license, ICU4C 73.2 upstream archive, pinned editable ICU
   data source tree and a CLDR 43.1 subset are included. Five ISO 639-3 files are
   omitted and their upstream source and terms require separate review. Matching versions
   do not prove the supplier built its libraries without patches from these inputs. Current Windows developer packages
   omit the software OpenGL library; historical deliveries retain their obligations.

## 显式测试数据前置与完整 runtime 批次

以下是独立测试阶段，先完成上述构建。CAB、ISO/UDF 与基础 RAR 的固定夹具分别在
tests/fixtures/cab-iso/ 和 tests/fixtures/native-archives/，随项目源码交付；7z 测试通过
刚构建的 Rust worker 创建自己的输入。此 runtime 批次不要求官方 7-Zip/UnRAR 工具或挂载映像。
CAB 专项另需子进程环境 XDZIP_CAB_BACKEND=rust，与固定 Rust worker 配套。

RAR PPM、旧密码等公共测试输入须先显式准备。当前交付不包含 preparer 使用的七个公共
缓存文件，Cargo --offline 只约束依赖获取，不能补齐这些输入或环境变量。保留其原始来源、
大小、SHA-256 与通知，按 scripts/rar-rust-fixtures.json 和 preparer 中的固定清单核对。
已有缓存时，显式执行（替换缓存路径，保持清单规定的文件及 junrar 子目录布局）：

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs --archives /path/to/fixed-rar-cache --offline
```

首次准备需要联网时，可单独显式执行同一命令，去掉 --archives 和 --offline；它只获取固定
小型公共测试材料，不是离线重建的一部分。prepare/build/test 不会隐式补做此步骤。
准备器生成 sources.json，但不导出当前 shell 环境。随后在源码根目录执行下面的子进程交接，
传入刚构建的 worker。Windows 将 python3 改为 python，并为 worker 加 .exe。

```sh
python3 -c "
import json, os, pathlib, subprocess, sys
root = pathlib.Path.cwd()
worker = pathlib.Path(sys.argv[1]).resolve(strict=True)
prepared = json.loads((root/'build/native-test-inputs/sources.json').read_text(encoding='utf-8'))
environment = os.environ.copy()
environment.update({name: str(root/value) for name, value in prepared['environment'].items()})
environment.update(XDZIP_NATIVE_CODEC=str(worker), XDZIP_NATIVE_CODEC_WORKER=str(worker), XDZIP_CAB_BACKEND='rust')
environment['XDZIP_RAR_FIXTURE_PASSWORD'] = 'password'
for command in [
    ['cargo', 'test', '--workspace', '--locked', '--offline'],
    ['cargo', 'test', '-p', 'xdzip-runtime', '--locked', '--offline', 'native_backend_tests', '--', '--ignored'],
]:
    subprocess.run(command, env=environment, check=True)
" build/relink/tools/native-codec/xdzip-native-codec
```

password 是该公共 fixture 既有的测试值，只传给测试子进程。上例不减少 ignored 批次，也不
下载输入或构建 worker。缺少缓存时保留失败，不能把主程序离线重建与启动冒烟记为上述批次通过。
English: Product/Rust source and vendored dependencies support the separately recorded offline app
rebuild. The full runtime test batch additionally needs explicitly prepared fixed public data and its
child environment. Those public cache files are not included in this delivery. Use --archives and
--offline with a verified cache, or explicitly prepare it online first. No test step downloads implicitly,
no ignored selection is reduced, and app rebuild/smoke evidence does not establish this batch passed.

该流程不宣称任意 ABI 不兼容的 Qt 都可替换。许可分支选择、完整
平台对应源码与配置核对、修改版 Qt 的行为验证仍在 WP-12 准入中单独登记。
'''


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
    all_archives = config['archives'] + config.get('build_tools', []) + [config['rust_stdlib']]
    with ThreadPoolExecutor(max_workers=4) as pool:
        sources = list(pool.map(lambda item: archive_source(item, args.cache), all_archives))
    output.mkdir(parents=True)
    materials = []
    for source, item in zip(sources, all_archives):
        shutil.copy2(source, output/source.name)
        materials.append(dict(item))
    qtbase_sources = [source for source, item in zip(sources, all_archives) if item.get('module') == 'qtbase']
    if len(qtbase_sources) != 1:
        raise ValueError('Expected exactly one Qt base source archive')
    materials.append(collect_qt_psl_source(output, qtbase_sources[0]))
    materials.append(collect_qt_tika_source(output, qtbase_sources[0]))
    materials.append(collect_qt_cldr_source(output, qtbase_sources[0], ROOT/'build/native-source-cache'))
    materials += collect_sparkle_sources(output, args.cache)
    from native_codec_materials import corresponding_sources
    materials += corresponding_sources(output)
    _, rar_mapping_record = collect_rar_mapping_materials(output)
    materials.append(rar_mapping_record)
    materials += collect_native_sources(output, ROOT/'build/native-source-cache')
    materials += collect_icu_data_sources(output, ROOT/'build/native-source-cache')
    collect_source_identity_proofs(output, all_archives)
    shutil.copy2(ROOT/'scripts/corresponding-sources.json', output/'corresponding-sources.json')
    patch_dir = output/'qt-patches'
    patch_dir.mkdir()
    for item in config.get('sdk_patches', []) + config.get('source_build_patches', []):
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
    write_vendor_config(output)
    vendor_notices = collect_vendor_notice_materials(vendor, output/'rust-vendor-notices')
    materials.append({'kind': 'rust-vendor-notice-supplements',
                      'file': 'rust-vendor-notices/index.json',
                      'packages': len(vendor_notices['packages'])})
    vendor_archive = output/'rust-vendor.tar.gz'
    with tarfile.open(vendor_archive, 'w:gz') as archive:
        archive.add(vendor, arcname='vendor')
    shutil.rmtree(vendor)
    instructions = rebuild_instructions()
    (output/'REBUILD.md').write_text(instructions, encoding='utf-8')
    files = [{'file': p.relative_to(output).as_posix(), 'sha256': digest(p), 'bytes': p.stat().st_size}
             for p in sorted(output.rglob('*')) if p.is_file()]
    report = {'schema_version': 1, 'revision': commit, 'qt_version': config['qt_version'],
              'source_delivery': 'collected-not-compliance-verified', 'materials': materials, 'files': files,
              'limits': ['Pinned Qt module archives and locked Cargo vendor inputs, not a proof of SDK build equivalence.',
                         'Exact linked components, platform runtimes and license selection require verification.']}
    (output/'source-delivery.json').write_text(json.dumps(report, indent=2)+'\n')
    print('Collected project source, locked Rust vendor inputs and verified Qt/runtime-tool/Rust-stdlib source archives')


if __name__ == '__main__':
    main()
