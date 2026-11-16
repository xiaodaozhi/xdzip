#!/usr/bin/env python3
"""Serve design sources with the generated UI font on loopback only."""
import argparse
from functools import partial
from pathlib import Path
from html_review_server import ReviewHTTPServer, ReviewHTTPRequestHandler

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--port', type=int, default=8000)
args = parser.parse_args()
server = ReviewHTTPServer(('127.0.0.1', args.port),
                         partial(ReviewHTTPRequestHandler, directory=str(Path(__file__).resolve().parents[1])))
print(f'HTML design: http://127.0.0.1:{server.server_port}/design/html/index.html', flush=True)
try:
    server.serve_forever()
except KeyboardInterrupt:
    pass
finally:
    server.server_close()
