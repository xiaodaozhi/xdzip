"""Disposable HTTP CONNECT/SOCKS5 fixtures restricted to one owned origin."""
import http.client
import select
import socket
import socketserver
import struct
import threading
import time
from urllib.parse import urlsplit

HOST = 'xdzip-owned-proxy.invalid'


class OwnedUpdateProxy:
    def __init__(self, target_port, scheme='http', paths=()):
        if scheme not in ('http', 'socks5') or not 0 < target_port < 65536:
            raise ValueError('Invalid owned proxy configuration')
        self.scheme = scheme
        self.target_port = target_port
        self.paths = frozenset(paths)
        self.evidence = []
        self.lock = threading.Lock()
        owner = self

        class Server(socketserver.ThreadingTCPServer):
            daemon_threads = True

            def handle_error(self, *_):
                owner.record('rejected', 0)

        class Handler(socketserver.StreamRequestHandler):
            def handle(self):
                self.connection.settimeout(30)
                if owner.scheme == 'http':
                    owner.http(self)
                else:
                    owner.socks(self)

        self.server = Server(('127.0.0.1', 0), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    @property
    def address(self):
        return self.scheme + '://127.0.0.1:' + str(self.server.server_address[1])

    def record(self, transport, status, path=None):
        with self.lock:
            item = {'transport': transport, 'status': status}
            if path is not None:
                item['path'] = path
            self.evidence.append(item)

    def valid_target(self, host, port):
        return host in (HOST, '127.0.0.1') and port == self.target_port

    def tunnel(self, connection):
        # Both proxy protocols map only the fixed fixture host/port to loopback.
        # Never resolve or connect to a client-supplied external destination.
        with socket.create_connection(('127.0.0.1', self.target_port), timeout=30) as target:
            total = 0
            deadline = time.monotonic() + 3600
            while total <= 2*1024**3 + 1024**2 and time.monotonic() < deadline:
                ready, _, _ = select.select([connection, target], [], [], 30)
                if not ready:
                    return
                for source in ready:
                    data = source.recv(64*1024)
                    if not data:
                        return
                    total += len(data)
                    (target if source is connection else connection).sendall(data)

    def http(self, handler):
        first = handler.rfile.readline(4096)
        header_bytes = len(first)
        while header_bytes < 16*1024:
            line = handler.rfile.readline(4096)
            header_bytes += len(line)
            if line in (b'\r\n', b''):
                break
        parts = first.split()
        if len(parts) != 3 or header_bytes >= 16*1024:
            return
        method, address, _ = parts
        if method == b'CONNECT':
            target = urlsplit('http://' + address.decode('ascii'))
            if (not self.valid_target(target.hostname, target.port) or
                    target.username or target.password or target.path or target.query or target.fragment):
                handler.wfile.write(b'HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n')
                self.record('http-connect', 403)
                return
            handler.wfile.write(b'HTTP/1.1 200 Connection Established\r\n\r\n')
            self.record('http-connect', 200)
            self.tunnel(handler.connection)
        elif method == b'GET':
            target = urlsplit(address.decode('ascii'))
            if (target.scheme != 'http' or not self.valid_target(target.hostname, target.port) or
                    target.username or target.password or target.query or target.fragment or
                    target.path not in self.paths):
                handler.wfile.write(b'HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n')
                self.record('http-get', 403)
                return
            connection = http.client.HTTPConnection('127.0.0.1', self.target_port, timeout=30)
            try:
                connection.request('GET', target.path)
                response = connection.getresponse()
                header = ('HTTP/1.1 ' + str(response.status) + ' Owned\r\nContent-Length: ' +
                          response.getheader('Content-Length', '0') + '\r\nConnection: close\r\n\r\n')
                handler.wfile.write(header.encode('ascii'))
                while data := response.read(64*1024):
                    handler.wfile.write(data)
                self.record('http-get', response.status, target.path)
            finally:
                connection.close()

    def socks(self, handler):
        def read(count):
            data = handler.rfile.read(count)
            if len(data) != count:
                raise ValueError('Short owned SOCKS request')
            return data

        version, count = read(2)
        if version != 5 or not 0 < count <= 8 or 0 not in read(count):
            return
        handler.wfile.write(b'\x05\x00')
        version, command, reserved, kind = read(4)
        if (version, command, reserved) != (5, 1, 0):
            return
        if kind == 1:
            host = socket.inet_ntoa(read(4))
        elif kind == 3:
            host = read(read(1)[0]).decode('ascii')
        else:
            return
        port = struct.unpack('!H', read(2))[0]
        if not self.valid_target(host, port):
            handler.wfile.write(b'\x05\x02\x00\x01\x7f\x00\x00\x01\x00\x00')
            self.record('socks5', 403)
            return
        handler.wfile.write(b'\x05\x00\x00\x01\x7f\x00\x00\x01' + struct.pack('!H', self.target_port))
        self.record('socks5', 200)
        self.tunnel(handler.connection)

    def close(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=5)

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()
