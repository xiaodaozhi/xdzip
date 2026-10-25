#pragma once
#include "rust/cxx.h"
#include <cstdint>

namespace xdzip::codec_bridge {
struct Transfer;
Transfer read_bytes(std::uint64_t offset, rust::Slice<std::uint8_t> buffer) noexcept;
Transfer read_input_bytes(std::uint32_t index, rust::Slice<std::uint8_t> buffer) noexcept;
std::uint8_t write_archive_at(std::uint64_t offset, rust::Slice<const std::uint8_t> bytes) noexcept;
std::uint8_t emit_creation_progress(std::uint64_t bytes,
                                    rust::Slice<const std::uint8_t> name) noexcept;
std::uint8_t emit_row(std::uint64_t index, std::uint8_t flags, std::uint64_t size,
                      std::uint64_t packed, std::uint32_t dos, std::uint16_t method,
                      rust::Slice<const std::uint8_t> name) noexcept;
std::uint8_t emit_data(rust::Slice<const std::uint8_t> bytes) noexcept;
std::uint8_t emit_precharged_data(rust::Slice<const std::uint8_t> bytes) noexcept;
std::uint8_t emit_work(std::uint64_t bytes) noexcept;
std::uint8_t emit_auxiliary(std::uint64_t bytes) noexcept;
Transfer complete_entry(std::uint64_t index) noexcept;
} // namespace xdzip::codec_bridge
