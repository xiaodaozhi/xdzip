#!/usr/bin/env python3
"""Reject tampered/mixed evidence and prevent automated release approval."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('readiness',ROOT/'scripts/wp12-readiness.py');gate=importlib.util.module_from_spec(spec);spec.loader.exec_module(gate)

class ReadinessTests(unittest.TestCase):
    def fixture(self,root):
        file=root/'build/ci.json';file.parent.mkdir();file.write_text(json.dumps({'status':'passed','revision':'a'*40,'all_jobs_succeeded':True,'product_platforms':['macos','windows','linux']}))
        return file,{'revision':'a'*40,'automatic':{'CI':[{'path':'build/ci.json','sha256':gate.digest(file),'kind':'ci'}]}}

    def test_hash_identity_and_path_are_enforced(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);file,manifest=self.fixture(root)
            gates={'automatic':[{'id':'CI','title':'owned','minimum_reports':1}],'external':[{'id':'LICENSE','title':'license review'}]}
            with patch.object(gate,'ROOT',root),patch.object(gate,'GATES',gates):
                result=gate.evaluate(manifest)
                self.assertTrue(result['automatic_complete']);self.assertFalse(result['release_ready'])
                self.assertEqual(result['external'][0]['status'],'pending')
                file.write_text('{}')
                with self.assertRaisesRegex(ValueError,'hash'):gate.evaluate(manifest)
                manifest['automatic']['CI'][0]['path']='../escape'
                with self.assertRaisesRegex(ValueError,'path'):gate.evaluate(manifest)

    def test_source_identity_and_dirty_are_not_accepted(self):
        record={'status':'passed','revision':'b'*40,'all_jobs_succeeded':True,'product_platforms':['macos','windows','linux']}
        with self.assertRaisesRegex(ValueError,'identity'):gate.validate_record('ci',record,'a'*40)
        record['revision']='a'*40;record['source_dirty']=True
        with self.assertRaisesRegex(ValueError,'Dirty'):gate.validate_record('ci',record,'a'*40)

    def test_performance_and_source_marker_are_required(self):
        with self.assertRaisesRegex(ValueError,'sample'):gate.validate_record('performance',{'status':'automated-observations-complete-reference-approval-pending','interactive_samples':5},'a'*40)
        with self.assertRaisesRegex(ValueError,'marker'):gate.validate_record('qt-source-product',{'status':'passed','events':[]},'a'*40)
        with self.assertRaisesRegex(ValueError,'actually'):gate.validate_record('qt-replacement',{'status':'passed'},'a'*40)

    def test_linkage_identity_string_is_checked(self):
        record={'status':'link-input-evidence-not-complete-attribution','identity':'xdzip 0.1.0-dev.4 (build 4, bbbbbbbbbbbb)','staticlib_members':['owned.o']}
        with self.assertRaisesRegex(ValueError,'identity'):gate.validate_record('linkage',record,'a'*40)

if __name__=='__main__':unittest.main()
