import QtQuick
import QtTest
import "../ui" as Ui

Item {
    width: 1024; height: 542
    Ui.WelcomePage { id: page; anchors.fill: parent }
    SignalSpy { id: openSpy; target: page; signalName: "openRequested" }
    SignalSpy { id: createSpy; target: page; signalName: "createRequested" }
    SignalSpy { id: cancelSpy; target: page; signalName: "cancelRequested" }
    TestCase {
        name: "WelcomePage"; when: windowShown
        function init() { Ui.Theme.dark = false; Ui.Theme.fontScale = 1; page.canOpen = true; page.busy = false; page.externalOpening = false; openSpy.clear(); createSpy.clear(); }
        function cleanup() { Ui.Theme.fontScale = 1; }
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
            return [{tag: "light", dark: false, scale: 1}, {tag: "dark", dark: true, scale: 1},
                    {tag: "large", dark: true, scale: 1.5}];
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
            grabImage(page).save("welcome-opening-" + data.tag + ".png");
            mouseClick(cancel, cancel.width / 2, cancel.height / 2);
            compare(cancelSpy.count, 1);
            const count = openSpy.count;
            mouseClick(findChild(page, "welcomeOpen"));
            compare(openSpy.count, count);
            grabImage(page).save("welcome-opening-" + data.tag + ".png");
            page.busy = false;
            verify(!overlay.visible);
        }
        function test_large_type_scroll() {
            Ui.Theme.fontScale = 2;
            const open = findChild(page, "welcomeOpen");
            const create = findChild(page, "welcomeCreate");
            tryVerify(() => open.height > 124 && create.height === open.height);
            verify(open.contentItem.height >= open.contentItem.implicitHeight);
        }
    }
}
