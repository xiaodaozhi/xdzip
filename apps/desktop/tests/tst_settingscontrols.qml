import QtQuick
import QtTest
import "../ui" as Ui

Item {
    id: surface
    width: 640; height: 620
    Ui.OperationSelect {
        id: select
        objectName: "settingsSelectRegression"
        x: 460; y: 40; width: 100
        density: Ui.Theme.settingsBodySize * Ui.Theme.fontScale / Ui.Theme.operationBodySize
        model: ["标准", "English (US)"]
    }
    Ui.OperationFooter {
        id: footer
        width: 612
        anchors.bottom: parent.bottom
        note: "设置仅在本机生效"
        Ui.OperationFooterButton { id: defaults; text: "恢复默认" }
        Ui.OperationFooterButton { id: cancel; text: "取消" }
        Ui.OperationFooterButton { id: done; text: "完成" }
    }
    TestCase {
        name: "SettingsControls"
        when: windowShown
        function cleanup() { select.popup.close(); Ui.Theme.fontScale = 1; select.model = ["标准", "English (US)"]; }
        function test_dropdown_data() { return [{tag:"100",scale:1},{tag:"125",scale:1.25},{tag:"150",scale:1.5},{tag:"legacy200",scale:2}]; }
        function test_dropdown(data) {
            Ui.Theme.fontScale = data.scale;
            select.popup.open();
            tryCompare(select.popup.contentItem, "count", 2);
            waitForRendering(select.popup.contentItem);
            verify(select.popup.width >= select.width);
            verify(select.popup.width <= Ui.Theme.selectPopupMaxWidth);
            verify(Math.abs(select.popup.x + select.popup.width - select.width) <= 1);
            // ListView count can update before its asynchronous delegates exist.
            tryVerify(() => findChild(select.popup.contentItem, "settingsSelectRegressionOption-1") !== null);
            const option = findChild(select.popup.contentItem, "settingsSelectRegressionOption-1");
            verify(option !== null);
            const label = findChild(option, option.objectName + "Label");
            verify(label !== null);
            verify(!label.truncated, "Full language option fits beside the check column");
            const y = done.mapToItem(surface, 0, 0).y;
            verify(Math.abs(defaults.mapToItem(surface,0,0).y-y) <= 1);
            verify(Math.abs(cancel.mapToItem(surface,0,0).y-y) <= 1);
            verify(done.mapToItem(surface,done.width,0).x <= footer.width);
        }
        function test_long_option_is_bounded() {
            select.model = ["x".repeat(300)];
            select.popup.open();
            tryCompare(select.popup.contentItem, "count", 1);
            tryCompare(select.popup, "width", Ui.Theme.selectPopupMaxWidth);
        }
    }
}
