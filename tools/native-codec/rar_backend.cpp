// tools/native-codec/rar_backend.cpp
#include "protocol.hpp"
#include "xdzip-codec-bridge/src/lib.rs.h"

namespace xdzip {
void rar_run() {
    // Request storage outlives this synchronous borrowed call. Rust receives
    // only authorized streams through callbacks, never output pathnames.
    const auto error = codec_bridge::run_rar(
        request.operation, request.length, request.selected, request.entries, request.name_bytes,
        rust::Slice<const std::uint64_t>(request.volumes.data(), request.volumes.size()),
        rust::Str(request.password), request.has_password);
    if (error)
        throw Failure{error};
}
} // namespace xdzip
