#!/usr/bin/env python3
"""Verify logical display size and equal HTML/reference scaling independently."""
from contextlib import contextmanager
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Thread
from urllib.parse import urlparse
import base64
import hashlib
import json
import os

from PIL import Image
from playwright.sync_api import sync_playwright


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "dist/html-scale-review"
LOGICAL_WIDTH = 1280
SOURCE_WINDOW = [20, 30, 1548, 940]
NORMAL_SCALE = LOGICAL_WIDTH / SOURCE_WINDOW[2]
# Approved source paths are independent of the reviewer implementation.
SOURCES = {
    "main": "design/hifi/v1/screens/main-light.png",
    "extract": "design/hifi/v1/screens/extract-macos.png",
    "create": "design/hifi/v2/screens/01-create.png",
    "progress": "design/hifi/v2/screens/03-progress.png",
    "conflict": "design/hifi/v2/screens/02-conflict.png",
    "settings": "design/hifi/v2/screens/05-settings.png",
}
VIEWS = [*SOURCES, "empty", "damaged"]
ZOOMS = ["fit", "1", "pixels"]
TOLERANCE = 0.06  # Chromium layout uses fractional CSS pixels, rounded to 1/64.

METRICS = """() => {
    const rect = element => {
        const r = element.getBoundingClientRect();
        return [r.x, r.y, r.width, r.height];
    };
    const frame = document.querySelector('#design-frame');
    const image = document.querySelector('#reference-image');
    const child = frame.contentDocument;
    return {
        viewportWidth: document.querySelector('#viewport').clientWidth,
        referenceViewportWidth: document.querySelector('#reference-viewport').clientWidth,
        iframe: rect(frame), reference: rect(image),
        wrapper: rect(document.querySelector('#scaled')),
        referenceWrapper: rect(document.querySelector('#reference-scaled')),
        rawFrame: [frame.clientWidth, frame.clientHeight],
        rawImage: [image.width, image.height],
        naturalImage: [image.naturalWidth, image.naturalHeight],
        referenceHidden: image.hidden,
        window: rect(child.querySelector('.window')),
        windowVisible: !child.querySelector('.window').hidden
            && child.defaultView.getComputedStyle(child.querySelector('.stage')).display !== 'none',
        theme: child.documentElement.dataset.theme,
        platform: child.body.dataset.platform,
        standalone: child.body.dataset.standalone,
        view: document.querySelector('#view').value,
        zoom: document.querySelector('#zoom').value,
        dpr: devicePixelRatio,
        documentWidth: document.documentElement.scrollWidth,
        browserWidth: innerWidth,
    };
}"""


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        """Keep HTTP access paths out of test output."""


@contextmanager
def local_server():
    server = ThreadingHTTPServer(("127.0.0.1", 0), partial(QuietHandler, directory=str(ROOT)))
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}"
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


