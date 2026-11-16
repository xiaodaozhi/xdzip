import QtQuick
import QtTest

TestCase {
    name: "SettingsStorage"
    when: windowShown
    function test_unicode_profile_environment() {
        verify(TestFixture.profileMatchesEnvironment(), "Qt and native environment retain the owned Unicode path");
    }
    function test_transient_reader_data() {
        return [{tag:"profile",target:"profile"}, {tag:"backup",target:"backup"}];
    }
    function test_transient_reader(data) {
        const result = TestFixture.contendSettingsWrite(data.target, true);
        if (!result.supported) skip("Windows native file-sharing regression");
        verify(!result.fixtureError, result.fixtureError || "");
        compare(result.saved, true, "A released reader permits the atomic commit");
        compare(result.error, "");
        verify(result.profileCommitted);
        verify(result.backupMatchesPrevious);
    }
    function test_persistent_reader_data() { return test_transient_reader_data(); }
    function test_persistent_reader(data) {
        const result = TestFixture.contendSettingsWrite(data.target, false);
        if (!result.supported) skip("Windows native file-sharing regression");
        verify(!result.fixtureError, result.fixtureError || "");
        compare(result.saved, false);
        verify(result.error.length > 0);
        verify(result.profileUnchanged);
        verify(result.modelUnchanged);
        if (data.target === "backup") verify(result.backupUnchanged);
        else verify(result.backupMatchesPrevious);
    }
}
