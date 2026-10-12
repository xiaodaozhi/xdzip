"use strict";
// Source pixels are measurement coordinates, not the application's logical size.
// Use the declared 1280-wide design window; never infer a scale from screen DPR.
window.XDZIP_DISPLAY_SPEC = Object.freeze({
    version: 1,
    logicalWindowWidth: 1280,
    sourceWindow: [20, 30, 1548, 940],
    sourceToLogical: 1280 / 1548,
    views: {
        main: { source: "../hifi/screens/main-light.png", size: [1586, 992] },
        extract: { source: "../hifi/screens/extract-macos.png", size: [1092, 1440] },
        create: { source: "../hifi/screens/create.png", size: [1073, 1466] },
        progress: { source: "../hifi/screens/progress.png", size: [1446, 1088] },
        conflict: { source: "../hifi/screens/conflict.png", size: [1478, 1064] },
        settings: { source: "../hifi/screens/settings.png", size: [1092, 1440] },
        empty: { source: null, size: [1586, 992] },
        damaged: { source: null, size: [1586, 992] },
    },
});
