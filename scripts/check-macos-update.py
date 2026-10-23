# scripts/check-macos-update.py

"""Real signed Sparkle replacement/relaunch regression in owned build-only apps."""
import argparse
import base64
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import importlib.util
import json
import math
import os
from pathlib import Path
import platform
import plistlib
import re
import secrets
import shutil
import subprocess
import sys
import tarfile
import threading
import time
import xml.etree.ElementTree as ET

from deployment_audit import audit, mac_rpaths, native
from desktop_environment import locked_rust_version
from macos_bundle import optimize_bundle, validate_optimized_bundle
from sparkle_bundle import configuration, embed, signing_metadata
from update_fixture import ROOT, digest, dump

VERSION, OLD_BUILD, NEW_BUILD = '99.0.0-alpha.1', 730, 731
BUNDLE_ID = 'org.xdzip.update-regression'
SPARKLE = 'http://www.andymatuschak.org/xml-namespaces/sparkle'
ET.register_namespace('sparkle', SPARKLE)
HTTPS_ATTEMPTS = 3
HTTPS_WAIT_LIMIT = 3700


def clean_environment():
    environment = os.environ.copy()
    for key in ('XDZIP_UPDATE_SIGNING_KEY_HEX', 'XDZIP_SPARKLE_SIGNING_KEY_HEX',
                'QT_QPA_PLATFORM', 'QT_QUICK_BACKEND', 'DYLD_LIBRARY_PATH', 'DYLD_FRAMEWORK_PATH'):
        environment.pop(key, None)
    environment['MACOSX_DEPLOYMENT_TARGET'] = '14.0'
    # Preparation installs this exact minimal compiler. Explicit selection
    # avoids rustup re-synchronizing workspace-only fmt/clippy components while
    # the native build is running, without substituting another Rust version.
    environment['RUSTUP_TOOLCHAIN'] = locked_rust_version()
    return environment


def invoke(arguments, log=None, *, environment=None, input_data=None):
    result = subprocess.run(list(map(str, arguments)), env=environment or clean_environment(),
                            input=input_data, capture_output=True)
    if log:
        with Path(log).open('ab') as stream:
            stream.write(result.stdout + result.stderr)
    if result.returncode:
        # Signers may diagnose invalid key data. Never forward their stdout or
        # stderr; test key material is delivered through stdin/environment only.
        raise RuntimeError('Native regression command failed: '+Path(str(arguments[0])).name)
    return result.stdout


def load_script(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), ROOT/'scripts'/name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def private_source(output):
    """Compile the exact checked-in revision, with only disposable public trust changed."""
    source = output/'source'
    archive = output/'source.tar'
    invoke(['git', '-C', ROOT, 'archive', '--format=tar', '--output', archive, 'HEAD'])
    source.mkdir()
    with tarfile.open(archive) as stream:
        stream.extractall(source, filter='data')
    archive.unlink()
    return source


