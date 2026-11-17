#pragma once
#include <windows.h>

#include <appmodel.h>
#include <filesystem>
#include <fstream>
#include <roapi.h>
#include <vector>
#include <winrt/Windows.ApplicationModel.h>
#include <winrt/Windows.Management.Deployment.h>

namespace xdzip_shell {
inline constexpr wchar_t PackageClassicStoreId[] = L"{D1915E65-8E17-4B3C-BB61-4E34A1551A25}";
inline constexpr wchar_t PackageClassicSideloadId[] = L"{635731B5-F385-42E3-984D-894217682DAA}";
inline constexpr wchar_t PackageModernId[] = L"{50F4499A-05FD-497B-9F72-C39041672093}";

inline bool packageMenuAllowed(bool registered, bool storeDistribution, bool storeSignature,
                               bool modern, bool windows11) {
    return registered && (!modern || (storeDistribution && storeSignature && windows11));
}
struct PackageMenuContext {
    bool registered = false;
    bool storeDistribution = false;
    bool storeSignature = false;
    bool windows11 = false;
    std::filesystem::path root;
    std::wstring applicationId;
    std::wstring settingsKey;
};
inline PackageMenuContext readPackageMenuContext(const std::filesystem::path &binary) noexcept {
    PackageMenuContext result;
    try {
        auto root = binary.parent_path();
        if (root.filename() == L"shell")
            root = root.parent_path();
        const auto fullName = root.filename().wstring();
        UINT32 size = 0;
        if (GetPackagePathByFullName(fullName.c_str(), &size, nullptr) !=
                ERROR_INSUFFICIENT_BUFFER ||
            size < 2 || size > 32768)
            return result;
        std::wstring path(size, L'\0');
        if (GetPackagePathByFullName(fullName.c_str(), &size, path.data()) != ERROR_SUCCESS)
            return result;
        std::error_code error;
        if (!std::filesystem::equivalent(root, std::filesystem::path(path.c_str()), error) || error)
            return result;
        size = 0;
        if (PackageFamilyNameFromFullName(fullName.c_str(), &size, nullptr) !=
                ERROR_INSUFFICIENT_BUFFER ||
            size < 2 || size > 1024)
            return result;
        std::wstring family(size, L'\0');
        if (PackageFamilyNameFromFullName(fullName.c_str(), &size, family.data()) != ERROR_SUCCESS)
            return result;
        family.resize(size - 1);
        UINT32 count = 0, characters = 0;
        if (GetPackagesByPackageFamily(family.c_str(), &count, nullptr, &characters, nullptr) !=
                ERROR_INSUFFICIENT_BUFFER ||
            !count || count > 64 || !characters || characters > 65536)
            return result;
        std::vector<PWSTR> names(count);
        std::vector<wchar_t> buffer(characters);
        if (GetPackagesByPackageFamily(family.c_str(), &count, names.data(), &characters,
                                       buffer.data()) != ERROR_SUCCESS)
            return result;
        bool current = false;
        for (UINT32 index = 0; index < count; ++index)
            current = current || fullName == names[index];
        if (!current)
            return result;
        std::ifstream marker(root / L"xdzip-msix-channel.txt", std::ios::binary);
        char bytes[64]{};
        marker.read(bytes, sizeof(bytes));
        const std::string channel(bytes, size_t(marker.gcount()));
        if (channel != "xdzip-msix-v1\nstore\n" && channel != "xdzip-msix-v1\nsideload\n")
            return result;
        result.registered = true;
        result.storeDistribution = channel == "xdzip-msix-v1\nstore\n";
        result.root = root;
        result.applicationId = family + L"!App";
        result.settingsKey = L"Software\\xdzip\\Shell\\Packages\\" + family;
        // A COM host's compatibility manifest must not make Windows 11 look
        // like Windows 8. Use the system ntdll ABI; missing/failed queries deny.
        using SystemVersion = LONG(WINAPI *)(PRTL_OSVERSIONINFOW);
        const auto system = GetModuleHandleW(L"ntdll.dll");
        const auto queryVersion =
            system ? reinterpret_cast<SystemVersion>(GetProcAddress(system, "RtlGetVersion"))
                   : nullptr;
        RTL_OSVERSIONINFOW version{};
        version.dwOSVersionInfoSize = sizeof(version);
        result.windows11 = queryVersion && queryVersion(&version) == 0 &&
                           version.dwMajorVersion >= 10 && version.dwBuildNumber >= 22000;
        if (result.storeDistribution) {
            // Bind signing evidence to this DLL's registered package, never to
            // Explorer's process. WinRT activation may fail in a host; fail closed.
            const auto initialized = RoInitialize(RO_INIT_MULTITHREADED);
            if (SUCCEEDED(initialized) || initialized == RPC_E_CHANGED_MODE) {
                try {
                    const auto package =
                        winrt::Windows::Management::Deployment::PackageManager().FindPackageForUser(
                            L"", fullName);
                    result.storeSignature =
                        package.SignatureKind() ==
                        winrt::Windows::ApplicationModel::PackageSignatureKind::Store;
                } catch (...) {
                    result.storeSignature = false;
                }
                if (SUCCEEDED(initialized))
                    RoUninitialize();
            }
        }
    } catch (...) {
        return {};
    }
    return result;
}
} // namespace xdzip_shell
