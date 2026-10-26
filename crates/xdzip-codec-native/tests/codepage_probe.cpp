// Public mathematical strings only. No archive or user password is consumed.
#include "xdzip-codec-bridge/src/lib.rs.h"
#include <algorithm>
#include <array>
#include <cstdlib>
#include <iostream>
#ifdef _WIN32
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#endif

namespace {
template <class T, std::size_t N> rust::Slice<const T> view(const std::array<T, N> &value) {
    return {value.data(), value.size()};
}
template <class T, std::size_t N> rust::Slice<T> edit(std::array<T, N> &value) {
    return {value.data(), value.size()};
}
void require(bool condition) {
    if (!condition)
        std::abort();
}
} // namespace

int main() {
    using namespace xdzip::codec_bridge;
    std::array<std::uint16_t, 512> wide{}, mapped{};
    std::array<std::uint8_t, 512> bytes{};
    wide[0] = 'A';
    wide[1] = 0;
    wide[2] = 'B';
#ifdef _WIN32
    auto result = wide_to_ansi(view(wide), edit(bytes));
    require(result.error == 0 && result.value == 1 && bytes[0] == 'A' && bytes[1] == 0);
    result = oem_to_ansi_wide(view(wide), edit(mapped));
    require(result.error == 0 && result.value == 1 && mapped[0] == 'A' && mapped[1] == 0);
    // The OS primitive has no crypt-layer 127-unit policy; that stays in Rust.
    wide.fill('A');
    wide[130] = 0;
    result = wide_to_ansi(view(wide), edit(bytes));
    require(result.error == 0 && result.value == 130 && bytes[129] == 'A' && bytes[130] == 0);
    // Independent direct WinAPI projection for public Unicode scalar/unit data.
    wide.fill(0);
    std::array<std::uint16_t, 6> units{0x4e2d, 0x6c49, 0x00e9, 0xd83d, 0xde00, 0};
    std::copy(units.begin(), units.end(), wide.begin());
    std::array<wchar_t, 512> native{}, expected{};
    std::copy(wide.begin(), wide.end(), native.begin());
    CPINFO codepage{};
    require(GetCPInfo(CP_OEMCP, &codepage) && codepage.MaxCharSize >= 1 &&
            codepage.MaxCharSize <= 4);
    std::array<char, 512 * 4> narrow{};
    std::array<char, 512> ansi{};
    int count =
        WideCharToMultiByte(CP_ACP, 0, native.data(), -1, ansi.data(), 512, nullptr, nullptr);
    require(count > 0);
    result = wide_to_ansi(view(wide), edit(bytes));
    require(result.error == 0 && result.value == static_cast<std::uint64_t>(count - 1));
    require(std::equal(ansi.begin(), ansi.begin() + count, bytes.begin(),
                       [](char a, std::uint8_t b) { return static_cast<unsigned char>(a) == b; }));
    require(CharToOemBuffW(native.data(), narrow.data(), 512) != 0);
    narrow[511] = 0;
    count = MultiByteToWideChar(CP_ACP, 0, narrow.data(), -1, expected.data(), 512);
    require(count > 0);
    result = oem_to_ansi_wide(view(wide), edit(mapped));
    require(result.error == 0 && result.value == static_cast<std::uint64_t>(count - 1));
    require(std::equal(mapped.begin(), mapped.end(), expected.begin()));
    for (std::size_t length : {1, 80, 255, 511}) {
        struct Guarded {
            std::uint64_t before = 0xabc123;
            std::array<std::uint16_t, 512> output{};
            std::uint64_t after = 0xdef456;
        } guarded;
        wide.fill(0);
        std::fill_n(wide.begin(), length, std::uint16_t{0x5bc6});
        std::copy(wide.begin(), wide.end(), native.begin());
        narrow.fill(0);
        expected.fill(0);
        require(CharToOemBuffW(native.data(), narrow.data(), 512) != 0);
        narrow[511] = 0;
        count = MultiByteToWideChar(CP_ACP, 0, narrow.data(), -1, expected.data(), 512);
        require(count > 0);
        result = oem_to_ansi_wide(view(wide), edit(guarded.output));
        require(result.error == 0 && result.value == static_cast<std::uint64_t>(count - 1));
        require(std::equal(guarded.output.begin(), guarded.output.end(), expected.begin()));
        require(guarded.before == 0xabc123 && guarded.after == 0xdef456);
    }
    mapped.fill(0xa5);
    bytes.fill(0xa5);
    wide.fill('A');
    require(oem_to_ansi_wide(view(wide), edit(mapped)).error == 3);
    require(wide_to_ansi(view(wide), edit(bytes)).error == 3);
    require(std::all_of(mapped.begin(), mapped.end(), [](auto value) { return value == 0xa5; }));
    require(std::all_of(bytes.begin(), bytes.end(), [](auto value) { return value == 0xa5; }));
    wide[511] = 0;
    require(wide_to_ansi(rust::Slice<const std::uint16_t>(wide.data(), 0), edit(bytes)).error == 3);
    require(
        oem_to_ansi_wide(rust::Slice<const std::uint16_t>(wide.data(), 511), edit(mapped)).error ==
        3);
    require(wide_to_ansi(view(wide), rust::Slice<std::uint8_t>(bytes.data(), 511)).error == 3);
#else
    require(wide_to_ansi(view(wide), edit(bytes)).error == 3);
    require(oem_to_ansi_wide(view(wide), edit(mapped)).error == 3);
    std::array<std::uint32_t, 128> scalars{'A', 0, 'B'};
    auto result = wide_to_local_prefix(view(scalars), edit(bytes));
    require(result.error == 0 && result.value == 1 && bytes[0] == 'A' && bytes[1] == 0);
    scalars = {'A', 0x110000, 0};
    require(wide_to_local_prefix(view(scalars), edit(bytes)).error == 3);
#endif
    std::cout << "OS codepage borrowed-buffer vectors passed\n";
}
