#include "protocol.hpp"
#include <cstdint>
#include <initializer_list>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

namespace xdzip {
void seven_run() {}
void rar_run() {}
void readonly_run() {}
} // namespace xdzip

static std::string bytes(std::initializer_list<unsigned> values) {
    std::string result;
    for (const auto value : values)
        result.push_back(static_cast<char>(value));
    return result;
}

static std::wstring units(std::initializer_list<std::uint32_t> values) {
    std::wstring result;
    for (const auto value : values)
        result.push_back(static_cast<wchar_t>(value));
    return result;
}

static bool check(bool condition, const char *message) {
    if (!condition)
        std::cerr << message << '\n';
    return condition;
}

template <typename F> static bool rejects(F operation, const char *message) {
    try {
        operation();
    } catch (const std::range_error &) {
        return true;
    } catch (...) {
        return check(false, "unexpected exception type");
    }
    return check(false, message);
}

int main() {
    bool ok = true;
    struct Case {
        std::string encoded;
        std::wstring wide;
    };
    std::vector<Case> cases = {
        {bytes({}), units({})},
        {bytes({0x00, 0x41, 0x00, 0x7f}), units({0x00, 0x41, 0x00, 0x7f})},
        {bytes({0xc2, 0x80}), units({0x80})},
        {bytes({0xdf, 0xbf}), units({0x7ff})},
        {bytes({0xe0, 0xa0, 0x80}), units({0x800})},
        {bytes({0xed, 0x9f, 0xbf}), units({0xd7ff})},
        {bytes({0xee, 0x80, 0x80}), units({0xe000})},
        {bytes({0xef, 0xbf, 0xbf}), units({0xffff})},
        {bytes({0xe4, 0xb8, 0xad, 0xe6, 0x96, 0x87}), units({0x4e2d, 0x6587})},
    };
    if constexpr (sizeof(wchar_t) == 2) {
        cases.push_back({bytes({0xf0, 0x90, 0x80, 0x80}), units({0xd800, 0xdc00})});
        cases.push_back({bytes({0xf0, 0x9f, 0x98, 0x80}), units({0xd83d, 0xde00})});
        cases.push_back({bytes({0xf4, 0x8f, 0xbf, 0xbf}), units({0xdbff, 0xdfff})});
    } else {
        cases.push_back({bytes({0xf0, 0x90, 0x80, 0x80}), units({0x10000})});
        cases.push_back({bytes({0xf0, 0x9f, 0x98, 0x80}), units({0x1f600})});
        cases.push_back({bytes({0xf4, 0x8f, 0xbf, 0xbf}), units({0x10ffff})});
    }
    for (const auto &item : cases) {
        ok &= check(xdzip::wide(item.encoded) == item.wide, "UTF-8 decode mismatch");
        ok &= check(xdzip::utf8(item.wide) == item.encoded, "UTF-8 encode mismatch");
    }

    const std::vector<std::string> invalid = {
        bytes({0x80}),
        bytes({0xbf}),
        bytes({0xc0, 0x80}),
        bytes({0xc1, 0xbf}),
        bytes({0xf5, 0x80, 0x80, 0x80}),
        bytes({0xff}),
        bytes({0xc2}),
        bytes({0xe0, 0xa0}),
        bytes({0xf0, 0x90, 0x80}),
        bytes({0xc2, 0x41}),
        bytes({0xe1, 0x80, 0x41}),
        bytes({0xf1, 0x80, 0x41, 0x80}),
        bytes({0xe0, 0x9f, 0xbf}),
        bytes({0xf0, 0x8f, 0xbf, 0xbf}),
        bytes({0xed, 0xa0, 0x80}),
        bytes({0xed, 0xbf, 0xbf}),
        bytes({0xf4, 0x90, 0x80, 0x80}),
        bytes({0x41, 0xc2}),
    };
    for (const auto &item : invalid)
        ok &= rejects([&] { (void)xdzip::wide(item); }, "accepted invalid UTF-8");

    const std::vector<std::wstring> invalid_wide = {
        units({0xd800}), units({0xdbff}),       units({0xdc00}),
        units({0xdfff}), units({0xd800, 0x41}), units({0xdc00, 0xd800}),
    };
    for (const auto &item : invalid_wide)
        ok &= rejects([&] { (void)xdzip::utf8(item); }, "accepted invalid wide character");
    if constexpr (sizeof(wchar_t) > 2) {
        ok &=
            rejects([] { (void)xdzip::utf8(units({0x110000})); }, "accepted scalar above Unicode");
        ok &= rejects([] { (void)xdzip::utf8(std::wstring(1, static_cast<wchar_t>(-1))); },
                      "accepted negative scalar");
    }
    std::cout << "native Unicode conversion, wchar_t=" << sizeof(wchar_t) * 8 << " bits\n";
    return ok ? 0 : 1;
}
