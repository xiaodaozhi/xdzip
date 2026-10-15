pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

ComboBox {
    id: root
    focusPolicy: Qt.StrongFocus
    property real density: 1
    property bool numericOnly: false
    property string editPlaceholder: qsTrId("ui.2449d5ec999a")
    implicitHeight: Math.max(Theme.operationFieldHeight * density, contentItem.implicitHeight + 16 * density)
    font.pixelSize: Math.round(Theme.operationBodySize * density)
    leftPadding: Theme.operationControlFieldTextInset
    rightPadding: Theme.operationControlSelectIndicatorSize + Theme.operationControlSelectIndicatorRightInset + Theme.operationControlSelectIndicatorTextGap
    contentItem: TextInput {
        objectName: root.objectName + "Editor"
        enabled: root.editable
        text: root.editable ? root.editText : root.displayText
        readOnly: !root.editable
        selectByMouse: root.editable
        validator: root.numericOnly ? digits : null
        maximumLength: root.numericOnly ? 20 : 32767
        font: root.font
        color: root.enabled ? Theme.text : Theme.textSecondary
        verticalAlignment: Text.AlignVCenter
        clip: true
        onTextEdited: root.editText = text
        Text {
            anchors.fill: parent
            visible: root.editable && !parent.text.length
            text: root.editPlaceholder
            font: root.font
            color: Theme.textSecondary
            verticalAlignment: Text.AlignVCenter
        }
    }
    RegularExpressionValidator { id: digits; regularExpression: /[0-9]*/ }
    indicator: Image {
        objectName: root.objectName + "Indicator"
        x: root.width - width - Theme.operationControlSelectIndicatorRightInset
        y: (root.height - height) / 2
        width: Theme.operationControlSelectIndicatorSize
        height: width
        opacity: root.enabled ? 1 : 0.45
        source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "chevron-down.png"
    }
    background: OperationFrame {
        objectName: root.objectName + "Background"
        control: root
        pressed: root.down
        hovered: root.hovered
    }
    delegate: CommandMenuItem {
        required property var modelData
        required property int index
        width: root.popup.availableWidth
        labelSize: Math.round(Theme.operationBodySize * root.density)
        text: String(modelData)
        checkable: true
        checked: root.currentIndex === index
        highlighted: choices.currentIndex === index
        onTriggered: {
            root.currentIndex = index;
            root.popup.close();
        }
    }
    popup: CommandMenu {
        // Keep the normal gap on either side of the field, including short windows.
        y: {
            const expectedHeight = Math.min(Math.max(root.count * Theme.menuRowHeight, contentItem.implicitHeight) + padding * 2, root.Window.height - 40);
            const fieldTop = root.mapToItem(null, 0, 0).y;
            return fieldTop + root.height + Theme.mainPopupGap + expectedHeight > root.Window.height - Theme.menuViewportInset ? -expectedHeight - Theme.mainPopupGap : root.height + Theme.mainPopupGap;
        }
        width: root.width
        implicitHeight: Math.min(contentItem.implicitHeight + topPadding + bottomPadding, root.Window.height - 40)
        onOpened: {
            choices.currentIndex = root.currentIndex;
            choices.positionViewAtIndex(root.currentIndex, ListView.Contain);
            choices.forceActiveFocus(Qt.PopupFocusReason);
        }
        contentItem: ListView {
            id: choices
            focus: true
            clip: true
            implicitHeight: contentHeight
            model: root.popup.visible ? root.delegateModel : null
            currentIndex: root.currentIndex
            // These delegates come from ComboBox.delegateModel, not Menu's
            // contentModel. Menu's item count is therefore empty: navigate this
            // actual list explicitly instead of relying on native Menu indices.
            Keys.onPressed: function (event) {
                let next = currentIndex;
                if (event.key === Qt.Key_Home)
                    next = 0;
                else if (event.key === Qt.Key_End)
                    next = count - 1;
                else if (event.key === Qt.Key_Down)
                    next = Math.min(count - 1, currentIndex + 1);
                else if (event.key === Qt.Key_Up)
                    next = Math.max(0, currentIndex - 1);
                else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                    if (currentIndex >= 0)
                        root.currentIndex = currentIndex;
                    root.popup.close();
                } else if (event.key === Qt.Key_Escape)
                    root.popup.close();
                else
                    return;
                if (next >= 0 && next < count) {
                    currentIndex = next;
                    positionViewAtIndex(next, ListView.Contain);
                }
                event.accepted = true;
            }
            ScrollIndicator.vertical: ScrollIndicator {}
        }
    }
}
