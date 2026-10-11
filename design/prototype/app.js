"use strict";
// Design-only state machine: all file operations use in-memory fixtures.
// This prototype never reads archives, writes files, or uploads user content.
const paths = {
    archive: "M5 3h10l4 4v14H5z M14 3v5h5 M10 4v2m0 2v2m0 2v2m0 2v3h3v-3z",
    folder: "M3 7V5h6l2 2h10v13H3z",
    folderOpen: "M3 8V5h6l2 2h10v3 M3 10h19l-3 10H2z",
    file: "M5 3h9l5 5v13H5z M14 3v6h5",
    image: "M4 3h16v18H4z M4 16l5-5 5 6 3-3 3 4 M15 7h.01",
    pdf: "M5 3h9l5 5v13H5z M14 3v6h5 M8 13h8 M8 16h6",
    search: "m20 20-5-5 M17 10a7 7 0 1 1-14 0 7 7 0 0 1 14 0",
    plus: "M12 5v14 M5 12h14",
    close: "m6 6 12 12 M6 18 18 6",
    chevronDown: "m6 9 6 6 6-6",
    chevronUp: "m6 15 6-6 6 6",
    chevronRight: "m9 6 6 6-6 6",
    extract: "M12 3v12 m-5-5 5 5 5-5 M4 15v6h16v-6",
    shield: "m12 3 8 3v6c0 5-8 9-8 9s-8-4-8-9V6z m-4 9 3 3 5-6",
    panel: "M3 4h18v16H3z M15 4v16",
    activity: "M3 12h4l3-7 4 14 3-7h4",
    settings:
        "M9 3h6l1 3 3 1 2 5-2 5-3 1-1 3H9l-1-3-3-1-2-5 2-5 3-1z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0",
    sort: "M7 4v16 m-3-3 3 3 3-3 M14 5h7 M14 10h5 M14 15h3",
    sun: "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8 M12 2v2 M12 20v2 M2 12h2 M20 12h2 M5 5l1 1 M18 18l1 1 M5 19l1-1 M18 6l1-1",
    moon: "M20 14A8 8 0 0 1 10 4a8 8 0 1 0 10 10",
    check: "m5 12 4 4 10-10",
    warning: "m12 3 10 18H2z M12 9v5 M12 17h.01",
    layers: "m12 3 10 5-10 5L2 8z M2 12l10 5 10-5 M2 16l10 5 10-5",
    arrow: "M4 12h15 m-5-5 5 5-5 5",
    clock: "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18 M12 7v6h4",
};
// Filled document marks distinguish file kinds without relying on color alone.
const marks = {
    folder: '<path fill="#8bb6ee" stroke="none" d="M2 5a2 2 0 0 1 2-2h5l3 3h8a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2z"/><path fill="#5592df" stroke="none" d="M2 9h20v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2z"/>',
    archive:
        '<rect x="3" y="2" width="18" height="20" rx="4" fill="#4169df" stroke="none"/><path stroke="white" d="M10 4h3m-2 3h3m-4 3h3m-2 3h3m-4 3h4v4h-4z"/>',
    pdf: '<path fill="#fbe8e8" stroke="#d97272" d="M5 2h9l5 5v15H5z M14 2v6h5"/><path stroke="#cc5353" d="M8 12h8m-8 3h8m-8 3h5"/>',
    file: '<path fill="var(--surface)" stroke="#8f9aaa" d="M5 2h9l5 5v15H5z M14 2v6h5 M8 12h8m-8 3h8m-8 3h5"/>',
    code: '<path stroke="#8193b1" d="m8 7-5 5 5 5m8-10 5 5-5 5m-3-13-2 20"/>',
    markdown:
        '<rect x="2" y="5" width="20" height="14" rx="2" stroke="#8193b1"/><path stroke="#8193b1" d="M5 16V8l4 4 4-4v8m5-8v8m-2-2 2 2 2-2"/>',
};
const icon = (name, cls = "") =>
    `<svg class="${cls}" viewBox="0 0 24 24" aria-hidden="true">${marks[name] || `<path d="${paths[name] || paths.file}"/>`}</svg>`;
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
const folders = [
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
const children = folders.flatMap((f, fi) =>
    Array.from({ length: [12, 8, 4][fi] }, (_, i) => ({
        id: `${f.id}-${i}`,
        parent: f.id,
        name: `${["旅途瞬间", "旅行笔记", "离线地图"][fi]}-${String(i + 1).padStart(2, "0")}.${["jpg", "pdf", "gpx"][fi]}`,
        size: (fi + 1) * MB + i * 100000,
        compressed: Math.round(((fi + 1) * MB + i * 100000) * 0.72),
        type: ["JPEG 图像", "PDF 文档", "GPX 文件"][fi],
    })),
);
const entries = [...folders, ...roots, ...children],
    files = entries.filter((e) => !e.folder),
    byId = new Map(entries.map((e) => [e.id, e]));
const archiveBytes = files.reduce((n, f) => n + f.size, 0),
    compressedBytes = files.reduce((n, f) => n + f.compressed, 0);
const state = {
    path: "",
    query: "",
    selected: new Set(),
    sort: 1,
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
        "搜索覆盖当前归档，忽略当前目录范围；结果显示来源路径。无选择时全部解压始终指整个归档。",
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
                .filter((x) => x.parent === id)
                .forEach((x) => found.set(x.id, x));
        else found.set(id, e);
    }
    return [...found.values()];
}
function visibleEntries() {
    if (state.archiveState !== "ready") return [];
    return entries
        .filter((e) =>
            state.query
                ? e.name.toLowerCase().includes(state.query.toLowerCase())
                : e.parent === state.path,
        )
        .sort(
            (a, b) =>
                Number(b.folder) - Number(a.folder) ||
                state.sort *
                    a.name.localeCompare(b.name, "zh-CN", { numeric: true }),
        );
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
    state.query = "";
    state.selected.clear();
    state.anchor = null;
    $("#search").value = "";
    render();
}
function renderTree() {
    if (state.archiveState !== "ready") {
        $("#tree").innerHTML =
            `<button class="tree-item active" disabled>${icon("layers")}<span>全部文件</span><small>${state.archiveState === "empty" ? "0" : "—"}</small></button>`;
        return;
    }
    const counts = (id) =>
        files.filter((f) => (id ? f.parent === id : true)).length;
    $("#tree").innerHTML =
        `<button class="tree-item ${!state.path ? "active" : ""}" data-path="">${icon("layers")}<span>全部文件</span><small>${files.length}</small></button>${folders.map((f) => `<button class="tree-item child ${state.path === f.id ? "active" : ""}" data-path="${f.id}">${icon("chevronRight")}${icon("folder")}<span>${f.name.slice(3)}</span><small>${counts(f.id)}</small></button>`).join("")}`;
    $("#tree")
        .querySelectorAll("button")
        .forEach((b) => (b.onclick = () => choosePath(b.dataset.path)));
}
function render() {
    renderTree();
    const list = visibleEntries(),
        chosen = selectedFiles(),
        sum = chosen.reduce((n, f) => n + f.size, 0);
    $("#archive-info").textContent = `ZIP · ${bytes(compressedBytes)}`;
    $("#original-size").textContent = bytes(archiveBytes);
    $("#compressed-size").textContent = bytes(compressedBytes);
    const saving = Math.round((1 - compressedBytes / archiveBytes) * 100);
    $("#saving").textContent = saving + "%";
    $("#saving-bar").style.width = saving + "%";
    if (state.archiveState !== "ready") {
        $("#archive-info").textContent =
            state.archiveState === "empty"
                ? "ZIP · 空归档"
                : "ZIP · 元数据未就绪";
        $("#original-size").textContent =
            state.archiveState === "empty" ? "0 B" : "未知";
        $("#compressed-size").textContent = "未知";
        $("#saving").textContent = "—";
        $("#saving-bar").style.width = "0%";
    }
    $("#breadcrumbs").innerHTML =
        `<button data-home>北境旅行.zip</button>${icon("chevronRight")}<span>${state.query ? "搜索结果" : state.path ? byId.get(state.path).name : "全部文件"}</span>`;
    $("#breadcrumbs [data-home]").onclick = () => choosePath("");
    $("#entry-count").textContent = `${list.length} 项`;
    $("#search-notice").hidden = !state.query;
    $("#search-notice").textContent =
        `当前归档中找到 ${list.length} 项 · 搜索结果不等同于全部归档`;
    $("#file-rows").innerHTML = list
        .map(
            (e) =>
                `<tr tabindex="0" data-id="${e.id}" class="${state.selected.has(e.id) ? "selected" : ""}" aria-selected="${state.selected.has(e.id)}"><td><input type="checkbox" tabindex="-1" aria-label="选择 ${esc(e.name)}" ${state.selected.has(e.id) ? "checked" : ""}></td><td><div class="entry-name">${icon(fileIcon(e), fileIcon(e))}<span title="${esc(e.name)}">${esc(e.name)}</span></div></td><td>${e.folder ? "—" : bytes(e.size)}</td><td>${e.folder ? "—" : bytes(e.compressed)}</td><td class="type-col">${e.folder ? "文件夹" : esc(e.type)}</td><td class="date-col">2026/10/08</td></tr>`,
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
    $("#extract-label").textContent = chosen.length ? "解压所选" : "全部解压";
    $("#extract").disabled = state.archiveState !== "ready";
    $("#extract-menu-button").disabled = state.archiveState !== "ready";
    $("#selection-status").textContent = chosen.length
        ? `已选择 ${chosen.length} 个文件 · ${bytes(sum)}`
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
    renderEmpty(list);
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
            `<div class="inspector-head">归档信息</div><div class="file-emblem">${icon("archive")}</div><h2>北境旅行.zip</h2><p class="inspector-footnote">${state.archiveState === "empty" ? "目录已读取，归档中没有可解压的文件。" : "目录信息未就绪，暂时无法展示文件属性。"}</p>`;
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
        `<div class="inspector-head"><span>${chosen.length ? "文件属性" : "归档信息"}</span><button class="icon-button" aria-label="关闭属性面板" id="hide-inspector">${icon("close")}</button></div><div class="file-emblem">${icon(multi ? "layers" : single ? fileIcon(single) : "archive")}<span>${multi ? "MULTIPLE FILES" : single ? single.name.split(".").pop().toUpperCase() : "ZIP ARCHIVE"}</span></div><h2>${multi ? `${chosen.length} 个文件` : single ? esc(single.name) : "北境旅行.zip"}</h2><span class="type-label">${multi ? "多个类型 · 已选择" : single ? single.type : "ZIP 压缩文件 · Deflate"}</span><dl class="property-list">${[
            ["原始大小", bytes(size)],
            ["压缩后", bytes(packed)],
            ["压缩方式", "Deflate"],
            ["修改日期", "2026/10/08"],
            [
                "位置",
                single && single.parent
                    ? byId.get(single.parent).name
                    : "归档根目录",
            ],
        ]
            .map(
                ([a, b]) =>
                    `<div class="property"><dt>${a}</dt><dd>${esc(b)}</dd></div>`,
            )
            .join(
                "",
            )}</dl><button id="inspector-extract" class="button" ${state.archiveState !== "ready" ? "disabled" : ""}>${icon("extract")}${chosen.length ? "解压所选文件" : "全部解压"}</button><p class="inspector-footnote">文件内容保留在压缩包中。只有开始解压后，才会写入目标位置。</p>`;
    $("#hide-inspector").onclick = () => ($("#inspector").hidden = true);
    $("#inspector-extract").onclick = () =>
        showExtract(chosen.length ? "selected" : "all");
}
function updateTaskCount() {
    $("#task-count").textContent = state.task
        ? `任务 ${["running", "stopping", "conflict"].includes(state.task.status) ? "1" : "· 已结束"}`
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
    return `<div class="os-titlebar">${mac ? controls : icon("archive")}<span class="os-title">${esc(title)}</span>${mac ? '<span class="os-balance"></span>' : controls}</div>`;
}
function dialog(html, kind, spec, focus) {
    const el = $("#modal");
    returnFocus = el.open ? returnFocus : document.activeElement;
    if (el.open) el.close();
    const template = document.createElement("template");
    template.innerHTML = html;
    const title = template.content.querySelector("h1")?.textContent || "xdzip";
    el.innerHTML = windowChrome(title, kind === "progress") + html;
    el.setAttribute("aria-label", title);
    el.style.left = el.style.top = el.style.margin = "";
    state.modal = kind;
    setSpec(spec);
    // Decision windows block only their owner; task details/settings remain modeless.
    const modeless = ["progress", "settings", "result"].includes(kind);
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
    if (returnFocus?.isConnected) returnFocus.focus();
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
    dialog(
        header(
            "打开压缩文件",
            "原生文件选择器的设计替身 · 仅提供示例归档",
            "folderOpen",
        ) +
            `<div class="modal-content"><p class="field-note">正式产品在这里调用系统文件选择器。原型不会读取设备上的文件。</p><div class="field-group"><button class="button" id="pick-sample">${icon("archive")}北境旅行.zip</button></div></div>` +
            footer(cancelButton(), "仅使用示例数据"),
        "picker",
        "B01",
        "#pick-sample",
    );
    bindClose();
    $("#pick-sample").onclick = () => {
        state.archiveState = "ready";
        state.path = "";
        state.query = "";
        state.selected.clear();
        state.scenario = "normal";
        $("#scenario").value = "normal";
        $("#search").value = "";
        $("#welcome").hidden = true;
        closeModal();
    };
}
function showWelcome() {
    state.selected.clear();
    $("#welcome").hidden = false;
    $("#welcome").innerHTML =
        `<div class="brand-tile">${icon("archive")}</div><h1>压缩文件，一目了然。</h1><p>先查看内容，再决定解压哪些文件。</p><div class="welcome-actions"><button class="button primary" id="welcome-open">${icon("folderOpen")}打开压缩文件</button><button class="button" id="welcome-create">${icon("plus")}新建压缩文件</button></div><small>ZIP · TAR · TAR.GZ / 所有处理均在本机完成</small>`;
    $("#welcome-open").onclick = openPicker;
    $("#welcome-create").onclick = showCreate;
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
    return base + (same && !base.endsWith("北境旅行") ? " / 北境旅行" : "");
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
        target: "下载 / 北境旅行",
        policy: "ask",
        same: true,
        open: true,
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
            `北境旅行.zip · ${scope === "all" ? "整个归档" : scope === "results" ? "当前搜索结果" : scope === "retry" ? "重试范围" : "所选文件"}`,
            "extract",
        ) +
            `<div class="modal-content"><div class="field-group"><label class="field-label" for="target-base">目标文件夹<span>仅在确认后写入</span></label><div class="input-row"><input id="target-base" class="text-input" value="${state.scenario === "permission" ? "系统 / 受保护目录" : "下载"}"><button class="button" id="change-target">更改</button></div><label class="check-label"><input id="same-folder" type="checkbox" checked>创建同名文件夹</label><p id="target-preview" class="field-note"></p><p id="target-valid" class="field-note success"></p></div><div class="field-group"><div class="field-label">解压范围</div><div class="scope-box">${
                scope !== "all"
                    ? `<label class="radio-row"><input type="radio" name="scope" value="${scope}" checked><span>${scope === "results" ? "当前搜索结果" : scope === "retry" ? "失败或未完成项" : "所选文件"}</span><small>${count} 项 · ${bytes(sum)}</small></label><div class="scope-files">${chosen
                          .slice(0, 3)
                          .map(
                              (e) =>
                                  `${esc(e.name)}<span style="float:right">${bytes(e.size)}</span>`,
                          )
                          .join(
                              "<br>",
                          )}${count > 3 ? `<br>另有 ${count - 3} 个文件` : ""}</div>`
                    : ""
            }<label class="radio-row"><input type="radio" name="scope" value="all" ${scope === "all" ? "checked" : ""}><span>全部归档</span><small>${files.length} 项 · ${bytes(archiveBytes)}</small></label></div></div><div class="field-group"><label for="conflict-policy" class="field-label">遇到重名文件时</label><select id="conflict-policy" class="field-select"><option value="ask">询问我如何处理</option value="rename">自动改名，保留两个版本</option><option value="skip">跳过已有文件</option><option value="overwrite">覆盖已有文件</option></select><p id="policy-note" class="field-note">保留已有文件，确认后再处理。</p></div><label class="check-label"><input id="open-after" type="checkbox" checked>完成后打开目标文件夹</label></div>` +
            footer(
                cancelButton() +
                    `<button id="start-extract" class="button primary">开始解压 ${icon("arrow")}</button>`,
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
        $("#target-valid").textContent = denied
            ? "此位置不可写，请更改目标文件夹。"
            : "目标位置可用 · 可用空间 126 GB";
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
        startTask(scopeFiles(range), "extract");
    };
    check();
}
function startTask(items, kind) {
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
        header("目标位置已有同名文件", "请选择要保留的版本", "layers", false) +
            `<div class="modal-content"><div class="conflict-name">${icon("pdf")}${esc(e.name)}</div><div class="comparison"><div><label>已有文件</label><strong>${bytes(Math.round(e.size * 0.875))}</strong><small>2026/10/01 · 09:42</small></div><div><label>压缩包中的文件</label><strong>${bytes(e.size)}</strong><small>2026/10/08 · 14:32</small></div></div><label class="check-label"><input id="apply-conflict-all" type="checkbox">对本次任务后续同类冲突应用此选择</label><div class="notice warning">覆盖会替换已有文件内容。选择“自动改名”可以保留两个版本。</div></div>` +
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
                  : "正在解压文件",
            `${t.kind === "create" ? "旅行资料" : "北境旅行.zip"} → ${esc(t.target)}`,
            t.kind === "create" ? "archive" : "extract",
            false,
        ) +
            `<div class="modal-content"><div class="progress-head"><strong id="progress-value">${t.progress}%</strong><span id="progress-phase">正在准备</span></div><div class="progress-track" role="progressbar" aria-label="任务进度" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${t.progress}"><i id="progress-fill" style="width:${t.progress}%"></i></div><div class="progress-current"><span id="progress-bytes"></span><span id="progress-count"></span></div><div class="notice"><span id="progress-file"></span><br><span class="field-note">${t.kind === "create" ? "归档完成校验后统一保存。取消不会替换已有归档。" : "文件通过校验后才发布。取消会保留已完成的文件。"}</span></div></div>` +
            footer(
                `<button id="background-task" class="button">返回浏览</button><button id="cancel-task" class="button ${t.status === "stopping" ? "" : "danger"}" ${t.status === "stopping" ? "disabled" : ""}>${t.status === "stopping" ? "正在停止…" : "取消任务"}</button>`,
                "后台处理",
            ),
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
    $("#progress-file").textContent =
        (t.status === "stopping" ? "停止中：" : "正在处理：") +
        t.items[done].name;
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
            created ? "新建压缩文件" : "北境旅行.zip",
            ok ? "check" : "warning",
        ) +
            `<div class="modal-content"><div class="result-hero"><div class="result-icon" style="${ok ? "" : `color:var(--warning);background:color-mix(in srgb,var(--warning) 9%,var(--surface))`}">${icon(ok ? "check" : cancel ? "close" : "warning")}</div><h2>${title}</h2><p>${ok ? (created ? "归档已完成校验并保存。" : "所选内容已处理完成。") : created ? "临时归档未发布，原有文件保持不变。" : "已经完成的文件会保留，不会因本次停止而删除。"}</p></div><div class="result-counts"><div><b>${successCount}</b><span>${created ? "已创建归档" : "成功文件"}</span></div><div><b>${created ? 0 : t.skipped.length}</b><span>跳过</span></div><div><b>${failedCount}</b><span>${cancel ? "未完成" : "失败"}</span></div></div>${!ok ? `<div class="notice ${cancel ? "" : "error"}">${cancel ? "任务已安全停止。可以重新规划未完成项。" : t.result === "source" ? "压缩文件在读取后被修改。请重新打开并重新选择内容。" : "目标磁盘空间不足。释放空间或选择其他位置后重试。"}</div>` : ""}<div class="result-destination">${icon("folder")}<span>${esc(t.target)}</span></div>${t.renamed ? `<p class="field-note">重名文件已改名为 ${esc(t.conflictEntry.name.replace(/(\.[^.]+)$/, " (2)$1"))}</p>` : ""}<details><summary class="details-title">查看处理明细</summary><div class="result-list">${t.completed.map((e) => `<div><span>${esc(e.name)}</span><span>已完成</span></div>`).join("")}${t.failed.map((e) => `<div class="bad"><span>${esc(e.name)}</span><span>${cancel ? "未完成" : "失败"}</span></div>`).join("")}${t.skipped.map((id) => `<div><span>${esc(byId.get(id).name)}</span><span>已跳过</span></div>`).join("")}</div></details></div>` +
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
        header("新建压缩文件", "添加内容，选择格式与保存位置", "archive") +
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
        $("#create-valid").textContent = valid
            ? "预检通过 · " + state.createItems.length + " 个文件 · 目标可写"
            : "请添加输入文件，并填写名称与保存位置。";
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
            "warning",
            false,
        ) +
            `<div class="modal-content"><div class="notice warning">已有归档会在新归档成功写入、校验并提交后被替换。取消或失败不会先删除已有文件。</div></div>` +
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
        header("设置", "外观与隐私", "settings") +
            `<div class="modal-content"><div class="settings-row"><label for="setting-theme">外观主题</label><select id="setting-theme" class="field-select"><option value="light">浅色</option><option value="dark">深色</option><option value="system">跟随系统</option></select></div><div class="settings-row"><label for="setting-density">列表密度</label><select id="setting-density" class="field-select"><option value="standard">标准</option><option value="compact">紧凑</option></select></div><label class="check-label"><input id="recent-enabled" type="checkbox" ${state.recent ? "checked" : ""}>在本机保留最近文件记录</label><div class="field-group"><button id="clear-recent" class="button">清空最近记录</button></div><div class="notice">所有归档处理都在本机进行。默认不上传文件、不启用遥测。清空最近记录不会删除原文件。</div></div>` +
            footer(cancelButton("完成"), "设置仅在本机生效"),
        "settings",
        "S01",
    );
    bindClose();
    $("#setting-theme").value = state.theme;
    $("#setting-theme").onchange = (e) => setTheme(e.target.value);
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
    state.theme = theme;
    const dark =
        theme === "dark" ||
        (theme === "system" &&
            matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.dataset.theme = dark ? "dark" : "light";
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
$("#app-brand").innerHTML = icon("archive") + "xdzip";
hydrate();
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
$("#extract-menu-button").onclick = () => {
    const n = $("#extract-menu");
    n.hidden = !n.hidden;
    $("#extract-menu-button").setAttribute("aria-expanded", String(!n.hidden));
};
$("#extract-menu")
    .querySelectorAll("button")
    .forEach(
        (b) =>
            (b.onclick = () => {
                $("#extract-menu").hidden = true;
                $("#extract-menu-button").setAttribute(
                    "aria-expanded",
                    "false",
                );
                showExtract(b.dataset.extract);
            }),
    );
document.addEventListener("click", (e) => {
    if (!e.target.closest(".split-button")) {
        $("#extract-menu").hidden = true;
        $("#extract-menu-button").setAttribute("aria-expanded", "false");
    }
});
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
    state.sort *= -1;
    render();
};
$("#tasks").onclick = showTask;
$("#settings").onclick = showSettings;
$("#platform").onchange = (event) => {
    document.body.dataset.platform = event.target.value;
    if ($("#modal").open) closeModal();
    $(".search-field kbd").textContent =
        event.target.value === "macos" ? "⌘ F" : "Ctrl F";
};
$("#modal").addEventListener("cancel", (event) => {
    event.preventDefault();
    windowClose();
});
document.addEventListener("keydown", (e) => {
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
        $("#extract-menu").hidden = true;
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
