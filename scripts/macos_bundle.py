#!/usr/bin/env python3
"""Arm64/Basic-only policy for an already deployed, disposable macOS app copy.

Optimization runs before signing. It never locates or modifies the SDK or the
source app; callers must supply their new staging copy. Validation is read-only.
"""
import os
from pathlib import Path
import re
import shutil
import stat
import tempfile

from deployment_audit import mac_imports, run


# Include 32-bit and FAT_MAGIC_64 variants too: unsupported images must fail
# inspection, rather than silently evade the arm64-only contract.
MACH_O_MAGICS = {
    b'\xfe\xed\xfa\xce', b'\xce\xfa\xed\xfe',
    b'\xfe\xed\xfa\xcf', b'\xcf\xfa\xed\xfe',
    b'\xca\xfe\xba\xbe', b'\xbe\xba\xfe\xca',
    b'\xca\xfe\xba\xbf', b'\xbf\xba\xfe\xca',
}
UNUSED_STYLES = ('Fusion', 'Material', 'Imagine', 'Universal', 'FluentWinUI3',
                 'Windows', 'macOS', 'iOS')
RETAINED_MODULES = {'QtQuick.Controls', 'QtQuick.Controls.impl',
                    'QtQuick.Controls.Basic', 'QtQuick.Controls.Basic.impl'}
REQUIRED_MODULE_PLUGINS = {
    'QtQuick.Controls': 'qtquickcontrols2plugin',
    'QtQuick.Controls.impl': 'qtquickcontrols2implplugin',
    'QtQuick.Controls.Basic': 'qtquickcontrols2basicstyleplugin',
    'QtQuick.Controls.Basic.impl': 'qtquickcontrols2basicstyleimplplugin',
    'QtQuick.Templates': 'qtquicktemplates2plugin',
}
RETAINED_FRAMEWORKS = {'QtQuickControls2', 'QtQuickControls2Impl',
                       'QtQuickControls2Basic', 'QtQuickControls2BasicStyleImpl'}
STYLE_FRAMEWORKS = {
    'QtQuickControls2' + suffix
    for suffix in ('Fusion', 'FusionStyleImpl', 'Material', 'MaterialStyleImpl',
                   'Imagine', 'ImagineStyleImpl', 'Universal', 'UniversalStyleImpl',
                   'FluentWinUI3', 'FluentWinUI3StyleImpl', 'WindowsStyleImpl',
                   'MacOSStyleImpl', 'IOSStyleImpl')
}
RETAINED_PLUGINS = {'qtquickcontrols2plugin', 'qtquickcontrols2implplugin',
                    'qtquickcontrols2basicstyleplugin', 'qtquickcontrols2basicstyleimplplugin'}
STYLE_PLUGINS = {'qtquickcontrols2' + style + suffix
                 for style in ('fusion', 'material', 'imagine', 'universal',
                               'fluentwinui3', 'windows', 'macos', 'ios')
                 for suffix in ('styleplugin', 'styleimplplugin')}
STYLE_PLUGINS.add('qtquickcontrols2nativestyleplugin')


def _tree(app):
    app = Path(app).absolute()
    if app.is_symlink() or not app.is_dir() or app.suffix != '.app':
        raise ValueError('Expected a real deployed application directory')
    app = app.resolve()
    paths = []
    # os.walk does not traverse directory links. Check every link before any
    # payload is opened, including dangling/cyclic links and directory aliases.
    for directory, directories, files in os.walk(app, followlinks=False):
        for name in sorted(directories + files):
            path = Path(directory)/name
            if path.is_symlink():
                try:
                    target = path.resolve(strict=True)
                except (OSError, RuntimeError) as error:
                    raise ValueError('Bundle contains an invalid symlink') from error
                if not target.is_relative_to(app):
                    raise ValueError('Bundle symlink escapes application')
            paths.append(path)
    return app, sorted(paths)


def mach_o_files(app):
    """Return unique real Mach-O files, after checking the full tree boundary."""
    _, paths = _tree(app)
    images = set()
    for path in paths:
        if path.is_file() and not path.is_symlink():
            with path.open('rb') as stream:
                if stream.read(4) in MACH_O_MAGICS:
                    images.add(path)
    return sorted(images)


def _sizes(app):
    app, paths = _tree(app)
    result = dict.fromkeys(('total', 'frameworks', 'plugins', 'qml', 'other'), 0)
    for path in paths:
        if not path.is_file() or path.is_symlink():
            continue
        size = path.stat().st_size
        relative = path.relative_to(app).as_posix()
        category = ('frameworks' if relative.startswith('Contents/Frameworks/') else
                    'plugins' if relative.startswith('Contents/PlugIns/') else
                    'qml' if relative.startswith('Contents/Resources/qml/') else 'other')
        result[category] += size
        result['total'] += size
    return result


def _removed_module(name):
    return (name == 'QtQuick.NativeStyle' or name.startswith('QtQuick.NativeStyle.') or
            any(name == 'QtQuick.Controls.' + style or
                name.startswith('QtQuick.Controls.' + style + '.') for style in UNUSED_STYLES))


