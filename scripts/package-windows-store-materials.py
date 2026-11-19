#!/usr/bin/env python3
"""Export checked bilingual Store text/images, without binaries or source archives."""
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
import re
import struct
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    archive = Path(str(output) + '.zip')
    if output.exists() or archive.exists():
        raise ValueError('Choose a fresh output directory and ZIP name')
    spec = json.loads((ROOT / 'specs/marketing/windows-store.json').read_text(encoding='utf-8'))
    version = json.loads((ROOT / 'VERSION.json').read_text(encoding='utf-8'))
    msix_version = f"1.0.{version['build']}.0"
    store_dimensions = ' × '.join(map(str, spec['screenshot_artwork']['store_canvas']))
    inventory = json.loads((ROOT / 'specs/marketing/windows-store-assets.json').read_text(encoding='utf-8'))
    payload = {}
    image_count = 0
    for locale, copy in spec['locales'].items():
        base = ROOT / ('developers/market/windows' if locale == 'en-US' else 'developers/zh-CN/market/windows')
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
            writer.writerow((f'{scene}.png', caption))
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
            data = (ROOT / asset['file']).read_bytes()
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
                scenes.append(Path(name).stem)
        if sorted(scenes) != sorted(copy['screenshot_captions']):
            raise ValueError('Desktop screenshot set is incomplete')
    guide = '''# xdzip Microsoft Store materials / 商店材料

Package / 对应程序包: MSIX {version}, x64

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
| Text-free hero / 无文字横幅 | <language>/branding/hero-1920x1080.png or hero-3840x2160.png |
| Restricted capability / 受限功能 | <language>/review/runFullTrust.txt |
| Certification notes / 认证备注 | <language>/review/certification-notes.txt |

The screenshots folder contains {store_dimensions} plain app views. Promotional images with
rounded borders, shadows and localized copy are in promotional/ at 1920 x 1080 and
3840 x 2160. feature-illustrations/ contains explicitly labeled vector illustrations.
Upload the four 1920 x 1080 files in promotional/ to the desktop screenshot field.

screenshots/ 是 {store_dimensions} 的纯应用界面。promotional/ 是带圆角细边框、阴影和双语文案的
1920 x 1080、3840 x 2160 宣传图。feature-illustrations/ 是标明用途的矢量功能示意图。
桌面截图栏上传 promotional/ 中的四张 1920 x 1080 融合宣传图。

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
'''.format(version=msix_version, store_dimensions=store_dimensions)
    payload['UPLOAD.md'] = guide.encode('utf-8')
    files = [{'file': name, 'bytes': len(data), 'sha256': digest(data)} for name, data in sorted(payload.items())]
    manifest = {'schema_version': 1, 'baseline': spec['baseline'], 'msix_version': msix_version,
                'image_count': image_count,
                'asset_inventory_sha256': digest((ROOT / 'specs/marketing/windows-store-assets.json').read_bytes()), 'files': files}
    payload['manifest.json'] = (json.dumps(manifest, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
    payload['SHA256SUMS.txt'] = ''.join(f"{digest(data)}  {name}\n" for name, data in sorted(payload.items())).encode('utf-8')
    output.mkdir(parents=True)
    for name, data in payload.items():
        file = output / name
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_bytes(data)
    with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as package:
        for name in sorted(payload):
            package.write(output / name, name)
    with zipfile.ZipFile(archive) as package:
        if package.testzip() is not None or set(package.namelist()) != set(payload):
            raise ValueError('Export ZIP integrity differs')
        for name, expected in payload.items():
            if package.read(name) != expected:
                raise ValueError('Export ZIP content differs')
    print(json.dumps({'files': len(payload), 'images': image_count, 'languages': list(spec['locales']),
                      'zip_bytes': archive.stat().st_size, 'zip_sha256': digest(archive.read_bytes())}, ensure_ascii=False))


if __name__ == '__main__':
    main()
