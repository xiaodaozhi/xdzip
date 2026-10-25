// tools/native-codec/seven_adapter.cpp

#include "seven_adapter.hpp"
#include "Windows/TimeUtils.h"
#include <algorithm>
#include <cstring>
#include <set>

namespace xdzip {
STDAPI GetNumberOfFormats(UInt32 *count);
STDAPI GetHandlerProperty2(UInt32 index, PROPID property, PROPVARIANT *value);
std::atomic<uint8_t> callback_fault{0};
void check(HRESULT result) {
    if (callback_fault)
        throw Failure{callback_fault};
    if (result == E_OUTOFMEMORY)
        throw Failure{4};
    if (result != S_OK)
        throw Failure{2};
}
HRESULT password(BSTR *value) noexcept {
    return guarded([&] {
        if (!request.has_password)
            throw Failure{6};
        auto text = wide(request.password);
        *value = SysAllocString(text.c_str());
        if (!*value)
            throw std::bad_alloc();
    });
}
uint64_t seek_position(uint64_t current, uint64_t length, Int64 offset, UInt32 origin) {
    uint64_t base = origin == STREAM_SEEK_SET   ? 0
                    : origin == STREAM_SEEK_CUR ? current
                    : origin == STREAM_SEEK_END ? length
                                                : UINT64_MAX;
    if (base == UINT64_MAX || (offset < 0 && uint64_t(-(offset + 1)) + 1 > base) ||
        (offset >= 0 && uint64_t(offset) > UINT64_MAX - base))
        throw Failure{2};
    return offset < 0 ? base - (uint64_t(-(offset + 1)) + 1) : base + uint64_t(offset);
}
Z7_COM7F_IMF(HostSource::Read(void *data, UInt32 size, UInt32 *processed)) {
    if (processed)
        *processed = 0;
    return guarded([&] {
        if (position > length)
            return;
        auto count =
            read_source(base + position, data, size_t(std::min<uint64_t>(size, length - position)));
        position += count;
        if (processed)
            *processed = UInt32(count);
    });
}
Z7_COM7F_IMF(HostSource::Seek(Int64 offset, UInt32 origin, UInt64 *result)) {
    return guarded([&] {
        position = seek_position(position, length, offset, origin);
        if (result)
            *result = position;
    });
}
Z7_COM7F_IMF(OpenCallback::SetTotal(const UInt64 *, const UInt64 *)) {
    return S_OK;
}
Z7_COM7F_IMF(OpenCallback::SetCompleted(const UInt64 *, const UInt64 *)) {
    return S_OK;
}
Z7_COM7F_IMF(OpenCallback::CryptoGetTextPassword(BSTR *value)) {
    return password(value);
}
Z7_CLASS_IMP_COM_1(HostOutput, ISequentialOutStream)
public:
bool selected = true;
};
Z7_COM7F_IMF(HostOutput::Write(const void *data, UInt32 size, UInt32 *processed)) {
    if (processed)
        *processed = 0;
    return guarded([&] {
        write_data(data, size, selected);
        if (processed)
            *processed = size;
    });
}
class ExtractCallback final : public IArchiveExtractCallback,
                              public ICryptoGetTextPassword,
                              public CMyUnknownImp {
    Z7_IFACES_IMP_UNK_2(IArchiveExtractCallback, ICryptoGetTextPassword)
    Z7_IFACE_COM7_IMP(IProgress)
    UInt32 current = 0;

  public:
    std::function<bool(UInt32)> allowed;
};
Z7_COM7F_IMF(ExtractCallback::SetTotal(UInt64)) {
    return S_OK;
}
Z7_COM7F_IMF(ExtractCallback::SetCompleted(const UInt64 *)) {
    return S_OK;
}
Z7_COM7F_IMF(ExtractCallback::GetStream(UInt32 index, ISequentialOutStream **stream, Int32 mode)) {
    current = index;
    *stream = nullptr;
    return guarded([&] {
        if (mode == NArchive::NExtract::NAskMode::kExtract) {
            if (index == request.selected && !allowed(index))
                throw Failure{3};
            auto output = new HostOutput;
            output->selected = index == request.selected;
            CMyComPtr<ISequentialOutStream> owner = output;
            *stream = owner.Detach();
        }
    });
}
Z7_COM7F_IMF(ExtractCallback::PrepareOperation(Int32)) {
    return S_OK;
}
Z7_COM7F_IMF(ExtractCallback::SetOperationResult(Int32 result)) {
    if (result == NArchive::NExtract::NOperationResult::kOK)
        return guarded([&] {
            if (current == request.selected)
                entry_done(current);
        });
    callback_fault = result == NArchive::NExtract::NOperationResult::kUnsupportedMethod ? 3
                     : result == NArchive::NExtract::NOperationResult::kWrongPassword   ? 7
                                                                                        : 2;
    return E_ABORT;
}
Z7_COM7F_IMF(ExtractCallback::CryptoGetTextPassword(BSTR *value)) {
    return password(value);
}
uint64_t unsigned_property(IInArchive *archive, UInt32 index, PROPID id) {
    NWindows::NCOM::CPropVariant value;
    check(archive->GetProperty(index, id, &value));
    if (value.vt == VT_UI8)
        return value.uhVal.QuadPart;
    if (value.vt == VT_UI4)
        return value.ulVal;
    if (value.vt == VT_EMPTY)
        return 0;
    throw Failure{2};
}
bool bool_property(IInArchive *archive, UInt32 index, PROPID id) {
    NWindows::NCOM::CPropVariant value;
    check(archive->GetProperty(index, id, &value));
    if (value.vt == VT_BOOL)
        return value.boolVal != VARIANT_FALSE;
    if (value.vt == VT_EMPTY)
        return false;
    throw Failure{2};
}
std::wstring text_property(IInArchive *archive, UInt32 index, PROPID id) {
    NWindows::NCOM::CPropVariant value;
    check(archive->GetProperty(index, id, &value));
    if (value.vt == VT_EMPTY)
        return {};
    if (value.vt != VT_BSTR)
        throw Failure{2};
    return {value.bstrVal, SysStringLen(value.bstrVal)};
}
uint32_t modified_dos(IInArchive *archive, UInt32 index) {
    NWindows::NCOM::CPropVariant modified;
    check(archive->GetProperty(index, kpidMTime, &modified));
    UInt32 dos = 0;
    if (modified.vt == VT_FILETIME)
        NWindows::NTime::FileTime_To_DosTime(modified.filetime, dos);
    return dos;
}
void seven_registry_check() {
    UInt32 formats = 0;
    check(GetNumberOfFormats(&formats));
    if (formats != 1)
        throw Failure{3};
    std::set<std::wstring> expected{L"7z"};
    for (UInt32 index = 0; index < formats; index++) {
        NWindows::NCOM::CPropVariant name;
        check(GetHandlerProperty2(index, NArchive::NHandlerPropID::kName, &name));
        if (name.vt != VT_BSTR || expected.erase(name.bstrVal) != 1)
            throw Failure{3};
    }
    if (!expected.empty())
        throw Failure{3};
}
void read_archive(IInArchive *archive, const std::function<ArchiveRow(UInt32)> &row) {
    if (request.operation != 0 && request.operation != 1 && request.operation != 3)
        throw Failure{3};
    CMyComPtr<IInStream> source = new HostSource;
    CMyComPtr<IArchiveOpenCallback> callback = new OpenCallback;
    const UInt64 search = 0;
    HRESULT result = archive->Open(source, &search, callback);
    if (result != S_OK && request.format == 7 && request.has_password && !callback_fault)
        throw Failure{7};
    check(result);
    NWindows::NCOM::CPropVariant errors;
    check(archive->GetArchiveProperty(kpidErrorFlags, &errors));
    if (errors.vt != VT_EMPTY && (errors.vt != VT_UI4 || errors.ulVal != 0))
        throw Failure{2};
    NWindows::NCOM::CPropVariant message;
    check(archive->GetArchiveProperty(kpidError, &message));
    if (message.vt != VT_EMPTY && (message.vt != VT_BSTR || SysStringLen(message.bstrVal) != 0))
        throw Failure{2};
    UInt32 count = 0;
    check(archive->GetNumberOfItems(&count));
    if (count > request.entries)
        throw Failure{4};
    if (request.operation == 0) {
        for (UInt32 index = 0; index < count; index++) {
            auto value = row(index);
            metadata_bytes(index, value.name, value.flags,
                           unsigned_property(archive, index, kpidSize),
                           unsigned_property(archive, index, kpidPackSize),
                           modified_dos(archive, index), value.method);
        }
        return;
    }
    if (request.selected >= count)
        throw Failure{2};
    if (row(UInt32(request.selected)).flags & 12)
        throw Failure{3};
    std::vector<UInt32> indices;
    // 7z solid blocks need explicit preceding output accounting.
    const UInt32 first = 0;
    const UInt32 end = request.operation == 3 ? count : UInt32(request.selected) + 1;
    for (UInt32 index = first; index < end; index++)
        indices.push_back(index);
    auto extract_callback = new ExtractCallback;
    extract_callback->allowed = [&](UInt32 index) { return (row(index).flags & 12) == 0; };
    CMyComPtr<IArchiveExtractCallback> extract = extract_callback;
    check(archive->Extract(indices.data(), UInt32(indices.size()), 0, extract));
}
} // namespace xdzip
