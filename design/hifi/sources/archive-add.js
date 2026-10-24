// design/hifi/sources/archive-add.js

(() => {
    "use strict";
    const query = new URLSearchParams(location.search);
    const locale = query.get("lang") === "en-US" ? "en-US" : "zh-CN";
    const theme = query.get("theme") === "dark" ? "dark" : "light";
    const scale = [1, 1.25, 1.5].includes(Number(query.get("textScale"))) ? Number(query.get("textScale")) : 1;
    document.documentElement.lang = locale;
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.setProperty("--text-scale", scale);
    document.documentElement.style.setProperty("--settings-text-scale", scale);
    if (query.has("export")) document.body.classList.add("export");
    const catalog = window.XDZIP_UI_MESSAGES[locale].messages;
    function text(id, n) {
        const message = catalog[id];
        return (typeof message === "object" ? message[n === 1 ? "one" : "other"] : message).replace(/%n/g, String(n));
    }
    for (const element of document.querySelectorAll("[data-i18n]")) element.textContent = text(element.dataset.i18n);
    const icon = name => `../../assets/png/${theme}/${name}@3x.png`;
    document.querySelector("#hero-icon").src = icon("archive-add");
    document.querySelector("#archive-icon").src = icon("file-archive");
    document.querySelector("#directory-icon").src = icon("folder");
    document.querySelector("#add-files img").src = icon("file-text");
    document.querySelector("#add-folder img").src = icon("folder");
    document.querySelector("#archive-name").textContent = query.has("long") ? "北境旅行与跨系统协作资料_".repeat(5) + ".zip" : "北境旅行.zip";
    document.querySelector("#directory-name").textContent = query.has("long") ? "资料/".repeat(18) : "旅行资料/";
    let inputs = query.has("empty") ? [] : [
        {name: "冰川湖.jpg", size: 8400000, icon: "image-file"},
        {name: "山间日出.jpg", size: 6200000, icon: "image-file"},
        {name: "行程手册.pdf", size: 12800000, icon: "file-pdf"}
    ];
    const size = n => n >= 1000000 ? (n / 1000000).toFixed(1) + " MB" : n >= 1000 ? Math.round(n / 1000) + " KB" : n + " B";
    function render() {
        const list = document.querySelector("#input-list");
        list.replaceChildren();
        inputs.forEach((item, index) => {
            const row = document.createElement("div");
            row.className = "create-row";
            row.setAttribute("role", "listitem");
            const image = document.createElement("img");
            image.className = "file-icon"; image.src = icon(item.directory ? "folder" : item.icon || "file-text"); image.alt = "";
            const name = document.createElement("span"); name.className = "input-name"; name.textContent = item.name; name.title = item.name;
            const amount = document.createElement("span"); amount.className = "input-size"; amount.textContent = item.directory ? "" : size(item.size);
            const remove = document.createElement("button"); remove.type = "button"; remove.className = "icon-button remove-input";
            remove.setAttribute("aria-label", text("ui.2ec7e443b1c7").replace("%1", item.name));
            const glyph = document.createElement("img"); glyph.src = icon("remove"); glyph.alt = ""; remove.append(glyph);
            remove.onclick = () => { inputs.splice(index, 1); render(); };
            row.append(image, name, amount, remove); list.append(row);
        });
        const files = inputs.filter(item => !item.directory), directories = inputs.length - files.length, parts = [];
        if (files.length) parts.push(text("ui.2e84a56c5f19", files.length));
        if (directories) parts.push(text("ui.43a28a01147e", directories));
        if (files.length) parts.push(size(files.reduce((n, item) => n + item.size, 0)));
        if (directories) parts.push(text("ui.2ad1d92b2858"));
        document.querySelector("#input-summary").textContent = parts.length ? parts.join(text("common.separator")) : text("ui.02256d747caf");
        document.querySelector("#confirm-add").disabled = inputs.length === 0;
    }
    document.querySelector("#add-files").onclick = () => { if (!inputs.some(item => item.name === "报告.txt")) inputs.push({name:"报告.txt",size:2048}); render(); };
    document.querySelector("#add-folder").onclick = () => { if (!inputs.some(item => item.name === "图片")) inputs.push({name:"图片",directory:true}); render(); };
    for (const id of ["cancel-add", "confirm-add"]) document.getElementById(id).onclick = () => {
        document.querySelector("#design-result").textContent = locale === "zh-CN" ? "设计预览，未修改任何压缩包。" : "Design preview, no archive was modified.";
    };
    render();
    XdzipControls.configure({icon: name => window.XDZIP_ASSETS.icons[theme][name] || ""});
    XdzipControls.enhanceSelects(document.getElementById("add-dialog"));
    window.addDialogDesign = {get inputs() { return inputs; }, setInputs(next) { inputs = next; render(); }};
})();
