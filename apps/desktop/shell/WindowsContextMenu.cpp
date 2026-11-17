#include "MenuModel.h"
#include "PackageMenu.h"
#include <array>
#include <atomic>
#include <cwctype>
#include <limits>
#include <new>
#include <shlobj.h>

namespace {
// Stable identity belongs only to xdzip; registration is per current user.
const CLSID ClassId{0x731d9a38, 0x549e, 0x4cde, {0xb3, 0x5b, 0x6e, 0xb3, 0xa2, 0xc8, 0x8b, 0x71}};
// A distinct identity prevents Explorer's cached legacy class factory from
// answering the first IExplorerCommand activation after an in-place upgrade.
const CLSID CommandClassId{
    0x54b70170, 0x1072, 0x49b5, {0x9b, 0xd2, 0x6d, 0xfb, 0x5e, 0xcb, 0xf1, 0xb1}};
std::atomic<long> objects{0};
HMODULE module = nullptr;
enum class MenuKind { Installed, StoreClassic, SideloadClassic, StoreModern };
const xdzip_shell::PackageMenuContext &packageContext() {
    static const auto value = [] {
        std::wstring path(32768, L'\0');
        const auto size = GetModuleFileNameW(module, path.data(), DWORD(path.size()));
        if (!size || size >= path.size())
            return xdzip_shell::PackageMenuContext{};
        path.resize(size);
        return xdzip_shell::readPackageMenuContext(path);
    }();
    return value;
}
HBITMAP iconBitmap(HICON icon, int width, int height) {
    if (!icon || width <= 0 || height <= 0)
        return nullptr;
    BITMAPINFO info{};
    info.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    info.bmiHeader.biWidth = width;
    info.bmiHeader.biHeight = -height;
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB;
    void *pixels = nullptr;
    const auto bitmap = CreateDIBSection(nullptr, &info, DIB_RGB_COLORS, &pixels, nullptr, 0);
    if (!bitmap)
        return nullptr;
    const auto dc = CreateCompatibleDC(nullptr);
    if (!dc) {
        DeleteObject(bitmap);
        return nullptr;
    }
    const auto previous = SelectObject(dc, bitmap);
    const bool drawn = previous && previous != HGDI_ERROR &&
                       DrawIconEx(dc, 0, 0, icon, width, height, 0, nullptr, DI_NORMAL);
    if (previous && previous != HGDI_ERROR)
        SelectObject(dc, previous);
    DeleteDC(dc);
    if (!drawn) {
        DeleteObject(bitmap);
        return nullptr;
    }
    return bitmap;
}
UINT insertionIndex(HMENU menu, UINT fallback) {
    const int count = GetMenuItemCount(menu);
    for (int i = 0; i < count; ++i) {
        wchar_t text[512]{};
        if (!GetMenuStringW(menu, UINT(i), text, int(std::size(text)), MF_BYPOSITION))
            continue;
        std::wstring normalized;
        for (const auto ch : std::wstring(text))
            if (ch != L'&')
                normalized += wchar_t(std::towlower(ch));
        if (normalized.rfind(L"open with", 0) == 0 || normalized.rfind(L"打开方式", 0) == 0)
            return UINT(i);
    }
    return fallback;
}
std::wstring wide(const std::string &text) {
    const auto size = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, text.data(),
                                          int(text.size()), nullptr, 0);
    std::wstring result(size, L'\0');
    if (size)
        MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, text.data(), int(text.size()),
                            result.data(), size);
    return result;
}
bool menuEnglish(const std::wstring &key) {
    wchar_t language[16]{};
    DWORD bytes = sizeof(language);
    return RegGetValueW(HKEY_CURRENT_USER, key.c_str(), L"Language", RRF_RT_REG_SZ, nullptr,
                        language, &bytes) == ERROR_SUCCESS &&
           std::wstring(language) == L"en-US";
}
class CommandEnumerator;
class Menu final : public IShellExtInit, public IContextMenu, public IExplorerCommand {
    std::atomic<ULONG> refs{1};
    std::vector<xdzip_shell::fs::path> paths;
    std::vector<xdzip_shell::MenuEntry> entries;
    bool archiveCandidates = false;
    MenuKind kind = MenuKind::Installed;
    bool packaged() const {
        return kind != MenuKind::Installed;
    }
    bool allowedContext(const xdzip_shell::PackageMenuContext &context) const {
        if ((kind == MenuKind::SideloadClassic) == context.storeDistribution)
            return false;
        return xdzip_shell::packageMenuAllowed(context.registered, context.storeDistribution,
                                               context.storeSignature,
                                               kind == MenuKind::StoreModern, context.windows11);
    }
    bool allowed() const {
        return !packaged() || allowedContext(packageContext());
    }
    std::wstring settingsKey() const {
        return packaged() ? packageContext().settingsKey : L"Software\\xdzip\\Shell";
    }
    bool english() const {
        return menuEnglish(settingsKey());
    }
    bool enabled() const {
        if (!allowed())
            return false;
        DWORD value = 1, size = sizeof(value);
        RegGetValueW(HKEY_CURRENT_USER, settingsKey().c_str(), L"Enabled", RRF_RT_REG_DWORD,
                     nullptr, &value, &size);
        return value != 0;
    }
    static constexpr size_t RootCommand = std::numeric_limits<size_t>::max();
    size_t commandIndex = RootCommand;
    HRESULT select(IShellItemArray *selection) {
        if (!selection)
            return paths.empty() ? E_INVALIDARG : S_OK;
        IDataObject *data = nullptr;
        auto result = selection->BindToHandler(nullptr, BHID_DataObject, IID_PPV_ARGS(&data));
        if (FAILED(result))
            return result;
        result = Initialize(nullptr, data, nullptr);
        data->Release();
        if (SUCCEEDED(result))
            entries = xdzip_shell::menu(paths, english(), archiveCandidates);
        return result;
    }
    static HRESULT copyText(const std::wstring &text, PWSTR *value) {
        if (!value)
            return E_POINTER;
        // Explorer frees successful string results with CoTaskMemFree. Failure
        // leaves no borrowed storage crossing the COM boundary.
        *value = static_cast<PWSTR>(CoTaskMemAlloc((text.size() + 1) * sizeof(wchar_t)));
        if (!*value)
            return E_OUTOFMEMORY;
        std::copy(text.c_str(), text.c_str() + text.size() + 1, *value);
        return S_OK;
    }
    friend class CommandEnumerator;
    // A menu instance owns its bitmaps until Explorer releases the COM object.
    std::array<HBITMAP, 4> icons{};
    void prepareIcons() {
        const auto foreground = GetForegroundWindow();
        const auto dpi = foreground ? GetDpiForWindow(foreground) : GetDpiForSystem();
        const auto width = GetSystemMetricsForDpi(SM_CXMENUCHECK, dpi);
        const auto height = GetSystemMetricsForDpi(SM_CYMENUCHECK, dpi);
        for (int i = 0; i < 3; ++i) {
            if (icons[size_t(i)])
                continue;
            const auto icon = static_cast<HICON>(
                LoadImageW(module, MAKEINTRESOURCEW(101 + i), IMAGE_ICON, width, height, 0));
            if (icon) {
                icons[size_t(i)] = iconBitmap(icon, width, height);
                DestroyIcon(icon);
            }
        }
        if (!icons[3]) {
            SHSTOCKICONINFO stock{};
            stock.cbSize = sizeof(stock);
            if (SUCCEEDED(SHGetStockIconInfo(SIID_FOLDER, SHGSI_ICON | SHGSI_SMALLICON, &stock))) {
                icons[3] = iconBitmap(stock.hIcon, width, height);
                DestroyIcon(stock.hIcon);
            }
        }
    }

