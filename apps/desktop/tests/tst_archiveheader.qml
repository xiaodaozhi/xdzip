import QtQuick
import QtTest
import "../ui" as Ui

Item {
    width: 1160
    height: 300
    Ui.ArchiveToolbar {
        id: toolbar
        width: parent.width
        canOpen: true
        canSearch: true
        canInspect: true
        hasSelection: true
    }
    SignalSpy {
        id: openSpy
        target: toolbar
        signalName: "openRequested"
    }
    SignalSpy {
        id: searchSpy
        target: toolbar
        signalName: "searchRequested"
    }
    TestCase {
        name: "ArchiveHeader"
        when: windowShown
        function cleanup() {
            Ui.Theme.fontScale = 1;
        }
        function test_controls_align_and_fit_data() {
            return [
                {
                    tag: "normal",
                    scale: 1,
                    width: 1024
                },
                {
                    tag: "large",
                    scale: 2,
                    width: 832
                }
            ];
        }
        function test_controls_align_and_fit(data) {
            Ui.Theme.fontScale = data.scale;
            toolbar.width = data.width;
            const open = findChild(toolbar, "archiveOpen");
            const search = findChild(toolbar, "archiveSearch");
            // Font changes and RowLayout polishing complete on different frames on Cocoa.
            // Wait for the observable layout state, not a platform-dependent fixed delay.
            tryCompare(open, "height", toolbar.controlHeight);
            tryCompare(search, "height", toolbar.controlHeight);
            const op = open.mapToItem(toolbar, 0, 0);
            const sp = search.mapToItem(toolbar, 0, 0);
            verify(Math.abs(op.y + open.height / 2 - sp.y - search.height / 2) < 0.5, "Open y/height=" + op.y + "/" + open.height + "; search=" + sp.y + "/" + search.height);
            verify(op.x + open.width < sp.x);
            verify(sp.x + search.width < toolbar.width);
            verify(open.contentItem.height >= open.contentItem.implicitHeight);
            const extract = findChild(toolbar, "archiveExtract");
            tryCompare(extract, "height", toolbar.controlHeight);
            tryVerify(() => extract.mapToItem(toolbar, extract.width, 0).x <= toolbar.width - 11, 5000, "Extract right=" + extract.mapToItem(toolbar, extract.width, 0).x + "; toolbar=" + toolbar.width);
            const icon = findChild(open, "actionIcon");
            const label = findChild(open, "actionLabel");
            tryVerify(() => Math.abs(icon.mapToItem(open, 0, icon.height / 2).y - open.height / 2) < 0.5);
            tryVerify(() => Math.abs(label.mapToItem(open, 0, label.height / 2).y - open.height / 2) < 0.5);
            verify(Math.abs(icon.width - 32 * 1024 / 1548) < 0.01);
            compare(label.font.pixelSize, 12 * data.scale);
            verify(extract.enabled);
            verify(findChild(extract, "actionSplitIndicator").visible);
            compare(findChild(extract, "actionLabel").opacity, 1);
            compare(findChild(extract, "actionBackground").opacity, 1);
        }
        function test_real_action_and_search() {
            const open = findChild(toolbar, "archiveOpen");
            const search = findChild(toolbar, "archiveSearch");
            openSpy.clear();
            mouseClick(open);
            compare(openSpy.count, 1);
            search.forceActiveFocus();
            search.text = "notes";
            searchSpy.clear();
            keyClick(Qt.Key_Return);
            compare(searchSpy.signalArguments[0][0], "notes");
            tryCompare(search, "activeFocus", false);
            search.forceActiveFocus();
            keyClick(Qt.Key_Escape);
            tryCompare(search, "activeFocus", false);
            compare(search.text, "notes");
            compare(searchSpy.count, 1);
        }
        function test_search_font_follows_user_text() {
            const search = findChild(toolbar, "archiveSearch");
            search.clear();
            compare(search.font.family, Ui.Theme.uiFontFamily);
            search.text = "稻草/日本語";
            compare(search.font.family, Ui.Theme.userDataFontFamily);
            search.clear();
            compare(search.font.family, Ui.Theme.uiFontFamily);
        }
    }
}
