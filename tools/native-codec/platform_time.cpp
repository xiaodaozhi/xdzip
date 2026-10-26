// Operating-system calendar projection only. DOS packing remains in Rust.
#include "xdzip-codec-bridge/src/lib.rs.h"
#include <cstdint>
#include <ctime>
#include <limits>
#ifdef _WIN32
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#endif

namespace xdzip::codec_bridge {
LocalCalendar project_local_calendar(std::uint8_t kind, std::uint64_t value) noexcept {
    try {
#ifdef _WIN32
        std::uint64_t ticks = value;
        if (kind == 1) {
            constexpr std::uint64_t epoch = 11644473600ULL;
            if (value > UINT64_MAX / 10000000ULL - epoch)
                return {};
            ticks = (value + epoch) * 10000000ULL;
        } else if (kind != 0) {
            return {};
        }
        FILETIME raw{static_cast<DWORD>(ticks), static_cast<DWORD>(ticks >> 32)};
        SYSTEMTIME utc{}, local{};
        // Windows' system time zone supplies historical local rules. A failed
        // conversion has no compatibility value; return an unavailable calendar.
        if (!FileTimeToSystemTime(&raw, &utc) ||
            !SystemTimeToTzSpecificLocalTime(nullptr, &utc, &local))
            return {};
        return {static_cast<std::int32_t>(local.wYear),
                static_cast<std::uint8_t>(local.wMonth),
                static_cast<std::uint8_t>(local.wDay),
                static_cast<std::uint8_t>(local.wHour),
                static_cast<std::uint8_t>(local.wMinute),
                static_cast<std::uint8_t>(local.wSecond),
                true};
#else
        std::int64_t seconds;
        if (kind == 0) {
            seconds = static_cast<std::int64_t>(value / 10000000ULL) - 11644473600LL;
        } else if (kind == 1 && value <= INT64_MAX) {
            seconds = static_cast<std::int64_t>(value);
        } else {
            return {};
        }
        if (seconds < std::numeric_limits<std::time_t>::min() ||
            seconds > std::numeric_limits<std::time_t>::max())
            return {};
        auto instant = static_cast<std::time_t>(seconds);
        std::tm local{};
        if (!localtime_r(&instant, &local) || local.tm_year > INT32_MAX - 1900)
            return {};
        return {static_cast<std::int32_t>(local.tm_year + 1900),
                static_cast<std::uint8_t>(local.tm_mon + 1),
                static_cast<std::uint8_t>(local.tm_mday),
                static_cast<std::uint8_t>(local.tm_hour),
                static_cast<std::uint8_t>(local.tm_min),
                static_cast<std::uint8_t>(local.tm_sec),
                true};
#endif
    } catch (...) {
        return {};
    }
}
} // namespace xdzip::codec_bridge
