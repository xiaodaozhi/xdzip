"""Exercise the real patched Sparkle downloader through owned HTTP/SOCKS5 proxies."""
import argparse
import base64
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import platform
import plistlib
import shutil
import ssl
import subprocess
import sys
import threading
import uuid
import zipfile

from sparkle_bundle import validate_sdk
from update_fixture import SEED, PUBLIC
from update_proxy_fixture import HOST, DIRECT_HOST, OwnedUpdateProxy
from system_proxy_fixture import OwnedSystemProxy, compile_helper, require_hosted

ROOT = Path(__file__).resolve().parents[1]


def check(sdk, output, system_conflicts=False):
    if system_conflicts:
        require_hosted()
    validate_sdk(sdk)
    output.mkdir(parents=True)
    temporary = output/'temporary'
    temporary.mkdir()

    def run(arguments, name, data=None, env=None):
        result = subprocess.run(list(map(str, arguments)), input=data, env=env,
                                capture_output=True, timeout=45)
        log = (result.stdout + result.stderr).replace(str(ROOT).encode(), b'<workspace>')
        log = log.replace(str(Path.home()).encode(), b'<user>')
        (output/(name+'.log')).write_bytes(log)
        if result.returncode:
            raise RuntimeError('Owned Sparkle proxy command failed: ' + name)
        return result.stdout

    run(['xcrun', 'clang', '-fobjc-arc', '-mmacosx-version-min=14.0', '-F', sdk,
         '-framework', 'Sparkle', '-framework', 'AppKit',
         ROOT/'apps/desktop/tests/sparkle_proxy_probe.m', '-Wl,-rpath,'+str(sdk),
         '-o', temporary/'probe'], 'compile')
    archive = temporary/'owned.zip'
    with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_STORED) as stream:
        stream.writestr('owned-content.txt', b'Owned transport content\n'*8192)
    secret = base64.b64encode(bytes.fromhex(SEED)) + b'\n'
    signature = run([sdk/'bin/sign_update', '-p', '--ed-key-file', '-', archive], 'sign-archive', secret).decode().strip()
    original = 'https://github.com/xiaodaozhi/xdzip/releases/download/owned-proxy/owned.zip'
    feed = temporary/'appcast.xml'
    feed.write_text(f'''<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle"><channel>
<title>Owned transport</title><item><title>Owned transport 1.1</title>
<sparkle:version>101</sparkle:version><sparkle:shortVersionString>1.1</sparkle:shortVersionString>
<enclosure url="{original}" length="{archive.stat().st_size}" type="application/octet-stream" sparkle:edSignature="{signature}" />
</item></channel></rss>\n''', encoding='utf-8')
    run([sdk/'bin/sign_update', '--ed-key-file', '-', '--disable-signing-warning', feed], 'sign-feed', secret)
    signed_feed = feed.read_bytes()
    state = {'tampered': False, 'requests': [], 'pac_requests': 0, 'pac_script': b''}

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_GET(self):
            if self.path.split('?', 1)[0] == '/proxy.pac':
                state['pac_requests'] += 1
                self.send_response(200)
                self.send_header('Content-Type', 'application/x-ns-proxy-autoconfig')
                self.send_header('Content-Length', str(len(state['pac_script'])))
                self.end_headers()
                self.wfile.write(state['pac_script'])
                return
            data = (signed_feed.replace(b'Owned transport 1.1', b'Owned transport 1.2')
                    if state['tampered'] else signed_feed) if self.path == '/feed' else archive.read_bytes()
            if self.path not in ('/feed', '/archive'):
                self.send_error(404)
                return
            self.send_response(200)
            self.send_header('Content-Length', str(len(data)))
            self.send_header('Content-Type', 'application/rss+xml' if self.path == '/feed' else 'application/octet-stream')
            self.end_headers()
            self.wfile.write(data)
            state['requests'].append(self.path)

    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    server.daemon_threads = True
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    certificate_config = temporary/'tls.cnf'
    certificate_config.write_text('[req]\nprompt=no\ndistinguished_name=dn\nx509_extensions=extensions\n'
        '[dn]\nCN='+DIRECT_HOST+'\n[extensions]\nsubjectAltName=DNS:'+DIRECT_HOST+
        '\nextendedKeyUsage=serverAuth\nbasicConstraints=critical,CA:FALSE\n'
        'keyUsage=critical,digitalSignature,keyEncipherment\n')
    run(['openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '1',
         '-config', certificate_config, '-keyout', temporary/'tls-key.pem',
         '-out', temporary/'tls-cert.pem'], 'owned-untrusted-certificate')
    tls_context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    tls_context.load_cert_chain(temporary/'tls-cert.pem', temporary/'tls-key.pem')

    class TLSServer(ThreadingHTTPServer):
        accepts = 0

        def get_request(self):
            connection, address = super().get_request()
            self.accepts += 1
            connection.settimeout(3)
            try:
                return tls_context.wrap_socket(connection, server_side=True), address
            except OSError:
                connection.close()
                raise

    tls_server = TLSServer(('127.0.0.1', 0), Handler)
    tls_server.daemon_threads = True
    tls_thread = threading.Thread(target=tls_server.serve_forever, daemon=True)
    tls_thread.start()
    results = []
    conflict_results = []
    restoration = []

    def exercise(name, mode, proxy_url, origin, *, xpc=False, tampered=False):
        case = temporary/name
        app = case/'OwnedProxy.app'
        binary = app/'Contents/MacOS/Probe'
        binary.parent.mkdir(parents=True)
        shutil.copy2(temporary/'probe', binary)
        home = case/'profile'
        home.mkdir()
        info = {'CFBundleIdentifier': 'org.xdzip.owned-proxy.'+uuid.uuid4().hex,
                'CFBundleName': 'Owned Proxy', 'CFBundleExecutable': 'Probe',
                'CFBundlePackageType': 'APPL', 'CFBundleVersion': '100',
                'CFBundleShortVersionString': '1.0', 'LSUIElement': True,
                'SUPublicEDKey': base64.b64encode(bytes.fromhex(PUBLIC)).decode(),
                'SUEnableAutomaticChecks': False, 'SUAutomaticallyUpdate': False,
                'SUAllowsAutomaticUpdates': False, 'SURequireSignedFeed': True,
                'SUSignedFeedFailureExpirationInterval': 0,
                'SUVerifyUpdateBeforeExtraction': True, 'SUEnableDownloaderService': xpc,
                'NSAppTransportSecurity': {'NSAllowsLocalNetworking': True, 'NSExceptionDomains': {
                    HOST: {'NSExceptionAllowsInsecureHTTPLoads': True},
                    DIRECT_HOST: {'NSExceptionAllowsInsecureHTTPLoads': True}}}}
        (app/'Contents/Info.plist').write_bytes(plistlib.dumps(info))
        state.update(tampered=tampered, requests=[])
        params = {'case': name, 'mode': mode, 'isolated_home': str(home), 'result': str(case/'result.json'),
                  'feed_url': origin+'/feed', 'asset_transport_url': origin+'/archive',
                  'original_asset_url': original, 'bytes': archive.stat().st_size, 'token': 'owned-test'}
        if proxy_url is not None:
            params['proxy_url'] = proxy_url
        path = case/'parameters.json'
        path.write_text(json.dumps(params))
        env = {k: v for k, v in os.environ.items() if not k.startswith(('DYLD_', 'QT_', 'QML_', 'XDZIP_'))}
        env['CFFIXED_USER_HOME'] = str(home)
        run([binary, path], name, env=env)
        result = json.loads((case/'result.json').read_bytes())
        result['origin_requests'] = list(state['requests'])
        return result

    def success(result, proxy_requests, direct):
        unexpected_proxy = bool(proxy_requests) if direct else len(proxy_requests) < 2
        if (result['outcome'] != 'downloaded_before_extraction' or not result.get('candidate_matched') or
                not result.get('original_request_matched') or not result.get('feed_signature_valid') or
                not result.get('proxy_change_rejected_after_start') or
                result['origin_requests'] != ['/feed', '/archive'] or
                unexpected_proxy or
                any(item['status'] != 200 for item in proxy_requests)):
            raise RuntimeError('Real Sparkle transport did not pass: '+result['case'])

    try:
        basic = [('http', 'http'), ('socks5', 'socks5'), ('unavailable', 'http'),
                 ('tampered-feed', 'http'), ('credentials-rejected', 'http'), ('xpc-rejected', 'http'),
                 ('direct', 'http'), ('direct-tampered-feed', 'http'), ('direct-url-rejected', 'http'),
                 ('direct-xpc-rejected', 'http'), ('invalid-mode-rejected', 'http')]
        for name, scheme in basic:
            with OwnedUpdateProxy(server.server_port, scheme, ('/feed', '/archive')) as proxy:
                mode = 'direct' if name.startswith('direct') else 'invalid' if name == 'invalid-mode-rejected' else 'custom'
                origin = 'http://'+('127.0.0.1' if mode == 'direct' else HOST)+':'+str(server.server_port)
                address = ('http://127.0.0.1:1' if name == 'unavailable' else
                           proxy.address.replace('://', '://user:secret@') if name == 'credentials-rejected' else
                           None if mode != 'custom' and name != 'direct-url-rejected' else proxy.address)
                result = exercise(name, mode, address, origin, xpc=name.endswith('xpc-rejected'),
                                  tampered='tampered-feed' in name)
                result['proxy_requests'] = list(proxy.evidence)
                if name in ('http', 'socks5', 'direct'):
                    success(result, result['proxy_requests'], mode == 'direct')
                elif (result['outcome'] not in ('updater_error', 'configuration_rejected') or
                      '/archive' in result['origin_requests']):
                    raise RuntimeError('Invalid Sparkle transport was not rejected: '+name)
                if 'rejected' in name or name == 'unavailable':
                    if result['origin_requests']:
                        raise RuntimeError('Rejected policy started transport')
                results.append(result)
        tls_server.accepts = 0
        result = exercise('direct-untrusted-tls', 'direct', None,
                          'https://127.0.0.1:'+str(tls_server.server_port))
        if (result['outcome'] != 'updater_error' or not tls_server.accepts or result['origin_requests'] or
                not any(e['domain'] == 'NSURLErrorDomain' and e['code'] in (-1200, -1202, -1203)
                        for e in result['errors'])):
            raise RuntimeError('Direct transport relaxed TLS validation')
        result['tls_connections'] = tls_server.accepts
        results.append(result)
        if system_conflicts:
            helper = compile_helper(temporary, run)
            with OwnedUpdateProxy(server.server_port, 'http', ('/feed', '/archive'), host=DIRECT_HOST) as proxy:
                state['pac_script'] = ('function FindProxyForURL(url, host) { return host === "'+DIRECT_HOST+
                    '" ? "PROXY 127.0.0.1:'+str(proxy.server.server_address[1])+'" : "DIRECT"; }').encode()
                pac_url = 'http://127.0.0.1:'+str(server.server_port)+'/proxy.pac?owned='+uuid.uuid4().hex
                fixture = OwnedSystemProxy(helper, proxy.server.server_address[1], pac_url)
                try:
                    with fixture:
                        origin = 'http://'+DIRECT_HOST+':'+str(server.server_port)
                        for profile in ('static', 'pac'):
                            fixture.apply(profile)
                            for mode in ('system', 'direct'):
                                with proxy.lock:
                                    proxy.evidence.clear()
                                result = exercise(profile+'-'+mode, mode, None, origin)
                                owned = [r for r in proxy.evidence if r['owned_target']]
                                success(result, owned, mode == 'direct')
                                result.update(profile=profile, proxy_requests=owned,
                                              system_profile_verified=True, pac_requests=state['pac_requests'])
                                conflict_results.append(result)
                            for mode in ('system', 'direct'):
                                with proxy.lock:
                                    proxy.evidence.clear()
                                tls_server.accepts = 0
                                result = exercise(profile+'-'+mode+'-untrusted-tls', mode, None,
                                    'https://'+DIRECT_HOST+':'+str(tls_server.server_port))
                                owned = [r for r in proxy.evidence if r['owned_target']]
                                if mode == 'system':
                                    if not owned or tls_server.accepts:
                                        raise RuntimeError('System HTTPS control did not use the active proxy')
                                elif (owned or not tls_server.accepts or
                                      not any(e['domain'] == 'NSURLErrorDomain' and e['code'] in (-1200, -1202, -1203)
                                              for e in result['errors'])):
                                    raise RuntimeError('Direct HTTPS did not bypass the proxy with TLS trust intact')
                                if result['outcome'] != 'updater_error' or result['origin_requests']:
                                    raise RuntimeError('Untrusted HTTPS reached appcast content')
                                result.update(profile=profile, proxy_requests=owned,
                                              tls_connections=tls_server.accepts, system_profile_verified=True)
                                conflict_results.append(result)
                            fixture.verify(profile)
                finally:
                    restoration = fixture.records
                    (output/'system-policy-restoration.json').write_text(json.dumps(restoration, indent=2)+'\n')
    finally:
        tls_server.shutdown()
        tls_server.server_close()
        tls_thread.join(timeout=5)
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
        shutil.rmtree(temporary)
    report = {'schema_version': 1, 'status': 'passed', 'platform': platform.machine(),
              'macos': platform.mac_ver()[0], 'cases': results,
              'system_conflicts': conflict_results, 'system_restoration': restoration,
              'scope': 'Actual patched Sparkle feed/archive transport, signed-feed and untrusted-TLS rejection; stopped before extraction/installation. System conflicts run only on disposable hosted Macs; successful public HTTPS is independently checked.'}
    (output/'proxy-verification.json').write_text(json.dumps(report, indent=2)+'\n')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sdk', type=Path, default=ROOT/'build/toolchain/Sparkle/2.10.0-xdzip-proxy.2')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--system-conflicts', action='store_true', help='Disposable hosted runner only: verify real static system proxy and PAC overrides')
    args = parser.parse_args()
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        parser.error('Native Apple Silicon macOS is required')
    if args.output.exists() or not args.output.resolve().is_relative_to(ROOT/'build'):
        parser.error('Use a fresh owned directory under build/')
    report = check(args.sdk.resolve(), args.output.resolve(), args.system_conflicts)
    print('Actual Sparkle transport regression passed: '+str(len(report['cases']))+' basic cases, '+str(len(report['system_conflicts']))+' real system-conflict cases')
