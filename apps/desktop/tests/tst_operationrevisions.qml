import QtQuick
import QtQuick.Controls
import QtTest
import "../ui" as Ui

Item {
    id: fixture
    width: 600; height: 400
    Window { id: ownerWindow; visible: true; width: 900; height: 760 }
    Ui.OperationButton { id: primary; x: 20; y: 20; text: "创建压缩包"; primary: true }
    Ui.OperationSegmentedGroup {
        id: segments; x: 20; y: 80; width: 300
        objectName: "formats"
        options: [{text: "ZIP"}, {text: "TAR"}, {text: "TAR.GZ"}]
    }
    Ui.InspectorPanel {
        id: inspector; x: 330; width: 260; height: 380
        detail: ({id: "1", name: "a.txt", size: "1", packed: "1", method: "Store"})
        selectionCount: 0; canExtract: true
        sizeFormatter: value => value + " B"
        iconForFile: () => "qrc:/xdzip/assets/light/file-text.png"
    }
    QtObject {
        id: controller
        property var extractionIntent: ({session: "s", archive: "a.zip", all: true, archiveCount: 9, selectedCount: 2, samples: []})
        property var targetCheck: ({valid: true})
        property bool busy: false
        property string sessionId: "s"
        function targetMatches(path) { return true; }
        function checkTarget(path) {}
        function chooseTarget(path) { return path; }
    }
    Ui.ExtractWindow { id: extract; owner: ownerWindow; controller: controller }
    Ui.ConflictWindow {
        id: conflict; owner: ownerWindow
        task: ({promptName: "a.txt", overwriteAllowed: true, existingBytes: "1", incomingBytes: "1"})
        sizeFormatter: value => value + " B"
    }
    TestCase {
        name: "OperationRevisions"; when: windowShown
        function init() { Ui.Theme.dark = false; Ui.Theme.fontScale = 1; primary.enabled = true; }
        function cleanup() { extract.close(); conflict.close(); Ui.Theme.fontScale = 1; }
        function test_primary_states_data() { return [{tag: "light", dark: false}, {tag: "dark", dark: true}]; }
        function test_primary_states(data) {
            Ui.Theme.dark = data.dark;
            const bg = primary.background;
            mouseMove(fixture, 590, 390);
            compare(bg.color, Ui.Theme.accent);
            mouseMove(primary, primary.width / 2, primary.height / 2);
            compare(bg.color, Ui.Theme.accentHover);
            mousePress(primary, primary.width / 2, primary.height / 2);
            compare(bg.color, Ui.Theme.accentPressed);
            mouseRelease(primary, primary.width / 2, primary.height / 2);
            primary.forceActiveFocus();
            keyPress(Qt.Key_Space);
            compare(bg.color, Ui.Theme.accentPressed);
            keyRelease(Qt.Key_Space);
            primary.enabled = false;
            compare(bg.color, Ui.Theme.accentDisabled);
            compare(primary.contentItem.color, Ui.Theme.accentDisabledText);
            grabImage(fixture).save("operation-controls-" + data.tag + ".png");
        }
        function test_segment_corners() {
            for (let i = 0; i < 3; ++i) {
                segments.currentIndex = i;
                const bg = findChild(segments, "formats-" + i + "Background");
                compare(bg.topLeftRadius, i === 0 ? 4 : 0);
                compare(bg.bottomLeftRadius, bg.topLeftRadius);
                compare(bg.topRightRadius, i === 2 ? 4 : 0);
                compare(bg.bottomRightRadius, bg.topRightRadius);
            }
        }
        function test_empty_inspector_and_extract_states() {
            inspector.selectionCount = 0;
            const action = findChild(inspector, "inspectorExtract");
            verify(!action.visible);
            inspector.selectionCount = 1;
            verify(action.visible);
            mouseMove(action, action.width / 2, action.height / 2);
            compare(action.background.color, Ui.Theme.controlHover);
            mousePress(action, action.width / 2, action.height / 2);
            compare(action.background.color, Ui.Theme.controlPressed);
            mouseRelease(action, action.width / 2, action.height / 2);
        }
        function test_scope_counts() {
            extract.present();
            const scope = findChild(extract, "extractAllScope");
            compare(scope.text, "全部归档");
            compare(findChild(extract, "extractAllScopeCount").text, "9 条记录");
            compare(findChild(extract, "extractSelectedScopeCount").text, "2 项");
        }
        function test_conflict_normal_density() {
            conflict.present(); waitForRendering(conflict.contentItem);
            compare(conflict.width, 491);
            verify(conflict.height <= Math.min(560, Screen.desktopAvailableHeight - 80));
            const rename = findChild(conflict, "conflictRename");
            compare(rename.contentItem.font.pixelSize, 13);
            compare(rename.background.radius, 5);
            compare(findChild(conflict, "conflictNotice").labelItem.font.pixelSize, 12);
            grabImage(conflict.contentItem).save("conflict-normal-density.png");
            Ui.Theme.fontScale = 2;
            waitForRendering(conflict.contentItem);
            verify(conflict.height <= Math.min(560, Screen.desktopAvailableHeight - 80));
            const p = rename.mapToItem(conflict.contentItem, 0, rename.height);
            verify(p.y <= conflict.height + 1);
        }
    }
}
