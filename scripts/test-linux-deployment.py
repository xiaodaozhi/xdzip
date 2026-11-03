#!/usr/bin/env python3
"""Boundary tests for Linux runtime planning and isolated product event evidence."""
import json
import subprocess
import tarfile
import sys
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from deploy_linux import qml_roots, relative_rpath, runtime_origin, tls_requirements, TLS_PLUGIN
from verify_linux_runtime import check_events, check_https, run_https_probe, main as verify_runtime_main
from package_linux import archive_payload


def https_success():
    return {'ssl_supported':True,'ssl_backend':'openssl','state':'up_to_date','error':'',
            'http_diagnostics_schema':1,'timeout':False,'authorization_present':False,
            'response_records_truncated':False,'encrypted_connections':1,'https_responses':1,
            'request_count':1,'http_responses':[{'host':'api.github.com','route':'release_list','status':200}]}


def https_limited():
    return {'ssl_supported':True,'ssl_backend':'openssl','state':'error','error':'rate_limited',
            'http_diagnostics_schema':1,'timeout':False,'authorization_present':False,
            'response_records_truncated':False,'confirmed_from_controller':True,
            'rate_limit_confirmed':True,'request_count':1,
            'http_responses':[{'status':429,'host':'api.github.com','route':'release_list',
                               'server_date_epoch':1000,'rate_headers':{'remaining':None,'reset':None,'retry_after':None}}]}


def probe_output(record):
    runtime={'qt_openssl_backend_present':True,'openssl_package_installed':True,
             'system_ca_bundle_nonempty':True,'ca_package_installed':True}
    return 'XDZIP_HTTPS_RUNTIME '+json.dumps(runtime)+'\nXDZIP_UPDATE_HTTPS '+json.dumps(record)+'\n'


