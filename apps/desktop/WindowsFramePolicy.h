#pragma once

// Unknown versions and server editions keep the existing rendering policy.
constexpr bool useWindows10ClientCaptionWorkaround(bool workstation, int major, int minor, int build) {
    return workstation && major == 10 && minor == 0 && build >= 10240 && build < 22000;
}
