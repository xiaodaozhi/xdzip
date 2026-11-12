#pragma once
#include "NativeMessages.h"
#include "NativeRequest.h"
#include <algorithm>
#include <cctype>
#include <regex>

namespace xdzip_shell {
struct MenuEntry {
    std::string action, label;
};
inline bool archiveCandidate(const fs::path &path) {
    static const std::regex suffix(
        R"(\.(zip|7z|rar|tar|tgz|gz|cab|iso|udf|z[0-9]{2,}|[r-z][0-9]{2}|(zip|7z)\.[0-9]{3,})$)",
        std::regex::icase);
    std::error_code error;
    return !fs::is_directory(path, error) && std::regex_search(path.filename().u8string(), suffix);
}
inline bool archiveMime(const std::string &mime) {
    static const std::array<const char *, 13> types{"application/zip",
                                                    "application/x-7z-compressed",
                                                    "application/vnd.rar",
                                                    "application/x-rar",
                                                    "application/x-rar-compressed",
                                                    "application/x-tar",
                                                    "application/gzip",
                                                    "application/x-gzip",
                                                    "application/x-compressed-tar",
                                                    "application/vnd.ms-cab-compressed",
                                                    "application/x-cab",
                                                    "application/x-iso9660-image",
                                                    "application/x-udf"};
    return std::any_of(types.begin(), types.end(), [&](const char *type) { return mime == type; });
}
inline std::vector<MenuEntry> menu(const std::vector<fs::path> &paths, bool english,
                                   bool archiveCandidates = false) {
    if (paths.empty() || paths.size() > 128)
        return {};
    auto base = paths.size() == 1 ? paths.front().filename().u8string()
                                  : paths.front().parent_path().filename().u8string();
    if (base.empty())
        base = "archive";
    const auto add = [&](const char *extension) {
        auto label = text(english, "shell.addNamed");
        label.replace(label.find("%1"), 2, base + extension);
        return label;
    };
    std::vector<MenuEntry> result{
        {"zip", add(".zip")}, {"7z", add(".7z")}, {"create", text(english, "shell.create")}};
    if (paths.size() > 1)
        result.push_back({"separate-zip", text(english, "shell.separateZip")});
    if (archiveCandidates || std::all_of(paths.begin(), paths.end(), archiveCandidate)) {
        result.push_back({"extract-here", text(english, "shell.extractHere")});
        result.push_back({"extract-smart", text(english, "shell.extractSmart")});
        result.push_back({"extract-separate", text(english, "shell.extractSeparate")});
        result.push_back({"extract-to", text(english, "shell.extractTo")});
    }
    return result;
}
} // namespace xdzip_shell
