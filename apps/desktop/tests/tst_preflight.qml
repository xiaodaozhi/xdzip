import QtQuick
import QtTest
import "../ui" as Ui

Item {
    Window { id: ownerWindow; visible: true; width: 1024; height: 720 }
    Ui.PreflightWindow { id: reportWindow; owner: ownerWindow }
    Ui.OperationFooter {
        id: referenceFooter
        width: 491
        height: implicitHeight
        opacity: 0
        Ui.OperationFooterButton { text: qsTrId("ui.26867436321f"); primary: true }
    }
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
            // At high DPR the offscreen desktop can already cap the compact report.
            tryCompare(reportWindow,"height",Math.min(600, Screen.desktopAvailableHeight - 80));
            verify(reportWindow.height>=compactHeight);
            verify(reportWindow.height<=600);
            const body = findChild(reportWindow,"preflightBody");
            verify(body.contentItem.contentHeight>body.height);
            const button = findChild(reportWindow,"preflightClose");
            const point = button.mapToItem(reportWindow.contentItem,0,0);
            verify(point.y+button.height<=reportWindow.height);
            mouseClick(button);
            compare(reportWindow.visible,false);
        }
        function test_footer_height_data() {
            return [{tag:"100",scale:1},{tag:"125",scale:1.25},{tag:"150",scale:1.5}];
        }
        function test_footer_height(data) {
            Ui.Theme.fontScale = data.scale;
            const footer = findChild(reportWindow,"preflightFooter");
            const note = findChild(reportWindow,"preflightNote");
            verify(note);
            compare(footer.note, "");
            tryCompare(footer,"height",referenceFooter.implicitHeight);
            const height = footer.height;
            const body = findChild(reportWindow,"preflightBody");
            const flick = body.contentItem;
            flick.contentY = Math.max(0,flick.contentHeight-flick.height);
            tryVerify(function(){return note.mapToItem(reportWindow.contentItem,0,note.height).y <= footer.y+1;});
            reportWindow.report = {entries:"10000",bytes:"unknown",findingCount:"64",findings:Array(64).fill("Record 1: The name needs filesystem review.")};
            waitForRendering(reportWindow.contentItem);
            compare(footer.height,height);
            compare(findChild(reportWindow,"preflightClose").mapToItem(reportWindow.contentItem,0,0).y >= footer.y,true);
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
