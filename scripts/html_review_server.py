"""Loopback HTTP server for concurrent browser review assets."""
from http.server import ThreadingHTTPServer


class ReviewHTTPServer(ThreadingHTTPServer):
    # Python's default backlog of five can refuse Chromium's parallel asset
    # connections on Windows before the threaded handler accepts them. Keep a
    # bounded queue for the reviewed page's scripts, styles, images and fonts.
    request_queue_size = 64
