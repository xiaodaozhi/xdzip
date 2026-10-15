pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Control {
    id: root
    required property var options
    property int currentIndex: 0
    property string accessibleLabel: ""
    implicitHeight: Math.max(Theme.operationControlSegmentedHeight, Theme.operationFooterFontSize * 1.45 + 8)
    implicitWidth: options.length * 72
    padding: 1
    Accessible.name: accessibleLabel
    function step(direction: int) {
        for (let offset = 1; offset <= options.length; ++offset) {
            const index = (currentIndex + direction * offset + options.length * 2) % options.length;
            if (options[index].enabled !== false) {
                currentIndex = index;
                segments.itemAt(index).forceActiveFocus();
                return;
            }
        }
    }
    background: Rectangle {
        objectName: root.objectName + "Track"
        radius: Theme.operationControlSegmentedRadius
        color: Theme.operationCard
        border.color: Theme.operationFieldBorder
    }
    ButtonGroup {
        id: exclusive
    }
    contentItem: RowLayout {
        spacing: 0
        Repeater {
            id: segments
            model: root.options
            AbstractButton {
                id: segment
                required property var modelData
                required property int index
                objectName: root.objectName + "-" + index
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.preferredWidth: 1
                hoverEnabled: true
                checkable: true
                checked: root.currentIndex === index
                enabled: modelData.enabled !== false
                focusPolicy: Qt.StrongFocus
                ButtonGroup.group: exclusive
                Accessible.role: Accessible.RadioButton
                Accessible.name: modelData.text
                Accessible.checked: checked
                onClicked: root.currentIndex = index
                Keys.onLeftPressed: root.step(-1)
                Keys.onRightPressed: root.step(1)
                CommandTooltip {
                    requested: segment.hovered && !segment.down
                    text: segment.modelData.reason || ""
                }
                contentItem: Text {
                    text: segment.modelData.text
                    font.pixelSize: Theme.operationFooterFontSize
                    color: !segment.enabled ? Theme.menuDisabled : segment.checked ? Theme.accentText : Theme.text
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
                background: Item {
                    id: segmentBackground
                    objectName: segment.objectName + "Background"
                    readonly property real outerRadius: Theme.operationControlSegmentedRadius - root.padding
                    property color color: segment.checked ? segment.down ? Theme.accentPressed : segment.hovered ? Theme.accentHover : Theme.accent : segment.hovered && segment.enabled ? Theme.controlHover : "transparent"
                    clip: true
                    // Extend the rounded fill past the inner edge, then clip the seam square.
                    // Uniform Rectangle radii render consistently on software and GPU backends.
                    Rectangle {
                        x: segment.index === root.options.length - 1 && segment.index !== 0 ? -segmentBackground.outerRadius : 0
                        width: parent.width + (root.options.length > 1 && (segment.index === 0 || segment.index === root.options.length - 1) ? segmentBackground.outerRadius : 0)
                        height: parent.height
                        radius: segment.index === 0 || segment.index === root.options.length - 1 ? segmentBackground.outerRadius : 0
                        antialiasing: true
                        color: segmentBackground.color
                    }
                    Rectangle {
                        visible: segment.index > 0 && !segment.checked && root.currentIndex !== segment.index - 1
                        x: 0
                        y: 0
                        width: 1
                        height: parent.height
                        color: Theme.operationFieldBorder
                    }
                }
            }
        }
    }
}
