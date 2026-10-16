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
    `<img class="${cls}" src="${window.XDZIP_ASSETS.photos[key]}" alt="${key.startsWith("lake") ? "冰川湖" : "山间日出"}，设计示例图片">`;
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
const imagePropertyOrder = ["尺寸", "原始大小", "压缩大小", "修改日期"];
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
    P01: ["解压元数据预检", "仅检查元数据", "展开所选目录并检查路径、容量与碰撞", "问题列表滚动，返回浏览不改变选择", "不允许输出，不替代真实内容验证"],
    W01: ["欢迎页", "设计演示，不读取压缩包或写入文件", "打开与新建为主要入口，文件在本机处理。", "支持键盘、拖入反馈、打开后浏览以及取消返回。", "文件选择与拖入仅选择示例数据，不读取用户文件。"],
    B01: [
        "压缩包浏览",
        "P01 / P02 / P08，AC-01 / AC-02 / AC-08",
        "打开后只读取目录和属性，不解压内容。目录双击进入；条目用稳定 ID 选择。",
        "单击、复选、Shift 范围选择、⌘/Ctrl+A。Enter 进入文件夹或查看属性。",
        "索引未完成时禁用全部解压；空压缩包与无搜索结果是不同状态。",
    ],
    B02: [
        "搜索与选择",
        "P02 / P03，AC-02 / AC-03",
        "搜索覆盖当前目录及其后代；结果显示来源路径。无选择时全部解压始终指整个压缩包。",
        "选择解压所选、全部内容或显式的搜索结果范围。清空搜索恢复原目录。",
        "输入法组合期间不提交查询。切换列表清除选择，避免隐藏选择造成范围误解。",
    ],
    E01: [
        "解压确认",
        "P03 / P07，AC-03 / AC-07",
        "路径预览只追加一次同名目录。所选项和全部内容具有不同计数与大小。",
        "路径、范围、重名策略均可修改。权限恢复后才允许开始；Esc 取消并恢复焦点。",
        "原型只模拟预检，不代表真实权限、安全路径或文件系统验证。",
    ],
    E02: [
        "重名决策",
        "P03 / P07，AC-03 / AC-07",
        "保留已有文件，等待跳过、自动改名或覆盖。覆盖不作为默认 Enter 操作。",
        "默认焦点为跳过。决定仅针对本次任务，可选择对后续同类冲突应用。",
        "覆盖前需由正式核心再次核验目标身份；安全错误不能通过覆盖绕过。",
    ],
    T01: [
        "任务进行中",
        "P06 / P15，AC-06 / AC-14",
        "演示准备、执行、校验、停止与结果；计数对应同一任务范围。",
        "取消后先进入正在停止，再显示已保留和未完成项；返回浏览不取消任务。",
        "所有进度是示例状态，不代表真实吞吐。任务详情可再次打开。",
    ],
    R01: [
        "任务结果",
        "P03 / P04 / P15，AC-03 / AC-04 / AC-14",
        "成功、跳过、失败或未完成分开显示。部分完成不是全部成功。",
        "重试只包含失败或未完成项，重新进入确认流程。",
        "创建压缩包整体提交；取消前未发布时不算成功创建压缩包。",
    ],
    C01: [
        "创建压缩包",
        "P04 / P05，AC-04 / AC-05",
        "示例输入支持添加、移除。ZIP 可选 Store/Deflate；TAR 无压缩级别，TAR.GZ 使用 GZIP。",
        "更改格式同步扩展名和参数。输入为空或名称为空时不可提交。",
        "首版不提供加密和分卷。已有目标进入独立覆盖确认，取消保留旧压缩包。",
    ],
    S01: [
        "设置",
        "P08 / P11，AC-08 / AC-11",
        "深浅主题和密度即时预览，最近文件记录可清空。",
        "设置仅影响设计原型；关闭返回当前浏览状态。",
        "无联网、遥测和外部扩展安装。手机布局用于审阅，不声明移动端产品。",
    ],
    X01: [
        "异常状态",
        "P05 / P07 / P15，AC-05 / AC-07 / AC-14",
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
    $("#state-label").textContent = `${id}，${s[0]}`;
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
        : e.name.toLowerCase().endsWith(".zip")
          ? "archive"
          : e.type?.includes("JPEG")
          ? "image"
          : e.type?.includes("PDF")
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
            state.archiveState === "empty" ? "空压缩包" : "元数据未就绪";
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
        `当前${state.path ? "目录及后代" : "压缩包"}中找到 ${list.length} 项，搜索结果不等同于全部内容`;
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
        ? `已选择 ${chosen.length} 项，${bytes(sum)}`
        : `${files.length} 个文件，${folders.length} 个文件夹`;
    $("#read-status").textContent =
        state.archiveState === "loading"
            ? "正在读取目录"
            : state.archiveState === "ready"
              ? "目录已读取"
              : state.archiveState === "empty"
                ? "空压缩包"
                : "无法读取";
    if (state.archiveState !== "ready") {
        $("#selection-status").textContent =
            state.archiveState === "empty"
                ? "0 个文件，0 个文件夹"
                : "目录信息未就绪";
    }
    window.XdzipMain?.sync(list);
    renderEmpty(list);
    syncInspectorContentOffset();
    renderInspector(chosen);
    updateTaskCount();
    if (document.body.classList.contains("welcome-mode")) $("#selection-status").textContent = "文件在本机处理";
    if (!state.modal)
        setSpec(
            document.body.classList.contains("welcome-mode")
                ? "W01"
                : state.archiveState !== "ready"
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
        title = "这个压缩包没有内容";
        desc = "目录读取完成，没有可解压的文件。";
        ico = "archive";
        label = "打开其他文件";
        fn = () => openPicker();
    }
    if (state.archiveState === "loading") {
        title = "正在读取目录";
        desc = "正在扫描压缩包目录，总条目数暂时未知。";
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
                : "无法读取压缩包";
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
            `<div class="inspector-head"><span>压缩包信息</span></div><div class="file-emblem">${icon("archive")}</div><h2>${esc(state.archiveTitle)}</h2><p class="inspector-footnote">${state.archiveState === "empty" ? "目录已读取，压缩包中没有可解压的文件。" : "目录信息未就绪，暂时无法展示文件属性。"}</p>`;
        return;
    }
    if (!chosen.length) {
        $("#inspector").innerHTML = `<div class="inspector-head"><span>文件预览</span><button class="icon-button" aria-label="关闭属性面板" id="hide-inspector">${icon("close")}</button></div><div class="file-emblem">${icon("archive")}</div><h2>未选择文件</h2><p class="inspector-footnote">选择文件查看属性</p>`;
        $("#hide-inspector").onclick = () => ($("#inspector").hidden = true);
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
        `<div class="inspector-head"><span>${single?.type.includes("JPEG") ? "文件预览" : chosen.length ? "文件属性" : "压缩包信息"}</span><button class="icon-button" aria-label="关闭属性面板" id="hide-inspector">${icon("close")}</button></div>${single && ["lake", "sunrise"].includes(single.id) ? `<div class="photo-preview">${photo(single.id === "lake" ? "lake-preview" : "sunrise")}</div>` : `<div class="file-emblem">${icon(multi ? "layers" : single ? fileIcon(single) : "archive")}</div>`}<h2>${multi ? `${chosen.length} 个文件` : single ? esc(single.name) : esc(state.archiveTitle)}</h2><span class="type-label">${multi ? "多个类型，已选择" : single ? single.type : "ZIP 压缩包，Deflate"}</span><dl class="property-list">${[
            ["原始大小", bytes(size)],
            ["压缩大小", bytes(packed)],
            [
                single?.type.includes("JPEG") ? "尺寸" : "压缩方式",
                single?.type.includes("JPEG") ? "6000 × 4000" : "Deflate",
            ],
            ["修改日期", single?.modified || "2026/10/08"],
            [
                "位置",
                single && single.parent
                    ? byId.get(single.parent).name
                    : "压缩包根目录",
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
    const count = Number(Boolean(state.task)) + Number(Boolean(state.testJob));
    $("#task-count").textContent = `任务 ${count}`;
    $("#task-count").title = `进行中 ${running} 项`;

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
let applyingReviewPreset = false;
function dialog(html, kind, spec, focus) {
    XdzipControls.close({ restoreFocus: false });
    XdzipControls.hideTooltip();
    const el = $("#modal");
    if (state.modal === "settings") restoreSettingsPreview();
    returnFocus = el.open ? returnFocus : document.activeElement;
    if (el.open) el.close();
    const template = document.createElement("template");
    template.innerHTML = html;
    const title = template.content.querySelector("h1")?.textContent || "xdzip";
    el.innerHTML =
        windowChrome(
            kind === "preflight" ? settingText("ui.ff6f3ae7c4fb") : kind === "archive-test" ? "完整性检查进度" : kind === "progress" ? (state.task?.kind === "create" ? "压缩进度" : "解压进度") : kind === "result" ? (state.task?.kind === "create" ? "压缩结果" : "解压结果") : title,
            ["progress", "archive-test"].includes(kind),
        ) + html;
    el.setAttribute("aria-label", title);
    el.style.left = el.style.top = el.style.margin = el.style.transform = "";
    state.modal = kind;
    el.dataset.kind = kind;
    if (!applyingReviewPreset && document.body.dataset.standalone === "true" && window.XDZIP_DISPLAY_SPEC.views[kind]) {
        setCanvasLogicalScale(kind);
        if (window.parent !== window)
            window.parent.postMessage({type:"xdzip-review-window", view:kind}, "*");
    }
    setSpec(spec);
    el.querySelectorAll("select.field-select").forEach(select => select.dataset.xdzipSelect = "true");
    XdzipControls.enhanceSelects(el);
    XdzipControls.enhanceSegmentedGroups(el);
    // Decision windows block only their owner; task details/settings remain modeless.
    const modeless = [
        "progress",
        "settings",
        "preflight",
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
function closeModal({ preserveReview = false } = {}) {
    if (state.modal === "settings") restoreSettingsPreview();
    if ($("#modal").open) $("#modal").close();
    $("#modal").querySelectorAll('input[type="password"]').forEach(input => input.value = "");
    state.modal = null;
    if (!preserveReview && document.body.dataset.standalone === "true") {
        document.body.dataset.standalone = "false";
        const baseView = document.body.classList.contains("welcome-artboard") ? (document.body.classList.contains("welcome-narrow") ? "welcome-narrow" : "welcome") : "main";
        setCanvasLogicalScale(baseView);
        if (window.parent !== window)
            window.parent.postMessage({type:"xdzip-review-window", view:baseView}, "*");
    }
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
function footer(content, ready = "准备就绪", ico = "shield") {
    return `<footer class="modal-footer${ready ? "" : " action-only-footer"}">${ready ? `<span class="ready-label">${icon(ico)}${ready}</span>` : ""}<div class="operation-footer-actions">${content}</div></footer>`;
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

function setWelcomeMode(visible) {
    document.body.classList.toggle("welcome-mode", visible);
    $("#welcome").hidden = !visible;
    if (!visible) document.body.classList.remove("welcome-artboard", "welcome-narrow");
}
let welcomeOpening = null;
function cancelWelcomeOpening(restoreFocus = true) {
    if (!welcomeOpening) return;
    clearTimeout(welcomeOpening.timer);
    welcomeOpening = null;
    $("#welcome-opening")?.remove();
    const content = $(".welcome-content");
    if (content) content.inert = false;
    if (restoreFocus) $("#welcome-open")?.focus();
}
function beginWelcomeOpening(name, complete = null) {
    if (welcomeOpening) return;
    const token = {};
    welcomeOpening = token;
    $(".welcome-content").inert = true;
    const overlay = document.createElement("div");
    overlay.id = "welcome-opening";
    overlay.className = "welcome-opening";
    overlay.innerHTML = `<section class="welcome-opening-card" role="dialog" aria-modal="true" aria-labelledby="welcome-opening-title"><h2 id="welcome-opening-title">${settingText("ui.821ffb8cd6a2")}</h2><p class="welcome-opening-name">${esc(name)}</p><div class="welcome-opening-progress" role="progressbar" aria-label="${settingText("welcome.opening.progress")}"><span></span></div><p role="status">${settingText("welcome.opening.note")}</p><button id="welcome-opening-cancel" class="button">${settingText("ui.caf06adff4f1")}</button></section>`;
    $("#welcome").append(overlay);
    const cancel = $("#welcome-opening-cancel");
    cancel.onclick = () => cancelWelcomeOpening();
    overlay.onkeydown = event => {
        if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); cancelWelcomeOpening(); }
        if (event.key === "Tab") { event.preventDefault(); cancel.focus(); }
    };
    cancel.focus();
    // Demonstrate a cancellable wait; this prototype never reads user archive bytes.
    if (complete) token.timer = setTimeout(() => {
        if (welcomeOpening !== token) return;
        cancelWelcomeOpening(false);
        complete();
    }, 500);
}
function showWelcome() {
    cancelWelcomeOpening(false);
    state.selected.clear();
    setWelcomeMode(true);
    const shortcut = document.body.dataset.platform === "macos" ? "⌘ " : "Ctrl+";
    $("#welcome").innerHTML = `<div class="welcome-content"><div class="welcome-identity">${icon("app-mark")}<span>xdzip</span></div><h1>打开压缩包，从容整理。</h1><p class="welcome-subtitle">先查看内容，再选择解压。也可以把文件打包，轻松分享。</p><div class="welcome-actions"><button class="welcome-action primary" id="welcome-open">${icon("folderOpen")}<span><strong>打开压缩包</strong><small>浏览内容，选择需要解压的文件</small><span class="welcome-shortcut">${shortcut}O</span></span></button><button class="welcome-action" id="welcome-create">${icon("new")}<span><strong>创建压缩包</strong><small>添加文件或文件夹，创建压缩包</small><span class="welcome-shortcut">${shortcut}N</span></span></button></div><div class="welcome-drop" id="welcome-drop">${icon("archive-grey")}<span>也可以将压缩包拖到这里</span></div><div class="welcome-formats"><span>支持打开与创建</span><b>ZIP</b><b>TAR</b><b>TAR.GZ</b></div><p class="welcome-feedback" role="status" aria-live="polite"></p></div>`;
    $("#welcome-open").onclick = openPicker;
    $("#welcome-create").onclick = showCreate;
    const resetDrop = () => {
        $("#welcome-drop").classList.remove("drag-over");
        $("#welcome-drop span").textContent = "也可以将压缩包拖到这里";
    };
    const validName = name => /\.(zip|tar|tar\.gz)$/i.test(name);
    const dragKind = transfer => {
        const items = [...(transfer?.items || [])];
        if (items.length !== 1 || items[0].kind !== "file" || items[0].webkitGetAsEntry?.()?.isDirectory) return "invalid";
        const file = items[0].getAsFile();
        return file ? (validName(file.name) ? "valid" : "invalid") : "unknown";
    };
    const zone = $("#welcome");
    zone.ondragenter = event => {
        event.preventDefault();
        const kind = dragKind(event.dataTransfer);
        $("#welcome-drop").classList.toggle("drag-over", kind === "valid");
        $("#welcome-drop span").textContent = kind === "valid" ? "松开以选择对应示例，不读取拖入文件" : kind === "unknown" ? "松开以检查格式，原型不读取文件内容" : "请拖入单个 ZIP、TAR 或 TAR.GZ 压缩包";
    };
    zone.ondragover = event => {
        event.preventDefault();
        if (event.dataTransfer) event.dataTransfer.dropEffect = dragKind(event.dataTransfer) === "invalid" ? "none" : "copy";
    };
    zone.ondragleave = event => {
        if (!zone.contains(event.relatedTarget)) resetDrop();
    };
    zone.ondrop = event => {
        event.preventDefault();
        resetDrop();
        const dropped = [...(event.dataTransfer?.files || [])];
        const directory = [...(event.dataTransfer?.items || [])].some(item => item.webkitGetAsEntry?.()?.isDirectory);
        if (directory || dropped.length !== 1 || !validName(dropped[0].name)) {
            $(".welcome-feedback").textContent = "请拖入单个 ZIP、TAR 或 TAR.GZ 压缩包。此原型不读取文件内容。";
            return;
        }
        // Validate the name only; never read user bytes.
        openPicker();
        toast("拖入演示，不读取文件内容。请选择示例压缩包继续。");
    };
    render();
    $("#state-label").textContent = "W01，欢迎页，设计演示";
}

// Keep a file dropped on window chrome from navigating away from the prototype.
for (const eventName of ["dragover", "drop"]) document.addEventListener(eventName, event => {
    if (document.body.classList.contains("welcome-mode")) event.preventDefault();
});
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
    const folder = state.archiveTitle.replace(/\.(tar\.gz|[^.]+)$/i, "");
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
        target: "下载 / " + state.archiveTitle.replace(/\.(tar\.gz|[^.]+)$/i, ""),
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
            "解压",
            `${esc(state.archiveTitle)}，${scope === "all" ? "整个压缩包" : scope === "results" ? "当前搜索结果" : scope === "retry" ? "重试范围" : "所选文件"}`,
            "archive",
        ) +
            `<div class="modal-content"><div class="field-group"><label class="field-label" for="target-base">解压到</label><div class="input-row destination-field">${icon("folder")}<input id="target-base" class="text-input" value="${state.scenario === "permission" ? "系统 / 受保护目录" : esc(state.plan.target)}"><button class="button" id="change-target">更改</button></div><label class="check-label destination-check"><span class="destination-mark"><input id="same-folder" type="checkbox" checked></span><span class="destination-text">创建同名文件夹</span></label><p id="target-preview" class="field-note"></p><p id="target-valid" class="field-note success"></p></div><div class="field-group scope-card"><div class="field-label">解压范围</div><div class="scope-box">${
                scope !== "all"
                    ? `<label class="radio-row"><input type="radio" name="scope" value="${scope}" checked><span>${scope === "results" ? "当前搜索结果" : scope === "retry" ? "失败或未完成项" : "所选文件"}</span><small>${count} 项，${bytes(sum)}</small></label><div class="scope-files">${chosen
                          .slice(0, 3)
                          .map(
                              (e) =>
                                  `<div class="scope-file">${["lake", "sunrise"].includes(e.id) ? photo(e.id) : icon(fileIcon(e))}<span>${esc(e.name)}</span><span>${bytes(e.size)}</span></div>`,
                          )
                          .join(
                              "",
                          )}${count > 3 ? `<br>另有 ${count - 3} 个文件` : ""}</div>`
                    : ""
            }<label class="radio-row"><input type="radio" name="scope" value="all" ${scope === "all" ? "checked" : ""}><span>全部内容</span><small>${files.length} 项，${bytes(archiveBytes)}</small></label></div></div><div class="field-group"><label for="conflict-policy" class="field-label">重名文件</label><select id="conflict-policy" class="field-select" data-xdzip-select><option value="ask">询问我如何处理</option><option value="skip">跳过已有文件</option><option value="rename">自动改名，保留两个版本</option><option value="overwrite">覆盖已有文件</option></select><p id="policy-note" class="field-note">保留已有文件，确认后再处理。</p></div><label class="check-label"><input id="open-after" type="checkbox" checked>完成后打开目标文件夹</label><label class="check-label"><input id="allow-high-ratio" type="checkbox">允许高压缩比文件（仍受字节限额约束）</label></div>` +
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
            `<span class="destination-mark">${icon(denied ? "status-warning" : "status-success")}</span><span class="destination-text">${
                denied
                    ? "此位置不可写，请更改目标文件夹。"
                    : "目标位置可用（示例检查）"
            }</span>`;
        $("#start-extract").disabled = denied;
    };
    $("#target-base").oninput = check;
    $("#same-folder").onchange = () => {
        const folder = " / " + state.archiveTitle.replace(/\.(tar\.gz|[^.]+)$/i, "");
        const input = $("#target-base");
        if (!$("#same-folder").checked && input.value.endsWith(folder))
            input.value = input.value.slice(0, -folder.length);
        else if ($("#same-folder").checked && !input.value.endsWith(folder))
            input.value += folder;
        check();
    };
    const targetField = $("#target-base");
    targetField.addEventListener("keydown", event => {
        if (["Escape", "Enter"].includes(event.key)) { event.preventDefault(); event.stopPropagation(); targetField.blur(); }
    });
    targetField.closest("dialog").addEventListener("pointerdown", event => {
        if (event.target !== targetField) targetField.blur();
    });
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
    const taskItems = kind === "create" ? selectedFiles(new Set(items.map(e => e.id))) : items;
    state.task = {
        kind,
        items: [...taskItems],
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
            `<div class="modal-content"><div class="conflict-name">${icon(e.type.includes("PDF") ? "conflict-pdf" : fileIcon(e))}${esc(e.name)}</div><div class="comparison"><div><span class="comparison-artwork">${icon(fileIcon(e))}</span><label>已有文件</label><strong>${bytes(Math.round(e.size * 0.875))}</strong><small>2026/10/01，09:42</small></div><div><span class="comparison-artwork">${icon(e.type.includes("PDF") ? "conflict-pdf" : fileIcon(e))}</span><label>压缩包中的文件</label><strong>${bytes(e.size)}</strong><small>2026/10/08，14:32</small></div></div><label class="check-label"><input id="apply-conflict-all" type="checkbox">对本次任务后续同类冲突应用此选择</label><div class="notice warning"><span class="notice-artwork">${icon("conflict-warning")}</span><span>覆盖会替换已有文件内容。选择“自动改名”可以保留两个版本。</span></div></div>` +
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
// All task kinds share one card hierarchy. Callers supply semantics and actions;
// sizing, colors, spacing and typography stay in the common progress selectors.
function taskProgressBody(p) {
    const percent = Math.max(0, Math.min(100, Number(p.progress) || 0));
    return `<div class="modal-content">
        <section class="path-card"><h2>${esc(p.scopeHeading)}</h2><div>${icon("folder")}<span>${esc(p.scopeValue)}</span></div></section>
        <section class="progress-card">
            <div class="progress-line"><div class="progress-track" role="progressbar" aria-label="${esc(p.progressName || "任务进度")}" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${percent}"><i id="progress-fill" style="width:${percent}%"></i></div><strong id="progress-value">${percent}%</strong></div>
            <div class="progress-current"><span id="progress-bytes">${esc(p.processed || "")}</span><span id="progress-count">${esc(p.count || "")}</span></div>
            <div class="task-metrics"><div><span>${esc(p.speedLabel || "速度")}</span><strong>${esc(p.speed || "6.2 MB/s")}</strong></div><div><span>预计剩余</span><strong>${esc(p.remaining || "约 5 秒")}</strong></div><div><span>当前阶段</span><strong id="progress-phase">${esc(p.phase || "正在准备")}</strong></div></div>
        </section>
        <section class="path-card"><h2>当前文件</h2><div><span id="progress-file-icon">${icon(p.entry ? fileIcon(p.entry) : "image-file")}</span><span id="progress-file">${esc(p.currentFile || "")}</span></div></section>
        <p class="info-line">${icon("info")}实际耗时取决于文件与磁盘状态。</p>
    </div>`;
}
function taskProgressFooter(p = {}) {
    return `<footer class="modal-footer"><span class="info-line">${icon("info")}${esc(p.note || "关闭此窗口后，任务仍会继续。")}</span><button id="${esc(p.backgroundId || "background-task")}" class="button">${esc(p.backgroundText || "返回浏览")}</button>${p.finished ? "" : `<button id="${esc(p.cancelId || "cancel-task")}" class="button" ${p.stopping ? "disabled" : ""}>${p.stopping ? "正在停止…" : esc(p.cancelText || "取消任务")}</button>`}</footer>`;
}
function showProgress() {
    const t = state.task;
    dialog(
        header(
            t.kind === "create"
                ? "正在创建压缩包"
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
            taskProgressBody({scopeHeading: t.kind === "create" ? "保存到" : "解压到", scopeValue: t.target, progress: t.progress}) +
            taskProgressFooter({stopping: t.status === "stopping"}),
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
function resultFileRow(entry, status, reason = "") {
    const unfinished = status === "unfinished";
    const label = reason || (unfinished ? "未完成" : status === "skipped" ? "已跳过" : "已完成");
    return `<div class="result-file-row ${unfinished ? "unfinished" : ""}">${icon(fileIcon(entry))}<div class="result-file-label"><strong>${esc(entry.name)}</strong><span>${bytes(entry.size)}</span></div><span class="result-file-status">${icon(unfinished ? "status-warning" : status === "skipped" ? "status-skipped" : "status-partial-success")}${esc(label)}</span></div>`;
}
function showResult() {
    const t = state.task, ok = t.result === "success", cancel = t.result === "cancelled", created = t.kind === "create";
    const title = ok ? created ? "压缩包已创建" : "解压完成" : cancel ? "任务已取消" : t.result === "source" ? "源文件已发生变化" : "部分文件未完成";
    const successCount = created ? (ok ? 1 : 0) : t.completed.length;
    const failedCount = created ? (ok ? 0 : 1) : t.failed.length;
    const reason = cancel ? "任务已安全停止。可以重新规划未完成项。" : t.result === "source" ? "压缩包在读取后被修改。请重新打开并重新选择内容。" : "目标磁盘空间不足，请释放空间或更改位置后重试。";
    const saved = t.completed.slice(0, 3).map(e=>e.name).join("、") + (t.completed.length > 3 ? ` 等 ${t.completed.length} 个文件` : "");
    dialog(
        header(title, ok ? created ? "压缩包已完成校验并保存。" : "所选内容已处理完成。" : created ? "临时压缩包未发布，原有文件保持不变。" : "已保留成功解压的文件。", ok ? "status-success" : cancel ? "cancelled-large" : "document-warning") +
        `<div class="modal-content result-content"><div class="result-counts"><div><span>${icon("status-partial-success")}${created ? "已保存" : "成功"}</span><b>${successCount}</b></div><div><span>${icon("status-skipped")}跳过</span><b>${created ? 0 : t.skipped.length}</b></div><div><span>${icon(cancel ? "status-stopped" : "status-warning")}未完成</span><b>${failedCount}</b></div></div>
        ${!ok ? `<section class="result-card"><h2>未完成的文件</h2><div class="result-inner">${t.failed.map(e=>resultFileRow(e,"unfinished",cancel ? "未完成" : t.result === "source" ? "源文件已变化" : "空间不足")).join("")}</div><div class="result-warning">${icon("status-warning")}<span>${reason}</span></div></section>` : ""}
        <section class="result-card"><h2>目标位置</h2><div class="result-location result-inner">${icon("folder")}<div><strong>${esc(t.target)}</strong><p>${created ? "压缩包已校验并保存。" : "已保存：" + (saved ? esc(saved) + "（" + bytes(t.completed.reduce((n,e)=>n+e.size,0)) + "）" : "尚未保存文件")}</p></div></div></section>
        <details class="result-details result-card"><summary>${icon("chevron-right")}<strong>查看处理明细</strong><span>${t.completed.length + t.failed.length + t.skipped.length} 个文件</span></summary><div class="result-list result-inner">${t.completed.map(e=>resultFileRow(e,"completed")).join("")}${t.failed.map(e=>resultFileRow(e,"unfinished")).join("")}${t.skipped.map(id=>resultFileRow(byId.get(id),"skipped")).join("")}</div></details></div>` +
        footer(cancelButton("关闭") + '<button id="reveal-result" class="button">打开目标文件夹</button>' + (t.result === "source" ? '<button id="reopen-result" class="button primary">重新打开</button>' : !ok && failedCount ? '<button id="retry-result" class="button primary">重试未完成项</button>' : ""), ok ? "已完成校验并保存。" : "重试将仅处理未完成的文件，并重新确认目标位置。", "info"),
        "result", "R01"
    );
    bindClose();
    $("#reveal-result").onclick = () => toast("原型演示：正式产品将在系统文件管理器中打开目标位置。");
    if ($("#reopen-result")) $("#reopen-result").onclick = () => { closeModal(); openPicker(); };
    if ($("#retry-result")) $("#retry-result").onclick = () => {
        state.scenario = "normal"; $("#scenario").value = "normal";
        if (created) showCreate(); else showExtract("retry", t.failed);
    };
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
    state.createItems = [byId.get("lake"),byId.get("sunrise"),byId.get("guide")].filter(Boolean);
    dialog(
        header("创建压缩包", "将文件整理为一个压缩包。", "archive-add") +
        `<div class="modal-content"><div class="field-group"><div class="field-label">添加文件</div><div class="create-input-panel"><div class="create-actions"><button id="add-input" class="button primary">${icon("file-text")}添加文件</button><button id="add-directory" class="button">${icon("folder")}添加文件夹</button><button id="clear-input" class="button">清空</button></div><div id="create-list" class="create-list"></div><p class="field-note" id="create-size"></p></div></div><div class="field-group"><div class="field-label">保存位置</div><div class="create-input-panel"><div class="create-form-row"><label for="create-name">文件名</label><input id="create-name" class="text-input" value="旅行资料.zip"></div><div class="create-form-row"><label for="create-destination">保存到</label><div class="create-target-row"><input id="create-destination" class="text-input" value="下载"><button id="create-change-target" class="button">更改</button></div></div><p id="create-valid" class="field-note success"></p></div></div><div class="field-group"><div class="field-label">压缩选项</div><div class="create-input-panel"><div class="create-form-row"><label>压缩格式</label>${XdzipControls.segmentedMarkup({id:"create-format",label:"压缩格式",selected:"zip",options:[{id:"create-format-zip",value:"zip",text:"ZIP"},{id:"create-format-tar",value:"tar",text:"TAR"},{id:"create-format-gzip",value:"gzip",text:"TAR.GZ"}]})}</div><div class="create-form-row" id="level-group"><label for="create-level">压缩级别</label><select id="create-level" class="field-select"><option value="0">0-不压缩</option><option value="1">1-快速压缩</option><option value="2" selected>2-正常压缩</option><option value="3">3-最大压缩</option></select></div><div class="create-form-row"><label for="create-volume">分卷</label><div class="create-volume-row"><select id="create-volume" class="field-select" data-editable-value="custom">${["不分卷", "3MB", "4MB", "5MB", "10MB", "20MB", "25MB (Gmail)", "100MB", "300MB", "650MB (CD)", "700MB (CD)", "4GB (FAT32)", "4.7GB (DVE)", "自定义"].map((label,i)=>`<option value="${i===13?"custom":i}">${label}</option>`).join("")}</select><select id="create-volume-unit" class="field-select"><option value="1">字节</option><option value="1000">KB</option><option value="1000000" selected>MB</option><option value="1000000000">GB</option></select></div></div><p id="create-volume-note" class="field-note">每卷使用所选字节上限，输出 .zip.001 等文件。</p><label class="check-label"><input id="create-use-password" type="checkbox">输入密码</label><input id="create-password" type="password" class="text-input" disabled aria-label="压缩包密码" placeholder="1–128 个 UTF-8 字节，不含控制字符" autocomplete="new-password"><p id="create-password-note" class="field-note">AES-256 加密文件内容，文件名仍可见。</p></div><details><summary class="details-title">重名演示</summary><div class="details-body"><label class="check-label"><input id="create-existing" type="checkbox">目标位置已有同名压缩包（演示覆盖确认）</label></div></details></div></div>` +
        footer(`<button id="cancel-modal" class="button" data-close>取消</button><button id="start-create" class="button primary">创建压缩包</button>`),
        "create", "C01", "#add-input",
    );
    bindClose();
    const selectedFormat = () => $("#create-format [aria-checked='true']")?.dataset.value || "zip";
    const extension = () => ({zip:".zip",tar:".tar",gzip:".tar.gz"})[selectedFormat()];
    const validate = () => {
        const optionsValid = advancedValid();
        const valid =
            state.createItems.length &&
            $("#create-name").value.trim().toLowerCase().endsWith(extension()) && $("#create-name").value.trim().length > extension().length &&
            $("#create-destination").value.trim() && optionsValid;
        $("#start-create").disabled = !valid;
        $("#create-valid").innerHTML =
            `${icon(valid ? "success" : "status-warning")}<span>${
                valid
                    ? "示例检查通过，" +
                      state.createItems.length +
                      " 个选择项，目标可写（示例）"
                    : "请添加输入文件，并填写名称与保存位置。"
            }</span>`;
        $("#create-valid").className =
            "field-note " + (valid ? "success" : "error");
    };
    const selectionSize = value => value >= 1000000 ? (value / 1000000).toFixed(1) + " MB" : value >= 1000 ? Math.round(value / 1000) + " KB" : value + " B";
    const draw = () => {
        $("#create-list").innerHTML = state.createItems.length
            ? state.createItems
                  .map(
                      (e, i) =>
                          `<div class="create-row">${icon(fileIcon(e))}<span>${esc(e.name)}</span><span>${e.folder ? "开始前扫描" : bytes(e.size)}</span><button class="icon-button" data-remove="${i}" aria-label="移除 ${esc(e.name)}">${icon("remove")}</button></div>`,
                  )
                  .join("")
            : '<div class="create-row">尚未添加内容</div>';
        const selectedFiles = state.createItems.filter(e => !e.folder);
        const selectedDirectories = state.createItems.length - selectedFiles.length;
        const summary = [];
        if (selectedFiles.length) summary.push(selectedFiles.length + " 个文件");
        if (selectedDirectories) summary.push(selectedDirectories + " 个文件夹");
        if (selectedFiles.length) summary.push(selectionSize(selectedFiles.reduce((n, e) => n + e.size, 0)));
        if (selectedDirectories) summary.push("文件夹大小将在开始前扫描");
        $("#create-size").textContent = summary.length ? summary.join("，") : "尚未添加文件";
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
            (e) => !e.folder && !state.createItems.some((f) => f.id === e.id),
        );
        if (next) {
            state.createItems.push(next);
            draw();
        } else toast("所有示例文件都已添加。");
    };
    $("#add-directory").onclick = () => { const next=rootFolders.find(e=>e.folder && !state.createItems.some(f=>f.id===e.id)); if (next) { state.createItems.push(next); draw(); } };
    $("#create-change-target").onclick = () => { $("#create-destination").value = $("#create-destination").value === "下载" ? "文稿" : "下载"; validate(); };
    $("#clear-input").onclick = () => {
        state.createItems = [];
        draw();
    };
    $("#create-name").oninput = validate;
    $("#create-destination").oninput = validate;
    const advancedValid = () => {
        const format = selectedFormat();
        $("#create-level").disabled = format === "tar";
        $("#create-volume").disabled = format !== "zip";
        $("#create-use-password").disabled = format !== "zip";
        const custom = $("#create-volume").value === "custom";
        $("#create-volume-unit").closest(".ui-select").hidden = !custom;
        const number = $("#create-volume")._xdzipEditor?.value || "";
        const volumeBytes = Number(number) * Number($("#create-volume-unit").value);
        const validVolume = !custom || (/^[0-9]+$/.test(number) && Number.isSafeInteger(volumeBytes) && volumeBytes > 0 && volumeBytes <= 25769803776);
        $("#create-volume-note").textContent = format === "tar" ? "TAR 仅打包，不压缩，不支持分卷和密码。" : format === "gzip" ? "TAR.GZ 使用 GZIP 压缩，不支持分卷和密码。" : validVolume ? "每卷使用所选字节上限，输出 .zip.001 等文件。现有分卷只能整组改名。" : "请输入正整数，每卷上限为 24 GiB，最多 1024 卷。";
        $("#create-volume-note").classList.toggle("error", !validVolume);
        const enabled = $("#create-use-password").checked;
        $("#create-password").disabled = !enabled;
        const password = $("#create-password").value;
        const passwordLength = new TextEncoder().encode(password).length;
        const validPassword = !enabled || (passwordLength > 0 && passwordLength <= 128 && !/[\x00-\x1f\x7f-\x9f]/.test(password) && password.isWellFormed());
        $("#create-password-note").textContent = format !== "zip" ? "此格式不支持密码加密。" : validPassword ? "AES-256 加密文件内容，文件名仍可见。" : "请输入有效密码；不得为空或包含控制字符。";
        $("#create-password-note").classList.toggle("error", !validPassword);
        return validVolume && validPassword;
    };
    $("#create-format").addEventListener("change", () => {
        $("#create-name").value = $("#create-name").value.replace(/\.(tar\.gz|zip|tar)$/i, "") + extension();
        $("#create-volume").value = "0";
        $("#create-use-password").checked = false;
        $("#create-password").value = "";
        validate();
        document.querySelectorAll("select[data-xdzip-select]").forEach(item => item._xdzipRefresh?.());
    });
    $("#create-volume").onchange = validate;
    $("#create-volume").oninput = validate;
    $("#create-volume-unit").onchange = validate;
    $("#create-use-password").onchange = () => { if (!$("#create-use-password").checked) $("#create-password").value = ""; validate(); };
    $("#create-password").oninput = validate;
    $("#start-create").onclick = () => {
        state.createTarget =
            $("#create-destination").value.trim() +
            " / " +
            $("#create-name").value.trim();
        state.createFormat = selectedFormat();
        state.createLevel = Number($("#create-level").value);
        state.createMethod = state.createFormat === "tar" || (state.createFormat === "zip" && state.createLevel === 0) ? "store" : "deflate";
        // This HTML is an interaction demo; never retain a secret in task state.
        state.createEncrypted = $("#create-use-password").checked;
        state.createVolume = $("#create-volume").value;
        $("#create-password").value = "";
        if ($("#create-existing").checked) confirmCreateOverwrite();
        else startTask(state.createItems, "create");
    };
    draw();
}
function confirmCreateOverwrite() {
    dialog(
        header(
            "替换已有压缩包？",
            esc(state.createTarget),
            "archive-warning",
            false,
        ) +
            `<div class="modal-content"><section class="existing-archive"><span class="existing-archive-artwork">${icon("existing-archive")}</span><div><strong>已有压缩包</strong><p>${esc(state.createTarget)}</p></div></section><div class="notice warning">已有压缩包会在新压缩包成功写入、校验并提交后被替换。取消或失败不会先删除已有文件。</div></div>` +
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
let savedSettingsDemo = null;
let settingsPreview = null;
const settingText = id => window.XDZIP_UI_MESSAGES["zh-CN"].messages[id];
function restoreSettingsPreview() {
    if (!settingsPreview) return;
    const original = settingsPreview.original;
    settingsPreview = null;
    setTheme(original.theme);
    document.documentElement.style.setProperty("--list-density-factor", original.density === "compact" ? .8 : 1);
    document.documentElement.dataset.listDensity = original.density;
    document.documentElement.style.setProperty("--settings-text-scale", original.textScale);
}
function showPreflight() {
    const t = settingText;
    dialog(`<div class="preflight-body"><h1>${t("ui.9cb79c4bee66")}</h1><p>${t("ui.db661e297081")}</p><div class="preflight-stats"><div><span>${t("ui.0790c26b1e36")}</span><strong>9</strong></div><div><span>${t("ui.377532e93480")}</span><strong>77619200</strong></div><div><span>${t("ui.9d959257ce7e")}</span><strong>0</strong></div></div><p>${t("ui.f644f6ec6fad")}</p><p class="preflight-note">${t("ui.0d92f3d4cae3")}</p></div>` + footer(`<button class="button primary" id="preflight-close">${t("ui.26867436321f")}</button>`, ""), "preflight", "P01", "#preflight-close");
    $("#preflight-close").onclick = closeModal;
}
function showSettings() {
    restoreSettingsPreview();
    const original = {theme:state.theme, textScale:Number(document.documentElement.style.getPropertyValue("--settings-text-scale")) || 1, language:"zh-CN", density:document.documentElement.dataset.listDensity || "standard", recent:state.recent, clearRecent:false};
    const draft = {...(savedSettingsDemo || original)};
    const t = settingText;
    const themePreviews = {system:icon("theme-system"),light:icon("theme-light"),dark:icon("theme-dark")};
    dialog(
        header(t("settings.title"), t("settings.subtitle"), "settings-tile") +
        `<div class="modal-content settings-content">
        <fieldset id="setting-theme" class="settings-theme"><legend>${t("settings.theme")}</legend>
        <div class="theme-cards">${["system","light","dark"].map(mode=>`<label class="theme-card">${themePreviews[mode]}<span><input type="radio" name="setting-theme" value="${mode}">${t("settings."+mode)}</span></label>`).join("")}</div></fieldset>
        <div class="settings-row"><label for="setting-density">${t("settings.density")}</label><select id="setting-density" class="field-select"><option value="standard">${t("settings.standard")}</option><option value="compact">${t("settings.compact")}</option></select></div>
        <div class="settings-row"><label for="setting-text-scale">${t("settings.textSize")}</label><select id="setting-text-scale" class="field-select">${[1,1.25,1.5].map(value=>`<option value="${value}">${value*100}%</option>`).join("")}</select></div>
        <div class="settings-section"><div class="settings-row"><label for="setting-language">${t("settings.language")}</label><select id="setting-language" class="field-select"><option value="zh-CN">${t("language.zhCN")}</option><option value="en-US">${t("language.enUS")}</option></select></div><p>${t("ui.3a593f5653b1")}</p></div><label class="check-label"><input id="recent-enabled" type="checkbox">${t("settings.recent")}</label><div><button id="clear-recent" class="button">${t("settings.clearRecent")}</button></div><div class="notice">${t("settings.privacy")}</div></div>` +
        footer(`<button id="settings-defaults" class="button">${t("settings.defaults")}</button><button class="button" data-close>${t("common.cancel")}</button><button id="settings-save" class="button">${t("settings.done")}</button>`,t("ui.5fb5c0083f18"), "shield"),
        "settings", "S01", "#settings-save",
    );
    settingsPreview = {original,draft};
    function refresh() {
        $(`input[name="setting-theme"][value="${draft.theme}"]`).checked = true;
        $("#setting-text-scale").value = String(draft.textScale);
        $("#setting-language").value = draft.language;
        $("#setting-density").value = draft.density;
        $("#recent-enabled").checked = draft.recent;
        $("#clear-recent").textContent = t(draft.clearRecent ? "settings.clearPending" : "settings.clearRecent");
        $("#clear-recent").disabled = draft.clearRecent;
        $("#modal").querySelectorAll("select").forEach(select=>select._xdzipRefresh?.());
        setTheme(draft.theme);
        document.documentElement.style.setProperty("--list-density-factor",draft.density === "compact" ? .8 : 1);
        document.documentElement.dataset.listDensity = draft.density;
        document.documentElement.style.setProperty("--settings-text-scale",draft.textScale);
    }
    refresh();
    bindClose();
    $("#setting-theme").onchange = e => { draft.theme=e.target.value;refresh(); };
    $("#setting-text-scale").onchange = e => { draft.textScale=Number(e.target.value);refresh(); };
    $("#setting-language").onchange = e => { draft.language=e.target.value; };
    $("#setting-density").onchange = e => { draft.density=e.target.value; refresh(); };
    $("#recent-enabled").onchange = e => { draft.recent=e.target.checked; };
    $("#clear-recent").onclick = () => { draft.clearRecent=true; refresh(); };
    $("#settings-defaults").onclick = () => { Object.assign(draft,{theme:"system",textScale:1,language:"zh-CN",density:"standard",recent:true,clearRecent:false});refresh(); };
    $("#settings-save").onclick = () => { savedSettingsDemo={...draft,clearRecent:false};state.recent=draft.recent;settingsPreview=null;closeModal(); };
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
    setWelcomeMode(false);
    if ($("#modal").open) closeModal({preserveReview:true});
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
    if ($("#modal").open) closeModal({preserveReview:true});
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
        if (e.key.toLowerCase() === "f" && !document.body.classList.contains("welcome-mode")) {
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
        cancelWelcomeOpening(false);
        // Direct standalone URLs can run before the main-window adapter's
        // deferred script; review messages arrive after that script is ready.
        if (["archive-test", "preflight", "welcome", "welcome-narrow"].includes(view) && !window.XdzipMain) {
            window.addEventListener("DOMContentLoaded", () => window.XdzipDesign.open(view), {once: true});
            return;
        }
        applyingReviewPreset = true;
        try {
        if (!["welcome", "welcome-narrow", "main"].includes(view) && document.body.classList.contains("welcome-mode")) {
            setWelcomeMode(false);
            window.XdzipMain?.openSample("travel");
        }
        if (state.task?.reviewPaused) state.task = null;
        if (state.testJob?.reviewPaused) window.XdzipMain.resetJobs();
        if ($("#modal").open) closeModal({preserveReview:true});
        // Artboard changes restore the ready demo fixture after an error scene.
        // Active user-started demo tasks retain their own background state.
        if (
            ["main", "extract", "create", "settings", "progress"].includes(view)
        ) {
            state.archiveState = "ready";
            state.scenario = "normal";
        }
        if (["welcome", "welcome-narrow"].includes(view)) {
            document.body.classList.add("welcome-artboard");
            window.XdzipMain.enterWelcome(view);
            return;
        }
        if (view === "main") {
            setWelcomeMode(false);
            window.XdzipMain?.openSample("travel");
            state.path = "";
            state.query = "";
            $("#search").value = "";
            state.selected = new Set(["lake"]);
            render();
            return;
        }
        if (view === "archive-test") {
            window.XdzipMain.resetJobs();
            window.XdzipDesign.open("main");
            window.XdzipMain.dispatch("test");
            state.testJob.progress = 60;
            state.testJob.reviewPaused = true;
            window.XdzipMain.dispatch("tasks");
            return;
        }
        if (view === "extract") {
            state.selected = new Set(["lake", "sunrise", "guide"]);
            render();
            showExtract("selected");
        } else if (view === "create") showCreate();
        else if (view === "settings") showSettings();
        else if (view === "preflight") showPreflight();
        else if (view === "result") {
            state.task = null; showExtract("selected");
            startTask([byId.get("lake"), byId.get("sunrise"), byId.get("guide")], "extract");
            clearInterval(timer); state.task.progress = 80;
            state.modal = "progress"; finishTask("partial");
        }
        else if (view === "progress") {
            showExtract("all");
            startTask(files, "extract");
            clearInterval(timer);
            state.task.progress = 60;
            state.task.reviewPaused = true;
            updateProgress();
        } else scenario(view);
        } finally { applyingReviewPreset = false; }
    },
    theme(value) { restoreSettingsPreview(); setTheme(value); },
    platform(value) {
        document.body.dataset.platform = value;
        if (document.body.classList.contains("welcome-mode")) showWelcome();
    },
};
window.XdzipControls.configure({ icon });
function setCanvasLogicalScale(view) {
    const spec = window.XDZIP_DISPLAY_SPEC;
    for (const key of ["captionButtonWidth", "captionIconSize"])
        document.documentElement.style.setProperty(`--${key}`, spec.windowsCaption[key]);
    const main = ["main", "welcome", "welcome-narrow", "empty", "damaged"].includes(view);
    document.documentElement.style.setProperty("--ui-source-factor", 1 / (spec.views[view]?.sourceToLogical ?? (main ? spec.sourceToLogical : spec.operationSourceToLogical)));
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
            "welcome",
            "welcome-narrow",
            "extract",
            "create",
            "progress",
            "archive-test",
            "result",
            "conflict",
            "settings",
            "preflight",
            "empty",
            "damaged",
        ].includes(config.view) ||
        !["macos", "windows", "linux"].includes(config.platform) ||
        !["light", "dark"].includes(config.theme)
    )
        return;
    document.body.dataset.standalone = ["main", "welcome", "welcome-narrow", "empty", "damaged"].includes(
        config.view,
    )
        ? "false"
        : "true";
    setCanvasLogicalScale(config.view);
    XdzipDesign.platform(config.platform);
    XdzipDesign.theme(config.theme);
    XdzipDesign.open(config.view);
    if (config.welcomeOpening && ["welcome", "welcome-narrow"].includes(config.view))
        beginWelcomeOpening("北境旅行.zip");
    if (config.welcomeDrag && ["welcome", "welcome-narrow"].includes(config.view)) {
        $("#welcome-drop").classList.add("drag-over");
        $("#welcome-drop span").textContent = "松开即可打开压缩包";
    }
});
if (window.parent !== window)
    window.parent.postMessage({ type: "xdzip-review-ready" }, "*");