  public:
    explicit Menu(MenuKind value = MenuKind::Installed) : kind(value) {
        ++objects;
    }
    Menu(const Menu &source, size_t index)
        : paths(source.paths), entries(source.entries), archiveCandidates(source.archiveCandidates),
          kind(source.kind), commandIndex(index) {
        ++objects;
    }
    ~Menu() {
        for (const auto bitmap : icons)
            if (bitmap)
                DeleteObject(bitmap);
        --objects;
    }
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID id, void **value) override {
        if (!value)
            return E_POINTER;
        *value = nullptr;
        if (id == IID_IUnknown || id == IID_IShellExtInit)
            *value = static_cast<IShellExtInit *>(this);
        else if (id == IID_IContextMenu && kind != MenuKind::StoreModern)
            *value = static_cast<IContextMenu *>(this);
        else if (id == IID_IExplorerCommand &&
                 (kind == MenuKind::Installed || kind == MenuKind::StoreModern))
            *value = static_cast<IExplorerCommand *>(this);
        if (!*value)
            return E_NOINTERFACE;
        AddRef();
        return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override {
        return ++refs;
    }
    ULONG STDMETHODCALLTYPE Release() override {
        const auto n = --refs;
        if (!n)
            delete this;
        return n;
    }
    HRESULT STDMETHODCALLTYPE Initialize(PCIDLIST_ABSOLUTE, IDataObject *data, HKEY) override {
        paths.clear();
        if (!data)
            return E_INVALIDARG;
        FORMATETC format{CF_HDROP, nullptr, DVASPECT_CONTENT, -1, TYMED_HGLOBAL};
        STGMEDIUM medium{};
        if (FAILED(data->GetData(&format, &medium)))
            return E_INVALIDARG;
        // IDataObject owns its source. ReleaseStgMedium releases our returned
        // reference after bounded Unicode names have been copied.
        const auto drop = static_cast<HDROP>(medium.hGlobal);
        const auto count = DragQueryFileW(drop, 0xffffffff, nullptr, 0);
        if (count && count <= 128)
            for (UINT i = 0; i < count; ++i) {
                const auto size = DragQueryFileW(drop, i, nullptr, 0);
                if (!size || size > 4096) {
                    paths.clear();
                    break;
                }
                std::wstring path(size + 1, L'\0');
                DragQueryFileW(drop, i, path.data(), size + 1);
                path.resize(size);
                paths.emplace_back(path);
            }
        ReleaseStgMedium(&medium);
        archiveCandidates =
            !paths.empty() && std::all_of(paths.begin(), paths.end(), [](const auto &path) {
                std::error_code error;
                if (xdzip_shell::fs::is_directory(path, error))
                    return false;
                if (xdzip_shell::archiveCandidate(path))
                    return true;
                wchar_t mime[128]{};
                DWORD size = sizeof(mime);
                return RegGetValueW(HKEY_CLASSES_ROOT, path.extension().c_str(), L"Content Type",
                                    RRF_RT_REG_SZ, nullptr, mime, &size) == ERROR_SUCCESS &&
                       xdzip_shell::archiveMime(xdzip_shell::fs::path(mime).u8string());
            });
        return paths.empty() ? E_INVALIDARG : S_OK;
    }
    HRESULT STDMETHODCALLTYPE QueryContextMenu(HMENU parent, UINT index, UINT first, UINT last,
                                               UINT flags) override {
        if ((flags & CMF_DEFAULTONLY) || !enabled())
            return MAKE_HRESULT(SEVERITY_SUCCESS, 0, 0);
        try {
            DWORD enabled = 1, size = sizeof(enabled);
            RegGetValueW(HKEY_CURRENT_USER, L"Software\\xdzip\\Shell", L"Enabled", RRF_RT_REG_DWORD,
                         nullptr, &enabled, &size);
            if (!packaged() && !enabled)
                return MAKE_HRESULT(SEVERITY_SUCCESS, 0, 0);
            entries = xdzip_shell::menu(paths, english(), archiveCandidates);
            if (entries.empty() || last < first || last - first + 1 < entries.size())
                return E_FAIL;
            const auto submenu = CreatePopupMenu();
            if (!submenu)
                return E_OUTOFMEMORY;
            prepareIcons();
            for (size_t i = 0; i < entries.size(); ++i) {
                auto label = wide(entries[i].label);
                for (size_t at = 0; (at = label.find(L'&', at)) != std::wstring::npos; at += 2)
                    label.insert(at, 1, L'&');
                if (!AppendMenuW(submenu, MF_STRING, first + UINT(i), label.c_str())) {
                    DestroyMenu(submenu);
                    return E_FAIL;
                }
                MENUITEMINFOW child{};
                child.cbSize = sizeof(child);
                child.fMask = MIIM_BITMAP;
                child.hbmpItem = entries[i].action == "7z"                     ? icons[2]
                                 : entries[i].action.rfind("extract-", 0) == 0 ? icons[3]
                                                                               : icons[1];
                if (child.hbmpItem)
                    SetMenuItemInfoW(submenu, UINT(i), TRUE, &child);
            }
            MENUITEMINFOW item{};
            item.cbSize = sizeof(item);
            item.fMask = MIIM_ID | MIIM_STRING | MIIM_SUBMENU | (icons[0] ? MIIM_BITMAP : 0);
            item.wID = first;
            item.hbmpItem = icons[0];
            item.hSubMenu = submenu;
            auto title = wide(xdzip_shell::text(english(), "shell.title"));
            item.dwTypeData = title.data();
            if (!InsertMenuItemW(parent, insertionIndex(parent, index), TRUE, &item)) {
                DestroyMenu(submenu);
                return E_FAIL;
            }
            return MAKE_HRESULT(SEVERITY_SUCCESS, 0, USHORT(entries.size()));
        } catch (...) {
            return E_FAIL;
        }
    }
    HRESULT STDMETHODCALLTYPE InvokeCommand(CMINVOKECOMMANDINFO *info) override {
        if (!info)
            return E_POINTER;
        if (!enabled())
            return E_ACCESSDENIED;
        try {
            size_t index = entries.size();
            if (!HIWORD(info->lpVerb))
                index = LOWORD(info->lpVerb);
            else
                for (size_t i = 0; i < entries.size(); ++i)
                    if ("xdzip." + entries[i].action == info->lpVerb)
                        index = i;
            if (index >= entries.size())
                return E_INVALIDARG;
            std::wstring path(32768, L'\0');
            const auto size = GetModuleFileNameW(module, path.data(), DWORD(path.size()));
            if (!size || size >= path.size())
                return E_FAIL;
            path.resize(size);
            // A loaded surrogate can outlive an upgrade or uninstall. Recheck
            // the DLL's own registration before activating any package target.
            const auto current = packaged() ? xdzip_shell::readPackageMenuContext(path)
                                            : xdzip_shell::PackageMenuContext{};
            if (packaged() && !allowedContext(current))
                return HRESULT_FROM_WIN32(ERROR_REVISION_MISMATCH);
            auto root = xdzip_shell::fs::path(path).parent_path();
            if (root.filename() == L"shell")
                root = root.parent_path();
            const auto executable = root / L"xdzip.exe";
            if ((!packaged() && !xdzip_shell::registeredTargetMatches(executable)) ||
                !xdzip_shell::supportsShellProtocol(executable))
                return HRESULT_FROM_WIN32(ERROR_REVISION_MISMATCH);
            return xdzip_shell::launch(executable, entries[index].action, paths, archiveCandidates,
                                       {}, packaged() ? current.applicationId : std::wstring())
                       ? S_OK
                       : E_FAIL;
        } catch (...) {
            return E_FAIL;
        }
    }
    HRESULT STDMETHODCALLTYPE GetCommandString(UINT_PTR id, UINT flags, UINT *, LPSTR buffer,
                                               UINT count) override {
        if (flags == GCS_VALIDATEA || flags == GCS_VALIDATEW)
            return id < entries.size() ? S_OK : E_INVALIDARG;
        if (id >= entries.size() || !buffer || !count)
            return E_INVALIDARG;
        const auto verb = "xdzip." + entries[id].action;
        if (flags == GCS_VERBW) {
            const auto value = wide(verb);
            if (count <= value.size())
                return E_INVALIDARG;
            std::copy(value.begin(), value.end(), reinterpret_cast<wchar_t *>(buffer));
            reinterpret_cast<wchar_t *>(buffer)[value.size()] = 0;
        } else if (flags == GCS_VERBA) {
            if (count <= verb.size())
                return E_INVALIDARG;
            std::copy(verb.begin(), verb.end(), buffer);
            buffer[verb.size()] = 0;
        } else
            return E_NOTIMPL;
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE GetTitle(IShellItemArray *selection, PWSTR *title) override {
        if (!title)
            return E_POINTER;
        *title = nullptr;
        try {
            const auto result = select(selection);
            if (FAILED(result))
                return result;
            if (commandIndex != RootCommand && commandIndex >= entries.size())
                return E_INVALIDARG;
            return copyText(wide(commandIndex == RootCommand
                                     ? xdzip_shell::text(english(), "shell.title")
                                     : entries[commandIndex].label),
                            title);
        } catch (...) {
            return E_FAIL;
        }
    }
    HRESULT STDMETHODCALLTYPE GetIcon(IShellItemArray *, PWSTR *icon) override {
        if (!icon)
            return E_POINTER;
        *icon = nullptr;
        try {
            int resource = 101;
            if (commandIndex != RootCommand) {
                if (commandIndex >= entries.size())
                    return E_INVALIDARG;
                const auto &action = entries[commandIndex].action;
                if (action.rfind("extract-", 0) == 0) {
                    SHSTOCKICONINFO stock{};
                    stock.cbSize = sizeof(stock);
                    const auto result = SHGetStockIconInfo(SIID_FOLDER, SHGSI_ICONLOCATION, &stock);
                    if (FAILED(result))
                        return result;
                    return copyText(
                        std::wstring(stock.szPath) + L"," + std::to_wstring(stock.iIcon), icon);
                }
                resource = action == "7z" ? 103 : 102;
            }
            std::wstring path(32768, L'\0');
            const auto size = GetModuleFileNameW(module, path.data(), DWORD(path.size()));
            if (!size || size >= path.size())
                return E_FAIL;
            path.resize(size);
            return copyText(path + L",-" + std::to_wstring(resource), icon);
        } catch (...) {
            return E_FAIL;
        }
    }
    HRESULT STDMETHODCALLTYPE GetToolTip(IShellItemArray *, PWSTR *tip) override {
        if (!tip)
            return E_POINTER;
        *tip = nullptr;
        return E_NOTIMPL;
    }
    HRESULT STDMETHODCALLTYPE GetCanonicalName(GUID *name) override {
        if (!name)
            return E_POINTER;
        *name = CommandClassId;
        if (commandIndex != RootCommand) {
            if (commandIndex >= entries.size())
                return E_INVALIDARG;
            static const std::array<const char *, 8> actions{"zip",
                                                             "7z",
                                                             "create",
                                                             "separate-zip",
                                                             "extract-here",
                                                             "extract-smart",
                                                             "extract-separate",
                                                             "extract-to"};
            const auto found =
                std::find(actions.begin(), actions.end(), entries[commandIndex].action);
            if (found == actions.end())
                return E_INVALIDARG;
            name->Data1 ^= DWORD(found - actions.begin() + 1);
        }
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE GetState(IShellItemArray *selection, BOOL,
                                       EXPCMDSTATE *state) override {
        if (!state)
            return E_POINTER;
        *state = ECS_HIDDEN;
        try {
            if (!enabled())
                return S_OK;
            DWORD enabled = 1, size = sizeof(enabled);
            RegGetValueW(HKEY_CURRENT_USER, L"Software\\xdzip\\Shell", L"Enabled", RRF_RT_REG_DWORD,
                         nullptr, &enabled, &size);
            wchar_t legacy[64]{};
            DWORD legacySize = sizeof(legacy);
            const bool rolledBack =
                RegGetValueW(
                    HKEY_CURRENT_USER,
                    L"Software\\Classes\\AllFilesystemObjects\\shellex\\ContextMenuHandlers\\xdzip",
                    nullptr, RRF_RT_REG_SZ, nullptr, legacy, &legacySize) == ERROR_SUCCESS &&
                _wcsicmp(legacy, L"{731D9A38-549E-4CDE-B35B-6EB3A2C88B71}") == 0;
            // Old installers restore the legacy handler. Hide the retained
            // newer command so rollback presents only that compatible entry.
            if (!packaged() && rolledBack)
                return S_OK;
            if ((packaged() || enabled) && SUCCEEDED(select(selection)) && !entries.empty() &&
                (commandIndex == RootCommand || commandIndex < entries.size()))
                *state = ECS_ENABLED;
            return S_OK;
        } catch (...) {
            return E_FAIL;
        }
    }
    HRESULT STDMETHODCALLTYPE Invoke(IShellItemArray *selection, IBindCtx *) override {
        try {
            if (commandIndex == RootCommand)
                return E_NOTIMPL;
            const auto result = select(selection);
            if (FAILED(result) || commandIndex >= entries.size())
                return E_INVALIDARG;
            CMINVOKECOMMANDINFO info{};
            info.cbSize = sizeof(info);
            info.lpVerb = MAKEINTRESOURCEA(commandIndex);
            return InvokeCommand(&info);
        } catch (...) {
            return E_FAIL;
        }
    }
    HRESULT STDMETHODCALLTYPE GetFlags(EXPCMDFLAGS *flags) override {
        if (!flags)
            return E_POINTER;
        *flags = commandIndex == RootCommand ? ECF_HASSUBCOMMANDS : ECF_DEFAULT;
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE EnumSubCommands(IEnumExplorerCommand **commands) override;
};
class CommandEnumerator final : public IEnumExplorerCommand {
    std::atomic<ULONG> refs{1};
    std::vector<Menu *> commands;
    size_t position = 0;

  public:
    explicit CommandEnumerator(const Menu &source) {
        ++objects;
        try {
            commands.reserve(source.entries.size());
            for (size_t i = 0; i < source.entries.size(); ++i)
                commands.push_back(new Menu(source, i));
        } catch (...) {
            for (auto *command : commands)
                command->Release();
            --objects;
            throw;
        }
    }
    CommandEnumerator(const CommandEnumerator &source)
        : commands(source.commands), position(source.position) {
        ++objects;
        for (auto *command : commands)
            command->AddRef();
    }
    ~CommandEnumerator() {
        for (auto *command : commands)
            command->Release();
        --objects;
    }
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID id, void **value) override {
        if (!value)
            return E_POINTER;
        *value = nullptr;
        if (id != IID_IUnknown && id != IID_IEnumExplorerCommand)
            return E_NOINTERFACE;
        *value = static_cast<IEnumExplorerCommand *>(this);
        AddRef();
        return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override {
        return ++refs;
    }
    ULONG STDMETHODCALLTYPE Release() override {
        const auto count = --refs;
        if (!count)
            delete this;
        return count;
    }
    HRESULT STDMETHODCALLTYPE Next(ULONG count, IExplorerCommand **values,
                                   ULONG *fetched) override {
        if (!values || (count != 1 && !fetched))
            return E_POINTER;
        ULONG read = 0;
        for (; read < count && position < commands.size(); ++read, ++position) {
            values[read] = commands[position];
            values[read]->AddRef();
        }
        if (fetched)
            *fetched = read;
        return read == count ? S_OK : S_FALSE;
    }
    HRESULT STDMETHODCALLTYPE Skip(ULONG count) override {
        const auto remaining = commands.size() - position;
        position += std::min(size_t(count), remaining);
        return count <= remaining ? S_OK : S_FALSE;
    }
    HRESULT STDMETHODCALLTYPE Reset() override {
        position = 0;
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE Clone(IEnumExplorerCommand **copy) override {
        if (!copy)
            return E_POINTER;
        *copy = nullptr;
        try {
            *copy = new CommandEnumerator(*this);
            return S_OK;
        } catch (...) {
            return E_OUTOFMEMORY;
        }
    }
};
HRESULT Menu::EnumSubCommands(IEnumExplorerCommand **commands) {
    if (!commands)
        return E_POINTER;
    *commands = nullptr;
    if (commandIndex != RootCommand)
        return E_NOTIMPL;
    try {
        *commands = new CommandEnumerator(*this);
        return S_OK;
    } catch (...) {
        return E_OUTOFMEMORY;
    }
}
class Factory final : public IClassFactory {
    std::atomic<ULONG> refs{1};
    MenuKind kind;

  public:
    explicit Factory(MenuKind value) : kind(value) {
        ++objects;
    }
    ~Factory() {
        --objects;
    }
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID id, void **value) override {
        if (!value)
            return E_POINTER;
        *value = nullptr;
        if (id != IID_IUnknown && id != IID_IClassFactory)
            return E_NOINTERFACE;
        *value = static_cast<IClassFactory *>(this);
        AddRef();
        return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override {
        return ++refs;
    }
    ULONG STDMETHODCALLTYPE Release() override {
        const auto n = --refs;
        if (!n)
            delete this;
        return n;
    }
    HRESULT STDMETHODCALLTYPE CreateInstance(IUnknown *outer, REFIID id, void **value) override {
        if (outer)
            return CLASS_E_NOAGGREGATION;
        auto *object = new (std::nothrow) Menu(kind);
        if (!object)
            return E_OUTOFMEMORY;
        const auto result = object->QueryInterface(id, value);
        object->Release();
        return result;
    }
    HRESULT STDMETHODCALLTYPE LockServer(BOOL lock) override {
        objects += lock ? 1 : -1;
        return S_OK;
    }
};
} // namespace
extern "C" BOOL WINAPI DllMain(HINSTANCE instance, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        module = instance;
        DisableThreadLibraryCalls(instance);
    }
    return TRUE;
}
STDAPI DllGetClassObject(REFCLSID id, REFIID iid, void **value) {
    CLSID storeClassic{}, sideloadClassic{}, modern{};
    CLSIDFromString(xdzip_shell::PackageClassicStoreId, &storeClassic);
    CLSIDFromString(xdzip_shell::PackageClassicSideloadId, &sideloadClassic);
    CLSIDFromString(xdzip_shell::PackageModernId, &modern);
    if (id != ClassId && id != CommandClassId && id != storeClassic && id != sideloadClassic &&
        id != modern)
        return CLASS_E_CLASSNOTAVAILABLE;
    const auto kind = id == storeClassic      ? MenuKind::StoreClassic
                      : id == sideloadClassic ? MenuKind::SideloadClassic
                      : id == modern          ? MenuKind::StoreModern
                                              : MenuKind::Installed;
    auto *factory = new (std::nothrow) Factory(kind);
    if (!factory)
        return E_OUTOFMEMORY;
    const auto result = factory->QueryInterface(iid, value);
    factory->Release();
    return result;
}
STDAPI DllCanUnloadNow() {
    return objects == 0 ? S_OK : S_FALSE;
}
