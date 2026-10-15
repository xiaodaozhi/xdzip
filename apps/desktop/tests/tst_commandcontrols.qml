import QtQuick
import QtTest
import "../ui" as Ui

Item {
    id: surface
    width: 1000
    height: 700
    Ui.ArchiveAction {
        id: trigger
        text: "更多操作"
        x: 20
        y: 20
        onClicked: menu.popup()
    }
    Ui.CommandMenu {
        id: menu
        parent: trigger
        Ui.CommandMenuItem {
            id: first
            text: "查看属性"
            asset: "file-text"
        }
        Ui.CommandMenuItem {
            id: disabled
            text: "解压所选"
            enabled: false
            disabledReason: "写入能力尚未接入"
        }
        Ui.CommandMenuSeparator {}
        Ui.CommandMenuItem {
            id: check
            text: "显示详情"
            checkable: true
        }
        Ui.CommandMenu {
            id: submenu
            title: "排序方式"
            Ui.CommandMenuItem {
                id: sort
                text: "名称"
                checkable: true
                checked: true
            }
        }
    }
    Ui.CommandTooltip {
        id: tooltip
        parent: trigger
        text: "<b>原始文件名</b>"
    }
    SignalSpy {
        id: firstSpy
        target: first
        signalName: "triggered"
    }
    SignalSpy {
        id: disabledSpy
        target: disabled
        signalName: "triggered"
    }
    TestCase {
        name: "CommandControls"
        when: windowShown
        function init() {
            surface.Window.window.requestActivate();
            tryCompare(surface.Window.window, "active", true);
            // Keep native pointer hover from choosing an item during keyboard navigation.
            mouseMove(surface, surface.width - 2, surface.height - 2);
        }
        function openWithKeyboard() {
            trigger.forceActiveFocus();
            keyClick(Qt.Key_Space);
            tryCompare(menu, "visible", true);
        }
        function cleanup() {
            tooltip.requested = false;
            menu.close();
            firstSpy.clear();
            disabledSpy.clear();
            Ui.Theme.fontScale = 1;
            Ui.Theme.dark = false;
        }
        function test_keyboard_selection_and_disabled_guard() {
            openWithKeyboard();
            keyClick(Qt.Key_Down);
            tryCompare(menu, "currentIndex", 0);
            keyClick(Qt.Key_Down);
            tryCompare(menu, "currentIndex", 3);
            keyClick(Qt.Key_Return);
            tryCompare(menu, "visible", false);
            compare(check.checked, true);
            verify(trigger.activeFocus);
            mouseClick(trigger);
            tryCompare(menu, "visible", true);
            mouseClick(disabled);
            compare(disabledSpy.count, 0);
            verify(menu.visible);
            keyClick(Qt.Key_Escape);
            tryCompare(menu, "visible", false);
            verify(trigger.activeFocus);
        }
        function test_submenu_return_and_activation() {
            openWithKeyboard();
            keyClick(Qt.Key_Up);
            tryCompare(menu, "currentIndex", 4);
            keyClick(Qt.Key_Right);
            tryCompare(submenu, "visible", true);
            keyClick(Qt.Key_Left);
            tryCompare(submenu, "visible", false);
            verify(menu.visible);
            keyClick(Qt.Key_Escape);
            openWithKeyboard();
            keyClick(Qt.Key_Down);
            keyClick(Qt.Key_Return);
            tryCompare(firstSpy, "count", 1);
        }
        function test_large_type_and_themes_data() {
            return [
                {
                    tag: "light",
                    scale: 1,
                    dark: false
                },
                {
                    tag: "dark",
                    scale: 1,
                    dark: true
                },
                {
                    tag: "large",
                    scale: 2,
                    dark: false
                }
            ];
        }
        function test_large_type_and_themes(data) {
            Ui.Theme.fontScale = data.scale;
            Ui.Theme.dark = data.dark;
            mouseClick(trigger);
            tryCompare(menu, "visible", true);
            tryCompare(menu, "width", Ui.Theme.menuWidth);
            verify(first.height >= Ui.Theme.menuRowHeight);
            verify(disabled.height >= first.height);
            compare(first.Accessible.description, "");
            compare(disabled.Accessible.description, disabled.disabledReason);
            compare(first.font.pixelSize, Ui.Theme.menuLabelSize);
            verify(menu.contentItem.height > 0);
            verify(menu.contentItem.height <= surface.height);
            verify(Math.abs(Ui.Theme.menuShadow.a - (data.dark ? 102 : 41) / 255) < 0.01);
        }
        function test_tooltip_is_plain_text_and_yields_to_menu() {
            tooltip.requested = true;
            tryCompare(tooltip, "visible", true);
            compare(tooltip.contentItem.textFormat, Text.PlainText);
            compare(tooltip.contentItem.text, "<b>原始文件名</b>");
            mouseClick(trigger);
            tryCompare(menu, "visible", true);
            tryCompare(tooltip, "visible", false);
        }
    }
}
