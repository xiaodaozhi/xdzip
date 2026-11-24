#!/usr/bin/env python3
"""Check the hosted-only native proxy fixture can start and restore state."""
import subprocess

from system_proxy_fixture import ROOT, OwnedSystemProxy, compile_helper, require_hosted


def run(arguments, _name):
    subprocess.run(list(map(str, arguments)), check=True)


def main():
    require_hosted()
    output = ROOT/'build/update-regression/system-proxy-preflight'
    output.mkdir(parents=True, exist_ok=True)
    helper = compile_helper(output, run)
    with OwnedSystemProxy(helper, 1234, 'http://127.0.0.1:1234/proxy.pac') as fixture:
        if fixture.records != [{'state': 'ready'}]:
            raise RuntimeError('Native system proxy fixture did not become ready')
    if fixture.records[-1] != {'state': 'restored', 'verified': True}:
        raise RuntimeError('Native system proxy fixture did not restore its state')
    print('Hosted native system proxy fixture started and restored state')


if __name__ == '__main__':
    main()