def _check_module_name(name):
    if (name.startswith('QtQuick.Controls.') and name not in RETAINED_MODULES and
            not _removed_module(name)):
        raise ValueError('Unknown Qt Quick Controls style module; review the deployment policy')


def _policy(app, paths):
    """Compute precise removals; unknown SDK style layouts require review."""
    qml = app/'Contents/Resources/qml'
    controls = qml/'QtQuick/Controls'
    if not (controls/'qmldir').is_file() or not (controls/'Basic/qmldir').is_file():
        raise ValueError('Basic Controls deployment is missing')
    removals = set()
    for child in controls.iterdir():
        if child.is_dir():
            if child.name in UNUSED_STYLES:
                removals.add(child)
            elif child.name not in ('Basic', 'impl', 'designer'):
                raise ValueError('Unknown Qt Quick Controls style directory; review the deployment policy')
    native_style = qml/'QtQuick/NativeStyle'
    if native_style.exists():
        removals.add(native_style)
    # QtQuick.Dialogs ships style-specific file-selector variants too. Their
    # strong imports are valid only with that optional style, so prune the
    # precise known variants along with it, preserving the default Basic path.
    dialogs = qml/'QtQuick/Dialogs/quickimpl/qml'
    if dialogs.is_dir():
        for child in dialogs.iterdir():
            if child.name.startswith('+'):
                if child.name in ('+Fusion', '+Material', '+Imagine', '+Universal'):
                    removals.add(child)
                elif child.name != '+Basic':
                    raise ValueError('Unknown Qt Quick Dialogs style selector; review the deployment policy')
    for path in paths:
        if path.parent == app/'Contents/Frameworks' and path.name.startswith('QtQuickControls2'):
            name = path.name.removesuffix('.framework')
            if not path.name.endswith('.framework') or name not in RETAINED_FRAMEWORKS | STYLE_FRAMEWORKS:
                raise ValueError('Unknown Qt Quick Controls framework; review the deployment policy')
            if name in STYLE_FRAMEWORKS:
                removals.add(path)
        if path.name.startswith('libqtquickcontrols2') and path.suffix == '.dylib':
            name = path.name[3:-6]
            if name not in RETAINED_PLUGINS | STYLE_PLUGINS:
                raise ValueError('Unknown Qt Quick Controls plugin; review the deployment policy')
            if name in STYLE_PLUGINS:
                removals.add(path)
    return sorted(path for path in removals if not any(parent in removals for parent in path.parents))


def _under(path, removals):
    return any(path == removed or path.is_relative_to(removed) for removed in removals)


def _qml_closure(app, paths, removals):
    modules, plugins, required = set(), set(), set()
    for path in paths:
        if _under(path, removals) or not path.is_file() or path.name != 'qmldir':
            continue
        lines = path.read_text(encoding='utf-8').splitlines()
        declarations = [tokens[1] for line in lines
                        if len(tokens := line.split('#', 1)[0].split()) == 2 and tokens[0] == 'module']
        if len(declarations) != 1:
            raise ValueError('Deployed qmldir must declare exactly one module')
        expected_plugin = REQUIRED_MODULE_PLUGINS.get(declarations[0])
        declared_plugins = set()
        for line in lines:
            tokens = line.split('#', 1)[0].split()
            if not tokens:
                continue
            if tokens[0] == 'module' and len(tokens) == 2:
                name = tokens[1]
                _check_module_name(name)
                if _removed_module(name):
                    raise ValueError('Unused style module remains outside its declared location')
                if name in RETAINED_MODULES:
                    modules.add(name)
            optional = tokens[0] == 'optional'
            directive = tokens[1:] if tokens[0] in ('optional', 'default') else tokens
            if len(directive) >= 2 and directive[0] in ('import', 'depends'):
                name = directive[1]
                _check_module_name(name)
                if _removed_module(name) and not optional:
                    raise ValueError('Retained QML module requires a removed style')
            if len(directive) >= 2 and directive[0] == 'plugin':
                name = directive[1]
                declared_plugins.add(name)
                if name.startswith('qtquickcontrols2') or name == expected_plugin:
                    if name not in RETAINED_PLUGINS:
                        if name != 'qtquicktemplates2plugin':
                            raise ValueError('Retained QML module requires an unused or unknown style plugin')
                    target = path.parent/('lib' + name + '.dylib')
                    if (not target.is_file() or _under(target, removals) or
                            _under(target.resolve(), removals)):
                        raise ValueError('Retained Controls module has no deployed plugin')
                    if name in RETAINED_PLUGINS:
                        plugins.add(name)
        if expected_plugin:
            if expected_plugin not in declared_plugins:
                raise ValueError('Required Basic Controls module does not declare its plugin')
            required.add(declarations[0])
    if required != set(REQUIRED_MODULE_PLUGINS):
        raise ValueError('Required Basic Controls or Templates modules are not declared')
    # .qml imports are always strong. Keep optional catalog declarations intact,
    # but never permit a real component to require an omitted style.
    for path in paths:
        if _under(path, removals) or path.suffix != '.qml' or not path.is_file():
            continue
        text = path.read_text(encoding='utf-8')
        text = re.sub(r'/\*.*?\*/|//[^\n]*', '', text, flags=re.S)
        for name in re.findall(r'^\s*import\s+([A-Za-z_][\w.]*)', text, re.M):
            _check_module_name(name)
            if _removed_module(name):
                raise ValueError('Retained QML component requires a removed style')
    return sorted(modules), sorted(plugins)


