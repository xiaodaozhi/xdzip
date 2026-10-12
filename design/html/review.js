"use strict";
const frame = document.querySelector("#design-frame");
const views = {
    main: "../hifi/v1/screens/main-light.png",
    extract: "../hifi/v1/screens/extract-macos.png",
    create: "../hifi/v2/screens/01-create.png",
    progress: "../hifi/v2/screens/03-progress.png",
    conflict: "../hifi/v2/screens/02-conflict.png",
    settings: "../hifi/v2/screens/05-settings.png",
};
const current = id => document.querySelector(`#${id}`).value;
function fit() {
    const width = document.querySelector("#viewport").clientWidth;
    const [canvasWidth, canvasHeight] = current("view") === "main" ? [1588,1000] : current("view") === "progress" ? [1446,1088] : [1092,1440];
    frame.style.width = `${canvasWidth}px`; frame.style.height = `${canvasHeight}px`;
    const scale = current("zoom") === "fit" ? Math.min(1, width / canvasWidth) : 1;
    frame.style.transform = `scale(${scale})`;
    document.querySelector("#scaled").style.cssText = `width:${canvasWidth * scale}px;height:${canvasHeight * scale}px`;
}
function updateReference() {
    const source = views[current("view")];
    const image = document.querySelector("#reference-image");
    image.hidden = !source;
    document.querySelector("#reference-message").textContent = source
        ? "原稿与 HTML 独立展示；图片没有覆盖在界面上。"
        : "此状态暂无独立的已确认原稿。";
    if (source) image.src = window.XDZIP_REFERENCES?.[source] || source;
    else image.removeAttribute("src");
}
function update() {
    // postMessage also works when local files have distinct opaque origins.
    frame.contentWindow?.postMessage({
        type: "xdzip-review-configure", view: current("view"),
        platform: current("platform"), theme: current("theme"),
    }, "*");
    updateReference();
    document.querySelector("#status").textContent = `${current("platform")} · ${current("theme") === "dark" ? "深色适配（无对应完整主稿）" : "浅色"} · 桌面画布 1548 × 940`;
    fit();
}
frame.addEventListener("load", update);
window.addEventListener("message", event => {
    if (event.source === frame.contentWindow && event.data?.type === "xdzip-review-ready") update();
});
document.querySelector("#reference-image").addEventListener("error", event => {
    event.currentTarget.hidden = true;
    document.querySelector("#reference-message").textContent = "原稿加载失败。请使用完整仓库目录，或重新下载离线单文件 HTML。";
});
if (window.XDZIP_CANVAS) frame.srcdoc = window.XDZIP_CANVAS;
for (const id of ["view", "platform", "theme"]) document.querySelector(`#${id}`).addEventListener("change", update);
document.querySelector("#zoom").addEventListener("change", fit);
window.addEventListener("resize", fit);
document.querySelector("#reference-toggle").onclick = event => {
    const reference = document.querySelector("#reference");
    reference.hidden = !reference.hidden;
    if (!reference.hidden) updateReference();
    event.currentTarget.setAttribute("aria-expanded", String(!reference.hidden));
};
document.querySelector("#separate").onclick = () => {
    const view = current("view") === "main" ? "extract" : current("view");
    const popup = window.open(window.XDZIP_CANVAS ? "about:blank" : `canvas.html?standalone=1&view=${view}&platform=${current("platform")}`, "xdzip-design-window", "popup,width=860,height=1020");
    if (!popup) { document.querySelector("#status").textContent = "浏览器拦截了独立预览窗口，可允许此次弹窗后重试。"; return; }
    if (window.XDZIP_CANVAS) {
        popup.document.open();
        popup.document.write(window.XDZIP_CANVAS);
        popup.document.close();
    }
    popup.addEventListener("load", () => {
        popup.document.body.dataset.standalone = "true";
        popup.XdzipDesign.platform(current("platform"));
        popup.XdzipDesign.theme(current("theme"));
        popup.XdzipDesign.open(view);
    });
};
update();
