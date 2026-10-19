import QtQuick
import QtQuick.Controls

// Preserve the same checked-state transition and intent signals for every input path.
CheckBox {
    id: root
    Accessible.name: text
    function requestToggle() {
        if (enabled) root.click();
    }
    Accessible.onPressAction: root.requestToggle()
    Accessible.onToggleAction: root.requestToggle()
}
