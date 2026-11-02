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
    Component { id: settingsComponent; Ui.SettingsWindow { owner: null } }
    Component { id: fieldComponent; Ui.OperationField { width: 240; text: "abcdef" } }
    Component { id: editableComponent; Ui.OperationSelect { width: 240; editable: true; model: ["abcdef"]; editText: "abcdef" } }
    Component {
        id: toolbarComponent
        Ui.ArchiveToolbar { width: 640; canOpen: true; canSearch: true; canInspect: false; hasSelection: false }
    }
    TestCase {
        name: "SettingsControls"
        when: windowShown
        function cleanup() { select.popup.close(); Ui.Theme.fontScale = 1; Ui.Theme.dark = false; select.model = ["标准", "English (US)"]; }
        function test_shared_checkboxes_data() {
            const rows = [];
            for (const dark of [false, true])
                for (const scale of [1, 1.25, 1.5])
                    rows.push({tag: dark + "-" + scale, dark, scale});
            return rows;
        }
        function test_shared_checkboxes(data) {
            const settings = createTemporaryObject(settingsComponent, surface);
            verify(settings !== null);
            Ui.Theme.dark = data.dark;
            Ui.Theme.fontScale = data.scale;
            for (const name of ["Tray", "Recent"]) {
                const control = findChild(settings.contentItem, "settings" + name);
                verify(control !== null);
                compare(control.indicator.objectName, control.objectName + "Indicator");
                compare(control.indicatorSize, Ui.Theme.operationControlIndicatorSize * control.density);
                const label = findChild(control, control.objectName + "Label");
                verify(label !== null);
                compare(label.font.pixelSize, Math.round(Ui.Theme.settingsBodySize * data.scale));
                const draft = "draft" + name;
                const before = settings[draft];
                control.requestToggle();
                compare(settings[draft], !before);
                control.requestToggle();
                compare(settings[draft], before);
            }
        }
        function test_settings_select_heights_data() {
            const rows = [];
            for (const dark of [false, true])
                for (const scale of [1, 1.25, 1.5])
                    rows.push({tag: dark + "-" + scale, dark, scale});
            return rows;
        }
        function test_settings_select_heights(data) {
            Ui.Theme.dark = data.dark;
            Ui.Theme.fontScale = data.scale;
            const settings = createTemporaryObject(settingsComponent, surface);
            verify(settings !== null);
            settings.show();
            verify(waitForRendering(settings.contentItem));
            const names = ["settingsDensity", "settingsTextScale", "settingsLanguage", "settingsOpenMode"];
            const controls = names.map(name => findChild(settings.contentItem, name));
            for (let index = 0; index < controls.length; ++index) {
                const control = controls[index];
                verify(control !== null, names[index]);
                verify(control.height > 0, names[index]);
                compare(control.height, controls[0].height, names[index] + " logical height");
                verify(control.contentItem.implicitHeight <= control.height, names[index] + " text fits");
                verify(control.indicator.y >= 0 && control.indicator.y + control.indicator.height <= control.height,
                       names[index] + " indicator fits");
            }
            settings.close();
        }
        function test_input_selection_colors_data() {
            return [{tag:"light",dark:false}, {tag:"dark",dark:true}];
        }
        function test_input_selection_colors(data) {
            Ui.Theme.dark = data.dark;
            const field = createTemporaryObject(fieldComponent, surface, {x: 10, y: 100});
            const combo = createTemporaryObject(editableComponent, surface, {x: 10, y: 150});
            // Keep the search field inside the captured physical-pixel region at DPR 2.
            const toolbar = createTemporaryObject(toolbarComponent, surface, {x: -300, y: 200});
            verify(field && combo && toolbar);
            const editor = combo.contentItem;
            const search = findChild(toolbar, "archiveSearch");
            verify(editor && search);
            search.text = "abcdef";
            for (const input of [field, editor, search]) {
                input.forceActiveFocus();
                input.select(1, 4);
                compare(input.selectedText, "bcd");
                compare(input.selectedTextColor.toString(), Ui.Theme.accentText.toString());
                compare(input.selectionColor.toString(), Ui.Theme.accent.toString());
                verify(input.selectByMouse);
                const start = input.positionToRectangle(1);
                const end = input.positionToRectangle(4);
                const inset = input.leftPadding || 0;
                mousePress(input, start.x + inset + 1, input.height / 2, Qt.LeftButton);
                mouseMove(input, end.x + inset + 1, input.height / 2, 20);
                mouseRelease(input, end.x + inset + 1, input.height / 2, Qt.LeftButton);
                verify(input.selectedText.length > 0, "Pointer drag selects text: " + input.objectName +
                       " start=" + start.x + " end=" + end.x + " width=" + input.width);
                keyClick(Qt.Key_A, Qt.ControlModifier);
                compare(input.selectedText, "abcdef");
                // QtTest captures physical pixels but clips to the logical item bounds.
                // Grab the shared fixture and convert the input rectangle at its real DPR.
                verify(waitForRendering(surface));
                const image = grabImage(surface);
                const origin = input.mapToItem(surface, 0, 0);
                const pixelScale = Screen.devicePixelRatio;
                const left = Math.max(1, Math.floor((origin.x + input.positionToRectangle(0).x + inset) * pixelScale));
                const right = Math.min(image.width - 2, Math.ceil((origin.x + input.positionToRectangle(6).x + inset) * pixelScale));
                const top = Math.max(0, Math.floor(origin.y * pixelScale));
                const bottom = Math.min(image.height - 1, Math.ceil((origin.y + input.height) * pixelScale));
                const accent = Ui.Theme.accent;
                function isAccent(x, y) {
                    return Math.abs(image.red(x, y) - Math.round(accent.r * 255)) < 5 &&
                           Math.abs(image.green(x, y) - Math.round(accent.g * 255)) < 5 &&
                           Math.abs(image.blue(x, y) - Math.round(accent.b * 255)) < 5;
                }
                let backgroundPixels = 0;
                let whiteGlyphPixels = 0;
                for (let y = top; y <= bottom; ++y) {
                    for (let x = left; x <= right; ++x) {
                        if (isAccent(x, y)) ++backgroundPixels;
                        if (image.red(x, y) > 185 && image.green(x, y) > 185 && image.blue(x, y) > 185 &&
                            (isAccent(x - 1, y) || isAccent(x + 1, y))) ++whiteGlyphPixels;
                    }
                }
                verify(backgroundPixels > 10, "Selected background was painted: " + input.objectName +
                       " left=" + left + " right=" + right + " size=" + image.width + "x" + image.height +
                       " inset=" + inset + " selected=" + input.selectedText);
                verify(whiteGlyphPixels > 0, "Selected white text was painted: " + input.objectName);
                keyClick(Qt.Key_X);
                compare(input.text, "x");
            }
            field.text = "abcdef";
            field.echoMode = TextInput.Password;
            field.selectAll();
            compare(field.text, "abcdef");
            field.echoMode = TextInput.Normal;
            compare(field.text, "abcdef");
        }
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
