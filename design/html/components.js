"use strict";
// This catalog runs the same controls as the main-window artboard.
(() => {
    const ui = window.XdzipControls;
    const icon = (name) =>
        window.XDZIP_ASSETS.icons[document.documentElement.dataset.theme][
            name
        ] || "";
    const notice = (text) => {
        document.querySelector("#component-notice").textContent = text;
        delete document.querySelector("#component-notice").dataset.error;
    };
    let sort = "name",
        inspector = true;
    ui.configure({ icon });
    document.querySelector("#component-open").onclick = () =>
        notice("已选择打开动作；请在主窗口中选择示例压缩包。");
    document.querySelector("#component-primary").onclick = () =>
        notice("示例主动作已触发，没有执行实际解压。");
    const confirmation = document.querySelector("#component-confirmation");
    document.querySelector("#component-remove").onclick = () => {
        notice("移除示例条目？确认前不会改变当前选择。");
        confirmation.hidden = false;
        document.querySelector("#component-cancel-remove").focus();
    };
    document.querySelector("#component-cancel-remove").onclick = () => {
        confirmation.hidden = true;
        notice("已取消移除，示例保留。");
        document.querySelector("#component-remove").focus();
    };
    document.querySelector("#component-confirm-remove").onclick = () => {
        document.querySelector("#component-context").hidden = true;
        document.querySelector("#component-cancel-remove").hidden = true;
        document.querySelector("#component-confirm-remove").hidden = true;
        document.querySelector("#component-restore").hidden = false;
        notice("示例条目已移除，可恢复。");
        document.querySelector("#component-restore").focus();
    };
    document.querySelector("#component-restore").onclick = () => {
        document.querySelector("#component-context").hidden = false;
        document.querySelector("#component-cancel-remove").hidden = false;
        document.querySelector("#component-confirm-remove").hidden = false;
        document.querySelector("#component-restore").hidden = true;
        confirmation.hidden = true;
        notice("示例条目已恢复。");
        document.querySelector("#component-remove").focus();
    };
    const sortItems = () =>
        [
            ["name", "名称"],
            ["size", "原始大小"],
            ["packed", "压缩大小"],
        ].map(([id, label]) => ({
            id: `catalog-sort-${id}`,
            label,
            type: "radio",
            checked: sort === id,
            run: () => {
                sort = id;
                notice(`当前排序：${label}`);
            },
        }));
    const viewItems = () => [
        {
            id: "catalog-preview",
            label: "显示文件预览",
            type: "checkbox",
            checked: inspector,
            run: () => {
                inspector = !inspector;
                document.querySelector("#component-inspector").checked =
                    inspector;
                notice(inspector ? "文件预览已显示" : "文件预览已隐藏");
            },
        },
        { id: "catalog-sort", label: "排序方式", children: sortItems() },
    ];
    const items = () => [
        {
            id: "catalog-properties",
            label: "文件属性",
            icon: "file-pdf",
            run: () => notice("已打开所选示例文件的属性"),
        },
        {
            id: "catalog-copy",
            label: "复制名称",
            icon: "file-text",
            shortcut: "⌘C",
            run: () => notice("示例动作：复制行程手册.pdf 的名称"),
        },
        { type: "separator" },
        {
            id: "catalog-extract",
            label: "解压所选…",
            icon: "external-open",
            disabled: true,
            reason: "Qt 运行时尚未提供写入能力",
        },
        {
            id: "catalog-test",
            label: "测试压缩包完整性…",
            icon: "archive-test",
            disabled: true,
            reason: "真实 CRC 校验尚未接入",
        },
        { type: "separator" },
        { id: "catalog-view", label: "视图", children: viewItems() },
    ];
    const open = (trigger, point) =>
        ui.open({ trigger, point, label: "文件操作", items: items() });
    const more = document.querySelector("#component-more");
    more.onclick = () => open(more);
    const context = document.querySelector("#component-context");
    context.oncontextmenu = (event) => {
        event.preventDefault();
        open(context, { x: event.clientX, y: event.clientY });
    };
    context.onkeydown = (event) => {
        if (
            event.key === "ContextMenu" ||
            (event.key === "F10" && event.shiftKey)
        ) {
            event.preventDefault();
            open(context);
        }
    };
    const scope = document.querySelector("#component-scope");
    scope.onclick = () =>
        ui.open({
            trigger: scope,
            label: "解压范围",
            items: [
                {
                    id: "catalog-all",
                    label: "全部解压…",
                    run: () => notice("示例范围：全部内容"),
                },
                {
                    id: "catalog-selected",
                    label: "解压所选…",
                    run: () => notice("示例范围：当前选择"),
                },
                { type: "separator" },
                {
                    id: "catalog-results",
                    label: "解压搜索结果…",
                    disabled: true,
                    reason: "当前没有搜索结果",
                },
            ],
        });
    document.querySelector("#component-extract").onclick = () =>
        notice("示例主动作：当前选择；没有执行实际解压");
    const sortButton = document.querySelector("#component-sort");
    sortButton.onclick = () =>
        ui.open({ trigger: sortButton, label: "排序方式", items: sortItems() });
    document.querySelector("#component-inspector").onchange = (event) => {
        inspector = event.target.checked;
        notice(inspector ? "文件预览已显示" : "文件预览已隐藏");
    };
    const edge = document.querySelector("#component-edge");
    edge.onclick = () => open(edge, { x: innerWidth - 4, y: innerHeight - 4 });
    document.querySelector("#component-error").onclick = () => {
        notice("读取失败。可重新打开压缩包；示例文件和选择保持不变。");
        document.querySelector("#component-notice").dataset.error = "true";
    };
    function hydrate() {
        document.querySelectorAll("[data-icon]").forEach((node) => {
            const text = node.dataset.label ?? node.textContent;
            node.dataset.label = text;
            node.innerHTML = icon(node.dataset.icon);
            node.append(document.createTextNode(text));
        });
    }
    document.querySelector("#component-theme").onclick = (event) => {
        ui.close({ restoreFocus: false });
        ui.hideTooltip();
        const dark = document.documentElement.dataset.theme !== "dark";
        document.documentElement.dataset.theme = dark ? "dark" : "light";
        event.target.textContent = dark ? "浅色主题" : "深色主题";
        hydrate();
    };
    hydrate();
})();
