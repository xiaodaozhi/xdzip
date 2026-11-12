#!/usr/bin/env python3
"""Observe native ImageIO decoding independently of Qt and product resource limits."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if platform.system() != 'Darwin' or not output.is_relative_to(ROOT/'build'):
        parser.error('Use native macOS and keep observations under build/')
    output.parent.mkdir(parents=True, exist_ok=True)
    env = {k: v for k, v in os.environ.items()
           if not k.startswith(('QT_', 'QML_', 'DYLD_', 'XDZIP_'))}
    records = []
    with tempfile.TemporaryDirectory(prefix='imageio-', dir=output.parent) as temp:
        for name in ('owned-wide.heic', 'owned-phone.heic'):
            source = ROOT/'tests/fixtures/preview'/name
            target = Path(temp)/(source.stem+'.png')
            result = subprocess.run(['/usr/bin/sips', '--resampleHeightWidthMax', '512',
                                     '-s', 'format', 'png', str(source), '--out', str(target)],
                                    capture_output=True, text=True, timeout=20, env=env)
            dimensions = None
            if target.is_file():
                header = target.read_bytes()[:24]
                if len(header) == 24 and header.startswith(b'\x89PNG\r\n\x1a\n'):
                    dimensions = list(struct.unpack('>II', header[16:24]))
            available = (result.returncode == 0 and dimensions is not None
                         and all(0 < n <= 512 for n in dimensions))
            diagnostic = (result.stdout+'\n'+result.stderr)
            for private, label in ((str(ROOT), '<workspace>'), (temp, '<temporary>'),
                                   (str(Path.home()), '<home>')):
                diagnostic = diagnostic.replace(private, label)
            records.append({'fixture': name, 'sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
                            'system_decode_available': available, 'sips_exit_code': result.returncode,
                            'png_dimensions': dimensions, 'diagnostic': diagnostic.strip()[:4096]})
    record = {'schema': 1, 'status': 'observed', 'macos': platform.mac_ver()[0], 'inputs': records,
              'scope': 'Owned native sips/ImageIO control without Qt or product limits; not a product test pass.'}
    output.write_text(json.dumps(record, indent=2)+'\n')
    print(json.dumps(record))


if __name__ == '__main__':
    main()
