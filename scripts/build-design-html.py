#!/usr/bin/env python3
"""Build offline xdzip design pages and one self-contained HTML document.

Requires Python 3.10+, lxml, pandoc and Graphviz dot. The two supported Mermaid
forms are deliberately parsed strictly: unsupported syntax fails the build
instead of silently producing a different architecture diagram.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import html as escaping
import json
import os
from pathlib import Path
import re
import subprocess
from urllib.parse import unquote, urlsplit
import zipfile

from lxml import etree, html


PROJECT = Path(__file__).resolve().parents[1]
SOURCE = PROJECT / "docs" / "design"
TEMPLATES = Path(__file__).resolve().parent / "design-html"
ORDER = ["index"] + sorted(p.stem for p in SOURCE.glob("[0-9][0-9]-*.md")) + [
    "architecture-decisions", "requirements-traceability", "glossary"
]
STATE_NAMES = {
    "Queued": "排队", "Preparing": "准备", "Running": "执行",
    "WaitingForUser": "等待用户", "Committing": "提交",
    "Cancelling": "正在取消", "CleaningUp": "收尾清理",
    "Succeeded": "成功", "Failed": "失败", "Cancelled": "已取消",
}


def text_command(arguments: list[str], *, data: str | None = None) -> str:
    """Run tools without a shell; failures include diagnostics and stop output."""
    result = subprocess.run(arguments, input=data, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f"{arguments[0]} failed: {result.stderr.strip()}")
    return result.stdout


def serialize(element: etree._Element) -> str:
    return etree.tostring(element, encoding="unicode", method="html")


def diagram_svg(source: str, identifier: str) -> str:
    lines = [line.strip() for line in source.splitlines() if line.strip()]
    nodes: dict[str, str] = {}
    edges: list[tuple[str, str, str]] = []
    if lines[0] == "flowchart TB":
        node = r"([A-Za-z][A-Za-z0-9_]*)(?:\[([^\]]+)\])?"
        for line in lines[1:]:
            match = re.fullmatch(node + r" --> " + node, line)
            if not match:
                raise ValueError(f"Unsupported Mermaid flow syntax: {line}")
            left, left_label, right, right_label = match.groups()
            if left_label:
                nodes[left] = left_label
            if right_label:
                nodes[right] = right_label
            edges.append((left, right, ""))
        for left, right, _ in edges:
            if left not in nodes or right not in nodes:
                raise ValueError("Unlabelled flow node")
    elif lines[0] == "stateDiagram-v2":
        for line in lines[1:]:
            match = re.fullmatch(r"(\[\*\]|\w+) --> (\[\*\]|\w+)(?:: (.+))?", line)
            if not match:
                raise ValueError(f"Unsupported Mermaid state syntax: {line}")
            left, right, label = match.groups()
            left = "_start" if left == "[*]" else left
            right = "_end" if right == "[*]" else right
            for name in (left, right):
                if not name.startswith("_") and name not in STATE_NAMES:
                    raise ValueError(f"Unknown task state: {name}")
                nodes[name] = STATE_NAMES.get(name, "") + ("\n" + name if not name.startswith("_") else "")
            edges.append((left, right, label or ""))
    else:
        raise ValueError("Unsupported Mermaid diagram kind")

    quote = lambda value: json.dumps(value, ensure_ascii=False)
    dot = [
        "digraph G {",
        'graph [rankdir=TB, bgcolor="transparent", pad="0.3", nodesep="0.35", ranksep="0.6"];',
        'node [shape=box, style="rounded,filled", fillcolor="#edf2ff", color="#adbee8", fontcolor="#26354d", fontname="Noto Sans CJK SC", fontsize=12, margin="0.18,0.12"];',
        'edge [color="#6b81b8", fontcolor="#465878", fontname="Noto Sans CJK SC", fontsize=10, arrowsize=0.65];',
    ]
    for name, label in nodes.items():
        attributes = f"label={quote(label)}"
        if name.startswith("_"):
            attributes += ', shape=circle, width=0.12, height=0.12, fillcolor="#3158c8"'
        elif name in ("Succeeded", "Failed", "Cancelled"):
            color = {"Succeeded": "#e1f3e9", "Failed": "#fbe7e7", "Cancelled": "#fff0d9"}[name]
            attributes += f", fillcolor={quote(color)}"
        dot.append(f"{quote(name)} [{attributes}];")
    for left, right, label in edges:
        dot.append(f"{quote(left)} -> {quote(right)} [label={quote(label)}];")
    dot.append("}")
    raw = text_command(["dot", "-Tsvg"], data="\n".join(dot))
    parser = etree.XMLParser(resolve_entities=False, no_network=True)
    root = etree.fromstring(raw.encode(), parser)
    # Inline SVG IDs must remain unique in the combined document.
    for element in root.iter():
        if element.get("id"):
            element.set("id", identifier + "-" + element.get("id"))
    root.set("role", "img")
    root.set("aria-label", "系统架构图" if lines[0].startswith("flowchart") else "任务状态机图")
    return etree.tostring(root, encoding="unicode")


def prepare_document(stem: str, output: Path) -> tuple[etree._Element, list[Path]]:
    markdown = (SOURCE / f"{stem}.md").read_text()
    fragment = text_command([
        "pandoc", "--from=markdown", "--to=html5", "--wrap=none",
    ], data=markdown)
    article = html.fragment_fromstring(fragment, create_parent="article")
    article.set("class", "document")
    article.set("id", "doc-" + stem)
    generated = []
    for number, pre in enumerate(article.xpath('.//pre[contains(concat(" ", @class, " "), " mermaid ")]'), 1):
        code = pre.find("code")
        if code is None:
            raise ValueError("Mermaid code missing")
        identifier = f"diagram-{stem}-{number}"
        svg = diagram_svg(code.text_content(), identifier)
        path = output / "assets" / (identifier + ".svg")
        path.write_text(svg)
        generated.append(path)
        figure = html.Element("figure")
        panel = html.Element("div", {"class": "diagram-panel"})
        panel.append(html.Element("img", src="assets/" + path.name, alt="系统架构图" if stem.startswith("02") else "任务状态机图"))
        figure.append(panel)
        caption = html.Element("figcaption")
        caption.text = "离线矢量图；可横向滚动查看细节。 "
        link = html.Element("a", href="assets/" + path.name, target="_blank", rel="noopener")
        link.text = "单独打开图表"
        caption.append(link)
        figure.append(caption)
        detail = html.Element("details")
        summary = html.Element("summary")
        summary.text = "查看 Mermaid 图表源码"
        detail.append(summary)
        source_pre = copy.deepcopy(pre)
        source_pre.set("class", "diagram-source")
        detail.append(source_pre)
        figure.append(detail)
        pre.getparent().replace(pre, figure)
    for table in article.xpath(".//table"):
        wrapper = html.Element("div", {"class": "table-wrap"})
        table.getparent().replace(table, wrapper)
        wrapper.append(table)
    return article, generated


def remap(article: etree._Element, stem: str, *, combined: bool, output: Path) -> etree._Element:
    tree = copy.deepcopy(article)
    original_ids = {e.get("id") for e in tree.iter() if e.get("id")}
    for anchor in tree.xpath(".//a[@href]"):
        target = urlsplit(anchor.get("href"))
        if target.path.endswith(".md") and not target.scheme:
            target_stem = Path(target.path).stem
            resolved = (SOURCE / target.path).resolve()
            if resolved.parent != SOURCE or target_stem not in ORDER:
                if not resolved.is_file():
                    raise ValueError(f"Missing supplementary document {target.path}")
                relative = resolved.relative_to(PROJECT)
                anchor.set("href", "https://github.com/xiaodaozhi/xdzip/blob/main/" + relative.as_posix()
                           + ("#" + target.fragment if target.fragment else ""))
                continue
            fragment = unquote(target.fragment)
            anchor.set("href", ("#doc-" + target_stem + ("-" + fragment if fragment else "")) if combined
                       else target_stem + ".html" + ("#" + fragment if fragment else ""))
        elif combined and not target.path and target.fragment in original_ids:
            anchor.set("href", "#doc-" + stem + "-" + target.fragment)
    if combined:
        for element in tree.iter():
            identifier = element.get("id")
            if identifier and element is not tree:
                element.set("id", "doc-" + stem + "-" + identifier)
        for panel in tree.xpath('.//div[@class="diagram-panel"]'):
            image = panel.find("img")
            if image is None:
                continue
            svg = etree.fromstring((output / image.get("src")).read_bytes())
            panel.replace(image, svg)
        for figure in tree.xpath(".//figure"):
            caption = figure.find("figcaption")
            if caption is not None:
                caption.clear()
                caption.text = "离线矢量图；可横向滚动查看细节。图表已内嵌在此文件中。"
    for heading in tree.xpath(".//h2 | .//h3"):
        link = html.Element("a", {"class": "heading-anchor", "href": "#" + heading.get("id"), "aria-label": "本节链接"})
        link.text = "#"
        heading.append(link)
    return tree


def search_records(documents: dict[str, etree._Element], *, combined: bool) -> list[dict]:
    records = []
    for stem, article in documents.items():
        title = article.find("h1").text_content().removeprefix("xdzip：")
        sections: list[tuple[str, str, list[str]]] = [("概览", "", [])]
        for child in article:
            if child.tag == "h2":
                sections.append((child.text_content(), child.get("id"), []))
            elif child.tag != "h1":
                sections[-1][2].append(child.text_content())
        for name, fragment, chunks in sections:
            text = re.sub(r"\s+", " ", " ".join(chunks)).strip()
            if not text:
                continue
            href = ("#doc-" + stem + ("-" + fragment if fragment else "")) if combined else stem + ".html" + ("#" + fragment if fragment else "")
            records.append({"title": title, "section": name, "text": text, "href": href, "order": len(records)})
    return records


def json_script(records: list[dict]) -> str:
    # Prevent a source excerpt from terminating the inline script element.
    return "window.XDZIP_SEARCH_INDEX = " + json.dumps(records, ensure_ascii=False).replace("<", "\\u003c") + ";\n"


def shell(title: str, body: str, documents: dict[str, etree._Element], *, current: str,
          combined: bool, toc: list[tuple[str, str]], css: str, js: str, records: list[dict]) -> str:
    escape = escaping.escape
    nav = []
    for number, (stem, article) in enumerate(documents.items()):
        label = article.find("h1").text_content().removeprefix("xdzip：")
        badge = "00" if stem == "index" else (stem[:2] if stem[0].isdigit() else {"architecture-decisions": "ADR", "requirements-traceability": "REQ", "glossary": "REF"}[stem])
        label = re.sub(r"^\d{2} ", "", label)
        href = "#doc-" + stem if combined else stem + ".html"
        active = ' aria-current="page"' if stem == current and not combined else ""
        nav.append(f'<a class="nav-link" href="{escape(href)}"{active}><span class="nav-number">{badge}</span><span>{escape(label)}</span></a>')
    navigation = ""
    if not combined:
        position = ORDER.index(current)
        neighbors = []
        if position:
            previous = ORDER[position - 1]
            neighbors.append(f'<a href="{previous}.html">← {escape(documents[previous].find("h1").text_content().removeprefix("xdzip："))}</a>')
        if position + 1 < len(ORDER):
            following = ORDER[position + 1]
            neighbors.append(f'<a href="{following}.html">{escape(documents[following].find("h1").text_content().removeprefix("xdzip："))} →</a>')
        navigation = '<nav class="page-navigation" aria-label="前后文档">' + "".join(neighbors) + "</nav>"
    styles = "<style>" + css + "</style>" if combined else '<link rel="stylesheet" href="assets/style.css">'
    scripts = ("<script>" + json_script(records) + js + "</script>") if combined else '<script src="assets/search-index.js"></script><script src="assets/app.js"></script>'
    toc_html = "".join(f'<a href="#{escape(identifier)}">{escape(label)}</a>' for identifier, label in toc)
    alternate = '<a class="button" href="index.html">分篇阅读</a>' if combined else '<a class="button" href="all.html">完整文档</a>'
    # A self-contained file must not retain a dependency on index.html.
    if combined:
        alternate = ""
    return f'''<!doctype html>
<html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="description" content="xdzip Qt Quick + Rust 软件设计文档，支持离线阅读与搜索。">
<title>{escape(title)}，xdzip 软件设计</title>{styles}</head>
<body><a class="skip-link" href="#main">跳至正文</a>
<header class="topbar"><button class="button menu-button" id="menu-toggle" aria-label="展开文档目录" aria-expanded="false" aria-controls="sidebar">目录</button>
<a class="brand" href="{'#doc-index' if combined else 'index.html'}">xd<span>zip</span></a><span class="top-label">软件设计，基线 0.1，16 篇</span>
<div class="top-actions">{alternate}<button class="button" id="open-search" aria-haspopup="dialog">搜索<kbd>⌘ / Ctrl K</kbd></button><button class="button" id="theme-toggle">深色</button></div></header>
<button class="mobile-shade" id="mobile-shade" aria-label="关闭目录"></button>
<div class="layout"><nav class="sidebar" id="sidebar" aria-label="文档目录"><p class="nav-label">SOFTWARE DESIGN</p>{''.join(nav)}<p class="sidebar-note">Qt Quick / QML + C++ + Rust<br>离线可用，无外部资源<br>设计文档，产品尚待实现</p></nav>
<main class="content" id="main" tabindex="-1">{body}{navigation}<footer class="footer">xdzip 软件设计，2026-10-11，基线 0.1</footer></main>
<aside class="toc" aria-label="本页目录"><div class="toc-title">{'完整文档目录' if combined else '本页目录'}</div>{toc_html}</aside></div>
<dialog class="search-dialog" id="search-dialog" aria-labelledby="search-title"><div class="search-header"><div class="search-title"><h2 id="search-title">搜索全部设计文档</h2><button class="button" id="close-search" aria-label="关闭搜索">关闭</button></div>
<label class="nav-label" for="search-input">关键词</label><input class="search-input" id="search-input" type="search" autocomplete="off" placeholder="例如：路径安全、取消、格式扩展">
<p class="search-status" id="search-status" role="status" aria-live="polite"></p></div><div class="search-results" id="search-results"></div></dialog>
{scripts}</body></html>'''


def validate(output: Path, pages: list[Path]) -> dict:
    parsed = {p: html.fromstring(p.read_text()) for p in pages}
    links = 0
    for path, tree in parsed.items():
        ids = tree.xpath("//@id")
        if len(ids) != len(set(ids)):
            raise ValueError(f"Duplicate HTML IDs: {path.name}")
        for target in tree.xpath("//@href | //@src"):
            url = urlsplit(target)
            if url.scheme or url.netloc:
                if url.scheme != "https" or tree.xpath('//script[@src=$url] | //link[@href=$url] | //img[@src=$url]', url=target):
                    raise ValueError(f"Unexpected external resource: {target}")
                continue
            destination = path.parent / unquote(url.path) if url.path else path
            if not destination.is_file():
                raise ValueError(f"Missing local target {target} in {path.name}")
            if url.fragment and destination.suffix == ".html":
                destination_tree = parsed.get(destination)
                if destination_tree is None:
                    destination_tree = html.fromstring(destination.read_text())
                if unquote(url.fragment) not in destination_tree.xpath("//@id"):
                    raise ValueError(f"Missing anchor {target} in {path.name}")
            links += 1
    combined = parsed[output / "all.html"]
    if combined.xpath("//script[@src] | //link[@href] | //img[@src]"):
        raise ValueError("Combined HTML contains an external file dependency")
    return {"status": "passed", "pages": len(pages), "source_documents": len(ORDER), "checked_local_references": links, "inline_diagrams": len(combined.xpath('//*[local-name()="svg"]')), "external_resources": 0}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=PROJECT / "build" / "docs-html")
    arguments = parser.parse_args()
    output = arguments.output.resolve()
    (output / "assets").mkdir(parents=True, exist_ok=True)
    font_config = Path(os.environ.get("XDZIP_FONTCONFIG_FILE", "/nonexistent/xdzip-fonts.conf"))
    if font_config.is_file() and "FONTCONFIG_FILE" not in os.environ:
        os.environ["FONTCONFIG_FILE"] = str(font_config)
    if set(ORDER) != {p.stem for p in SOURCE.glob("*.md")}:
        raise ValueError("Update document order for the changed source set")
    generated = []
    documents = {}
    for stem in ORDER:
        article, diagrams = prepare_document(stem, output)
        documents[stem] = article
        generated.extend(diagrams)
    css = (TEMPLATES / "style.css").read_text()
    js = (TEMPLATES / "app.js").read_text()
    for name, content in [("style.css", css), ("app.js", js), ("search-index.js", json_script(search_records(documents, combined=False)))]:
        path = output / "assets" / name
        path.write_text(content)
        generated.append(path)
    pages = []
    for stem, article in documents.items():
        tree = remap(article, stem, combined=False, output=output)
        toc = [(h.get("id"), h.text_content().removesuffix("#")) for h in tree.xpath(".//h2")]
        page = output / (stem + ".html")
        page.write_text(shell(article.find("h1").text_content(), serialize(tree), documents, current=stem, combined=False, toc=toc, css=css, js=js, records=[]))
        pages.append(page)
    combined_trees = [remap(article, stem, combined=True, output=output) for stem, article in documents.items()]
    combined_path = output / "all.html"
    combined_path.write_text(shell("完整设计文档", "".join(serialize(tree) for tree in combined_trees), documents, current="index", combined=True, toc=[(tree.get("id"), tree.find("h1").text_content().removeprefix("xdzip：")) for tree in combined_trees], css=css, js=js, records=search_records(documents, combined=True)))
    pages.append(combined_path)
    generated.extend(pages)
    report = validate(output, pages)
    report["generator"] = {"pandoc": text_command(["pandoc", "--version"]).splitlines()[0], "renderer": "Graphviz SVG; source edges and labels preserved"}
    report_path = PROJECT / "build/review/docs/VALIDATION.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    instructions = output / "README.txt"
    instructions.write_text(
        "xdzip HTML 软件设计文档\n\n"
        "多页面阅读：解压整个目录后，用浏览器打开 index.html。请保留 assets 目录。\n"
        "单文件阅读：all.html 包含全部 16 篇、样式、搜索数据和两张 SVG 图，可以单独复制分享。\n"
        "搜索：点击顶部搜索按钮，或按 Cmd/Ctrl+K；可用空格组合关键词。\n"
        "显示：顶部按钮切换深浅色；窄屏点击目录按钮切换文档。\n"
        "打印：使用浏览器打印功能，可另存为 PDF；导航和搜索控件会自动隐藏。\n"
        "文档无需网络或服务器，搜索需启用 JavaScript。\n"
        "本包为软件设计文档，不是已经实现的 xdzip 桌面应用。补充工程和视觉文档链接指向 GitHub，访问它们需要网络和仓库权限。\n"
    )
    generated.append(instructions)
    browser_report = PROJECT / "dist" / "xdzip-html-browser-validation.json"
    if browser_report.is_file():
        browser_validation = json.loads(browser_report.read_text())
        hashes = browser_validation.get("artifact_sha256", {})
        expected = {str(path.relative_to(output)): hashlib.sha256(path.read_bytes()).hexdigest()
                    for path in generated if path.suffix in (".html", ".css", ".js", ".svg")}
        # Include browser evidence only if it covers precisely these artifacts.
        if browser_validation.get("status") == "passed" and hashes == expected:
            destination = PROJECT / "build/review/docs/BROWSER-VALIDATION.json"
            destination.write_bytes(browser_report.read_bytes())
    checksums = output / "CHECKSUMS.sha256"
    checksums.write_text("".join(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(output)}\n" for path in sorted(generated)))
    generated.append(checksums)
    (PROJECT / "dist").mkdir(exist_ok=True)
    bundle = PROJECT / "dist" / "xdzip-html-docs.zip"
    with zipfile.ZipFile(bundle, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in sorted(generated):
            archive.write(path, "xdzip-html-docs/" + str(path.relative_to(output)))
    with zipfile.ZipFile(bundle) as archive:
        if archive.testzip() is not None:
            raise ValueError("Generated ZIP integrity check failed")
    standalone = PROJECT / "dist" / "xdzip-design.html"
    standalone.write_bytes(combined_path.read_bytes())
    print(json.dumps({"validation": report, "bundle": str(bundle), "bundle_bytes": bundle.stat().st_size,
                      "standalone": str(standalone), "standalone_bytes": standalone.stat().st_size}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
