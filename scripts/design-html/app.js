/* Offline documentation controls. Search data is bundled; no network or fetch. */
(() => {
  "use strict";
  const root = document.documentElement;
  const themeButton = document.getElementById("theme-toggle");
  let storedTheme = null;
  try { storedTheme = localStorage.getItem("xdzip-docs-theme"); } catch (_) { /* file:// can restrict storage. */ }
  let theme = storedTheme === "light" || storedTheme === "dark"
    ? storedTheme : (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
  const setTheme = () => {
    root.dataset.theme = theme;
    themeButton.textContent = theme === "dark" ? "浅色" : "深色";
    themeButton.setAttribute("aria-label", `切换${theme === "dark" ? "浅色" : "深色"}模式`);
  };
  setTheme();
  themeButton.addEventListener("click", () => {
    theme = theme === "dark" ? "light" : "dark";
    setTheme();
    try { localStorage.setItem("xdzip-docs-theme", theme); } catch (_) { /* Optional preference. */ }
  });

  const menu = document.getElementById("menu-toggle");
  const sidebar = document.getElementById("sidebar");
  const shade = document.getElementById("mobile-shade");
  const closeMenu = () => {
    sidebar.classList.remove("open");
    shade.classList.remove("open");
    menu.setAttribute("aria-expanded", "false");
  };
  menu.addEventListener("click", () => {
    const opened = sidebar.classList.toggle("open");
    shade.classList.toggle("open", opened);
    menu.setAttribute("aria-expanded", String(opened));
  });
  shade.addEventListener("click", closeMenu);
  sidebar.addEventListener("click", event => { if (event.target.closest("a")) closeMenu(); });

  const dialog = document.getElementById("search-dialog");
  const input = document.getElementById("search-input");
  const results = document.getElementById("search-results");
  const status = document.getElementById("search-status");
  const fold = value => value.normalize("NFKC").toLocaleLowerCase();
  const records = (window.XDZIP_SEARCH_INDEX || []).map(record => ({
    ...record, foldedText: fold(record.text), foldedTitle: fold(record.title + " " + record.section)
  }));
  const fragment = (text, query) => {
    const found = fold(text).indexOf(query);
    const start = Math.max(0, found - 42);
    return (start ? "…" : "") + text.slice(start, start + 180) + (text.length > start + 180 ? "…" : "");
  };
  const showSearch = () => {
    closeMenu();
    if (!dialog.open) dialog.showModal();
    input.focus();
    input.select();
  };
  // User content becomes text nodes, never HTML, including excerpts from code.
  const renderSearch = () => {
    const query = fold(input.value.trim());
    results.replaceChildren();
    if (!query) { status.textContent = "输入关键词搜索全部 16 篇文档，可用空格组合关键词。"; return; }
    const tokens = query.split(/\s+/).filter(Boolean);
    const matches = records.filter(record => tokens.every(token =>
      record.foldedText.includes(token) || record.foldedTitle.includes(token)
    )).map(record => ({
      record, score: tokens.reduce((sum, token) => sum + (record.foldedTitle.includes(token) ? 5 : 1), 0)
    })).sort((a, b) => b.score - a.score || a.record.order - b.record.order);
    status.textContent = matches.length ? `找到 ${matches.length} 个相关章节，显示前 ${Math.min(20, matches.length)} 项。` : "没有匹配的章节，请尝试其他关键词。";
    for (const {record} of matches.slice(0, 20)) {
      const link = document.createElement("a");
      link.className = "search-result";
      link.href = record.href;
      const title = document.createElement("strong");
      title.textContent = `${record.title} · ${record.section}`;
      const excerpt = document.createElement("p");
      excerpt.textContent = fragment(record.text, tokens[0]);
      link.append(title, excerpt);
      link.addEventListener("click", () => dialog.close());
      results.append(link);
    }
  };
  input.addEventListener("input", event => { if (!event.isComposing) renderSearch(); });
  input.addEventListener("compositionend", renderSearch);
  document.getElementById("open-search").addEventListener("click", showSearch);
  document.getElementById("close-search").addEventListener("click", () => dialog.close());
  dialog.addEventListener("click", event => {
    const box = dialog.getBoundingClientRect();
    if (event.target === dialog && (event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom)) dialog.close();
  });
  document.addEventListener("keydown", event => {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") { event.preventDefault(); showSearch(); }
    if (event.key === "Escape") closeMenu();
  });
  renderSearch();

  if ("IntersectionObserver" in window) {
    const links = new Map([...document.querySelectorAll(".toc a")].map(link => [decodeURIComponent(link.hash.slice(1)), link]));
    const observer = new IntersectionObserver(entries => {
      for (const entry of entries) {
        if (!entry.isIntersecting || !links.has(entry.target.id)) continue;
        for (const link of links.values()) link.classList.remove("active");
        links.get(entry.target.id).classList.add("active");
      }
    }, {rootMargin: "-80px 0px -65% 0px"});
    for (const id of links.keys()) { const heading = document.getElementById(id); if (heading) observer.observe(heading); }
  }
})();
