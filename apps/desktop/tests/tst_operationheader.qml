import QtQuick
import QtTest
import "../ui" as Ui

Item {
    width: 491; height: 600
    Ui.OperationHeader {
        id: header
        width: parent.width; height: implicitHeight
        source: "qrc:/xdzip/assets/light/archive.png"
        title: "Title"; subtitle: "archive.zip"
    }
    TestCase {
        name: "OperationHeader"; when: windowShown
        function cleanup() { Ui.Theme.fontScale = 1; Ui.Theme.dark = false; }
        function test_content_growth_data() {
            const rows=[];
            for (const english of [false,true])
                for (const dark of [false,true])
                    for (const factor of [1,1.25,1.5])
                        rows.push({tag:english+"-"+dark+"-"+factor,english,dark,factor});
            return rows;
        }
        function test_content_growth(data) {
            Ui.Theme.fontScale=data.factor; Ui.Theme.dark=data.dark;
            header.title=data.english?"Extract files":"解压文件";
            header.subtitle="archive.zip";
            wait(0);
            const normal=header.implicitHeight;
            verify(normal>=96);
            compare(header.iconItem.width,48); compare(header.iconItem.height,48);
            compare(header.titleItem.font.pixelSize,Math.round(20*data.factor));
            compare(header.subtitleItem.font.pixelSize,Math.round(13*data.factor));
            header.subtitle=(data.english?"Owned archive with a long file name / ":"自有归档，长文件名称 / ").repeat(20)+"<b>.zip";
            tryVerify(()=>header.implicitHeight>normal);
            compare(header.subtitleItem.textFormat,Text.PlainText);
            verify(!header.subtitleItem.truncated);
            verify(header.subtitleItem.mapToItem(header,0,header.subtitleItem.height).y<=header.height-20+.5);
            header.subtitle="";
            tryVerify(()=>header.implicitHeight<=normal);
        }
    }
}
