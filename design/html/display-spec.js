"use strict";
// Source pixels are measurement coordinates, not the application's logical size.
// Use the declared 1280-wide design window; never infer a scale from screen DPR.
window.XDZIP_DISPLAY_SPEC = Object.freeze({
    version: 1,
    logicalWindowWidth: 1280,
    sourceWindow: [20, 30, 1548, 940],
    sourceToLogical: 1280 / 1548,
    views: {
        main: { source: "../hifi/v1/screens/main-light.png", size: [1586, 992] },
        extract: { source: "../hifi/v1/screens/extract-macos.png", size: [1092, 1440] },
        create: { source: "../hifi/v2/screens/01-create.png", size: [1073, 1466] },
        progress: { source: "../hifi/v2/screens/03-progress.png", size: [1446, 1088] },
        conflict: { source: "../hifi/v2/screens/02-conflict.png", size: [1478, 1064] },
        settings: { source: "../hifi/v2/screens/05-settings.png", size: [1092, 1440] },
        empty: { source: null, size: [1586, 992] },
        damaged: { source: null, size: [1586, 992] },
    },
});
