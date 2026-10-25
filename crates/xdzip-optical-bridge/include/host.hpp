#pragma once
#include "rust/cxx.h"
#include <cstdint>

namespace xdzip::optical {
struct Transfer;
Transfer read_bytes(std::uint64_t offset, rust::Slice<std::uint8_t> buffer) noexcept;
std::uint8_t emit_row(std::uint64_t index, std::uint8_t flags, std::uint64_t size,
                      std::uint64_t packed, std::uint32_t dos, std::uint16_t method,
                      rust::Slice<const std::uint8_t> name) noexcept;
std::uint8_t emit_data(rust::Slice<const std::uint8_t> bytes) noexcept;
Transfer complete_entry(std::uint64_t index) noexcept;
} // namespace xdzip::optical
