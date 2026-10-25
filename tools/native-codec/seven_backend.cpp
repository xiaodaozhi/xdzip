// tools/native-codec/seven_backend.cpp
#include "protocol.hpp"
#include "xdzip-codec-bridge/src/lib.rs.h"

namespace xdzip {
void seven_run() {
    std::vector<codec_bridge::CreationRow> records;
    std::size_t names = 0;
    if (request.records.size() > 100000)
        throw Failure{4};
    for (const auto &record : request.records) {
        auto name = utf8(record.name);
        if (name.size() > 4096 || name.size() > 16 * 1024 * 1024 - names)
            throw Failure{4};
        names += name.size();
        records.push_back(
            {rust::String(name), record.size, record.old_index, record.modified, record.directory});
    }
    codec_bridge::CreationOptions options{request.level, request.copy, request.solid,
                                          request.encrypt_headers};
    const auto error = codec_bridge::run_seven(
        request.operation, request.length, request.selected, request.entries, request.name_bytes,
        rust::Slice<const std::uint64_t>(request.volumes.data(), request.volumes.size()),
        rust::Str(request.password), request.has_password,
        rust::Slice<const codec_bridge::CreationRow>(records.data(), records.size()), options);
    if (error)
        throw Failure{error};
    return;
}
} // namespace xdzip
