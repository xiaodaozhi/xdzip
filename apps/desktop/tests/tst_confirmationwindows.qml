import QtQuick
import QtTest
import "../ui" as Ui

Item {
    Window { id: ownerWindow; visible: true; width: 1024; height: 720 }
    Ui.QuitWindow { id: quit; owner: ownerWindow }
    Ui.RestartWindow { id: restart; owner: ownerWindow }
    SignalSpy { id: waitSpy; target: quit; signalName: "waitRequested" }
    SignalSpy { id: cancelSpy; target: quit; signalName: "cancelAndQuitRequested" }
    SignalSpy { id: restartSpy; target: restart; signalName: "restartRequested" }
    TestCase {
        name: "ConfirmationWindows"
        when: windowShown
        function cleanup() {
            quit.close(); restart.close(); quit.stopping = false;
            quit.availableClientHeight = Screen.desktopAvailableHeight - 80;
            restart.availableClientHeight = Screen.desktopAvailableHeight - 80;
            Ui.Theme.fontScale = 1; Ui.Theme.dark = false;
            waitSpy.clear(); cancelSpy.clear(); restartSpy.clear();
        }
        function test_geometry_data() {
            const rows = [];
            for (const dark of [false, true])
                for (const scale of [1, 1.25, 1.5])
                    for (const english of [false, true])
                        for (const kind of ["quit", "stopping", "restart"])
                            rows.push({tag: kind + "-" + dark + "-" + scale + "-" + english, kind, dark, scale, english});
            return rows;
        }
        function test_geometry(data) {
            Ui.Theme.dark = data.dark; Ui.Theme.fontScale = data.scale;
            const window = data.kind === "restart" ? restart : quit;
            quit.stopping = data.kind === "stopping";
            if (data.kind === "restart") {
                window.heading = data.english ? "Restart the application to apply settings?" : "是否要重启应用使设置生效？";
                findChild(restart,"restartLater").text = data.english ? "Later" : "稍后";
                findChild(restart,"restartNow").text = data.english ? "Restart now" : "现在重启";
            } else {
                window.heading = data.kind === "stopping" ? (data.english ? "Stopping tasks" : "正在停止任务") : (data.english ? "Tasks are still running" : "仍有任务正在进行");
                window.message = data.kind === "stopping" ? (data.english ? "Waiting for the current file commit and temporary-data cleanup before quitting." : "等待当前文件提交与暂存清理完成后退出。") : (data.english ? "Wait for tasks to finish, or cancel unfinished work and quit. Committed files will be kept." : "可以等待任务完成，或取消未完成的工作后退出。已提交的文件会保留。");
                findChild(quit,"quitReturn").text = data.english ? "Return to app" : "返回应用";
                findChild(quit,"quitWait").text = data.english ? "Wait until finished" : "等待完成";
                findChild(quit,"quitCancel").text = data.english ? "Cancel and quit" : "取消并退出";
            }
            window.present(); verify(waitForRendering(window.contentItem));
            // Text changes can require another layout polish after the first rendered frame.
            // Check the settled native layout without weakening the normal-height contract.
            if (data.scale === 1) {
                const normalNames = data.kind === "restart" ? ["restartLater","restartNow"] : ["quitReturn","quitWait","quitCancel"];
                for (const name of normalNames)
                    tryCompare(findChild(window,name),"height",32);
            }
            compare(window.width,491);
            compare(window.headingItem.font.pixelSize,20 * data.scale);
            compare(window.messageItem.font.pixelSize,Math.round(13 * data.scale));
            compare(window.modality,Qt.ApplicationModal);
            verify(window.height <= Math.min(600,Screen.desktopAvailableHeight-80));
            verify(window.height < 350, "Short confirmation must not retain legacy blank height");
            const footer = window.footerItem;
            verify(Math.abs(footer.y + footer.height - window.height) <= 1);
            const names = data.kind === "restart" ? ["restartLater","restartNow"] : ["quitReturn","quitWait","quitCancel"];
            let top = -1;
            for (const name of names) {
                const button = findChild(window,name);
                const pos = button.mapToItem(window.contentItem,0,0);
                if (top < 0) top = pos.y;
                verify(Math.abs(pos.y-top) <= 1,"Footer actions stay on one row");
                verify(pos.x >= 0 && pos.x+button.width <= window.width+1,"Footer action must remain inside client area");
                verify(button.contentItem.paintedWidth <= button.availableWidth+1,"Button label must fit, using wrapping when needed");
                verify(button.contentItem.paintedHeight <= button.availableHeight+1,"Wrapped footer label must not be clipped");
                compare(button.contentItem.font.pixelSize,Math.round(13 * data.scale));
                compare(button.background.radius,5);
                if (data.scale === 1) compare(button.height,32,"Normal footer actions retain the approved 32-pixel height");
                compare(button.enabled,data.kind !== "stopping");
            }
            console.info("CONFIRMATION_GEOMETRY " + JSON.stringify({kind:data.kind,dark:data.dark,english:data.english,fontScale:data.scale,dpr:Screen.devicePixelRatio,width:window.width,height:window.height,footerHeight:footer.height,titleSize:window.headingItem.font.pixelSize,bodySize:window.messageItem.font.pixelSize}));
            grabImage(window.contentItem).save("confirmation-"+data.tag+".png");
        }
        function test_small_screen_scroll_keeps_footer_reachable() {
            Ui.Theme.fontScale = 1.5;
            quit.heading = "Tasks are still running";
            quit.message = "Committed files will be kept. ".repeat(100);
            quit.availableClientHeight = 210;
            quit.present(); verify(waitForRendering(quit.contentItem));
            compare(quit.height,210);
            verify(quit.bodyItem.contentHeight > quit.bodyItem.height);
            verify(quit.footerItem.y >= 0);
            grabImage(quit.contentItem).save("small-screen.png");
            verify(Math.abs(quit.footerItem.y+quit.footerItem.height-quit.height)<=1);
            const compact = quit.height;
            quit.message = "Files will be kept.";
            verify(waitForRendering(quit.contentItem));
            verify(quit.height <= compact);
        }
        function test_decision_signals() {
            quit.present(); verify(waitForRendering(quit.contentItem));
            const waitButton=findChild(quit,"quitWait"), cancelButton=findChild(quit,"quitCancel");
            mouseClick(waitButton);compare(waitSpy.count,1);compare(cancelSpy.count,0);
            mouseClick(cancelButton);compare(cancelSpy.count,1);
            quit.stopping=true;verify(!waitButton.enabled);verify(!cancelButton.enabled);verify(!quit.escapeClosesWindow);
            quit.close();restart.present();verify(waitForRendering(restart.contentItem));
            mouseClick(findChild(restart,"restartNow"));compare(restartSpy.count,1);
        }
    }
}
