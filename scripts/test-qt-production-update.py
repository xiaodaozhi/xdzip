"""Verify private Qt upgrade admission with real signatures and inert packages.

Both target contracts run on every host. These checks use the public RFC 8032
fixture key only, never an installed product, production key or native installer.
"""
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import qt_update_fixture as fixture
import update_fixture as shared

ROOT=Path(__file__).resolve().parents[1]
PRIVATE=('XDZIP_UPDATE_SIGNING_KEY_HEX','XDZIP_SPARKLE_SIGNING_KEY_HEX')


class ProductionQtAdmissionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if any(name in os.environ for name in PRIVATE):
            raise RuntimeError('Contract checks must not receive production signing authority')
        cls.tool=ROOT/'target/debug'/('xdzip-update-tool.exe' if os.name=='nt' else 'xdzip-update-tool')
        subprocess.run(['cargo','build','--locked','--quiet','-p','xdzip-update','--bin','xdzip-update-tool'],
                       cwd=ROOT,check=True)

    def setUp(self):
        (ROOT/'build').mkdir(exist_ok=True)
        temporary=tempfile.TemporaryDirectory(prefix='qt-production-contract-',dir=ROOT/'build')
        self.addCleanup(temporary.cleanup)
        self.root=Path(temporary.name).resolve()
        (self.root/'resources').mkdir()
        shared.dump(self.root/'resources/update-trust.json',shared.KEYRING)
        shared.dump(self.root/'resources/sparkle-update-key.json',{'schema_version':1,'public_key':shared.PUBLIC})
        root_patch=patch.object(fixture,'ROOT',self.root)
        root_patch.start();self.addCleanup(root_patch.stop)
        env_patch=patch.dict(os.environ,{'GITHUB_SHA':'c'*40})
        env_patch.start();self.addCleanup(env_patch.stop)

    def prepare(self,platform):
        output=self.root/platform;output.mkdir()
        (output/'resources').mkdir();(output/'signing').mkdir()
        for name in ('update-trust.json','sparkle-update-key.json'):
            shutil.copy2(self.root/'resources'/name,output/'resources'/name)
        tool=output/'signing'/('xdzip-update-tool.exe' if platform=='windows' else 'xdzip-update-tool')
        shutil.copy2(self.tool,tool)
        prepared={'production_trust':True,'source_commit':'c'*40}
        products={};frozen=[tool,*list((output/'resources').glob('*.json'))]
        for label in ('old','new','failure'):
            directory=output/label;directory.mkdir()
            number=1 if label=='old' else 2
            commit=('a' if label=='old' else 'b')*40
            identity={'version':'0.2.0-alpha.'+str(number),'build':199+number,'revision':commit[:12],'dirty':False}
            products[label]={'identity':identity,'commit':commit}
            suffix='windows-x86_64-setup.exe' if platform=='windows' else 'ubuntu24.04-x86_64.deb'
            package=directory/('xdzip-'+identity['version']+'-'+suffix)
            package.write_bytes(('Owned inert '+label).encode())
            prepared[label]={'package':package.name,'identity':{**identity,'commit':commit}}
            frozen.append(package)
            if label!='old':
                shared.manifest(package,platform,tool,{**identity,'commit':commit})
                frozen.append(directory/'release.json')
        prepared['production_products']=products
        record={'schema_version':1,'platform':platform,'source_commit':'c'*40,
                'tool':{'path':tool.relative_to(output).as_posix(),'sha256':shared.digest(tool)},
                'artifacts':{p.relative_to(output).as_posix():{'bytes':p.stat().st_size,'sha256':shared.digest(p)} for p in frozen},
                'products':products}
        shared.dump(output/'production-update-prepared.json',record)
        return output,prepared,record

    def check(self,platform,output,prepared):
        with patch.object(fixture,'sys',SimpleNamespace(platform='win32' if platform=='windows' else 'linux')):
            return fixture.production_trust_check(output,prepared)

    def test_both_platforms_accept_real_fixture_signatures(self):
        for platform in ('windows','linux'):
            with self.subTest(platform=platform):
                output,prepared,_=self.prepare(platform)
                result=self.check(platform,output,prepared)
                self.assertEqual(result['status'],'passed')
                self.assertFalse(result['signing_secrets_received'])
                self.assertEqual(len(result['signatures']),2)

    def test_signing_authority_is_rejected_before_any_child(self):
        for name in PRIVATE:
            with self.subTest(name=name),patch.dict(os.environ,{name:'inert-test-marker'}),patch.object(fixture,'run') as child:
                with self.assertRaisesRegex(RuntimeError,'must not receive'):
                    fixture.production_trust_check(self.root,{})
                child.assert_not_called()

    def test_wrong_workflow_commit_is_rejected(self):
        for platform in ('windows','linux'):
            with self.subTest(platform=platform):
                output,prepared,_=self.prepare(platform)
                with patch.dict(os.environ,{'GITHUB_SHA':'d'*40}),patch.object(fixture,'run') as child:
                    with self.assertRaisesRegex(RuntimeError,'source or mode'):
                        self.check(platform,output,prepared)
                    child.assert_not_called()

    def test_changed_installer_is_rejected_before_verification(self):
        for platform in ('windows','linux'):
            with self.subTest(platform=platform):
                output,prepared,_=self.prepare(platform)
                (output/'new'/prepared['new']['package']).write_bytes(b'changed installer')
                with patch.object(fixture,'run') as child:
                    with self.assertRaisesRegex(RuntimeError,'input changed'):
                        self.check(platform,output,prepared)
                    child.assert_not_called()

    def test_wrong_signature_is_rejected_by_real_verifier(self):
        for platform in ('windows','linux'):
            with self.subTest(platform=platform):
                output,prepared,_=self.prepare(platform)
                signature=output/'new/release.json.sig'
                value=json.loads(signature.read_bytes());value['signature']='00'*64;shared.dump(signature,value)
                with self.assertRaises(subprocess.CalledProcessError):
                    self.check(platform,output,prepared)
                self.assertFalse((output/'production-trust-verification.json').exists())

    def test_changed_public_root_is_rejected(self):
        for platform in ('windows','linux'):
            with self.subTest(platform=platform):
                output,prepared,record=self.prepare(platform)
                path=output/'resources/update-trust.json';path.write_bytes(b'{}')
                # Updating only the unsigned inventory cannot introduce trust.
                record['artifacts']['resources/update-trust.json']={'bytes':path.stat().st_size,'sha256':shared.digest(path)}
                shared.dump(output/'production-update-prepared.json',record)
                with self.assertRaisesRegex(RuntimeError,'checked-out trust'):
                    self.check(platform,output,prepared)

    def test_missing_frozen_input_is_rejected(self):
        for platform in ('windows','linux'):
            with self.subTest(platform=platform):
                output,prepared,record=self.prepare(platform)
                record=copy.deepcopy(record);record['artifacts'].pop('new/release.json')
                shared.dump(output/'production-update-prepared.json',record)
                with self.assertRaisesRegex(RuntimeError,'every required input'):
                    self.check(platform,output,prepared)

    def test_unsigned_manifests_do_not_invoke_signer(self):
        for platform in ('windows','linux'):
            with self.subTest(platform=platform):
                directory=self.root/platform;directory.mkdir()
                package=directory/'owned-inert-package';package.write_bytes(b'not executable')
                with patch.object(shared,'run') as child:
                    shared.manifest(package,platform,self.tool,{**shared.NEW,'commit':'b'*40},sign=False)
                child.assert_not_called()
                self.assertTrue((directory/'release.json').is_file())
                self.assertFalse((directory/'release.json.sig').exists())
                self.assertFalse((directory/'test-keyring.json').exists())


if __name__=='__main__':unittest.main()
