#!/usr/bin/env python3
"""Boundary tests for import resolution, evidence scope, and source/binary binding."""
from pathlib import Path
import tempfile
import importlib.util
import unittest
from unittest.mock import patch
import deployment_audit as audit
from sbom import artifact_bom, source_bom


class AuditTests(unittest.TestCase):
    def test_native_tool_parsers_preserve_missing_imports_and_spaces(self):
        self.assertEqual(audit.windows_imports('    Qt6Core.dll\n  KERNEL32.dll\n  delay.dll\n Summary'), ['KERNEL32.dll','Qt6Core.dll','delay.dll'])
        self.assertEqual(audit.mac_imports('binary:\n\t@rpath/QtCore.framework/QtCore (compatibility version 6.0.0, current version 6.8.2)'), ['@rpath/QtCore.framework/QtCore'])
        self.assertEqual(audit.mac_rpaths('cmd LC_RPATH\n cmdsize 56\n path /SDK with spaces/lib (offset 12)'), ['/SDK with spaces/lib'])
        self.assertEqual(audit.linux_imports('linux-vdso.so.1 (0x0)\n libQt6Core.so => not found\n libx.so => /SDK with spaces/libx.so (0x1)'), [('libQt6Core.so',None),('libx.so',Path('/SDK with spaces/libx.so'))])

    def test_unresolved_does_not_become_ready(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);app=root/'app';app.write_bytes(b'\x7fELFowned fixture')
            with patch.object(audit,'run',return_value='libQt6Core.so => not found'):
                report=audit.audit(root,app,None,'linux','sdk')
            self.assertEqual(report['status'],'failed')
            self.assertEqual(report['unresolved_imports'],1)
            self.assertFalse(report['sdk_independent_imports'])

    def test_sdk_resolution_is_not_deployment(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root=base/'app';root.mkdir();app=root/'app';app.write_bytes(b'\x7fELFowned')
            sdk=base/'private-user-sdk';sdk.mkdir();lib=sdk/'libQt6Core.so';lib.write_bytes(b'\x7fELFqt')
            def output(*command):
                return f'libQt6Core.so => {lib} (0x1)' if Path(command[-1]).resolve()==app.resolve() else ''
            with patch.object(audit,'run',side_effect=output):
                report=audit.audit(root,app,sdk,'linux','sdk')
                deployed=audit.audit(root,app,sdk,'linux','deployed')
            self.assertEqual(report['status'],'passed')
            self.assertEqual(deployed['status'],'failed')
            self.assertFalse(report['sdk_independent_imports'])
            self.assertEqual(report['external_files'][0]['path'],'qt-sdk/libQt6Core.so')
            self.assertNotIn(str(base),str(report))
            before=report['files'][0]['sha256'];app.write_bytes(b'\x7fELFchanged')
            self.assertNotEqual(before,audit.digest(app))

    @unittest.skipUnless(hasattr(Path,'symlink_to'),'Symlink API unavailable')
    def test_symlink_escape_rejected_before_reading(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root=base/'app';root.mkdir();app=root/'app';app.write_bytes(b'\x7fELFowned')
            outside=base/'private';outside.write_text('private')
            try:(root/'escape').symlink_to(outside)
            except OSError:self.skipTest('Host cannot create symlinks')
            with self.assertRaisesRegex(ValueError,'symlink outside'):
                audit.audit(root,app,None,'linux','sdk')

    def test_source_graph_and_binary_identity_are_not_fabricated(self):
        inventory={'version_source':{'build':4},'display_version':'1.2.3-dev.4','revision':'123456789abc'+'f'*28,'dirty':False,
            'rust_packages':[{'name':'internal','version':'1.2.3-dev.4','license':'Apache-2.0','bom_ref':'cargo:internal','source_kind':'workspace'}],
            'rust_dependencies':[{'ref':'cargo:internal','dependsOn':[]}], 'qt_declared_product_modules':['Core'],'qt_version':'6.8.2','assets':[]}
        source=source_bom(inventory)
        self.assertNotIn('purl',source['components'][0])
        self.assertEqual(source['dependencies'],inventory['rust_dependencies'])
        observed={'status':'passed','platform':'macos','mode':'sdk','sdk_independent_imports':False,
            'identity':{'build':4,'version':'1.2.3-dev.4','revision':'123456789abc','dirty':False},
            'files':[{'path':'app','sha256':'a'*64}],'external_files':[], 'binary_imports':[]}
        bom=artifact_bom(observed,source)
        self.assertEqual([c['type'] for c in bom['components']],['file'])
        self.assertEqual(bom['compositions'][0]['aggregate'],'incomplete')
        observed['identity']['dirty']=True
        with self.assertRaisesRegex(ValueError,'identities differ'):artifact_bom(observed,source)
        observed['status']='failed'
        with self.assertRaisesRegex(ValueError,'failed imports'):artifact_bom(observed,source)

class ManualTests(unittest.TestCase):
    def setUp(self):
        spec=importlib.util.spec_from_file_location('private_test_form',Path(__file__).with_name('prepare-private-test.py'))
        self.form=importlib.util.module_from_spec(spec);spec.loader.exec_module(self.form)

    def test_preparation_never_marks_a_case_passed(self):
        record=self.form.prepare('macos')
        self.assertEqual(self.form.validate(record)['pending'],8)
        with self.assertRaisesRegex(ValueError,'incomplete'):self.form.validate(record,True)
        record['cases'].pop()
        with self.assertRaisesRegex(ValueError,'Missing'):self.form.validate(record)

    def test_empty_pass_and_unapproved_exclusion_rejected(self):
        record=self.form.prepare('windows');record['cases'][0]['status']='passed'
        with self.assertRaisesRegex(ValueError,'actual result'):self.form.validate(record)
        record['cases'][0]['actual']='candidate checked'
        with self.assertRaisesRegex(ValueError,'operator role'):self.form.validate(record)
        record['cases'][0]['status']='not_applicable'
        with self.assertRaisesRegex(ValueError,'release-owner decision'):self.form.validate(record)

if __name__=='__main__':unittest.main()
