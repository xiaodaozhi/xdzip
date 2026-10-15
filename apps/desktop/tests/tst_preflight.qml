import QtQuick
import QtTest
import "../ui" as Ui

Item {
    Window { id: ownerWindow; visible: true; width: 1024; height: 720 }
    Ui.PreflightWindow { id: reportWindow; owner: ownerWindow }
    TestCase {
        name: "PreflightWindow"
        when: windowShown
        function init() {
            Ui.Theme.fontScale = 1;
            reportWindow.report = {entries:"9",bytes:"77619200",findingCount:"0",findings:[]};
            reportWindow.present();
            waitForRendering(reportWindow.contentItem);
        }
        function cleanup() { reportWindow.close(); Ui.Theme.fontScale = 1; }
        function test_compact_and_scroll_body() {
            compare(reportWindow.width,491);
            compare(reportWindow.transientParent,ownerWindow);
            compare(findChild(reportWindow,"preflightHeading").font.pixelSize,20);
            verify(reportWindow.height < 450);
            const compactHeight = reportWindow.height;
            reportWindow.report = {entries:"10000",bytes:"unknown",findingCount:"64",findings:Array(64).fill("Record 1: The name needs filesystem review.")};
            tryVerify(function(){return reportWindow.height>compactHeight;});
            verify(reportWindow.height<=600);
            const body = findChild(reportWindow,"preflightBody");
            verify(body.contentItem.contentHeight>body.height);
            const button = findChild(reportWindow,"preflightClose");
            const point = button.mapToItem(reportWindow.contentItem,0,0);
            verify(point.y+button.height<=reportWindow.height);
            mouseClick(button);
            compare(reportWindow.visible,false);
        }
        function test_large_text_and_error() {
            Ui.Theme.fontScale = 1.5;
            reportWindow.report = {error:"The archive changed. Read it again.",findings:[]};
            waitForRendering(reportWindow.contentItem);
            compare(findChild(reportWindow,"preflightHeading").font.pixelSize,30);
            verify(reportWindow.height<=600);
            const button = findChild(reportWindow,"preflightClose");
            verify(button.mapToItem(reportWindow.contentItem,0,button.height).y<=reportWindow.height);
        }
    }
}
