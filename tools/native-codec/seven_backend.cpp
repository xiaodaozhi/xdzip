// tools/native-codec/seven_backend.cpp

#include "7zip/Archive/7z/7zHandler.h"
#include "seven_adapter.hpp"
#include <algorithm>

namespace xdzip {
Z7_CLASS_IMP_COM_1(HostInput, ISequentialInStream)
public:
uint32_t index = 0;
uint64_t position = 0;
};
Z7_COM7F_IMF(HostInput::Read(void *data, UInt32 size, UInt32 *processed)) {
    if (processed)
        *processed = 0;
    return guarded([&] {
        auto remaining = request.records[index].size - position;
        auto count = read_input(index, data, size_t(std::min<uint64_t>(size, remaining)));
        position += count;
        if (processed)
            *processed = UInt32(count);
    });
}
class UpdateCallback final : public IArchiveUpdateCallback,
                             public ICryptoGetTextPassword2,
                             public CMyUnknownImp {
    Z7_IFACES_IMP_UNK_2(IArchiveUpdateCallback, ICryptoGetTextPassword2)
    Z7_IFACE_COM7_IMP(IProgress)
  public:
    CMyComPtr<IInArchive> original;
};
Z7_COM7F_IMF(UpdateCallback::SetTotal(UInt64)) {
    return S_OK;
}
Z7_COM7F_IMF(UpdateCallback::SetCompleted(const UInt64 *)) {
    return S_OK;
}
Z7_COM7F_IMF(UpdateCallback::GetUpdateItemInfo(UInt32, Int32 *data, Int32 *properties,
                                               UInt32 *old)) {
    if (data)
        *data = 1;
    if (properties)
        *properties = 1;
    if (old)
        *old = UInt32(-1);
    return S_OK;
}
Z7_COM7F_IMF(UpdateCallback::GetProperty(UInt32 index, PROPID property, PROPVARIANT *value)) {
    return guarded([&] {
        if (index >= request.records.size())
            throw Failure{2};
        auto &row = request.records[index];
        NWindows::NCOM::CPropVariant result;
        if (original && row.old_index != UINT64_MAX && property != kpidPath &&
            property != kpidIsDir && property != kpidSize && property != kpidIsAnti) {
            if (row.old_index > UINT32_MAX)
                throw Failure{2};
            check(original->GetProperty(UInt32(row.old_index), property, value));
            return;
        }
        switch (property) {
        case kpidPath:
            result = row.name.c_str();
            break;
        case kpidIsDir:
            result = row.directory;
            break;
        case kpidSize:
            result = UInt64(row.size);
            break;
        case kpidIsAnti:
            result = false;
            break;
        case kpidMTime:
            if (row.modified >= -11644473600LL &&
                row.modified <= INT64_MAX / 10000000 - 11644473600LL) {
                uint64_t ticks = uint64_t(row.modified + 11644473600LL) * 10000000;
                result.vt = VT_FILETIME;
                result.filetime.dwLowDateTime = DWORD(ticks);
                result.filetime.dwHighDateTime = DWORD(ticks >> 32);
            }
            break;
        }
        result.Detach(value);
    });
}
Z7_COM7F_IMF(UpdateCallback::GetStream(UInt32 index, ISequentialInStream **stream)) {
    *stream = nullptr;
    return guarded([&] {
        if (index >= request.records.size())
            throw Failure{2};
        if (request.records[index].directory)
            return;
        auto input = new HostInput;
        input->index = index;
        CMyComPtr<ISequentialInStream> owner = input;
        *stream = owner.Detach();
    });
}
Z7_COM7F_IMF(UpdateCallback::SetOperationResult(Int32 result)) {
    return result == 0 ? S_OK : E_FAIL;
}
Z7_COM7F_IMF(UpdateCallback::CryptoGetTextPassword2(Int32 *defined, BSTR *value)) {
    *defined = request.has_password ? 1 : 0;
    if (!*defined) {
        *value = nullptr;
        return S_OK;
    }
    return password(value);
}
class ArchiveOutput final : public IOutStream, public CMyUnknownImp {
    Z7_IFACES_IMP_UNK_1(IOutStream)
    Z7_IFACE_COM7_IMP(ISequentialOutStream)
  public:
    uint64_t length = 0, position = 0;
};
Z7_COM7F_IMF(ArchiveOutput::Write(const void *data, UInt32 size, UInt32 *processed)) {
    if (processed)
        *processed = 0;
    return guarded([&] {
        if (position > INT64_MAX || size > INT64_MAX - position)
            throw Failure{4};
        write_at(position, data, size);
        position += size;
        length = (std::max)(length, position);
        if (processed)
            *processed = size;
    });
}
Z7_COM7F_IMF(ArchiveOutput::Seek(Int64 offset, UInt32 origin, UInt64 *result)) {
    return guarded([&] {
        position = seek_position(position, length, offset, origin);
        if (result)
            *result = position;
    });
}
Z7_COM7F_IMF(ArchiveOutput::SetSize(UInt64 size)) {
    return size == length ? S_OK : E_NOTIMPL;
}
static std::wstring raw_path(NArchive::N7z::CHandler *handler, UInt32 index) {
    const void *bytes = nullptr;
    UInt32 size = 0, type = 0;
    check(static_cast<IArchiveGetRawProps *>(handler)->GetRawProp(index, kpidPath, &bytes, &size,
                                                                  &type));
    if (!bytes || size < 2 || size % 2 || size > request.name_bytes * 2 + 2 ||
        type != NPropDataType::kUtf16z)
        throw Failure{4};
    auto data = static_cast<const uint8_t *>(bytes);
    std::wstring name;
    for (uint32_t offset = 0; offset < size; offset += 2) {
        uint32_t value = uint32_t(data[offset]) | (uint32_t(data[offset + 1]) << 8);
        if (offset == size - 2) {
            if (value != 0)
                throw Failure{2};
            break;
        }
        if (value == 0)
            throw Failure{2};
        if (value >= 0xd800 && value <= 0xdbff) {
            if (offset + 4 >= size)
                throw Failure{2};
            uint32_t low = uint32_t(data[offset + 2]) | (uint32_t(data[offset + 3]) << 8);
            if (low < 0xdc00 || low > 0xdfff)
                throw Failure{2};
#if WCHAR_MAX <= 0xffff
            name.push_back(wchar_t(value));
            name.push_back(wchar_t(low));
#else
            name.push_back(wchar_t(0x10000 + ((value - 0xd800) << 10) + (low - 0xdc00)));
#endif
            offset += 2;
        } else {
            if (value >= 0xdc00 && value <= 0xdfff)
                throw Failure{2};
            name.push_back(wchar_t(value));
        }
    }
    return name;
}
void seven_run() {
    seven_registry_check();
    auto handler = new NArchive::N7z::CHandler;
    CMyComPtr<IInArchive> archive = handler;
    if (request.operation == 2) {
        CMyComPtr<IInArchive> old_archive;
        if (request.length) {
            auto old_handler = new NArchive::N7z::CHandler;
            old_archive = static_cast<IInArchive *>(old_handler);
            CMyComPtr<IInStream> old_source = new HostSource;
            CMyComPtr<IArchiveOpenCallback> old_callback = new OpenCallback;
            check(old_archive->Open(old_source, nullptr, old_callback));
        }
        const wchar_t *names[] = {L"0", L"x", L"s", L"he", L"mt", L"0d"};
        NWindows::NCOM::CPropVariant properties[6];
        properties[0] = request.copy ? L"Copy" : L"LZMA2";
        properties[1] = UInt32(request.level);
        properties[2] = request.solid;
        properties[3] = request.encrypt_headers;
        properties[4] = UInt32(1);
        properties[5] = UInt32(23);
        // Copy has no dictionary property; the default 8MiB dictionary bounds normal creation.
        check(static_cast<ISetProperties *>(handler)->SetProperties(names, properties,
                                                                    request.copy ? 5 : 6));
        CMyComPtr<IOutStream> output = new ArchiveOutput;
        auto update_callback = new UpdateCallback;
        update_callback->original = old_archive;
        CMyComPtr<IArchiveUpdateCallback> callback = update_callback;
        check(static_cast<IOutArchive *>(handler)->UpdateItems(
            output, UInt32(request.records.size()), callback));
        return;
    }
    read_archive(archive, [&](UInt32 index) {
        bool directory = bool_property(archive, index, kpidIsDir),
             encrypted = bool_property(archive, index, kpidEncrypted);
        auto mode = (unsigned_property(archive, index, kpidAttrib) >> 16) & 0170000;
        bool link = mode == 0120000,
             special = (mode != 0 && mode != 0100000 && mode != 0040000 && !link) ||
                       bool_property(archive, index, kpidIsAnti);
        return ArchiveRow{utf8(raw_path(handler, index)),
                          uint8_t(unsigned(directory) | (unsigned(encrypted) << 1) |
                                  (unsigned(link) << 2) | (unsigned(special) << 3)),
                          0};
    });
}
} // namespace xdzip
