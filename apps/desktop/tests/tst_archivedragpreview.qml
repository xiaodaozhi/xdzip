import QtQuick
import QtTest
import "../ui" as Ui

Item {
    id: fixture
    width: 520; height: 360
    MouseArea { id: receiver; anchors.fill: parent; property int clicks: 0; onClicked: clicks++ }
    Ui.ArchiveDragPreview {
        id: preview
        visible: false
        x: 16; y: 16
        fileName: "<b>很长的旅行归档文件名与说明以及更多完整的原始文件名内容</b>.txt"
        itemCount: 3
        message: "不能移动到所选文件夹或其子文件夹。"
    }
    TestCase {
        name: "ArchiveDragPreview"; when: windowShown
        function test_layout_and_input_data() {
            const rows=[];
            for (const dark of [false,true]) for (const factor of [1,1.25,1.5])
                rows.push({tag:String(dark)+"-"+factor,dark:dark,factor:factor});
            return rows;
        }
        function test_layout_and_input(data) {
            Ui.Theme.dark=data.dark; Ui.Theme.fontScale=data.factor;
            preview.itemCount=3; preview.message="不能移动到所选文件夹或其子文件夹。";
            preview.visible=true;
            verify(waitForRendering(preview));
            const name=findChild(preview,"dragPreviewName"), count=findChild(preview,"dragPreviewCount");
            compare(name.textFormat,Text.PlainText);
            tryVerify(()=>name.width>0 && name.truncated);
            verify(count.visible && count.text==="3");
            verify(preview.width<=280*data.factor && preview.height<fixture.height-32);
            compare(name.font.pixelSize,Math.round(Ui.Theme.menuLabelSize));
            const before=receiver.clicks;
            mouseClick(preview,preview.width/2,preview.height/2);
            compare(receiver.clicks,before+1,"Preview must not intercept input");
            grabImage(preview).save("drag-preview-multiple-"+data.tag+".png");
            preview.itemCount=1; preview.message="";
            verify(waitForRendering(preview)); verify(!count.visible);
            grabImage(preview).save("drag-preview-single-"+data.tag+".png");
        }
        function cleanup() { preview.visible=false; Ui.Theme.dark=false; Ui.Theme.fontScale=1; }
    }
}
