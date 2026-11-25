"""Prepare exact native dependencies for the candidate and formal Release entry points."""
import argparse
from functools import partial
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
from time import perf_counter
from urllib.error import URLError
import zipfile

from ci_dependency_artifact import api, fetch_release_bundle
import qt_image_bundle as images
import qt_image_security as image_build
import qt_network_bundle as network
import qt_network_security as network_build
import sparkle_reuse as sparkle
from windows_ninja_safety import utf8_console


ROOT = network_build.ROOT
RUNNERS = {('win32', 'x86_64'): 'windows-2022', ('darwin', 'arm64'): 'macos-15',
           ('darwin', 'x86_64'): 'macos-15-intel', ('linux', 'x86_64'): 'ubuntu-24.04'}
REJECTIONS = (OSError, ValueError, KeyError, TypeError, AttributeError, URLError, subprocess.SubprocessError,
              zipfile.BadZipFile, tarfile.TarError, EOFError)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True)+'\n', encoding='utf-8')


def start_component(args, runner, token, label, key, filename, maximum):
    start = perf_counter()
    incoming = args.output/'incoming'/filename
    incoming.unlink(missing_ok=True)
    receipt = {'key': key, 'fetch': {'status': 'forced-cold' if args.mode == 'cold' else 'miss'}}
    if args.mode == 'auto' and token:
        try:
            receipt['fetch'] = fetch_release_bundle(partial(api, token=token), key, incoming, runner,
                                                    'Preserve verified '+label, maximum)
        except REJECTIONS as error:
            receipt['fetch'] = {'status': 'fetch-rejected', 'error_type': type(error).__name__}
    receipt['fetch_seconds'] = round(perf_counter()-start, 3)
    return incoming, receipt


def finish_component(receipt, status, start):
    receipt.update(status=status, prepare_seconds=round(perf_counter()-start, 3))
    print(json.dumps(receipt), flush=True)


def prepare_network(args, runner, token, expected):
    prefix, output = args.qt_prefix, args.output
    filename = network.BUNDLE_NAME
    incoming, receipt = start_component(args, runner, token, 'Qt Network', network.key(expected),
                                        filename, network.MAX_LIBRARY_BYTES + 2*1024*1024)
    if args.mode == 'auto' and token and not incoming.is_file() and runner != 'ubuntu-24.04':
        # Reuse existing verified update-regression artifacts when they fit this native SDK.
        start = perf_counter()
        manifest = output/'network-inputs.json'
        write_json(manifest, expected)
        subprocess.run([sys.executable, ROOT/'scripts/ci-qt-network.py', 'fetch', '--inputs', manifest,
                        '--output', incoming.parent, '--mode', 'auto'], check=True,
                       env={**os.environ, 'GITHUB_TOKEN': token})
        receipt['update_fetch'] = json.loads((incoming.parent/'fetch.json').read_bytes())
        receipt['fetch_seconds'] += round(perf_counter()-start, 3)
    start = perf_counter()
    if (prefix/network_build.RECEIPT).exists():
        raise ValueError('Release dependency reuse requires a stock Qt Network SDK')
    work, cache = output/'network', ROOT/'build/source-cache'
    status = 'cold'
    target = output/filename if args.publish else None
    if incoming.is_file():
        try:
            result = network_build.prepare(prefix, work/'restore', cache, bundle=incoming, bundle_identity=expected)
        except REJECTIONS as error:
            if (prefix/network_build.RECEIPT).exists() or network.inputs(prefix) != expected:
                raise ValueError('Qt Network rollback did not restore the stock SDK') from error
            status = 'rejected-cold'
        else:
            if target:
                shutil.copyfile(incoming, target)
            finish_component(receipt, 'hit', start)
            return {**receipt, 'sha256': result['network_sha256']}
    result = network_build.prepare(prefix, work/'source', cache, export=target, bundle_identity=expected)
    finish_component(receipt, status, start)
    return {**receipt, 'sha256': result['network_sha256']}


def prepare_images(args, runner, token, expected):
    prefix, output = args.qt_prefix, args.output
    filename = 'qt-image.zip'
    incoming, receipt = start_component(args, runner, token, 'Qt ImageFormats', images.key(expected),
                                        filename, 2*network.MAX_LIBRARY_BYTES + 2*1024*1024)
    start = perf_counter()
    status = 'cold'
    work = output/'images'
    if incoming.is_file():
        def restore(sdk, directory, _parallel):
            return images.restore(incoming, sdk, directory, expected)

        try:
            result = image_build.prepare(prefix, work, 6, build_modules=restore)
        except REJECTIONS as error:
            if images.inputs(prefix, expected) != expected:
                raise ValueError('Qt image module rollback did not restore the stock SDK') from error
            status = 'rejected-cold'
        else:
            status = 'hit'
    if status != 'hit':
        result = image_build.prepare(prefix, work, 6)
    finish_component(receipt, status, start)
    if args.probe:
        receipt['decode_seconds'] = images.check_decode(prefix, work)
    if args.publish:
        images.pack(output/filename, prefix, expected)
    return {**receipt, 'sha256': result['module_sha256']}


def prepare_sparkle(args, runner, token, expected):
    filename = 'sparkle-sdk.tar.gz'
    incoming, receipt = start_component(args, runner, token, 'Sparkle', sparkle.key(expected),
                                        filename, 128*1024*1024)
    variant = expected['variant']
    sdk = ROOT/'build/toolchain/Sparkle'/variant
    start = perf_counter()
    status = 'cold'
    if sdk.exists():
        sparkle.verify(sdk, expected)
        status = 'local'
    elif incoming.is_file():
        try:
            sparkle.restore(incoming, sdk, expected)
        except REJECTIONS:
            status = 'rejected-cold'
        else:
            status = 'hit'
    if status in ('cold', 'rejected-cold'):
        network_build.run([sys.executable, ROOT/'scripts/prepare-sparkle.py', '--output', sdk],
                          args.output/'sparkle-prepare.log')
        sparkle.verify(sdk, expected)
    finish_component(receipt, status, start)
    if args.publish:
        sparkle.pack(args.output/filename, sdk, expected)
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--mode', choices=('auto', 'cold'), required=True)
    parser.add_argument('--probe', action='store_true')
    parser.add_argument('--publish', action='store_true')
    args = parser.parse_args()
    if args.publish and not args.probe:
        parser.error('Publish dependencies from the native dependency probe')
    args.qt_prefix, args.output = args.qt_prefix.resolve(), args.output.resolve()
    if args.output == ROOT/'build' or not args.output.is_relative_to(ROOT/'build'):
        parser.error('Use an owned output directory under build/')
    args.output.mkdir(parents=True, exist_ok=True)
    token = os.environ.pop('GITHUB_TOKEN', '')
    net = network.inputs(args.qt_prefix)
    runner = RUNNERS[net['os'], net['arch']]
    img = images.inputs(args.qt_prefix, net)
    record = {'revision': os.environ.get('GITHUB_SHA', ''), 'runner': runner}
    record['network'] = prepare_network(args, runner, token, net)
    record['images'] = prepare_images(args, runner, token, img)
    if net['os'] == 'darwin':
        record['sparkle'] = prepare_sparkle(args, runner, token, sparkle.inputs(net))
    write_json(args.output/'dependencies.json', record)
    if args.publish and (github_output := os.environ.get('GITHUB_OUTPUT')):
        with Path(github_output).open('a', encoding='utf-8') as stream:
            for component in ('network', 'images', 'sparkle'):
                if component in record:
                    print(f'{component}={record[component]["key"]}', file=stream)


if __name__ == '__main__':
    with utf8_console():
        main()
