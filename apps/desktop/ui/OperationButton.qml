import QtQuick

// Shared operation action role. Windows supply intent and state, not geometry.
AppButton {
    labelSize: Theme.operationFooterFontSize
    baseHeight: Theme.operationControlBodyButtonHeight
    horizontalInset: Theme.operationControlHorizontalInset
    verticalInset: Theme.operationControlVerticalInset
    cornerRadius: Theme.operationControlRadius
    surfaceColor: Theme.operationButtonSurface
    outlineColor: Theme.operationFieldBorder
}
