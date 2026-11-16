"""Loopback HTTP server for concurrent browser review assets."""
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import unquote, urlsplit
from ui_fonts import DEFAULT_OUTPUT, FONT_NAME, generate


class ReviewHTTPRequestHandler(SimpleHTTPRequestHandler):
    def translate_path(self, path):
        requested = unquote(urlsplit(path).path)
        for suffix in ('.woff2', '.ttf'):
            if requested == '/design/assets/fonts/' + FONT_NAME + suffix:
                return str(self.server.font_directory / (FONT_NAME + suffix))
        return super().translate_path(path)


class ReviewHTTPServer(ThreadingHTTPServer):
    # Python's default backlog of five can refuse Chromium's parallel asset
    # connections on Windows before the threaded handler accepts them. Keep a
    # bounded queue for the reviewed page's scripts, styles, images and fonts.
    request_queue_size = 64

    def __init__(self, *args, font_directory=DEFAULT_OUTPUT, **kwargs):
        generate(font_directory)
        self.font_directory = font_directory
        super().__init__(*args, **kwargs)
