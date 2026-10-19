#!/usr/bin/env python3
"""Source patch replay, resumed tree integrity and actual linker-map boundaries."""
import hashlib
import importlib.util
import io
import json
import shutil
import subprocess
from pathlib import Path
import tarfile
import tempfile
import struct
from types import SimpleNamespace
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[1]
def load(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/'scripts'/(name+'.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
source=load('build-qt-source');linkage=load('linkage-evidence');binary_diff=load('classify-qt-binary-differences')

class SourceTests(unittest.TestCase):
    def test_recorded_source_patches_and_tool_pins(self):
        data=json.loads((ROOT/'scripts/corresponding-sources.json').read_text())
        self.assertEqual({i['module'] for i in data['archives']+data['build_tools']},set(source.ORDER))
        for item in data['source_build_patches']:
            self.assertEqual(source.digest(ROOT/item['patch']),item['patch_sha256'])
        self.assertEqual(data['rust_stdlib']['toolchain'],'1.99.0')

    @unittest.skipUnless(shutil.which('cmake'), 'CMake unavailable')
    def test_optional_clang_discovery_cannot_visit_broken_runner_package(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);modules=root/'modules';modules.mkdir();poison=root/'broken';poison.mkdir()
            (root/'CMakeLists.txt').write_text('cmake_minimum_required(VERSION 3.21)\nproject(ClangIsolation NONE)\nlist(PREPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/modules")\nfind_package(WrapLibClang QUIET)\n')
            (modules/'FindWrapLibClang.cmake').write_text('find_package(Clang CONFIG)\n')
            (poison/'ClangConfig.cmake').write_text('message(FATAL_ERROR "broken runner Clang package visited")\n')
            command=['cmake','-S',str(root),'-DCMAKE_PREFIX_PATH='+str(poison)]
            negative=subprocess.run(command+['-B',str(root/'negative')]+[f for f in source.TOOLS if 'CMAKE_DISABLE_FIND_PACKAGE' not in f],capture_output=True,text=True)
            self.assertNotEqual(negative.returncode,0)
            self.assertIn('broken runner Clang',negative.stderr)
            positive=subprocess.run(command+['-B',str(root/'positive')]+source.TOOLS,capture_output=True,text=True)
            self.assertEqual(positive.returncode,0,positive.stderr)

    def test_resumed_modified_source_is_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);archive=root/'owned.tar.xz'
            with tarfile.open(archive,'w:xz') as output:
                member=tarfile.TarInfo('owned/a.txt');member.size=5;output.addfile(member,io.BytesIO(b'owned'))
            destination=root/'sources';destination.mkdir()
            config={'archives':[{'module':'owned','file':'owned.tar.xz'}],'build_tools':[],'source_build_patches':[],'sdk_patches':[]}
            with patch.object(source,'ORDER',['owned']),patch.object(source,'load_delivery',return_value=SimpleNamespace(archive_source=lambda *_:archive)):
                source.source_inputs(config,root,destination)
                (destination/'owned/extra.cpp').touch()
                with self.assertRaisesRegex(ValueError,'extra'):source.source_inputs(config,root,destination)
                (destination/'owned/extra.cpp').unlink()
                (destination/'owned/a.txt').write_bytes(b'tampered')
                with self.assertRaisesRegex(ValueError,'modified'):source.source_inputs(config,root,destination)

    def test_linker_map_ignores_dead_stripped_symbols_and_private_paths(self):
        text='# Object files:\n[ 1] /Users/owned/private/libowned.a(unit.o)\n[ 2] /Users/owned/private/dead.o\n# Symbols:\n# Address\tSize\tFile Name\n0x1000 0x10 [ 1] _live\n# Dead Stripped Symbols:\n<<dead>> 0x20 [ 2] _dead\n'
        result=linkage.mac_objects(text)
        self.assertEqual(result,[{'object':'libowned.a(unit.o)','retained_symbol_bytes':16}])
        self.assertNotIn('/Users/',json.dumps(result))

    def test_linux_discarded_and_windows_import_map(self):
        linux='Discarded input sections\n.text 0x0 0x20 dead.o\nLinker script and memory map\n.text 0x1000 0x10 /home/private/libowned.a(unit.o)\n'
        self.assertEqual(linkage.other_objects(linux),[{'object':'libowned.a(unit.o)','retained_section_bytes':16}])
        windows=' 0001:00000000 _owned 0000000140001000 f owned:unit.rcgu.o\n'
        self.assertEqual(linkage.other_objects(windows),[{'object':'owned:unit.rcgu.o','retained_public_symbols':1}])

    def test_pe_transform_requires_exact_bounded_certificate_tail(self):
        unsigned=bytearray(512);unsigned[:2]=b'MZ';struct.pack_into('<I',unsigned,0x3c,128);unsigned[128:132]=b'PE\0\0';optional=152
        struct.pack_into('<H',unsigned,optional,0x20b)
        signed=bytearray(unsigned)+bytearray(b'owned-certificate')
        struct.pack_into('<II',signed,optional+144,512,17);struct.pack_into('<I',signed,optional+64,123)
        candidates=list(binary_diff.pe_unsigned_candidates(signed))
        self.assertTrue(any(value==unsigned for _,value in candidates))
        signed.append(1)
        self.assertEqual(list(binary_diff.pe_unsigned_candidates(signed)),[])
        self.assertEqual(list(binary_diff.pe_unsigned_candidates(b'untrusted')),[])

if __name__=='__main__':unittest.main()
