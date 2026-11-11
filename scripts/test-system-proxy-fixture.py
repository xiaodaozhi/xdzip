#!/usr/bin/env python3
"""Verify isolation guards and that test proxies forward only their owned origin."""
import http.client
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import os
import socket
import struct
import threading
import unittest
from unittest.mock import patch

from system_proxy_fixture import OwnedSystemProxy, require_hosted
from update_proxy_fixture import DIRECT_HOST, HOST, OwnedUpdateProxy


class SystemProxyFixtureTests(unittest.TestCase):
    def test_system_changes_require_all_disposable_host_and_architecture_guards(self):
        expected = {'CI': 'true', 'GITHUB_ACTIONS': 'true',
                    'RUNNER_ENVIRONMENT': 'github-hosted', 'XDZIP_ISOLATED_UPDATE_FIXTURE': '1'}
        with patch('system_proxy_fixture.sys.platform', 'darwin'), patch('system_proxy_fixture.platform.machine', return_value='arm64'):
            for missing in expected:
                env = {k: v for k, v in expected.items() if k != missing}
                with self.subTest(missing=missing), patch.dict(os.environ, env, clear=True), patch('system_proxy_fixture.subprocess.Popen') as spawn:
                    with self.assertRaisesRegex(RuntimeError, 'disposable'):
                        OwnedSystemProxy('not-a-helper', 1234, 'http://127.0.0.1:1234/proxy.pac')
                    spawn.assert_not_called()
            with patch.dict(os.environ, expected, clear=True):
                require_hosted()
                with patch('system_proxy_fixture.platform.machine', return_value='x86_64'):
                    with self.assertRaises(RuntimeError):
                        require_hosted()
                with patch('system_proxy_fixture.sys.platform', 'linux'):
                    with self.assertRaises(RuntimeError):
                        require_hosted()

    def setUp(self):
        self.origin_requests = []
        requests = self.origin_requests

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def do_GET(self):
                requests.append(self.path)
                self.send_response(200)
                self.send_header('Content-Length', '5')
                self.end_headers()
                self.wfile.write(b'owned')

        self.server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        self.server.daemon_threads = True
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.addCleanup(self.close)

    def close(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=5)

    def test_http_owned_origin_and_denied_https_target_remain_distinguishable(self):
        with OwnedUpdateProxy(self.server.server_port, 'http', ('/feed',), host=DIRECT_HOST) as proxy:
            port = proxy.server.server_address[1]
            client = http.client.HTTPConnection('127.0.0.1', port, timeout=5)
            client.request('GET', 'http://'+DIRECT_HOST+':'+str(self.server.server_port)+'/feed')
            result = client.getresponse()
            self.assertEqual((result.status, result.read()), (200, b'owned'))
            client.close()
            for host, target_port, owned in ((HOST, self.server.server_port, False),
                                              (DIRECT_HOST, self.server.server_port+1 if self.server.server_port<65535 else 1, True)):
                client = http.client.HTTPConnection('127.0.0.1', port, timeout=5)
                client.request('CONNECT', host+':'+str(target_port))
                result = client.getresponse()
                self.assertEqual(result.status, 403)
                result.read()
                client.close()
                self.assertTrue(any(r['status'] == 403 and r['owned_target'] == owned for r in proxy.evidence))
            self.assertEqual(self.origin_requests, ['/feed'])
            with self.assertRaises(ValueError):
                OwnedUpdateProxy(self.server.server_port, host='outside.invalid')

    def test_socks_owned_name_is_mapped_only_to_the_fixed_loopback_port(self):
        with OwnedUpdateProxy(self.server.server_port, 'socks5', ('/feed',), host=DIRECT_HOST) as proxy:
            with socket.create_connection(proxy.server.server_address, timeout=5) as connection:
                connection.sendall(b'\x05\x01\x00')
                self.assertEqual(connection.recv(2), b'\x05\x00')
                host = DIRECT_HOST.encode()
                connection.sendall(b'\x05\x01\x00\x03'+bytes([len(host)])+host+struct.pack('!H', self.server.server_port))
                reply = b''
                while len(reply) < 10:
                    reply += connection.recv(10-len(reply))
                self.assertEqual(reply[:2], b'\x05\x00')
                connection.sendall(b'GET /feed HTTP/1.0\r\n\r\n')
                response = b''
                while block := connection.recv(4096):
                    response += block
                self.assertTrue(response.endswith(b'owned'))
            self.assertEqual(self.origin_requests, ['/feed'])


if __name__ == '__main__':
    unittest.main()
