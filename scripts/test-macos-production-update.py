# scripts/test-macos-production-update.py

"""Check the unsigned-production preparation boundary without running a Mac app."""
import base64
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import shutil
import tempfile
import tarfile
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('macos_production_update',ROOT/'scripts/check-macos-product-update.py')
mac=importlib.util.module_from_spec(spec);spec.loader.exec_module(mac)


class ProductionPreparationTests(unittest.TestCase):
    def setUp(self):
        self.temporary=tempfile.TemporaryDirectory(prefix='xdzip-mac-production-contract-')
        self.addCleanup(self.temporary.cleanup)
        self.root=Path(self.temporary.name).resolve()
        (self.root/'resources').mkdir()
        for name in mac.PUBLIC_RESOURCES:
            shutil.copyfile(ROOT/'resources'/name,self.root/'resources'/name)
        self.output=self.root/'build/production-update/macos';self.output.mkdir(parents=True)
        self.name='xdzip-owned-update-更新-'+('a'*32)+'.app'
        self.source_commit='c'*40
        self.public=base64.b64encode(bytes.fromhex(json.loads((self.root/'resources/sparkle-update-key.json').read_bytes())['public_key'])).decode('ascii')
        self.products={}
        for label,number in (('old',1),('new',2)):
            app=self.output/label/self.name;(app/'Contents/MacOS').mkdir(parents=True)
            (app/'Contents/Info.plist').write_bytes(plistlib.dumps({'SUPublicEDKey':self.public}))
            (app/'Contents/MacOS/xdzip').write_bytes(('private '+label).encode())
            commit=str(number)*40
            self.products[label]=dict(app=app,commit=commit,identity=dict(product='xdzip',version='0.2.0-alpha.'+str(number),
                build=199+number,revision=commit[:12],dirty=False))
        self.signer=self.root/'compiled-tool';self.signer.write_bytes(b'compiled without signing authority')
        self.root_patch=patch.object(mac,'ROOT',self.root);self.root_patch.start();self.addCleanup(self.root_patch.stop)
        self.env_patch=patch.dict(os.environ,{},clear=True);self.env_patch.start();self.addCleanup(self.env_patch.stop)
        self.calls=[]
        self.run_patch=patch.object(mac,'run',side_effect=self.native);self.run_patch.start();self.addCleanup(self.run_patch.stop)

    def native(self,arguments,*_,**__):
        self.calls.append(list(map(str,arguments)))
        if arguments[0]=='git':return (self.source_commit+'\n').encode()
        if arguments[0]=='ditto':Path(arguments[-1]).write_bytes(b'owned unsigned native archive');return b''
        raise AssertionError('Preparation or validation unexpectedly ran a signer or product')

    def prepare(self):
        return mac.production_prepared(self.output,self.products,self.signer)

    def load(self):
        with patch.object(mac,'verify_release_updates') as verifier:
            result=mac.load_production_prepared(self.output)
        return result,verifier

    def test_preparation_has_no_signing_command_and_preserves_exact_public_bytes(self):
        record=self.prepare()
        self.assertEqual(set(record),{'schema_version','platform','source_commit','tool','artifacts','products'})
        self.assertEqual(record['source_commit'],self.source_commit)
        self.assertEqual(len(record['artifacts']),5)
        self.assertTrue(all(call[0] in ('git','ditto') for call in self.calls))
        self.assertEqual(record['tool']['path'],'signing/xdzip-update-tool')
        for name in mac.PUBLIC_RESOURCES:
            self.assertEqual((self.output/'resources'/name).read_bytes(),(self.root/'resources'/name).read_bytes())
        for name in ('release.json.sig','macos-update.json','appcast.xml'):
            self.assertFalse((self.output/'owned'/name).exists())
        self.assertEqual({p['SUPublicEDKey'] for p in record['products'].values()},{self.public})

    def test_execution_reloads_frozen_identity_and_verifies_all_signed_attachments(self):
        self.prepare()
        (record,products,manifest,archive),verifier=self.load()
        self.assertEqual(record['app_name'],self.name)
        self.assertEqual(record['token'],'a'*32)
        self.assertEqual(products,self.products)
        self.assertEqual(manifest['commit'],self.products['new']['commit'])
        self.assertEqual(archive.name,'xdzip-0.2.0-alpha.2-macos-arm64.app.zip')
        verifier.assert_called_once_with(self.output/'owned',manifest,tool=self.output/'signing/xdzip-update-tool',
            keyring=self.output/'resources/update-trust.json',public_key=self.output/'resources/sparkle-update-key.json')

    def test_changed_archive_is_rejected_before_verification_or_native_execution(self):
        self.prepare()
        (self.output/'owned/xdzip-0.2.0-alpha.2-macos-arm64.app.zip').write_bytes(b'changed')
        with patch.object(mac,'verify_release_updates') as verifier:
            with self.assertRaisesRegex(RuntimeError,'input changed'):mac.load_production_prepared(self.output)
        verifier.assert_not_called()

    def test_changed_native_plist_is_rejected_even_if_archive_is_unchanged(self):
        self.prepare()
        (self.products['old']['app']/'Contents/Info.plist').write_bytes(plistlib.dumps({'SUPublicEDKey':'wrong'}))
        with self.assertRaisesRegex(RuntimeError,'application identity changed'):self.load()

    def test_missing_production_plist_key_prevents_preparation(self):
        (self.products['new']['app']/'Contents/Info.plist').write_bytes(plistlib.dumps({}))
        with self.assertRaisesRegex(RuntimeError,'does not embed'):self.prepare()
        self.assertFalse((self.output/'production-update-prepared.json').exists())

    def test_changed_checkout_is_rejected_before_signed_input_validation(self):
        self.prepare();self.source_commit='d'*40
        with self.assertRaisesRegex(RuntimeError,'source or trust identity changed'):self.load()

    def test_foreign_record_path_cannot_select_a_signer(self):
        record=self.prepare();record['tool']['path']='../../compiled-tool'
        mac.dump(self.output/'production-update-prepared.json',record)
        with self.assertRaisesRegex(RuntimeError,'tool changed'):self.load()

    def test_signature_validation_failure_cannot_launch_a_product(self):
        self.prepare()
        with patch.object(mac,'verify_release_updates',side_effect=ValueError('signature_mismatch')):
            with self.assertRaisesRegex(ValueError,'signature_mismatch'):mac.load_production_prepared(self.output)
        self.assertTrue(all(call[0] in ('git','ditto') for call in self.calls))

    def test_prepare_and_execute_reject_inherited_production_secrets(self):
        self.prepare()
        for name in mac.PRIVATE_KEY_NAMES:
            with self.subTest(name=name),patch.dict(os.environ,{name:'fixture-value-never-used'}):
                with self.assertRaisesRegex(RuntimeError,'without signing keys'):mac.production_prepared(self.output,self.products,self.signer)
                with self.assertRaisesRegex(RuntimeError,'without signing keys'):mac.load_production_prepared(self.output)

    def test_child_environment_always_removes_signing_authority(self):
        with patch.dict(os.environ,{name:'fixture-value-never-used' for name in mac.PRIVATE_KEY_NAMES}):
            clean=mac.environment()
        self.assertFalse(any(name in clean for name in mac.PRIVATE_KEY_NAMES))

    def archived_source(self,arguments,*_,**__):
        if arguments[0]=='git' and 'archive' in arguments:
            with tarfile.open(arguments[-2],'w') as archive:
                for name in mac.PUBLIC_RESOURCES:
                    archive.add(self.root/'resources'/name,arcname='resources/'+name)
        return b''

    def test_production_source_keeps_both_committed_trust_files(self):
        with patch.object(mac,'run',side_effect=self.archived_source):
            source=mac.private_source(self.output,production_trust=True)
        for name in mac.PUBLIC_RESOURCES:
            self.assertEqual((source/'resources'/name).read_bytes(),(self.root/'resources'/name).read_bytes())

    def test_default_source_still_uses_disposable_fixture_trust(self):
        with patch.object(mac,'run',side_effect=self.archived_source):
            source=mac.private_source(self.output)
        self.assertEqual(json.loads((source/'resources/update-trust.json').read_bytes()),mac.KEYRING)
        self.assertEqual(json.loads((source/'resources/sparkle-update-key.json').read_bytes())['public_key'],mac.PUBLIC)


if __name__=='__main__':unittest.main()
