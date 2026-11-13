#!/usr/bin/env python3
"""Record package signature and staged origin without inferring the install channel."""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
ORIGINS = {
    0: 'Unknown', 1: 'Unsigned', 2: 'Inbox', 3: 'Store',
    4: 'DeveloperUnsigned', 5: 'DeveloperSigned', 6: 'LineOfBusiness',
}
SCENARIOS = ('unspecified', 'store-install', 'copied-sideload', 'self-signed-sideload',
             'upgrade', 'other-user')


def installed_package(name):
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9.-]{2,49}', name):
        raise ValueError('Invalid package name')
    # The name is passed via the environment, never interpolated into PowerShell.
    command = r'''
$ErrorActionPreference = 'Stop'
$items = @(Get-AppxPackage -Name $env:XDZIP_PROBE_NAME |
    Where-Object { $_.Name -ceq $env:XDZIP_PROBE_NAME } |
    ForEach-Object { @{ full_name = $_.PackageFullName; signature = $_.SignatureKind.ToString() } })
ConvertTo-Json -InputObject $items -Compress
'''
    result = subprocess.run(
        ['powershell.exe', '-NoProfile', '-NonInteractive', '-Command', command],
        env={**os.environ, 'XDZIP_PROBE_NAME': name}, capture_output=True,
        text=True, encoding='utf-8', timeout=60, check=False)
    if result.returncode:
        raise ValueError('Could not query the current user package registration')
    try:
        packages = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise ValueError('Invalid package query result') from error
    if (not isinstance(packages, list) or len(packages) != 1 or
            not isinstance(packages[0], dict) or
            not isinstance(packages[0].get('full_name'), str) or
            not packages[0]['full_name'] or
            not isinstance(packages[0].get('signature'), str)):
        raise ValueError('Expected exactly one current-user package registration')
    return packages[0]


def staged_origin(full_name):
    function = ctypes.WinDLL('kernelbase', use_last_error=True).GetStagedPackageOrigin
    function.argtypes = (ctypes.c_wchar_p, ctypes.POINTER(ctypes.c_int))
    function.restype = ctypes.c_long
    origin = ctypes.c_int(-1)
    status = function(full_name, ctypes.byref(origin))
    return status, origin.value if status == 0 else None


def observe(name, scenario, *, package_query=installed_package, origin_query=staged_origin):
    if scenario not in SCENARIOS:
        raise ValueError('Unknown scenario label')
    package = package_query(name)
    status, value = origin_query(package['full_name'])
    return {
        'schema_version': 1,
        'scenario_label': scenario,
        'scenario_label_is_evidence': False,
        'package_full_name_sha256': hashlib.sha256(package['full_name'].encode('utf-8')).hexdigest(),
        'signature_kind': package['signature'],
        'staged_origin_api_status': status,
        'staged_origin_value': value,
        'staged_origin_name': ORIGINS.get(value, 'Unrecognized') if value is not None else None,
        'os_version': platform.version(),
        'os_architecture': platform.machine(),
        'limits': [
            'The package lookup is scoped to the current user.',
            'The staged origin is not established as this user installation channel.',
            'The scenario is a caller label, not proof of how the package was installed.',
            'This observation does not authorize a context menu or classify a Store installation.',
        ],
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--name', required=True, help='Exact package identity name registered for the current user')
    parser.add_argument('--scenario', choices=SCENARIOS, default='unspecified')
    parser.add_argument('--output', type=Path, required=True, help='New JSON evidence file below build/')
    args = parser.parse_args(argv)
    if os.name != 'nt':
        raise ValueError('This read-only observation requires Windows')
    output = args.output.resolve()
    build = (ROOT / 'build').resolve()
    if not output.is_relative_to(build) or output == build or output.suffix.lower() != '.json':
        raise ValueError('Use a dedicated JSON file below build/')
    if not output.parent.is_dir():
        raise ValueError('Output parent must already exist')
    report = observe(args.name, args.scenario)
    with output.open('x', encoding='utf-8') as target:
        json.dump(report, target, indent=2)
        target.write('\n')
    print('Recorded package observation; installation channel is not established')


if __name__ == '__main__':
    main()
