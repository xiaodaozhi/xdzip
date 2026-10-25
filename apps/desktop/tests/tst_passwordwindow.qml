import QtQuick
import QtTest
import "../ui" as Ui

Item {
    Window { id: ownerWindow; visible: true; width: 1024; height: 720 }
    QtObject {
        id: testController
        property var passwordPrompt: ({})
        property bool accept: true
        property int submittedCount: 0
        property int submittedLength: -1
        property bool submittedMatches: false
        property string expectedInput: ""
        property int cancelledCount: 0
        property string cancelledTask: ""
        function submitPassword(task, prompt, value) {
            if (task !== passwordPrompt.taskId || prompt !== passwordPrompt.promptId) return false;
            ++submittedCount;
            submittedLength = value.length;
            submittedMatches = value === expectedInput;
            return accept;
        }
        function cancelTask(task) { ++cancelledCount; cancelledTask = task; }
    }
    Ui.PasswordWindow { id: passwordWindow; owner: ownerWindow; controller: testController }
    TestCase {
        name: "PasswordWindow"
        when: windowShown
        function request(token, retry) {
            testController.passwordPrompt = {taskId:"task-1",promptId:token,name:"项目资料 / 设计文件.pdf",retry:!!retry};
            tryCompare(passwordWindow,"visible",true);
            verify(waitForRendering(passwordWindow.contentItem));
        }
        function cleanup() {
            testController.passwordPrompt = {};
            testController.accept = true;
            testController.submittedCount = 0;
            testController.submittedLength = -1;
            testController.submittedMatches = false;
            testController.expectedInput = "";
            testController.cancelledCount = 0;
            testController.cancelledTask = "";
            passwordWindow.availableClientHeight = Screen.desktopAvailableHeight - 80;
            Ui.Theme.fontScale = 1;
            Ui.Theme.dark = false;
        }
        function test_empty_and_pending_submission() {
            request("empty",false);
            compare(passwordWindow.fieldItem.echoMode,TextInput.Password);
            verify(findChild(passwordWindow,"passwordSubmit").enabled);
            grabImage(passwordWindow.contentItem).save("password-request.png");
            mouseClick(findChild(passwordWindow,"passwordSubmit"));
            compare(testController.submittedCount,1);
            compare(testController.submittedLength,0);
            verify(testController.submittedMatches);
            verify(passwordWindow.pending);
            verify(!findChild(passwordWindow,"passwordSubmit").enabled);
            passwordWindow.submit();
            compare(testController.submittedCount,1);
            compare(passwordWindow.fieldItem.text,"");
            verify(passwordWindow.visible,"Only the next snapshot dismisses an accepted request");
            verify(waitForRendering(passwordWindow.contentItem));
            grabImage(passwordWindow.contentItem).save("password-pending.png");
            testController.passwordPrompt = {};
            tryCompare(passwordWindow,"visible",false);
            compare(testController.cancelledCount,0);
        }
        function test_check_password_uses_check_operation_text() {
            testController.passwordPrompt = {taskId:"check-1",promptId:"check-password",name:"secret.pdf",retry:false,kind:"check"};
            tryCompare(passwordWindow,"visible",true);
            compare(passwordWindow.title,qsTrId("integrity.passwordTitle"));
            compare(findChild(passwordWindow,"passwordSubmit").text,qsTrId("integrity.continue"));
        }
        function test_open_password_uses_open_operation_and_identity() {
            testController.passwordPrompt = {taskId:"open",promptId:"open-1",name:"secret.7z",retry:false,kind:"open"};
            tryCompare(passwordWindow,"visible",true);
            compare(passwordWindow.title,qsTrId("password.openTitle"));
            compare(findChild(passwordWindow,"passwordSubmit").text,qsTrId("password.openContinue"));
            compare(findChild(passwordWindow,"passwordCancel").text,qsTrId("common.cancel"));
            passwordWindow.close();
            compare(testController.cancelledTask,"open");
        }
        function test_control_characters_are_not_trimmed() {
            request("controls",false);
            testController.expectedInput = " \t\u0000\r\n ";
            passwordWindow.fieldItem.text = testController.expectedInput;
            verify(passwordWindow.fieldItem.text === testController.expectedInput);
            passwordWindow.submit();
            compare(testController.submittedCount,1);
            verify(testController.submittedMatches);
            compare(passwordWindow.fieldItem.text,"");
        }
        function test_retry_replacement_and_rejected_submission() {
            request("first",false);
            passwordWindow.fieldItem.text = "  synthetic input  ";
            testController.expectedInput = passwordWindow.fieldItem.text;
            mouseClick(findChild(passwordWindow,"passwordVisibility"));
            compare(passwordWindow.fieldItem.echoMode,TextInput.Normal);
            testController.accept = false;
            passwordWindow.submit();
            verify(!passwordWindow.pending);
            verify(passwordWindow.rejected);
            compare(testController.submittedCount,1);
            verify(testController.submittedMatches,"The original input must not be trimmed");
            verify(passwordWindow.fieldItem.text.length > 0,"Immediate rejection retains editable input");
            request("retry",true);
            compare(passwordWindow.fieldItem.text,"");
            compare(passwordWindow.fieldItem.echoMode,TextInput.Password);
            verify(!passwordWindow.rejected);
            verify(findChild(passwordWindow,"passwordError").visible);
            testController.accept = true;
            testController.expectedInput = "";
            passwordWindow.submit();
            verify(passwordWindow.pending);
            compare(passwordWindow.fieldItem.text,"");
            compare(passwordWindow.fieldItem.echoMode,TextInput.Password);
        }
        function test_utf8_limit_and_cancel() {
            request("limit",false);
            passwordWindow.fieldItem.text = "文".repeat(400);
            verify(passwordWindow.tooLong);
            verify(!findChild(passwordWindow,"passwordSubmit").enabled);
            passwordWindow.submit();
            compare(testController.submittedCount,0);
            compare(passwordWindow.utf8Size("🙂"),4);
            compare(passwordWindow.utf8Size("a\u0000文"),5);
            mouseClick(findChild(passwordWindow,"passwordCancel"));
            compare(testController.cancelledCount,1);
            compare(testController.cancelledTask,"task-1");
            compare(passwordWindow.fieldItem.text,"");
            compare(passwordWindow.fieldItem.echoMode,TextInput.Password);
        }
        function test_system_close_cancels_current_task() {
            request("close",false);
            passwordWindow.fieldItem.text = "synthetic input";
            passwordWindow.close();
            compare(testController.cancelledCount,1);
            compare(passwordWindow.fieldItem.text,"");
        }
        function test_small_screen_keeps_footer() {
            Ui.Theme.fontScale = 1.5;
            passwordWindow.availableClientHeight = 220;
            testController.passwordPrompt = {taskId:"task-1",promptId:"small",name:"A long synthetic entry name / ".repeat(30),retry:true};
            tryCompare(passwordWindow,"visible",true);
            verify(waitForRendering(passwordWindow.contentItem));
            compare(passwordWindow.height,220);
            const flick = passwordWindow.bodyItem.contentItem;
            verify(flick.contentHeight > flick.height);
            const footer = passwordWindow.footerItem;
            verify(footer.y >= 0);
            verify(Math.abs(footer.y + footer.height - passwordWindow.height) <= 1);
            grabImage(passwordWindow.contentItem).save("password-small-screen.png");
        }
        function test_z_geometry_data() {
            const rows = [];
            for (const dark of [false,true])
                for (const scale of [1,1.25,1.5])
                    for (const english of [false,true])
                        rows.push({tag:dark+"-"+scale+"-"+english,dark,scale,english});
            return rows;
        }
        function test_z_geometry(data) {
            Ui.Theme.dark = data.dark;
            Ui.Theme.fontScale = data.scale;
            request(data.tag,true);
            const labels = {
                passwordHeading:["输入密码以继续解压","Enter a password to continue"],
                passwordDescription:["此文件已加密，需要密码才能继续。","This file is encrypted. Enter its password to continue."],
                passwordPrivacy:["密码仅用于当前任务，不会保存。关闭此窗口将取消任务。","Your password is used only for this task and is not saved. Closing this window cancels the task."],
                passwordLabel:["密码","Password"],
                passwordError:["密码不正确，请重试。","Incorrect password. Try again."],
                passwordVisibility:["显示密码","Show password"],
                passwordCancel:["取消任务","Cancel task"],
                passwordSubmit:["继续解压","Continue extraction"]
            };
            for (const name in labels) findChild(passwordWindow,name).text = labels[name][data.english ? 1 : 0];
            passwordWindow.fieldItem.placeholderText = data.english ? "Enter the archive password" : "输入此压缩包的密码";
            verify(waitForRendering(passwordWindow.contentItem));
            compare(passwordWindow.width,491);
            compare(passwordWindow.headingItem.font.pixelSize,Math.round(20 * data.scale));
            compare(passwordWindow.fieldItem.font.pixelSize,Math.round(13 * data.scale));
            compare(passwordWindow.modality,Qt.ApplicationModal);
            verify(passwordWindow.height <= Math.min(600,Screen.desktopAvailableHeight - 80));
            const footer = passwordWindow.footerItem;
            // A rendered frame need not include the layout polish queued by a
            // native resize notification after language or font changes.
            tryVerify(() => Math.abs(footer.y + footer.height - passwordWindow.height) <= 1, 5000,
                      "Footer must settle at the client bottom: " + JSON.stringify({
                          tag:data.tag,windowHeight:passwordWindow.height,
                          requestedHeight:passwordWindow.requestedHeight,
                          footerY:footer.y,footerHeight:footer.height,
                          bodyHeight:passwordWindow.bodyItem.height}));
            let top = -1;
            for (const name of ["passwordCancel","passwordSubmit"]) {
                const button = findChild(passwordWindow,name), point = button.mapToItem(passwordWindow.contentItem,0,0);
                if (top < 0) top = point.y;
                verify(Math.abs(top - point.y) <= 1);
                verify(point.x >= 0 && point.x + button.width <= passwordWindow.width + 1);
                verify(button.contentItem.paintedHeight <= button.availableHeight + 1);
                verify(button.contentItem.paintedWidth <= button.availableWidth + 1);
                compare(button.contentItem.font.pixelSize,Math.round(13 * data.scale));
                compare(button.background.radius,5);
            }
            console.info("PASSWORD_GEOMETRY " + JSON.stringify({dark:data.dark,english:data.english,scale:data.scale,dpr:Screen.devicePixelRatio,width:passwordWindow.width,height:passwordWindow.height,footerHeight:footer.height}));
            grabImage(passwordWindow.contentItem).save("password-"+data.tag+".png");
        }
    }
}