def _symlink_closure(paths, removals):
    for path in paths:
        if path.is_symlink() and not _under(path, removals) and _under(path.resolve(), removals):
            raise ValueError('Retained bundle symlink requires removed style content')


def _native_closure(images, removals, runner):
    for image in images:
        if _under(image, removals):
            continue
        for name in mac_imports(runner('otool', '-arch', 'arm64', '-L', str(image))):
            frameworks = re.findall(r'(QtQuickControls2[^/]*?)\.framework(?:/|$)', name)
            if any(value not in RETAINED_FRAMEWORKS for value in frameworks):
                raise ValueError('Retained native image requires an omitted or unknown Controls framework')
            basename = Path(name).name
            if basename.startswith('libqtquickcontrols2') and basename.endswith('.dylib'):
                if basename[3:-6] not in RETAINED_PLUGINS:
                    raise ValueError('Retained native image requires an omitted or unknown Controls plugin')


def _architectures(images, runner, *, optimized):
    result = {}
    for image in images:
        archs = runner('lipo', '-archs', str(image)).split()
        if not archs or len(set(archs)) != len(archs) or 'arm64' not in archs:
            raise ValueError('Every deployed Mach-O image must contain arm64')
        if optimized and archs != ['arm64']:
            raise ValueError('Deployed Mach-O image is not arm64-only')
        result[image] = archs
    if not images:
        raise ValueError('Application contains no Mach-O images')
    return result


def validate_optimized_bundle(app, *, runner=None):
    """Read-only verification, never silently repairs an invalid deployment."""
    runner = runner or run
    app, paths = _tree(app)
    images = mach_o_files(app)
    _architectures(images, runner, optimized=True)
    removals = _policy(app, paths)
    if removals:
        raise ValueError('Unused Qt Quick Controls styles remain in the deployed application')
    modules, plugins = _qml_closure(app, paths, [])
    _native_closure(images, [], runner)
    return {'status': 'passed', 'target_arch': 'arm64', 'control_style': 'Basic',
            'native_binary_count': len(images), 'architectures': ['arm64'],
            'native_files': [path.relative_to(app).as_posix() for path in images],
            'style_modules': modules, 'style_plugins': plugins,
            'required_qml_modules': sorted(REQUIRED_MODULE_PLUGINS), 'bytes': _sizes(app)}


def optimize_bundle(app, *, runner=None):
    """Prune unused Controls styles and thin every real image before signing.

    All architecture and dependency checks run before mutation. A later tool
    failure can leave the disposable stage incomplete; callers must discard it.
    No file outside the checked app tree is modified or followed for writes.
    """
    runner = runner or run
    app, paths = _tree(app)
    images = mach_o_files(app)
    architectures = _architectures(images, runner, optimized=False)
    removals = _policy(app, paths)
    _symlink_closure(paths, removals)
    _qml_closure(app, paths, removals)
    _native_closure(images, removals, runner)
    before = _sizes(app)
    for path in removals:
        if path.is_symlink() or path.is_file():
            path.unlink()
        else:
            shutil.rmtree(path)
    pruned = _sizes(app)
    thinned = []
    for path, archs in architectures.items():
        if _under(path, removals) or archs == ['arm64']:
            continue
        # New output + atomic replace preserves framework aliases and prevents
        # accidental in-place edits through a hard link to an SDK/source image.
        mode = stat.S_IMODE(path.stat().st_mode)
        descriptor, temporary_name = tempfile.mkstemp(prefix='.arm64-', dir=path.parent)
        os.close(descriptor)
        temporary = Path(temporary_name)
        try:
            runner('lipo', str(path), '-thin', 'arm64', '-output', str(temporary))
            _architectures([temporary], runner, optimized=True)
            temporary.chmod(mode)
            os.replace(temporary, path)
        finally:
            temporary.unlink(missing_ok=True)
        thinned.append(path.relative_to(app).as_posix())
    validation = validate_optimized_bundle(app, runner=runner)
    after = validation['bytes']
    return {'status': 'passed', 'target_arch': 'arm64', 'control_style': 'Basic',
            'before_bytes': before, 'after_bytes': after,
            'bytes_saved': before['total'] - after['total'],
            'style_bytes_saved': before['total'] - pruned['total'],
            'architecture_bytes_saved': pruned['total'] - after['total'],
            'thinned_binaries': thinned,
            'removed_paths': [path.relative_to(app).as_posix() for path in removals],
            'validation': validation}
