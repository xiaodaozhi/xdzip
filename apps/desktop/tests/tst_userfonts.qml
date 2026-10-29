import QtQuick
import QtTest
import "../ui" as Ui

Item {
    id: surface
    width: 1024; height: 720

    Ui.ArchiveTabs {
        id: tabs
        width: parent.width
        archiveTitle: ""
        tabs: [{id: "archive", title: "文件稻草.zip", selected: true}]
    }
    Ui.Breadcrumb {
        id: breadcrumbs
        y: 60; width: 500; height: 32
        entries: [{id: "archive", label: "文件稻草.zip", current: false},
                  {id: "directory", label: "稻草 / 日本語", current: true}]
    }
    Ui.TaskPathCard {
        id: card
        y: 100; width: 480
        heading: "目标位置"; value: "C:/稻草/日本語/long-name.txt"
        iconSource: "qrc:/xdzip/assets/folder.png"
    }
    Ui.OperationHeader {
        id: header
        y: 250; width: 491; height: implicitHeight
        source: "qrc:/xdzip/assets/light/archive.png"
        title: "解压文件"; subtitle: "文件稻草.zip"; subtitleUserData: true
    }
    Ui.ArchiveDragPreview {
        id: preview
        y: 400; fileName: "文件稻草/日本語.txt"; itemCount: 1
    }
    Ui.CommandMenuItem {
        id: menuEntry
        objectName: "filenameMenu"
        text: "文件稻草.zip"; userData: true
    }

    TestCase {
        name: "UserDataFonts"; when: windowShown

        function test_primary_families_and_measurement() {
            verify(Ui.Theme.userDataFontFamily.length > 0);
            verify(Ui.Theme.uiFontFamily.length > 0);
            tryVerify(() => findChild(tabs, "archiveTabLabel-0") !== null);
            compare(findChild(tabs, "archiveTabLabel-0").font.family, Ui.Theme.userDataFontFamily);
            compare(findChild(breadcrumbs, "filePaneTitle").font.family, Ui.Theme.userDataFontFamily);
            compare(card.valueItem.font.family, Ui.Theme.userDataFontFamily);
            compare(header.subtitleItem.font.family, Ui.Theme.userDataFontFamily);
            compare(header.titleItem.font.family, Ui.Theme.uiFontFamily);
            compare(findChild(menuEntry, "filenameMenuLabel").font.family, Ui.Theme.userDataFontFamily);
            const label = findChild(preview, "dragPreviewName");
            const metrics = findChild(preview, "dragPreviewNameMetrics");
            compare(label.font.family, Ui.Theme.userDataFontFamily);
            compare(metrics.font.family, label.font.family);
            compare(metrics.font.pixelSize, label.font.pixelSize);
            compare(label.textFormat, Text.PlainText);
        }

        function test_static_labels_keep_ui_font() {
            tabs.tabs = [{id: "empty", title: "", selected: true}];
            tryVerify(() => findChild(tabs, "archiveTabLabel-0") !== null);
            tryVerify(() => findChild(tabs, "archiveTabLabel-0").font.family === Ui.Theme.uiFontFamily);
            header.subtitleUserData = false;
            compare(header.subtitleItem.font.family, Ui.Theme.uiFontFamily);
            card.valueUserData = false;
            compare(card.valueItem.font.family, Ui.Theme.uiFontFamily);
            menuEntry.userData = false;
            compare(findChild(menuEntry, "filenameMenuLabel").font.family, Ui.Theme.uiFontFamily);
        }
    }
}
