"""Owned system-proxy/PAC conflicts, permitted only on disposable hosted Macs."""
import json
import os
from pathlib import Path
import platform
import selectors
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def require_hosted():
    expected = {'CI': 'true', 'GITHUB_ACTIONS': 'true',
                'RUNNER_ENVIRONMENT': 'github-hosted', 'XDZIP_ISOLATED_UPDATE_FIXTURE': '1'}
    if (sys.platform != 'darwin' or platform.machine() != 'arm64' or
            any(os.environ.get(key) != value for key, value in expected.items())):
        raise RuntimeError('System proxy conflicts require an explicitly disposable GitHub-hosted runner')


class OwnedSystemProxy:
    def __init__(self, helper, proxy_port, pac_url):
        require_hosted()
        helper = Path(helper)
        if helper.is_symlink() or not helper.is_file() or helper.name != 'system-proxy-fixture' or not helper.resolve().is_relative_to(ROOT/'build'):
            raise ValueError('Use the freshly compiled owned system proxy helper under build/')
        self.records = []
        self.process = subprocess.Popen([
            'sudo', '-n', '--preserve-env=CI,GITHUB_ACTIONS,RUNNER_ENVIRONMENT,XDZIP_ISOLATED_UPDATE_FIXTURE',
            str(helper), str(proxy_port), pac_url], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, bufsize=0)
        self.pending = b''
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)
        try:
            ready = self.read()
            if ready != {'state': 'ready'}:
                raise RuntimeError('Owned system proxy fixture did not become ready')
        except BaseException:
            self.close()
            raise

    def read(self):
        # Keep explicit byte buffering: TextIOWrapper may read the final restore
        # receipt ahead of select(), which must not hide successful restoration.
        while b'\n' not in self.pending:
            if not self.selector.select(timeout=15):
                raise RuntimeError('Owned system proxy control timed out: '+json.dumps(self.records))
            block = self.process.stdout.read(4096)
            if not block:
                raise RuntimeError('Owned system proxy fixture exited without its receipt: '+json.dumps(self.records))
            self.pending += block
            if len(self.pending) > 16384:
                raise RuntimeError('Owned system proxy receipt exceeded its bound')
        line, self.pending = self.pending.split(b'\n', 1)
        record = json.loads(line)
        self.records.append(record)
        return record

    def apply(self, profile):
        if profile not in ('static', 'pac'):
            raise ValueError('Unknown owned system proxy profile')
        self.process.stdin.write((profile+'\n').encode())
        self.process.stdin.flush()
        if self.read() != {'state': 'applied', 'profile': profile, 'verified': True}:
            raise RuntimeError('Owned system proxy profile was not confirmed')

    def verify(self, profile):
        self.process.stdin.write(b'verify\n')
        self.process.stdin.flush()
        if self.read() != {'state': 'verified', 'profile': profile}:
            raise RuntimeError('Owned system proxy changed during the scenario')

    def close(self):
        if self.process.stdin and not self.process.stdin.closed:
            try:
                self.process.stdin.close()
            except BrokenPipeError:
                pass
        try:
            while self.process.poll() is None or not self.records or self.records[-1].get('state') != 'restored':
                if self.records and self.records[-1].get('state') == 'restored':
                    break
                self.read()
            status = self.process.wait(timeout=15)
            if not self.records or self.records[-1] != {'state': 'restored', 'verified': True}:
                raise RuntimeError('Owned system proxy restoration was not confirmed: '+json.dumps(self.records))
            if status:
                raise RuntimeError('Owned system proxy fixture failed after restoration: '+json.dumps(self.records))
        finally:
            self.selector.close()
            self.process.stdout.close()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()


def compile_helper(output, run):
    require_hosted()
    helper = Path(output)/'system-proxy-fixture'
    run(['xcrun', 'clang', '-fobjc-arc', '-mmacosx-version-min=14.0',
         '-framework', 'Foundation', '-framework', 'SystemConfiguration', '-framework', 'CFNetwork',
         ROOT/'apps/desktop/tests/system_proxy_fixture.m', '-o', helper], 'compile-system-proxy')
    return helper
