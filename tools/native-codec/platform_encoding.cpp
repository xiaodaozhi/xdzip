// Borrowed operating-system code-page services. Password policy stays in Rust.
#include "xdzip-codec-bridge/src/lib.rs.h"
#include <algorithm>
#include <array>
#include <cstdint>
#include <cwchar>
#ifdef _WIN32
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#endif

namespace xdzip::codec_bridge {
namespace {
template <class T, std::size_t N> struct Erased {
    std::array<T, N> bytes{};
    ~Erased() {
        // These bounded scalar buffers can hold secrets. Volatile byte writes
        // keep destruction observable; no slice or temporary is retained.
        auto *raw = reinterpret_cast<volatile unsigned char *>(bytes.data());
        for (std::size_t i = 0; i < sizeof(bytes); ++i)
            raw[i] = 0;
    }
};
template <class T> bool terminated(rust::Slice<const T> input) {
    return std::find(input.begin(), input.end(), T{}) != input.end();
}
} // namespace

Transfer oem_to_ansi_wide(rust::Slice<const std::uint16_t> input,
                          rust::Slice<std::uint16_t> output) noexcept {
#ifdef _WIN32
    if (input.size() != 512 || output.size() != 512 || !terminated(input))
        return {0, 3};
    CPINFO info{};
    if (!GetCPInfo(CP_OEMCP, &info) || info.MaxCharSize == 0 || info.MaxCharSize > 4)
        return {0, 3};
    Erased<wchar_t, 512> wide, converted;
    // CharToOemBuffW counts source UTF16 units, including NUL padding, and
    // accepts no destination capacity. Admit at most four bytes per unit;
    // apply the format's byte-511 termination only after the bounded OS call.
    Erased<char, 512 * 4> narrow;
    for (std::size_t i = 0; i < 512; ++i)
        wide.bytes[i] = static_cast<wchar_t>(input[i]);
    if (!CharToOemBuffW(wide.bytes.data(), narrow.bytes.data(), 512))
        return {0, 3};
    narrow.bytes[511] = 0;
    int count =
        MultiByteToWideChar(CP_ACP, 0, narrow.bytes.data(), -1, converted.bytes.data(), 512);
    if (count <= 0)
        return {0, 3};
    for (std::size_t i = 0; i < 512; ++i)
        output[i] = static_cast<std::uint16_t>(converted.bytes[i]);
    return {static_cast<std::uint64_t>(count - 1), 0};
#else
    (void)input;
    (void)output;
    return {0, 3};
#endif
}

Transfer wide_to_ansi(rust::Slice<const std::uint16_t> input,
                      rust::Slice<std::uint8_t> output) noexcept {
#ifdef _WIN32
    if (input.empty() || input.size() > 512 || output.size() != 512 || !terminated(input))
        return {0, 3};
    Erased<wchar_t, 512> wide;
    for (std::size_t i = 0; i < input.size(); ++i)
        wide.bytes[i] = static_cast<wchar_t>(input[i]);
    int count = WideCharToMultiByte(CP_ACP, 0, wide.bytes.data(), -1,
                                    reinterpret_cast<char *>(output.data()), 512, nullptr, nullptr);
    if (count <= 0)
        return {0, 3};
    return {static_cast<std::uint64_t>(count - 1), 0};
#else
    (void)input;
    (void)output;
    return {0, 3};
#endif
}

Transfer wide_to_local_prefix(rust::Slice<const std::uint32_t> input,
                              rust::Slice<std::uint8_t> output) noexcept {
#ifdef _WIN32
    (void)input;
    (void)output;
    return {0, 3};
#else
    if (input.empty() || input.size() > 128 || output.size() != 512 || !terminated(input))
        return {0, 3};
    static_assert(sizeof(wchar_t) == 4, "Supported Unix hosts use scalar-width wchar_t");
    Erased<wchar_t, 128> wide;
    for (std::size_t i = 0; i < input.size(); ++i) {
        if (input[i] > 0x10ffff)
            return {0, 3};
        wide.bytes[i] = static_cast<wchar_t>(input[i]);
    }
    std::fill(output.begin(), output.end(), 0);
    Erased<std::mbstate_t, 1> state;
    const wchar_t *source = wide.bytes.data();
    auto result =
        std::wcsrtombs(reinterpret_cast<char *>(output.data()), &source, 512, &state.bytes[0]);
    if (result == static_cast<std::size_t>(-1)) {
        // The fixed legacy policy retries with empty conversion state and
        // retains only the representable prefix; it does not adopt LANG here.
        std::fill(output.begin(), output.end(), 0);
        state.bytes[0] = {};
        source = wide.bytes.data();
        (void)std::wcsrtombs(reinterpret_cast<char *>(output.data()), &source, 512,
                             &state.bytes[0]);
    }
    output[511] = 0;
    auto end = std::find(output.begin(), output.end(), std::uint8_t{});
    return {static_cast<std::uint64_t>(end - output.begin()), 0};
#endif
}
} // namespace xdzip::codec_bridge
