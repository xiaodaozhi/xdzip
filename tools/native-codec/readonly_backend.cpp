// tools/native-codec/readonly_backend.cpp

#include "protocol.hpp"
#include "xdzip-codec-bridge/src/lib.rs.h"

namespace xdzip {
void readonly_run() {
    if (request.operation != 0 && request.operation != 1 && request.operation != 3)
        throw Failure{3};
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
        const auto error = codec_bridge::run_cab(
            request.operation, request.length, request.selected, request.entries,
            request.name_bytes,
            rust::Slice<const std::uint64_t>(request.volumes.data(), request.volumes.size()),
            rust::Slice<const std::uint8_t>(names.data(), names.size()));
        if (error)
            throw Failure{error};
        return;
    }
    if (request.format == 9 || request.format == 10) {
        const auto error =
            codec_bridge::run_optical(request.operation, request.format, request.length,
                                      request.selected, request.entries, request.name_bytes);
        if (error)
            throw Failure{error};
        return;
    }
    throw Failure{3};
}
} // namespace xdzip
