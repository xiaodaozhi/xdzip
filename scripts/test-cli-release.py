#!/usr/bin/env python3
"""Verify independent CLI assets/signatures without weakening desktop metadata."""
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from release_protocol import CLI_TARGETS, TARGETS, create, create_cli, validate, validate_cli
from release_updates import run_tool, sign_manifest, verify_cli_updates

ROOT = Path(__file__).resolve().parents[1]


class CliReleaseTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        subprocess.run(['cargo','build','--locked','-p','xdzip-update','--bin','xdzip-update-tool'],cwd=ROOT,check=True)
        cls.tool = Path(os.environ.get('CARGO_TARGET_DIR',ROOT/'target')).resolve()/'debug'/('xdzip-update-tool.exe' if os.name=='nt' else 'xdzip-update-tool')

    def setUp(self):
        base = ROOT/'build/test-tmp';base.mkdir(parents=True,exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix='cli-release-',dir=base)
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)
        self.version = dict(display_version='0.2.0',channel='stable',build=27)
        for suffix,_ in TARGETS.values():
            (self.directory/('xdzip-0.2.0-'+suffix)).write_bytes(b'owned desktop asset')
        for suffix,_ in CLI_TARGETS.values():
            (self.directory/('xdzip-cli-0.2.0-'+suffix)).write_bytes(b'owned CLI asset')
        self.desktop = create(self.directory,self.version,'xiaodaozhi/xdzip','a'*40)
        self.cli = create_cli(self.directory,self.version,'xiaodaozhi/xdzip','a'*40)
        (self.directory/'cli-release.json').write_text(json.dumps(self.cli)+'\n')

    def test_separate_complete_targets_and_minimum_systems(self):
        self.assertEqual(len(self.desktop['assets']),6)
        self.assertEqual(len(self.cli['assets']),3)
        self.assertNotIn('component',self.desktop)
        self.assertEqual(validate(self.desktop,self.directory),self.desktop)
        self.assertEqual(validate_cli(self.cli,self.directory),self.cli)
        linux = next(a for a in self.cli['assets'] if a['platform']=='linux')
        self.assertNotIn('display_server',linux['minimum_system'])
        for field,value in [('component','desktop'),('repository','other/repository'),('commit','bad')]:
            changed=copy.deepcopy(self.cli);changed[field]=value
            with self.subTest(field=field), self.assertRaises(ValueError):
                validate_cli(changed,self.directory)
        changed=copy.deepcopy(self.cli);changed['assets'].pop()
        with self.assertRaises(ValueError):validate_cli(changed,self.directory)
        changed=copy.deepcopy(self.cli);changed['assets'][0]['url']='https://example.invalid/payload'
        with self.assertRaises(ValueError):validate_cli(changed,self.directory)

    def test_publisher_signature_exact_bytes_and_same_candidate_publication_gate(self):
        keys=self.directory/'private-test-public-key.json'
        # A fixed private fixture seed is supplied only to already-built signer
        # subprocesses; it is not a product key or a workflow secret.
        with patch.dict(os.environ,{'XDZIP_UPDATE_SIGNING_KEY_HEX':'73'*32}):
            run_tool(['public-key','--key-id','cli-private-fixture','--output',str(keys)],self.tool)
            sign_manifest(self.directory,key_id='cli-private-fixture',required=True,tool=self.tool,keyring=keys,component='cli')
        verify_cli_updates(self.directory,self.desktop,tool=self.tool,keyring=keys)
        changed=copy.deepcopy(self.desktop);changed['build']+=1
        with self.assertRaises(ValueError):verify_cli_updates(self.directory,changed,tool=self.tool,keyring=keys)
        asset=self.directory/self.cli['assets'][0]['filename'];asset.write_bytes(b'changed')
        with self.assertRaises(ValueError):verify_cli_updates(self.directory,self.desktop,tool=self.tool,keyring=keys)
        asset.write_bytes(b'owned CLI asset')
        manifest=self.directory/'cli-release.json';manifest.write_bytes(manifest.read_bytes()+b' ')
        with self.assertRaises(ValueError):verify_cli_updates(self.directory,self.desktop,tool=self.tool,keyring=keys)


if __name__=='__main__':
    unittest.main()
