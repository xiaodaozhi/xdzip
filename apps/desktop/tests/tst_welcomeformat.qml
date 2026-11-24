import QtQuick
import QtTest
import "../ui" as Ui
import "../ui/ArchiveFormats.js" as ArchiveFormats

Item {
    id: fixture
    width: 1024; height: 622
    Ui.WelcomePage { id: welcome; anchors.fill: parent }
    TestCase {
        name: "WelcomeSuffixLabels"
        when: windowShown
        function cleanup() { Ui.Theme.fontScale = 1; welcome.expandedFormatGroup = "archives"; }
        function test_suffixes_data() {
            return [{tag: "normal", scale: 1}, {tag: "large", scale: 1.25}, {tag: "larger", scale: 1.5}];
        }
        function test_suffixes(data) {
            Ui.Theme.fontScale = data.scale;
            for (const group of ArchiveFormats.formatGroups) {
                welcome.expandedFormatGroup = group.id;
                const preview = findChild(welcome, "welcomeFormatPreview");
                compare(preview.Accessible.name, group.suffixes.map(s => s.toUpperCase()).join(", "));
                for (const suffix of group.suffixes) {
                    const chip = findChild(welcome, "welcomeFormatSuffix-" + suffix);
                    verify(chip !== null, suffix);
                    const label = findChild(chip, chip.objectName + "Label");
                    verify(label !== null, suffix);
                    compare(label.text, suffix.toUpperCase());
                    const center = label.y + label.baselineOffset + label.inkBounds.y + label.inkBounds.height / 2;
                    verify(Math.abs(center - chip.height / 2) <= 0.01, suffix + " ink center " + center + " chip height " + chip.height);
                }
            }
        }
    }
}
