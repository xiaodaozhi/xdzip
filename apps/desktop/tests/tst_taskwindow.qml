import QtQuick
import QtTest
import "../ui" as Ui

Item {
    id: fixture
    Window {
        id: ownerWindow
        width: 1000
        height: 760
        visible: true
    }
    Ui.TaskWindow {
        id: task
        owner: ownerWindow
        archiveName: "北境旅行.zip"
        destination: "下载 / 北境旅行"
        currentFile: "旅途瞬间-08.jpg"
        processedSummary: task.totalKnown ? qsTr("已处理 46.5 MB / 77.5 MB") : qsTr("已处理 46.5 MB")
        fileCountSummary: "18 / 30 个文件"
        speedSummary: "6.2 MB/s"
        remainingSummary: "约 5 秒"
        phaseSummary: "写入文件"
        fraction: 0.6
        totalKnown: true
        stopping: false
    }
    SignalSpy {
        id: cancelSpy
        target: task
        signalName: "cancelRequested"
    }
    TestCase {
        name: "TaskWindow"
        when: windowShown

        function init() {
            Ui.Theme.dark = false;
            Ui.Theme.fontScale = 1;
            task.totalKnown = true;
            task.currentFile = "旅途瞬间-08.jpg";
            task.stopping = false;
            cancelSpy.clear();
            task.show();
        }
        function cleanup() { task.close(); }

        function test_native_window_close_does_not_cancel() {
            compare(task.transientParent, ownerWindow);
            compare(task.modality, Qt.NonModal);
            verify((task.flags & Qt.FramelessWindowHint) === 0);
            task.close();
            compare(task.visible, false);
            compare(cancelSpy.count, 0);
            task.present();
            compare(task.visible, true);
            verify(isFinite(task.x) && isFinite(task.y));
            compare(task.stopping, false);
        }
        function test_alignment_data() {
            return [{tag: "light", dark: false, scale: 1},
                    {tag: "dark", dark: true, scale: 1},
                    {tag: "large-type", dark: false, scale: 2}];
        }
        function test_alignment(data) {
            Ui.Theme.dark = data.dark;
            Ui.Theme.fontScale = data.scale;
            waitForRendering(task.contentItem);
            const first = task.timingHint;
            const second = task.closingHint;
            compare(first.iconItem.status, Image.Ready);
            compare(second.iconItem.status, Image.Ready);
            const icon1 = first.iconItem.mapToItem(task.contentItem, first.iconItem.width / 2, 0);
            const icon2 = second.iconItem.mapToItem(task.contentItem, second.iconItem.width / 2, 0);
            const label1 = first.labelItem.mapToItem(task.contentItem, 0, 0);
            const label2 = second.labelItem.mapToItem(task.contentItem, 0, 0);
            compare(icon1.x, icon2.x);
            compare(label1.x, label2.x);
            compare(first.iconItem.width, second.iconItem.width);
            verify(first.labelItem.height >= first.labelItem.contentHeight);
            verify(second.labelItem.height >= second.labelItem.contentHeight);
            grabImage(task.contentItem).save("task-window-" + data.tag + ".png");
        }
        function test_large_type_actions_remain_reachable() {
            Ui.Theme.fontScale = 2;
            waitForRendering(task.contentItem);
            const scroll = findChild(task, "taskScroll");
            const flickable = scroll.contentItem;
            verify(flickable.contentHeight > flickable.height);
            flickable.contentY = flickable.contentHeight - flickable.height;
            waitForRendering(task.contentItem);
            const button = findChild(task, "cancelButton");
            const point = button.mapToItem(task.contentItem, button.width / 2, button.height / 2);
            verify(point.y > 0 && point.y < task.height);
            grabImage(task.contentItem).save("task-window-large-type-bottom.png");
            mouseClick(button);
            compare(cancelSpy.count, 1);
        }
        function test_unknown_total_never_shows_percentage() {
            task.totalKnown = false;
            const label = findChild(task, "progressLabel");
            compare(label.text, "总量未知");
            compare(findChild(task, "progressFill").width, 0);
            compare(findChild(task, "processedLabel").text, "已处理 46.5 MB");
        }
        function test_normal_footer_shares_one_row() {
            waitForRendering(task.contentItem);
            const button = findChild(task, "cancelButton");
            const hintCenter = task.closingHint.mapToItem(task.contentItem, 0, task.closingHint.height / 2);
            const buttonCenter = button.mapToItem(task.contentItem, 0, button.height / 2);
            verify(Math.abs(hintCenter.y - buttonCenter.y) <= 1);
            verify(buttonCenter.y < task.height);
        }
        function test_filename_is_plain_text() {
            task.currentFile = "目录/<b>不能作为富文本</b>/图片.jpg";
            const card = findChild(task, "currentFileCard");
            compare(card.valueItem.textFormat, Text.PlainText);
            compare(card.valueItem.text, task.currentFile);
        }
        function test_cancel_is_explicit_and_not_completion() {
            const button = findChild(task, "cancelButton");
            // Activate the control independently of the current scroll position.
            button.clicked();
            compare(cancelSpy.count, 1);
            compare(task.visible, true);
            task.stopping = true;
            compare(button.enabled, false);
            task.close();
            task.present();
            compare(task.stopping, true);
            compare(cancelSpy.count, 1);
        }
    }
}
