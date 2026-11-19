"""Pinned UI font preparation and deterministic, offline subset generation."""
from contextlib import contextmanager
import hashlib
import io
import json
import os
from pathlib import Path
import re
import sys
import time
import urllib.request
from urllib.parse import quote
from ui_languages import load_languages
import uuid

ROOT = Path(__file__).resolve().parents[1]
SPEC = Path('specs/dependencies/ui-font.json')
FONT_NAME = 'xdzip-design-sans-sc'
DEFAULT_OUTPUT = ROOT / 'build/generated/fonts'
DEFAULT_CACHE = ROOT / 'build/toolchain/fonts'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def configuration(root=ROOT):
    return json.loads((root / SPEC).read_text(encoding='utf-8'))


def generated_directory(path, root=ROOT):
    path = Path(path).resolve()
    if not path.is_relative_to((root / 'build').resolve()) or path == (root / 'build').resolve():
        raise ValueError('Font outputs and cache must use a dedicated directory under build/')
    return path


def atomic_write(path, data):
    """Publish complete bytes on the same filesystem; failure leaves the old file."""
    path = Path(path)
    if path.is_file() and path.read_bytes() == data:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + '.' + uuid.uuid4().hex + '.tmp')
    try:
        temporary.write_bytes(data)
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


@contextmanager
def locked(path):
    """OS-owned locks release on process exit, including failed generation."""
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('a+b') as stream:
        if stream.tell() == 0:
            stream.write(b'\0')
            stream.flush()
        deadline = time.monotonic() + 120
        while True:
            try:
                stream.seek(0)
                if os.name == 'nt':
                    import msvcrt
                    msvcrt.locking(stream.fileno(), msvcrt.LK_NBLCK, 1)
                else:
                    import fcntl
                    fcntl.flock(stream.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except OSError:
                if time.monotonic() >= deadline:
                    raise ValueError('Timed out waiting for another font generation process') from None
                time.sleep(0.1)
        try:
            yield
        finally:
            stream.seek(0)
            if os.name == 'nt':
                msvcrt.locking(stream.fileno(), msvcrt.LK_UNLCK, 1)
            else:
                fcntl.flock(stream.fileno(), fcntl.LOCK_UN)


def source_path(cache=DEFAULT_CACHE, root=ROOT):
    return generated_directory(cache, root) / configuration(root)['upstream']['sha256'] / 'source.ttf'


def prepare(cache=DEFAULT_CACHE, *, source=None, offline=False, root=ROOT):
    """Only this stage may fetch; both local imports and downloads require the pin."""
    cache = generated_directory(cache, root)
    spec = configuration(root)
    target = source_path(cache, root)
    with locked(cache.parent / '.ui-font-prepare.lock'):
        if target.is_file():
            if digest(target) != spec['upstream']['sha256']:
                raise ValueError('Cached UI font SHA-256 mismatch; remove the corrupt source.ttf and prepare again')
            return target
        if source is not None:
            data = Path(source).read_bytes()
        elif offline:
            raise ValueError('Pinned UI font is missing; run scripts/ui-fonts.py prepare before offline build')
        else:
            with urllib.request.urlopen(spec['upstream']['url'], timeout=30) as response:
                data = response.read(32 * 1024 * 1024 + 1)
            if len(data) > 32 * 1024 * 1024:
                raise ValueError('Upstream UI font exceeds the download budget')
        if hashlib.sha256(data).hexdigest() != spec['upstream']['sha256']:
            raise ValueError('Source UI font SHA-256 does not match the pinned upstream font')
        atomic_write(target, data)
    return target


def localized_sources(cache, spec, *, offline):
    upstream = spec['upstream']
    sources = {}
    with locked(cache.parent / '.ui-font-prepare.lock'):
        for group, data in spec.get('localized_fonts', {}).items():
            target = cache / data['sha256'] / 'source.ttf'
            if not target.is_file():
                if offline:
                    raise ValueError(f'Pinned UI font {group} is missing; run scripts/ui-fonts.py prepare')
                url = 'https://raw.githubusercontent.com/google/fonts/' + upstream['commit'] + '/'
                with urllib.request.urlopen(url + quote(data['path']), timeout=40) as response:
                    contents = response.read(32 * 1024 * 1024 + 1)
                if hashlib.sha256(contents).hexdigest() != data['sha256']:
                    raise ValueError(f'Upstream UI font {group} differs from the pinned SHA-256')
                atomic_write(target, contents)
            if digest(target) != data['sha256']:
                raise ValueError(f'Cached UI font {group} differs from the pinned SHA-256')
            sources[group] = target
    return sources


def inputs(root=ROOT):
    spec = configuration(root)
    paths = set()
    for pattern in spec['inputs']:
        paths.update(path for path in root.glob(pattern) if path.is_file())
    return sorted(path for path in paths if path.name not in spec['exclude_names']
                  and not any(path.relative_to(root).as_posix().startswith(prefix)
                              for prefix in spec.get('exclude_prefixes', [])))


def decoded_characters(text):
    # Decode Unicode escapes without interpreting UTF-8 as a legacy code page.
    text = re.sub(r'\\u\{([0-9a-fA-F]{1,6})\}|\\U([0-9a-fA-F]{8})|\\u([0-9a-fA-F]{4})',
                  lambda match: chr(int(next(value for value in match.groups() if value), 16)), text)
    # JSON/JavaScript may encode a supplementary character as a surrogate pair.
    text = text.encode('utf-16-le', errors='surrogatepass').decode('utf-16-le', errors='replace')
    return {ord(c) for c in text if ord(c) >= 0x20 and c not in '\ufffd\ufeff'
            and not 0xD800 <= ord(c) <= 0xDFFF}


def current_characters(root=ROOT):
    spec = configuration(root)
    characters = set(range(0x20, 0x7F)) | set(map(ord, spec['extra_characters']))
    localized = {locale for locs in localized_catalogs(root).values() for locale, _, _ in locs}
    for path in inputs(root):
        text = path.read_text(encoding='utf-8')
        if path.suffix == '.json':
            if path.parent == root / 'resources/i18n' and path.stem in localized:
                continue
            catalog = json.loads(text)
            values = catalog.get('messages', {}).values()
            for value in values:
                for label in (value.values() if isinstance(value, dict) else [value]):
                    characters.update(decoded_characters(label))
        elif path.suffix in {'.qml', '.cpp', '.h', '.mm'}:
            # Include literals outside translation calls too (for example native
            # menu labels and QML accessibility names). Test data is excluded.
            for label in re.findall(r'"((?:[^"\\]|\\.)*)"', text):
                characters.update(decoded_characters(label))
        else:
            characters.update(decoded_characters(text))
    return characters


def localized_catalogs(root):
    registry = root / 'resources/ui-languages.json'
    if not registry.is_file():
        return {}
    grouped = {}
    for language in load_languages(registry)['languages']:
        group, locale = language['font'], language['id']
        path = root / 'resources/i18n' / f'{locale}.json'
        if group != 'sc' and path.is_file():
            grouped.setdefault(group, []).append((locale, language['name'], path))
    return grouped


def localized_characters(root):
    groups = {}
    for group, catalogs in localized_catalogs(root).items():
        characters = set(range(0x20, 0x7F))
        for _, name, path in catalogs:
            characters.update(decoded_characters(name))
            messages = json.loads(path.read_text(encoding='utf-8'))['messages']
            for key, value in messages.items():
                if key in ('language.zhCN', 'language.enUS'):
                    continue  # These language choices use their own font family.
                for text in value.values() if isinstance(value, dict) else (value,):
                    characters.update(decoded_characters(text))
        groups[group] = sorted(characters)
    return groups


def tools(root=ROOT):
    try:
        import brotli
        import fontTools
    except ImportError:
        raise ValueError('UI font tools missing; install design/assets/fonts/requirements.txt in the selected Python environment') from None
    actual = {'fonttools': fontTools.__version__, 'Brotli': brotli.__version__}
    if actual != configuration(root)['tools']:
        raise ValueError('UI font tool versions differ; install the pinned design/assets/fonts/requirements.txt')
    return actual


def input_identity(root=ROOT):
    spec = configuration(root)
    codepoints = sorted(current_characters(root))
    identity = {'spec': spec, 'generator_sha256': digest(Path(__file__)),
                'tools': tools(root), 'codepoints': codepoints,
                'localized_codepoints': localized_characters(root)}
    return hashlib.sha256(json.dumps(identity, sort_keys=True).encode()).hexdigest(), codepoints


def load_manifest(directory, *, root=ROOT, expected_key=None):
    """Bind consumers to a complete generation; never regenerate while packaging."""
    directory = generated_directory(directory, root)
    manifest = json.loads((directory / 'manifest.json').read_text(encoding='utf-8'))
    spec = configuration(root)
    if (manifest.get('schema_version') != 1 or manifest.get('upstream') != spec['upstream']
            or manifest.get('tools') != spec['tools']
            or manifest.get('recipe_sha256') != digest(root / SPEC)
            or manifest.get('generator_sha256') != digest(Path(__file__))
            or manifest.get('requested_codepoints') != sorted(current_characters(root))
            or (expected_key is not None and manifest.get('input_key') != expected_key)):
        raise ValueError('Generated UI font is stale; rebuild the matching product first')
    if digest(root / spec['license']['path']) != spec['license']['sha256']:
        raise ValueError('UI font license SHA-256 mismatch')
    for kind, suffix in [('subset', '.woff2'), ('native', '.ttf')]:
        item = manifest[kind]
        if item['path'] != FONT_NAME + suffix or digest(directory / item['path']) != item['sha256']:
            raise ValueError('Generated UI font bytes differ from their manifest; rebuild first')
    groups = localized_characters(root)
    generated = manifest.get('localized', {})
    if groups.keys() != generated.keys():
        raise ValueError('Generated localized font set is stale; rebuild the matching product first')
    for group, codepoints in groups.items():
        item = generated[group]
        recipe = spec['localized_fonts'][group]
        license_info = recipe.get('license', spec['license'])
        path = f'xdzip-design-sans-{group}.ttf'
        if (item['path'] != path or item['source_sha256'] != recipe['sha256']
                or item['license'] != license_info or digest(root / license_info['path']) != license_info['sha256']
                or item['requested_codepoints'] != codepoints or digest(directory / path) != item['sha256']):
            raise ValueError(f'Generated UI font {group} differs from its build manifest')
    return manifest


def localized_font(source, codepoints, recipe, axis):
    from fontTools import subset
    from fontTools.ttLib import TTFont
    from fontTools.varLib.instancer import instantiateVariableFont

    font = TTFont(source, recalcTimestamp=False)
    missing = sorted(set(codepoints) - set(font.getBestCmap()))
    if missing:
        raise ValueError(f"{recipe['family']} lacks required glyphs: " + ', '.join(f'U+{value:04X}' for value in missing))
    options = subset.Options()
    options.recommended_glyphs = True
    options.name_IDs = [0, 1, 2, 3, 4, 5, 6, 13, 14, 16, 17, 25]
    options.name_languages = [0x409]
    subsetter = subset.Subsetter(options=options)
    subsetter.populate(unicodes=codepoints)
    subsetter.subset(font)
    limits = {'wght': (axis['min'], axis['default'], axis['max'])}
    if 'wdth' in {item.axisTag for item in font['fvar'].axes}:
        limits['wdth'] = 100
    font = instantiateVariableFont(font, limits, inplace=True)
    family = recipe['family']
    identifier = family.replace(' ', '')
    names = {1: family, 2: 'Regular', 3: identifier + '-v1', 4: family,
             6: identifier, 16: family, 17: 'Regular', 25: identifier}
    for name, value in names.items():
        font['name'].setName(value, name, 3, 1, 0x409)
    output = io.BytesIO()
    font.save(output)
    return output.getvalue()


def generate(output=DEFAULT_OUTPUT, *, cache=DEFAULT_CACHE, check=False, root=ROOT):
    tools(root)
    from fontTools import subset
    from fontTools.ttLib import TTFont
    from fontTools.varLib.instancer import instantiateVariableFont
    output = generated_directory(output, root)
    key, codepoints = input_identity(root)
    with locked(output / '.build.lock'):
        source = prepare(cache, offline=True, root=root)
        if (output / 'manifest.json').is_file():
            try:
                return load_manifest(output, root=root, expected_key=key)
            except (OSError, ValueError, KeyError):
                if check:
                    raise
        elif check:
            raise ValueError('Generated UI font missing; run scripts/ui-fonts.py build')
        spec = configuration(root)
        font = TTFont(source, recalcTimestamp=False)
        missing = sorted(set(codepoints) - set(font.getBestCmap()))
        if missing:
            raise ValueError('Upstream UI font lacks required glyphs: ' + ', '.join(f'U+{v:04X}' for v in missing))
        options = subset.Options()
        options.notdef_outline = True
        options.recommended_glyphs = True
        options.name_IDs = [0, 1, 2, 3, 4, 5, 6, 13, 14, 16, 17, 25]
        options.name_legacy = True
        options.name_languages = [0x409]
        subsetter = subset.Subsetter(options=options)
        subsetter.populate(unicodes=codepoints)
        subsetter.subset(font)
        axis = spec['weight_axis']
        font = instantiateVariableFont(font, {'wght': (axis['min'], axis['default'], axis['max'])}, inplace=True)
        family = spec['family']
        names = {1: family, 2: 'Regular', 3: 'XdzipDesignSansSC-HTML-v1', 4: family,
                 6: 'XdzipDesignSansSC', 16: family, 17: 'Regular', 25: 'XdzipDesignSansSC'}
        for identifier, value in names.items():
            font['name'].setName(value, identifier, 3, 1, 0x409)
        font['OS/2'].usWeightClass = 400
        font.flavor = 'woff2'
        compressed = io.BytesIO()
        font.save(compressed)
        # Preserve the existing WOFF2 -> TTF export's outlines and metrics.
        native = TTFont(io.BytesIO(compressed.getvalue()), recalcTimestamp=False)
        native.flavor = None
        uncompressed = io.BytesIO()
        native.save(uncompressed)
        manifest = {'schema_version': 1, 'input_key': key, 'upstream': spec['upstream'],
                    'license': spec['license'], 'recipe_sha256': digest(root / SPEC),
                    'generator_sha256': digest(Path(__file__)), 'tools': spec['tools'],
                    'weight_axis': axis, 'requested_codepoints': codepoints}
        for kind, suffix, data in [('subset', '.woff2', compressed.getvalue()), ('native', '.ttf', uncompressed.getvalue())]:
            filename = FONT_NAME + suffix
            manifest[kind] = {'path': filename, 'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data)}
            atomic_write(output / filename, data)
        groups = localized_characters(root)
        if groups:
            active = {group: spec['localized_fonts'][group] for group in groups}
            sources = localized_sources(generated_directory(cache, root), {**spec, 'localized_fonts': active},
                                        offline=True)
            manifest['localized'] = {}
            for group, points in groups.items():
                recipe = active[group]
                data = localized_font(sources[group], points, recipe, axis)
                filename = f'xdzip-design-sans-{group}.ttf'
                manifest['localized'][group] = {
                    'path': filename, 'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data),
                    'source_sha256': recipe['sha256'], 'license': recipe.get('license', spec['license']),
                    'requested_codepoints': points,
                }
                atomic_write(output / filename, data)
        atomic_write(output / 'manifest.json', (json.dumps(manifest, ensure_ascii=False, indent=2) + '\n').encode())
        return load_manifest(output, root=root, expected_key=key)


def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['prepare', 'build', 'check', 'localized-files'])
    parser.add_argument('--cache-dir', type=Path, default=DEFAULT_CACHE)
    parser.add_argument('--output-dir', type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument('--source', type=Path, help='Import a local upstream TTF after verifying the fixed SHA-256')
    parser.add_argument('--offline', action='store_true', help='Prepare only from an existing source cache')
    args = parser.parse_args()
    try:
        if args.operation == 'prepare':
            prepare(args.cache_dir, source=args.source, offline=args.offline)
            localized_sources(generated_directory(args.cache_dir), configuration(), offline=args.offline)
        elif args.operation == 'localized-files':
            print(';'.join(f'xdzip-design-sans-{group}.ttf' for group in localized_catalogs(ROOT)))
        else:
            result = generate(args.output_dir, cache=args.cache_dir, check=args.operation == 'check')
            print(f"UI font: {len(result['requested_codepoints'])} characters, TTF {result['native']['sha256']}")
    except (OSError, ValueError, KeyError, ImportError) as error:
        print(str(error) if isinstance(error, ValueError) else 'UI font stage failed: ' + type(error).__name__, file=sys.stderr)
        return 1
    return 0
