(function initializeReview() {
    "use strict";
    const frame = document.querySelector("#design-frame");
    const displaySpec = window.XDZIP_DISPLAY_SPEC;
    const query = new URLSearchParams(window.location.search);
    const preset = window.XDZIP_REVIEW_PRESET || {
        presentation:query.get("presentation") === "1", view:query.get("view"),
        platform:query.get("platform"), theme:query.get("theme"), zoom:query.get("zoom"),
    };
    const viewport = document.querySelector("#viewport");
    const referenceViewport = document.querySelector("#reference-viewport");
    const current = id => document.querySelector(`#${id}`).value;
    const number = value => Number(value.toFixed(2));
    const isMainView = () => ["main","welcome","welcome-narrow","empty","damaged"].includes(current("view"));
    const logicalScale = () => displaySpec.views[current("view")].sourceToLogical ?? (isMainView() ? displaySpec.sourceToLogical : displaySpec.operationSourceToLogical);

    // Source-image pixels are measurement coordinates. Normal presentation uses
    // explicit CSS logical dimensions; devicePixelRatio never changes this ratio.
    function sourceScale() {
        const [width] = displaySpec.views[current("view")].size;
        if (current("zoom") === "pixels") return 1;
        if (current("zoom") === "fit") return Math.min(logicalScale(), viewport.clientWidth / width);
        return logicalScale();
    }

    function fit() {
        const [width,height] = displaySpec.views[current("view")].size;
        const scale = sourceScale();
        const image = document.querySelector("#reference-image");
        for (const surface of [frame,image]) {
            surface.style.width = `${width}px`;
            surface.style.height = `${height}px`;
            surface.style.transform = `scale(${scale})`;
        }
        for (const wrapper of [document.querySelector("#scaled"),document.querySelector("#reference-scaled")]) {
            wrapper.style.width = `${width * scale}px`;
            wrapper.style.height = `${height * scale}px`;
        }
        const windowView = isMainView();
        const [, ,windowWidth,windowHeight] = displaySpec.views[current("view")].window || displaySpec.sourceWindow;
        const dimensions = windowView ? [windowWidth,windowHeight] : [width,height];
        const logical = dimensions.map(value => number(value * logicalScale()));
        const mode = current("zoom") === "pixels" ? "原稿像素 1:1" : current("zoom") === "fit" ? "适应屏幕" : "100% 设计尺寸";
        const theme = current("theme") === "dark" && !displaySpec.views[current("view")].darkSource ? "深色适配（无对应完整主稿）" : current("theme") === "dark" ? "深色" : "浅色";
        document.querySelector("#status").textContent = `${current("platform")}，${theme}，设计${windowView ? "窗口" : "图板"} ${logical.join(" × ")} CSS px，${mode}，当前 ${number(scale / logicalScale() * 100)}% 审阅倍率，DPR ${window.devicePixelRatio}（${number(scale * 100)}% 原稿倍率）`;
    }

    function updateReference() {
        const view = displaySpec.views[current("view")];
        const source = current("theme") === "dark" && current("view") === "welcome-narrow" ? null : current("theme") === "dark" && view.darkSource ? view.darkSource : view.source;
        const image = document.querySelector("#reference-image");
        image.hidden = !source;
        document.querySelector("#reference-message").textContent = source
            ? current("view") === "main" ? "当前设计图与 HTML 采用相同画布和倍率，横向滚动同步；独立量测使用原图。" : "原稿与 HTML 独立展示，采用相同画布尺寸与缩放倍率；横向滚动同步。"
            : "此状态暂无独立的已确认原稿。";
        if (source) image.src = window.XDZIP_REFERENCES?.[source] || source;
        else image.removeAttribute("src");
    }

    function update() {
        // This also configures canvases opened from opaque local-file origins.
        frame.contentWindow?.postMessage({
            type:"xdzip-review-configure", view:current("view"),
            platform:current("platform"), theme:current("theme"),
        }, "*");
        updateReference();
        fit();
    }

    function openIndependent() {
        const [width,height] = displaySpec.views[current("view")].size;
        const scale = sourceScale();
        const configurationValues = {
            presentation:true, view:current("view"), platform:current("platform"),
            theme:current("theme"), zoom:current("zoom"),
        };
        let target = "about:blank";
        if (!window.XDZIP_CANVAS) {
            const url = new URL("index.html", document.baseURI);
            for (const [key,value] of Object.entries(configurationValues)) url.searchParams.set(key, key === "presentation" ? "1" : value);
            target = url.href;
        }
        const popup = window.open(target, "xdzip-design-window", `popup,width=${Math.ceil(width * scale)},height=${Math.ceil(height * scale)}`);
        if (!popup) {
            document.querySelector("#status").textContent = "浏览器拦截了独立预览窗口，可允许此次弹窗后重试。";
            return;
        }
        if (!window.XDZIP_CANVAS) return;
        // Reuse the full reviewer so served and offline windows share fit(), assets
        // and configuration. Only review controls are hidden in presentation mode.
        const page = document.documentElement.cloneNode(true);
        page.querySelector("#design-frame").removeAttribute("srcdoc");
        page.querySelector("#reference-image").removeAttribute("src");
        const base = document.createElement("base");
        base.href = document.baseURI;
        page.querySelector("head").prepend(base);
        const configuration = document.createElement("script");
        configuration.textContent = "window.XDZIP_REVIEW_PRESET=" + JSON.stringify(configurationValues).replace(/</g, "\\u003c") + ";";
        page.querySelector("head").prepend(configuration);
        popup.document.open();
        popup.document.write("<!doctype html>" + page.outerHTML);
        popup.document.close();
    }

    for (const id of ["view","platform","theme","zoom"]) {
        const control = document.querySelector(`#${id}`);
        const value = preset[id] || (id === "zoom" ? (window.innerWidth < 650 ? "fit" : "1") : control.value);
        if ([...control.options].some(option => option.value === value)) control.value = value;
    }
    if (preset.presentation) {
        document.documentElement.classList.add("presentation");
        document.body.classList.add("presentation");
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
    for (const id of ["view","platform","theme"]) document.querySelector(`#${id}`).addEventListener("change", update);
    document.querySelector("#zoom").addEventListener("change", fit);
    window.addEventListener("resize", fit);
    // Classic page scrollbars can change available width without a window resize,
    // for example when switching artboards or showing the reference below them.
    let observedWidth;
    new ResizeObserver(([entry]) => {
        if (entry.contentRect.width === observedWidth) return;
        observedWidth = entry.contentRect.width;
        fit();
    }).observe(viewport);
    document.querySelector("#reference-toggle").onclick = event => {
        const reference = document.querySelector("#reference");
        reference.hidden = !reference.hidden;
        if (!reference.hidden) {
            updateReference();
            referenceViewport.scrollLeft = viewport.scrollLeft;
        }
        event.currentTarget.setAttribute("aria-expanded", String(!reference.hidden));
    };
    document.querySelector("#separate").onclick = openIndependent;
    // Ignore the event produced by our own mirrored position. Without this,
    // a delayed event can overwrite a newer scroll on the opposite panel.
    const mirroredPositions = new WeakMap();
    for (const [source,target] of [[viewport,referenceViewport],[referenceViewport,viewport]]) {
        source.addEventListener("scroll", () => {
            if (document.querySelector("#reference").hidden || preset.presentation) return;
            const mirrored = mirroredPositions.get(source);
            mirroredPositions.delete(source);
            if (mirrored !== undefined && Math.abs(source.scrollLeft - mirrored) <= .5) return;
            if (Math.abs(target.scrollLeft - source.scrollLeft) > .5) {
                target.scrollLeft = source.scrollLeft;
                mirroredPositions.set(target, target.scrollLeft);
            }
        });
    }
    window.addEventListener("message", event => {
        if (event.source !== frame.contentWindow || event.data?.type !== "xdzip-review-window") return;
        const view = event.data.view;
        if (!displaySpec.views[view]) return;
        document.querySelector("#view").value = view;
        // Resize the active window without replaying a preset or resetting its task.
        updateReference();
        fit();
    });
    update();
})();
