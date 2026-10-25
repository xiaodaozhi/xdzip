// tools/native-codec/main.cpp

#include "protocol.hpp"
#include <algorithm>
#include <codecvt>
#include <cstdio>
#include <cstring>
#include <limits>
#include <locale>
#include <mutex>
#include <new>
#ifdef _WIN32
#define NOMINMAX
#include <fcntl.h>
#include <io.h>
#include <windows.h>
#else
#include <sys/resource.h>
#ifdef __APPLE__
#include <mach/mach.h>
#endif
#endif

namespace xdzip {
Request request;
uint8_t io_fault = 0;
static constexpr size_t MaxFrame = 1024 * 1024, Chunk = 64 * 1024;
static std::recursive_mutex io_mutex;
static void exact_read(void *bytes, size_t size) {
    if (size && fread(bytes, 1, size, stdin) != size)
        throw Failure{1};
}
static void exact_write(const void *bytes, size_t size) {
    if (size && fwrite(bytes, 1, size, stdout) != size)
        throw Failure{8};
}
void put32(std::vector<uint8_t> &out, uint32_t value) {
    for (unsigned i = 0; i < 4; i++)
        out.push_back(uint8_t(value >> (i * 8)));
}
void put64(std::vector<uint8_t> &out, uint64_t value) {
    for (unsigned i = 0; i < 8; i++)
        out.push_back(uint8_t(value >> (i * 8)));
}
void put_string(std::vector<uint8_t> &out, const std::string &value) {
    put32(out, uint32_t(value.size()));
    out.insert(out.end(), value.begin(), value.end());
}
void send(Kind kind, const std::vector<uint8_t> &bytes) {
    std::lock_guard<std::recursive_mutex> guard(io_mutex);
    if (bytes.size() >= MaxFrame)
        throw Failure{4};
    std::vector<uint8_t> header;
    put32(header, uint32_t(bytes.size() + 1));
    header.push_back(kind);
    exact_write(header.data(), header.size());
    exact_write(bytes.data(), bytes.size());
    if (fflush(stdout) != 0)
        throw Failure{8};
}
struct Fields {
    const std::vector<uint8_t> &bytes;
    size_t position = 0;
    uint8_t byte() {
        if (position >= bytes.size())
            throw Failure{2};
        return bytes[position++];
    }
    uint32_t u32() {
        uint32_t value = 0;
        for (unsigned i = 0; i < 4; i++)
            value |= uint32_t(byte()) << (8 * i);
        return value;
    }
    uint64_t u64() {
        uint64_t value = 0;
        for (unsigned i = 0; i < 8; i++)
            value |= uint64_t(byte()) << (8 * i);
        return value;
    }
    std::string string(size_t limit) {
        uint32_t size = u32();
        if (size > limit || size > bytes.size() - position)
            throw Failure{4};
        std::string out(bytes.begin() + position, bytes.begin() + position + size);
        position += size;
        return out;
    }
    void end() {
        if (position != bytes.size())
            throw Failure{2};
    }
};
std::vector<uint8_t> receive(Kind expected) {
    std::lock_guard<std::recursive_mutex> guard(io_mutex);
    uint8_t length_bytes[4];
    exact_read(length_bytes, 4);
    uint32_t length = 0;
    for (unsigned i = 0; i < 4; i++)
        length |= uint32_t(length_bytes[i]) << (8 * i);
    if (length == 0 || length > MaxFrame)
        throw Failure{4};
    uint8_t kind;
    exact_read(&kind, 1);
    if (kind != expected)
        throw Failure{2};
    std::vector<uint8_t> bytes(length - 1);
    exact_read(bytes.data(), bytes.size());
    return bytes;
}
void ack() {
    if (!receive(Ack).empty())
        throw Failure{2};
}
std::string utf8(const std::wstring &value) {
#if WCHAR_MAX <= 0xffff
    std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>> conversion;
#else
    std::wstring_convert<std::codecvt_utf8<wchar_t>> conversion;
#endif
    return conversion.to_bytes(value);
}
std::wstring wide(const std::string &value) {
#if WCHAR_MAX <= 0xffff
    std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>> conversion;
#else
    std::wstring_convert<std::codecvt_utf8<wchar_t>> conversion;
#endif
    return conversion.from_bytes(value);
}
size_t read_source(uint64_t offset, void *data, size_t size) {
    std::lock_guard<std::recursive_mutex> guard(io_mutex);
    size = std::min(size, Chunk);
    if (offset > request.length)
        throw Failure{2};
    size = size_t(std::min<uint64_t>(size, request.length - offset));
    std::vector<uint8_t> args;
    put64(args, offset);
    put32(args, uint32_t(size));
    send(Read, args);
    auto bytes = receive(Bytes);
    if (bytes.size() > size)
        throw Failure{2};
    memcpy(data, bytes.data(), bytes.size());
    return bytes.size();
}
size_t read_input(uint32_t index, void *data, size_t size) {
    std::lock_guard<std::recursive_mutex> guard(io_mutex);
    size = std::min(size, Chunk);
    std::vector<uint8_t> args;
    put32(args, index);
    put32(args, uint32_t(size));
    send(Input, args);
    auto bytes = receive(Bytes);
    if (bytes.size() > size)
        throw Failure{2};
    memcpy(data, bytes.data(), bytes.size());
    return bytes.size();
}
static void write_data_impl(const void *data, size_t size, bool selected, bool precharged) {
    std::lock_guard<std::recursive_mutex> guard(io_mutex);
    if (precharged && !selected)
        throw Failure{2};
    auto bytes = static_cast<const uint8_t *>(data);
    while (size) {
        size_t count = std::min(size, Chunk);
        std::vector<uint8_t> frame;
        if (selected) {
            frame.assign(bytes, bytes + count);
            // WorkerCredit::data in the Rust host accepts this kind only for
            // CAB after successful WORK admission, and debits it once before
            // delivery. Runtime sinks independently retain output quotas.
            send(precharged ? PrechargedData : Data, frame);
        } else {
            put64(frame, count);
            send(Work, frame);
        }
        ack();
        size -= count;
        bytes += count;
    }
}
void write_data(const void *data, size_t size, bool selected) {
    write_data_impl(data, size, selected, false);
}
void write_precharged_data(const void *data, size_t size) {
    write_data_impl(data, size, true, true);
}
void write_at(uint64_t offset, const void *data, size_t size) {
    std::lock_guard<std::recursive_mutex> guard(io_mutex);
    auto bytes = static_cast<const uint8_t *>(data);
    while (size) {
        size_t count = std::min(size, Chunk);
        std::vector<uint8_t> frame;
        put64(frame, offset);
        frame.insert(frame.end(), bytes, bytes + count);
        send(WriteAt, frame);
        ack();
        offset += count;
        bytes += count;
        size -= count;
    }
}
void metadata(uint64_t index, const std::wstring &name, uint8_t flags, uint64_t size,
              uint64_t packed, uint32_t dos, uint16_t method) {
    metadata_bytes(index, utf8(name), flags, size, packed, dos, method);
}
void metadata_bytes(uint64_t index, const std::string &name, uint8_t flags, uint64_t size,
                    uint64_t packed, uint32_t dos, uint16_t method) {
    std::lock_guard<std::recursive_mutex> guard(io_mutex);
    auto path = name;
    if (path.size() > request.name_bytes)
        throw Failure{4};
    if ((flags & 1) != 0 && (path.empty() || path.back() != '/'))
        path.push_back('/');
    if (path.size() > request.name_bytes)
        throw Failure{4};
    std::vector<uint8_t> frame;
    put64(frame, index);
    frame.push_back(flags);
    put64(frame, size);
    put64(frame, packed);
    put32(frame, dos);
    put32(frame, method);
    put_string(frame, path);
    send(Row, frame);
    ack();
}
void entry_done(uint64_t index) {
    std::lock_guard<std::recursive_mutex> guard(io_mutex);
    if (request.operation != 3)
        return;
    std::vector<uint8_t> complete;
    put64(complete, index);
    send(EntryDone, complete);
    auto next = receive(Next);
    Fields fields{next};
    auto selected = fields.u64();
    fields.end();
    if (selected <= index || selected >= request.entries)
        throw Failure{2};
    request.selected = selected;
}
static void limit_memory(uint64_t bytes) {
#ifdef _WIN32
    HANDLE job = CreateJobObjectW(nullptr, nullptr);
    if (!job)
        throw Failure{4};
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION limits{};
    limits.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION;
    limits.ProcessMemoryLimit = SIZE_T(bytes);
    if (!SetInformationJobObject(job, JobObjectExtendedLimitInformation, &limits, sizeof(limits)) ||
        !AssignProcessToJobObject(job, GetCurrentProcess())) {
        CloseHandle(job);
        throw Failure{4};
    }
#else
    uint64_t address_space = bytes;
#ifdef __APPLE__
    // dyld reserves a large shared-cache address range even for a tiny worker.
    // Bound additional virtual space, rather than treating that reservation as
    // decoded memory. This is an address-space guard, not an RSS measurement.
    mach_task_basic_info_data_t info{};
    mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
    if (task_info(mach_task_self(), MACH_TASK_BASIC_INFO, reinterpret_cast<task_info_t>(&info),
                  &count) != KERN_SUCCESS ||
        info.virtual_size > UINT64_MAX - bytes)
        throw Failure{4};
    address_space += info.virtual_size;
#endif
    rlimit limit{rlim_t(address_space), rlim_t(address_space)};
    if (setrlimit(RLIMIT_AS, &limit) != 0)
        throw Failure{4};
#endif
}
static void parse_request() {
    auto bytes = receive(Start);
    Fields fields{bytes};
    request.operation = fields.byte();
    request.format = fields.byte();
    request.memory = fields.u64();
    request.entries = fields.u32();
    request.name_bytes = fields.u32();
    request.length = fields.u64();
    request.has_password = fields.byte() != 0;
    request.password = fields.string(4096);
    request.selected = fields.u64();
    auto count = fields.u32();
    if (count > 1024)
        throw Failure{4};
    uint64_t total = 0;
    for (uint32_t i = 0; i < count; i++) {
        auto length = fields.u64();
        if (length > UINT64_MAX - total)
            throw Failure{2};
        total += length;
        request.volumes.push_back(length);
    }
    if (count && total != request.length)
        throw Failure{2};
    if (request.memory == 0 || request.memory > 1024ULL * 1024 * 1024 || request.entries > 100000 ||
        request.name_bytes > 4096 || request.length > INT64_MAX)
        throw Failure{4};
    limit_memory(request.memory);
    if (request.operation == 2) {
        request.level = fields.byte();
        request.solid = fields.byte() != 0;
        request.encrypt_headers = fields.byte() != 0;
        request.copy = fields.byte() != 0;
        count = fields.u32();
        if (count > request.entries || request.level > 9)
            throw Failure{4};
        fields.end();
        for (uint32_t i = 0; i < count; i++) {
            auto record = receive(Row);
            Fields item{record};
            Record row;
            row.directory = item.byte() != 0;
            row.size = item.u64();
            row.modified = int64_t(item.u64());
            row.old_index = item.u64();
            row.name = wide(item.string(request.name_bytes));
            item.end();
            request.records.push_back(row);
        }
    } else if (request.format == 8) {
        count = fields.u32();
        if (count == 0 || count != request.volumes.size())
            throw Failure{2};
        for (uint32_t i = 0; i < count; i++) {
            auto name = fields.string(4096);
            if (name.empty() || name.find('\0') != std::string::npos ||
                std::find(request.volume_names.begin(), request.volume_names.end(), name) !=
                    request.volume_names.end())
                throw Failure{2};
            request.volume_names.push_back(std::move(name));
        }
        fields.end();
    } else
        fields.end();
}
} // namespace xdzip
int main() {
#ifdef _WIN32
    _setmode(_fileno(stdin), _O_BINARY);
    _setmode(_fileno(stdout), _O_BINARY);
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX);
#endif
    try {
        xdzip::parse_request();
        if (xdzip::request.format == 7)
            xdzip::seven_run();
        else if (xdzip::request.format >= 8 && xdzip::request.format <= 10)
            xdzip::readonly_run();
        else if (xdzip::request.format == 4)
            xdzip::rar_run();
        else
            throw xdzip::Failure{3};
        xdzip::send(xdzip::Done);
        return 0;
    } catch (const xdzip::Failure &failure) {
        try {
            xdzip::send(xdzip::Error, {failure.code});
        } catch (...) {
        }
        return 1;
    } catch (const std::bad_alloc &) {
        try {
            xdzip::send(xdzip::Error, {4});
        } catch (...) {
        }
        return 1;
    } catch (...) {
        try {
            xdzip::send(xdzip::Error, {2});
        } catch (...) {
        }
        return 1;
    }
}
