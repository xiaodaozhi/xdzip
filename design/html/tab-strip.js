// design/html/tab-strip.js

"use strict";
// Presentation and input only. The design session adapter owns tab identities.
window.XdzipTabStrip = (() => {
    const attached = new WeakMap();
    let suppressClick = false;
    document.addEventListener("pointerdown", () => { suppressClick = false; }, true);
    document.addEventListener("click", event => {
        if (!suppressClick) return;
        event.preventDefault(); event.stopImmediatePropagation(); suppressClick = false;
    }, true);
    function attach(bar, options) {
        attached.get(bar)?.();
        const viewport = bar.querySelector(".tab-viewport");
        const previous = bar.querySelector("#tabs-previous");
        const next = bar.querySelector("#tabs-next");
        const list = bar.querySelector("#tabs-list");
        const tokens = window.XDZIP_TAB_TOKENS;
        const factor = parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--ui-source-factor"));
        const gap = tokens.tab_gap * factor;
        let drag = null;
        function revealActive() {
            const active = viewport.querySelector('[aria-selected="true"]');
            if (!active || drag) return;
            const left = active.offsetLeft - viewport.offsetLeft;
            if (left < viewport.scrollLeft) viewport.scrollLeft = left;
            else if (left + active.offsetWidth > viewport.scrollLeft + viewport.clientWidth)
                viewport.scrollLeft = left + active.offsetWidth - viewport.clientWidth;
        }
        function arrows() {
            previous.disabled = viewport.scrollLeft <= 0.5;
            next.disabled = viewport.scrollLeft + viewport.clientWidth >= viewport.scrollWidth - 0.5;
        }
        function layout() {
            cancelDrag();
            const count = viewport.children.length;
            const scale = Math.min(1.5, Number(document.documentElement.style.getPropertyValue("--settings-text-scale")) || 1);
            const preferred = tokens.tab_preferred_width * factor * scale;
            const minimum = tokens.tab_minimum_width * factor * scale;
            const base = Math.max(0, bar.clientWidth - bar.querySelector("#new-tab").offsetWidth - gap);
            const overflow = count * minimum + (count - 1) * gap > base;
            for (const button of [previous, next, list]) button.hidden = !overflow;
            const capacity = Math.max(0, bar.clientWidth - bar.querySelector(".tab-strip-actions").offsetWidth - gap);
            const width = Math.max(minimum, Math.min(preferred, (capacity - (count - 1) * gap) / count));
            bar.style.setProperty("--tab-width", width + "px");
            viewport.style.width = Math.min(capacity, count * width + (count - 1) * gap) + "px";
            revealActive(); arrows();
        }
        const observer = new ResizeObserver(layout);
        observer.observe(bar);
        const scaleObserver = new MutationObserver(layout);
        scaleObserver.observe(document.documentElement, {attributes: true, attributeFilter: ["style"]});
        viewport.onscroll = arrows;
        viewport.onwheel = event => {
            if (viewport.scrollWidth <= viewport.clientWidth) return;
            if (Math.abs(event.deltaY) > Math.abs(event.deltaX)) {
                viewport.scrollLeft += event.deltaY; event.preventDefault();
            }
        };
        const step = () => parseFloat(bar.style.getPropertyValue("--tab-width")) + gap;
        previous.onclick = () => viewport.scrollBy({left: -step(), behavior: "smooth"});
        next.onclick = () => viewport.scrollBy({left: step(), behavior: "smooth"});
        list.onclick = () => {
            // Finish a preceding arrow scroll before the shared menu snapshots
            // scroll positions; a late scroll event would dismiss the new menu.
            viewport.scrollTo({left: viewport.scrollLeft, behavior: "instant"});
            requestAnimationFrame(() => { if (list.isConnected) options.list(list); });
        };
        function cancelDrag() {
            if (!drag) return;
            if (drag.ghost) suppressClick = true;
            clearInterval(drag.timer);
            drag.node.classList.remove("drag-origin"); drag.ghost?.remove(); drag.marker?.remove(); drag = null;
        }
        function updateDrag(event) {
            if (!drag) return;
            if (event) { drag.x = event.clientX; drag.y = event.clientY; }
            if (!drag.ghost && Math.abs(drag.x - drag.start) < 8 * factor) return;
            if (!drag.ghost) {
                uiClose();
                const ghost = drag.node.cloneNode(true);
                ghost.removeAttribute("id"); ghost.removeAttribute("data-tab");
                ghost.querySelectorAll("[id]").forEach(node => node.removeAttribute("id"));
                ghost.classList.add("tab-drag-preview"); ghost.setAttribute("aria-hidden", "true");
                ghost.style.width = drag.width + "px";
                ghost.style.height = drag.height + "px";
                ghost.style.borderRadius = drag.height / 2 + "px";
                const marker = document.createElement("div"); marker.className = "tab-drop-indicator";
                document.body.append(ghost, marker);
                drag.ghost = ghost; drag.marker = marker; drag.node.classList.add("drag-origin");
                drag.timer = setInterval(() => {
                    if (!drag) return;
                    const rect = viewport.getBoundingClientRect(), edge = tokens.tab_drag_edge * factor;
                    if (drag.x < rect.left + edge) viewport.scrollLeft -= tokens.tab_drag_scroll_step * factor;
                    else if (drag.x > rect.right - edge) viewport.scrollLeft += tokens.tab_drag_scroll_step * factor;
                    updateDrag();
                }, tokens.tab_drag_scroll_interval);
            }
            const rect = viewport.getBoundingClientRect();
            const center = drag.x - drag.offset - rect.left + viewport.scrollLeft + drag.width / 2;
            drag.to = Math.max(0, Math.min(options.tabs().length - 1, Math.floor(center / step())));
            drag.ghost.style.left = Math.max(rect.left, Math.min(rect.right - drag.width, drag.x - drag.offset)) + "px";
            drag.ghost.style.top = rect.top + "px";
            drag.marker.style.display = drag.from === drag.to ? "none" : "block";
            drag.marker.style.left = Math.max(rect.left, Math.min(rect.right - tokens.tab_drop_indicator_width * factor,
                rect.left + (drag.to + (drag.to > drag.from ? 1 : 0)) * step() - viewport.scrollLeft - gap / 2)) + "px";
            drag.marker.style.top = rect.top + "px"; drag.marker.style.height = rect.height + "px";
        }
        function uiClose() { window.XdzipControls.close(); window.XdzipControls.hideTooltip(); }
        function pointerUp(event) {
            if (!drag) return;
            const rect = bar.getBoundingClientRect();
            const {id, from, to, ghost} = drag;
            const inside = event.clientY >= rect.top && event.clientY <= rect.bottom && event.clientX >= rect.left && event.clientX <= rect.right;
            cancelDrag();
            if (!ghost) return;
            suppressClick = true;
            if (inside && from !== to) options.move(id, to);
        }
        viewport.querySelectorAll("[data-tab]").forEach(node => {
            node.onpointerdown = event => {
                if (event.button !== 0 || event.target.closest("[data-close-tab]") || options.tabs().length < 2) return;
                const rect = node.getBoundingClientRect();
                const from = options.tabs().findIndex(tab => tab.id === node.dataset.tab);
                drag = {node, id: node.dataset.tab, from, to: from, start: event.clientX,
                    x: event.clientX, y: event.clientY, offset: event.clientX - rect.left, width: rect.width, height: rect.height};
                node.setPointerCapture(event.pointerId);
            };
            node.onauxclick = event => { if (event.button === 1) { event.preventDefault(); options.close(node.dataset.tab); } };
        });
        function keyboard(event) {
            if (event.key === "Escape" && drag) {
                event.preventDefault(); event.stopImmediatePropagation();
                cancelDrag();
                return;
            }
            if (document.querySelector("dialog[open],#welcome-opening")) return;
            if (!event.ctrlKey || !event.shiftKey || !["PageUp", "PageDown"].includes(event.key)) return;
            const tabs = options.tabs(), index = tabs.findIndex(tab => tab.id === options.active());
            const to = index + (event.key === "PageUp" ? -1 : 1);
            if (index >= 0 && to >= 0 && to < tabs.length) { event.preventDefault(); options.move(tabs[index].id, to); }
        }
        document.addEventListener("pointermove", updateDrag);
        document.addEventListener("pointerup", pointerUp);
        document.addEventListener("pointercancel", cancelDrag);
        window.addEventListener("keydown", keyboard, true);
        window.addEventListener("blur", cancelDrag);
        attached.set(bar, () => {
            cancelDrag(); observer.disconnect(); scaleObserver.disconnect();
            document.removeEventListener("pointermove", updateDrag);
            document.removeEventListener("pointerup", pointerUp);
            document.removeEventListener("pointercancel", cancelDrag);
            window.removeEventListener("keydown", keyboard, true);
            window.removeEventListener("blur", cancelDrag);
        });
        requestAnimationFrame(layout);
    }
    return {attach};
})();
