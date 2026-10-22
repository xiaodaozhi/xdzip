// apps/desktop/ui/BrowserLayout.qml

pragma Singleton
import QtQuick

QtObject {
    property var columnWidths: [0, 0, 0, 0, 0]
    function resizeColumn(column: int, width: real) {
        if (column < 0 || column >= columnWidths.length || !isFinite(width))
            return;
        const widths = columnWidths.slice();
        widths[column] = Math.max(0, width);
        columnWidths = widths;
    }
    function resetColumn(column: int) {
        resizeColumn(column, 0);
    }
}
