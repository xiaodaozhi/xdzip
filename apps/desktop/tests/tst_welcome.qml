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
        function test_actions_and_states_data() { return [{tag:"light", dark:false}, {tag:"dark", dark:true}]; }
        function test_actions_and_states(data) {
            Ui.Theme.dark = data.dark;
            const open = findChild(page, "welcomeOpen");
            const create = findChild(page, "welcomeCreate");
            mouseMove(page, 0, 0);
            compare(open.background.color, Ui.Theme.accent);
            mouseMove(open, open.width / 2, open.height / 2);
            compare(open.background.color, Ui.Theme.accentHover);
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
        function test_format_group_summary_data() {
            return [1, 1.25, 1.5].map(scale => ({tag: String(scale), scale: scale}));
        }
        function test_format_group_summary(data) {
            page.expandedFormatGroup = "archives";
            Ui.Theme.fontScale = data.scale;
            surface.width = 832;
            const formats = findChild(page, "welcomeFormats");
            const groups = Formats.formatGroups.map(group => findChild(page, "welcomeFormatGroup-" + group.id));
            verify(groups.every(group => group && group.visible));
            const button = findChild(page, "welcomeFormatsSettings");
            verify(button && button.visible && button.enabled);
            mouseClick(button);
            compare(formatsSpy.count, 1);
            formatsSpy.clear();
            tryVerify(() => groups.every(group => group.contentItem.children[0].font.pixelSize === Math.round(11 * data.scale)));
            // A child Text can update its font metrics before the parent layout
            // schedules a second polish. Measure the frame with the new metrics.
            verify(waitForRendering(page));
            verify(waitForPolish(formats));
            tryVerify(() => groups.every(group => group.width > 0 && group.height > 0));
            verify(formats.width <= formats.parent.width);
            console.log("Welcome format group geometry", JSON.stringify({scale: data.scale,
                available: formats.parent.width, width: formats.width}));
            for (const group of groups) {
                const point = group.mapToItem(page, 0, 0);
                verify(point.x >= 0 && point.x + group.width <= page.width);
                verify(group.width > 0 && group.height > 0);
            }
            const preview = findChild(page, "welcomeFormatPreview");
            verify(preview.visible);
            if (data.scale === 1.5) verify(preview.truncated);
            compare(preview.suffixes, Formats.formatGroups[0].suffixes);
            for (const [index, group] of groups.entries()) {
                compare(group.contentItem.children[1].text, String(Formats.formatGroups[index].suffixes.length));
            }
            const textEnvelopes = findChild(page, "welcomeFormatGroup-text-envelopes");
            textEnvelopes.forceActiveFocus(Qt.TabFocusReason);
            keyClick(Qt.Key_Space);
            tryCompare(page, "expandedFormatGroup", "text-envelopes");
            compare(preview.suffixes, Formats.formatGroups.find(group => group.id === "text-envelopes").suffixes);
            keyClick(Qt.Key_Space);
            tryCompare(preview, "visible", false);
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
