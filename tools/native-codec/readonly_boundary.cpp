// tools/native-codec/readonly_boundary.cpp
#include "protocol.hpp"
#include "xdzip-readonly-bridge/src/lib.rs.h"
#include <array>
#include <new>

namespace xdzip::readonly_codec {
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
static std::uint8_t emit_output(rust::Slice<const std::uint8_t> bytes, bool precharged) noexcept {
    try {
        if (precharged)
            xdzip::write_precharged_data(bytes.data(), bytes.size());
        else
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
std::uint8_t emit_data(rust::Slice<const std::uint8_t> bytes) noexcept {
    return emit_output(bytes, false);
}
std::uint8_t emit_precharged_data(rust::Slice<const std::uint8_t> bytes) noexcept {
    return emit_output(bytes, true);
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
std::uint8_t emit_work(std::uint64_t bytes) noexcept {
    try {
        if (bytes > 64 * 1024)
            return 4;
        // write_data advances its pointer even for skipped work. A bounded real
        // buffer keeps that arithmetic valid; selected=false emits only counts.
        static constexpr std::array<std::uint8_t, 64 * 1024> ignored{};
        xdzip::write_data(ignored.data(), static_cast<std::size_t>(bytes), false);
        return 0;
    } catch (const Failure &failure) {
        return failure.code;
    } catch (const std::bad_alloc &) {
        return 4;
    } catch (...) {
        return 2;
    }
}
std::uint8_t emit_auxiliary(std::uint64_t bytes) noexcept {
    // Only a scalar count crosses this synchronous call, by value. No Rust
    // pointer is retained; protocol/host rejection and every C++ exception
    // become stable failures before returning across CXX.
    try {
        xdzip::charge_auxiliary(bytes);
        return 0;
    } catch (const Failure &failure) {
        return failure.code;
    } catch (const std::bad_alloc &) {
        return 4;
    } catch (...) {
        return 2;
    }
}
} // namespace xdzip::readonly_codec
