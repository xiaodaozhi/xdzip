// design/html/file-columns.js

"use strict";
// Demo-only layout state. Widths last for this page's lifetime and never touch files.
window.XdzipFileColumns = (() => {
    const widths = [0, 0, 0, 0, 0];
    let apply = () => {};
    function install() {
        const table = document.querySelector(".table-scroll table");
        if (!table || table.dataset.resizable) return;
        table.dataset.resizable = "true";
        const viewport = table.closest(".table-scroll");
        const headers = [...table.querySelectorAll("thead th")];
        const factor = () => parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--ui-source-factor"));
        const textScale = () => Math.max(1, Number(document.documentElement.style.getPropertyValue("--settings-text-scale")) || 1);
        const defaults = headers.map(header => header.getBoundingClientRect().width / factor());
        const group = document.createElement("colgroup");
        const columns = headers.map(() => group.appendChild(document.createElement("col")));
        apply = () => {
            if (!widths.some(Boolean)) {
                group.remove(); table.style.removeProperty("width"); table.style.removeProperty("table-layout");
                return;
            }
            if (!group.isConnected) table.prepend(group);
            const scale = textScale(), source = factor();
            const sizes = widths.map((width, index) => width ? width * scale : defaults[index + 1] * scale);
            sizes[0] = widths[0] ? widths[0] * scale - defaults[0] : Math.max(160 * scale - defaults[0], viewport.clientWidth / source - defaults[0] - sizes.slice(1).reduce((sum, width) => sum + width, 0));
            columns[0].style.width = defaults[0] * source + "px";
            sizes.forEach((width, index) => { columns[index + 1].style.width = width * source + "px"; });
            table.style.tableLayout = "fixed";
            table.style.width = (defaults[0] + sizes.reduce((sum, width) => sum + width, 0)) * source + "px";
        };
        headers.slice(1).forEach((header, column) => {
            const handle = document.createElement("span");
            handle.setAttribute("role", "separator");
            handle.setAttribute("aria-orientation", "vertical");
            handle.tabIndex = 0;
            handle.className = "column-resize-handle";
            handle.dataset.column = String(column);
            handle.setAttribute("aria-label", `调整${header.textContent.trim()}列宽，双击恢复默认`);
            let dragging = null;
            handle.onpointerdown = event => {
                event.preventDefault(); event.stopPropagation();
                dragging = {x: event.clientX, width: header.getBoundingClientRect().width + (column === 0 ? defaults[0] * factor() : 0)};
                handle.setPointerCapture(event.pointerId);
            };
            handle.onpointermove = event => {
                if (!dragging) return;
                widths[column] = Math.max((column === 0 ? 160 : 64) * textScale(), (dragging.width + event.clientX - dragging.x) / factor()) / textScale();
                apply();
            };
            handle.onpointerup = handle.onpointercancel = () => { dragging = null; };
            handle.onclick = event => event.stopPropagation();
            handle.ondblclick = event => { event.stopPropagation(); widths[column] = 0; apply(); };
            handle.onkeydown = event => {
                if (!["ArrowLeft", "ArrowRight", "Home"].includes(event.key)) return;
                event.preventDefault(); event.stopPropagation();
                const current = header.getBoundingClientRect().width / factor() + (column === 0 ? defaults[0] : 0);
                widths[column] = event.key === "Home" ? 0 : Math.max((column === 0 ? 160 : 64) * textScale(), current + (event.key === "ArrowRight" ? 8 : -8)) / textScale();
                apply();
            };
            header.appendChild(handle);
        });
        new ResizeObserver(apply).observe(viewport);
        new MutationObserver(apply).observe(document.documentElement, {attributes: true, attributeFilter: ["style"]});
    }
    document.fonts.ready.then(install);
    return {widths, refresh: () => apply()};
})();
