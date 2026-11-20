#pragma once

#include "UiLanguageData.h"
#include <string>

namespace xdzip_i18n {

inline const Language *findLanguage(std::string_view id) {
    for (const auto &language : languages)
        if (language.id == id)
            return &language;
    return nullptr;
}

inline std::string_view storedLanguage(std::string_view legacy, std::string_view extended) {
    const auto *language = findLanguage(extended);
    return language && language->id != language->legacy && language->legacy == legacy ? language->id
                                                                                      : legacy;
}

inline std::string_view systemLanguage(std::string_view preferred) {
    // POSIX locale names can add an encoding and modifier to the primary language tag.
    std::string tag(preferred.substr(0, preferred.find_first_of(".@")));
    for (auto &character : tag) {
        if (character >= 'A' && character <= 'Z')
            character = char(character + ('a' - 'A'));
        else if (character == '_')
            character = '-';
    }
    const auto separator = tag.find('-');
    const std::string_view base(tag.data(),
                                separator == std::string::npos ? tag.size() : separator);
    if (base == "zh") {
        std::string_view subtag(tag);
        if (separator != std::string::npos) {
            subtag.remove_prefix(separator + 1);
            subtag = subtag.substr(0, subtag.find('-'));
        }
        if (subtag == "hant" || subtag == "tw" || subtag == "hk" || subtag == "mo")
            return "zh-TW";
        return "zh-CN";
    }
    for (const auto &language : languages) {
        // For other languages, the registry has one variant per base language,
        // so the country (or region) subtag does not affect the choice.
        if (base == language.id.substr(0, 2))
            return language.id;
    }
    return defaultLanguage;
}

inline const char *pluralCategory(PluralRule rule, int n) {
    switch (rule) {
    case PluralRule::other:
        return "other";
    case PluralRule::one:
        return n == 1 ? "one" : "other";
    case PluralRule::zeroOneMillion:
    case PluralRule::oneMillion:
        if (n == 1 || (n == 0 && rule == PluralRule::zeroOneMillion))
            return "one";
        return n > 0 && n % 1000000 == 0 ? "many" : "other";
    case PluralRule::russian:
        if (n % 10 == 1 && n % 100 != 11)
            return "one";
        if (n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14))
            return "few";
        return "many";
    }
    return "other";
}

} // namespace xdzip_i18n
