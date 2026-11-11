#!/usr/bin/env python3
"""Bounded real-parser and TLS controls for the actual security-patched QtNetwork."""
import argparse
import json
import os
from pathlib import Path
import socketserver
import ssl
import subprocess
import sys
import threading
from urllib.parse import urlsplit

from qt_network_security import ROOT, digest, library, run
from update_proxy_fixture import DIRECT_HOST, OwnedUpdateProxy


def check(qt, source, output):
    output.mkdir(parents=True, exist_ok=True)
    environment = {k: v for k, v in os.environ.items()
                   if not k.startswith(('QT_', 'QML_', 'DYLD_', 'LD_LIBRARY_PATH', 'XDZIP_'))}
    environment['PATH'] = str(qt/'bin')+os.pathsep+environment.get('PATH', '')
    environment['QT_PLUGIN_PATH'] = str(qt/'plugins')
    suffix = '.exe' if sys.platform == 'win32' else ''
    run(['cmake', '-S', ROOT/'tools/qt-network-security', '-B', output/'probe', '-G', 'Ninja',
         '-DCMAKE_BUILD_TYPE=Release', '-DCMAKE_PREFIX_PATH='+str(qt),
         '-DQTBASE_SOURCE='+str(source)], output/'probe-configure.log')
    run(['cmake', '--build', output/'probe', '--parallel', '2'], output/'probe-build.log')
    try:
        result = subprocess.run([str(output/'probe'/('xdzip-qt-authentication-tests'+suffix)),
                                 '-o', str(output/'authentication.log')+',txt'], env=environment,
                                capture_output=True, timeout=60)
    finally:
        # QtTest failure locations contain __FILE__; sanitize failure and timeout
        # logs too, before raising or leaving evidence in the build directory.
        log = output/'authentication.log'
        if log.exists():
            content = log.read_text(errors='replace')
            for path, replacement in ((ROOT, '<workspace>'), (Path.home(), '<user>')):
                content = content.replace(str(path), replacement).replace(path.as_posix(), replacement)
            log.write_text(content)
    if result.returncode:
        raise ValueError('Official patched Qt authentication regression failed')
    authentication = (output/'authentication.log').read_text(errors='replace')
    if 'ntlmMalformedTargetInfo' not in authentication or 'ntlmV2KnownVectors' not in authentication:
        raise ValueError('Official NTLM regression did not execute the required controls')
    state = {'mode': 'valid'}

    class Server(socketserver.ThreadingTCPServer):
        daemon_threads = True

        def handle_error(self, *_):
            pass  # Expected peer close on deliberately rejected bounded inputs.

    class Handler(socketserver.StreamRequestHandler):
        def handle(self):
            self.connection.settimeout(4)
            first = self.rfile.readline(4096)
            if not first.startswith((b'GET ', b'CONNECT ')):
                return
            total = len(first)
            while total < 16384:
                line = self.rfile.readline(4096); total += len(line)
                if line in (b'\r\n', b'\n', b''):
                    break
            if total >= 16384:
                return
            mode = state['mode']
            if mode == 'status':
                data = b'HTTP/1.1 200 '+b'a'*(128*1024)
            elif mode in ('header', 'fragmented'):
                data = b'HTTP/1.1 200 OK\r\n'+(b'X-Owned: '+b'a'*32768+b'\r\n')*17
            elif mode == 'proxy-502':
                data = b'HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n'
            else:
                reason = b'a'*1024 if mode == 'long-reason' else b'OK'
                ending = { 'crlf': b'\r\n\r\n', 'crlf-lf': b'\r\n\n',
                           'lf-crlf': b'\n\r\n', 'lf': b'\n\n' }.get(mode, b'\r\n\r\n')
                large = (b'X-One: '+b'a'*90000+b'\r\nX-Two: '+b'a'*90000+b'\r\n') if mode == 'large-valid' else b''
                data = b'HTTP/1.1 200 '+reason+b'\r\n'+large+b'Content-Length: 5'+ending+b'owned'
            try:
                step = 8192 if mode == 'fragmented' else len(data)
                for offset in range(0, len(data), step):
                    self.wfile.write(data[offset:offset+step]); self.wfile.flush()
                    if mode == 'fragmented':
                        threading.Event().wait(0.002)
                if mode in ('status', 'header', 'fragmented'):
                    self.connection.recv(1)  # Short fixture deadline, never infinite.
            except OSError:
                pass

    server = Server(('127.0.0.1', 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True); thread.start()
    tls_server = None; tls_thread = None; cases = []
    expected_digest = digest(library(qt, 'Network'))

    def execute(name, url, *, proxy=None, ca=None, good=False, tls_reject=False):
        params = dict(url=url, result=str(output/(name+'.json')))
        if proxy is not None:
            params['proxy_port'] = proxy
        if ca is not None:
            params['ca'] = str(ca)
        path = output/(name+'-input.json'); path.write_text(json.dumps(params))
        process = subprocess.run([str(output/'probe'/('xdzip-qt-network-security'+suffix)), str(path)],
                                 env=environment, capture_output=True, timeout=10)
        if process.returncode:
            raise ValueError('Native Qt network probe failed: '+name)
        result = json.loads((output/(name+'.json')).read_bytes()); path.unlink()
        if result['network_sha256'] != expected_digest or result['cancelled_by_fixture'] or result['body_overflow']:
            raise ValueError('Wrong QtNetwork loaded or parser failed to reject before fixture timeout: '+name)
        if good:
            if result['error'] != 0 or result['http_status'] != 200 or result['body'] != b'owned'.hex():
                raise ValueError('Legitimate QtNetwork control failed: '+name)
        elif not result['error'] or result['body']:
            raise ValueError('Invalid QtNetwork input was accepted: '+name)
        if tls_reject and not result['tls_errors']:
            raise ValueError('Untrusted TLS certificate was not rejected')
        cases.append(dict(case=name, result=result))

    try:
        origin = 'http://127.0.0.1:'+str(server.server_address[1])
        for mode in ('valid', 'long-reason', 'large-valid', 'crlf', 'crlf-lf', 'lf-crlf', 'lf', 'status', 'header', 'fragmented'):
            state['mode'] = mode
            execute('http-'+mode, origin+'/'+mode, good=mode not in ('status', 'header', 'fragmented'))
        for mode in ('status', 'header', 'fragmented', 'proxy-502'):
            state['mode'] = mode
            execute('connect-'+mode, 'https://'+DIRECT_HOST+':443/', proxy=server.server_address[1])
        certificate = output/'tls'
        certificate.mkdir(exist_ok=True)
        (certificate/'leaf.cnf').write_text('[extensions]\nsubjectAltName=DNS:'+DIRECT_HOST+',IP:127.0.0.1\n'
            'extendedKeyUsage=serverAuth\nbasicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\n')
        (certificate/'ca.cnf').write_text('[req]\nprompt=no\ndistinguished_name=dn\nx509_extensions=ca\n'
            '[dn]\nCN=xdzip-owned-test-ca\n[ca]\nbasicConstraints=critical,CA:TRUE\nkeyUsage=critical,keyCertSign,cRLSign\n')
        commands = [ ['openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '1',
                      '-config', certificate/'ca.cnf', '-keyout', certificate/'ca.key', '-out', certificate/'ca.pem'],
                     ['openssl', 'req', '-newkey', 'rsa:2048', '-nodes', '-subj', '/CN='+DIRECT_HOST,
                      '-keyout', certificate/'leaf.key', '-out', certificate/'leaf.csr'],
                     ['openssl', 'x509', '-req', '-in', certificate/'leaf.csr', '-CA', certificate/'ca.pem',
                      '-CAkey', certificate/'ca.key', '-CAcreateserial', '-days', '1',
                      '-extfile', certificate/'leaf.cnf', '-extensions', 'extensions', '-out', certificate/'leaf.pem'] ]
        for index, command in enumerate(commands):
            run(command, output/('certificate-'+str(index)+'.log'))
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(certificate/'leaf.pem', certificate/'leaf.key')

        class TLSServer(Server):
            def get_request(self):
                connection, address = super().get_request(); connection.settimeout(4)
                try:
                    return context.wrap_socket(connection, server_side=True), address
                except OSError:
                    connection.close(); raise

        tls_server = TLSServer(('127.0.0.1', 0), Handler)
        tls_thread = threading.Thread(target=tls_server.serve_forever, daemon=True); tls_thread.start()
        state['mode'] = 'valid'
        tls_origin = 'https://127.0.0.1:'+str(tls_server.server_address[1])+'/valid'
        execute('tls-owned-ca', tls_origin, ca=certificate/'ca.pem', good=True)
        execute('tls-untrusted', tls_origin, tls_reject=True)
        with OwnedUpdateProxy(tls_server.server_address[1], 'http', ('/valid',), host=DIRECT_HOST) as proxy:
            execute('connect-tls-owned-ca', 'https://'+DIRECT_HOST+':'+str(tls_server.server_address[1])+'/valid',
                    proxy=proxy.server.server_address[1], ca=certificate/'ca.pem', good=True)
            if not any(x['transport'] == 'http-connect' and x['status'] == 200 for x in proxy.evidence):
                raise ValueError('TLS CONNECT control did not actually traverse its proxy')
    finally:
        if tls_server:
            tls_server.shutdown(); tls_server.server_close(); tls_thread.join(timeout=5)
        server.shutdown(); server.server_close(); thread.join(timeout=5)
    report = dict(status='passed', network_sha256=expected_digest, cases=cases,
                  authentication='Patched upstream QAuthenticator suite, including malformed NTLM and known vectors',
                  scope='Actual SDK binary, bounded owned HTTP/CONNECT/TLS services, per-request test CA only; no system network/trust changes')
    (output/'result.json').write_text(json.dumps(report, indent=2)+'\n')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if not args.output.resolve().is_relative_to(ROOT/'build') or not args.source.resolve().is_relative_to(ROOT/'build'):
        parser.error('Use owned build/ directories')
    result = check(args.qt_prefix.resolve(), args.source.resolve(), args.output.resolve())
    print('QtNetwork security checks passed: '+str(len(result['cases']))+' real transport cases and upstream authentication suite')
