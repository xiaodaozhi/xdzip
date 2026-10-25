// tools/native-codec/seven_adapter.hpp

#pragma once
#include "7zip/Archive/IArchive.h"
#include "7zip/IPassword.h"
#include "Common/MyCom.h"
#include "Windows/PropVariant.h"
#include "protocol.hpp"
#include <atomic>
#include <functional>

namespace xdzip {
extern std::atomic<uint8_t> callback_fault;
template <class F> HRESULT guarded(F action) noexcept {
    try {
        action();
        return S_OK;
    } catch (const Failure &error) {
        callback_fault = error.code;
        return E_ABORT;
    } catch (const std::bad_alloc &) {
        callback_fault = 4;
        return E_OUTOFMEMORY;
    } catch (...) {
        callback_fault = 2;
        return E_FAIL;
    }
}
void check(HRESULT result);
HRESULT password(BSTR *value) noexcept;
uint64_t seek_position(uint64_t current, uint64_t length, Int64 offset, UInt32 origin);
Z7_CLASS_IMP_IInStream(HostSource) uint64_t position = 0, base = 0, length;

public:
explicit HostSource(uint64_t start = 0, uint64_t size = UINT64_MAX)
    : base(start), length(size == UINT64_MAX ? request.length : size) {}
};
class OpenCallback final : public IArchiveOpenCallback,
                           public IArchiveOpenVolumeCallback,
                           public ICryptoGetTextPassword,
                           public CMyUnknownImp {
    Z7_IFACES_IMP_UNK_3(IArchiveOpenCallback, IArchiveOpenVolumeCallback, ICryptoGetTextPassword)
};
uint64_t unsigned_property(IInArchive *archive, UInt32 index, PROPID id);
bool bool_property(IInArchive *archive, UInt32 index, PROPID id);
std::wstring text_property(IInArchive *archive, UInt32 index, PROPID id);
uint32_t modified_dos(IInArchive *archive, UInt32 index);
struct ArchiveRow {
    std::string name;
    uint8_t flags = 0;
    uint16_t method = 0;
};
void seven_registry_check();
void read_archive(IInArchive *archive, const std::function<ArchiveRow(UInt32)> &row);
} // namespace xdzip