def deploy(source, build, qt, sdk, output, label):
    log = output/(label+'.log')
    app = output/label/'xdzip-update-probe.app'
    app.parent.mkdir()
    shutil.copytree(build/'apps/desktop/xdzip-update-probe.app', app, symlinks=True)
    invoke([qt/'bin/macdeployqt', app, '-always-overwrite', '-no-strip',
            '-qmldir='+str(source/'apps/desktop/ui'), '-qmlimport='+str(build/'qml'), '-verbose=1'], log)
    framework = embed(sdk, app, installable=False)
    builder = load_script('build-macos-dmg.py')
    def checked(arguments):
        return invoke(arguments, log).decode('utf-8')
    metadata = signing_metadata(framework, checked)
    info_path = app/'Contents/Info.plist'
    info = plistlib.loads(info_path.read_bytes())
    if info['CFBundleIdentifier'] != BUNDLE_ID:
        raise RuntimeError('Unexpected fixture bundle identity')
    # Loopback transport is confined to the separately named test bundle. Both
    # archive and complete feed signatures remain mandatory, using real Sparkle.
    info['NSAppTransportSecurity'] = {'NSAllowsLocalNetworking': True}
    info.pop('CFBundleDocumentTypes', None)
    info_path.write_bytes(plistlib.dumps(info, sort_keys=False))
    for driver in (app/'Contents/PlugIns/sqldrivers').glob('*.dylib'):
        if driver.name != 'libqsqlite.dylib':
            driver.unlink()
    builder.configure_translation_path(app)
    optimize_bundle(app)
    for path in {p.resolve() for p in app.rglob('*') if p.is_file() and native(p, 'macos')}:
        for rpath in mac_rpaths(checked(['otool', '-l', path])):
            if rpath.startswith('/') and not rpath.startswith(('/usr/lib/', '/System/Library/')):
                checked(['install_name_tool', '-delete_rpath', rpath, path])
    builder.sign_local(app, metadata, app.parent)
    for entitlements in app.parent.glob('sparkle-helper-*.entitlements'):
        entitlements.unlink()
    validate_optimized_bundle(app)
    return app


def feed_bytes(url, signature, length):
    rss = ET.Element('rss', {'version': '2.0'})
    channel = ET.SubElement(rss, 'channel')
    ET.SubElement(channel, 'title').text = 'Owned native updater fixture'
    item = ET.SubElement(channel, 'item')
    for name, text in [('version', str(NEW_BUILD)), ('shortVersionString', VERSION),
                       ('minimumSystemVersion', '14.0'), ('channel', 'alpha')]:
        ET.SubElement(item, '{'+SPARKLE+'}'+name).text = text
    ET.SubElement(item, 'enclosure', {'url': url, 'length': str(length),
                  'type': 'application/octet-stream', '{'+SPARKLE+'}edSignature': signature})
    return ET.tostring(rss, encoding='utf-8', xml_declaration=True)+b'\n'


def https_retry_plan(record, attempt, now):
    """Only a newly diagnosed anonymous GitHub API limit may delay another probe."""
    stop = lambda reason: dict(retry=False, reason=reason)
    if (type(record.get('http_diagnostics_schema')) is not int or
            record['http_diagnostics_schema'] != 1 or record.get('error') != 'rate_limited' or
            record.get('rate_limit_confirmed') is not True or
            record.get('confirmed_from_controller') is not True or record.get('timeout') is not False):
        return stop('not_confirmed_rate_limit')
    responses = record.get('http_responses')
    if not isinstance(responses, list) or not responses or not isinstance(responses[-1], dict):
        return stop('missing_http_response')
    requests = record.get('request_count')
    if (record.get('response_records_truncated') is not False or type(requests) is not int or
            requests < len(responses) or requests > 32):
        return stop('incomplete_http_diagnostics')
    response = responses[-1]
    status = response.get('status')
    if (type(status) is not int or status not in (403, 429) or
            response.get('host') != 'api.github.com' or response.get('route') != 'release_list' or
            record.get('authorization_present') is not False):
        return stop('not_anonymous_github_api_limit')
    headers = response.get('rate_headers')
    if not isinstance(headers, dict):
        return stop('missing_rate_headers')
    def integer_header(name):
        value = headers.get(name)
        if value is None:
            return None
        if (not isinstance(value, str) or not value.isascii() or not value.isdecimal()
                or len(value) > 20):
            raise ValueError(name)
        return int(value)
    try:
        remaining = integer_header('remaining')
        retry_after = integer_header('retry_after')
    except ValueError:
        return stop('invalid_rate_header')
    delay, reason, clock = None, None, {}
    if remaining == 0:
        try:
            reset = integer_header('reset')
        except ValueError:
            return stop('invalid_primary_reset')
        if reset is None:
            return stop('missing_primary_reset')
        server_time = response.get('server_date_epoch')
        server_clock = type(server_time) is int and server_time > 0
        current = server_time if server_clock else now
        if type(current) not in (int, float) or not math.isfinite(current) or current <= 0:
            return stop('invalid_reference_clock')
        if reset <= current:
            return stop('primary_reset_not_future')
        delay, reason = math.ceil(reset - current) + 1, 'primary_reset'
        clock = dict(reference_epoch=current, reference_clock='http_date' if server_clock else 'runner_wall_clock')
    if retry_after is not None:
        delay = max(delay or 1, retry_after, 1)
        reason = 'retry_after_and_primary_reset' if reason else 'retry_after'
    if delay is None:
        # Controller confirms explicit rate-limit responses; plain 403 never
        # reaches this branch. GitHub documents at least one minute without a
        # server deadline, followed by exponential backoff.
        delay, reason = 60 * 2 ** (attempt - 1), 'confirmed_limit_exponential_backoff'
    return dict(retry=True, reason=reason, delay_seconds=delay, **clock)


