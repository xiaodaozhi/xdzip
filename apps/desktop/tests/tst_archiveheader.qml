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
                    scale: 1
                },
                {
                    tag: "large",
                    scale: 2
                }
            ];
        }
        function test_controls_align_and_fit(data) {
            Ui.Theme.fontScale = data.scale;
            wait(30);
            const open = findChild(toolbar, "archiveOpen");
            const search = findChild(toolbar, "archiveSearch");
            const op = open.mapToItem(toolbar, 0, 0);
            const sp = search.mapToItem(toolbar, 0, 0);
            verify(Math.abs(op.y + open.height / 2 - sp.y - search.height / 2) < 0.5);
            verify(op.x + open.width < sp.x);
            verify(sp.x + search.width < toolbar.width);
            verify(open.contentItem.height >= open.contentItem.implicitHeight);
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
            keyClick(Qt.Key_Escape);
            compare(search.text, "");
            compare(searchSpy.signalArguments[1][0], "");
        }
    }
}
