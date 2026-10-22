// apps/desktop/tests/tst_archivetabs.qml

import QtQuick
import QtTest
import "../ui" as Ui

Item {
    id: surface
    width: 1024; height: 600
    Ui.ArchiveTabs {
        id: bar
        width: surface.width
        archiveTitle: ""
        sessionControls: true
        onTabSelected: identity => { tabs = tabs.map(tab => ({id:tab.id,title:tab.title,selected:tab.id === identity})); }
        onTabMoved: (identity, target) => {
            const next = tabs.slice(), from = next.findIndex(tab => tab.id === identity);
            next.splice(target,0,next.splice(from,1)[0]); tabs = next;
        }
    }
    SignalSpy { id: moves; target: bar; signalName: "tabMoved" }
    SignalSpy { id: windowDrags; target: bar; signalName: "dragRequested" }
    SignalSpy { id: scrollClicks; signalName: "clicked" }
    TestCase {
        name: "ArchiveTabs"
        when: windowShown
        function populate(count, selected) {
            bar.tabs = Array.from({length:count}, (_, index) => ({id:"tab"+index,title:"archive-"+index+".zip",selected:index === selected}));
        }
        function init() { Ui.Theme.fontScale=1; Ui.Theme.dark=false; bar.width=1024; bar.windowsControls=false; bar.integratedTitlebar=false; moves.clear(); windowDrags.clear(); populate(3,0); }
        function cleanup() { bar.cancelReorder(); Ui.Theme.fontScale=1; }
        function test_release_position_controls_drop_data() {
            return [{tag:"final-tab",outside:false},{tag:"outside-titlebar",outside:true}];
        }
        function test_release_position_controls_drop(data) {
            const strip=findChild(bar,"archiveTabStrip");
            tryVerify(()=>strip.itemAtIndex(0)!==null && strip.itemAtIndex(2)!==null);
            waitForRendering(bar,100);
            const first=strip.itemAtIndex(0), second=strip.itemAtIndex(1), third=strip.itemAtIndex(2);
            const start=first.mapToItem(surface,first.width/2,first.height/2);
            const intermediate=second.mapToItem(surface,second.width/2,second.height/2);
            const finish=third.mapToItem(surface,third.width/2,third.height/2);
            mousePress(surface,start.x,start.y,Qt.LeftButton);
            mouseMove(surface,intermediate.x,intermediate.y,20);
            tryCompare(bar,"reordering",true);
            tryCompare(bar,"dropIndex",1);
            // A release carries its own final position without requiring another move.
            mouseRelease(surface,finish.x,data.outside ? bar.height+20 : finish.y,Qt.LeftButton);
            tryCompare(bar,"reordering",false);
            compare(moves.count,data.outside ? 0 : 1);
            if (!data.outside) {
                compare(moves.signalArguments[0][0],"tab0");
                compare(moves.signalArguments[0][1],2);
            }
            compare(bar.tabs.map(tab=>tab.id).join(","),data.outside ? "tab0,tab1,tab2" : "tab1,tab2,tab0");
            compare(bar.tabs.find(tab=>tab.selected).id,"tab0");
            compare(windowDrags.count,0);
        }
        function test_weight_and_drag() {
            const strip=findChild(bar,"archiveTabStrip");
            tryVerify(()=>strip.itemAtIndex(0)!==null && strip.itemAtIndex(2)!==null);
            waitForRendering(bar,100);
            compare(findChild(bar,"archiveTabLabel-0").font.weight,Font.DemiBold);
            compare(findChild(bar,"archiveTabLabel-1").font.weight,Font.Normal);
            const first=strip.itemAtIndex(0), third=strip.itemAtIndex(2);
            const start=first.mapToItem(surface,first.width/2,first.height/2);
            const finish=third.mapToItem(surface,third.width/2,third.height/2);
            mousePress(surface,start.x,start.y,Qt.LeftButton);
            mouseMove(surface,start.x+20,start.y,20);
            mouseMove(surface,finish.x,finish.y,20);
            tryVerify(()=>findChild(bar,"archiveTabDragPreview").visible);
            verify(findChild(bar,"archiveTabDropIndicator").visible);
            mouseRelease(surface,finish.x,finish.y,Qt.LeftButton);
            tryCompare(moves,"count",1);
            compare(moves.signalArguments[0][0],"tab0"); compare(moves.signalArguments[0][1],2);
            compare(bar.tabs.map(tab=>tab.id).join(","),"tab1,tab2,tab0");
            compare(bar.tabs.find(tab=>tab.selected).id,"tab0");
            compare(windowDrags.count,0);
            keyClick(Qt.Key_PageUp,Qt.ControlModifier|Qt.ShiftModifier);
            tryCompare(moves,"count",2);
            compare(bar.tabs.map(tab=>tab.id).join(","),"tab1,tab0,tab2");
        }
        function test_escape_cancels_drag() {
            const strip=findChild(bar,"archiveTabStrip");
            tryVerify(()=>strip.itemAtIndex(0)!==null && strip.itemAtIndex(2)!==null);
            waitForRendering(bar,100);
            const first=strip.itemAtIndex(0), third=strip.itemAtIndex(2);
            const start=first.mapToItem(surface,first.width/2,first.height/2);
            const finish=third.mapToItem(surface,third.width/2,third.height/2);
            mousePress(surface,start.x,start.y,Qt.LeftButton);
            mouseMove(surface,finish.x,finish.y,30);
            tryVerify(()=>bar.reordering);
            keyClick(Qt.Key_Escape);
            tryCompare(bar,"reordering",false);
            mouseRelease(surface,finish.x,finish.y,Qt.LeftButton);
            compare(moves.count,0);
            compare(bar.tabs[0].id,"tab0");
        }
        function test_drag_scrolls_at_edge() {
            populate(16,0);
            const strip=findChild(bar,"archiveTabStrip");
            tryVerify(()=>bar.overflowing && strip.itemAtIndex(0)!==null);
            const item=strip.itemAtIndex(0), start=item.mapToItem(surface,item.width/2,item.height/2);
            const edge=strip.mapToItem(surface,strip.width-2,strip.height/2);
            mousePress(surface,start.x,start.y,Qt.LeftButton);
            mouseMove(surface,edge.x,edge.y,20);
            tryVerify(()=>bar.reordering && strip.contentX>strip.originX);
            keyClick(Qt.Key_Escape);
            mouseRelease(surface,edge.x,edge.y,Qt.LeftButton);
            compare(moves.count,0);
            compare(bar.tabs[0].id,"tab0");
        }
        function test_overflow_data() {
            return [{tag:"normal",width:1024,scale:1,windows:false,mac:false},
                {tag:"windows-large",width:832,scale:1.5,windows:true,mac:false},
                {tag:"mac-large-dark",width:832,scale:1.5,windows:false,mac:true}];
        }
        function test_overflow(data) {
            Ui.Theme.fontScale=data.scale; Ui.Theme.dark=data.mac;
            bar.width=data.width; bar.windowsControls=data.windows; bar.integratedTitlebar=data.mac;
            populate(32,31);
            const strip=findChild(bar,"archiveTabStrip"), add=findChild(bar,"archiveNewTab");
            const list=findChild(bar,"archiveTabList"), settings=findChild(bar,"archiveSettings");
            tryVerify(()=>bar.overflowing && list.visible && strip.itemAtIndex(31)!==null);
            const active=strip.itemAtIndex(31);
            tryVerify(()=>active.mapToItem(strip,0,0).x>=-.5 && active.mapToItem(strip,active.width,0).x<=strip.width+.5);
            verify(add.x+add.width<=list.mapToItem(bar,0,0).x);
            verify(list.mapToItem(bar,list.width,0).x<=settings.x);
            verify(active.width>=bar.minimumTabWidth);
            waitForRendering(bar,100);
            const previous=findChild(bar,"archiveTabsPrevious");
            scrollClicks.target=previous; scrollClicks.clear();
            mouseClick(previous);
            tryCompare(scrollClicks,"count",1);
            tryVerify(()=>strip.contentX<strip.originX+strip.contentWidth-strip.width-.5);
            mouseClick(list);
            const menu=findChild(bar,"archiveTabListMenu");
            tryVerify(()=>menu.visible);
            const target=findChild(menu,"archiveTabListItem-tab2");
            verify(target!==null);
            keyClick(Qt.Key_Home); keyClick(Qt.Key_Down); keyClick(Qt.Key_Down); keyClick(Qt.Key_Return);
            tryCompare(menu,"visible",false);
            tryVerify(()=>bar.tabs[2].selected && strip.itemAtIndex(2)!==null);
            const selected=strip.itemAtIndex(2);
            tryVerify(()=>selected.mapToItem(strip,0,0).x>=-.5 && selected.mapToItem(strip,selected.width,0).x<=strip.width+.5);
            waitForRendering(bar,100);
            grabImage(bar).save("archive-tabs-"+data.tag+".png");
        }
    }
}
