"use strict";
// The reviewer owns artboard navigation; embedded documents keep a fixed canvas.
// Comparing window identities also works when an offline srcdoc has an opaque origin.
if (window.parent !== window)
    document.documentElement.dataset.embeddedCanvas = "true";
// Design-only state machine: all file operations use in-memory fixtures.
// This prototype never reads archives, writes files, or uploads user content.
// Icons come from the same reviewed SVG sources as Qt; no second hand-drawn set.
const assetNames = {
    folderOpen: "open",
    file: "file-text",
    pdf: "file-pdf",
    image: "image-file",
    code: "file-code",
    markdown: "file-markdown",
    chevronDown: "chevron-down",
    chevronUp: "chevron-up",
    chevronRight: "chevron-right",
    arrow: "arrow-right",
    layers: "folder",
};
let iconInstance = 0;
const icon = (name, cls = "") => {
    const key = assetNames[name] || name;
    const theme =
        document.documentElement.dataset.theme === "dark" ? "dark" : "light";
    const source = window.XDZIP_ASSETS?.icons?.[theme]?.[key];
    // A missing shared asset is a packaging error, not permission to draw a substitute.
    if (!source) throw new Error(`Missing xdzip icon asset: ${theme}/${key}`);
    const prefix = `icon-${++iconInstance}-`;
    return source
        .replace(/id="([^"]+)"/g, (_, id) => `id="${prefix}${id}"`)
        .replace(/url\(#([^)]+)\)/g, (_, id) => `url(#${prefix}${id})`)
        .replace(
            "<svg ",
            `<svg class="${cls}" data-asset="${key}" aria-hidden="true" `,
        );
};
const photo = (key, cls = "") =>
    `<img class="${cls}" src="${window.XDZIP_ASSETS.photos[key]}" alt="${key.startsWith("lake") ? "冰川湖" : "山间日出"} · 设计示例图片">`;
const $ = (s) => document.querySelector(s);
const esc = (s) =>
    String(s).replace(
        /[&<>"']/g,
        (c) =>
            ({
                "&": "&amp;",
                "<": "&lt;",
                ">": "&gt;",
                '"': "&quot;",
                "'": "&#39;",
            })[c],
    );
const MB = 1000000;
const bytes = (n) =>
    n === 0
        ? "0 B"
        : n >= MB
          ? `${(n / MB).toFixed(1)} MB`
          : n >= 1000
            ? `${Math.round(n / 1000)} KB`
            : `${n} B`;
const rootFolders = [
    { id: "photos", name: "01 照片", parent: "", folder: true },
    { id: "documents", name: "02 文档", parent: "", folder: true },
    { id: "maps", name: "03 地图", parent: "", folder: true },
];
const roots = [
    ["lake", "冰川湖.jpg", 8.4, 8.1, "JPEG 图像"],
    ["sunrise", "山间日出.jpg", 6.2, 5.9, "JPEG 图像"],
    ["guide", "行程手册.pdf", 12.8, 7.1, "PDF 文档"],
    ["list", "旅行清单.txt", 0.004, 0.002, "文本文件"],
    ["route", "路线规划.gpx", 0.116, 0.068, "GPX 文件"],
    ["readme", "README.md", 0.008, 0.004, "Markdown 文档"],
].map(([id, name, size, compressed, type]) => ({
    id,
    name,
    size: Math.round(size * MB),
    compressed: Math.round(compressed * MB),
    type,
    parent: "",
}));
const children = rootFolders.flatMap((f, fi) =>
    Array.from({ length: [12, 8, 4][fi] }, (_, i) => ({
        id: `${f.id}-${i}`,
        parent: f.id,
        name: `${["旅途瞬间", "旅行笔记", "离线地图"][fi]}-${String(i + 1).padStart(2, "0")}.${["jpg", "pdf", "gpx"][fi]}`,
        size: (fi + 1) * MB + i * 100000,
        compressed: Math.round(((fi + 1) * MB + i * 100000) * 0.72),
        type: ["JPEG 图像", "PDF 文档", "GPX 文件"][fi],
    })),
);
let folders = [
    ...rootFolders,
    { id: "ice", name: "冰川湖", parent: "photos", folder: true },
    { id: "shore", name: "湖岸步道", parent: "ice", folder: true },
    { id: "shore-sub", name: "沿湖照片", parent: "shore", folder: true },
    { id: "snow", name: "雪山远景", parent: "ice", folder: true },
    { id: "sunrise-dir", name: "山间日出", parent: "photos", folder: true },
    {
        id: "sunrise-sub",
        name: "日出照片",
        parent: "sunrise-dir",
        folder: true,
    },
    { id: "other-photos", name: "其他照片", parent: "photos", folder: true },
    { id: "docs-sub", name: "旅行资料", parent: "documents", folder: true },
];
for (const entry of children) {
    const number = Number(entry.id.split("-").pop());
    if (entry.parent === "photos")
        entry.parent =
            number < 3
                ? "shore-sub"
                : number < 6
                  ? "snow"
                  : number < 10
                    ? "sunrise-sub"
                    : "other-photos";
    else if (entry.parent === "documents") entry.parent = "docs-sub";
}
let entries = [...folders, ...roots, ...children],
    files = entries.filter((e) => !e.folder),
    byId = new Map(entries.map((e) => [e.id, e]));
// Demo timestamps follow the approved scene; every entry keeps its own metadata.
const sceneDates = {
    photos: "2026/10/01",
    documents: "2026/10/02",
    maps: "2026/10/03",
    lake: "2026/10/08",
    sunrise: "2026/10/07",
    guide: "2026/10/06",
    list: "2026/10/01",
    route: "2026/10/03",
    readme: "2026/10/01",
};
for (const entry of entries)
    entry.modified = sceneDates[entry.id] || "2026/10/08";
const typeAbbreviations = {
    jpg: "JPEG",
    pdf: "PDF",
    txt: "TXT",
    gpx: "GPX",
    md: "MD",
};
const imagePropertyOrder = ["尺寸", "原始大小", "压缩后", "修改日期"];
const displayType = (entry) =>
    entry.folder
        ? "文件夹"
        : typeAbbreviations[entry.name.split(".").pop()] ||
          entry.type.replace(" 文件", "");
let archiveBytes = files.reduce((n, f) => n + f.size, 0),
    compressedBytes = files.reduce((n, f) => n + f.compressed, 0);
const state = {
    path: "",
    query: "",
    selected: new Set(["lake"]),
    sort: 0,
    sortKey: "name",
    view: "list",
    expanded: new Set(),
    treeFocus: "",
    archiveTitle: "北境旅行.zip",
    activeTab: "travel",
    anchor: null,
    scenario: "normal",
    archiveState: "ready",
    modal: null,
    scope: "all",
    task: null,
    plan: null,
    createItems: [],
    theme: "light",
    recent: true,
};
const specs = {
    B01: [
        "归档浏览",
        "P01 / P02 / P08 · AC-01 / AC-02 / AC-08",
        "打开后只读取目录和属性，不解压内容。目录双击进入；条目用稳定 ID 选择。",
        "单击、复选、Shift 范围选择、⌘/Ctrl+A。Enter 进入文件夹或查看属性。",
        "索引未完成时禁用全部解压；空归档与无搜索结果是不同状态。",
    ],
    B02: [
        "搜索与选择",
        "P02 / P03 · AC-02 / AC-03",
        "搜索覆盖当前目录及其后代；结果显示来源路径。无选择时全部解压始终指整个归档。",
        "选择解压所选、全部归档或显式的搜索结果范围。清空搜索恢复原目录。",
        "输入法组合期间不提交查询。切换列表清除选择，避免隐藏选择造成范围误解。",
    ],
    E01: [
        "解压确认",
        "P03 / P07 · AC-03 / AC-07",
        "路径预览只追加一次同名目录。所选项和全部归档具有不同计数与大小。",
        "路径、范围、重名策略均可修改。权限恢复后才允许开始；Esc 取消并恢复焦点。",
        "原型只模拟预检，不代表真实权限、安全路径或文件系统验证。",
    ],
    E02: [
        "重名决策",
        "P03 / P07 · AC-03 / AC-07",
        "保留已有文件，等待跳过、自动改名或覆盖。覆盖不作为默认 Enter 操作。",
        "默认焦点为跳过。决定仅针对本次任务，可选择对后续同类冲突应用。",
        "覆盖前需由正式核心再次核验目标身份；安全错误不能通过覆盖绕过。",
    ],
    T01: [
        "任务进行中",
        "P06 / P15 · AC-06 / AC-14",
        "演示准备、执行、校验、停止与结果；计数对应同一任务范围。",
        "取消后先进入正在停止，再显示已保留和未完成项；返回浏览不取消任务。",
        "所有进度是示例状态，不代表真实吞吐。任务详情可再次打开。",
    ],
    R01: [
        "任务结果",
        "P03 / P04 / P15 · AC-03 / AC-04 / AC-14",
        "成功、跳过、失败或未完成分开显示。部分完成不是全部成功。",
        "重试只包含失败或未完成项，重新进入确认流程。",
        "创建归档整体提交；取消前未发布时不算成功创建归档。",
    ],
    C01: [
        "新建压缩文件",
        "P04 / P05 · AC-04 / AC-05",
        "示例输入支持添加、移除。ZIP 可选 Store/Deflate；TAR 无压缩级别，TAR.GZ 使用 GZIP。",
        "更改格式同步扩展名和参数。输入为空或名称为空时不可提交。",
        "首版不提供加密和分卷。已有目标进入独立覆盖确认，取消保留旧归档。",
    ],
    S01: [
        "设置",
        "P08 / P11 · AC-08 / AC-11",
        "深浅主题和密度即时预览，最近文件记录可清空。",
        "设置仅影响设计原型；关闭返回当前浏览状态。",
        "无联网、遥测和外部扩展安装。手机布局用于审阅，不声明移动端产品。",
    ],
    X01: [
        "异常状态",
        "P05 / P07 / P15 · AC-05 / AC-07 / AC-14",
        "区分空内容、加载、损坏、不支持、磁盘满与源变化。",
        "错误提供明确恢复入口，不能以空列表或假成功掩盖失败。",
        "使用顶部场景选择器触发；它属于设计审阅工具，不进入产品界面。",
    ],
};
let currentSpec = "B01",
    timer = null,
    loadTimer = null,
    toastTimer = null,
    returnFocus = null;
function hydrate() {
    document
        .querySelectorAll("[data-icon]")
        .forEach((n) => (n.innerHTML = icon(n.dataset.icon)));
    // The inactive archive has its own reviewed artwork, rather than a CSS recolor.
    const inactive = $(".archive-tab.inactive .tab-icon");
    if (inactive) inactive.innerHTML = icon("archive-grey");
}
function toast(text) {
    $("#toast").textContent = text;
    $("#toast").classList.add("toast-show");
    clearTimeout(toastTimer);
    toastTimer = setTimeout(
        () => $("#toast").classList.remove("toast-show"),
        3000,
    );
}
function setSpec(id) {
    currentSpec = id;
    const s = specs[id];
    $("#state-label").textContent = `${id} · ${s[0]}`;
    $("#spec").innerHTML =
        `<h2>${id} / ${s[0]} <small style="font-size:11px;color:var(--secondary);font-weight:400">${s[1]}</small></h2><div class="spec-grid">${["行为与范围", "交互与恢复", "验收边界"].map((x, i) => `<div><h3>${x}</h3><p>${s[i + 2]}</p></div>`).join("")}</div>`;
}
function selectedFiles(ids = state.selected) {
    const found = new Map();
    for (const id of ids) {
        const e = byId.get(id);
        if (!e) continue;
        if (e.folder)
            files
                .filter((x) => withinDirectory(x, id))
                .forEach((x) => found.set(x.id, x));
        else found.set(id, e);
    }
    return [...found.values()];
}
function withinDirectory(entry, directory) {
    if (!directory) return true;
    let parent = entry.parent;
    while (parent) {
        if (parent === directory) return true;
        parent = byId.get(parent)?.parent || "";
    }
    return false;
}
function archivePath(entry) {
    const segments = [entry.name];
    let parent = entry.parent;
    while (parent) { const node = byId.get(parent); if (!node) break; segments.unshift(node.name); parent = node.parent; }
    return segments.join("/") + (entry.folder ? "/" : "");
}
function displayEntryName(entry) { return state.query ? archivePath(entry) : entry.name; }
function visibleEntries() {
    if (state.archiveState !== "ready") return [];
    const list = entries.filter((entry) =>
        state.query
            ? withinDirectory(entry, state.path) &&
              archivePath(entry).toLowerCase().includes(state.query.toLowerCase())
            : entry.parent === state.path,
    );
    if (state.sort)
        list.sort((a, b) => {
            const grouping = Number(Boolean(b.folder)) - Number(Boolean(a.folder));
            if (grouping) return grouping;
            const key = state.sortKey;
            const compared =
                key === "size" || key === "compressed"
                    ? (a[key] || 0) - (b[key] || 0)
                    : String(a[key] || "").localeCompare(
                          String(b[key] || ""),
                          "zh-CN",
                          { numeric: true },
                      );
            return (
                compared * state.sort || entries.indexOf(a) - entries.indexOf(b)
            );
        });
    return list;
}

function fileIcon(e) {
    return e.folder
        ? "folder"
        : e.type.includes("JPEG")
          ? "image"
          : e.type.includes("PDF")
            ? "pdf"
            : e.name.endsWith(".gpx")
              ? "code"
              : e.name.endsWith(".md")
                ? "markdown"
                : "file";
}
function choosePath(path) {
    state.path = path;
    state.selected.clear();
    state.anchor = null;
    render();
}
function renderTree() {
    if (window.XdzipMain) return window.XdzipMain.renderTree();
    if (state.archiveState !== "ready") {
        $("#tree").innerHTML =
            `<button class="tree-item active" disabled>${icon("layers")}<span>根目录</span><small>${state.archiveState === "empty" ? "0" : "—"}</small></button>`;
        return;
    }
    const counts = (id) =>
        files.filter((f) => (id ? f.parent === id : true)).length;
    $("#tree").innerHTML =
        `<button class="tree-item ${!state.path ? "active" : ""}" data-path="">${icon("layers")}<span>根目录</span></button>${folders.map((f) => `<button class="tree-item child ${state.path === f.id ? "active" : ""}" data-path="${f.id}">${icon("chevronRight")}${icon("folder")}<span>${f.name}</span></button>`).join("")}`;
    $("#tree")
        .querySelectorAll("button")
        .forEach((b) => (b.onclick = () => choosePath(b.dataset.path)));
}
function syncInspectorContentOffset() {
    // Search notices occupy a row above the table; preserve the shared content edge.
    const notice = $("#search-notice");
    const style = getComputedStyle(notice);
    const offset = notice.hidden
        ? 0
        : notice.getBoundingClientRect().height +
          parseFloat(style.marginTop) +
          parseFloat(style.marginBottom);
    $("#inspector").style.setProperty("--pane-content-offset", `${offset}px`);
}
function render() {
    XdzipControls.close({ restoreFocus: false });
    renderTree();
    const list = visibleEntries(),
        chosen = selectedFiles(),
        sum = chosen.reduce((n, f) => n + f.size, 0);
    $(".archive-card strong").textContent = state.archiveTitle;
    $("#archive-info").textContent = bytes(compressedBytes);
    $("#original-size").textContent = bytes(archiveBytes);
    $("#compressed-size").textContent = bytes(compressedBytes);
    const saving = Math.round((1 - compressedBytes / archiveBytes) * 100);
    $("#saving").textContent = saving + "%";
    $("#saving-bar").style.width = saving + "%";
    if (state.archiveState !== "ready") {
        $("#archive-info").textContent =
            state.archiveState === "empty" ? "空归档" : "元数据未就绪";
        $("#original-size").textContent =
            state.archiveState === "empty" ? "0 B" : "未知";
        $("#compressed-size").textContent = "未知";
        $("#saving").textContent = "—";
        $("#saving-bar").style.width = "0%";
    }
    const pathNodes = [];
    let breadcrumbId = state.path;
    while (breadcrumbId && byId.has(breadcrumbId)) {
        const node = byId.get(breadcrumbId);
        pathNodes.unshift(node);
        breadcrumbId = node.parent;
    }
    const separator =
        '<span class="breadcrumb-separator" aria-hidden="true">/</span>';
    $("#breadcrumbs").innerHTML =
        `<button data-home>${esc(state.archiveTitle)}</button>` +
        separator +
        (pathNodes.length
            ? pathNodes
                  .map((node, index) =>
                      index < pathNodes.length - 1 || state.query
                          ? `<button data-breadcrumb="${node.id}" data-tooltip="${esc(node.name)}">${esc(node.name)}</button>`
                          : `<span aria-current="page">${esc(node.name)}</span>`,
                  )
                  .join(separator)
            : "<span>根目录</span>") +
        (state.query
            ? separator + '<span aria-current="page">搜索结果</span>'
            : "");
    $("#breadcrumbs [data-home]").onclick = () => choosePath("");
    $("#breadcrumbs")
        .querySelectorAll("[data-breadcrumb]")
        .forEach(
            (button) =>
                (button.onclick = () => choosePath(button.dataset.breadcrumb)),
        );
    $("#entry-count").textContent =
        `${list.length} 项`;
    $("#search-notice").hidden = !state.query;
    $("#search-notice").textContent =
        `当前${state.path ? "目录及后代" : "归档"}中找到 ${list.length} 项 · 搜索结果不等同于全部归档`;
    $("#file-rows").innerHTML = list
        .map(
            (e) =>
                `<tr tabindex="0" data-id="${e.id}" class="${e.folder ? "folder-row " : ""}${state.selected.has(e.id) ? "selected" : ""}" aria-selected="${state.selected.has(e.id)}"><td><input type="checkbox" tabindex="-1" aria-label="选择 ${esc(e.name)}" ${state.selected.has(e.id) ? "checked" : ""}></td><td><div class="entry-name"><span class="entry-icon">${["lake", "sunrise"].includes(e.id) ? photo(e.id, "entry-photo") : icon(fileIcon(e), fileIcon(e))}</span><span title="${esc(e.name)}">${esc(displayEntryName(e))}</span></div></td><td>${e.folder ? "—" : bytes(e.size)}</td><td>${e.folder ? "—" : bytes(e.compressed)}</td><td class="type-col">${esc(displayType(e))}</td><td class="date-col">${esc(e.modified)}</td></tr>`,
        )
        .join("");
    $("#file-rows")
        .querySelectorAll("tr")
        .forEach((row) => {
            row.onclick = (ev) => {
                const id = row.dataset.id,
                    idx = list.findIndex((e) => e.id === id);
                if (ev.shiftKey && state.anchor !== null) {
                    const anchor = list.findIndex((e) => e.id === state.anchor);
                    if (anchor >= 0) {
                        state.selected = new Set(
                            list
                                .slice(
                                    Math.min(anchor, idx),
                                    Math.max(anchor, idx) + 1,
                                )
                                .map((e) => e.id),
                        );
                    }
                } else if (
                    ev.ctrlKey ||
                    ev.metaKey ||
                    ev.target.matches("input")
                ) {
                    state.selected.has(id)
                        ? state.selected.delete(id)
                        : state.selected.add(id);
                    state.anchor = id;
                } else {
                    state.selected = new Set([id]);
                    state.anchor = id;
                }
                render();
                const next = $(`[data-id="${id}"]`);
                if (next) next.focus({ preventScroll: true });
            };
            row.ondblclick = () => activateEntry(row.dataset.id);
            row.onkeydown = (ev) => {
                if (ev.key === "Enter") {
                    ev.preventDefault();
                    activateEntry(row.dataset.id);
                }
                if (ev.key === " ") {
                    ev.preventDefault();
                    row.click();
                }
                if (["ArrowDown", "ArrowUp"].includes(ev.key)) {
                    ev.preventDefault();
                    const next =
                        ev.key === "ArrowDown"
                            ? row.nextElementSibling
                            : row.previousElementSibling;
                    if (next) {
                        next.click();
                        next.focus();
                    }
                }
            };
        });
    const all = $("#select-all");
    all.checked = !!list.length && list.every((e) => state.selected.has(e.id));
    all.indeterminate =
        !all.checked && list.some((e) => state.selected.has(e.id));
    all.disabled = !list.length;
    $("#sort-name [data-icon]").innerHTML = icon("sort-ascending");
    $("#sort-name [data-icon]").classList.toggle("descending", state.sort < 0);
    $("#sort-name [data-icon]").style.visibility =
        state.sort && state.sortKey === "name" ? "visible" : "hidden";
    $("#sort-name").setAttribute(
        "aria-label",
        state.sort === 0 ? "按名称升序排序" : "切换名称排序方向",
    );
    $("#sort-name")
        .closest("th")
        .setAttribute(
            "aria-sort",
            state.sort === 0
                ? "none"
                : state.sort < 0
                  ? "descending"
                  : "ascending",
        );
    $("#extract-label").textContent = chosen.length ? "解压所选" : "全部解压";
    $("#extract").disabled = state.archiveState !== "ready";
    $("#extract-menu-button").disabled = state.archiveState !== "ready";
    $("#selection-status").textContent = chosen.length
        ? `已选择 ${chosen.length} 项 · ${bytes(sum)}`
        : `${files.length} 个文件 · ${folders.length} 个文件夹`;
    $("#read-status").textContent =
        state.archiveState === "loading"
            ? "正在读取目录"
            : state.archiveState === "ready"
              ? "目录已读取"
              : state.archiveState === "empty"
                ? "空归档"
                : "无法读取";
    if (state.archiveState !== "ready") {
        $("#selection-status").textContent =
            state.archiveState === "empty"
                ? "0 个文件 · 0 个文件夹"
                : "目录信息未就绪";
    }
    window.XdzipMain?.sync(list);
    renderEmpty(list);
    syncInspectorContentOffset();
    renderInspector(chosen);
    updateTaskCount();
    if (!state.modal)
        setSpec(
            state.archiveState !== "ready"
                ? "X01"
                : state.query || chosen.length
                  ? "B02"
                  : "B01",
        );
}
function activateEntry(id) {
    const e = byId.get(id);
    if (e.folder) choosePath(id);
    else {
        state.selected = new Set([id]);
        render();
        $("#inspector").classList.remove("collapsed-narrow");
        $("#inspector").hidden = false;
    }
}
function renderEmpty(list) {
    const box = $("#empty-state");
    box.hidden = list.length > 0;
    let title = "没有找到匹配项",
        desc = "试试其他名称，或清空搜索查看所有文件。",
        ico = "search",
        label = "清空搜索",
        fn = () => {
            state.query = "";
            $("#search").value = "";
            render();
        };
    if (state.archiveState === "empty") {
        title = "这个压缩文件没有内容";
        desc = "目录读取完成，没有可解压的文件。";
        ico = "archive";
        label = "打开其他文件";
        fn = () => openPicker();
    }
    if (state.archiveState === "loading") {
        title = "正在读取目录";
        desc = "正在扫描归档目录，总条目数暂时未知。";
        ico = "clock";
        label = "取消读取";
        fn = () => {
            clearTimeout(loadTimer);
            showWelcome();
        };
    }
    if (["unsupported", "damaged"].includes(state.archiveState)) {
        title =
            state.archiveState === "unsupported"
                ? "暂不支持此加密方式"
                : "无法读取压缩文件";
        desc =
            state.archiveState === "unsupported"
                ? "此文件使用 ZIP AES。当前版本支持未加密的 ZIP Store/Deflate。"
                : "文件目录已损坏或下载不完整。请重新获取完整文件后再试。";
        ico = "warning";
        label = "打开其他文件";
        fn = () => openPicker();
    }
    box.innerHTML = `${icon(ico)}<h2>${title}</h2><p>${desc}</p>${state.archiveState === "loading" ? '<div style="width:180px"><div class="skeleton"></div><div class="skeleton"></div></div>' : ""}<button class="button">${label}</button>`;
    box.querySelector("button").onclick = fn;
}
function renderInspector(chosen) {
    if (state.archiveState !== "ready") {
        $("#inspector").innerHTML =
            `<div class="inspector-head"><span>归档信息</span></div><div class="file-emblem">${icon("archive")}</div><h2>${esc(state.archiveTitle)}</h2><p class="inspector-footnote">${state.archiveState === "empty" ? "目录已读取，归档中没有可解压的文件。" : "目录信息未就绪，暂时无法展示文件属性。"}</p>`;
        return;
    }
    const multi = chosen.length > 1,
        single = chosen.length === 1 ? chosen[0] : null;
    const size = chosen.length
            ? chosen.reduce((n, e) => n + e.size, 0)
            : archiveBytes,
        packed = chosen.length
            ? chosen.reduce((n, e) => n + e.compressed, 0)
            : compressedBytes;
    $("#inspector").innerHTML =
        `<div class="inspector-head"><span>${single?.type.includes("JPEG") ? "文件预览" : chosen.length ? "文件属性" : "归档信息"}</span><button class="icon-button" aria-label="关闭属性面板" id="hide-inspector">${icon("close")}</button></div>${single && ["lake", "sunrise"].includes(single.id) ? `<div class="photo-preview">${photo(single.id === "lake" ? "lake-preview" : "sunrise")}</div>` : `<div class="file-emblem">${icon(multi ? "layers" : single ? fileIcon(single) : "archive")}</div>`}<h2>${multi ? `${chosen.length} 个文件` : single ? esc(single.name) : esc(state.archiveTitle)}</h2><span class="type-label">${multi ? "多个类型 · 已选择" : single ? single.type : "ZIP 压缩文件 · Deflate"}</span><dl class="property-list">${[
            ["原始大小", bytes(size)],
            ["压缩后", bytes(packed)],
            [
                single?.type.includes("JPEG") ? "尺寸" : "压缩方式",
                single?.type.includes("JPEG") ? "6000 × 4000" : "Deflate",
            ],
            ["修改日期", single?.modified || "2026/10/08"],
            [
                "位置",
                single && single.parent
                    ? byId.get(single.parent).name
                    : "归档根目录",
            ],
        ]
            .filter(
                ([label]) => !single?.type.includes("JPEG") || label !== "位置",
            )
            .sort((a, b) =>
                single?.type.includes("JPEG")
                    ? imagePropertyOrder.indexOf(a[0]) -
                      imagePropertyOrder.indexOf(b[0])
                    : 0,
            )
            .map(
                ([a, b]) =>
                    `<div class="property"><dt>${a}</dt><dd${a === "修改日期" ? ' class="modified-value"' : ""}>${esc(b)}</dd></div>`,
            )
            .join(
                "",
            )}</dl><button id="inspector-extract" class="button" ${state.archiveState !== "ready" ? "disabled" : ""}>${icon("external-open")}${single ? "解压此文件" : chosen.length ? "解压所选文件" : "全部解压"}</button><p class="inspector-footnote">文件内容保留在压缩包中。只有开始解压后，才会写入目标位置。</p>`;
    $("#hide-inspector").onclick = () => ($("#inspector").hidden = true);
    $("#inspector-extract").onclick = () =>
        window.XdzipMain?.dispatch(
            chosen.length ? "extract-selected" : "extract-all",
        );
    if (window.XdzipMain)
        $("#inspector-extract").disabled =
            !window.XdzipMain.commands[
                chosen.length ? "extract-selected" : "extract-all"
            ].enabled();
}
function updateTaskCount() {
    const running =
        Number(
            state.task &&
                ["running", "stopping", "conflict"].includes(state.task.status),
        ) + Number(state.testJob?.status === "running");
    $("#task-count").textContent = running
        ? `任务 ${running}`
        : state.task || state.testJob
          ? "任务 · 已结束"
          : "任务 0";
}
// Web chrome is a design simulation. Production uses separate QQuickWindows.
function windowClose() {
    if (state.modal === "conflict") return cancelTask();
    if (state.modal === "create-overwrite") return showCreate();
    closeModal(); // Closing a task view never cancels its underlying task.
}
function windowChrome(title, taskView) {
    const mac = document.body.dataset.platform === "macos";
    const controls = `<div class="os-controls ${mac ? "mac-controls" : "pc-controls"}">
        <button class="os-close" aria-label="关闭操作窗口">${icon("close")}</button>
        <button class="os-minimize" aria-label="收起任务窗口" ${taskView ? "" : "disabled"}><span>−</span></button>
        <button class="os-maximize" aria-label="此窗口不可最大化" disabled><span>□</span></button></div>`;
    return `<div class="os-titlebar">${mac ? controls : taskView ? icon(document.body.dataset.platform === "windows" ? "windows-document" : "linux-document") : icon("app-mark")}<span class="os-title">${esc(title)}</span>${mac ? '<span class="os-balance"></span>' : controls}</div>`;
}
function dialog(html, kind, spec, focus) {
    XdzipControls.close({ restoreFocus: false });
    XdzipControls.hideTooltip();
    const el = $("#modal");
    returnFocus = el.open ? returnFocus : document.activeElement;
    if (el.open) el.close();
    const template = document.createElement("template");
    template.innerHTML = html;
    const title = template.content.querySelector("h1")?.textContent || "xdzip";
    el.innerHTML =
        windowChrome(
            kind === "progress" ? "解压进度" : title,
            ["progress", "archive-test"].includes(kind),
        ) + html;
    el.setAttribute("aria-label", title);
    el.style.left = el.style.top = el.style.margin = el.style.transform = "";
    state.modal = kind;
    el.dataset.kind = kind;
    setSpec(spec);
    // Decision windows block only their owner; task details/settings remain modeless.
    const modeless = [
        "progress",
        "settings",
        "result",
        "archive-test",
    ].includes(kind);
    el.dataset.modeless = String(modeless);
    if (modeless) el.show();
    else el.showModal();
    $(".os-close").onclick = windowClose;
    $(".os-minimize").onclick = closeModal;
    const bar = $(".os-titlebar");
    bar.onpointerdown = (event) => {
        if (event.target.closest("button") || event.button !== 0) return;
        const rect = el.getBoundingClientRect();
        const dx = event.clientX - rect.left,
            dy = event.clientY - rect.top;
        bar.setPointerCapture(event.pointerId);
        bar.onpointermove = (move) => {
            el.style.margin = "0";
            el.style.transform = "none";
            el.style.left = `${Math.max(0, Math.min(innerWidth - rect.width, move.clientX - dx))}px`;
            el.style.top = `${Math.max(0, Math.min(innerHeight - 44, move.clientY - dy))}px`;
        };
        bar.onpointerup = bar.onlostpointercapture = () => {
            bar.onpointermove = null;
        };
    };
    if (focus) $(focus)?.focus();
}
function closeModal() {
    if ($("#modal").open) $("#modal").close();
    state.modal = null;
    render();
    const restored = returnFocus?.isConnected
        ? returnFocus
        : returnFocus?.id
          ? document.getElementById(returnFocus.id)
          : returnFocus?.dataset.id
            ? window.XdzipMain?.entryNode(returnFocus.dataset.id)
            : null;
    restored?.focus({ preventScroll: true });
}
function header(title, subtitle, ico = "archive") {
    return `<header class="modal-header"><div class="modal-symbol">${icon(ico)}</div><div><h1>${title}</h1><p>${subtitle}</p></div></header>`;
}
function footer(content, ready = "准备就绪") {
    return `<footer class="modal-footer"><span class="ready-label">${icon("shield")}${ready}</span>${content}</footer>`;
}
function cancelButton(label = "取消") {
    return `<button class="button" data-close>${label}</button>`;
}
function bindClose() {
    document
        .querySelectorAll("[data-close]")
        .forEach((b) => (b.onclick = closeModal));
}
function openPicker() {
    window.XdzipMain.dispatch("open");
}

function showWelcome() {
    state.selected.clear();
    $("#welcome").hidden = false;
    $("#welcome").innerHTML =
        `<div class="brand-tile">${icon("archive")}</div><h1>压缩文件，一目了然。</h1><p>先查看内容，再决定解压哪些文件。</p><div class="welcome-actions"><button class="button primary" id="welcome-open">${icon("folderOpen")}打开压缩文件</button><button class="button" id="welcome-create">${icon("plus")}新建压缩文件</button></div><small>ZIP · TAR · TAR.GZ / 所有处理均在本机完成</small>`;
    $("#welcome-open").onclick = openPicker;
    $("#welcome-create").onclick = showCreate;
    render();
    setSpec("B01");
}
function scopeFiles(scope) {
    if (scope === "retry") return state.plan.retryFiles;
    if (scope === "selected") return selectedFiles();
    if (scope === "results")
        return selectedFiles(new Set(visibleEntries().map((e) => e.id)));
    return files;
}
function finalTarget() {
    const base = ($("#target-base")?.value || "下载")
        .trim()
        .replace(/[\/]+$/, "");
    const same = $("#same-folder")?.checked;
    const folder = state.archiveTitle.replace(/\.[^.]+$/, "");
    return base + (same && !base.endsWith(folder) ? " / " + folder : "");
}
function showExtract(scope = "selected", retryFiles = null) {
    if (
        state.task &&
        ["running", "stopping", "conflict"].includes(state.task.status)
    ) {
        showTask();
        return;
    }
    state.scope = scope;
    state.plan = {
        retryFiles,
        target: "下载 / " + state.archiveTitle.replace(/\.[^.]+$/, ""),
        policy: "ask",
        same: true,
        open: true,
        allowHighRatio: false,
    };
    const chosen = scopeFiles(scope);
    if (!chosen.length) {
        toast("请先选择文件，或切换为全部解压。");
        return;
    }
    const count = chosen.length,
        sum = chosen.reduce((n, e) => n + e.size, 0);
    dialog(
        header(
            "解压文件",
            `${esc(state.archiveTitle)} · ${scope === "all" ? "整个归档" : scope === "results" ? "当前搜索结果" : scope === "retry" ? "重试范围" : "所选文件"}`,
            "archive",
        ) +
            `<div class="modal-content"><div class="field-group"><label class="field-label" for="target-base">解压到</label><div class="input-row"><input id="target-base" class="text-input" value="${state.scenario === "permission" ? "系统 / 受保护目录" : "下载"}"><button class="button" id="change-target">更改</button></div><label class="check-label"><input id="same-folder" type="checkbox" checked>创建同名文件夹</label><p id="target-preview" class="field-note"></p><p id="target-valid" class="field-note success"></p></div><div class="field-group"><div class="field-label">解压范围</div><div class="scope-box">${
                scope !== "all"
                    ? `<label class="radio-row"><input type="radio" name="scope" value="${scope}" checked><span>${scope === "results" ? "当前搜索结果" : scope === "retry" ? "失败或未完成项" : "所选文件"}</span><small>${count} 项 · ${bytes(sum)}</small></label><div class="scope-files">${chosen
                          .slice(0, 3)
                          .map(
                              (e) =>
                                  `<div class="scope-file">${["lake", "sunrise"].includes(e.id) ? photo(e.id) : icon(fileIcon(e))}<span>${esc(e.name)}</span><span>${bytes(e.size)}</span></div>`,
                          )
                          .join(
                              "",
                          )}${count > 3 ? `<br>另有 ${count - 3} 个文件` : ""}</div>`
                    : ""
            }<label class="radio-row"><input type="radio" name="scope" value="all" ${scope === "all" ? "checked" : ""}><span>全部归档</span><small>${files.length} 项 · ${bytes(archiveBytes)}</small></label></div></div><div class="field-group"><label for="conflict-policy" class="field-label">遇到重名文件时</label><select id="conflict-policy" class="field-select"><option value="ask">询问我如何处理</option><option value="rename">自动改名，保留两个版本</option><option value="skip">跳过已有文件</option><option value="overwrite">覆盖已有文件</option></select><p id="policy-note" class="field-note">保留已有文件，确认后再处理。</p></div><label class="check-label"><input id="open-after" type="checkbox" checked>完成后打开目标文件夹</label><label class="check-label"><input id="allow-high-ratio" type="checkbox">允许高压缩比文件（仍受字节限额约束）</label></div>` +
            footer(
                cancelButton() +
                    `<button id="start-extract" class="button primary">开始解压</button>`,
            ),
        "extract",
        "E01",
        "#start-extract",
    );
    bindClose();
    const check = () => {
        state.plan.target = finalTarget();
        const denied =
            $("#target-base").value.includes("系统") ||
            !$("#target-base").value.trim();
        $("#target-preview").textContent = "最终位置：" + state.plan.target;
        $("#target-valid").className =
            "field-note " + (denied ? "error" : "success");
        $("#target-valid").innerHTML =
            `${icon(denied ? "status-warning" : "success")}<span>${
                denied
                    ? "此位置不可写，请更改目标文件夹。"
                    : "目标位置可用（示例检查）"
            }</span>`;
        $("#start-extract").disabled = denied;
    };
    $("#target-base").oninput = check;
    $("#same-folder").onchange = check;
    $("#change-target").onclick = () => {
        $("#target-base").value = "下载";
        check();
    };
    $("#conflict-policy").onchange = () => {
        $("#policy-note").textContent =
            $("#conflict-policy").value === "overwrite"
                ? "将替换已有文件内容，请确认你需要保留的版本。"
                : "保留已有文件，按所选策略处理。";
        $("#policy-note").className =
            "field-note " +
            ($("#conflict-policy").value === "overwrite" ? "error" : "");
    };
    $("#start-extract").onclick = () => {
        const range = $("input[name=scope]:checked").value;
        state.plan.policy = $("#conflict-policy").value;
        state.plan.open = $("#open-after").checked;
        state.plan.allowHighRatio = $("#allow-high-ratio").checked;
        startTask(scopeFiles(range), "extract");
    };
    check();
}
function startTask(items, kind) {
    window.XdzipMain?.resetJobs();
    clearInterval(timer);
    state.task = {
        kind,
        items: [...items],
        status: "running",
        progress: 0,
        target: kind === "create" ? state.createTarget : state.plan.target,
        policy: state.plan?.policy || "ask",
        open: kind === "extract" && state.plan.open,
        skipped: [],
        renamed: false,
        archiveTitle: state.archiveTitle,
        createdAt: Date.now(),
        result: null,
    };
    updateTaskCount();
    if (kind === "extract" && state.scenario === "conflict") {
        if (state.task.policy === "ask") {
            state.task.status = "conflict";
            showConflict();
            return;
        }
        applyConflict(state.task.policy, false);
    }
    showProgress();
    runTimer();
}
function showConflict() {
    const t = state.task,
        e = t.items.find((f) => f.id === "guide") || t.items[0];
    t.conflictEntry = e;
    dialog(
        header(
            "目标位置已有同名文件",
            "请选择要保留的版本",
            "document-conflict",
            false,
        ) +
            `<div class="modal-content"><div class="conflict-name">${icon(e.type.includes("PDF") ? "conflict-pdf" : fileIcon(e))}${esc(e.name)}</div><div class="comparison"><div><span class="comparison-artwork">${icon(fileIcon(e))}</span><label>已有文件</label><strong>${bytes(Math.round(e.size * 0.875))}</strong><small>2026/10/01 · 09:42</small></div><div><span class="comparison-artwork">${icon(e.type.includes("PDF") ? "conflict-pdf" : fileIcon(e))}</span><label>压缩包中的文件</label><strong>${bytes(e.size)}</strong><small>2026/10/08 · 14:32</small></div></div><label class="check-label"><input id="apply-conflict-all" type="checkbox">对本次任务后续同类冲突应用此选择</label><div class="notice warning"><span class="notice-artwork">${icon("conflict-warning")}</span><span>覆盖会替换已有文件内容。选择“自动改名”可以保留两个版本。</span></div></div>` +
            footer(
                `<button class="button" id="skip-conflict">跳过</button><button class="button primary" id="rename-conflict">自动改名</button><button class="button danger" id="overwrite-conflict">覆盖文件</button>`,
                "等待你的决定",
            ),
        "conflict",
        "E02",
        "#skip-conflict",
    );
    for (const [id, policy] of [
        ["skip-conflict", "skip"],
        ["rename-conflict", "rename"],
        ["overwrite-conflict", "overwrite"],
    ])
        $("#" + id).onclick = () => {
            t.applyAll = $("#apply-conflict-all").checked;
            applyConflict(policy);
        };
}
function applyConflict(policy, resume = true) {
    const t = state.task,
        e =
            t.conflictEntry ||
            t.items.find((f) => f.id === "guide") ||
            t.items[0];
    t.conflictEntry = e;
    t.conflictDecision = policy;
    if (policy === "skip") t.skipped = [e.id];
    t.renamed = policy === "rename";
    t.status = "running";
    if (resume) {
        showProgress();
        runTimer();
    }
}
function showProgress() {
    const t = state.task;
    dialog(
        header(
            t.kind === "create"
                ? "正在创建压缩文件"
                : t.status === "stopping"
                  ? "正在停止任务"
                  : "正在解压",
            `${t.kind === "create" ? "旅行资料" : t.archiveTitle || "北境旅行.zip"}`,
            t.status === "stopping"
                ? "spinner"
                : t.kind === "create"
                  ? "archive-add"
                  : document.body.dataset.platform === "windows"
                    ? "windows-document"
                    : document.body.dataset.platform === "linux"
                      ? "linux-document"
                      : "document-download",
            false,
        ) +
            `<div class="modal-content"><section class="path-card"><h2>解压到</h2><div>${icon("folder")}<span>${esc(t.target)}</span></div></section><section class="progress-card"><div class="progress-line"><div class="progress-track" role="progressbar" aria-label="任务进度" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${t.progress}"><i id="progress-fill" style="width:${t.progress}%"></i></div><strong id="progress-value">${t.progress}%</strong></div><div class="progress-current"><span id="progress-bytes"></span><span id="progress-count"></span></div><div class="task-metrics"><div><span>速度</span><strong>6.2 MB/s</strong></div><div><span>预计剩余</span><strong>约 5 秒</strong></div><div><span>当前阶段</span><strong id="progress-phase">正在准备</strong></div></div></section><section class="path-card"><h2>当前文件</h2><div><span id="progress-file-icon">${icon("image-file")}</span><span id="progress-file"></span></div></section><p class="info-line">${icon("info")}实际耗时取决于文件与磁盘状态。</p></div><footer class="modal-footer"><span class="info-line">${icon("info")}关闭此窗口后，任务仍会继续。</span><button id="background-task" class="button">返回浏览</button><button id="cancel-task" class="button" ${t.status === "stopping" ? "disabled" : ""}>${t.status === "stopping" ? "正在停止…" : "取消任务"}</button></footer>`,
        "progress",
        "T01",
        "#background-task",
    );
    $("#cancel-task").onclick = cancelTask;
    $("#background-task").onclick = closeModal;
    updateProgress();
}
function updateProgress() {
    const t = state.task;
    if (!t || state.modal !== "progress") return;
    const total = t.items.reduce((n, e) => n + e.size, 0),
        done = Math.min(
            t.items.length - 1,
            Math.floor((t.progress / 100) * t.items.length),
        );
    $("#progress-value").textContent = t.progress + "%";
    $("#progress-fill").style.transition = t.reviewPaused ? "none" : "";
    $("#progress-fill").style.width = t.progress + "%";
    $(".progress-track").setAttribute("aria-valuenow", t.progress);
    $("#progress-phase").textContent =
        t.status === "stopping"
            ? "正在清理未完成文件"
            : t.progress < 15
              ? "正在准备"
              : t.progress > 87
                ? "正在校验"
                : "正在处理文件";
    $("#progress-bytes").textContent =
        `${bytes(Math.round((total * t.progress) / 100))} / ${bytes(total)}`;
    $("#progress-count").textContent =
        `${Math.floor((t.progress / 100) * t.items.length)} / ${t.items.length} 项`;
    $("#progress-file-icon").innerHTML = icon(
        t.items[done].type.includes("JPEG")
            ? "progress-image"
            : fileIcon(t.items[done]),
    );
    $("#progress-file").textContent =
        (t.status === "stopping" ? "停止中：" : "") + t.items[done].name;
}
function runTimer() {
    clearInterval(timer);
    timer = setInterval(() => {
        const t = state.task;
        if (!t || !["running", "conflict"].includes(t.status)) return;
        const next = t.progress + 7;
        if (["disk", "source"].includes(state.scenario) && next >= 70) {
            t.progress = 68;
            finishTask(state.scenario === "disk" ? "partial" : "source");
            return;
        }
        t.progress = Math.min(100, next);
        updateProgress();
        if (t.progress === 100) finishTask("success");
    }, 420);
}
function cancelTask() {
    const t = state.task;
    if (!t || !["running", "conflict"].includes(t.status)) return;
    clearInterval(timer);
    t.status = "stopping";
    showProgress();
    setTimeout(() => {
        if (state.task === t && t.status === "stopping")
            finishTask("cancelled");
    }, 700);
}
function finishTask(result) {
    clearInterval(timer);
    const t = state.task;
    t.status = "finished";
    t.result = result;
    const eligible = t.items.filter((e) => !t.skipped.includes(e.id));
    const done =
        result === "success"
            ? eligible.length
            : t.kind === "create"
              ? 0
              : Math.min(
                    eligible.length - 1,
                    Math.floor((t.progress / 100) * eligible.length),
                );
    t.completed = eligible
        .slice(0, Math.max(0, done))
        .map((entry) =>
            t.renamed && entry.id === t.conflictEntry.id
                ? { ...entry, name: entry.name.replace(/(\.[^.]+)$/, " (2)$1") }
                : entry,
        );
    t.failed = eligible.slice(Math.max(0, done));
    if (t.kind === "create" && result === "success") {
        t.completed = [
            { name: t.target, size: t.items.reduce((n, e) => n + e.size, 0) },
        ];
        t.failed = [];
    }
    updateTaskCount();
    if (state.modal === "progress") showResult();
}
function showResult() {
    const t = state.task,
        ok = t.result === "success",
        cancel = t.result === "cancelled",
        created = t.kind === "create",
        title = ok
            ? created
                ? "压缩文件已创建"
                : "解压完成"
            : cancel
              ? "任务已取消"
              : t.result === "source"
                ? "源文件已发生变化"
                : "部分文件未能完成";
    const successCount = created ? (ok ? 1 : 0) : t.completed.length,
        failedCount = created ? (ok ? 0 : 1) : t.failed.length;
    dialog(
        header(
            "任务结果",
            created ? "新建压缩文件" : esc(t.archiveTitle),
            ok
                ? "status-success"
                : cancel
                  ? "cancelled-large"
                  : "document-warning",
        ) +
            `<div class="modal-content"><div class="result-hero"><div class="result-icon">${icon(ok ? "status-success" : cancel ? "cancelled-large" : "document-warning")}</div><h2>${title}</h2><p>${ok ? (created ? "归档已完成校验并保存。" : "所选内容已处理完成。") : created ? "临时归档未发布，原有文件保持不变。" : "已经完成的文件会保留，不会因本次停止而删除。"}</p></div><div class="result-counts"><div><b>${successCount}</b><span>${icon("status-partial-success")}${created ? "已创建归档" : "成功文件"}</span></div><div><b>${created ? 0 : t.skipped.length}</b><span>${icon("status-skipped")}跳过</span></div><div><b>${failedCount}</b><span>${icon(cancel ? "status-stopped" : "status-warning")}${cancel ? "未完成" : "失败"}</span></div></div>${!ok ? `<div class="notice ${cancel ? "" : "error"}">${cancel ? "任务已安全停止。可以重新规划未完成项。" : t.result === "source" ? "压缩文件在读取后被修改。请重新打开并重新选择内容。" : "目标磁盘空间不足。释放空间或选择其他位置后重试。"}</div>` : ""}<div class="result-destination">${icon("folder")}<span>${esc(t.target)}</span></div>${t.renamed ? `<p class="field-note">重名文件已改名为 ${esc(t.conflictEntry.name.replace(/(\.[^.]+)$/, " (2)$1"))}</p>` : ""}<details><summary class="details-title">查看处理明细</summary><div class="result-list">${t.completed.map((e) => `<div><span>${esc(e.name)}</span><span>已完成</span></div>`).join("")}${t.failed.map((e) => `<div class="bad"><span>${esc(e.name)}</span><span>${cancel ? "未完成" : "失败"}</span></div>`).join("")}${t.skipped.map((id) => `<div><span>${esc(byId.get(id).name)}</span><span>已跳过</span></div>`).join("")}</div></details></div>` +
            footer(
                cancelButton("完成") +
                    (t.result === "source"
                        ? '<button id="reopen-result" class="button primary">重新打开</button>'
                        : !ok && failedCount
                          ? '<button id="retry-result" class="button primary">重试未完成项</button>'
                          : '<button id="reveal-result" class="button primary">打开目标文件夹</button>'),
            ),
        "result",
        "R01",
    );
    bindClose();
    if ($("#reveal-result"))
        $("#reveal-result").onclick = () =>
            toast("原型演示：正式产品将在系统文件管理器中打开目标位置。");
    if ($("#reopen-result"))
        $("#reopen-result").onclick = () => {
            closeModal();
            openPicker();
        };
    if ($("#retry-result"))
        $("#retry-result").onclick = () => {
            state.scenario = "normal";
            $("#scenario").value = "normal";
            if (created) showCreate();
            else showExtract("retry", t.failed);
        };
    if (ok && t.open) toast("原型演示：已触发“完成后打开目标文件夹”。");
}
function showTask() {
    if (!state.task) {
        toast("目前没有后台任务。");
        return;
    }
    if (state.task.status === "conflict") showConflict();
    else if (state.task.status === "finished") showResult();
    else showProgress();
}
function showCreate() {
    if (
        state.task &&
        ["running", "stopping", "conflict"].includes(state.task.status)
    ) {
        showTask();
        return;
    }
    state.createItems = [...roots.slice(0, 3)];
    dialog(
        header("新建压缩文件", "添加内容，选择格式与保存位置", "archive-add") +
            `<div class="modal-content"><div class="field-group"><div class="field-label">待压缩内容<span id="create-size"></span></div><div id="create-list" class="create-list"></div><div class="create-actions"><button id="add-input" class="button">${icon("plus")}添加文件</button><button id="clear-input" class="button">清空</button></div></div><div class="field-group"><label for="create-name" class="field-label">压缩文件名称</label><input id="create-name" class="text-input" value="旅行资料.zip"></div><div class="field-group"><label for="create-destination" class="field-label">保存到</label><input id="create-destination" class="text-input" value="下载"></div><div class="two-fields"><div class="field-group"><label for="create-format" class="field-label">归档格式</label><select id="create-format" class="field-select"><option value="zip">ZIP</option><option value="tar">TAR</option><option value="tar.gz">TAR.GZ</option></select></div><div class="field-group" id="method-group"><label for="create-method" class="field-label">压缩方式</label><select id="create-method" class="field-select"><option value="deflate">Deflate</option><option value="store">Store · 不压缩</option></select></div></div><div class="field-group" id="level-group"><label for="create-level" class="field-label">压缩级别</label><select id="create-level" class="field-select"><option>均衡</option><option>快速</option><option>更小</option></select></div><p id="create-valid" class="field-note success"></p><details><summary class="details-title">更多选项</summary><div class="details-body"><label class="check-label"><input id="create-existing" type="checkbox">目标位置已有同名归档（演示覆盖确认）</label></div></details></div>` +
            footer(
                cancelButton() +
                    `<button id="start-create" class="button primary">开始压缩 ${icon("arrow")}</button>`,
            ),
        "create",
        "C01",
    );
    bindClose();
    const validate = () => {
        const valid =
            state.createItems.length &&
            $("#create-name").value.trim() &&
            $("#create-destination").value.trim();
        $("#start-create").disabled = !valid;
        $("#create-valid").innerHTML =
            `${icon(valid ? "success" : "status-warning")}<span>${
                valid
                    ? "预检通过 · " +
                      state.createItems.length +
                      " 个文件 · 目标可写"
                    : "请添加输入文件，并填写名称与保存位置。"
            }</span>`;
        $("#create-valid").className =
            "field-note " + (valid ? "success" : "error");
    };
    const draw = () => {
        $("#create-list").innerHTML = state.createItems.length
            ? state.createItems
                  .map(
                      (e, i) =>
                          `<div class="create-row">${icon(fileIcon(e))}<span>${esc(e.name)}</span><span>${bytes(e.size)}</span><button class="icon-button" data-remove="${i}" aria-label="移除 ${esc(e.name)}">${icon("close")}</button></div>`,
                  )
                  .join("")
            : '<div class="create-row">尚未添加内容</div>';
        $("#create-size").textContent = bytes(
            state.createItems.reduce((n, e) => n + e.size, 0),
        );
        $("#create-list")
            .querySelectorAll("[data-remove]")
            .forEach(
                (b) =>
                    (b.onclick = () => {
                        state.createItems.splice(Number(b.dataset.remove), 1);
                        draw();
                    }),
            );
        validate();
    };
    $("#add-input").onclick = () => {
        const next = roots.find(
            (e) => !state.createItems.some((f) => f.id === e.id),
        );
        if (next) {
            state.createItems.push(next);
            draw();
        } else toast("所有示例文件都已添加。");
    };
    $("#clear-input").onclick = () => {
        state.createItems = [];
        draw();
    };
    $("#create-name").oninput = validate;
    $("#create-destination").oninput = validate;
    const format = () => {
        const kind = $("#create-format").value;
        $("#create-name").value =
            $("#create-name").value.replace(/\.(zip|tar(?:\.gz)?)$/, "") +
            "." +
            kind;
        $("#method-group").hidden = kind !== "zip";
        $("#level-group").hidden =
            kind === "tar" ||
            (kind === "zip" && $("#create-method").value === "store");
        validate();
    };
    $("#create-format").onchange = format;
    $("#create-method").onchange = format;
    $("#start-create").onclick = () => {
        state.createTarget =
            $("#create-destination").value.trim() +
            " / " +
            $("#create-name").value.trim();
        state.createFormat = $("#create-format").value;
        state.createMethod =
            state.createFormat === "zip"
                ? $("#create-method").value
                : state.createFormat === "tar"
                  ? "none"
                  : "gzip";
        state.createLevel = $("#create-level").value;
        if ($("#create-existing").checked) confirmCreateOverwrite();
        else startTask(state.createItems, "create");
    };
    draw();
}
function confirmCreateOverwrite() {
    dialog(
        header(
            "替换已有压缩文件？",
            esc(state.createTarget),
            "archive-warning",
            false,
        ) +
            `<div class="modal-content"><section class="existing-archive"><span class="existing-archive-artwork">${icon("existing-archive")}</span><div><strong>已有压缩文件</strong><p>${esc(state.createTarget)}</p></div></section><div class="notice warning">已有归档会在新归档成功写入、校验并提交后被替换。取消或失败不会先删除已有文件。</div></div>` +
            footer(
                `<button id="cancel-create-overwrite" class="button">返回修改</button><button id="confirm-create-overwrite" class="button danger">替换并创建</button>`,
                "需要确认",
            ),
        "create-overwrite",
        "C01",
        "#cancel-create-overwrite",
    );
    $("#cancel-create-overwrite").onclick = showCreate;
    $("#confirm-create-overwrite").onclick = () =>
        startTask(state.createItems, "create");
}
function showSettings() {
    dialog(
        header("设置", "外观与隐私", "settings-tile") +
            `<div class="modal-content"><fieldset id="setting-theme" class="settings-theme"><legend>外观主题</legend><div class="theme-cards"><label class="theme-card">${icon("theme-system")}<span><input type="radio" name="setting-theme" value="system">跟随系统</span></label><label class="theme-card">${icon("theme-light")}<span><input type="radio" name="setting-theme" value="light">浅色</span></label><label class="theme-card">${icon("theme-dark")}<span><input type="radio" name="setting-theme" value="dark">深色</span></label></div></fieldset><div class="settings-row"><label for="setting-density">列表密度</label><select id="setting-density" class="field-select"><option value="standard">标准</option><option value="compact">紧凑</option></select></div><label class="check-label"><input id="recent-enabled" type="checkbox" ${state.recent ? "checked" : ""}>在本机保留最近文件记录</label><div class="field-group"><button id="clear-recent" class="button">清空最近记录</button></div><div class="notice">所有归档处理都在本机进行。默认不上传文件、不启用遥测。清空最近记录不会删除原文件。</div></div>` +
            footer(cancelButton("完成"), "设置仅在本机生效"),
        "settings",
        "S01",
    );
    bindClose();
    $(`input[name="setting-theme"][value="${state.theme}"]`).checked = true;
    $("#setting-theme").onchange = (e) => {
        if (e.target.matches('input[name="setting-theme"]'))
            setTheme(e.target.value);
    };
    $("#setting-density").onchange = (e) => {
        document.documentElement.style.setProperty(
            "--row-height",
            e.target.value === "compact" ? "32px" : "40px",
        );
        document
            .querySelectorAll("td")
            .forEach((td) => (td.style.height = "var(--row-height)"));
    };
    $("#recent-enabled").onchange = (e) => (state.recent = e.target.checked);
    $("#clear-recent").onclick = () =>
        toast("最近记录已清空，原文件未被删除。");
}
function setTheme(theme) {
    XdzipControls.close({ restoreFocus: false });
    state.theme = theme;
    const dark =
        theme === "dark" ||
        (theme === "system" &&
            matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.dataset.theme = dark ? "dark" : "light";
    const tick = window.XDZIP_ASSETS.icons[
        dark ? "dark" : "light"
    ].check.replace(/stroke="[^"]*"/, 'stroke="white"');
    document.documentElement.style.setProperty(
        "--checkbox-tick",
        `url("data:image/svg+xml,${encodeURIComponent(tick)}")`,
    );
    document.documentElement.style.setProperty(
        "--checkbox-selected",
        `url("data:image/svg+xml,${encodeURIComponent(icon("check-selected"))}")`,
    );
    document.documentElement.style.setProperty(
        "--radio-selected",
        `url("data:image/svg+xml,${encodeURIComponent(icon("radio-selected"))}")`,
    );
    document.documentElement.style.setProperty(
        "--radio-unselected",
        `url("data:image/svg+xml,${encodeURIComponent(icon("radio-unselected"))}")`,
    );
    const selectedTheme = $(`input[name="setting-theme"][value="${theme}"]`);
    if (selectedTheme) selectedTheme.checked = true;
    document.querySelectorAll("svg[data-asset]").forEach((el) => {
        el.outerHTML = icon(el.dataset.asset, el.getAttribute("class") || "");
    });
    $("#theme").innerHTML = icon(dark ? "sun" : "moon");
    $("#theme").setAttribute(
        "aria-label",
        dark ? "切换浅色主题" : "切换深色主题",
    );
}
function scenario(value) {
    clearInterval(timer);
    clearTimeout(loadTimer);
    state.task = null;
    window.XdzipMain?.resetJobs();
    state.scenario = value;
    state.selected.clear();
    state.path = "";
    state.query = "";
    state.archiveState = "ready";
    $("#search").value = "";
    $("#welcome").hidden = true;
    if ($("#modal").open) closeModal();
    if (["empty", "unsupported", "damaged", "loading"].includes(value)) {
        state.archiveState = value;
        if (value === "loading")
            loadTimer = setTimeout(() => {
                if (state.archiveState === "loading") {
                    state.archiveState = "ready";
                    render();
                    toast("目录读取完成。");
                }
            }, 8000);
    } else if (value === "noresult") {
        state.query = "会议记录";
        $("#search").value = state.query;
    } else if (value !== "normal") {
        state.selected = new Set(["lake", "sunrise", "guide"]);
    }
    render();
    if (["conflict", "permission", "disk", "source", "cancel"].includes(value))
        showExtract("selected");
}
$("#app-brand").innerHTML =
    icon("app-mark") + '<span class="brand-label">xdzip</span>';
hydrate();
// Font loading and wrapping may change a search notice after its first render.
const previewLayoutObserver = new ResizeObserver(syncInspectorContentOffset);
previewLayoutObserver.observe($("#search-notice"));
setTheme("light");
if (innerWidth <= 1050) $("#inspector").classList.add("collapsed-narrow");
$("#scenario").onchange = (e) => scenario(e.target.value);
$("#theme").onclick = () =>
    setTheme(
        document.documentElement.dataset.theme === "dark" ? "light" : "dark",
    );
$("#spec-toggle").onclick = () => {
    $("#spec").hidden = !$("#spec").hidden;
};
$("#reset").onclick = () => {
    $("#scenario").value = "normal";
    scenario("normal");
};
$("#title-settings").onclick = showSettings;

$("#open").onclick = openPicker;
$("#new-tab").onclick = openPicker;
$("#close-archive").onclick = showWelcome;
$("#create").onclick = showCreate;
$("#properties-toggle").onclick = () => {
    const n = $("#inspector");
    if (n.hidden || n.classList.contains("collapsed-narrow")) {
        n.hidden = false;
        n.classList.remove("collapsed-narrow");
    } else n.hidden = true;
};
$("#extract").onclick = () =>
    showExtract(state.selected.size ? "selected" : "all");
// Shared menus, command state and contextual selection are installed by main-controls.js.

let composing = false;
const query = () => {
    if (composing) return;
    state.query = $("#search").value.trim();
    state.selected.clear();
    state.anchor = null;
    render();
};
$("#search").addEventListener("compositionstart", () => (composing = true));
$("#search").addEventListener("compositionend", () => {
    composing = false;
    query();
});
$("#search").oninput = query;
$("#select-all").onchange = (e) => {
    state.selected = e.target.checked
        ? new Set(visibleEntries().map((x) => x.id))
        : new Set();
    render();
};
$("#sort-name").onclick = $("#sort-direction").onclick = () => {
    state.sort = state.sort === 1 ? -1 : 1;
    render();
};
$("#tasks").onclick = showTask;
$("#settings").onclick = showSettings;
$("#platform").onchange = (event) => {
    document.body.dataset.platform = event.target.value;
    if ($("#modal").open) closeModal();
    $("#search").setAttribute(
        "aria-keyshortcuts",
        event.target.value === "macos" ? "Meta+F" : "Control+F",
    );
};
$("#modal").addEventListener("cancel", (event) => {
    event.preventDefault();
    windowClose();
});
document.addEventListener("keydown", (e) => {
    if (e.defaultPrevented || e.isComposing || XdzipControls.isOpen) return;
    if (
        e.key === "Escape" &&
        $("#modal").open &&
        $("#modal").dataset.modeless === "true"
    ) {
        e.preventDefault();
        windowClose();
        return;
    }
    if (e.key === "Escape" && !$("#modal").open) {
        XdzipControls.close();
        if (document.activeElement === $("#search")) {
            $("#search").value = "";
            query();
        }
    }
    if ((e.metaKey || e.ctrlKey) && !$("#modal").open) {
        if (e.key.toLowerCase() === "f") {
            e.preventDefault();
            $("#search").focus();
        }
        if (e.key.toLowerCase() === "o") {
            e.preventDefault();
            openPicker();
        }
        if (e.key.toLowerCase() === "n") {
            e.preventDefault();
            showCreate();
        }
        if (
            e.key.toLowerCase() === "a" &&
            !e.target.matches("input,textarea")
        ) {
            e.preventDefault();
            state.selected = new Set(visibleEntries().map((x) => x.id));
            render();
        }
    }
});
render();

// Public review API: UI demonstrations only, no file IO or codecs.
window.XdzipDesign = {
    open(view) {
        if (state.task?.reviewPaused) state.task = null;
        if ($("#modal").open) closeModal();
        // Artboard changes restore the ready demo fixture after an error scene.
        // Active user-started demo tasks retain their own background state.
        if (
            ["main", "extract", "create", "settings", "progress"].includes(view)
        ) {
            state.archiveState = "ready";
            state.scenario = "normal";
        }
        if (view === "main") {
            window.XdzipMain?.openSample("travel");
            state.path = "";
            state.query = "";
            $("#search").value = "";
            state.selected = new Set(["lake"]);
            render();
            return;
        }
        if (view === "extract") {
            state.selected = new Set(["lake", "sunrise", "guide"]);
            render();
            showExtract("selected");
        } else if (view === "create") showCreate();
        else if (view === "settings") showSettings();
        else if (view === "progress") {
            showExtract("all");
            startTask(files, "extract");
            clearInterval(timer);
            state.task.progress = 60;
            state.task.reviewPaused = true;
            updateProgress();
        } else scenario(view);
    },
    theme: setTheme,
    platform(value) {
        document.body.dataset.platform = value;
    },
};
window.XdzipControls.configure({ icon });
function setCanvasLogicalScale(view) {
    const spec = window.XDZIP_DISPLAY_SPEC;
    const main = ["main", "empty", "damaged"].includes(view);
    document.documentElement.style.setProperty("--ui-source-factor", 1 / (main ? spec.sourceToLogical : spec.operationSourceToLogical));
}
const reviewQuery = new URLSearchParams(location.search);
setCanvasLogicalScale(reviewQuery.get("view") || "main");
if (reviewQuery.get("standalone") === "1")
    document.body.dataset.standalone = "true";
if (reviewQuery.has("platform"))
    window.XdzipDesign.platform(reviewQuery.get("platform"));
if (reviewQuery.has("view")) window.XdzipDesign.open(reviewQuery.get("view"));

// Only the containing review page may configure this demonstration canvas.
window.addEventListener("message", (event) => {
    const config = event.data;
    if (
        event.source !== window.parent ||
        config?.type !== "xdzip-review-configure"
    )
        return;
    if (
        ![
            "main",
            "extract",
            "create",
            "progress",
            "conflict",
            "settings",
            "empty",
            "damaged",
        ].includes(config.view) ||
        !["macos", "windows", "linux"].includes(config.platform) ||
        !["light", "dark"].includes(config.theme)
    )
        return;
    document.body.dataset.standalone = ["main", "empty", "damaged"].includes(
        config.view,
    )
        ? "false"
        : "true";
    setCanvasLogicalScale(config.view);
    XdzipDesign.platform(config.platform);
    XdzipDesign.theme(config.theme);
    XdzipDesign.open(config.view);
});
if (window.parent !== window)
    window.parent.postMessage({ type: "xdzip-review-ready" }, "*");
