// tools/native-codec/rar_backend.cpp

#include "protocol.hpp"
#include "rar.hpp"
#include <algorithm>
#include <cstring>

namespace xdzip {
static uint8_t rar_fault = 0;
static bool selected = false;
static int CALLBACK callback(UINT message, LPARAM, LPARAM first, LPARAM second) {
    try {
        if (message == UCM_PROCESSDATA) {
            if (second < 0)
                throw Failure{2};
            write_data(reinterpret_cast<void *>(first), size_t(second), selected);
            return 1;
        }
        if (message == UCM_NEEDPASSWORDW) {
            if (!request.has_password) {
                rar_fault = 6;
                return -1;
            }
            auto password = wide(request.password);
            if (password.size() + 1 > size_t(second)) {
                rar_fault = 4;
                return -1;
            }
            memcpy(reinterpret_cast<void *>(first), password.c_str(),
                   (password.size() + 1) * sizeof(wchar_t));
            return 1;
        }
        if (message == UCM_NEEDPASSWORD) {
            if (!request.has_password) {
                rar_fault = 6;
                return -1;
            }
            if (request.password.size() + 1 > size_t(second)) {
                rar_fault = 4;
                return -1;
            }
            memcpy(reinterpret_cast<void *>(first), request.password.c_str(),
                   request.password.size() + 1);
            return 1;
        }
        if (message == UCM_CHANGEVOLUMEW || message == UCM_CHANGEVOLUME)
            return io_fault ? -1
                            : 1; // Refuse a missing authorized volume instead of retrying forever.
        if (message == UCM_LARGEDICT) {
            rar_fault = 4;
            return -1;
        }
        return 0;
    } catch (const Failure &error) {
        rar_fault = error.code;
        return -1;
    } catch (...) {
        rar_fault = 4;
        return -1;
    }
}
static void check_rar(int code) {
    if (io_fault)
        throw Failure{io_fault};
    if (rar_fault)
        throw Failure{rar_fault};
    if (code == ERAR_SUCCESS)
        return;
    switch (code) {
    case ERAR_NO_MEMORY:
    case ERAR_LARGE_DICT:
        throw Failure{4};
    case ERAR_MISSING_PASSWORD:
        throw Failure{6};
    case ERAR_BAD_PASSWORD:
        throw Failure{7};
    case ERAR_UNKNOWN_FORMAT:
        throw Failure{3};
    case ERAR_EOPEN:
    case ERAR_EREAD:
        throw Failure{1};
    default:
        throw Failure{2};
    }
}
void rar_run() {
    if (request.operation != 0 && request.operation != 1 && request.operation != 3)
        throw Failure{3};
    std::wstring name = L"input.part1.rar";
    RAROpenArchiveDataEx open{};
    open.ArcNameW = &name[0];
    open.OpenMode = request.operation == 0 ? RAR_OM_LIST : RAR_OM_EXTRACT;
    open.Callback = callback;
    HANDLE archive = RAROpenArchiveEx(&open);
    check_rar(open.OpenResult);
    if (!archive)
        throw Failure{2};
    struct Close {
        HANDLE handle;
        ~Close() {
            RARCloseArchive(handle);
        }
    } close{archive};
    uint64_t index = 0;
    for (;;) {
        RARHeaderDataEx header{};
        std::vector<wchar_t> full_name(MAXPATHSIZE + 1);
        header.FileNameEx = full_name.data();
        header.FileNameExSize = uint32_t(full_name.size());
        int result = RARReadHeaderEx(archive, &header);
        if (result == ERAR_END_ARCHIVE)
            break;
        check_rar(result);
        if (index >= request.entries)
            throw Failure{4};
        std::wstring path = full_name[0] ? full_name.data() : header.FileNameW;
        if (path.size() >= MAXPATHSIZE)
            throw Failure{4}; // Never accept an API-truncated name as a shorter authorized path.
        if (header.HostOS != 3)
            std::replace(path.begin(), path.end(), L'\\', L'/');
        if (request.operation == 0) {
            uint8_t flags = uint8_t(unsigned(bool(header.Flags & RHDF_DIRECTORY)) |
                                    (unsigned(bool(header.Flags & RHDF_ENCRYPTED)) << 1) |
                                    (unsigned(header.RedirType != 0) << 2));
            if (header.UnpVer > UINT16_MAX)
                throw Failure{2};
            metadata(index, path, flags, (uint64_t(header.UnpSizeHigh) << 32) | header.UnpSize,
                     (uint64_t(header.PackSizeHigh) << 32) | header.PackSize, header.FileTime,
                     uint16_t(header.UnpVer));
            check_rar(RARProcessFile(archive, RAR_SKIP, nullptr, nullptr));
        } else {
            selected = index == request.selected;
            // RAR_TEST invokes process-data callbacks and verifies the actual stream.
            // Earlier solid records are decoded with callbacks charged as skipped work.
            bool decode =
                selected || (open.Flags & ROADF_SOLID) != 0 || (header.Flags & RHDF_SOLID) != 0;
            check_rar(RARProcessFile(archive, decode ? RAR_TEST : RAR_SKIP, nullptr, nullptr));
            if (selected) {
                if (request.operation == 3)
                    entry_done(index);
                else
                    return;
            }
        }
        index++;
    }
    if (request.operation == 1 || request.operation == 3)
        throw Failure{2};
}
} // namespace xdzip