def https_probe(app, output, source_sha=''):
    """Read GitHub through the deployed Qt stack with no SDK environment fallback."""
    binary = app/'Contents/MacOS/xdzip-update-probe'
    deployment = audit(app, binary, None, 'macos', 'deployed')
    dump(output/'https-deployment.json', deployment)
    if deployment['status'] != 'passed':
        raise RuntimeError('HTTPS probe has dependencies outside the deployed application')
    environment = clean_environment()
    for key in list(environment):
        if (key.startswith(('QT_', 'QML_', 'QML2_', 'DYLD_')) or
                key in ('QTDIR', 'XDZIP_QT_PREFIX', 'CMAKE_PREFIX_PATH', 'LD_LIBRARY_PATH')):
            environment.pop(key)
    environment['PATH'] = '/usr/bin:/bin:/usr/sbin:/sbin'
    evidence = app.parent/'https.json'
    binary_sha = digest(binary)
    waits, waited = [], 0.0
    for attempt in range(1, HTTPS_ATTEMPTS + 1):
        before = digest(binary)
        if evidence.exists():
            evidence.unlink()  # Only the exact owned probe result, never prior attempt records.
        try:
            if before != binary_sha:
                log, returncode = b'', None
                record = dict(success=False, error='probe_binary_changed', timeout=False)
            else:
                result = subprocess.run([str(binary), '--update-https-probe'], env=environment,
                                        capture_output=True, timeout=200)
                log = result.stdout + result.stderr
                try:
                    record = json.loads(evidence.read_bytes())
                    if not isinstance(record, dict):
                        raise ValueError('Expected an object')
                except (OSError, ValueError):
                    record = dict(success=False, error='probe_evidence_invalid', timeout=False)
                returncode = result.returncode
        except subprocess.TimeoutExpired as error:
            log = (error.stdout or b'') + (error.stderr or b'')
            record = dict(success=False, error='probe_process_timeout', timeout=True)
            returncode = None
        except OSError as error:
            log = b''
            record = dict(success=False, error='probe_process_error', timeout=False,
                          process_error_kind=type(error).__name__, process_error_number=error.errno)
            returncode = None
        (output/f'https-attempt-{attempt}.log').write_bytes(log)
        (output/'https.log').write_bytes(log)
        after = digest(binary)
        unchanged = before == after == binary_sha
        record.update(sdk_environment_cleared=True, dependency_audit='https-deployment.json',
                      source_sha=source_sha, probe_binary_sha256=before,
                      probe_binary_after_sha256=after, probe_binary_unchanged=unchanged,
                      attempt=attempt, process_returncode=returncode)
        if not unchanged:
            record.update(success=False, error='probe_binary_changed')
        dump(output/f'https-attempt-{attempt}.json', record)
        dump(output/'https.json', record)
        if returncode == 0 and record.get('success') is True:
            dump(output/'https-waits.json', waits)
            return record
        plan = (https_retry_plan(record, attempt, time.time()) if returncode == 69
                else dict(retry=False, reason='probe_process_failure'))
        plan.update(attempt=attempt, waited_seconds=waited,
                    remaining_wait_budget_seconds=max(0, HTTPS_WAIT_LIMIT - waited))
        if attempt == HTTPS_ATTEMPTS:
            plan.update(retry=False, reason='attempt_limit_reached')
        elif plan.get('retry') and plan['delay_seconds'] > HTTPS_WAIT_LIMIT - waited:
            # Never clamp a server deadline and then request before it expires.
            plan.update(retry=False, reason='wait_budget_exceeded')
        waits.append(plan)
        dump(output/'https-waits.json', waits)
        if not plan['retry']:
            raise RuntimeError('Deployed Qt HTTPS probe failed: ' + plan['reason'])
        started = time.monotonic()
        plan['wait_status'] = 'waiting'
        dump(output/'https-waits.json', waits)
        finished = False
        try:
            time.sleep(plan['delay_seconds'])
            finished = True
        finally:
            elapsed = max(0.0, time.monotonic() - started)
            waited += elapsed
            enough = finished and elapsed >= plan['delay_seconds']
            plan.update(wait_status='finished' if enough else 'interrupted_or_short', actual_wait_seconds=elapsed,
                        total_actual_wait_seconds=waited)
            dump(output/'https-waits.json', waits)
        if not enough:
            raise RuntimeError('Deployed Qt HTTPS probe wait ended before the required deadline')
        if waited > HTTPS_WAIT_LIMIT:
            raise RuntimeError('Deployed Qt HTTPS probe wait budget exhausted')


