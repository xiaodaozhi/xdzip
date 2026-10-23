"use strict";
// Reusable input mechanics. Commands and archive state belong to the host.
// Menus use live DOM controls, never baked screenshots or document-level overlays.
window.XdzipControls = (() => {
    const tokens = window.XDZIP_CONTROL_TOKENS;
    let renderIcon = () => "",
        active = null,
        hoverTimer = null;
    let tooltipTimer = null,
        tooltip = null,
        tooltipOwner = null,
        previousDescription = null,
        pointerPressed = false,
        serial = 0;
    const sourceScale = () =>
        parseFloat(
            getComputedStyle(document.documentElement).getPropertyValue(
                "--ui-source-factor",
            ),
        ) || 1;
    const available = (item) => !item.disabled && item.type !== "separator";
    const focusable = (level) =>
        [...level.element.querySelectorAll(":scope > .ui-menu-item")].filter(
            (node) => node.getAttribute("aria-disabled") !== "true",
        );
    function scrollSnapshot(trigger) {
        const snapshot = new Map();
        // A preceding input can queue scroll on another existing container,
        // including one whose overflow has already disappeared before opening.
        document.querySelectorAll("*").forEach((node) =>
            snapshot.set(node, [node.scrollLeft, node.scrollTop]),
        );
        for (
            let node = trigger;
            node instanceof Element;
            node = node.parentElement
        )
            snapshot.set(node, [node.scrollLeft, node.scrollTop]);
        const root = document.scrollingElement;
        if (root) snapshot.set(root, [root.scrollLeft, root.scrollTop]);
        return snapshot;
    }

    function restoreOrigin(origin) {
        const target = typeof origin === "function" ? origin() : origin;
        if (target?.isConnected) target.focus({ preventScroll: true });
    }
    function hideTooltip() {
        clearTimeout(tooltipTimer);
        tooltip?.remove();
        if (tooltipOwner) {
            if (previousDescription)
                tooltipOwner.setAttribute(
                    "aria-describedby",
                    previousDescription,
                );
            else tooltipOwner.removeAttribute("aria-describedby");
        }
        previousDescription = null;
        tooltip = tooltipOwner = null;
    }
    function close({ restoreFocus = true } = {}) {
        if (!active) return;
        const previous = active;
        active = null;
        clearTimeout(hoverTimer);
        clearTimeout(previous.typeTimer);
        previous.levels.forEach((level) => level.element.remove());
        if (previous.trigger?.isConnected)
            previous.trigger.setAttribute("aria-expanded", "false");
        if (restoreFocus) restoreOrigin(previous.origin);
        previous.onClose?.();
    }
    function position(element, anchor, point, submenu = false, placement = "bottom-start") {
        const inset = tokens.size.menuViewportInset * sourceScale();
        const gap = tokens.size.menuGap * sourceScale();
        const owner = element.closest('dialog[open]')?.getBoundingClientRect();
        const left = (owner?.left || 0) + inset;
        const top = (owner?.top || 0) + inset;
        const right = (owner?.right || innerWidth) - inset;
        const bottom = (owner?.bottom || innerHeight) - inset;
        element.style.maxHeight = `${Math.max(80, bottom - top)}px`;
        element.style.maxWidth = `${Math.max(120, right - left)}px`;
        const box = element.getBoundingClientRect();
        let x = point ? point.x : submenu ? anchor.right + gap : placement.endsWith("end") ? anchor.right - box.width : anchor.left;
        let y = point ? point.y : submenu ? anchor.top : placement.startsWith("top") ? anchor.top - box.height - gap : anchor.bottom + gap;
        if (!point && !submenu && placement.startsWith("top") && y < top)
            y = anchor.bottom + gap;
        if (x + box.width > right)
            x = submenu
                ? anchor.left - box.width - gap
                : right - box.width;
        if (y + box.height > bottom)
            y =
                point || submenu
                    ? bottom - box.height
                    : anchor.top - box.height - gap;
        element.style.left = `${Math.max(left, x)}px`;
        element.style.top = `${Math.max(top, y)}px`;
    }
    function truncate(levelIndex) {
        if (!active) return;
        active.levels
            .splice(levelIndex + 1)
            .forEach((level) => level.element.remove());
        active.levels[levelIndex].element
            .querySelectorAll('[aria-haspopup="menu"]')
            .forEach((node) => node.setAttribute("aria-expanded", "false"));
    }
    function submenu(levelIndex, node, item, keyboard) {
        if (!active || !available(item)) return;
        truncate(levelIndex);
        const level = build(
            item.children,
            item.label,
            active.levels[levelIndex].element.parentElement,
        );
        level.parentItem = node;
        active.levels.push(level);
        node.setAttribute("aria-expanded", "true");
        position(level.element, node.getBoundingClientRect(), null, true);
        if (keyboard) focusable(level)[0]?.focus();
    }
    function activate(levelIndex, node, item) {
        if (!active || !available(item)) return;
        if (item.children) return submenu(levelIndex, node, item, true);
        const callback = item.run;
        close();
        callback?.(); // The host rechecks capability and current archive identity.
    }
    function build(items, label, owner) {
        const element = document.createElement("div");
        element.className = "ui-menu";
        element.id = `xdzip-menu-${++serial}`;
        element.setAttribute("role", "menu");
        element.setAttribute("aria-label", label);
        const level = { element, items };
        items.forEach((item) => {
            if (item.type === "separator") {
                const separator = document.createElement("div");
                separator.className = "ui-menu-separator";
                separator.setAttribute("role", "separator");
                element.append(separator);
                return;
            }
            const node = document.createElement("button");
            node.type = "button";
            node.className = `ui-menu-item${item.danger ? " ui-menu-danger" : ""}`;
            node.tabIndex = -1;
            node.dataset.command = item.id || "";
            Object.entries(item.data || {}).forEach(([key, value]) => {
                node.dataset[key] = String(value);
            });
            node.setAttribute(
                "role",
                item.type === "radio"
                    ? "menuitemradio"
                    : item.type === "checkbox"
                      ? "menuitemcheckbox"
                      : "menuitem",
            );
            node.setAttribute("aria-disabled", String(Boolean(item.disabled)));
            if (item.type === "radio" || item.type === "checkbox")
                node.setAttribute(
                    "aria-checked",
                    String(Boolean(item.checked)),
                );
            if (item.children) {
                node.setAttribute("aria-haspopup", "menu");
                node.setAttribute("aria-expanded", "false");
            }
            if (item.disabled && item.reason) {
                node.setAttribute("aria-description", item.reason);
            }
            const symbol = document.createElement("span");
            symbol.className = "ui-menu-symbol";
            symbol.innerHTML = item.checked
                ? renderIcon("check")
                : item.icon
                  ? renderIcon(item.icon)
                  : "";
            const text = document.createElement("span");
            text.className = "ui-menu-label";
            text.textContent = item.label;
            const shortcut = document.createElement("span");
            shortcut.className = item.status ? "task-status" : "ui-menu-shortcut";
            shortcut.textContent = item.status || item.shortcut || "";
            shortcut.setAttribute("aria-hidden", "true");
            if (item.status) {
                shortcut.dataset.status = item.statusKind || "running";
                node.setAttribute("aria-label", item.label + "，" + item.status);
            }
            const arrow = document.createElement("span");
            arrow.className = "ui-menu-arrow";
            arrow.innerHTML = item.children ? renderIcon("chevron-right") : "";
            node.append(symbol, text, shortcut, arrow);
            node.addEventListener("pointerenter", () => {
                clearTimeout(hoverTimer);
                if (!active) return;
                const index = active.levels.indexOf(level);
                if (index < 0) return;
                if (available(item)) node.focus({ preventScroll: true });
                hoverTimer = setTimeout(
                    () =>
                        item.children
                            ? submenu(index, node, item, false)
                            : truncate(index),
                    tokens.motion.submenuDelayMs,
                );
            });
            node.addEventListener("click", (event) => {
                event.stopPropagation();
                if (active) activate(active.levels.indexOf(level), node, item);
            });
            element.append(node);
        });
        element.addEventListener("pointerenter", () =>
            clearTimeout(hoverTimer),
        );
        element.addEventListener("keydown", (event) => {
            if (!active || event.isComposing) return;
            const index = active.levels.indexOf(level);
            const nodes = focusable(level),
                current = nodes.indexOf(document.activeElement);
            const selected = level.items.find(
                (item) =>
                    item.id &&
                    item.id === document.activeElement?.dataset.command,
            );
            if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
                event.preventDefault();
                const next =
                    event.key === "Home"
                        ? 0
                        : event.key === "End"
                          ? nodes.length - 1
                          : (current +
                                (event.key === "ArrowDown" ? 1 : -1) +
                                nodes.length) %
                            nodes.length;
                nodes[next]?.focus();
            } else if (event.key === "ArrowRight" && selected?.children) {
                event.preventDefault();
                submenu(index, document.activeElement, selected, true);
            } else if (event.key === "ArrowLeft" || event.key === "Escape") {
                event.preventDefault();
                if (index > 0) {
                    const parent = level.parentItem;
                    truncate(index - 1);
                    parent.focus();
                } else close();
            } else if (["Enter", " "].includes(event.key)) {
                event.preventDefault();
                if (selected) activate(index, document.activeElement, selected);
            } else if (event.key === "Tab") {
                close();
            } else if (
                event.key.length === 1 &&
                !event.ctrlKey &&
                !event.metaKey &&
                !event.altKey
            ) {
                event.preventDefault();
                clearTimeout(active.typeTimer);
                active.typeahead += event.key.toLocaleLowerCase();
                const sequence = [
                    ...nodes.slice(current + 1),
                    ...nodes.slice(0, current + 1),
                ];
                sequence
                    .find((node) =>
                        node
                            .querySelector(".ui-menu-label")
                            .textContent.toLocaleLowerCase()
                            .startsWith(active.typeahead),
                    )
                    ?.focus();
                active.typeTimer = setTimeout(() => {
                    if (active) active.typeahead = "";
                }, tokens.motion.typeaheadResetMs);
            }
            // Menu input must not activate global shortcuts or the file row beneath it.
            event.stopPropagation();
        });
        owner.append(element);
        // A modal's scroll container must not clip its choices or scroll when a
        // distant option receives focus. Keep DOM ownership and use the top layer.
        if (owner.closest?.('dialog[open]')) {
            element.popover = "manual";
            element.style.inset = "auto";
            element.style.margin = "0";
            element.showPopover();
        }
        return level;
    }
    function open({
        trigger,
        items,
        label,
        point,
        origin = trigger,
        placement = "bottom-start",
        owner = document.body,
        id,
        onClose,
        suppressTouchRelease = false,
    }) {
        if (active?.trigger === trigger && !point) return close();
        close({ restoreFocus: false });
        hideTooltip();
        if (!items.length) return;
        active = {
            trigger,
            origin,
            levels: [],
            typeahead: "",
            onClose,
            scrollPositions: scrollSnapshot(trigger),
            touchReleaseUntil: suppressTouchRelease ? Date.now() + 700 : 0,
        };
        const level = build(items, label, owner);
        if (id) level.element.id = id;
        active.levels.push(level);
        if (trigger?.isConnected) {
            trigger.setAttribute("aria-haspopup", "menu");
            trigger.setAttribute("aria-expanded", "true");
            trigger.setAttribute("aria-controls", level.element.id);
        }
        position(level.element, trigger?.getBoundingClientRect(), point, false, placement);
        focusable(level)[0]?.focus({ preventScroll: true });
    }
    function queueTooltip(target) {
        hideTooltip();
        if (pointerPressed || !target?.dataset.tooltip || (active && !target.closest(".ui-menu")))
            return;
        tooltipTimer = setTimeout(() => {
            if (!target.isConnected || pointerPressed) return;
            tooltipOwner = target;
            previousDescription = target.getAttribute("aria-describedby");
            tooltip = document.createElement("div");
            tooltip.className = "ui-tooltip";
            tooltip.id = `xdzip-tooltip-${++serial}`;
            tooltip.setAttribute("role", "tooltip");
            tooltip.textContent = target.dataset.tooltip;
            (target.closest("dialog[open]") || document.body).append(tooltip);
            target.setAttribute(
                "aria-describedby",
                [previousDescription, tooltip.id].filter(Boolean).join(" "),
            );
            position(tooltip, target.getBoundingClientRect(), null, false, "top-start");
            tooltipTimer = setTimeout(
                hideTooltip,
                tokens.motion.tooltipDurationMs,
            );
        }, tokens.motion.hoverDelayMs);
    }
    const focusedControl = () => {
        const node = document.activeElement;
        return node instanceof HTMLElement && node !== document.body &&
            node.matches('button, input, select, textarea, a[href], [tabindex], [contenteditable="true"]') ? node : null;
    };
    document.addEventListener("keydown", event => {
        if (event.key !== "Escape" || event.isComposing || event.repeat ||
            event.altKey || event.ctrlKey || event.metaKey) return;
        const node = focusedControl();
        // An all-disabled menu has no focusable item; Escape still closes it.
        if (active) {
            if (!node?.closest(".ui-menu")) {
                event.preventDefault();
                event.stopImmediatePropagation();
                close();
            }
            return;
        }
        if (node && !node.closest(".ui-menu")) {
            event.preventDefault();
            event.stopImmediatePropagation();
            node.blur();
        }
    }, true);
    document.addEventListener("keyup", event => {
        if (["Enter", "Escape"].includes(event.key) && !event.isComposing && !event.altKey && !event.ctrlKey && !event.metaKey)
            setTimeout(() => { const node = focusedControl(); if (node && !active && !node.closest(".ui-menu")) node.blur(); }, 0);
    });
    document.addEventListener(
        "pointerdown",
        (event) => {
            pointerPressed = true;
            const focused = focusedControl();
            const group = focused?.closest(".ui-select, .search-field, .destination-field") || focused;
            if (focused && !focused.closest(".ui-menu") && !group.contains(event.target)) focused.blur();
            if (
                active &&
                !event.target.closest(".ui-menu") &&
                !active.trigger?.contains(event.target)
            )
                close({ restoreFocus: false });
            hideTooltip();
        },
        true,
    );
    document.addEventListener(
        "click",
        (event) => {
            if (
                active?.touchReleaseUntil > Date.now() &&
                (event.pointerType === "touch" ||
                    event.sourceCapabilities?.firesTouchEvents)
            ) {
                event.preventDefault();
                event.stopImmediatePropagation();
                active.touchReleaseUntil = 0;
            }
        },
        true,
    );
    document.addEventListener("pointerover", (event) =>
        queueTooltip(event.target.closest("[data-tooltip]")),
    );
    document.addEventListener("pointerup", () => { pointerPressed = false; }, true);
    document.addEventListener("pointercancel", () => { pointerPressed = false; hideTooltip(); }, true);
    window.addEventListener("blur", () => { pointerPressed = false; hideTooltip(); });
    document.addEventListener("pointermove", (event) => {
        pointerPressed = Boolean(event.buttons);
        if (!event.buttons) queueTooltip(event.target.closest("[data-tooltip]"));
        else hideTooltip();
    });
    document.addEventListener("pointerout", (event) => {
        if (event.target.closest("[data-tooltip]") !== event.relatedTarget?.closest?.("[data-tooltip]")) hideTooltip();
    });
    document.addEventListener("focusin", (event) => {
        if (!event.target.closest(".ui-menu"))
            queueTooltip(event.target.closest("[data-tooltip]"));
    });
    document.addEventListener("focusout", hideTooltip);
    document.addEventListener(
        "scroll",
        (event) => {
            if (event.target.closest?.(".ui-menu")) return;
            const node =
                event.target === document
                    ? document.scrollingElement
                    : event.target;
            const initial = active?.scrollPositions.get(node);
            // A scroll dispatched after opening may belong to the preceding click's
            // scroll-into-view. Close only when its position actually changed.
            if (
                !initial ||
                initial[0] !== node.scrollLeft ||
                initial[1] !== node.scrollTop
            )
                close({ restoreFocus: false });
            hideTooltip();
        },
        true,
    );
    window.addEventListener("resize", () => {
        close({ restoreFocus: false });
        hideTooltip();
    });
    function enhanceSelects(owner) {
        owner.querySelectorAll('select[data-xdzip-select]').forEach((select) => {
            if (select.dataset.enhanced) return;
            select.dataset.enhanced = "true";
            const wrapper = document.createElement("div");
            wrapper.className = "ui-select";
            select.before(wrapper);
            wrapper.append(select);
            select.hidden = true;
            const trigger = document.createElement("button");
            trigger.type = "button";
            trigger.id = select.id + "-trigger";
            trigger.className = "field-select ui-select-trigger";
            trigger.setAttribute("role", "combobox");
            trigger.setAttribute("aria-haspopup", "listbox");
            trigger.setAttribute("aria-expanded", "false");
            const label = owner.querySelector(`label[for="${select.id}"]`);
            if (label) {
                label.htmlFor = trigger.id;
                trigger.setAttribute("aria-label", label.textContent);
            }
            wrapper.append(trigger);
            let editor = null;
            if (select.dataset.editableValue) {
                editor = document.createElement("input");
                editor.id = select.id + "-input";
                editor.className = "ui-select-editor";
                editor.inputMode = "numeric";
                editor.maxLength = 20;
                editor.placeholder = "输入数字";
                editor.setAttribute("role", "combobox");
                editor.setAttribute("aria-autocomplete", "none");
                editor.setAttribute("aria-label", label?.textContent || "自定义数值");
                wrapper.append(editor);
                select._xdzipEditor = editor;
                editor.oninput = () => select.dispatchEvent(new Event("input", {bubbles:true}));
                editor.onkeydown = event => {
                    if (["ArrowDown", "ArrowUp"].includes(event.key)) {
                        event.preventDefault(); expand();
                    }
                };
            }
            function refresh() {
                const text = document.createElement("span");
                text.textContent = select.selectedOptions[0]?.textContent || "";
                trigger.replaceChildren(text);
                trigger.insertAdjacentHTML("beforeend", renderIcon("chevron-down"));
                trigger.disabled = select.disabled;
                if (editor) {
                    const editable = select.value === select.dataset.editableValue;
                    editor.hidden = !editable;
                    editor.disabled = select.disabled;
                    wrapper.classList.toggle("editable", editable);
                    trigger.querySelector("span").style.visibility = editable ? "hidden" : "";
                }
            }
            function expand() {
                const menuId = trigger.id + "-options";
                trigger.setAttribute("aria-controls", menuId);
                open({trigger, owner, id:menuId, label:label?.textContent || "选择选项",
                    items:[...select.options].map(option => ({
                        id:select.id + "-" + option.value, label:option.textContent, type:"radio", checked:option.selected,
                        disabled:option.disabled,
                        run:() => {
                            select.value = option.value;
                            select.dispatchEvent(new Event("change", {bubbles:true}));
                        },
                    }))});
                const menu = owner.querySelector(".ui-menu");
                if (!menu) return;
                trigger.setAttribute("aria-haspopup", "listbox");
                menu.setAttribute("role", "listbox");
                menu.classList.add("ui-select-options");
                const anchor = trigger.getBoundingClientRect();
                const font = getComputedStyle(trigger);
                menu.style.font = font.font;
                const optionScale = Number(document.documentElement.style.getPropertyValue("--settings-text-scale")) || 1;
                for (const [property,key] of [["icon-size","menuIconSize"],["label-inset","menuLabelInset"],["padding","menuPadding"],["row-height","menuRowHeight"]])
                    menu.style.setProperty(`--ui-menu-${property}`, `${tokens.size[key] * sourceScale() * optionScale}px`);
                const canvas = document.createElement("canvas");
                const metrics = canvas.getContext("2d");
                metrics.font = font.font;
                const widest = Math.max(...[...select.options].map(option => metrics.measureText(option.textContent).width));
                const cap = Math.min(tokens.size.selectPopupMaxWidth * sourceScale(), innerWidth - tokens.size.menuViewportInset * sourceScale() * 2);
                const padding = (tokens.size.menuIconSize + tokens.size.menuLabelInset * 3 + tokens.size.menuPadding * 2) * sourceScale() * optionScale;
                const border = parseFloat(getComputedStyle(menu).borderLeftWidth) + parseFloat(getComputedStyle(menu).borderRightWidth);
                menu.style.width = `${Math.max(anchor.width, Math.min(Math.ceil(widest + padding + border), cap))}px`;
                position(menu, anchor, null, false, "bottom-end");
                const options = menu.querySelectorAll(".ui-menu-item");
                options.forEach((option, index) => {
                    option.setAttribute("role", "option");
                    option.setAttribute("aria-selected", String(select.options[index].selected));
                    option.removeAttribute("aria-checked");
                });
                options[select.selectedIndex]?.focus({preventScroll:true});
            }
            trigger.onclick = expand;
            trigger.onkeydown = event => {
                if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
                    event.preventDefault(); expand();
                }
            };
            select._xdzipRefresh = refresh;
            select.addEventListener("change", () => {
                // Host change handlers may update dependent format/method controls.
                queueMicrotask(() => owner.querySelectorAll("select[data-xdzip-select]").forEach(item => item._xdzipRefresh?.()));
            });
            refresh();
        });
    }
    function segmentedMarkup({id,label,options,selected}) {
        const escape = value => String(value).replace(/[&<>"']/g, c=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"})[c]);
        return `<div id="${escape(id)}" class="operation-segmented" role="radiogroup" aria-label="${escape(label)}">${options.map(option=>`<button type="button" id="${escape(option.id)}" class="operation-segment" data-value="${escape(option.value)}" role="radio" aria-checked="${option.value===selected}" tabindex="${option.value===selected?0:-1}" ${option.disabled?'disabled':''}>${escape(option.text)}</button>`).join("")}</div>`;
    }
    function enhanceSegmentedGroups(owner) {
        owner.querySelectorAll('.operation-segmented').forEach(group=>{
            if (group.dataset.enhanced) return;
            group.dataset.enhanced = "true";
            const all = [...group.querySelectorAll('[role="radio"]')];
            function choose(button) {
                if (button.disabled) return;
                all.forEach(item=>{item.setAttribute('aria-checked',String(item===button));item.tabIndex=item===button?0:-1;});
                group.dispatchEvent(new Event('change',{bubbles:true}));
            }
            all.forEach(button=>{
                button.addEventListener('click',()=>choose(button));
                button.addEventListener('keydown',event=>{
                    if (!['ArrowLeft','ArrowRight','Home','End'].includes(event.key)) return;
                    event.preventDefault();
                    const available = all.filter(item=>!item.disabled);
                    const current = available.indexOf(button);
                    const index = event.key==='Home'?0:event.key==='End'?available.length-1:(current+(event.key==='ArrowRight'?1:-1)+available.length)%available.length;
                    const next = available[index];
                    if (next) {choose(next);next.focus();}
                });
            });
        });
    }
    return {
        open,
        close,
        enhanceSelects,
        segmentedMarkup,
        enhanceSegmentedGroups,
        hideTooltip,
        configure(options) {
            renderIcon = options.icon;
        },
        get isOpen() {
            return Boolean(active);
        },
    };
})();
