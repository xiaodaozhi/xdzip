import QtQuick
import QtTest
import "../ui" as Ui

Item {
    Window { id: ownerWindow; visible: true; width: 1000; height: 760 }
    Ui.IntegrityWindow {
        id: task
        owner: ownerWindow
        sampleData: true
        archiveName: "北境旅行.zip"
        destination: "下载 / 北境旅行.zip"
        currentFile: "旅途瞬间-08.jpg"
        processedSummary: "已读取 46.5 MB / 77.5 MB"
        fileCountSummary: "18 / 30 个文件"
        speedSummary: "6.2 MB/s"
        remainingSummary: "约 5 秒"
        phaseSummary: "读取并校验"
        resultSummary: "界面预览已结束，未执行校验。"
        fraction: 0.6; totalKnown: true; stopping: false
    }
    SignalSpy { id: cancelSpy; target: task; signalName: "cancelRequested" }
    SignalSpy { id: outputSpy; target: task; signalName: "openOutputRequested" }
    TestCase {
        name: "IntegrityWindow"; when: windowShown
        function init() {
            Ui.Theme.dark = false; Ui.Theme.fontScale = 1;
            task.taskStatus = "running"; task.totalKnown = true;
            cancelSpy.clear(); outputSpy.clear(); task.present();
            waitForRendering(task.contentItem);
        }
        function cleanup() { task.close(); Ui.Theme.fontScale = 1; }
        function test_native_shared_layout() {
            compare(task.transientParent, ownerWindow);
            verify((task.flags & Qt.FramelessWindowHint) === 0);
            compare(task.width, 491);
            verify(task.height > 0 && task.height <= Math.min(600, Screen.desktopAvailableHeight - 80));
            compare(task.heading, "正在检查压缩包");
            compare(findChild(task, "progressLabel").text, "60%");
            compare(task.closingHint.text, "界面预览，不执行实际校验。");
            const bodyText = task.timingHint.labelItem.mapToItem(task.contentItem, 0, 0);
            const footerText = task.closingHint.labelItem.mapToItem(task.contentItem, 0, 0);
            compare(bodyText.x, footerText.x);
            if (task.height === 490) {
                const footer = findChild(task, "taskFooter");
                const bodyBottom = task.timingHint.mapToItem(task.contentItem, 0, task.timingHint.height).y;
                verify(bodyBottom <= footer.mapToItem(task.contentItem, 0, 0).y);
            }
            grabImage(task.contentItem).save("integrity-progress.png");
        }
        function test_unknown_total_is_indeterminate() {
            task.totalKnown = false;
            compare(findChild(task, "progressLabel").text, "总量未知");
            compare(findChild(task, "progressFill").width, 0);
        }
        function test_close_and_cancel_are_distinct() {
            task.close(); compare(cancelSpy.count, 0);
            task.present();
            const cancel = findChild(task, "cancelButton");
            compare(cancel.text, "取消检查");
            mouseClick(cancel);
            compare(cancelSpy.count, 1); compare(outputSpy.count, 0);
        }
        function test_terminal_never_claims_saved_files() {
            task.taskStatus = "succeeded";
            compare(task.heading, "检查完成");
            verify(!findChild(task, "taskResultPanel").visible);
            verify(!findChild(task, "cancelButton").visible);
            task.taskStatus = "cancelled";
            compare(task.heading, "检查已取消");
            compare(outputSpy.count, 0);
        }
        function test_large_type_and_dark() {
            Ui.Theme.dark = true; Ui.Theme.fontScale = 2;
            waitForRendering(task.contentItem);
            const cancel = findChild(task, "cancelButton");
            const top = cancel.mapToItem(task.contentItem, 0, 0);
            verify(top.y >= 0 && top.y + cancel.height <= task.height);
            verify(cancel.width >= cancel.contentItem.implicitWidth);
            grabImage(task.contentItem).save("integrity-progress-large-dark.png");
        }
    }
}
