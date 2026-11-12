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
inline std::vector<MenuEntry> menu(const std::vector<fs::path> &paths, bool english) {
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
    static const std::regex suffix(
        R"(\.(zip|7z|rar|tar|tgz|gz|cab|iso|udf|z[0-9][0-9]|[r-z][0-9][0-9]|(zip|7z)\.[0-9]{3})$)",
        std::regex::icase);
    if (std::all_of(paths.begin(), paths.end(), [&](const fs::path &p) {
            std::error_code error;
            return !fs::is_directory(p, error) &&
                   std::regex_search(p.filename().u8string(), suffix);
        })) {
        result.push_back({"extract-here", text(english, "shell.extractHere")});
        result.push_back({"extract-smart", text(english, "shell.extractSmart")});
        result.push_back({"extract-separate", text(english, "shell.extractSeparate")});
        result.push_back({"extract-to", text(english, "shell.extractTo")});
    }
    return result;
}
} // namespace xdzip_shell
