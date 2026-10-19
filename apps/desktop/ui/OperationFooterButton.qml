import QtQuick

// Footer density is already normal; never multiply it by a window's density.
OperationButton {
    baseHeight: Theme.operationControlFooterButtonHeight
    minimumWidth: primary ? Theme.operationControlFooterPrimaryMinWidth : Theme.operationControlFooterButtonMinWidth
}