class Handler(SimpleHTTPRequestHandler):
    def log_message(self, *_):
        pass


def scenario(output, label, old_app, archive, server_url, sdk, signing, public, secret, signature):
    cancel = label.startswith('cancel-ready')
    directory = output/'cases'/label
    directory.mkdir(parents=True)
    installed = directory/'xdzip-update-probe.app'
    shutil.copytree(old_app, installed, symlinks=True)
    # Never let a skipped fixture version in this dedicated domain change the
    # next scenario. No production preferences or launch associations are used.
    subprocess.run(['defaults', 'delete', BUNDLE_ID], capture_output=True)
    candidate_archive = directory/archive.name
    shutil.copy2(archive, candidate_archive)
    asset_url = server_url+'/cases/'+label+'/'+archive.name
    appcast = directory/'appcast.xml'
    appcast.write_bytes(feed_bytes(asset_url, signature, archive.stat().st_size))
    sign_update = sdk/'bin/sign_update'
    invoke([sign_update, '--ed-key-file', '-', '--disable-signing-warning', appcast], input_data=secret)
    invoke([sign_update, '--verify', '--ed-key-file', '-', appcast], input_data=secret)
    invoke([signing, 'verify-appcast', '--appcast', appcast, '--public-key', public])
    signed_hash = digest(appcast)
    if label == 'tampered-feed':
        appcast.write_bytes(appcast.read_bytes().replace(b'Owned native updater fixture', b'Owned native updater altered', 1))
    if label == 'tampered-archive':
        data = bytearray(candidate_archive.read_bytes())
        data[len(data)//2] ^= 1
        candidate_archive.write_bytes(data)
    dump(directory/'probe.json', dict(marker='xdzip-owned-mac-update-regression-v1',
         target_build=NEW_BUILD, mode='cancel' if cancel else 'install', candidate={
         'version': VERSION, 'build': NEW_BUILD, 'channel': 'alpha', 'asset_url': asset_url,
         'appcast_url': server_url+'/cases/'+label+'/appcast.xml', 'bytes': archive.stat().st_size}))
    log = (directory/'native.log').open('wb')
    process = subprocess.Popen([str(installed/'Contents/MacOS/xdzip-update-probe')],
                               env=clean_environment(), stdout=log, stderr=subprocess.STDOUT)
    try:
        deadline = time.monotonic()+180
        result_path = directory/'result.json'
        while not result_path.exists() and time.monotonic() < deadline:
            if process.poll() is not None:
                # Sparkle terminates the old process before replacement/relaunch.
                # Its detached installer may still be completing the transaction.
                if label != 'success':
                    raise RuntimeError('Fixture exited without its native result: '+label)
            time.sleep(0.1)
        if not result_path.exists():
            raise RuntimeError('Native replacement/relaunch timed out: '+label)
        result = json.loads(result_path.read_bytes())
        process.wait(timeout=20)
        # A cancelled ready installer must not install on subsequent application
        # quit. Give the detached installer time to expose that regression.
        if cancel:
            time.sleep(3)
        final_info = plistlib.loads((installed/'Contents/Info.plist').read_bytes())
        build = int(final_info['CFBundleVersion'])
        events = result.get('events', {})
        if label == 'success':
            passed = result.get('success') is True and result.get('relaunched_build') == NEW_BUILD and build == NEW_BUILD
            passed = passed and events.get('about_install_clicked') and events.get('barrier_before_install')
            passed = passed and events.get('guard_release_after_commit') and events.get('guard_release_after_join')
            invoke(['codesign', '--verify', '--deep', '--strict', installed], directory/'verify.log')
        else:
            error = 'cancelled' if cancel else 'native_update'
            passed = result.get('success') is False and result.get('error') == error and build == OLD_BUILD
            passed = passed and events.get('about_install_clicked') and not events.get('barrier_before_install')
            passed = passed and not events.get('guard_release_after_commit')
            if cancel:
                passed = passed and events.get('about_cancel_clicked') and events.get('native_quit_deferred')
                passed = passed and events.get('native_cycle_settled_before_exit')
                exit_status = events.get('native_exit_status', {})
                passed = passed and exit_status.get('installer_seen_at_ready') and exit_status.get('installer_pids_at_ready')
                passed = passed and exit_status.get('installer_gone_before_exit') and exit_status.get('native_cycle_finished')
                passed = passed and not exit_status.get('installer_observation_error')
        record = dict(scenario=label, passed=bool(passed), installed_build=build,
                      signed_feed_sha256=signed_hash, result=result)
        dump(directory/'verification.json', record)
        if not passed:
            raise RuntimeError('Native updater scenario did not meet its assertions: '+label)
        return record
    finally:
        if process.poll() is None:
            process.terminate()
            process.wait(timeout=10)
        log.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if sys.platform != 'darwin' or platform.machine() != 'arm64':
        parser.error('Requires a native Apple Silicon desktop runner')
    output, qt = args.output.resolve(), args.qt_prefix.resolve()
    if not output.is_relative_to(ROOT/'build') or output == ROOT/'build' or output.exists():
        parser.error('Use a fresh dedicated output directory beneath build/')
    if not (qt/'bin/macdeployqt').is_file():
        parser.error('A native Qt 6.8 SDK is required')
    output.mkdir(parents=True)
    source_sha = invoke(['git', '-C', ROOT, 'rev-parse', 'HEAD']).decode().strip()
    if re.fullmatch('[0-9a-f]{40}', source_sha) is None:
        raise RuntimeError('Unexpected source revision')
    production_key = ROOT/'resources/sparkle-update-key.json'
    before = digest(production_key)
    source = private_source(output)
    # Compile the signer with no signing authority. Its build scripts never see
    # this invocation's ephemeral key, and production trust is unchanged.
    invoke(['cargo', 'build', '--locked', '--manifest-path', source/'Cargo.toml',
            '--target-dir', output/'signing', '-p', 'xdzip-update', '--bin', 'xdzip-update-tool'], output/'signer.log')
    signing = output/'signing/debug/xdzip-update-tool'
    seed = secrets.token_bytes(32)
    signing_environment = clean_environment()
    signing_environment['XDZIP_SPARKLE_SIGNING_KEY_HEX'] = seed.hex()
    public = output/'public-key.json'
    invoke([signing, 'sparkle-public-key', '--output', public], environment=signing_environment)
    shutil.copy2(public, source/'resources/sparkle-update-key.json')
    # Sparkle 2.10's new export format is the 32-byte seed. The legacy 96-byte
    # format contains a hashed private key and must never be seed+public-key.
    secret = base64.b64encode(seed)+b'\n'
    sdk = output/'sdk'
    invoke([sys.executable, source/'scripts/prepare-sparkle.py', '--output', sdk], output/'sdk.log')
    apps = []
    for label, build_number in [('old', OLD_BUILD), ('new', NEW_BUILD)]:
        build = output/('build-'+label)
        invoke(['cmake', '-S', source, '-B', build, '-DCMAKE_BUILD_TYPE=Release',
                '-DCMAKE_PREFIX_PATH='+str(qt), '-DCMAKE_OSX_ARCHITECTURES=arm64',
                '-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0', '-DXDZIP_BUILD_QT_PROBE=OFF',
                '-DXDZIP_BUILD_DESKTOP_PREVIEW=ON', '-DBUILD_TESTING=ON',
                '-DXDZIP_BUILD_MAC_UPDATE_REGRESSION=ON', '-DXDZIP_MAC_UPDATE_PROBE_BUILD='+str(build_number),
                '-DXDZIP_SPARKLE_SDK='+str(sdk)], output/(label+'-configure.log'))
        invoke(['cmake', '--build', build, '--target', 'xdzip-mac-update-probe', '--parallel', '3'], output/(label+'-build.log'))
        apps.append(deploy(source, build, qt, sdk, output, label))
    https = https_probe(apps[0], output, source_sha)
    archive = output/('xdzip-'+VERSION+'-macos-arm64.app.zip')
    invoke(['ditto', '-c', '-k', '--sequesterRsrc', '--keepParent', apps[1], archive])
    metadata = output/'archive.json'
    invoke([signing, 'sign-archive', '--archive', archive, '--public-key', public, '--version', VERSION,
            '--build', str(NEW_BUILD), '--output', metadata], environment=signing_environment)
    archive_signature = json.loads(metadata.read_bytes())['ed_signature']
    # Official SDK signing is cross-checked with the production release signer.
    official = invoke([sdk/'bin/sign_update', '-p', '--ed-key-file', '-', archive], input_data=secret).decode().strip()
    if official != archive_signature:
        raise RuntimeError('Official Sparkle archive signature differs from release signer')
    invoke([sdk/'bin/sign_update', '--verify', '--ed-key-file', '-', archive, archive_signature], input_data=secret)
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(Handler, directory=str(output)))
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    results = []
    try:
        address = 'http://127.0.0.1:'+str(server.server_port)
        for label in ('tampered-feed', 'tampered-archive', 'cancel-ready', 'cancel-ready-repeat-1',
                      'cancel-ready-repeat-2', 'success'):
            results.append(scenario(output, label, apps[0], archive, address, sdk, signing, public, secret, archive_signature))
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
        subprocess.run(['defaults', 'delete', BUNDLE_ID], capture_output=True)
        for app in output.glob('cases/*/*.app'):
            invoke(['/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister', '-u', app])
    if digest(production_key) != before:
        raise RuntimeError('Production Sparkle trust unexpectedly changed')
    dump(output/'verification.json', dict(schema_version=1, platform='macos', architecture='arm64',
         status='passed', sdk=configuration()['version'], production_trust_unchanged=True,
         scope='Real About controls, two managed sessions, Sparkle signed feed/archive, cancellation, replacement and relaunch; candidate discovery is independently tested.',
         https=https, scenarios=results))
    print('Native signed Sparkle regression passed: '+str(len(results))+' scenarios')


if __name__ == '__main__':
    main()
