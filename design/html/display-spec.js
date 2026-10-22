"use strict";
// Generated from docs/visual/layout-scale.json; never infer logical size from DPR.
window.XDZIP_DISPLAY_SPEC = Object.freeze({
    "password": {
        "schema_version": 1,
        "logical_width": 491,
        "maximum_height": 600,
        "content_inset": 24,
        "section_gap": 12,
        "title_size": 20,
        "body_size": 13,
        "note_size": 12,
        "screen_margin": 80,
        "field_height": 28,
        "field_vertical_inset": 4,
        "field_gap": 10,
        "title_line_factor": 1.4,
        "body_line_factor": 1.5,
        "source": "design/hifi/sources/password-extraction.html",
        "states": [
            "request",
            "retry",
            "submitting",
            "zh-CN",
            "en-US",
            "light",
            "dark",
            "100%",
            "125%",
            "150%",
            "small-screen"
        ],
        "notes": [
            "Standalone authored source freezes the normal-density password request before the QML implementation",
            "Native titlebar is outside client measurements and is never redrawn by the product",
            "Content-driven height, bounded scrolling body and fixed shared operation footer",
            "Password input is never trimmed or retained after submission, cancellation or replacement",
            "Screenshots and measurement reports belong in build/ or dist/ and must use an empty password field"
        ],
        "source_sha256": "2d119520066d75abcf088288ddbf3a38f6eff4aa2eca0c79f1f6683ab854307d"
    },
    "about": {
        "schema_version": 1,
        "logical_width": 491,
        "maximum_height": 640,
        "content_inset": 24,
        "section_gap": 16,
        "row_gap": 10,
        "artwork_size": 64,
        "title_size": 24,
        "body_size": 13,
        "note_size": 12,
        "source": "design/hifi/about.html",
        "states": [
            "light",
            "dark",
            "zh-CN",
            "en-US",
            "100%",
            "125%",
            "150%",
            "dirty",
            "unknown-revision",
            "update-checking",
            "update-current",
            "update-available",
            "update-downloading",
            "update-waiting-tasks",
            "update-error",
            "update-manual-install"
        ],
        "artwork_source": "design/assets/brand/xdzip.svg",
        "macos_artwork_source": "design/assets/brand/macos/xdzip.png",
        "update_progress_height": 6
    },
    "version": 2,
    "logicalWindowWidth": 1024,
    "windowsCaption": {
        "iconButtonSize": 28,
        "fileActionGap": 6.4,
        "sortIndicatorGap": 6,
        "columnResizeHitWidth": 8,
        "columnMinimumWidth": 64,
        "nameColumnMinimumWidth": 160,
        "toolbarContentGap": 6,
        "breadcrumbHeight": 28,
        "popupGap": 4,
        "treeTrailingInset": 16,
        "gridInset": 16,
        "gridCheckboxInset": 8,
        "gridCellWidth": 144,
        "gridCellHeight": 124,
        "gridGap": 12,
        "gridIconSize": 40,
        "formatBadgeMinimumWidth": 32,
        "formatBadgeInset": 6,
        "entryBandTopInset": 1,
        "entryBandBottomInset": 2,
        "entrySeparatorHeight": 1,
        "captionButtonWidth": 46,
        "captionIconSize": 18,
        "captionRestoreOffset": 3
    },
    "sourceWindow": [
        20,
        30,
        1548,
        940
    ],
    "sourceToLogical": 0.661498708010336,
    "operationSourceToLogical": 0.8268733850129198,
    "mainDensity": 0.8,
    "views": {
        "password": {
            "source": null,
            "size": [
                780,
                900
            ]
        },
        "quit": {
            "source": null,
            "size": [
                780,
                900
            ]
        },
        "restart": {
            "source": null,
            "size": [
                780,
                900
            ]
        },
        "about": {
            "source": null,
            "size": [
                780,
                900
            ]
        },
        "welcome": {
            "source": "../hifi/screens/welcome-top-light.png",
            "size": [
                1586,
                992
            ],
            "sourceToLogical": 0.661498708010336,
            "darkSource": "../hifi/screens/welcome-top-dark.png"
        },
        "welcome-narrow": {
            "source": "../hifi/screens/welcome-top-narrow.png",
            "size": [
                1296,
                875
            ],
            "sourceToLogical": 0.661498708010336,
            "window": [
                20,
                30,
                1257.75,
                822.375
            ]
        },
        "main": {
            "source": "../hifi/screens/main-capsule-light.png",
            "size": [
                1586,
                992
            ],
            "darkSource": "../hifi/screens/main-capsule-dark.png"
        },
        "extract": {
            "source": "../hifi/screens/extract-macos.png",
            "size": [
                1092,
                1440
            ],
            "sourceToLogical": 0.5291989664082688
        },
        "create": {
            "source": "../hifi/screens/create.png",
            "size": [
                1073,
                1466
            ],
            "sourceToLogical": 0.5291989664082688
        },
        "progress": {
            "source": "../hifi/screens/progress.png",
            "size": [
                1446,
                1088
            ],
            "sourceToLogical": 0.5291989664082688
        },
        "archive-test": {
            "source": "../hifi/screens/integrity-progress.png",
            "size": [
                1445,
                1088
            ],
            "sourceToLogical": 0.5291989664082688
        },
        "result": {
            "source": "../hifi/screens/partial-result.png",
            "size": [
                1421,
                1108
            ],
            "sourceToLogical": 0.5313971742543171
        },
        "conflict": {
            "source": "../hifi/screens/conflict.png",
            "size": [
                1478,
                1064
            ]
        },
        "settings": {
            "source": "../hifi/screens/settings-current-light.png",
            "size": [
                780,
                1007
            ],
            "darkSource": "../hifi/screens/settings-current-dark.png"
        },
        "preflight": {
            "source": "../hifi/screens/preflight-body-note-light.png",
            "size": [
                634,
                476
            ],
            "darkSource": "../hifi/screens/preflight-body-note-dark.png"
        },
        "empty": {
            "source": null,
            "size": [
                1586,
                992
            ]
        },
        "damaged": {
            "source": null,
            "size": [
                1586,
                992
            ]
        }
    }
});