def near(actual, expected, tolerance=TOLERANCE):
    return len(actual) == len(expected) and all(abs(a - e) <= tolerance for a, e in zip(actual, expected))


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    report = {
        "passed": False, "logical_window_width": LOGICAL_WIDTH,
        "source_window": SOURCE_WINDOW, "source_to_logical": NORMAL_SCALE,
        "sources": {}, "checks": [], "failures": [],
        "page_errors": [], "external_requests": [],
        "limits": [
            "CSS logical pixels are independent of devicePixelRatio and physical display pixels.",
            "Equal dimensions do not prove font, color or full visual fidelity.",
            "PNG evidence rounds fractional CSS bounds to whole screenshot pixels.",
        ],
    }

    def check(name, condition, **evidence):
        item = {"name": name, "passed": bool(condition), **evidence}
        report["checks"].append(item)
        if not condition:
            report["failures"].append(item)

    for view, path in SOURCES.items():
        source = ROOT / path
        with Image.open(source) as image:
            size = list(image.size)
        report["sources"][view] = {
            "path": path, "size": size,
            "sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        }
    independent = json.loads((ROOT / "design/html/header-reference.json").read_text())
    check("approved main source hash", report["sources"]["main"]["sha256"] == independent["sha256"])
    check("independent source window", independent["window"] == SOURCE_WINDOW)
    token_width = json.loads((ROOT / "docs/visual/tokens.json").read_text())["size"]["initialWindow"][0]
    check("declared logical width", token_width == LOGICAL_WIDTH, actual=token_width, expected=LOGICAL_WIDTH)
    offline = (ROOT / "dist/xdzip-hifi-design.html").read_text()
    check("offline display contract embedded", "window.XDZIP_DISPLAY_SPEC" in offline)

    def wait_ready(page, view="main", reference=True):
        page.wait_for_function("""() => {
            const frame = document.querySelector('#design-frame');
            return window.XDZIP_DISPLAY_SPEC && frame?.contentDocument?.querySelector('.window')
                && frame.contentWindow.XdzipDesign;
        }""")
        page.evaluate("document.fonts.ready")
        # postMessage configuration and font loading are asynchronous. Wait for
        # observable selected state, rather than a guessed sleep duration.
        page.wait_for_function("""expectedView => {
            const iframe = document.querySelector('#design-frame');
            const child = iframe.contentDocument;
            if (document.querySelector('#view').value !== expectedView || !child?.body) return false;
            const windowView = ['main','empty','damaged'].includes(expectedView);
            return child.body.dataset.platform === document.querySelector('#platform').value
                && child.documentElement.dataset.theme === document.querySelector('#theme').value
                && child.body.dataset.standalone === String(!windowView)
                && (windowView ? !child.querySelector('dialog[open]') : !!child.querySelector('dialog[open]'));
        }""", arg=view)
        frame = next(frame for frame in page.frames if frame.parent_frame)
        frame.evaluate("document.fonts.ready")
        if view in SOURCES and reference:
            page.wait_for_function("""() => {
                const image = document.querySelector('#reference-image');
                return image.complete && image.naturalWidth > 0 && !image.hidden;
            }""")
        return frame

    def verify_spec(page, mode):
        spec = page.evaluate("window.XDZIP_DISPLAY_SPEC")
        check(f"{mode}: logical display contract",
              spec["logicalWindowWidth"] == LOGICAL_WIDTH
              and spec["sourceWindow"] == SOURCE_WINDOW
              and abs(spec["sourceToLogical"] - NORMAL_SCALE) < 1e-12)
        for view, source in report["sources"].items():
            declared = spec["views"][view]
            resolved = (ROOT / "design/html" / declared["source"]).resolve()
            check(f"{mode}: {view} source metadata",
                  declared["size"] == source["size"] and resolved == ROOT / source["path"],
                  expected=source, actual=declared)

    def verify_layout(page, mode, width, dpr, view, zoom, popup=False):
        metrics = page.evaluate(METRICS)
        size = report["sources"][view]["size"] if view in SOURCES else report["sources"]["main"]["size"]
        scale = 1 if zoom == "pixels" else NORMAL_SCALE
        if zoom == "fit":
            scale = min(NORMAL_SCALE, metrics["viewportWidth"] / size[0])
        expected = [size[0] * scale, size[1] * scale]
        prefix = f"{mode}/{width}/DPR{dpr}/{view}/{zoom}" + ("/popup" if popup else "")
        common = {"mode": mode, "width": width, "dpr": dpr, "view": view,
                  "zoom": zoom, "popup": popup, "expected_scale": scale}
        checks = {
            "source canvas size": metrics["rawFrame"] == size,
            "HTML display size": near(metrics["iframe"][2:], expected),
            "HTML wrapper size": near(metrics["wrapper"][2:], expected),
            "HTML origin": near([metrics["iframe"][i] - metrics["wrapper"][i] for i in [0, 1]], [0, 0]),
            "DPR preserved": metrics["dpr"] == dpr,
            "outer document does not overflow": metrics["documentWidth"] <= metrics["browserWidth"],
        }
        if view in SOURCES and not popup:
            checks.update({
                "PNG natural size": metrics["naturalImage"] == size,
                "reference source size": metrics["rawImage"] == size,
                "reference display size": near(metrics["reference"][2:], expected),
                "reference wrapper size": near(metrics["referenceWrapper"][2:], expected),
                "both display sizes equal": near(metrics["iframe"][2:], metrics["reference"][2:]),
                "reference origin": near([metrics["reference"][i] - metrics["referenceWrapper"][i] for i in [0, 1]], [0, 0]),
                "viewport widths equal": metrics["viewportWidth"] == metrics["referenceViewportWidth"],
            })
        if view in ["main", "empty", "damaged"]:
            actual_window = [value * scale for value in metrics["window"]]
            expected_window = [value * scale for value in SOURCE_WINDOW]
            checks["main window logical bounds"] = near(actual_window, expected_window)
            checks["main window visible"] = metrics["windowVisible"] and metrics["standalone"] == "false"
            if zoom == "1":
                checks["normal window width 1280"] = abs(actual_window[2] - LOGICAL_WIDTH) <= TOLERANCE
            if zoom == "pixels":
                checks["raw window width 1548"] = abs(actual_window[2] - SOURCE_WINDOW[2]) <= TOLERANCE
            if view != "main":
                checks["no invented reference"] = metrics["referenceHidden"]
        check(prefix, all(checks.values()), **common, assertions=checks,
              expected_canvas=expected, actual=metrics)

    try:
        with local_server() as base, sync_playwright() as playwright:
            browser = playwright.chromium.launch(
                executable_path=os.environ.get("XDZIP_CHROMIUM", "/usr/bin/chromium"),
                args=["--no-sandbox", "--disable-dev-shm-usage"],
            )
            try:
                for mode in ["served", "offline"]:
                    for width in [390, 1200, 1827]:
                        for dpr in [1, 2]:
                            context = browser.new_context(viewport={"width": width, "height": 1200}, device_scale_factor=dpr)

                            def route_request(route):
                                url = route.request.url
                                if mode == "served" and url.startswith(base + "/"):
                                    route.continue_()
                                elif urlparse(url).scheme in ["http", "https", "file"]:
                                    report["external_requests"].append({"mode": mode, "scheme": urlparse(url).scheme, "host": urlparse(url).hostname})
                                    route.abort()
                                else:
                                    route.continue_()

                            context.route("**/*", route_request)
                            context.on("page", lambda page: page.on("pageerror", lambda error: report["page_errors"].append(str(error))))
                            page = context.new_page()
                            if mode == "served":
                                page.goto(base + "/design/html/index.html")
                            else:
                                page.set_content(offline)
                            wait_ready(page)
                            if width == 390:
                                check(f"{mode}/DPR{dpr}: mobile defaults to fit", page.input_value("#zoom") == "fit")
                            else:
                                check(f"{mode}/{width}/DPR{dpr}: desktop defaults to normal", page.input_value("#zoom") == "1")
                            if width == 1827 and dpr == 1:
                                verify_spec(page, mode)
                            page.click("#reference-toggle")
                            for view in VIEWS:
                                page.select_option("#view", view)
                                wait_ready(page, view)
                                for zoom in ZOOMS:
                                    page.select_option("#zoom", zoom)
                                    page.wait_for_timeout(20)
                                    verify_layout(page, mode, width, dpr, view, zoom)
                            # Scroll paths must remain synchronized in both directions.
                            page.select_option("#view", "main")
                            page.select_option("#zoom", "pixels")
                            if width == 390:
                                for source, target in [("viewport", "reference-viewport"), ("reference-viewport", "viewport")]:
                                    page.eval_on_selector(f"#{source}", "element => element.scrollLeft = 300")
                                    page.wait_for_function("document.querySelector('#viewport').scrollLeft === 300 && document.querySelector('#reference-viewport').scrollLeft === 300")
                                    values = page.evaluate("[document.querySelector('#viewport').scrollLeft, document.querySelector('#reference-viewport').scrollLeft]")
                                    check(f"{mode}/DPR{dpr}: {source} horizontal scroll synchronized", near(values, [300, 300]), actual=values)
                                    page.eval_on_selector(f"#{target}", "element => element.scrollLeft = 0")
                                    page.wait_for_function("document.querySelector('#viewport').scrollLeft === 0 && document.querySelector('#reference-viewport').scrollLeft === 0")
                                page.click("#reference-toggle")
                                page.eval_on_selector("#viewport", "element => element.scrollLeft = 450")
                                page.wait_for_function("document.querySelector('#viewport').scrollLeft === 450")
                                page.click("#reference-toggle")
                                page.wait_for_function("document.querySelector('#reference-viewport').scrollLeft === 450")
                                values = page.evaluate("[document.querySelector('#viewport').scrollLeft, document.querySelector('#reference-viewport').scrollLeft]")
                                check(f"{mode}/DPR{dpr}: opening reference preserves existing horizontal position",
                                      near(values, [450, 450]), actual=values)
                            # A selected main view must stay the main view in a new window.
                            if width == 1200:
                                page.select_option("#platform", "windows")
                                page.select_option("#theme", "dark")
                                for zoom in ZOOMS:
                                    page.select_option("#zoom", zoom)
                                    with page.expect_popup() as popup_event:
                                        page.click("#separate")
                                    popup = popup_event.value
                                    child = wait_ready(popup, reference=False)
                                    selections = popup.evaluate("Object.fromEntries(['view','platform','theme','zoom'].map(id => [id,document.querySelector('#'+id).value]))")
                                    check(f"{mode}/DPR{dpr}/{zoom}: independent main preserves configuration",
                                          selections == {"view": "main", "platform": "windows", "theme": "dark", "zoom": zoom}
                                          and child.locator("dialog[open]").count() == 0
                                          and child.evaluate("document.documentElement.dataset.theme") == "dark"
                                          and child.evaluate("document.body.dataset.platform") == "windows", actual=selections)
                                    verify_layout(popup, mode, popup.viewport_size["width"], dpr, "main", zoom, popup=True)
                                    # Reuse the same named window without closing it first.
                                    page.select_option("#platform", "linux")
                                    page.select_option("#theme", "light")
                                    page.click("#separate")
                                    popup.wait_for_function("document.querySelector('#platform')?.value === 'linux'")
                                    child = wait_ready(popup, reference=False)
                                    check(f"{mode}/DPR{dpr}/{zoom}: named window can be reopened",
                                          child.locator("dialog[open]").count() == 0
                                          and child.evaluate("document.documentElement.dataset.theme") == "light"
                                          and child.evaluate("document.body.dataset.platform") == "linux")
                                    popup.close()
                                    page.select_option("#platform", "windows")
                                    page.select_option("#theme", "dark")
                            # Screenshots use identical browser scaling for the two layers.
                            if mode == "offline" and width == 1827 and dpr == 1:
                                page.select_option("#view", "main")
                                page.select_option("#zoom", "1")
                                page.select_option("#platform", "macos")
                                page.select_option("#theme", "light")
                                wait_ready(page)
                                page.locator("#design-frame").screenshot(path=str(OUTPUT / "main-normal.png"))
                                page.locator("#reference-image").screenshot(path=str(OUTPUT / "reference-normal.png"))
                                with Image.open(OUTPUT / "main-normal.png") as actual, Image.open(OUTPUT / "reference-normal.png") as source:
                                    check("normal screenshot layers have equal pixel dimensions", actual.size == source.size,
                                          actual=list(actual.size), reference=list(source.size))
                                report["normal_evidence"] = {"source_canvas_css": [1586 * NORMAL_SCALE, 992 * NORMAL_SCALE],
                                                             "window_css": [1280, 940 * NORMAL_SCALE],
                                                             "screenshot_px": list(actual.size)}
                            context.close()
            finally:
                browser.close()
    except Exception as error:
        report["failures"].append({"name": "checker execution", "error": str(error)})
    check("no page JavaScript errors", not report["page_errors"], actual=report["page_errors"])
    check("no external network requests", not report["external_requests"], actual=report["external_requests"])
    report["passed"] = not report["failures"]
    (OUTPUT / "validation.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    (OUTPUT / "validation.js").write_text("window.XDZIP_SCALE_REPORT=" + json.dumps(report, ensure_ascii=False) + ";\n")
    template = ROOT / "docs/qa/html-scale/index.html"
    if all((OUTPUT / name).exists() for name in ["main-normal.png", "reference-normal.png"]):
        images = {key: "data:image/png;base64," + base64.b64encode((OUTPUT / name).read_bytes()).decode()
                  for key, name in {"html": "main-normal.png", "reference": "reference-normal.png"}.items()}
        data = "window.XDZIP_SCALE_REPORT=" + json.dumps(report, ensure_ascii=False) + ";"
        data += "window.XDZIP_SCALE_IMAGES=" + json.dumps(images) + ";"
        bundled = template.read_text().replace("<script>", "<script>" + data, 1)
        (ROOT / "dist/xdzip-scale-review.html").write_text(bundled)
    passed = sum(item["passed"] for item in report["checks"])
    print(f"{passed}/{len(report['checks'])} display-scale cases passed; report: {OUTPUT / 'validation.json'}")
    if report["failures"]:
        for failure in report["failures"][:8]:
            print(json.dumps(failure, ensure_ascii=False))
        raise SystemExit(1)


if __name__ == "__main__":
    main()
