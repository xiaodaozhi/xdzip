pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import Xdzip.Settings 1.0

GridLayout {
    id: root
    property string label: ""
    property string description: ""
    property var options: []
    property var optionFonts: []
    property int selectedIndex: 0
    property string controlName: ""
    signal selectionRequested(int index)
    readonly property real preferredSelectWidth: Math.max(160 * Theme.fontScale,
        select.widestOption + select.leftPadding + select.rightPadding + 4)
    readonly property real minimumTextWidth: {
        let widest = 0;
        const parts = [
            {text: root.label, metrics: labelMetrics},
            {text: root.description, metrics: descriptionMetrics}
        ];
        for (const part of parts)
            for (const word of part.text.split(/[\s\u2e80-\u9fff\uac00-\ud7af]+/))
                widest = Math.max(widest, part.metrics.advanceWidth(word));
        return Math.ceil(widest);
    }
    columns: width >= minimumTextWidth + preferredSelectWidth + columnSpacing ? 2 : 1
    columnSpacing: 16
    rowSpacing: 8
    Layout.minimumWidth: 0
    FontMetrics { id: labelMetrics; font: labelText.font }
    FontMetrics { id: descriptionMetrics; font: descriptionText.font }
    ColumnLayout {
        Layout.fillWidth: true
        Layout.minimumWidth: 0
        spacing: 4
        Text {
            id: labelText
            Layout.fillWidth: true
            text: root.label
            textFormat: Text.PlainText
            font.pixelSize: Math.round(Theme.settingsBodySize * Theme.fontScale)
            color: Theme.text
            wrapMode: Text.Wrap
        }
        Text {
            id: descriptionText
            visible: root.description.length > 0
            Layout.fillWidth: true
            text: root.description
            textFormat: Text.PlainText
            font.pixelSize: Math.round(Theme.settingsNoteSize * Theme.fontScale)
            color: Theme.textSecondary
            wrapMode: Text.Wrap
        }
    }
    OperationSelect {
        id: select
        objectName: root.controlName
        Accessible.name: root.label
        Layout.preferredWidth: Math.min(Math.max(80, root.width), root.preferredSelectWidth)
        Layout.minimumWidth: 80
        Layout.maximumWidth: Math.max(80, root.width)
        Layout.alignment: Qt.AlignRight
        density: Theme.settingsBodySize * Theme.fontScale / Theme.operationBodySize
        implicitHeight: Math.ceil(Math.max(Theme.operationControlBodyButtonHeight,
                                           UiFonts.maxUiLineHeight(font) + 8))
        model: root.options
        optionFonts: root.optionFonts
        currentIndex: root.selectedIndex
        onActivated: index => {
            root.selectionRequested(index);
            currentIndex = root.selectedIndex;
        }
        Connections {
            target: root
            function onSelectedIndexChanged() { select.currentIndex = root.selectedIndex; }
        }
    }
}
