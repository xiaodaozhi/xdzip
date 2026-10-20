#!/usr/bin/env python3
"""Cross-platform policy/negative tests, without SDKs, signatures or app launch."""
import json
import os
from pathlib import Path
import shutil
import stat
import tempfile
import unittest

from macos_bundle import (MACH_O_MAGICS, mach_o_files, optimize_bundle,
                          validate_optimized_bundle)


THIN = b'\xcf\xfa\xed\xfe'
FAT = b'\xca\xfe\xba\xbe'


def binary(path, arches=('arm64',), imports=(), magic=None):
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {'arches': list(arches), 'imports': list(imports)}
    path.write_bytes((magic or (FAT if len(arches) > 1 else THIN)) +
                     json.dumps(payload).encode() + b'\n' + b'x' * (128 * len(arches)))
    path.chmod(0o755)
    return path


class FakeTools:
    def __init__(self):
        self.commands = []

    def __call__(self, *args):
        self.commands.append(args)
        if args[0] == 'lipo':
            if args[1] == '-archs':
                return ' '.join(json.loads(Path(args[2]).read_bytes()[4:].split(b'\n')[0])['arches'])
            source, target = Path(args[1]), Path(args[-1])
            info = json.loads(source.read_bytes()[4:].split(b'\n')[0])
            binary(target, ('arm64',), info['imports'])
            return ''
        if args[0] == 'otool':
            info = json.loads(Path(args[-1]).read_bytes()[4:].split(b'\n')[0])
            return 'image:\n' + ''.join('\t' + name + ' (compatibility version 1.0.0)\n'
                                           for name in info['imports'])
        raise AssertionError('Unexpected tool: ' + args[0])


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding='utf-8')


def fixture(root):
    app = root/'xdzip.app'
    binary(app/'Contents/MacOS/xdzip')
    framework = app/'Contents/Frameworks/QtCore.framework'
    binary(framework/'Versions/A/QtCore', ('x86_64', 'arm64'))
    (framework/'Versions/Current').symlink_to('A')
    (framework/'QtCore').symlink_to('Versions/Current/QtCore')
    controls = app/'Contents/Resources/qml/QtQuick/Controls'
    write(controls/'qmldir', 'module QtQuick.Controls\nplugin qtquickcontrols2plugin\n'
          'optional import QtQuick.Controls.Material auto\ndefault import QtQuick.Controls.Basic auto\n')
    write(controls/'Basic/qmldir', 'module QtQuick.Controls.Basic\nplugin qtquickcontrols2basicstyleplugin\n')
    write(controls/'impl/qmldir', 'module QtQuick.Controls.impl\nplugin qtquickcontrols2implplugin\n')
    write(controls/'Basic/impl/qmldir', 'module QtQuick.Controls.Basic.impl\nplugin qtquickcontrols2basicstyleimplplugin\n')
    templates = controls.parent/'Templates'
    write(templates/'qmldir', 'module QtQuick.Templates\nplugin qtquicktemplates2plugin\n')
    for directory, name in ((controls, 'qtquickcontrols2plugin'),
                            (controls/'Basic', 'qtquickcontrols2basicstyleplugin'),
                            (controls/'impl', 'qtquickcontrols2implplugin'),
                            (controls/'Basic/impl', 'qtquickcontrols2basicstyleimplplugin'),
                            (templates, 'qtquicktemplates2plugin')):
        plugin = binary(app/'Contents/PlugIns/quick'/('lib' + name + '.dylib'), ('x86_64', 'arm64'))
        (directory/plugin.name).symlink_to(os.path.relpath(plugin, directory))
    write(controls/'Material/qmldir', 'module QtQuick.Controls.Material\nplugin qtquickcontrols2materialstyleplugin\n')
    write(controls/'Material/Button.qml', 'import QtQuick.Controls.Material\n')
    binary(app/'Contents/Frameworks/QtQuickControls2Material.framework/Versions/A/QtQuickControls2Material', ('x86_64', 'arm64'))
    binary(app/'Contents/PlugIns/quick/libqtquickcontrols2materialstyleplugin.dylib', ('x86_64', 'arm64'))
    # QWidget's macOS style serves native dialogs, independent of QML Basic.
    binary(app/'Contents/PlugIns/styles/libqmacstyle.dylib', ('x86_64', 'arm64'))
    return app


class BundlePolicyTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.app = fixture(self.root)
        self.tools = FakeTools()

    def optimize(self):
        return optimize_bundle(self.app, runner=self.tools)

    def snapshot(self):
        return {p.relative_to(self.app).as_posix(): (os.readlink(p) if p.is_symlink() else p.read_bytes())
                for p in self.app.rglob('*') if p.is_file() or p.is_symlink()}

    def test_thins_real_images_once_preserving_framework_aliases_and_modes(self):
        report = self.optimize()
        self.assertEqual(report['validation']['architectures'], ['arm64'])
        self.assertEqual(report['validation']['native_binary_count'], 8)
        self.assertEqual(len(report['thinned_binaries']), 7)
        self.assertGreater(report['style_bytes_saved'], 0)
        self.assertGreater(report['architecture_bytes_saved'], 0)
        self.assertEqual(report['bytes_saved'], report['style_bytes_saved'] + report['architecture_bytes_saved'])
        framework = self.app/'Contents/Frameworks/QtCore.framework'
        self.assertTrue((framework/'QtCore').is_symlink())
        self.assertEqual(os.readlink(framework/'Versions/Current'), 'A')
        self.assertEqual(stat.S_IMODE((framework/'QtCore').stat().st_mode), 0o755)
        self.assertTrue((self.app/'Contents/PlugIns/styles/libqmacstyle.dylib').exists())
        self.assertFalse((self.app/'Contents/Resources/qml/QtQuick/Controls/Material').exists())
        self.assertFalse(any('Material' in path.name for path in self.app.rglob('*')))
        self.assertTrue(all(not str(path).startswith(str(self.root)) for path in report['removed_paths']))

    def test_missing_arm64_fails_before_mutation(self):
        binary(self.app/'Contents/PlugIns/quick/intel.dylib', ('x86_64',))
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, 'contain arm64'):
            self.optimize()
        self.assertEqual(self.snapshot(), before)

    def test_validator_rejects_universal_without_repair(self):
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, 'arm64-only'):
            validate_optimized_bundle(self.app, runner=self.tools)
        self.assertEqual(self.snapshot(), before)

    def test_every_macho_magic_is_inspected_including_fat64_and_32bit(self):
        for index, magic in enumerate(sorted(MACH_O_MAGICS)):
            binary(self.app/'Contents/MacOS'/str(index), ('x86_64', 'arm64'), magic=magic)
        images = mach_o_files(self.app)
        self.assertEqual(len(images), 18)
        self.optimize()
        self.assertTrue(all(self.tools('lipo', '-archs', str(image)) == 'arm64'
                            for image in mach_o_files(self.app)))

    def test_escape_symlink_fails_without_opening_or_modifying_outside(self):
        outside = binary(self.root/'outside.dylib', ('x86_64', 'arm64'))
        content = outside.read_bytes()
        (self.app/'Contents/escape').symlink_to(self.root)
        with self.assertRaisesRegex(ValueError, 'escapes'):
            self.optimize()
        self.assertEqual(outside.read_bytes(), content)
        self.assertEqual(self.tools.commands, [])

    def test_dangling_or_cyclic_symlinks_fail_closed(self):
        path = self.app/'Contents/dangling'
        path.symlink_to('not-present')
        with self.assertRaisesRegex(ValueError, 'invalid symlink'):
            self.optimize()
        path.unlink()
        path.symlink_to('dangling')
        with self.assertRaisesRegex(ValueError, 'invalid symlink'):
            self.optimize()

    def test_root_symlink_is_not_a_mutation_entrypoint(self):
        link = self.root/'alias.app'
        link.symlink_to(self.app)
        with self.assertRaisesRegex(ValueError, 'real deployed'):
            optimize_bundle(link, runner=self.tools)

    def test_native_strong_dependency_on_removed_style_fails_before_pruning(self):
        binary(self.app/'Contents/MacOS/xdzip', imports=(
            '@rpath/QtQuickControls2Material.framework/Versions/A/QtQuickControls2Material',))
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, 'requires an omitted'):
            self.optimize()
        self.assertEqual(before, self.snapshot())

    def test_qmldir_strong_import_or_depends_on_removed_style_fails(self):
        for directive in ('import', 'default import', 'depends'):
            with self.subTest(directive=directive):
                write(self.app/'Contents/Resources/qml/Other/qmldir',
                      'module Other\n' + directive + ' QtQuick.Controls.Material auto\n')
                with self.assertRaisesRegex(ValueError, 'requires a removed style'):
                    self.optimize()
                self.assertTrue((self.app/'Contents/Resources/qml/QtQuick/Controls/Material').exists())

    def test_qml_component_strong_import_fails_even_without_qmldir_dependency(self):
        write(self.app/'Contents/Resources/qml/Other/Thing.qml', 'import QtQuick.Controls.Fusion\n')
        with self.assertRaisesRegex(ValueError, 'component requires a removed style'):
            self.optimize()

    def test_dialog_style_selectors_pruned_while_default_and_basic_remain(self):
        dialogs = self.app/'Contents/Resources/qml/QtQuick/Dialogs/quickimpl/qml'
        for style in ('Fusion', 'Material', 'Imagine', 'Universal'):
            write(dialogs/('+' + style)/'FileDialog.qml',
                  'import QtQuick.Controls.' + style + '\n')
        write(dialogs/'FileDialog.qml', 'import QtQuick.Controls.Basic\n')
        write(dialogs/'+Basic/FileDialog.qml', 'import QtQuick.Controls.Basic\n')
        report = self.optimize()
        self.assertTrue((dialogs/'FileDialog.qml').is_file())
        self.assertTrue((dialogs/'+Basic/FileDialog.qml').is_file())
        self.assertFalse((dialogs/'+Material').exists())
        self.assertEqual(sum('Dialogs/quickimpl/qml/+' in path for path in report['removed_paths']), 4)

    def test_unknown_dialog_selector_requires_review(self):
        write(self.app/'Contents/Resources/qml/QtQuick/Dialogs/quickimpl/qml/+Next/FileDialog.qml',
              'import QtQuick.Controls.Basic\n')
        with self.assertRaisesRegex(ValueError, 'Unknown Qt Quick Dialogs style selector'):
            self.optimize()

    def test_optional_catalog_remains_and_basic_is_required(self):
        self.optimize()
        catalog = self.app/'Contents/Resources/qml/QtQuick/Controls/qmldir'
        self.assertIn('optional import QtQuick.Controls.Material', catalog.read_text())
        before = self.snapshot()
        validate_optimized_bundle(self.app, runner=self.tools)
        self.assertEqual(before, self.snapshot())
        (catalog.parent/'Basic/libqtquickcontrols2basicstyleplugin.dylib').unlink()
        with self.assertRaisesRegex(ValueError, 'no deployed plugin'):
            validate_optimized_bundle(self.app, runner=self.tools)

    def test_required_impl_and_templates_modules_cannot_be_omitted(self):
        for relative in ('Controls/impl', 'Controls/Basic/impl', 'Templates'):
            with self.subTest(relative=relative):
                app = fixture(self.root/relative.replace('/', '-'))
                shutil.rmtree(app/'Contents/Resources/qml/QtQuick'/relative)
                with self.assertRaisesRegex(ValueError, 'modules are not declared'):
                    optimize_bundle(app, runner=self.tools)

    def test_required_module_cannot_drop_its_plugin_declaration(self):
        path = self.app/'Contents/Resources/qml/QtQuick/Templates/qmldir'
        write(path, 'module QtQuick.Templates\n')
        with self.assertRaisesRegex(ValueError, 'does not declare its plugin'):
            self.optimize()

    def test_retained_symlink_cannot_point_into_pruned_style(self):
        path = self.app/'Contents/Resources/qml/Other/retained.qml'
        path.parent.mkdir(parents=True)
        path.symlink_to('../QtQuick/Controls/Material/Button.qml')
        with self.assertRaisesRegex(ValueError, 'symlink requires removed style'):
            self.optimize()

    def test_validator_rejects_leftover_styles_even_if_all_images_are_arm64(self):
        for path in mach_o_files(self.app):
            info = json.loads(path.read_bytes()[4:].split(b'\n')[0])
            binary(path, ('arm64',), info['imports'])
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, 'Unused Qt Quick Controls styles remain'):
            validate_optimized_bundle(self.app, runner=self.tools)
        self.assertEqual(self.snapshot(), before)

    def test_unknown_sdk_style_layout_is_not_silently_pruned(self):
        paths = (
            ('Contents/Resources/qml/QtQuick/Controls/Next/qmldir', 'module QtQuick.Controls.Next\n'),
            ('Contents/Resources/qml/Other/qmldir', 'module Other\noptional import QtQuick.Controls.Next auto\n'),
            ('Contents/Frameworks/QtQuickControls2Next.framework/payload', 'data'),
            ('Contents/PlugIns/quick/libqtquickcontrols2nextstyleplugin.dylib', 'data'),
        )
        for relative, contents in paths:
            with self.subTest(relative=relative):
                path = self.app/relative
                write(path, contents)
                before = self.snapshot()
                with self.assertRaisesRegex(ValueError, 'Unknown Qt Quick Controls'):
                    self.optimize()
                self.assertEqual(before, self.snapshot())
                if '.framework/' in relative:
                    shutil.rmtree(path.parent)
                elif '/Next/' in relative:
                    shutil.rmtree(path.parent)
                else:
                    path.unlink()

    def test_source_and_hardlinked_sdk_payload_are_not_edited(self):
        source = self.app
        source_bytes = self.snapshot()
        stage = self.root/'stage/xdzip.app'
        shutil.copytree(source, stage, symlinks=True)
        primary = stage/'Contents/Frameworks/QtCore.framework/Versions/A/QtCore'
        sdk = self.root/'sdk/QtCore'
        sdk.parent.mkdir()
        os.link(primary, sdk)
        sdk_content = sdk.read_bytes()
        optimize_bundle(stage, runner=self.tools)
        self.assertEqual(sdk.read_bytes(), sdk_content)
        self.assertEqual(self.snapshot(), source_bytes)

    def test_lipo_failure_leaves_no_temporary_image_and_does_not_claim_success(self):
        def failing(*command):
            if command[0] == 'lipo' and '-thin' in command:
                raise ValueError('lipo failed')
            return self.tools(*command)
        with self.assertRaisesRegex(ValueError, 'lipo failed'):
            optimize_bundle(self.app, runner=failing)
        self.assertFalse(list(self.app.rglob('.arm64-*')))


if __name__ == '__main__':
    unittest.main()
