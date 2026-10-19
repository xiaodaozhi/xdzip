import QtQuick
import QtQuick.Layouts

// Footer density is already normal; never multiply it by a window's density.
OperationButton {
    id: root
    // Use the same Text shaping as the visible label, without its constrained width.
    // Font-metric advances can differ from Text's wrapping threshold on native backends.
    implicitWidth: Math.max(minimumWidth, Math.ceil(naturalLabel.implicitWidth + horizontalInset * 2 * density))
    Text {
        id: naturalLabel
        visible: false
        text: root.text
        textFormat: Text.PlainText
        font: (root.contentItem as Text).font
        wrapMode: Text.NoWrap
    }
    wrapLabel: true
    Layout.fillWidth: true
    Layout.fillHeight: true
    Layout.minimumWidth: minimumWidth
    Layout.maximumWidth: implicitWidth
    baseHeight: Theme.operationControlFooterButtonHeight
    minimumWidth: primary ? Theme.operationControlFooterPrimaryMinWidth : Theme.operationControlFooterButtonMinWidth
}
