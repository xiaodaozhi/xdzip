import QtQuick

// Read-only presentation. A future CRC task adapter supplies the same progress
// data as other tasks; this component never opens files or starts a codec.
TaskWindow {
    objectName: "integrityWindow"
    operationKind: "check"
}
