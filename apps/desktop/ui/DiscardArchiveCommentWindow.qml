pragma ComponentBehavior: Bound
import QtQuick

OperationConfirmationWindow {
    id: root
    objectName: "discardArchiveCommentWindow"
    signal confirmed()
    title: qsTrId("archive.comment.discardTitle")
    heading: title
    message: qsTrId("archive.comment.discardMessage")
    function confirm() {
        present();
        keep.forceActiveFocus();
    }
    OperationFooterButton {
        id: keep
        objectName: "keepArchiveComment"
        text: qsTrId("archive.comment.keep")
        onClicked: root.close()
    }
    OperationFooterButton {
        objectName: "discardArchiveComment"
        primary: true
        text: qsTrId("archive.comment.discard")
        onClicked: {
            root.close();
            root.confirmed();
        }
    }
}
