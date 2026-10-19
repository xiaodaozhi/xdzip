#!/usr/bin/env python3
"""Check attribution evidence boundaries, cross-document graph and tool exclusion."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from runtime_attribution import attribute, bounded, closure, license_materials, load_qt
from unittest.mock import patch
from runtime_rust import runtime_candidates
from sbom import artifact_bom


class AttributionTests(unittest.TestCase):
    def fixture(self, base):
        sdk=base/'sdk';(sdk/'sbom').mkdir(parents=True);(sdk/'lib').mkdir()
        library=sdk/'lib/libQt6Core.so.6';library.write_bytes(b'\x7fELFowned test')
        document={'spdxVersion':'SPDX-2.3','documentNamespace':'https://qt.io/test',
            'packages':[{'SPDXID':'repo','name':'qtbase','versionInfo':'6.8.2'},
                        {'SPDXID':'module','name':'Core','versionInfo':'6.8.2','licenseConcluded':'LGPL-3.0-only'},
                        {'SPDXID':'third','name':'embedded-test','versionInfo':'1','licenseConcluded':'MIT'}],
            'files':[{'SPDXID':'file','fileName':'./lib/libQt6Core.so.6','checksums':[{'algorithm':'SHA1','checksumValue':'0'*40}]}],
            'relationships':[{'relationshipType':'CONTAINS','spdxElementId':'module','relatedSpdxElement':'file'},
                             {'relationshipType':'DEPENDS_ON','spdxElementId':'module','relatedSpdxElement':'third'}]}
        (sdk/'sbom/qtbase-6.8.2.spdx.json').write_text(json.dumps(document))
        root=base/'payload';root.mkdir();deployed=root/'libQt6Core.so.6';deployed.write_bytes(library.read_bytes())
        audit={'status':'passed','platform':'linux','mode':'deployed','identity':{'version':'1','revision':'a'*12,'build':1,'dirty':False},
               'files':[{'path':deployed.name,'sha256':hashlib.sha256(deployed.read_bytes()).hexdigest()}],'external_files':[],'binary_imports':[],'sdk_independent_imports':True}
        return sdk,root,audit

    def test_checksum_difference_is_not_misrepresented_as_verification(self):
        with tempfile.TemporaryDirectory() as temp:
            sdk,root,audit=self.fixture(Path(temp));report=attribute(audit,root,sdk)
            self.assertEqual(len(report['components']),2)
            self.assertEqual(report['files'][0]['evidence'],'sdk-byte-identity')
            self.assertFalse(report['files'][0]['upstream_checksums'][0]['matches_sdk'])
            self.assertIn('upstream-sdk-checksum-difference',{g['kind'] for g in report['gaps']})
            self.assertNotIn(str(sdk),json.dumps(report))

    def test_changed_artifact_and_stale_rewrite_record_fail(self):
        with tempfile.TemporaryDirectory() as temp:
            sdk,root,audit=self.fixture(Path(temp));(root/'libQt6Core.so.6').write_bytes(b'\x7fELFrewritten')
            with self.assertRaisesRegex(ValueError,'changed after'):attribute(audit,root,sdk)
            audit['files'][0]['sha256']=hashlib.sha256((root/'libQt6Core.so.6').read_bytes()).hexdigest()
            provenance={'sdk_files':[{'path':'libQt6Core.so.6','sdk_path':'lib/libQt6Core.so.6','sdk_sha256':'0'*64,'artifact_sha256':audit['files'][0]['sha256']}]}
            with self.assertRaisesRegex(ValueError,'Stale'):attribute(audit,root,sdk,provenance)

    def test_filename_only_rewrite_remains_a_gap(self):
        with tempfile.TemporaryDirectory() as temp:
            sdk,root,audit=self.fixture(Path(temp));(root/'libQt6Core.so.6').write_bytes(b'\x7fELFpatched')
            audit['files'][0]['sha256']=hashlib.sha256((root/'libQt6Core.so.6').read_bytes()).hexdigest()
            report=attribute(audit,root,sdk)
            self.assertEqual(report['files'][0]['evidence'],'filename-only-unverified-rewrite')
            self.assertIn('unverified-rewrite',{g['kind'] for g in report['gaps']})

    def test_sdk_metadata_escape_and_version_mismatch_fail(self):
        with tempfile.TemporaryDirectory() as temp:
            sdk,root,audit=self.fixture(Path(temp))
            with self.assertRaisesRegex(ValueError,'Unbounded'):bounded(sdk,'../private')
            with self.assertRaisesRegex(ValueError,'Unbounded'):bounded(sdk,'C:\\private')
            with self.assertRaisesRegex(ValueError,'Unbounded'):bounded(sdk,'C:/private')
            path=sdk/'sbom/qtbase-6.8.2.spdx.json';doc=json.loads(path.read_text());doc['packages'][0]['versionInfo']='6.9.0';path.write_text(json.dumps(doc))
            with self.assertRaisesRegex(ValueError,'version mismatch'):load_qt(sdk)

    def test_cross_document_missing_edges_are_retained(self):
        selected,missing=closure({'base:core'},{'base:core':{},'declarative:qml':{}},
            {'base:core':{'declarative:qml'},'declarative:qml':{'absent:third'}})
        self.assertEqual(selected,{'base:core','declarative:qml'});self.assertEqual(missing,{'absent:third'})

    def test_sbom_attribution_is_bound_to_audit_and_preserves_partial_scope(self):
        with tempfile.TemporaryDirectory() as temp:
            sdk,root,audit=self.fixture(Path(temp));report=attribute(audit,root,sdk)
            source={'metadata':{'component':{'version':'1'},'properties':[{'name':'xdzip:'+k,'value':v} for k,v in {'revision':'a'*40,'dirty':'false','build':'1'}.items()]}}
            bom=artifact_bom(audit,source,report)
            self.assertEqual(bom['compositions'][0]['aggregate'],'incomplete')
            self.assertTrue(any(c['name']=='embedded-test' for c in bom['components']))
            report['audit_sha256']='0'*64
            with self.assertRaisesRegex(ValueError,'identities differ'):artifact_bom(audit,source,report)

    def test_license_cache_corruption_is_not_trusted(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);(base/'scripts').mkdir();cache=base/'cache';cache.mkdir()
            config={'licenses':[{'id':'MIT','url':'https://raw.githubusercontent.com/qt/qtbase/'+'a'*40+'/LICENSES/MIT.txt','sha256':'0'*64}]}
            (base/'scripts/qt-license-sources.json').write_text(json.dumps(config));(cache/'MIT.txt').write_text('corrupt')
            with patch('runtime_attribution.ROOT',base):
                with self.assertRaisesRegex(ValueError,'hash mismatch'):license_materials(base/'out',cache)

    def test_normal_rust_graph_excludes_build_dev_and_proc_macros(self):
        packages=[{'id':n,'name':'xdzip-qt-bridge' if n=='root' else n,'source':None if n=='root' else 'registry',
                   'targets':[{'kind':['proc-macro'] if n=='macro' else ['lib']}]} for n in ['root','runtime','build','dev','macro','macrodep']]
        def dep(name,kind):return {'pkg':name,'dep_kinds':[{'kind':kind}]}
        nodes=[{'id':'root','deps':[dep('runtime',None),dep('build','build'),dep('dev','dev'),dep('macro',None)]},
               {'id':'macro','deps':[dep('macrodep',None)]}]+[{'id':n,'deps':[]} for n in ['runtime','build','dev','macrodep']]
        result=runtime_candidates({'packages':packages,'resolve':{'nodes':nodes}},{p['id']:p['id'] for p in packages})
        self.assertEqual(result['packages'],['root','runtime'])


if __name__=='__main__':unittest.main()
