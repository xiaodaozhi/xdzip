// tools/native-codec/readonly_backend.cpp

#include "CabHandler.h"
#include "IsoHandler.h"
#include "UdfHandler.h"
#include "seven_adapter.hpp"

namespace xdzip {
static std::string raw_bytes(const std::wstring &name) {
    std::string result;
    for (auto value : name) {
        if (value <= 0 || value > 255)
            throw Failure{2};
        result.push_back(char(value));
    }
    return result;
}
void readonly_run() {
    seven_registry_check();
    if (request.operation == 2)
        throw Failure{3};
    if (request.format == 8) {
        auto handler = new NArchive::NCab::CHandler;
        CMyComPtr<IInArchive> archive = handler;
        read_archive(archive, [&](UInt32 index) {
            const auto &item = handler->XdzipRawItem(index);
            return ArchiveRow{std::string(item.Name.Ptr(), item.Name.Len()),
                              uint8_t(0x10 | (item.IsNameUTF() ? 0x20 : 0)), 0};
        });
        return;
    }
    if (request.format == 9) {
        CMyComPtr<IInArchive> archive = new NArchive::NIso::CHandler;
        read_archive(archive, [&](UInt32 index) {
            const bool directory = bool_property(archive, index, kpidIsDir);
            const auto mode = unsigned_property(archive, index, kpidPosixAttrib) & 0170000;
            const bool link =
                mode == 0120000 || !text_property(archive, index, kpidSymLink).empty();
            const bool special = mode != 0 && mode != 0100000 && mode != 0040000 && !link;
            const bool unicode = bool_property(archive, index, kpidUserDefined + 0x101);
            const auto name = text_property(archive, index, kpidPath);
            return ArchiveRow{unicode ? utf8(name) : raw_bytes(name),
                              uint8_t(unsigned(directory) | (unsigned(link) << 2) |
                                      (unsigned(special) << 3) | (unicode ? 0 : 0x10)),
                              0};
        });
        return;
    }
    if (request.format == 10) {
        CMyComPtr<IInArchive> archive = new NArchive::NUdf::CHandler;
        read_archive(archive, [&](UInt32 index) {
            const auto type = unsigned_property(archive, index, kpidUserDefined + 0x100);
            const bool directory = type == 4;
            const bool link =
                type == 12 || (!directory && unsigned_property(archive, index, kpidLinks) > 1);
            const bool special = !link && type != 4 && type != 5 && type != 249;
            return ArchiveRow{
                utf8(text_property(archive, index, kpidPath)),
                uint8_t(unsigned(directory) | (unsigned(link) << 2) | (unsigned(special) << 3)),
                uint16_t(type)};
        });
        return;
    }
    throw Failure{3};
}
} // namespace xdzip
