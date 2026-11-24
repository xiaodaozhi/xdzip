"""Check that UDF preparation clears only owned build registrations."""
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('udf_compatibility', Path(__file__).with_name('check-macos-udf-compatibility.py'))
udf = importlib.util.module_from_spec(spec)
spec.loader.exec_module(udf)


class RegistrationPreparationTests(unittest.TestCase):
    def test_clears_only_the_exact_owned_bundle_and_verifies_removal(self):
        app = Path('owned.app').resolve()
        with patch.object(udf, 'registered_apps', side_effect=[{app}, set()]), \
                patch.object(udf, 'build_copy', return_value=True), patch.object(udf, 'unregister') as remove:
            udf.clear_owned_build_registration(app)
            remove.assert_called_once_with(app)

    def test_clears_multiple_owned_build_copies(self):
        app = Path('owned.app').resolve()
        other = Path('owned-copy.app').resolve()
        with patch.object(udf, 'registered_apps', side_effect=[{app, other}, set()]), \
                patch.object(udf, 'build_copy', return_value=True), patch.object(udf, 'unregister') as remove:
            udf.clear_owned_build_registration(app)
            self.assertEqual({call.args[0] for call in remove.call_args_list}, {app, other})

    def test_rejects_foreign_or_nonconvergent_registrations(self):
        app = Path('owned.app').resolve()
        foreign = Path('foreign.app').resolve()
        for entries in ({foreign}, {app, foreign}):
            with self.subTest(entries=entries), \
                    patch.object(udf, 'registered_apps', return_value=entries), \
                    patch.object(udf, 'build_copy', side_effect=lambda path: path == app), patch.object(udf, 'unregister') as remove:
                with self.assertRaisesRegex(RuntimeError, 'unrelated'):
                    udf.clear_owned_build_registration(app)
                remove.assert_not_called()
        with patch.object(udf, 'registered_apps', return_value={app}), \
                patch.object(udf, 'build_copy', return_value=True), patch.object(udf, 'unregister'):
            with self.assertRaisesRegex(RuntimeError, 'did not clear'):
                udf.clear_owned_build_registration(app)


if __name__ == '__main__':
    unittest.main()
