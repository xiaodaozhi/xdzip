// tools/native-codec/host_file.cpp

// UnRAR File boundary replacement. No archive-controlled string reaches an OS
// open/create/rename/delete function. Only synthetic authorized volumes exist.
#include "protocol.hpp"
#include "rar.hpp"
#include <map>

namespace {
struct Source {
    uint64_t start = 0, length = 0, position = 0;
};
std::map<const File *, Source> sources;
bool volume(const std::wstring &name, size_t &index) {
    size_t count = xdzip::request.volumes.empty() ? 1 : xdzip::request.volumes.size();
    for (size_t current = 0; current < 1024; current++) {
        if (name == L"input.part" + std::to_wstring(current + 1) + L".rar") {
            index = current;
            if (current >= count) {
                xdzip::io_fault = 9;
                return false;
            }
            return true;
        }
        if (current == 0 && name == L"input.rar") {
            index = 0;
            return true;
        }
        if (current > 0 && current <= 900) {
            auto digits = std::to_wstring((current - 1) % 100);
            if (digits.size() == 1)
                digits = L"0" + digits;
            std::wstring extension(1, wchar_t(L'r' + (current - 1) / 100));
            extension += digits;
            if (name == L"input.part1." + extension || name == L"input." + extension) {
                index = current;
                if (current >= count) {
                    xdzip::io_fault = 9;
                    return false;
                }
                return true;
            }
        }
    }
    return false;
}
} // namespace
File::File()
    : hFile(FILE_BAD_HANDLE), LastWrite(false), HandleType(FILE_HANDLENORMAL), LineInput(false),
      SkipClose(false), ReadErrorMode(FREM_TRUNCATE), NewFile(false), AllowDelete(false),
      AllowExceptions(false), PreserveAtime(false), TruncatedAfterReadError(false), CurFilePos(0),
      OpenShared(false), ErrorType(FILE_SUCCESS) {}
File::~File() {
    Close();
}
void File::operator=(File &source) {
    Close();
    auto found = sources.find(&source);
    if (found != sources.end())
        sources[this] = found->second;
    FileName = source.FileName;
}
bool File::Open(const std::wstring &name, uint mode) {
    Close();
    size_t index;
    if ((mode & (FMF_UPDATE | FMF_WRITE)) != 0 || !volume(name, index)) {
        ErrorType = FILE_NOTFOUND;
        return false;
    }
    Source source;
    auto &lengths = xdzip::request.volumes;
    if (lengths.empty())
        source.length = xdzip::request.length;
    else {
        for (size_t i = 0; i < index; i++)
            source.start += lengths[i];
        source.length = lengths[index];
    }
    sources[this] = source;
    FileName = name;
    ErrorType = FILE_SUCCESS;
    return true;
}
void File::TOpen(const std::wstring &name) {
    if (!Open(name))
        throw xdzip::Failure{1};
}
bool File::WOpen(const std::wstring &name) {
    return Open(name);
}
bool File::Create(const std::wstring &, uint) {
    return false;
}
void File::TCreate(const std::wstring &, uint) {
    throw xdzip::Failure{8};
}
bool File::WCreate(const std::wstring &, uint) {
    return false;
}
bool File::Close() {
    sources.erase(this);
    return true;
}
bool File::Delete() {
    return false;
}
bool File::Rename(const std::wstring &) {
    return false;
}
bool File::Write(const void *, size_t) {
    return false;
}
int File::Read(void *data, size_t size) {
    return DirectRead(data, size);
}
int File::DirectRead(void *data, size_t size) {
    auto found = sources.find(this);
    if (found == sources.end())
        return -1;
    auto &source = found->second;
    if (source.position >= source.length)
        return 0;
    size_t count =
        xdzip::read_source(source.start + source.position, data,
                           size_t(std::min<uint64_t>(size, source.length - source.position)));
    source.position += count;
    return int(count);
}
void File::Seek(int64 offset, int method) {
    if (!RawSeek(offset, method))
        throw xdzip::Failure{1};
}
bool File::RawSeek(int64 offset, int method) {
    auto found = sources.find(this);
    if (found == sources.end())
        return false;
    auto &source = found->second;
    uint64_t base = method == SEEK_SET   ? 0
                    : method == SEEK_CUR ? source.position
                    : method == SEEK_END ? source.length
                                         : UINT64_MAX;
    if (base == UINT64_MAX || (offset < 0 && uint64_t(-(offset + 1)) + 1 > base) ||
        (offset >= 0 && uint64_t(offset) > INT64_MAX - base))
        return false;
    source.position = offset < 0 ? base - (uint64_t(-(offset + 1)) + 1) : base + uint64_t(offset);
    return true;
}
int64 File::Tell() {
    auto found = sources.find(this);
    return found == sources.end() ? -1 : int64(found->second.position);
}
bool File::IsOpened() {
    return sources.find(this) != sources.end();
}
int64 File::FileLength() {
    auto found = sources.find(this);
    return found == sources.end() ? 0 : int64(found->second.length);
}
bool File::IsDevice() {
    return false;
}
void File::Prealloc(int64) {}
byte File::GetByte() {
    byte value = 0;
    Read(&value, 1);
    return value;
}
void File::PutByte(byte) {}
bool File::Truncate() {
    return false;
}
void File::Flush() {}
void File::SetOpenFileTime(RarTime *, RarTime *, RarTime *) {}
void File::SetCloseFileTime(RarTime *, RarTime *) {}
void File::SetCloseFileTimeByName(const std::wstring &, RarTime *, RarTime *) {}
void File::GetOpenFileTime(RarTime *, RarTime *, RarTime *) {}
#ifdef _UNIX
void File::StatToRarTime(struct stat &, RarTime *, RarTime *, RarTime *) {}
#endif
bool File::RemoveCreated() {
    return true;
}
int64 File::Copy(File &, int64) {
    throw xdzip::Failure{3};
}