class LinuxDeploymentTests(unittest.TestCase):
    def test_runtime_refuses_to_move_directory_at_evidence_path(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);output=root/'runtime.json';output.write_text('previous report')
            directory=output.with_suffix('.https.log');directory.mkdir()
            args=['verify_linux_runtime.py','--root',str(root),'--fixture',str(root/'fixture.zip'),
                  '--output',str(output)]
            with patch.object(sys,'argv',args),patch('verify_linux_runtime.subprocess.run') as run:
                with self.assertRaisesRegex(ValueError,'not a file'):verify_runtime_main()
            self.assertEqual(output.read_text(),'previous report')
            self.assertTrue(directory.is_dir());run.assert_not_called()

    def test_runtime_failed_rerun_archives_previous_success_before_build(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);payload=root/'payload';(payload/'share').mkdir(parents=True)
            (payload/'share/deployment.json').write_text(json.dumps({'system_packages':['libssl3t64']}))
            output=root/'review/linux-runtime.json';output.parent.mkdir()
            previous={output:'old passed report',output.with_suffix('.https.json'):'old HTTPS success',
                      output.with_suffix('.https.attempt-1.log'):'old attempt'}
            for path,value in previous.items():path.write_text(value)
            args=['verify_linux_runtime.py','--root',str(payload),'--fixture',str(root/'fixture.zip'),
                  '--output',str(output)]
            failed=subprocess.CompletedProcess([],1,stdout='',stderr='owned build failure')
            with patch.object(sys,'argv',args),patch('verify_linux_runtime.subprocess.run',return_value=failed):
                with self.assertRaisesRegex(SystemExit,'image build failed'):verify_runtime_main()
            self.assertFalse(output.exists())
            self.assertFalse(output.with_suffix('.https.json').exists())
            history=list((output.parent/(output.stem+'.history')).iterdir())
            self.assertEqual(len(history),1)
            for path,value in previous.items():self.assertEqual((history[0]/path.name).read_text(),value)
            self.assertIn('owned build failure',output.with_suffix('.build.log').read_text())

    def test_https_failed_rerun_archives_prior_success_and_attempts(self):
        with tempfile.TemporaryDirectory() as temp:
            output=Path(temp)/'runtime.json'
            success=subprocess.CompletedProcess([],0,stdout=probe_output(https_success()),stderr='')
            failed=https_limited();failed['error']='network'
            failure=subprocess.CompletedProcess([],13,stdout=probe_output(failed),stderr='new failure')
            with patch('verify_linux_runtime.subprocess.run',side_effect=[success,failure]):
                run_https_probe(['owned'],output)
                with self.assertRaisesRegex(SystemExit,'not_confirmed_rate_limit'):
                    run_https_probe(['owned'],output)
            self.assertFalse(output.with_suffix('.https.json').exists())
            self.assertIn('new failure',output.with_suffix('.https.attempt-1.log').read_text())
            history=list((output.parent/(output.stem+'.history')).iterdir())
            self.assertEqual(len(history),1)
            self.assertEqual(json.loads((history[0]/output.with_suffix('.https.json').name).read_text()),
                             https_success())
            self.assertIn('up_to_date',(history[0]/output.with_suffix('.https.attempt-1.log').name).read_text())

    def test_runtime_keeps_https_failure_log_and_exit_code(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);payload=root/'payload';(payload/'share').mkdir(parents=True)
            (payload/'share/deployment.json').write_text(json.dumps({'system_packages':['libssl3t64']}))
            fixture=root/'owned.zip';fixture.touch();output=root/'review/linux-runtime.json'
            events=[{'event':'first_frame','runtime_available':True},
                    {'event':'open_ready','records':100000},
                    {'event':'cancel_requested','pending':True},
                    {'event':'cancel_settled','cancelled':True}, {'event':'quit_requested'}]
            runtime='\n'.join('XDZIP_PROBE '+json.dumps(event) for event in events)
            report={'ssl_supported':True,'ssl_backend':'openssl','state':'error','error':'network'}
            https='XDZIP_UPDATE_HTTPS '+json.dumps(report)+'\n'
            completed=[subprocess.CompletedProcess([],0,stdout='built',stderr=''),
                       subprocess.CompletedProcess([],0,stdout=runtime,stderr=''),
                       subprocess.CompletedProcess([],13,stdout=https,stderr='probe failed (exit 13)\n')]
            args=['verify_linux_runtime.py','--root',str(payload),'--fixture',str(fixture),'--output',str(output)]
            with patch.object(sys,'argv',args),patch('verify_linux_runtime.subprocess.run',side_effect=completed) as run:
                with self.assertRaisesRegex(SystemExit,'exit 13'):verify_runtime_main()
            self.assertEqual(output.with_suffix('.https.log').read_text(),https+'probe failed (exit 13)\n')
            self.assertFalse(output.exists())
            self.assertFalse(output.with_suffix('.https.json').exists())
            self.assertTrue(any('verify-https-probe.sh:/verify-https-probe.sh:ro' in arg
                                for arg in run.call_args_list[2].args[0]))

    @unittest.skipIf(sys.platform=='win32','POSIX shell probe runs on Unix')
    def test_https_probe_preserves_failures_and_never_accepts_missing_report(self):
        helper=Path(__file__).resolve().parents[1]/'apps/desktop/packaging/linux/verify-https-probe.sh'
        self.assertTrue(helper.is_file())
        good=https_success()
        failed={'ssl_supported':True,'ssl_backend':'openssl','state':'error','error':'network'}
        cases=[('success',good,0,0),('failed-with-report',failed,13,13),
               ('failed-without-report',None,13,13),('success-without-report',None,0,1),
               ('malformed-report','not-json',0,1),('malformed-failed-report','not-json',13,13)]
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)
            for name,report,probe_exit,expected_exit in cases:
                with self.subTest(name=name):
                    fake=root/(name+'.sh');target=root/(name+'.json')
                    script='#!/bin/sh\ntest "$1" = "--update-probe" || exit 91\n'
                    if report is not None:
                        content=json.dumps(report) if isinstance(report,dict) else report
                        script+="cat > \"$2\" <<'OWNED_REPORT'\n"+content+"\nOWNED_REPORT\n"
                    fake.write_text(script+f'exit {probe_exit}\n');fake.chmod(0o755)
                    result=subprocess.run(['sh','-c','. "$1"; run_https_probe "$2" "$3"',
                                           'https-test',str(helper),str(fake),str(target)],
                                          capture_output=True,text=True,check=False)
                    self.assertEqual(result.returncode,expected_exit, result.stdout+result.stderr)
                    if isinstance(report,dict):
                        self.assertIn('XDZIP_UPDATE_HTTPS ',result.stdout)
                        actual=json.loads(result.stdout.split('XDZIP_UPDATE_HTTPS ',1)[1])
                        self.assertEqual(actual,report)
                    else:
                        self.assertNotIn('XDZIP_UPDATE_HTTPS ',result.stdout)
                    if report is None:
                        self.assertIn('did not write a report',result.stderr)
                    if name in ('failed-with-report','malformed-failed-report'):
                        self.assertIn('exit 13',result.stderr)
                    if name=='failed-with-report':
                        with self.assertRaisesRegex(ValueError,'GitHub'):check_https(result.stdout)

    def test_https_requires_backend_system_ssl_and_nonempty_ca_store(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);sdk=root/'sdk'
            plugin=sdk/TLS_PLUGIN;plugin.parent.mkdir(parents=True);plugin.touch()
            cert=root/'etc/ssl/certs/ca-certificates.crt';cert.parent.mkdir(parents=True);cert.write_text('owned CA fixture')
            library=root/'usr/lib/x86_64-linux-gnu';library.mkdir(parents=True)
            for name in ('libssl.so.3','libcrypto.so.3'):(library/name).touch()
            with patch('deploy_linux.command',return_value='install ok installed\t3.0.13-1\topenssl\t3.0.13-1'),patch('deploy_linux.system_package',return_value='libssl3t64:amd64'):
                selected,records=tls_requirements(sdk,root)
                self.assertEqual(selected,plugin)
                self.assertEqual({p['package'] for p in records},{'libssl3t64','ca-certificates'})
                self.assertEqual(records[0]['source_package'],'openssl')
                cert.write_text('')
                with self.assertRaisesRegex(ValueError,'trust store'):tls_requirements(sdk,root)
            plugin.unlink()
            with self.assertRaisesRegex(ValueError,'backend missing'):tls_requirements(sdk,root)

    def test_sdk_free_https_evidence_requires_real_success_and_openssl(self):
        value=https_success()
        def output():return 'XDZIP_UPDATE_HTTPS '+json.dumps(value)
        self.assertEqual(check_https(output()),value)
        value['ssl_backend']=''
        with self.assertRaisesRegex(ValueError,'OpenSSL'):check_https(output())
        value['ssl_backend']='openssl';value['state']='error';value['error']='network'
        with self.assertRaisesRegex(ValueError,'GitHub'):check_https(output())
        for key,invalid in [('timeout',True),('authorization_present',True),('encrypted_connections',0),
                            ('https_responses',0),('request_count',0),('http_diagnostics_schema',True)]:
            with self.subTest(key=key):
                value=https_success();value[key]=invalid
                with self.assertRaisesRegex(ValueError,'TLS evidence'):check_https(output())

    def test_sdk_free_https_retries_only_confirmed_anonymous_limits_and_preserves_attempts(self):
        class Clock:
            now=1000
            def monotonic(self):return self.now
            def sleep(self,seconds):self.now+=seconds
        with tempfile.TemporaryDirectory() as temp:
            output=Path(temp)/'runtime.json';clock=Clock()
            first=https_limited();second=https_success()
            calls=[subprocess.CompletedProcess([],13,stdout=probe_output(first),stderr='limit\n'),
                   subprocess.CompletedProcess([],0,stdout=probe_output(second),stderr='')]
            with patch('verify_linux_runtime.subprocess.run',side_effect=calls) as run:
                actual,count,runtime=run_https_probe(['owned'],output,sleep=clock.sleep,monotonic=clock.monotonic,
                                             wall_clock=lambda:1000)
            self.assertEqual((actual,count),(second,2));self.assertEqual(run.call_count,2)
            self.assertTrue(runtime['system_ca_bundle_nonempty'])
            self.assertEqual(json.loads(output.with_suffix('.https.attempt-1.json').read_text()),first)
            self.assertIn('limit',output.with_suffix('.https.attempt-1.log').read_text())
            self.assertEqual(json.loads(output.with_suffix('.https.json').read_text()),second)
            wait=json.loads(output.with_suffix('.https-waits.json').read_text())[0]
            self.assertEqual((wait['delay_seconds'],wait['wait_status']),(60,'finished'))

    def test_sdk_free_https_requires_separate_runtime_capability_record(self):
        for record,status in [(https_success(),0),(https_limited(),13)]:
            with self.subTest(status=status),tempfile.TemporaryDirectory() as temp:
                output=Path(temp)/'runtime.json'
                result=subprocess.CompletedProcess([],status,stdout='XDZIP_UPDATE_HTTPS '+json.dumps(record)+'\n',stderr='')
                with patch('verify_linux_runtime.subprocess.run',return_value=result) as run, \
                     patch('verify_linux_runtime.time.sleep') as sleep:
                    with self.assertRaises(SystemExit):run_https_probe(['owned'],output)
                self.assertEqual(run.call_count,1);sleep.assert_not_called()
                self.assertFalse(output.with_suffix('.https.json').exists())

    def test_sdk_free_https_does_not_retry_network_tls_or_unproven_limits(self):
        variants=[]
        for error in ('network','tls','timeout'):
            value=https_limited();value['error']=error;variants.append(value)
        for key,value in [('authorization_present',True),('timeout',True),
                          ('response_records_truncated',True),('confirmed_from_controller',False),
                          ('ssl_backend',''),('http_diagnostics_schema',True)]:
            record=https_limited();record[key]=value;variants.append(record)
        record=https_limited();record['http_responses'][0]['route']='other';variants.append(record)
        record=https_limited();record['http_responses'][0]['rate_headers']['remaining']='0';variants.append(record)
        for record in variants:
            with self.subTest(record=record),tempfile.TemporaryDirectory() as temp:
                output=Path(temp)/'runtime.json'
                result=subprocess.CompletedProcess([],13,stdout=probe_output(record),stderr='failure\n')
                with patch('verify_linux_runtime.subprocess.run',return_value=result) as run, \
                     patch('verify_linux_runtime.time.sleep') as sleep:
                    with self.assertRaises(SystemExit):run_https_probe(['owned'],output)
                self.assertEqual(run.call_count,1);sleep.assert_not_called()
                self.assertTrue(output.with_suffix('.https.attempt-1.log').exists())
                self.assertFalse(output.with_suffix('.https.json').exists())

    def test_sdk_free_https_respects_wait_budget_and_short_wait(self):
        with tempfile.TemporaryDirectory() as temp:
            output=Path(temp)/'runtime.json';record=https_limited()
            record['http_responses'][0]['rate_headers'].update(remaining='0',reset='2000')
            result=subprocess.CompletedProcess([],13,stdout=probe_output(record),stderr='')
            with patch('verify_linux_runtime.subprocess.run',return_value=result) as run:
                with self.assertRaisesRegex(SystemExit,'wait_budget_exceeded'):
                    run_https_probe(['owned'],output,sleep=lambda _:None,wall_clock=lambda:1000)
            self.assertEqual(run.call_count,1)
            record=https_limited();result.stdout=probe_output(record)
            with patch('verify_linux_runtime.subprocess.run',return_value=result) as run:
                with self.assertRaisesRegex(SystemExit,'wait ended early'):
                    run_https_probe(['owned'],output,sleep=lambda _:None,monotonic=lambda:1,wall_clock=lambda:1000)
            self.assertEqual(run.call_count,1)

    def test_sdk_free_https_fails_after_bounded_retries_and_retains_timeouts(self):
        class Clock:
            now=0
            def monotonic(self):return self.now
            def sleep(self,seconds):self.now+=seconds
        with tempfile.TemporaryDirectory() as temp:
            output=Path(temp)/'runtime.json';clock=Clock();record=https_limited()
            result=subprocess.CompletedProcess([],13,stdout=probe_output(record),stderr='')
            with patch('verify_linux_runtime.subprocess.run',return_value=result) as run:
                with self.assertRaisesRegex(SystemExit,'attempt_limit_reached'):
                    run_https_probe(['owned'],output,sleep=clock.sleep,monotonic=clock.monotonic,
                                    wall_clock=lambda:1000)
            self.assertEqual(run.call_count,3);self.assertEqual(clock.now,180)
            self.assertTrue(output.with_suffix('.https.attempt-3.log').exists())
            self.assertFalse(output.with_suffix('.https.json').exists())
        with tempfile.TemporaryDirectory() as temp:
            output=Path(temp)/'runtime.json'
            failure=subprocess.TimeoutExpired(['owned'],80,output=b'partial output',stderr=b'partial error')
            with patch('verify_linux_runtime.subprocess.run',side_effect=failure) as run:
                with self.assertRaisesRegex(SystemExit,'probe_process_failure'):
                    run_https_probe(['owned'],output)
            self.assertEqual(run.call_count,1)
            log=output.with_suffix('.https.attempt-1.log').read_text()
            self.assertIn('partial output',log);self.assertIn('partial error',log)
            self.assertIn('timed out',log)

    def test_only_sdk_modules_and_explicit_static_or_optional_modules(self):
        with tempfile.TemporaryDirectory() as temp:
            sdk=Path(temp)
            (sdk/'qml/QtQuick/Controls').mkdir(parents=True);(sdk/'qml/QtQml').mkdir()
            entries=[{'name':'QtQuick','type':'module','path':str(sdk/'qml/QtQuick')},
                     {'name':'QtQuick.Controls','type':'module','path':str(sdk/'qml/QtQuick/Controls')},
                     {'name':'QtQml','type':'module','path':str(sdk/'qml/QtQml')},
                     {'name':'Xdzip.Settings','type':'module'},
                     {'name':'QtQuick.Controls.macOS','type':'module'}]
            self.assertEqual(qml_roots(entries,sdk),(['QtQml','QtQuick'],['QtQuick.Controls.macOS']))
            entries.append({'name':'Unknown.Required','type':'module'})
            with self.assertRaisesRegex(ValueError,'Unresolved QML'):qml_roots(entries,sdk)
            entries[-1]={'name':'Unknown.Required','type':'module','path':str(sdk/'outside')}
            with self.assertRaisesRegex(ValueError,'escapes'):qml_roots(entries,sdk)

    def test_copied_icu_sibling_is_an_internal_dependency(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);sdk=base/'sdk';payload=base/'app'
            (sdk/'lib').mkdir(parents=True);(payload/'lib').mkdir(parents=True)
            copied=payload/'lib/libicudata.so.73';copied.touch()
            self.assertEqual(runtime_origin(copied,sdk,payload),'artifact')
            self.assertEqual(runtime_origin(sdk/'lib/libicudata.so.73',sdk,payload),'sdk')
            with self.assertRaisesRegex(ValueError,'Unexpected external'):
                runtime_origin(base/'unknown/libicudata.so.73',sdk,payload)

    def test_nested_plugin_origin_paths_remain_inside_relocatable_root(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)
            for relative in ['bin/xdzip','lib/libQt6Core.so.6','qml/QtQuick/Layouts/libplugin.so','plugins/platforms/libqxcb.so']:
                binary=root/relative;binary.parent.mkdir(parents=True,exist_ok=True);binary.touch()
                rpath=relative_rpath(binary,root)
                self.assertTrue(rpath.startswith('$ORIGIN/'))
                target=binary.parent/rpath.removeprefix('$ORIGIN/')
                self.assertEqual(target.resolve(),(root/'lib').resolve())
            with self.assertRaisesRegex(ValueError,'outside'):relative_rpath(root.parent/'foreign',root)

    def test_runtime_evidence_requires_real_open_and_interrupted_cancel(self):
        events=[{'event':'first_frame','runtime_available':True}, {'event':'open_ready','records':100000},
                {'event':'cancel_requested','pending':True}, {'event':'cancel_settled','cancelled':True},
                {'event':'quit_requested'}]
        def output():return '\n'.join('XDZIP_PROBE '+json.dumps(e) for e in events)
        self.assertEqual(check_events(output()),events)
        events[3]['cancelled']=False
        with self.assertRaisesRegex(ValueError,'cancellation'):check_events(output())
        events[3]['cancelled']=True;events[1]['records']=1
        with self.assertRaisesRegex(ValueError,'owned fixture'):check_events(output())
        events.pop()
        with self.assertRaisesRegex(ValueError,'finish all stages'):check_events(output())

    @unittest.skipIf(sys.platform=='win32','POSIX mode bits require a Unix filesystem')
    def test_archive_preserves_entrypoint_and_has_stable_metadata(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root=base/'payload';(root/'bin').mkdir(parents=True)
            (root/'bin/xdzip').write_bytes(b'owned fixture');(root/'bin/xdzip').chmod(0o755)
            (root/'notice').write_text('license')
            first=archive_payload(root,base/'one.tar.gz');second=archive_payload(root,base/'two.tar.gz')
            self.assertEqual(first['sha256'],second['sha256'])
            with tarfile.open(base/'one.tar.gz') as archive:
                app=archive.getmember('xdzip-linux-x86_64/bin/xdzip')
                self.assertEqual(app.mode,0o755);self.assertEqual(app.uid,0);self.assertEqual(app.mtime,0)
            with self.assertRaisesRegex(ValueError,'outside'):archive_payload(root,root/'bad.tar.gz')

if __name__=='__main__':unittest.main()
