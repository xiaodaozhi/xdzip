import QtQuick
import QtQuick.Controls

// Checked state belongs to the selection model, not the accessibility adapter.
RadioButton {
    id: root
    // Consumers draw the approved cards/segments; keep only native radio behavior.
    indicator: null
    padding: 0
    spacing: 0
    signal selectionRequested()
    function requestSelection() {
        if (enabled) selectionRequested();
    }
    Accessible.role: Accessible.RadioButton
    Accessible.checkable: checkable
    Accessible.checked: checked
    Accessible.onPressAction: root.requestSelection()
    Accessible.onToggleAction: root.requestSelection()
    onClicked: root.requestSelection()
}
