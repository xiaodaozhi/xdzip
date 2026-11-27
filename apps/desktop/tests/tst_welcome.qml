import QtQuick
import QtTest
import "../ui" as Ui
import "../ui/ArchiveFormats.js" as Formats

Item {
    id: surface
    width: 1024; height: 542
    Ui.WelcomePage { id: page; anchors.fill: parent }
    SignalSpy { id: openSpy; target: page; signalName: "openRequested" }
    SignalSpy { id: createSpy; target: page; signalName: "createRequested" }
    SignalSpy { id: formatsSpy; target: page; signalName: "formatsRequested" }
    SignalSpy { id: cancelSpy; target: page; signalName: "cancelRequested" }
    TestCase {
        name: "WelcomePage"; when: windowShown
        function init() { Ui.Theme.dark = false; Ui.Theme.fontScale = 1; page.message=""; page.canOpen = true; page.busy = false; page.externalOpening = false; openSpy.clear(); createSpy.clear(); }
        function cleanup() { Ui.Theme.fontScale = 1; surface.width = 1024; }
        function test_brand_wordmark_data() {
            return [1, 1.25, 1.5].map(scale => ({tag: String(scale), scale}));
        }
        function test_brand_wordmark(data) {
            surface.width = 832;
            Ui.Theme.fontScale = data.scale;
            const brand = findChild(page, "welcomeBrandWordmark");
            const headline = findChild(page, "welcomeHeadline");
            tryCompare(brand, "status", Image.Ready);
            verify(brand.source.toString().endsWith("brand-wordmark-light.svg"));
            // ColumnLayout rounds preferred heights to logical-pixel boundaries.
            tryVerify(() => Math.abs(brand.height - 26 * data.scale) <= 0.5);
            verify(waitForPolish(brand.parent));
            const center = brand.mapToItem(page, brand.width / 2, 0);
            fuzzyCompare(center.x, page.width / 2, 0.5);
            compare(center.y, Ui.Theme.welcomeContentTopInset);
            verify(brand.mapToItem(page, 0, brand.height).y < headline.mapToItem(page, 0, 0).y);
            Ui.Theme.dark = true;
            tryCompare(brand, "status", Image.Ready);
            verify(brand.source.toString().endsWith("brand-wordmark-dark.svg"));
        }
        function test_actions_and_states_data() { return [{tag:"light", dark:false}, {tag:"dark", dark:true}]; }
        function test_actions_and_states(data) {
            Ui.Theme.dark = data.dark;
            const open = findChild(page, "welcomeOpen");
            const create = findChild(page, "welcomeCreate");
            mouseMove(page, 0, 0);
            const recent = findChild(page, "welcomeRecentToggle");
            const recentBackground = findChild(page, "welcomeRecentBackground");
            compare(open.background.color, Ui.Theme.accent);
            verify(!recentBackground.visible);
            mouseMove(open, open.width / 2, open.height / 2);
            compare(open.background.color, Ui.Theme.accentHover);
            compare(recentBackground.color, Ui.Theme.accent);
            mouseMove(recent, recent.width / 2, recent.height / 2);
            compare(open.background.color, Ui.Theme.accent);
            compare(recentBackground.color, Ui.Theme.accentHover);
            mouseMove(open, open.width / 2, open.height / 2);
            mousePress(open, open.width / 2, open.height / 2);
            compare(open.background.color, Ui.Theme.accentPressed);
            mouseRelease(open, open.width / 2, open.height / 2);
            compare(openSpy.count, 1);
            page.focusOpen();
            keyClick(Qt.Key_Space);
            compare(openSpy.count, 2);
            create.forceActiveFocus(Qt.TabFocusReason);
            keyClick(Qt.Key_Space);
            compare(createSpy.count, 1);
            page.canOpen = false;
            page.busy = true;
            verify(findChild(open, "welcomeBusyIndicator").running);
            compare(open.background.color, Ui.Theme.accentDisabled);
            mouseClick(open);
            compare(openSpy.count, 2);
        }
        function test_opening_overlay_data() {
            return [{tag: "light", dark: false, scale: 1, height:542}, {tag: "dark", dark: true, scale: 1, height:542},
                    {tag: "large", dark: true, scale: 1.5, height:542}, {tag:"short",dark:false,scale:1.5,height:160}];
        }
        function test_external_open_hides_welcome_until_cancel_or_failure() {
            page.externalOpening = true;
            page.busy = true;
            const content = findChild(page, "welcomeContent");
            const overlay = findChild(page, "welcomeOpeningOverlay");
            verify(!content.visible);
            verify(overlay.visible);
            verify(waitForRendering(overlay));
            grabImage(page).save("welcome-external-opening.png");
            page.busy = false;
            page.externalOpening = false;
            verify(content.visible);
            verify(!overlay.visible);
        }
        function test_opening_overlay(data) {
            Ui.Theme.dark = data.dark; Ui.Theme.fontScale = data.scale;
            surface.height = data.height;
            page.openingName = "very-long-name.zip";
            page.canOpen = false; page.busy = true;
            cancelSpy.clear();
            const overlay = findChild(page, "welcomeOpeningOverlay");
            const card = findChild(page, "welcomeOpeningCard");
            const cancel = findChild(page, "welcomeOpeningCancel");
            const progress = findChild(page, "welcomeOpeningProgress");
            tryVerify(() => overlay.visible && cancel.visible && cancel.width > 0);
            verify(waitForRendering(card));
            verify(progress.indeterminate);
            verify(card.width <= page.width && card.height <= page.height);
            const bottom = cancel.mapToItem(card, 0, cancel.height);
            verify(bottom.y <= card.height, "Cancel must remain reachable");
            verify(Math.abs(card.height-bottom.y-Ui.Theme.welcomeOpeningInset)<1);
            grabImage(page).save("welcome-opening-" + data.tag + ".png");
            mouseClick(cancel, cancel.width / 2, cancel.height / 2);
            compare(cancelSpy.count, 1);
            verify(!cancel.enabled);
            mouseClick(cancel);
            compare(cancelSpy.count,1);
            const count = openSpy.count;
            mouseClick(findChild(page, "welcomeOpen"));
            compare(openSpy.count, count);
            grabImage(page).save("welcome-opening-" + data.tag + ".png");
            page.busy = false;
            surface.height=542;
            verify(!overlay.visible);
        }
        function test_large_type_scroll() {
            Ui.Theme.fontScale = 2;
            const open = findChild(page, "welcomeOpen");
            const create = findChild(page, "welcomeCreate");
            tryVerify(() => open.height > 124 && create.height === open.height);
            verify(open.contentItem.height >= open.contentItem.implicitHeight);
        }
        function test_formats_settings_hover_data() {
            return [{tag: "light", dark: false}, {tag: "dark", dark: true}];
        }
        function test_formats_settings_hover(data) {
            Ui.Theme.dark = data.dark;
            formatsSpy.clear();
            const button = findChild(page, "welcomeFormatsSettings");
            verify(waitForPolish(button.parent));
            page.revealAction(button);
            tryVerify(() => {
                const center = button.mapToItem(page, button.width / 2, button.height / 2);
                return button.width > 0 && button.height > 0
                    && center.x >= 0 && center.x < page.width
                    && center.y >= 0 && center.y < page.height;
            });
            mouseMove(page, 0, 0);
            compare(button.background.color, "#00000000");
            mouseMove(button, button.width / 2, button.height / 2);
            tryCompare(button, "hovered", true);
            compare(button.leftPadding, Ui.Theme.welcomeFormatLinkInset);
            compare(button.rightPadding, Ui.Theme.welcomeFormatLinkInset);
            verify(Math.abs(button.contentItem.x - button.leftPadding) <= 0.5);
            verify(Math.abs(button.width - button.contentItem.x - button.contentItem.width - button.rightPadding) <= 0.5);
            compare(button.background.color, Qt.rgba(Ui.Theme.controlHover.r, Ui.Theme.controlHover.g, Ui.Theme.controlHover.b, 0.5));
            mousePress(button, button.width / 2, button.height / 2);
            compare(button.background.color, Ui.Theme.controlPressed);
            mouseRelease(button, button.width / 2, button.height / 2);
            compare(formatsSpy.count, 1);
            compare(button.background.color, Qt.rgba(Ui.Theme.controlHover.r, Ui.Theme.controlHover.g, Ui.Theme.controlHover.b, 0.5));
            mouseMove(page, 0, 0);
            tryCompare(button, "hovered", false);
            compare(button.background.color, "#00000000");
        }
        function test_format_summary_data() {
            return [1, 1.25, 1.5].map(scale => ({tag: String(scale), scale: scale}));
        }
        function test_format_summary(data) {
            Ui.Theme.fontScale = data.scale;
            surface.width = 832;
            const summary = findChild(page, "welcomeFormatSummary");
            const button = findChild(page, "welcomeFormatsSettings");
            const labels = Formats.formatGroups.filter(group => group.id !== "installer-metadata")
                .reduce((all, group) => all.concat(group.labels), []);
            compare(page.readableFormatCount, new Set(labels).size);
            compare(summary.text, qsTrId("welcome.formatSummary", page.readableFormatCount));
            compare(summary.color, Ui.Theme.textSecondary);
            compare(summary.font.weight, Font.Normal);
            tryVerify(() => summary.font.pixelSize === Math.round(13 * data.scale));
            verify(waitForPolish(summary.parent));
            verify(summary.mapToItem(button, summary.width, 0).x <= 0);
        }
        function test_feedback_preserves_top_anchors_data() {
            const cases=[];
            for (const scale of [1,1.25,1.5])
                for (const language of ["zh-CN","en-US"])
                    cases.push({tag:language+scale,dark:language==="en-US",language:language,scale:scale});
            return cases;
        }
        function test_feedback_preserves_top_anchors(data) {
            Ui.Theme.dark=data.dark; Ui.Theme.fontScale=data.scale;
            const headline=findChild(page,"welcomeHeadline"), open=findChild(page,"welcomeOpen");
            verify(waitForPolish(headline.parent));
            waitForRendering(page,100);
            const headlineY=headline.mapToItem(page,0,0).y, openY=open.mapToItem(page,0,0).y;
            page.message=(data.language==="zh-CN"?"暂不支持此压缩包的卷布局、加密或压缩方法。":"Unsupported archive layout, encryption or compression method. ").repeat(8);
            const feedback=findChild(page,"welcomeFeedback"), icon=findChild(page,"welcomeFeedbackIcon");
            tryVerify(()=>feedback.visible && feedback.lineCount>1 && icon.status===Image.Ready);
            compare(headline.mapToItem(page,0,0).y,headlineY);
            compare(open.mapToItem(page,0,0).y,openY);
            compare(feedback.color,Ui.Theme.menuDanger);
            verify(icon.visible);
            waitForRendering(page,100);
            grabImage(page).save("welcome-feedback-"+data.tag+".png");
            surface.height=800;
            verify(waitForPolish(headline.parent));
            waitForRendering(page,100);
            compare(headline.parent.y,Ui.Theme.welcomeContentTopInset);
            // Fractional native DPR may round the text baseline by one pixel on resize.
            // Error-only movement above remains an exact comparison.
            verify(Math.abs(headline.mapToItem(page,0,0).y-headlineY)<=1);
            page.message="";
            surface.height=542;
        }
    }
}
