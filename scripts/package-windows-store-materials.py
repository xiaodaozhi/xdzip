#!/usr/bin/env python3
"""Export checked bilingual Store text/images, without binaries or source archives."""
import argparse
import csv
from datetime import datetime
import hashlib
import io
import json
from pathlib import Path
import re
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
MARKETING = Path('specs/marketing/windows-store.json')
CAPTURES = Path('specs/marketing/windows-store-captures.json')
ASSETS = Path('specs/marketing/windows-store-assets.json')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_json(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def store_base(locale):
    return Path('developers/market/windows' if locale == 'en-US' else 'developers/zh-CN/market/windows')


def stage_capture_source(archive, stage):
    spec = read_json(ROOT / MARKETING)
    saved = read_json(archive / MARKETING)
    baseline = saved['baseline']
    captures = read_json(archive / CAPTURES)
    if (captures['schema_version'] != 3 or captures['window_mode'] != 'default'
            or not re.fullmatch(r'1\.0\.[1-9][0-9]*\.0', baseline['package_version'])
            or not re.fullmatch(r'[a-f0-9]{40}', baseline['revision'])
            or captures['baseline_revision'] != baseline['revision']
            or any(not re.fullmatch(r'[a-f0-9]{64}', baseline[key]) or captures[key] != baseline[key]
                   for key in ('package_sha256', 'runtime_exe_sha256'))):
        raise ValueError('Capture source differs from the archived package identity')
    expected = {(locale, scene): caption for locale, copy in spec['locales'].items()
                for scene, caption in copy['screenshot_captions'].items()}
    actual = [(image['locale'], image['scene']) for image in captures['images']]
    if len(actual) != len(expected) or set(actual) != set(expected):
        raise ValueError('Capture source must contain all scenes for both languages')
    spec['baseline'] = baseline
    canvas = saved['screenshot_artwork']['store_canvas']
    if (not isinstance(canvas, list) or len(canvas) != 2
            or any(type(value) is not int for value in canvas)
            or not (1366 <= canvas[0] <= 3840 and 768 <= canvas[1] <= 2160)):
        raise ValueError('Invalid archived Store preview canvas')
    spec['screenshot_artwork']['store_canvas'] = canvas
    files = [MARKETING, CAPTURES]
    for image in captures['images']:
        locale, scene = image['locale'], image['scene']
        native = Path('design/assets/marketing/windows-store-native') / locale / (scene + '.png')
        cutout = store_base(locale) / 'images/screenshots' / (scene + '.png')
        if (image['native_file'] != native.as_posix() or image['file'] != cutout.as_posix()
                or image['cutout_file'] != cutout.as_posix()):
            raise ValueError('Unexpected file path in capture source')
        for file, expected_hash in ((native, image['source_sha256']), (cutout, image['cutout_sha256'])):
            source = (archive / file).resolve()
            if not source.is_relative_to(archive):
                raise ValueError('Capture source file escapes the archive')
            data = source.read_bytes()
            if digest(data) != expected_hash:
                raise ValueError(f'Archived screenshot differs from its recorded digest: {file.as_posix()}')
            destination = stage / file
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
            files.append(file)
        image['caption'] = expected[locale, scene]
    write_json(stage / MARKETING, spec)
    write_json(stage / CAPTURES, captures)
    brand = Path('design/assets/brand/xdzip.svg')
    (stage / brand).parent.mkdir(parents=True, exist_ok=True)
    (stage / brand).write_bytes((ROOT / brand).read_bytes())
    return spec, files


def export_materials(stage, output, spec, capture_files, msix_version):
    store_dimensions = ' × '.join(map(str, spec['screenshot_artwork']['store_canvas']))
    inventory = read_json(stage / ASSETS)
    payload = {f'capture-source/{file.as_posix()}': (stage / file).read_bytes() for file in capture_files}
    image_count = 0
    for locale, copy in spec['locales'].items():
        base = ROOT / store_base(locale)
        field_files = {
            'description.txt': copy['description'],
            'whats-new.txt': copy['release_notes'],
            'features.txt': '\n'.join(copy['features']),
            'short-description.txt': copy['short_description'],
            'keywords.txt': '\n'.join(copy['keywords']),
            'copyright.txt': copy['copyright'],
        }
        for name, value in field_files.items():
            payload[f'{locale}/text/{name}'] = (value + '\n').encode('utf-8')
        fields = {key: copy[key] for key in ('product_name', 'short_title', 'sort_title', 'voice_title',
                  'developer', 'description', 'short_description', 'release_notes', 'features',
                  'keywords', 'copyright', 'additional_license_terms', 'system_requirements', 'privacy_url')}
        fields.update(website=spec['links']['website'], support=spec['links']['support'])
        payload[f'{locale}/text/fields.json'] = (json.dumps(fields, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
        notes = re.findall(r'```text\n(.*?)\n```', (base / 'reviewer-notes.md').read_text(encoding='utf-8'), re.S)
        if len(notes) != 2:
            raise ValueError('Expected capability and certification notes')
        for name, value in zip(('runFullTrust.txt', 'certification-notes.txt'), notes):
            payload[f'{locale}/review/{name}'] = (value + '\n').encode('utf-8')
        stream = io.StringIO(newline='')
        writer = csv.writer(stream)
        writer.writerow(('filename', 'caption'))
        for scene, caption in copy['screenshot_captions'].items():
            writer.writerow((f'{scene}-1920x1080.png', caption))
        payload[f'{locale}/text/screenshot-captions.csv'] = stream.getvalue().encode('utf-8-sig')
        scenes = []
        for asset in inventory['assets']:
            if asset['locale'] not in (locale, 'shared'):
                continue
            role = asset['role']
            folder = {'store-screenshot': 'screenshots', 'screenshot-artwork': 'promotional',
                      'feature-illustration': 'feature-illustrations', 'app-tile-icon': 'branding',
                      'reusable-avatar': 'branding', 'text-free-hero': 'branding',
                      'poster': 'branding', 'box-art': 'branding'}.get(role)
            if folder is None:
                continue
            data = (stage / asset['file']).read_bytes()
            if digest(data) != asset['sha256'] or data[:8] != b'\x89PNG\r\n\x1a\n':
                raise ValueError('Asset differs from the checked inventory: ' + asset['file'])
            dimensions = list(struct.unpack('>II', data[16:24]))
            if dimensions != [asset['width'], asset['height']] or len(data) > 5_000_000:
                raise ValueError('Invalid asset dimensions or byte limit')
            name = Path(asset['file']).name
            target = f'{locale}/{folder}/{name}'
            if target in payload:
                raise ValueError('Duplicate exported image')
            payload[target] = data
            image_count += 1
            if role == 'store-screenshot':
                if dimensions[0] < 1366 or dimensions[1] < 768:
                    raise ValueError('Desktop screenshot is too small')
            if role == 'screenshot-artwork' and dimensions == [1920, 1080]:
                scenes.append(name.removesuffix('-1920x1080.png'))
        if sorted(scenes) != sorted(copy['screenshot_captions']):
            raise ValueError('Desktop screenshot set is incomplete')
    guide = '''# xdzip Microsoft Store materials / 商店材料

Listing export / 一览导出版本: MSIX {version}, x64

Screenshot source / 实机截图采集版本: MSIX {capture_version}

Use en-US for English (United States), zh-CN for Chinese (China).
英语（美国）使用 en-US，中文（中国）使用 zh-CN。

| Partner Center field / 页面字段 | File / 文件 |
| --- | --- |
| Description / 说明 | <language>/text/description.txt |
| What's new / 此版本的新增功能 | <language>/text/whats-new.txt |
| Product features / 产品功能 | <language>/text/features.txt, one entry per line / 每行单独填写 |
| Short description / 简短描述 | <language>/text/short-description.txt |
| Keywords / 关键字 | <language>/text/keywords.txt, one entry per line / 每行单独填写 |
| Copyright / 版权 | <language>/text/copyright.txt |
| Other fields and links / 其他字段与链接 | <language>/text/fields.json |
| Desktop screenshots / 桌面截图 | <language>/promotional/01-browse-1920x1080.png through 04-workspace-1920x1080.png |
| Screenshot captions / 截图说明 | <language>/text/screenshot-captions.csv, reference only / 仅供对应填写 |
| App icons / 应用图标 | <language>/branding/app-icon-300.png, app-icon-150.png, app-icon-71.png |
| Avatars / 头像 | <language>/branding/avatar-512.png, avatar-1024.png, reusable brand exports / 补充品牌导出 |
| Poster / 招贴画 | <language>/branding/poster-720x1080.png or poster-1440x2160.png |
| Square art / 方形品牌图 | <language>/branding/box-art-1080.png or box-art-2160.png |
| Text-free hero / 无文字横幅 | <language>/branding/hero-1920x1080.png |
| Restricted capability / 受限功能 | <language>/review/runFullTrust.txt |
| Certification notes / 认证备注 | <language>/review/certification-notes.txt |

The screenshots folder contains {store_dimensions} plain app views. The promotional
folder contains 1920 x 1080 artwork with borders, shadows and localized copy.
Use --4k when exporting to add 3840 x 2160 versions. Review both language versions as individual images.
Upload the four 1920 x 1080 files in promotional/ to the desktop screenshot field.

screenshots/ 是 {store_dimensions} 的纯应用界面。promotional/ 提供带圆角细边框、阴影和对应语言文案的
1920 x 1080 宣传图。导出时加 --4k 可另得 3840 x 2160 版本。请逐张审阅两种语言的图片。
桌面截图栏上传 promotional/ 中的四张 1920 x 1080 融合宣传图。

Keep this directory as individual files in your backup after submitting updated assets.
Use capture-source/ to render later layout or copy changes from the same captured pixels.

更新素材提交后，请将此目录中的独立文件归档到备份位置。
今后修改版式或文案时，可从 capture-source/ 恢复相同的截图像素。

Use the values in fields.json for the optional fields.
fields.json and the caption CSV are copy references, not a Partner Center
CSV import template. Privacy URLs are provided in fields.json.

可选字段按 fields.json 填写。
fields.json 和截图说明 CSV 是填写参考，不是合作伙伴中心 CSV 导入模板。
隐私政策链接见 fields.json。

This directory does not include an application package, binaries, project source,
account credentials, screenshots of personal files, or a submitted certification request.
Pricing, age ratings and other account-level submission sections are completed separately.

本目录不包含程序包、二进制、项目源码、账户凭据或个人文件截图，也不表示已经提交认证。
定价、年龄分级等提交栏目需在账户中另行完成。
'''.format(version=msix_version, capture_version=spec['baseline']['package_version'],
           store_dimensions=store_dimensions)
    payload['UPLOAD.md'] = guide.encode('utf-8')
    files = [{'file': name, 'bytes': len(data), 'sha256': digest(data)} for name, data in sorted(payload.items())]
    manifest = {'schema_version': 1, 'baseline': spec['baseline'], 'msix_version': msix_version,
                'render_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                'image_count': image_count,
                'asset_inventory_sha256': digest((stage / ASSETS).read_bytes()), 'files': files}
    payload['manifest.json'] = (json.dumps(manifest, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
    payload['SHA256SUMS.txt'] = ''.join(f"{digest(data)}  {name}\n" for name, data in sorted(payload.items())).encode('utf-8')
    for name, data in payload.items():
        file = output / name
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_bytes(data)
    print(json.dumps({'files': len(payload), 'images': image_count, 'languages': list(spec['locales']),
                      'screenshot_package': spec['baseline']['package_version']}, ensure_ascii=False))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--from-archive', type=Path, required=True, help='capture-source directory from a verified export or backup')
    parser.add_argument('--output', type=Path, help='new directory for individual bilingual files')
    parser.add_argument('--4k', dest='include_4k', action='store_true', help='also render 3840 × 2160 artwork')
    args = parser.parse_args()
    version = read_json(ROOT / 'VERSION.json')
    msix_version = f"1.0.{version['build']}.0"
    output = (args.output or (ROOT / 'build/store-listing' / msix_version /
                             datetime.now().strftime('%Y%m%d-%H%M%S'))).resolve()
    archive = args.from_archive.resolve()
    allowed = (ROOT / 'build', ROOT / 'dist')
    if (output.exists() or output.is_relative_to(archive) or
            (output.is_relative_to(ROOT) and not any(output != path and output.is_relative_to(path) for path in allowed))):
        parser.error('--output must be a fresh directory below build/ or dist/, or outside the repository')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=output.parent, prefix='.store-export-') as temporary:
        stage = Path(temporary) / 'render'
        spec, files = stage_capture_source(archive, stage)
        render = ['node', str(ROOT / 'scripts/build-windows-store-assets.cjs'), '--root', str(stage)]
        if args.include_4k:
            render.append('--4k')
        subprocess.run(render, cwd=ROOT, check=True)
        pending = Path(temporary) / 'export'
        export_materials(stage, pending, spec, files, msix_version)
        pending.rename(output)
    print(f'Store materials exported to {output}')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from error
