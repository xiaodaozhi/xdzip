#pragma once
#include "ArchiveFormatsGenerated.h"
#include <algorithm>

namespace xdzip_formats {
template <std::size_t N>
inline bool hasSuffix(std::string_view name, const std::array<std::string_view, N> &suffixes) {
    return std::any_of(suffixes.begin(), suffixes.end(), [&](std::string_view suffix) {
        if (name.size() < suffix.size())
            return false;
        const auto tail = name.substr(name.size() - suffix.size());
        return std::equal(tail.begin(), tail.end(), suffix.begin(), [](unsigned char a, char b) {
            return (a >= 'A' && a <= 'Z' ? a + ('a' - 'A') : a) == b;
        });
    });
}
} // namespace xdzip_formats
