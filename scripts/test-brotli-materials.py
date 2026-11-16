#!/usr/bin/env python3
"""Verify exact RFC-data notice retention and rejection without invoking packaging."""
import json
from pathlib import Path
import shutil
import tempfile
import unittest
import brotli_data_materials as materials


class MaterialTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(prefix='brotli-materials-', dir=materials.ROOT/'build')
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)/'source'
        self.output = Path(self.scratch.name)/'notices'
        for relative in ['third_party/brotli-rfc7932', 'crates/xdzip-compression/src/brotli/data.rs']:
            source = materials.ROOT/relative
            target = self.root/relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copytree(source,target) if source.is_dir() else shutil.copyfile(source,target)

    def test_exact_notices_are_retained_idempotently_without_reference_runtime(self):
        files, record = materials.collect(self.output,root=self.root)
        self.assertEqual([p.name for p in files],['README.md','LICENSE.txt','sources.json'])
        self.assertFalse(record['reference_implementation_in_product'])
        for path in files:
            self.assertEqual(path.read_bytes(),(self.root/'third_party/brotli-rfc7932'/path.name).read_bytes())
        self.assertEqual(materials.collect(self.output,root=self.root)[1],record)
        self.assertFalse((self.output/materials.DIRECTORY/'dictionary.bin').exists())

    def test_changed_license_and_dictionary_fail_before_writing(self):
        for name in ['LICENSE.txt','dictionary.bin']:
            path=self.root/'third_party/brotli-rfc7932'/name
            original=path.read_bytes();path.write_bytes(original+b'changed')
            with self.assertRaises(ValueError):materials.collect(self.output,root=self.root)
            self.assertFalse(self.output.exists());path.write_bytes(original)

    def test_changed_wire_tables_fail_before_writing(self):
        path=self.root/'crates/xdzip-compression/src/brotli/data.rs';path.write_bytes(path.read_bytes()+b'changed')
        with self.assertRaises(ValueError):materials.collect(self.output,root=self.root)
        self.assertFalse(self.output.exists())

    def test_changed_existing_notices_are_preserved_instead_of_overwritten(self):
        materials.collect(self.output,root=self.root)
        path=self.output/materials.DIRECTORY/'LICENSE.txt';path.write_bytes(b'keep')
        with self.assertRaises(ValueError):materials.collect(self.output,root=self.root)
        self.assertEqual(path.read_bytes(),b'keep')

    def test_inconsistent_scope_is_rejected_before_writing(self):
        path=self.root/'third_party/brotli-rfc7932/sources.json';record=json.loads(path.read_bytes());record['reference_implementation_in_product']=True;path.write_text(json.dumps(record),encoding='utf-8')
        with self.assertRaises(ValueError):materials.collect(self.output,root=self.root)
        self.assertFalse(self.output.exists())

    def test_misstated_original_identity_is_rejected_before_writing(self):
        path=self.root/'third_party/brotli-rfc7932/sources.json';record=json.loads(path.read_bytes());record['dictionary_sha256']='0'*64;path.write_text(json.dumps(record),encoding='utf-8')
        with self.assertRaises(ValueError):materials.collect(self.output,root=self.root)
        self.assertFalse(self.output.exists())


if __name__=='__main__':
    unittest.main()
