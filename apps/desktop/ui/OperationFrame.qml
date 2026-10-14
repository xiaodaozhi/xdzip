import QtQuick

Rectangle {
    // TextField derives from TextInput; ComboBox derives from Control. Both
    // expose the state used here, but do not share the Controls wrapper type.
    required property Item control
    property bool framed: true
    property bool pressed: false
    property bool hovered: false
    radius: Theme.operationControlRadius
    color: !framed ? "transparent" : control && !control.enabled ? Theme.operationCard : pressed ? Theme.controlPressed : hovered ? Theme.operationButtonSurface : Theme.surface
    border.color: !framed ? "transparent" : control && control.activeFocus ? Theme.accent : Theme.operationFieldBorder
    border.width: control && control.activeFocus ? 2 : 1
}
