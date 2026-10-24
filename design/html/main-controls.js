"use strict";
// Main-window command adapter. Shared controls own input/focus; this adapter
// owns demo archive state and never reads, writes or executes archive content.
window.XdzipMain = (() => {
    const ui = window.XdzipControls;
    const separator = () => ({ type: "separator" });
    const ready = () =>
        state.archiveState === "ready" &&
        tabs.some((tab) => tab.id === state.activeTab);
    const jobRunning = () =>
        state.testJob?.status === "running" ||
        (state.task &&
            ["running", "stopping", "conflict"].includes(state.task.status));
    const shortcut = (key) =>
        `${document.body.dataset.platform === "macos" ? "⌘" : "Ctrl+"}${key}`;
    const hasSelection = () => state.selected.size > 0;
    const showInspector = () => {
        $("#inspector").hidden = false;
        $("#inspector").classList.remove("collapsed-narrow");
    };
    const initialData = {
        entries,
        files,
        folders,
        byId,
        archiveBytes,
        compressedBytes,
    };
    const projectFolders = [
        { id: "project-design", name: "设计素材", parent: "", folder: true },
        { id: "project-docs", name: "文档", parent: "", folder: true },
    ];
    const projectFiles = Array.from({ length: 8 }, (_, index) => ({
        id: `project-file-${index}`,
        name: `${index < 4 ? "界面说明" : "贡献指南"}-${index + 1}.${index < 4 ? "pdf" : "md"}`,
        parent: index < 4 ? "project-design" : "project-docs",
        size: 1200000 + index * 50000,
        compressed: 800000 + index * 30000,
        type: index < 4 ? "PDF 文档" : "Markdown 文档",
        modified: "2026/10/08",
    }));
    const projectEntries = [...projectFolders, ...projectFiles];
    const projectData = {
        entries: projectEntries,
        files: projectFiles,
        folders: projectFolders,
        byId: new Map(projectEntries.map((entry) => [entry.id, entry])),
        archiveBytes: projectFiles.reduce((sum, file) => sum + file.size, 0),
        compressedBytes: projectFiles.reduce(
            (sum, file) => sum + file.compressed,
            0,
        ),
    };
    const freshSession = (id, title, data) => ({
        id,
        title,
        data,
        path: "",
        query: "",
        selected: new Set(),
        expanded: new Set(),
        sort: 0,
        sortKey: "name",
        view: "list",
        treeFocus: "",
    });
    let tabs = [
        freshSession("travel", "北境旅行.zip", initialData),
        freshSession("project", "项目素材.zip", projectData),
    ];
    tabs[0].selected = new Set(state.selected);
    const closedTabs = [];
    let testTimer = null;

    function entryNode(id) {
        return $(
            state.view === "grid"
                ? `#file-grid [data-id="${id}"]`
                : `#file-rows [data-id="${id}"]`,
        );
    }
    function saveTab() {
        const tab = tabs.find((item) => item.id === state.activeTab);
        if (tab)
            Object.assign(tab, {
                path: state.path,
                query: state.query,
                selected: new Set(state.selected),
                expanded: new Set(state.expanded),
                sort: state.sort,
                sortKey: state.sortKey,
                view: state.view,
                treeFocus: state.treeFocus,
            });
    }
    function loadData(data) {
        entries = data.entries;
        files = data.files;
        folders = data.folders;
        byId = data.byId;
        archiveBytes = data.archiveBytes;
        compressedBytes = data.compressedBytes;
    }
    function switchTab(id, focus = false) {
        const tab = tabs.find((item) => item.id === id);
        if (!tab) return;
        ui.close({ restoreFocus: false });
        saveTab();
        loadData(tab.data);
        Object.assign(state, {
            activeTab: tab.id,
            archiveTitle: tab.title,
            path: tab.path,
            query: tab.query,
            selected: new Set(tab.selected),
            expanded: new Set(tab.expanded),
            sort: tab.sort,
            sortKey: tab.sortKey,
            view: tab.view,
            treeFocus: tab.treeFocus,
            archiveState: "ready",
        });
        $("#search").value = tab.query;
        const wasWelcomeArtboard = document.body.classList.contains("welcome-artboard");
        setWelcomeMode(false);
        if (wasWelcomeArtboard && !applyingReviewPreset) {
            setCanvasLogicalScale("main");
            window.parent.postMessage({type:"xdzip-review-window", view:"main"}, "*");
        }
        render();
        if (focus) $(`[data-tab="${id}"]`)?.focus();
    }
    function closeTab(id) {
        const index = tabs.findIndex((tab) => tab.id === id);
        if (index < 0) return;
        saveTab();
        closedTabs.push(tabs[index]);
        if (closedTabs.length > 10) closedTabs.shift();
        tabs.splice(index, 1);
        if (!tabs.length) {
            state.activeTab = null;
            state.archiveState = "empty";
            enterWelcome();
            $("#welcome-open")?.focus();
        } else if (state.activeTab === id)
            switchTab(tabs[Math.min(index, tabs.length - 1)].id, true);
        else renderTabs();
    }
    function closeTabRange(anchor, rightOnly) {
        const index = tabs.findIndex(tab => tab.id === anchor);
        if (index < 0) return;
        const closing = tabs.filter((tab, offset) => tab.id !== anchor && (!rightOnly || offset > index));
        const ids = new Set(closing.map(tab => tab.id));
        saveTab();
        closedTabs.push(...closing);
        if (closedTabs.length > 10) closedTabs.splice(0, closedTabs.length - 10);
        tabs = tabs.filter(tab => !ids.has(tab.id));
        if (ids.has(state.activeTab)) switchTab(anchor, true);
        else renderTabs();
    }
    function enterWelcome(view = "welcome") {
        saveTab();
        document.body.classList.add("welcome-artboard");
        document.body.classList.toggle("welcome-narrow", view === "welcome-narrow");
        document.body.dataset.standalone = "false";
        setCanvasLogicalScale(view);
        if (window.parent !== window) window.parent.postMessage({type:"xdzip-review-window", view}, "*");
        state.activeTab = null;
        state.archiveState = "welcome";
        showWelcome();
    }
    function openSample(id) {
        if (document.body.classList.contains("welcome-mode")) {
            if (welcomeOpening) return;
            if ($("#modal").open) closeModal();
            beginWelcomeOpening(id === "travel" ? "北境旅行.zip" : "项目素材.zip", () => completeOpenSample(id));
            return;
        }
        completeOpenSample(id);
    }
    function completeOpenSample(id) {
        let tab = tabs.find((item) => item.id === id);
        if (!tab) {
            tab =
                closedTabs.find((item) => item.id === id) ||
                freshSession(
                    id,
                    id === "travel" ? "北境旅行.zip" : "项目素材.zip",
                    id === "travel" ? initialData : projectData,
                );
            tabs.push(tab);
        }
        if ($("#modal").open) closeModal();
        switchTab(tab.id, true);
    }
    function moveTab(id, target) {
        const from = tabs.findIndex(tab => tab.id === id);
        if (from < 0 || target < 0 || target >= tabs.length || from === target) return;
        tabs.splice(target, 0, tabs.splice(from, 1)[0]);
        renderTabs();
    }
    function renderTabs() {
        const bar = $(".archive-tabs");
        bar.setAttribute("role", "tablist");
        bar.innerHTML =
            `<div class="tab-viewport">` +
            (state.activeTab === null ? `<div class="archive-tab welcome-tab" role="tab" aria-selected="true"><span class="tab-icon">${icon("archive")}</span><span class="tab-label">起始页</span></div>` : "") +
            (state.activeTab === null ? [] : tabs)
                .map(
                    (tab) =>
                        `<div class="archive-tab ${tab.id === state.activeTab ? "" : "inactive"}" data-tab="${tab.id}" role="tab" aria-selected="${tab.id === state.activeTab}" tabindex="${tab.id === state.activeTab ? 0 : -1}" data-tooltip="${esc(tab.title)}"><span class="tab-icon">${icon(tab.id === state.activeTab ? "archive" : "archive-grey")}</span><span class="tab-label">${esc(tab.title)}</span><button class="tab-close" ${tab.id === "travel" ? 'id="close-archive"' : 'id="close-example-tab"'} data-close-tab="${tab.id}" aria-label="关闭 ${esc(tab.title)}" data-tooltip="关闭压缩包，${shortcut("W")}">${icon("close")}</button></div>`,
                )
                .join("") +
            `</div><div class="tab-strip-actions"><button id="new-tab" class="header-icon" aria-label="打开其他压缩包" data-tooltip="打开其他压缩包，${shortcut("O")}">${icon("plus")}</button><button id="tabs-previous" class="header-icon tab-overflow-control" aria-label="${settingText("tabs.previous")}" hidden>${icon("chevron-right")}</button><button id="tabs-next" class="header-icon tab-overflow-control" aria-label="${settingText("tabs.next")}" hidden>${icon("chevron-right")}</button><button id="tabs-list" class="header-icon tab-overflow-control" aria-label="${settingText("tabs.list")}" hidden>${icon("chevron-down")}</button></div>`;
        bar.querySelectorAll("[data-tab]").forEach((node) => {
            node.onclick = (event) => {
                if (!event.target.closest("[data-close-tab]"))
                    switchTab(node.dataset.tab, true);
            };
            node.onkeydown = (event) => {
                if (event.target !== node) return;
                const index = tabs.findIndex(
                    (tab) => tab.id === node.dataset.tab,
                );
                if (
                    ["ArrowLeft", "ArrowRight", "Home", "End"].includes(
                        event.key,
                    )
                ) {
                    event.preventDefault();
                    const target =
                        event.key === "Home"
                            ? 0
                            : event.key === "End"
                              ? tabs.length - 1
                              : (index +
                                    (event.key === "ArrowRight" ? 1 : -1) +
                                    tabs.length) %
                                tabs.length;
                    switchTab(tabs[target].id, true);
                } else if (["Enter", " "].includes(event.key)) {
                    event.preventDefault();
                    switchTab(node.dataset.tab, true);
                } else if (event.key === "Delete") {
                    event.preventDefault();
                    closeTab(node.dataset.tab);
                } else if (
                    (event.key === "F10" && event.shiftKey) ||
                    event.key === "ContextMenu"
                ) {
                    event.preventDefault();
                    tabMenu(node.dataset.tab, node);
                }
            };
            node.oncontextmenu = (event) => {
                event.preventDefault();
                tabMenu(node.dataset.tab, node, {
                    x: event.clientX,
                    y: event.clientY,
                });
            };
        });
        bar.querySelectorAll("[data-close-tab]").forEach(
            (button) =>
                (button.onclick = (event) => {
                    event.stopPropagation();
                    closeTab(button.dataset.closeTab);
                }),
        );
        $("#new-tab").onclick = () => dispatch("open");
        window.XdzipTabStrip.attach(bar, {
            tabs: () => state.activeTab === null ? [] : tabs,
            active: () => state.activeTab,
            move: moveTab,
            close: closeTab,
            list: trigger => { ui.open({trigger, label: settingText("tabs.list"), items: tabs.map(tab => ({
                type: "radio", id: "activate-" + tab.id, label: tab.title, checked: tab.id === state.activeTab,
                run: () => switchTab(tab.id, true),
            }))}); document.querySelector(".ui-menu")?.classList.add("tab-list-menu"); },
        });
    }
    function tabMenu(id, trigger, point) {
        const index = tabs.findIndex(tab => tab.id === id);
        if (index < 0) return;
        ui.open({
            trigger, point,
            label: "压缩包标签操作",
            origin: () => $(`[data-tab="${state.activeTab}"]`) || $("#new-tab"),
            items: [
                { id: "tab-close", label: settingText("tabs.closeCommand"), run: () => closeTab(id) },
                { id: "tab-close-others", label: settingText("tabs.closeOthers"), disabled: tabs.length < 2, run: () => closeTabRange(id, false) },
                { id: "tab-close-right", label: settingText("tabs.closeRight"), disabled: index === tabs.length - 1, run: () => closeTabRange(id, true) },
            ],
        });
    }
    function openPickerView() {
        dialog(
            header(
                "打开压缩包",
                "系统文件选择器的设计替身，使用示例压缩包",
                "open",
            ) +
                `<div class="modal-content"><p class="field-note">选择下列示例验证多压缩包交互。</p><div class="field-group"><button id="pick-sample" class="button">${icon("archive")}北境旅行.zip</button><button id="pick-project" class="button">${icon("archive-grey")}项目素材.zip</button></div></div>` +
                footer(cancelButton(), "仅使用示例数据"),
            "picker",
            "B01",
            "#pick-sample",
        );
        bindClose();
        $("#pick-sample").onclick = () => openSample("travel");
        $("#pick-project").onclick = () => openSample("project");
    }
    function setSort(key, direction = null) {
        state.sort =
            direction === null
                ? state.sortKey === key && state.sort === 1
                    ? -1
                    : 1
                : direction;
        state.sortKey = key;
        render();
    }
    function setView(view) {
        state.view = view;
        render();
    }
    function selectList(mode) {
        const list = visibleEntries();
        state.selected = new Set(
            mode === "all"
                ? list.map((entry) => entry.id)
                : mode === "invert"
                  ? list
                        .filter((entry) => !state.selected.has(entry.id))
                        .map((entry) => entry.id)
                  : [],
        );
        state.anchor = null;
        render();
    }
    async function copySelection(fullPath) {
        const text = entries
            .filter((entry) => state.selected.has(entry.id))
            .map((entry) =>
                fullPath
                    ? directoryPath(entry.parent).concat(entry.name).join(" / ")
                    : entry.name,
            )
            .join("\n");
        if (!text) return;
        try {
            if (!navigator.clipboard?.writeText)
                throw new Error("clipboard-unavailable");
            await navigator.clipboard.writeText(text);
            toast(fullPath ? "压缩包内路径已复制。" : "文件名称已复制。");
        } catch {
            dialog(
                header(
                    "复制文本",
                    "浏览器限制了直接复制，请选择下方文本后复制。",
                    "file-text",
                ) +
                    `<div class="modal-content"><textarea id="copy-text" class="field-input" rows="5" readonly>${esc(text)}</textarea></div>` +
                    footer(cancelButton("完成")),
                "copy",
                "B01",
                "#copy-text",
            );
            bindClose();
            $("#copy-text").select();
        }
    }
    const commands = {
        open: {
            label: "打开压缩包…",
            icon: "open",
            hotkey: "O",
            run: openPickerView,
        },
        create: {
            label: "创建压缩包…",
            icon: "new",
            hotkey: "N",
            enabled: () => !jobRunning(),
            reason: "请等待当前演示任务结束",
            run: showCreate,
        },
        test: {
            label: "测试压缩包完整性…",
            icon: "archive-test",
            enabled: () => ready() && !jobRunning(),
            reason: "请先打开压缩包并等待当前任务结束",
            run: showTest,
        },
        properties: {
            label: "文件属性",
            enabled: () => ready() && hasSelection(),
            reason: "",
            run: showInspector,
        },
        "archive-properties": {
            label: "压缩包信息",
            icon: "archive",
            enabled: ready,
            reason: "压缩包目录尚未就绪",
            run: () => {
                state.selected.clear();
                render();
                showInspector();
            },
        },
        "extract-all": {
            label: "全部解压…",
            enabled: () => ready() && files.length > 0 && !jobRunning(),
            reason: "请等待压缩包读取或当前任务结束",
            run: () => showExtract("all"),
        },
        "extract-selected": {
            label: "解压所选…",
            enabled: () =>
                ready() && selectedFiles().length > 0 && !jobRunning(),
            reason: "",
            run: () => showExtract("selected"),
        },
        "extract-results": {
            label: "解压搜索结果…",
            enabled: () =>
                ready() &&
                Boolean(state.query) &&
                visibleEntries().length > 0 &&
                !jobRunning(),
            reason: "仅有搜索结果时可用",
            run: () => showExtract("results"),
        },
        "select-all": {
            label: "全选当前列表",
            hotkey: "A",
            enabled: () => ready() && visibleEntries().length > 0,
            reason: "当前没有可选择条目",
            run: () => selectList("all"),
        },
        "select-none": {
            label: "取消选择",
            hotkey: "⇧A",
            enabled: hasSelection,
            reason: "当前没有选择",
            run: () => selectList("none"),
        },
        "select-invert": {
            label: "反向选择",
            enabled: () => ready() && visibleEntries().length > 0,
            reason: "当前没有可选择条目",
            run: () => selectList("invert"),
        },
        "copy-name": {
            label: "复制名称",
            enabled: hasSelection,
            reason: "请先选择条目",
            run: () => copySelection(false),
        },
        "copy-path": {
            label: "复制压缩包内路径",
            enabled: hasSelection,
            reason: "请先选择条目",
            run: () => copySelection(true),
        },
        "view-list": {
            label: "列表视图",
            icon: "view-list",
            run: () => setView("list"),
        },
        "view-grid": {
            label: "网格视图",
            icon: "view-grid",
            run: () => setView("grid"),
        },
        "toggle-inspector": {
            label: "文件预览 / 详情",
            run: () => {
                if ($("#inspector").hidden) showInspector();
                else $("#inspector").hidden = true;
            },
        },
        settings: { label: "设置…", icon: "settings", run: showSettings },
        tasks: {
            label: "查看任务",
            icon: "task-list",
            run: () =>
                state.testJob?.status === "running" ||
                (!state.task && state.testJob)
                    ? showTestView()
                    : showTask(),
        },
        search: {
            label: "搜索当前范围",
            icon: "search",
            hotkey: "F",
            enabled: ready,
            reason: "请先打开可浏览压缩包",
            run: () => {
                $("#search").focus();
                $("#search").select();
            },
        },
    };
    function dispatch(id) {
        if (welcomeOpening && ["open", "create"].includes(id)) return false;
        const command = commands[id];
        if (!command || (command.enabled && !command.enabled())) return false;
        command.run();
        return true;
    }
    function item(id, additions = {}) {
        const command = commands[id];
        return {
            id,
            label: command.label,
            icon: command.icon,
            shortcut: command.hotkey ? shortcut(command.hotkey) : "",
            disabled: command.enabled ? !command.enabled() : false,
            reason: command.reason,
            run: () => dispatch(id),
            ...additions,
        };
    }
    function sortItems() {
        return [
            ["name", "名称"],
            ["size", "原始大小"],
            ["compressed", "压缩大小"],
            ["type", "类型"],
            ["modified", "修改日期"],
        ]
            .map(([key, label]) => ({
                id: `sort-${key}`,
                label,
                type: "radio",
                checked: state.sort !== 0 && state.sortKey === key,
                disabled: !ready(),
                run: () => setSort(key, state.sort || 1),
            }))
            .concat([
                separator(),
                {
                    id: "sort-asc",
                    label: "升序",
                    type: "radio",
                    checked: state.sort > 0,
                    disabled: !ready(),
                    run: () => setSort(state.sortKey, 1),
                },
                {
                    id: "sort-desc",
                    label: "降序",
                    type: "radio",
                    checked: state.sort < 0,
                    disabled: !ready(),
                    run: () => setSort(state.sortKey, -1),
                },
                separator(),
                {
                    id: "sort-original",
                    label: "按原始顺序排列",
                    type: "radio",
                    checked: state.sort === 0,
                    disabled: !ready(),
                    run: () => setSort("name", 0),
                },
            ]);
    }
    function viewItems() {
        return [
            item("view-list", {
                type: "radio",
                checked: state.view === "list",
            }),
            item("view-grid", {
                type: "radio",
                checked: state.view === "grid",
            }),
            separator(),
            item("toggle-inspector", {
                type: "checkbox",
                checked: !$("#inspector").hidden,
            }),
        ];
    }
    function listMenu(
        trigger,
        point,
        entryId = null,
        suppressTouchRelease = false,
    ) {
        const target = entryId ? byId.get(entryId) : null;
        if (target && !state.selected.has(entryId)) {
            state.selected = new Set([entryId]);
            state.anchor = entryId;
            render();
        }
        const items = [];
        if (target)
            items.push({ id: "open-entry", label: "打开", run: () => activateEntry(entryId) });
        if (target?.folder)
            items.push(
                {
                    id: "enter-directory",
                    label: "进入目录",
                    icon: "folder",
                    run: () => choosePath(entryId),
                },
                separator(),
            );
        if (target)
            items.push(
                item("extract-selected"),
                item("properties"),
                separator(),
                item("copy-name"),
                item("copy-path"),
                separator(),
            );
        else
            items.push(
                item("select-all"),
                item("select-none"),
                item("select-invert"),
                separator(),
            );
        items.push(
            { id: "view-menu", label: "视图", children: viewItems() },
            { id: "sort-menu", label: "排序方式", children: sortItems() },
        );
        const restored = entryId
            ? () => entryNode(entryId)
            : $(".table-scroll");
        ui.open({
            trigger: entryId ? entryNode(entryId) : trigger,
            point,
            label: target ? "条目操作" : "文件区域操作",
            origin: restored,
            items,
            suppressTouchRelease,
        });
    }
    function bindEntry(node, entry, list) {
        node.oncontextmenu = (event) => {
            event.preventDefault();
            listMenu(node, { x: event.clientX, y: event.clientY }, entry.id);
        };
        node.addEventListener(
            "keydown",
            (event) => {
                if (
                    event.key === "ContextMenu" ||
                    (event.key === "F10" && event.shiftKey)
                ) {
                    event.preventDefault();
                    event.stopImmediatePropagation();
                    listMenu(node, null, entry.id);
                }
            },
            true,
        );
        let held = null;
        node.addEventListener("pointerdown", (event) => {
            if (event.pointerType !== "touch") return;
            held = setTimeout(() => {
                held = null;
                listMenu(
                    node,
                    { x: event.clientX, y: event.clientY },
                    entry.id,
                    true,
                );
            }, 550);
        });
        ["pointerup", "pointercancel", "pointermove"].forEach((type) =>
            node.addEventListener(type, () => {
                clearTimeout(held);
                held = null;
            }),
        );
        if (state.view === "grid")
            node.onclick = (event) => {
                if (
                    event.shiftKey &&
                    state.anchor &&
                    list.some((item) => item.id === state.anchor)
                ) {
                    const start = list.findIndex(
                            (item) => item.id === state.anchor,
                        ),
                        end = list.findIndex((item) => item.id === entry.id);
                    state.selected = new Set(
                        list
                            .slice(
                                Math.min(start, end),
                                Math.max(start, end) + 1,
                            )
                            .map((item) => item.id),
                    );
                } else if (
                    event.ctrlKey ||
                    event.metaKey ||
                    event.target.matches("input")
                ) {
                    state.selected.has(entry.id)
                        ? state.selected.delete(entry.id)
                        : state.selected.add(entry.id);
                    state.anchor = entry.id;
                } else {
                    state.selected = new Set([entry.id]);
                    state.anchor = entry.id;
                }
                render();
                entryNode(entry.id)?.focus({ preventScroll: true });
            };
        node.ondblclick = () => activateEntry(entry.id);
        node.onkeydown = (event) => {
            if (event.key === "Enter") {
                event.preventDefault();
                activateEntry(entry.id);
            } else if (event.key === " ") {
                event.preventDefault();
                state.selected.has(entry.id)
                    ? state.selected.delete(entry.id)
                    : state.selected.add(entry.id);
                state.anchor = entry.id;
                render();
                entryNode(entry.id)?.focus({ preventScroll: true });
            } else if (
                [
                    "ArrowLeft",
                    "ArrowRight",
                    "ArrowUp",
                    "ArrowDown",
                    "Home",
                    "End",
                ].includes(event.key)
            ) {
                event.preventDefault();
                const index = list.indexOf(entry),
                    columns =
                        state.view === "grid"
                            ? Math.max(
                                  1,
                                  getComputedStyle($("#file-grid")).gridTemplateColumns.trim().split(/\s+/).length,
                              )
                            : 1;
                const offset = {
                    ArrowLeft: -1,
                    ArrowRight: 1,
                    ArrowUp: -columns,
                    ArrowDown: columns,
                }[event.key];
                const next =
                    list[
                        event.key === "Home"
                            ? 0
                            : event.key === "End"
                              ? list.length - 1
                              : Math.max(
                                    0,
                                    Math.min(list.length - 1, index + offset),
                                )
                    ];
                if (next) {
                    if (event.shiftKey && state.anchor) {
                        const start = list.findIndex(
                                (item) => item.id === state.anchor,
                            ),
                            end = list.indexOf(next);
                        state.selected = new Set(
                            list
                                .slice(
                                    Math.min(start, end),
                                    Math.max(start, end) + 1,
                                )
                                .map((item) => item.id),
                        );
                        render();
                    } else if (!event.ctrlKey && !event.metaKey) {
                        state.selected = new Set([next.id]);
                        state.anchor = next.id;
                        render();
                    }
                    entryNode(next.id)?.focus();
                }
            }
        };
    }
    function sync(list) {
        renderTabs();
        $("#list-view").classList.toggle("view-active", state.view === "list");
        $("#grid-view").classList.toggle("view-active", state.view === "grid");
        $("#list-view").setAttribute(
            "aria-pressed",
            String(state.view === "list"),
        );
        $("#grid-view").setAttribute(
            "aria-pressed",
            String(state.view === "grid"),
        );
        $(".files-pane").classList.toggle(
            "view-grid-active",
            state.view === "grid",
        );
        const grid = $("#file-grid");
        grid.hidden = state.view !== "grid";
        grid.innerHTML =
            state.view === "grid"
                ? list
                      .map(
                          (entry) =>
                              `<div class="grid-entry ${state.selected.has(entry.id) ? "selected" : ""}" tabindex="0" data-id="${entry.id}" role="option" aria-selected="${state.selected.has(entry.id)}"><input type="checkbox" tabindex="-1" aria-label="选择 ${esc(entry.name)}" ${state.selected.has(entry.id) ? "checked" : ""}><span class="entry-icon">${["lake", "sunrise"].includes(entry.id) ? photo(entry.id, "entry-photo") : icon(fileIcon(entry))}</span><span class="grid-entry-name" data-tooltip="${esc(entry.name)}">${esc(displayEntryName(entry))}</span><small>${entry.folder ? "文件夹" : bytes(entry.size)}</small></div>`,
                      )
                      .join("")
                : "";
        document
            .querySelectorAll(
                state.view === "grid"
                    ? "#file-grid [data-id]"
                    : "#file-rows tr",
            )
            .forEach((node) =>
                bindEntry(node, byId.get(node.dataset.id), list),
            );
        $("#select-all").setAttribute("aria-label", "选择当前列表全部条目");
        [
            "sort-direction",
            "list-view",
            "grid-view",
            "title-settings",
            "tasks",
            "properties-toggle",
        ].forEach((id) => {
            const node = $("#" + id);
            if (node)
                node.dataset.tooltip =
                    node.getAttribute("aria-label") || node.textContent.trim();
        });
        $("#test-archive").disabled = !commands.test.enabled();
        $("#create").disabled = !commands.create.enabled();
        $("#extract").disabled = !(
            hasSelection()
                ? commands["extract-selected"]
                : commands["extract-all"]
        ).enabled();
        $("#extract-menu-button").disabled = !ready();
        document.querySelectorAll("[data-sort]").forEach((button) => {
            button
                .closest("th")
                .setAttribute(
                    "aria-sort",
                    state.sort && state.sortKey === button.dataset.sort
                        ? state.sort > 0
                            ? "ascending"
                            : "descending"
                        : "none",
                );
        });
        $("#sort-name")
            .closest("th")
            .setAttribute(
                "aria-sort",
                state.sort && state.sortKey === "name"
                    ? state.sort > 0
                        ? "ascending"
                        : "descending"
                    : "none",
            );
        if (!ready()) $("#search").disabled = true;
        else $("#search").disabled = false;
        $("#search-clear").hidden = !$("#search").value;
    }
    function directoryPath(id) {
        const names = [];
        while (id) {
            const node = byId.get(id);
            if (!node) break;
            names.unshift(node.name);
            id = node.parent;
        }
        return names;
    }
    function visibleDirectories() {
        const output = [];
        const append = (parent, depth, continuations = []) => {
            const siblings = folders.filter((node) => node.parent === parent);
            siblings.forEach((node, index) => {
                const last = index === siblings.length - 1;
                output.push({ node, depth, last, continuations });
                if (state.expanded.has(node.id))
                    append(
                        node.id,
                        depth + 1,
                        depth === 0 ? [] : [...continuations, !last],
                    );
            });
        };
        append("", 0);
        return output;
    }
    function renderTree() {
        const tree = $("#tree");
        tree.setAttribute("role", "tree");
        if (!ready()) {
            tree.innerHTML = `<div class="tree-row tree-root active" role="treeitem" aria-disabled="true" tabindex="-1"><span class="tree-root-icon">${icon("folder")}</span><span class="tree-label">根目录</span></div>${state.archiveState === "loading" ? '<p class="tree-notice">正在读取完整目录…</p>' : state.archiveState === "empty" ? '<p class="tree-notice">没有子目录</p>' : '<p class="tree-notice">目录信息未就绪</p>'}`;
            return;
        }
        const visible = visibleDirectories();
        const pathAncestors = [];
        let parent = state.path;
        while (parent) {
            pathAncestors.push(parent);
            parent = byId.get(parent)?.parent;
        }
        tree.innerHTML =
            `<div class="tree-row tree-root ${!state.path ? "active" : ""}" data-directory="" role="treeitem" aria-label="根目录，${entries.filter(entry => entry.parent === "").length} 项" aria-level="1" aria-selected="${!state.path}" tabindex="${state.treeFocus ? -1 : 0}"><span class="tree-root-icon">${icon("folder")}</span><span class="tree-label">根目录</span></div>` +
            `<div class="tree-scroll"><div class="tree-hierarchy">${visible
                .map(({ node, depth, last, continuations }) => {
                    const hasChildren = folders.some(
                            (child) => child.parent === node.id,
                        ),
                        expanded = state.expanded.has(node.id),
                        current = state.path === node.id;
                    const marker =
                        !current &&
                        !expanded &&
                        pathAncestors.includes(node.id);
                    const guides =
                        depth > 0
                            ? continuations
                                  .map((continues, index) =>
                                      continues
                                          ? `<i class="tree-guide-continue" style="--guide-depth:${index}"></i>`
                                          : "",
                                  )
                                  .join("") +
                              `<i class="tree-guide-parent ${last ? "tree-guide-last" : ""}" style="--guide-depth:${depth - 1}"></i>`
                            : "";
                    return `<div class="tree-row ${current ? "active" : ""}" data-directory="${node.id}" role="treeitem" aria-label="${esc(node.name)}，${files.filter(file => withinDirectory(file, node.id)).length} 个文件" aria-level="${depth + 1}" aria-selected="${current}" ${hasChildren ? `aria-expanded="${expanded}"` : ""} tabindex="${state.treeFocus === node.id ? 0 : -1}" style="--tree-depth:${depth}">${guides}<button class="tree-disclosure" tabindex="-1" data-toggle="${node.id}" aria-label="${expanded ? "收起" : "展开"} ${esc(node.name)}" ${hasChildren ? "" : 'disabled aria-hidden="true"'}>${hasChildren ? icon(expanded ? "chevron-down" : "chevron-right") : ""}</button><span class="tree-folder">${icon("folder")}</span><span class="tree-label">${esc(node.name)}</span>${marker ? `<span class="tree-branch-marker" aria-label="当前目录位于此分支">${icon("tree-active-branch")}</span>` : ""}</div>`;
                })
                .join("")}</div></div>`;
        tree.querySelectorAll("[data-directory]").forEach((row) => {
            row.onclick = (event) => {
                if (event.target.closest(".tree-disclosure")) return;
                state.treeFocus = row.dataset.directory;
                choosePath(row.dataset.directory);
            };
            row.onfocus = () => {
                state.treeFocus = row.dataset.directory;
                tree.querySelectorAll("[data-directory]").forEach((item) => {
                    item.tabIndex = item === row ? 0 : -1;
                });
            };
            row.onkeydown = (event) => {
                if (event.target !== row) return;
                const rows = [...tree.querySelectorAll("[data-directory]")],
                    index = rows.indexOf(row),
                    node = byId.get(row.dataset.directory);
                if (
                    ["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)
                ) {
                    event.preventDefault();
                    rows[
                        event.key === "Home"
                            ? 0
                            : event.key === "End"
                              ? rows.length - 1
                              : Math.max(
                                    0,
                                    Math.min(
                                        rows.length - 1,
                                        index +
                                            (event.key === "ArrowDown"
                                                ? 1
                                                : -1),
                                    ),
                                )
                    ]?.focus();
                } else if (event.key === "ArrowRight" && node) {
                    event.preventDefault();
                    if (!state.expanded.has(node.id)) toggleDirectory(node.id);
                    else rows[index + 1]?.focus();
                } else if (event.key === "ArrowLeft" && node) {
                    event.preventDefault();
                    if (state.expanded.has(node.id)) toggleDirectory(node.id);
                    else
                        tree.querySelector(
                            `[data-directory="${node.parent}"]`,
                        )?.focus();
                } else if (["Enter", " "].includes(event.key)) {
                    event.preventDefault();
                    choosePath(row.dataset.directory);
                    tree.querySelector(
                        `[data-directory="${row.dataset.directory}"]`,
                    )?.focus();
                } else if (
                    event.key === "ContextMenu" ||
                    (event.key === "F10" && event.shiftKey)
                ) {
                    event.preventDefault();
                    directoryMenu(row.dataset.directory, row);
                }
            };
            row.oncontextmenu = (event) => {
                event.preventDefault();
                directoryMenu(row.dataset.directory, row, {
                    x: event.clientX,
                    y: event.clientY,
                });
            };
        });
        const scroller = tree.querySelector(".tree-scroll");
        scroller.onscroll = () => scroller.style.setProperty("--tree-scroll-offset", `${scroller.scrollLeft}px`);
        tree.querySelectorAll("[data-toggle]").forEach(
            (button) =>
                (button.onclick = (event) => {
                    event.stopPropagation();
                    toggleDirectory(button.dataset.toggle);
                }),
        );
    }
    function toggleDirectory(id) {
        if (!id || !folders.some((node) => node.parent === id)) return;
        state.expanded.has(id)
            ? state.expanded.delete(id)
            : state.expanded.add(id);
        state.treeFocus = id;
        renderTree();
        $("#tree")
            .querySelector(`[data-directory="${id}"]`)
            ?.focus({ preventScroll: true });
    }
    function directoryMenu(id, trigger, point) {
        const node = byId.get(id),
            hasChildren = folders.some((child) => child.parent === id);
        ui.open({
            trigger,
            point,
            label: "目录操作",
            origin: () => $("#tree").querySelector(`[data-directory="${id}"]`),
            items: [
                {
                    id: "directory-open",
                    label: id ? "进入目录" : "根目录",
                    icon: "folder",
                    run: () => choosePath(id),
                },
                {
                    id: "directory-expand",
                    label: settingText(!id || state.expanded.has(id) ? "ui.9c9c21cc25c0" : "ui.521dbd520244"),
                    disabled: !id || !hasChildren,
                    reason: !id ? settingText("directory.rootAlwaysExpanded") : !hasChildren ? settingText("ui.fa58108c4823") : "",
                    run: () => toggleDirectory(id),
                },
                separator(),
                {
                    id: "directory-search",
                    label: "在此目录中搜索",
                    icon: "search",
                    run: () => {
                        choosePath(id);
                        $("#search").focus();
                    },
                },
            ],
        });
    }
    function showTest() {
        if (!commands.test.enabled()) return;
        clearInterval(testTimer);
        state.testJob = {
            title: state.archiveTitle,
            total: files.length,
            progress: 0,
            status: "running",
        };
        state.task = null;
        render();
        showTestView();
        testTimer = setInterval(() => {
            const job = state.testJob;
            if (!job || job.status !== "running" || job.reviewPaused) return;
            job.progress = Math.min(100, job.progress + 10);
            if (job.progress === 100) {
                job.status = "finished";
                clearInterval(testTimer);
                render();
            }
            if (state.modal === "archive-test") showTestView(false);
            updateTaskCount();
        }, 240);
    }
    function showTestView(focus = true) {
        const job = state.testJob;
        if (!job) return;
        const running = job.status === "running";
        const outcome = running ? "正在检查压缩包" : job.status === "cancelled" ? "检查已取消" : "检查完成";
        const totalBytes = files.reduce((sum, entry) => sum + entry.size, 0);
        const index = Math.min(files.length - 1, Math.floor(files.length * job.progress / 100));
        const file = files[Math.max(0, index)];
        const content = header(outcome, `${esc(job.title)}，界面演示`, "document-check") +
            taskProgressBody({
                scopeHeading: "检查对象", scopeValue: `下载 / ${job.title}`,
                progressName: "完整性检查进度", progress: job.progress,
                processed: `已读取 ${bytes(Math.round(totalBytes * job.progress / 100))} / ${bytes(totalBytes)}`,
                count: `${Math.floor(job.total * job.progress / 100)} / ${job.total} 个文件`,
                speedLabel: "平均读取", speed: running ? "6.2 MB/s" : "—",
                remaining: running ? "约 5 秒" : "—",
                phase: running ? "读取并校验" : job.status === "cancelled" ? "已停止" : "已结束",
                currentFile: file?.name || "—", entry: file,
            }) + taskProgressFooter({
                note: "界面预览，不执行实际校验。", backgroundId: "close-archive-test",
                backgroundText: running ? "返回浏览" : "关闭", cancelId: "cancel-archive-test",
                cancelText: "取消检查", finished: !running,
            });
        if (focus || state.modal !== "archive-test")
            dialog(content, "archive-test", "T01", "#close-archive-test");
        else {
            const area = $("#modal .modal-content");
            area.innerHTML = new DOMParser()
                .parseFromString(content, "text/html")
                .querySelector(".modal-content").innerHTML;
            $("#modal h1").textContent = outcome;
            if (job.status !== "running") {
                $("#cancel-archive-test")?.remove();
                $("#close-archive-test").textContent = "关闭";
            }
        }
        $("#close-archive-test").onclick = closeModal;
        if ($("#cancel-archive-test"))
            $("#cancel-archive-test").onclick = () => {
                clearInterval(testTimer);
                job.status = "cancelled";
                render();
                showTestView();
            };
    }
    function resetJobs() {
        clearInterval(testTimer);
        state.testJob = null;
    }
    function install() {
        $("#open").onclick = () => dispatch("open");
        $("#create").onclick = () => dispatch("create");
        $("#test-archive").onclick = () => dispatch("test");
        $("#grid-view").onclick = () => dispatch("view-grid");
        $("#list-view").onclick = () => dispatch("view-list");
        $("#tasks").onclick = () => ui.open({
            trigger: $("#tasks"),
            label: "任务菜单",
            placement: "top-end",
            items: [
                ...(state.task ? [{ id: "task-details", label: state.task.kind === "create" ? "创建任务" : "解压任务", icon: "task-list", status: state.task.status === "finished" ? state.task.result === "success" ? "已完成" : state.task.result === "cancelled" ? "已取消" : "未完成" : state.task.status === "conflict" ? "等待处理" : "进行中", statusKind: state.task.status === "finished" ? state.task.result === "success" ? "succeeded" : state.task.result === "cancelled" ? "cancelled" : "failed" : state.task.status === "conflict" ? "waiting" : "running", run: () => dispatch("tasks") }] : []),
                ...(state.testJob ? [{ id: state.task ? "check-details" : "task-details", label: "完整性检查预览", icon: "task-list", status: state.testJob.status === "running" ? "进行中" : "已结束", run: showTestView }] : []),
                ...(!state.task && !state.testJob ? [{ id: "no-tasks", label: "暂无任务", disabled: true }] : []),
            ],
        });
        $("#extract").onclick = () =>
            dispatch(hasSelection() ? "extract-selected" : "extract-all");
        $("#search-clear").onclick = () => {
            $("#search").value = "";
            $("#search").dispatchEvent(new Event("input", { bubbles: true }));
            $("#search").focus();
        };
        $("#sort-name").onclick = () => setSort("name");
        document
            .querySelectorAll("[data-sort]")
            .forEach(
                (button) =>
                    (button.onclick = () => setSort(button.dataset.sort)),
            );
        $("#sort-direction").onclick = () =>
            ui.open({
                trigger: $("#sort-direction"),
                label: "排序方式",
                placement: "bottom-end",
                items: sortItems(),
            });
        $("#more-actions").onclick = () =>
            ui.open({
                trigger: $("#more-actions"),
                label: "更多压缩包操作",
                items: [item("settings")],
            });
        $("#extract-menu-button").onclick = () =>
            ui.open({
                trigger: $("#extract-menu-button"),
                label: "解压范围",
                id: "extract-menu",
                items: [
                    item("extract-all", { data: { extract: "all" } }),
                    item("extract-selected", { data: { extract: "selected" } }),
                    separator(),
                    item("extract-results", { data: { extract: "results" } }),
                ],
            });
        $(".table-scroll").tabIndex = -1;
        $(".table-scroll").addEventListener("contextmenu", (event) => {
            if (event.target.closest("[data-id]")) return;
            event.preventDefault();
            listMenu($(".table-scroll"), {
                x: event.clientX,
                y: event.clientY,
            });
        });
        document
            .querySelector("thead")
            .addEventListener("contextmenu", (event) => {
                event.preventDefault();
                event.stopPropagation();
                ui.open({
                    trigger: event.target.closest("th"),
                    point: { x: event.clientX, y: event.clientY },
                    label: "表头排序",
                    origin: $("#sort-name"),
                    items: sortItems(),
                });
            });
        document.addEventListener(
            "keydown",
            (event) => {
                if (
                    event.defaultPrevented ||
                    event.isComposing ||
                    $("#modal").open ||
                    ui.isOpen
                )
                    return;
                const modifier = event.metaKey || event.ctrlKey;
                const key = event.key.toLowerCase();
                if (modifier && ["o", "n", "f"].includes(key)) {
                    event.preventDefault();
                    if (key !== "f" || !document.body.classList.contains("welcome-mode"))
                        dispatch({ o: "open", n: "create", f: "search" }[key]);
                } else if (
                    modifier &&
                    key === "a" &&
                    !event.target.matches("input,textarea")
                ) {
                    event.preventDefault();
                    dispatch(event.shiftKey ? "select-none" : "select-all");
                } else if (
                    modifier &&
                    key === "c" &&
                    !event.target.matches("input,textarea") &&
                    event.target.closest(".files-pane")
                ) {
                    event.preventDefault();
                    dispatch("copy-name");
                } else if (modifier && key === "w") {
                    event.preventDefault();
                    closeTab(state.activeTab);
                } else if (event.ctrlKey && event.key === "Tab") {
                    event.preventDefault();
                    const index = tabs.findIndex(
                        (tab) => tab.id === state.activeTab,
                    );
                    if (tabs.length)
                        switchTab(
                            tabs[
                                (index +
                                    (event.shiftKey ? -1 : 1) +
                                    tabs.length) %
                                    tabs.length
                            ].id,
                            true,
                        );
                } else if (
                    modifier &&
                    event.shiftKey &&
                    event.key.toLowerCase() === "t"
                ) {
                    event.preventDefault();
                    const tab = closedTabs.pop();
                    if (tab) {
                        tabs.push(tab);
                        switchTab(tab.id, true);
                    }
                } else if (
                    modifier &&
                    event.shiftKey &&
                    event.key.toLowerCase() === "a" &&
                    !event.target.matches("input,textarea")
                ) {
                    event.preventDefault();
                    dispatch("select-none");
                }
            },
            true,
        );
        sync(visibleEntries());
    }
    setTimeout(install, 0);
    return {
        dispatch,
        sync,
        renderTree,
        entryNode,
        directoryPath,
        openSample,
        enterWelcome,
        switchTab,
        closeTab,
        moveTab,
        commands,
        resetJobs,
        menuState: () => ({
            tabs: tabs.map((tab) => ({ id: tab.id, title: tab.title })),
            activeTab: state.activeTab,
            view: state.view,
            selected: [...state.selected],
            path: state.path,
            expanded: [...state.expanded],
        }),
    };
})();
render();
