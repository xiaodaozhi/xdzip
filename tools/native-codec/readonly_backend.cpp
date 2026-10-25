// tools/native-codec/readonly_backend.cpp

#include "CabHandler.h"

#include "seven_adapter.hpp"
#include "xdzip-optical-bridge/src/lib.rs.h"

namespace xdzip {
void readonly_run() {
    if (request.format == 9 || request.format == 10) {
        const auto error =
            optical::run_optical(request.operation, request.format, request.length,
                                 request.selected, request.entries, request.name_bytes);
        if (error)
            throw Failure{error};
        return;
    }
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
    throw Failure{3};
}
} // namespace xdzip
