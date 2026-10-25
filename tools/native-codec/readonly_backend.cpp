// tools/native-codec/readonly_backend.cpp

#include "CabHandler.h"

#include "seven_adapter.hpp"
#include "xdzip-readonly-bridge/src/lib.rs.h"

namespace xdzip {
void readonly_run() {
    if (request.operation != 0 && request.operation != 1 && request.operation != 3)
        throw Failure{3};
#if defined(XDZIP_RUST_CAB_TEST_ROUTE)
    // This opt-in is confined to dedicated test builds. Normal product builds
    // retain the official route until the independent CAB acceptance is met.
    if (request.format == 8) {
        if (request.volumes.empty() || request.volumes.size() > 1024 ||
            request.volume_names.size() != request.volumes.size())
            throw Failure{2};
        std::vector<std::uint8_t> names;
        for (const auto &name : request.volume_names) {
            if (name.empty() || name.size() > 4096)
                throw Failure{2};
            put32(names, static_cast<std::uint32_t>(name.size()));
            names.insert(names.end(), name.begin(), name.end());
        }
        const auto error = readonly_codec::run_cab(
            request.operation, request.length, request.selected, request.entries,
            request.name_bytes,
            rust::Slice<const std::uint64_t>(request.volumes.data(), request.volumes.size()),
            rust::Slice<const std::uint8_t>(names.data(), names.size()));
        if (error)
            throw Failure{error};
        return;
    }
#endif
    if (request.format == 9 || request.format == 10) {
        const auto error =
            readonly_codec::run_optical(request.operation, request.format, request.length,
                                        request.selected, request.entries, request.name_bytes);
        if (error)
            throw Failure{error};
        return;
    }
    seven_registry_check();
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
    throw Failure{3};
}
} // namespace xdzip
