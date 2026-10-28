// apps/desktop/tests/tst_commandcontrols.qml

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
        // This fixture supplies its own tooltip below; do not race two popups on one owner.
        tooltipEnabled: false
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
            disabledReason: "请选择至少一个条目"
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
    Ui.CommandMenu {
        id: layoutMenu
        parent: trigger
        Ui.CommandMenuItem {
            id: layoutCommand
            objectName: "layoutCommand"
            enabled: false
            disabledReason: "Accessible description only"
        }
        Ui.CommandMenu {
            id: layoutSubmenu
            Ui.CommandMenuItem { text: "Child" }
        }
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
        function containsVisibleText(item, text) {
            if (item.visible && item.text === text)
                return true;
            for (var i = 0; i < item.children.length; ++i) {
                if (containsVisibleText(item.children[i], text))
                    return true;
            }
            return false;
        }
        function init() {
            surface.Window.window.requestActivate();
            tryCompare(surface.Window.window, "active", true);
            // Keep native pointer hover from choosing an item during keyboard navigation.
            mouseMove(surface, surface.width - 2, surface.height - 2);
            trigger.forceActiveFocus();
        }
        function openWithKeyboard() {
            trigger.forceActiveFocus();
            keyClick(Qt.Key_Space);
            tryCompare(menu, "visible", true);
        }
        function cleanup() {
            tooltip.requested = false;
            tooltip.suppressed = false;
            tooltip.needsFreshPointer = false;
            menu.close();
            layoutMenu.close();
            tryCompare(menu, "visible", false);
            firstSpy.clear();
            disabledSpy.clear();
            Ui.Theme.fontScale = 1;
            Ui.Theme.dark = false;
        }
        function verifyCommandGeometry(item) {
            var content = item.contentItem;
            var label = findChild(item, item.objectName + "Label");
            verify(label && label.visible);
            compare(label.height, label.implicitHeight, "Text must use its current font height");
            compare(label.font.pixelSize, Math.round(Ui.Theme.menuLabelSize));
            verify(Math.abs(content.y + label.y + label.height / 2 - item.height / 2) <= 0.5);
            verify(item.height >= Ui.Theme.menuRowHeight);
            var previous = content.children[0];
            compare(previous.width, Ui.Theme.menuIconSize);
            for (var i = 1; i < content.children.length; ++i) {
                var child = content.children[i];
                if (!child.visible)
                    continue;
                verify(Math.abs(child.y + child.height / 2 - content.height / 2) <= 0.5);
                verify(Math.abs(child.x - previous.x - previous.width - content.gap) < 0.01);
                previous = child;
            }
            verify(label.width >= 0);
            verify(Math.abs(previous.x + previous.width - content.width) < 0.01);
            var tags = [
                { suffix: "Sequence", text: item.sequenceLabel },
                { suffix: "Status", text: item.statusLabel }
            ];
            for (var j = 0; j < tags.length; ++j) {
                var tag = tags[j];
                var badge = findChild(item, item.objectName + tag.suffix);
                verify(badge);
                compare(badge.visible, tag.text.length > 0);
                compare(badge.text, tag.text);
                var badgeText = findChild(badge, badge.objectName + "Text");
                verify(badgeText);
                compare(badgeText.text, tag.text);
                compare(badgeText.width, badgeText.implicitWidth);
                compare(badgeText.height, badgeText.implicitHeight);
                compare(badgeText.font.pixelSize, Math.round(Ui.Theme.menuShortcutSize));
                compare(badge.width, badgeText.implicitWidth + Ui.Theme.menuTagHorizontalInset * 2);
                compare(badge.height, badgeText.implicitHeight + Ui.Theme.menuTagVerticalInset * 2);
                verify(Math.abs(badgeText.x + badgeText.width / 2 - badge.width / 2) <= 0.5,
                       tag.suffix + " text x/width/tagWidth=" + [badgeText.x, badgeText.width, badge.width]);
                verify(Math.abs(badgeText.y + badgeText.height / 2 - badge.height / 2) <= 0.5,
                       tag.suffix + " text y/height/tagHeight=" + [badgeText.y, badgeText.height, badge.height]);
            }
            verify(!containsVisibleText(item, item.disabledReason));
        }
        function test_first_render_after_font_and_slot_changes_data() {
            return [
                { tag: "zh-light", title: "收起子目录", dark: false },
                { tag: "zh-dark", title: "收起子目录", dark: true },
                { tag: "en-light", title: "Collapse subfolders", dark: false },
                { tag: "en-dark", title: "Collapse subfolders", dark: true }
            ];
        }
        function test_first_render_after_font_and_slot_changes(data) {
            Ui.Theme.dark = data.dark;
            layoutCommand.text = data.title;
            layoutSubmenu.title = data.title;
            var submenuCommand = layoutMenu.itemAt(1);
            verify(submenuCommand && submenuCommand.subMenu === layoutSubmenu);
            var scales = [1, 1.25, 1.5];
            for (var slots = 0; slots < 8; ++slots) {
                layoutCommand.statusLabel = submenuCommand.statusLabel = (slots & 1) ? "Done" : "";
                layoutCommand.shortcutLabel = submenuCommand.shortcutLabel = (slots & 2) ? "Ctrl+A" : "";
                layoutCommand.sequenceLabel = submenuCommand.sequenceLabel = (slots & 4) ? "42" : "";
                for (var i = 0; i < scales.length; ++i) {
                    layoutMenu.close();
                    compare(layoutMenu.visible, false);
                    Ui.Theme.fontScale = scales[i];
                    // Hidden popup inheritance must not remove logical trailing slots.
                    compare(layoutCommand.contentItem.hasStatus, Boolean(slots & 1));
                    compare(layoutCommand.contentItem.hasShortcut, Boolean(slots & 2));
                    compare(layoutCommand.contentItem.hasSequence, Boolean(slots & 4));
                    compare(submenuCommand.contentItem.hasStatus, Boolean(slots & 1));
                    compare(submenuCommand.contentItem.hasShortcut, Boolean(slots & 2));
                    compare(submenuCommand.contentItem.hasSequence, Boolean(slots & 4));
                    compare(submenuCommand.contentItem.hasSubmenu, true);
                    layoutMenu.popup();
                    verify(waitForRendering(layoutCommand));
                    // No geometry retry: check the first observed rendering after reopen.
                    verifyCommandGeometry(layoutCommand);
                    verifyCommandGeometry(submenuCommand);
                    compare(layoutCommand.Accessible.description, layoutCommand.disabledReason);
                    // Also leave a different value for the next closed-popup transition.
                    var openedScale = scales[(i + 2) % scales.length];
                    verify(Ui.Theme.fontScale !== openedScale);
                    Ui.Theme.fontScale = openedScale;
                    verify(waitForRendering(layoutCommand));
                    verifyCommandGeometry(layoutCommand);
                    verifyCommandGeometry(submenuCommand);
                }
            }
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
            tryCompare(trigger, "activeFocus", false);
            mouseClick(trigger);
            tryCompare(menu, "visible", true);
            mouseClick(disabled);
            compare(disabledSpy.count, 0);
            verify(menu.visible);
            keyClick(Qt.Key_Escape);
            tryCompare(menu, "visible", false);
            tryCompare(trigger, "activeFocus", false);
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
            var label = findChild(disabled, disabled.objectName + "Label");
            verify(label && label.visible);
            compare(label.text, disabled.text);
            verify(!containsVisibleText(disabled, disabled.disabledReason));
            compare(first.font.pixelSize, Ui.Theme.menuLabelSize);
            verify(menu.contentItem.height > 0);
            verify(menu.contentItem.height <= surface.height);
            verify(Math.abs(Ui.Theme.menuShadow.a - (data.dark ? 102 : 41) / 255) < 0.01);
        }
        function test_tooltip_waits_for_pointer_rest_and_restarts_after_leaving() {
            tooltip.requested = Qt.binding(function() { return trigger.hovered; });
            mouseMove(trigger, 10, 10);
            wait(700);
            verify(!tooltip.visible, "No immediate tooltip on entry");
            mouseMove(trigger, 20, 10);
            wait(700);
            verify(!tooltip.visible, "Motion must restart the full dwell period");
            tryCompare(tooltip, "visible", true, 3000);
            verify(tooltip.y >= trigger.height, "Near the top edge the tooltip must open below its owner");
            mouseMove(trigger, 25, 10);
            tryCompare(tooltip, "visible", false);
            mouseMove(surface, surface.width - 2, surface.height - 2);
            mouseMove(trigger, 10, 10);
            wait(700);
            verify(!tooltip.visible, "Re-entry must not reuse an earlier tooltip timer");
            mouseMove(surface, surface.width - 2, surface.height - 2);
            wait(600);
            verify(!tooltip.visible, "An expired cancelled timer must not reopen a tooltip");
        }
        function test_tooltip_stays_hidden_while_pressed() {
            tooltip.requested = Qt.binding(function() { return trigger.hovered; });
            mouseMove(trigger, 10, 10);
            tryCompare(tooltip, "visible", true, 3000);
            mousePress(trigger, 10, 10);
            try {
                tryCompare(tooltip, "visible", false);
                wait(1300);
                verify(!tooltip.visible, "Holding the pointer down must not reopen the tooltip");
            } finally {
                mouseRelease(trigger, 10, 10);
            }
            tryCompare(menu, "visible", true);
            verify(!tooltip.visible);
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
        function test_drag_suppression_cancels_a_stale_source_hover_timer() {
            tooltip.requested = true;
            tryCompare(tooltip,"visible",true,2500);
            tooltip.suppressed = true;
            tryCompare(tooltip,"visible",false);
            wait(1300);
            verify(!tooltip.visible,"A stale source hover cannot show a tooltip during drag");
            tooltip.suppressed = false;
            wait(1300);
            verify(!tooltip.visible,"A cached source hover remains invalid after native drag ends");
            mouseMove(trigger,25,10);
            wait(700);
            verify(!tooltip.visible,"Ending a drag requires a fresh full dwell");
            tryCompare(tooltip,"visible",true,1800);
        }
    }
}
