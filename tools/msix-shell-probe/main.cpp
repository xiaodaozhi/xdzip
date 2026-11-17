// Owned, non-product fixture for disposable Windows package tests. It never
// extracts archives or changes defaults. Production menu code is built unchanged.
#include "PackageMenu.h"
#include <iostream>
#include <shlobj.h>
#include <windows.storage.h>

namespace fs = std::filesystem;
using Microsoft::WRL::ComPtr;
using Microsoft::WRL::Wrappers::HString;
using Microsoft::WRL::Wrappers::HStringReference;
struct ComFailure {
    HRESULT value;
    const char *operation;
};
void check(HRESULT value, const char *operation = "COM call") {
    if (FAILED(value))
        throw ComFailure{value, operation};
}
void require(bool condition, const char *message) {
    if (!condition)
        throw std::runtime_error(message);
}
fs::path executable() {
    std::wstring path(32768, L'\0');
    const auto size = GetModuleFileNameW(nullptr, path.data(), DWORD(path.size()));
    require(size && size < path.size(), "Fixture module path failed");
    path.resize(size);
    return fs::path(path);
}
std::string read(const fs::path &path) {
    std::ifstream input(path, std::ios::binary);
    char bytes[65536]{};
    input.read(bytes, sizeof(bytes));
    require(input.gcount() < std::streamsize(sizeof(bytes)), "Fixture receipt too large");
    return std::string(bytes, size_t(input.gcount()));
}
void receive(const fs::path &request) {
    require(xdzip_shell::readPackageMenuContext(executable()).registered,
            "Fixture activation has no own registered package");
    const auto bytes = read(request);
    require(bytes.find("\"action\":\"zip\"") != std::string::npos,
            "Fixture activation received wrong action");
    ComPtr<ABI::Windows::Storage::IApplicationDataStatics> factory;
    check(RoGetActivationFactory(HStringReference(L"Windows.Storage.ApplicationData").Get(),
                                 IID_PPV_ARGS(factory.GetAddressOf())));
    ComPtr<ABI::Windows::Storage::IApplicationData> application;
    check(factory->get_Current(&application));
    ComPtr<ABI::Windows::Storage::IStorageFolder> local;
    check(application->get_LocalFolder(&local));
    ComPtr<ABI::Windows::Storage::IStorageItem> localItem;
    check(local.As(&localItem));
    HString location;
    check(localItem->get_Path(location.GetAddressOf()));
    const fs::path folder(location.GetRawBuffer(nullptr));
    std::ofstream receipt(folder / L"shell-receipt.txt", std::ios::binary);
    receipt << "activated-package-zip\n";
    receipt.close();
    require(bool(receipt), "Fixture receipt write failed");
    // NativeRequest created this exact private file and directory. Never recurse.
    require(request.filename() == L"request.json" &&
                request.parent_path().filename().wstring().rfind(L"xdzip-shell-", 0) == 0,
            "Unexpected fixture request location");
    fs::remove(request);
    fs::remove(request.parent_path());
}
void verify(const fs::path &root, const std::wstring &family, bool store) {
    const auto context = xdzip_shell::readPackageMenuContext(root / L"xdzip-shell.dll");
    require(context.registered && context.storeDistribution == store && context.windows11 &&
                (!store || context.signatureKnown) && !context.storeSignature &&
                context.applicationId == family + L"!App",
            "Own-package channel or self-signature policy failed");
    CLSID classic{}, modern{};
    check(CLSIDFromString(store ? xdzip_shell::PackageClassicStoreId
                                : xdzip_shell::PackageClassicSideloadId,
                          &classic));
    check(CLSIDFromString(xdzip_shell::PackageModernId, &modern));
    // Real packaged COM activation runs the production DLL in the OS surrogate.
    // RAII releases every acquired interface. HRESULT failure rejects the check.
    ComPtr<IUnknown> remote;
    check(CoCreateInstance(classic, nullptr, CLSCTX_LOCAL_SERVER,
                           IID_PPV_ARGS(remote.GetAddressOf())),
          "activate packaged classic COM");
    remote = nullptr;
    ComPtr<IUnknown> modernRemote;
    const auto modernResult = CoCreateInstance(modern, nullptr, CLSCTX_LOCAL_SERVER,
                                               IID_PPV_ARGS(modernRemote.GetAddressOf()));
    require(store ? SUCCEEDED(modernResult) : FAILED(modernResult),
            "Modern COM registration differs from channel");
    modernRemote = nullptr;
    // Inspect native HMENU and invoke without cross-process HMENU marshalling.
    // The trusted input is the exact DLL installed by the OS for this fixture.
    const auto library = LoadLibraryW((root / L"xdzip-shell.dll").c_str());
    require(library != nullptr, "Installed menu load failed");
    using GetClass = HRESULT(WINAPI *)(REFCLSID, REFIID, void **);
    const auto getClass = reinterpret_cast<GetClass>(GetProcAddress(library, "DllGetClassObject"));
    require(getClass != nullptr, "Installed class factory missing");
    ComPtr<IClassFactory> factory;
    check(getClass(classic, IID_PPV_ARGS(factory.GetAddressOf())));
    ComPtr<IContextMenu> menu;
    check(factory->CreateInstance(nullptr, IID_PPV_ARGS(menu.GetAddressOf())));
    ComPtr<IExplorerCommand> command;
    require(FAILED(menu->QueryInterface(IID_PPV_ARGS(command.GetAddressOf()))),
            "Classic class also exposes modern interface");
    if (store) {
        ComPtr<IClassFactory> modernFactory;
        check(getClass(modern, IID_PPV_ARGS(modernFactory.GetAddressOf())));
        check(modernFactory->CreateInstance(nullptr, IID_PPV_ARGS(command.GetAddressOf())));
        EXPCMDSTATE state = ECS_ENABLED;
        check(command->GetState(nullptr, FALSE, &state));
        require(state == ECS_HIDDEN, "Self-signed Store flavor enables modern menu");
        command = nullptr;
    }
    const auto selected = fs::current_path() / L"owned-selection.txt";
    std::ofstream(selected) << "owned fixture\n";
    const auto pidl = ILCreateFromPathW(selected.c_str());
    require(pidl != nullptr, "Fixture selection failed");
    PCIDLIST_ABSOLUTE ids[] = {pidl};
    ComPtr<IShellItemArray> selection;
    const auto selectionResult =
        SHCreateShellItemArrayFromIDLists(1, ids, selection.GetAddressOf());
    CoTaskMemFree(pidl);
    check(selectionResult, "create selected shell items");
    ComPtr<IDataObject> data;
    check(selection->BindToHandler(nullptr, BHID_DataObject, IID_PPV_ARGS(data.GetAddressOf())),
          "bind selected files");
    ComPtr<IShellExtInit> init;
    check(menu.As(&init));
    check(init->Initialize(nullptr, data.Get(), nullptr), "initialize classic menu");
    const auto popup = CreatePopupMenu();
    require(popup != nullptr, "Fixture popup creation failed");
    const auto query = menu->QueryContextMenu(popup, 0, 1000, 2000, 0);
    const auto count = GetMenuItemCount(popup);
    DestroyMenu(popup);
    require(SUCCEEDED(query) && HRESULT_CODE(query) > 0 && count == 1,
            "Registered classic menu is empty");
    CMINVOKECOMMANDINFO invoke{};
    invoke.cbSize = sizeof(invoke);
    invoke.nShow = SW_SHOWNORMAL;
    check(menu->InvokeCommand(&invoke), "invoke classic menu");
    wchar_t localData[32768]{};
    const auto environmentSize =
        GetEnvironmentVariableW(L"LOCALAPPDATA", localData, DWORD(std::size(localData)));
    require(environmentSize && environmentSize < std::size(localData), "Missing local app data");
    const auto receipt =
        fs::path(localData) / L"Packages" / family / L"LocalState" / L"shell-receipt.txt";
    bool received = false;
    for (int attempt = 0; attempt < 100 && !received; ++attempt) {
        received = read(receipt) == "activated-package-zip\n";
        if (!received)
            Sleep(100);
    }
    require(received, "Package activation receipt missing");
    fs::remove(selected);
    // Keep the module loaded until all COM objects leave this scope.
    std::cout << "Packaged COM registration, classic menu and package activation passed\n";
}
int wmain(int argc, wchar_t **argv) {
    if (argc == 2 && std::wstring(argv[1]) == L"--version-json") {
        std::cout << "{\"version\":\"0.2.0\",\"build\":27,\"revision\":\"fixture\","
                     "\"dirty\":false,\"packaged_update_policy\":1,"
                     "\"packaged_shell_policy\":1,\"packaged_association_policy\":1}\n";
        return 0;
    }
    const auto initialized = RoInitialize(RO_INIT_SINGLETHREADED);
    if (FAILED(initialized))
        return 1;
    int result = 0;
    try {
        if (argc == 3 && std::wstring(argv[1]) == L"--shell-request")
            receive(argv[2]);
        else if (argc == 5 && std::wstring(argv[1]) == L"--verify")
            verify(argv[2], argv[3], std::wstring(argv[4]) == L"store");
        else
            result = 2;
    } catch (const ComFailure &error) {
        std::cerr << "Fixture COM error in " << error.operation << ": " << std::hex
                  << unsigned(error.value) << '\n';
        result = 1;
    } catch (const std::exception &error) {
        std::cerr << error.what() << '\n';
        result = 1;
    }
    RoUninitialize();
    return result;
}
