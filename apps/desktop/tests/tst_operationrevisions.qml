import QtQuick
import QtQuick.Controls
import QtTest
import "../ui" as Ui

Item {
    id: fixture
    width: 600; height: 400
    Rectangle { anchors.fill: parent; color: Ui.Theme.surface; z: -1 }
    Window { id: ownerWindow; visible: true; width: 900; height: 760 }
    Ui.OperationButton { id: primary; x: 20; y: 20; text: "创建压缩包"; primary: true }
    Ui.OperationSegmentedGroup {
        id: segments; x: 20; y: 80; width: 300
        objectName: "formats"
        options: [{text: "ZIP"}, {text: "TAR"}, {text: "TAR.GZ"}, {text: "7z"}]
    }
    Ui.InspectorPanel {
        id: inspector; x: 330; width: 260; height: 380
        detail: ({id: "1", name: "a.txt", size: "1", packed: "1", method: "Store"})
        selectionCount: 0; canExtract: true
        previewSelectionEntries: []
        sizeFormatter: value => value + " B"
        iconForFile: () => "qrc:/xdzip/assets/light/file-text.png"
    }
    QtObject {
        id: controller
        property var extractionIntent: ({session: "s", archive: "a.zip", all: true, archiveCount: 9, selectedCount: 2, samples: []})
        property var targetCheck: ({valid: true})
        property bool busy: false
        property bool inputActionsEnabled: true
        property string sessionId: "s"
        function targetMatches(path) { return true; }
        function checkTarget(path) {}
        function invalidateTarget() {}
        function chooseTarget(path) { return path; }
    }
    Ui.OperationWindow { id: positioningWindow; centerOnScreen: true; requestedHeight: 240 }
    Ui.ExtractWindow { id: extract; owner: ownerWindow; controller: controller }
    Ui.ConflictWindow {
        id: conflict; owner: ownerWindow
        task: ({id: "1", status: "waiting", promptId: "1", promptName: "a.txt", overwriteAllowed: true, existingBytes: "1", incomingBytes: "1"})
        sizeFormatter: value => value + " B"
    }
    TestCase {
        name: "OperationRevisions"; when: windowShown
        function init() { Ui.Theme.dark = false; Ui.Theme.fontScale = 1; primary.enabled = true; segments.width = 300; }
        function cleanup() { positioningWindow.close(); positioningWindow.requestedHeight = 240; extract.close(); extract.shellMode = false; conflict.close(); Ui.Theme.fontScale = 1; }
        function test_visible_window_keeps_position_when_resized() {
            positioningWindow.present();
            tryVerify(() => positioningWindow.visible);
            wait(100);
            positioningWindow.x += 30;
            positioningWindow.y += 20;
            const x = positioningWindow.x;
            const y = positioningWindow.y;
            positioningWindow.requestedHeight = 360;
            tryCompare(positioningWindow, "height", 360);
            wait(100);
            compare(positioningWindow.x, x);
            compare(positioningWindow.y, y);
            positioningWindow.present();
            wait(100);
            compare(positioningWindow.x, x);
            compare(positioningWindow.y, y);
            positioningWindow.requestedHeight = 240;
            tryCompare(positioningWindow, "height", 240);
            wait(100);
            compare(positioningWindow.x, x);
            compare(positioningWindow.y, y);
        }
        function test_extract_window_title_is_distinct_from_toolbar_action() {
            compare(extract.title, qsTrId("extract.windowTitle"));
            compare(extract.title, "解压文件");
            const heading=findChild(extract,"extractWindowHeading");
            verify(heading!==null); compare(heading.text,extract.title);
        }
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
            const normalBorder = bg.border.color;
            const normalWidth = bg.border.width;
            primary.forceActiveFocus();
            compare(bg.border.color, normalBorder);
            compare(bg.border.width, normalWidth);
            keyPress(Qt.Key_Space);
            compare(bg.color, Ui.Theme.accentPressed);
            keyRelease(Qt.Key_Space);
            primary.enabled = false;
            compare(bg.color, Ui.Theme.accentDisabled);
            compare(primary.contentItem.color, Ui.Theme.accentDisabledText);
            grabImage(fixture).save("operation-controls-" + data.tag + ".png");
        }
        function test_segment_painted_corners_data() {
            const rows = [];
            for (const dark of [false, true])
                for (const index of [0, segments.options.length - 1])
                    for (const state of ["normal", "hover", "pressed"])
                        rows.push({tag: (dark ? "dark" : "light") + "-" + index + "-" + state, index: index, dark: dark, state: state});
            return rows;
        }
        function test_segment_painted_corners(data) {
            Ui.Theme.dark = data.dark;
            segments.width = 200;
            segments.currentIndex = data.index;
            mouseMove(fixture, 590, 390);
            const button = findChild(segments, "formats-" + data.index);
            if (data.state !== "normal") mouseMove(button, button.width / 2, button.height / 2);
            if (data.state === "pressed") mousePress(button, button.width / 2, button.height / 2);
            const bg = findChild(segments, "formats-" + data.index + "Background");
            verify(waitForRendering(bg));
            const image = grabImage(fixture);
            if (data.state === "pressed") mouseRelease(button, button.width / 2, button.height / 2);
            const origin = bg.mapToItem(fixture, 0, 0);
            // QtTest grabs physical pixels but crops to the item logical bounds.
            // Keep the fixture region inside that crop at DPR 1 and 2.
            const ratio = Screen.devicePixelRatio;
            const edgeX = Math.round((origin.x + (data.index === 0 ? 0 : bg.width)) * ratio);
            const seamX = Math.round((origin.x + (data.index === 0 ? bg.width : 0)) * ratio) - (data.index === 0 ? 1 : 0);
            const top = Math.round(origin.y * ratio);
            const read = (x, y) => data.dark ? image.blue(x, y) : image.red(x, y);
            const fillColor = data.state === "pressed" ? Ui.Theme.accentPressed : data.state === "hover" ? Ui.Theme.accentHover : Ui.Theme.accent;
            const fill = Math.round((data.dark ? fillColor.b : fillColor.r) * 255);
            const backdrop = Math.round((data.dark ? Ui.Theme.operationCard.b : Ui.Theme.operationCard.r) * 255);
            let blended = 0;
            for (let y = 0; y < Math.ceil(4 * ratio); ++y) {
                for (let dx = 0; dx < Math.ceil(4 * ratio); ++dx) {
                    const x = data.index === 0 ? edgeX + dx : edgeX - dx - 1;
                    const value = read(x, top + y);
                    if (value > Math.min(fill, backdrop) + 12 && value < Math.max(fill, backdrop) - 12) ++blended;
                }
            }
            verify(blended > 0, "Outer corner must blend the selected fill into the track");
            verify(Math.abs(read(seamX, top) - fill) <= 1, "Inner seam must stay square");
            image.save("segment-corner-" + data.tag + ".png");
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
        function test_shell_extract_layout_data() {
            return [{tag: "normal", scale: 1}, {tag: "large", scale: 1.25}, {tag: "larger", scale: 1.5}];
        }
        function test_shell_extract_layout(data) {
            Ui.Theme.fontScale = data.scale;
            extract.shellMode = true;
            extract.configure();
            verify(waitForRendering(extract.contentItem));
            const layout = findChild(extract, "extractShellLayout");
            const policy = findChild(extract, "extractConflictPolicy");
            verify(layout && policy);
            compare(policy.currentIndex, 2);
            compare(layout.density, policy.density);
            compare(layout.font.pixelSize, policy.font.pixelSize);
            compare(layout.height, policy.height);
            layout.popup.open();
            tryCompare(layout.popup.contentItem, "count", 3);
            tryVerify(() => findChild(layout.popup.contentItem, "extractShellLayoutOption-0") !== null);
            for (let i = 0; i < layout.count; ++i) {
                const option = findChild(layout.popup.contentItem, "extractShellLayoutOption-" + i);
                verify(option !== null);
                const label = findChild(option, option.objectName + "Label");
                verify(label !== null);
                compare(label.font.pixelSize, layout.font.pixelSize);
            }
            layout.popup.close();
            const scroll = findChild(extract, "extractScrollView");
            const lastControl = findChild(extract, "extractHighRatio");
            tryCompare(extract, "height", extract.requestedHeight);
            const gap = scroll.height - lastControl.mapToItem(scroll, 0, lastControl.height).y;
            verify(gap <= extract.unit(20) + 1, "Unexpected space between the last control and footer: " + gap);

            extract.shellMode = false;
        }
        function test_scope_counts() {
            extract.present();
            const scope = findChild(extract, "extractAllScope");
            compare(scope.text, "全部内容");
            compare(findChild(extract, "extractAllScopeCount").text, "9 条记录");
            compare(findChild(extract, "extractSelectedScopeCount").text, "2 项");
        }
        function test_conflict_normal_density() {
            conflict.presentPrompt(true); waitForRendering(conflict.contentItem);
            const client = findChild(conflict, "operationClientArea");
            compare(client.width, 491);
            verify(client.height <= Math.min(560, Screen.desktopAvailableHeight - 80));
            const rename = findChild(conflict, "conflictRename");
            compare(rename.contentItem.font.pixelSize, 13);
            compare(rename.background.radius, 5);
            compare(findChild(conflict, "conflictNotice").labelItem.font.pixelSize, 12);
            grabImage(conflict.contentItem).save("conflict-normal-density.png");
            Ui.Theme.fontScale = 2;
            waitForRendering(conflict.contentItem);
            verify(client.height <= Math.min(560, Screen.desktopAvailableHeight - 80));
            const p = rename.mapToItem(client, 0, rename.height);
            verify(p.y <= client.height + 1);
        }
    }
}
