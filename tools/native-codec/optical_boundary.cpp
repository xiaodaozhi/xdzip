// tools/native-codec/optical_boundary.cpp
#include "protocol.hpp"
#include "xdzip-optical-bridge/src/lib.rs.h"
#include <new>

namespace xdzip::optical {
Transfer read_bytes(std::uint64_t offset, rust::Slice<std::uint8_t> buffer) noexcept {
    try {
        if (buffer.size() > 64 * 1024)
            return {0, 4};
        return {xdzip::read_source(offset, buffer.data(), buffer.size()), 0};
    } catch (const Failure &failure) {
        return {0, failure.code};
    } catch (const std::bad_alloc &) {
        return {0, 4};
    } catch (...) {
        return {0, 2};
    }
}
std::uint8_t emit_row(std::uint64_t index, std::uint8_t flags, std::uint64_t size,
                      std::uint64_t packed, std::uint32_t dos, std::uint16_t method,
                      rust::Slice<const std::uint8_t> name) noexcept {
    try {
        xdzip::metadata_bytes(index,
                              std::string(reinterpret_cast<const char *>(name.data()), name.size()),
                              flags, size, packed, dos, method);
        return 0;
    } catch (const Failure &failure) {
        return failure.code;
    } catch (const std::bad_alloc &) {
        return 4;
    } catch (...) {
        return 2;
    }
}
std::uint8_t emit_data(rust::Slice<const std::uint8_t> bytes) noexcept {
    try {
        xdzip::write_data(bytes.data(), bytes.size());
        return 0;
    } catch (const Failure &failure) {
        return failure.code;
    } catch (const std::bad_alloc &) {
        return 4;
    } catch (...) {
        return 2;
    }
}
Transfer complete_entry(std::uint64_t index) noexcept {
    try {
        xdzip::entry_done(index);
        return {xdzip::request.selected, 0};
    } catch (const Failure &failure) {
        return {0, failure.code};
    } catch (const std::bad_alloc &) {
        return {0, 4};
    } catch (...) {
        return {0, 2};
    }
}
} // namespace xdzip::optical
