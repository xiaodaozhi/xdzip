// tools/native-codec/protocol.hpp

#pragma once
#include <cstdint>
#include <string>
#include <vector>

namespace xdzip {
enum Kind : uint8_t {
    Start = 1,
    Read = 10,
    Input = 11,
    Data = 12,
    Row = 13,
    Error = 14,
    Done = 15,
    Progress = 16,
    Work = 17,
    EntryDone = 18,
    WriteAt = 19,
    Bytes = 20,
    Ack = 21,
    Next = 22,
    PrechargedData = 23,
    AuxWork = 24
};
struct Failure {
    uint8_t code;
};
struct Record {
    std::wstring name;
    uint64_t size = 0, old_index = UINT64_MAX;
    int64_t modified = 0;
    bool directory = false;
};
struct Request {
    uint8_t operation = 0, format = 0, level = 0;
    bool solid = false, encrypt_headers = false, copy = false, has_password = false;
    uint64_t memory = 0, length = 0, selected = 0;
    uint32_t entries = 0, name_bytes = 0;
    std::string password;
    std::vector<uint64_t> volumes;
    std::vector<std::string> volume_names;
    std::vector<Record> records;
};
extern Request request;
extern uint8_t io_fault;
void send(Kind kind, const std::vector<uint8_t> &bytes = {});
std::vector<uint8_t> receive(Kind expected);
void ack();
void put32(std::vector<uint8_t> &out, uint32_t value);
void put64(std::vector<uint8_t> &out, uint64_t value);
void put_string(std::vector<uint8_t> &out, const std::string &value);
std::string utf8(const std::wstring &value);
std::wstring wide(const std::string &value);
size_t read_source(uint64_t offset, void *data, size_t size);
size_t read_input(uint32_t index, void *data, size_t size);
void write_data(const void *data, size_t size, bool selected = true);
void write_precharged_data(const void *data, size_t size);
void charge_auxiliary(uint64_t size);
void write_at(uint64_t offset, const void *data, size_t size);
void metadata(uint64_t index, const std::wstring &name, uint8_t flags, uint64_t size,
              uint64_t packed, uint32_t dos, uint16_t method = 0);
void metadata_bytes(uint64_t index, const std::string &name, uint8_t flags, uint64_t size,
                    uint64_t packed, uint32_t dos, uint16_t method = 0);
void entry_done(uint64_t index);
void seven_run();
void readonly_run();
void rar_run();
} // namespace xdzip
