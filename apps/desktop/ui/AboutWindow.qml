pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Xdzip.Settings 1.0

OperationWindow {
    id: root
    objectName: "aboutWindow"
    title: qsTrId("about.title")
    property bool copied: false
    width: Theme.aboutLogicalWidth
    requestedHeight: Math.min(Theme.aboutMaximumHeight, Screen.desktopAvailableHeight - 80,
                              body.implicitHeight + footer.implicitHeight)
    onVisibleChanged: if (visible) copied = false
    onActiveFocusItemChanged: if (visible) Qt.callLater(() => root.revealFocus(scroll))
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        ScrollView {
            id: scroll
            objectName: "aboutScroll"
            onHeightChanged: if (root.visible) Qt.callLater(() => root.revealFocus(scroll))
            onContentHeightChanged: if (root.visible) Qt.callLater(() => root.revealFocus(scroll))
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentWidth: availableWidth
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
            ColumnLayout {
                id: body
                width: scroll.availableWidth
                spacing: Theme.aboutSectionGap
                Item { Layout.preferredHeight: Theme.aboutContentInset - Theme.aboutSectionGap }
                Image {
                    source: "qrc:/xdzip/assets/app-mark.png"
                    Layout.alignment: Qt.AlignHCenter
                    Layout.preferredWidth: Theme.aboutArtworkSize
                    Layout.preferredHeight: Theme.aboutArtworkSize
                    fillMode: Image.PreserveAspectFit
                    Accessible.ignored: true
                }
                Text { text: "xdzip"; Layout.alignment: Qt.AlignHCenter; font.pixelSize: Theme.aboutTitleSize * Theme.fontScale; font.bold: true; color: Theme.text }
                Text {
                    Layout.leftMargin: Theme.aboutContentInset; Layout.rightMargin: Theme.aboutContentInset; Layout.fillWidth: true
                    text: qsTrId("about.description"); horizontalAlignment: Text.AlignHCenter
                    font.pixelSize: Theme.aboutBodySize * Theme.fontScale; color: Theme.textSecondary; wrapMode: Text.Wrap
                }
                GridLayout {
                    Layout.leftMargin: Theme.aboutContentInset; Layout.rightMargin: Theme.aboutContentInset; Layout.fillWidth: true
                    columns: 2; columnSpacing: Theme.aboutSectionGap; rowSpacing: Theme.aboutRowGap
                    Repeater {
                        model: [qsTrId("about.version"), BuildInfo.version,
                                qsTrId("about.build"), String(BuildInfo.buildNumber),
                                qsTrId("about.revision"), BuildInfo.revision === "unknown" ? qsTrId("about.unknown") : BuildInfo.revision,
                                qsTrId("about.runtime"), "Qt " + BuildInfo.qtVersion + ", " + BuildInfo.platform + ", " + BuildInfo.architecture]
                        delegate: Text {
                            required property string modelData
                            required property int index
                            objectName: "aboutValue-" + index
                            Layout.fillWidth: index % 2 === 1
                            Layout.alignment: Qt.AlignTop
                            text: modelData
                            font.pixelSize: Theme.aboutBodySize * Theme.fontScale
                            color: index % 2 === 0 ? Theme.textSecondary : Theme.text
                            wrapMode: Text.WrapAnywhere
                        }
                    }
                }
                Text {
                    Layout.leftMargin: Theme.aboutContentInset; Layout.rightMargin: Theme.aboutContentInset; Layout.fillWidth: true
                    visible: BuildInfo.channel !== "stable" || BuildInfo.dirty
                    text: (BuildInfo.channel !== "stable" ? qsTrId("about.development") : "") + (BuildInfo.dirty ? "\n" + qsTrId("about.modified") : "")
                    color: Theme.textSecondary; font.pixelSize: Theme.aboutNoteSize * Theme.fontScale; wrapMode: Text.Wrap
                }
                Text {
                    Layout.leftMargin: Theme.aboutContentInset; Layout.rightMargin: Theme.aboutContentInset; Layout.fillWidth: true
                    text: qsTrId("about.license"); color: Theme.textSecondary
                    font.pixelSize: Theme.aboutNoteSize * Theme.fontScale; wrapMode: Text.Wrap
                }
                OperationButton {
                    objectName: "aboutHomepage"
                    Layout.alignment: Qt.AlignHCenter
                    text: qsTrId("about.homepage")
                    onClicked: Qt.openUrlExternally("https://github.com/xiaodaozhi/xdzip")
                }
                Item { Layout.preferredHeight: Theme.aboutContentInset - Theme.aboutSectionGap }
            }
        }
        OperationFooter {
            id: footer
            objectName: "aboutFooter"
            Layout.fillWidth: true
            OperationFooterButton {
                objectName: "aboutCopy"
                text: root.copied ? qsTrId("about.copied") : qsTrId("about.copy")
                onClicked: { BuildInfo.copyVersionInfo(); root.copied = true; }
            }
            OperationFooterButton { objectName: "aboutDone"; text: qsTrId("about.done"); onClicked: root.close() }
        }
    }
}
