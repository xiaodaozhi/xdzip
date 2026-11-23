#include "MenuModel.h"
#include <gio/gio.h>
#include <gmodule.h>
#include <nautilus-extension.h>

namespace {
struct Extension {
    GObject parent;
};
struct ExtensionClass {
    GObjectClass parent;
};
GType extensionType = 0;
#ifdef XDZIP_NAUTILUS_SNAP
#ifndef XDZIP_SNAP_MARKER
#define XDZIP_SNAP_MARKER "/snap/xdzip/current/meta/xdzip-shell-protocol"
#endif
#ifndef XDZIP_SNAP_COMMAND
#define XDZIP_SNAP_COMMAND "/usr/bin/snap"
#endif
bool snapAvailable() {
    return g_file_test(XDZIP_SNAP_COMMAND, G_FILE_TEST_IS_EXECUTABLE) &&
           xdzip_shell::protocolMarker(XDZIP_SNAP_MARKER);
}
#endif
std::pair<bool, std::string> preferences() {
    auto *settings = g_key_file_new();
#ifdef XDZIP_NAUTILUS_SNAP
    const std::string file = std::string(g_get_home_dir()) + "/snap/xdzip/current/.config/xdzip/settings.ini";
#else
    const std::string file = std::string(g_get_user_config_dir()) + "/xdzip/settings.ini";
#endif
    g_key_file_load_from_file(settings, file.c_str(), G_KEY_FILE_NONE, nullptr);
    const bool enabled = !g_key_file_has_key(settings, "desktop", "shellEnabled", nullptr) ||
                         g_key_file_get_boolean(settings, "desktop", "shellEnabled", nullptr);
    auto *legacy = g_key_file_get_string(settings, "locale", "language", nullptr);
    auto *extended = g_key_file_get_string(settings, "locale", "extendedLanguage", nullptr);
    auto selected = xdzip_i18n::storedLanguage(legacy ? legacy : "auto", extended ? extended : "");
    const auto *system = g_get_language_names()[0];
    auto language = std::string(
        selected == "auto" ? xdzip_i18n::systemLanguage(system ? system : "") : selected);
    g_free(legacy);
    g_free(extended);
    g_key_file_unref(settings);
    return {enabled, std::move(language)};
}
struct Invocation {
    std::vector<xdzip_shell::fs::path> paths;
    std::string action;
    bool archiveCandidates;
};
#ifdef XDZIP_NAUTILUS_SNAP
bool launchSnap(const Invocation &call) {
    const auto bytes = xdzip_shell::requestBytes(call.action, call.paths, call.archiveCandidates, {},
                                                  xdzip_shell::randomHex());
    if (bytes.empty() || !snapAvailable())
        return false;
    auto *process = g_subprocess_new(G_SUBPROCESS_FLAGS_STDIN_PIPE, nullptr, XDZIP_SNAP_COMMAND,
                                     "run", "xdzip", "--shell-request-stdin", nullptr);
    if (!process)
        return false;
    auto *buffer = g_bytes_new(bytes.data(), bytes.size());
    auto *input = g_memory_input_stream_new_from_bytes(buffer);
    g_bytes_unref(buffer);
    // The asynchronous splice owns both streams. Closing the write end lets the
    // confined launcher see EOF without pausing the Files UI or waiting for XDZip.
    g_output_stream_splice_async(
        g_subprocess_get_stdin_pipe(process), input,
        GOutputStreamSpliceFlags(G_OUTPUT_STREAM_SPLICE_CLOSE_SOURCE |
                                 G_OUTPUT_STREAM_SPLICE_CLOSE_TARGET),
        G_PRIORITY_DEFAULT, nullptr, nullptr, nullptr);
    g_object_unref(input);
    g_object_unref(process);
    return true;
}
#endif
void activate(NautilusMenuItem *, gpointer data) {
    const auto *call = static_cast<Invocation *>(data);
    try {
#ifdef XDZIP_NAUTILUS_SNAP
        if (!launchSnap(*call))
            g_warning("xdzip: could not hand off the selected operation to the Snap");
#else
        if (!xdzip_shell::protocolMarker("/usr/share/xdzip/shell-protocol")) {
            g_warning("xdzip: installed application does not support shell actions");
            return;
        }
        if (!xdzip_shell::launch("/usr/bin/xdzip", call->action, call->paths,
                                 call->archiveCandidates))
            g_warning("xdzip: could not hand off the selected operation");
#endif
    } catch (...) {
        g_warning("xdzip: invalid operation selection");
    }
}
GList *items(NautilusMenuProvider *, GList *files) {
    try {
#ifdef XDZIP_NAUTILUS_SNAP
        if (!snapAvailable())
            return nullptr;
#endif
        const auto [enabled, language] = preferences();
        if (!enabled)
            return nullptr;
        std::vector<xdzip_shell::fs::path> paths;
        bool archiveCandidates = true;
        for (auto *it = files; it; it = it->next) {
            if (paths.size() == 128)
                return nullptr;
            auto *location = nautilus_file_info_get_location(NAUTILUS_FILE_INFO(it->data));
            auto *path = g_file_get_path(location);
            g_object_unref(location);
            if (!path)
                return nullptr;
            paths.emplace_back(path);
            g_free(path);
            auto *mime = nautilus_file_info_get_mime_type(NAUTILUS_FILE_INFO(it->data));
            archiveCandidates &= xdzip_shell::archiveCandidate(paths.back()) ||
                                 (mime && xdzip_shell::archiveMime(mime));
            g_free(mime);
        }
        const auto commands = xdzip_shell::menu(paths, language, archiveCandidates);
        if (commands.empty())
            return nullptr;
        auto *root = nautilus_menu_item_new(
            "xdzip::actions", xdzip_shell::text(language, "shell.title").c_str(), nullptr, "xdzip");
        auto *submenu = nautilus_menu_new();
        for (const auto &command : commands) {
            auto *item = nautilus_menu_item_new(("xdzip::" + command.action).c_str(),
                                                command.label.c_str(), nullptr, nullptr);
            auto *call = new Invocation{paths, command.action, archiveCandidates};
            g_signal_connect_data(
                item, "activate", G_CALLBACK(activate), call,
                [](gpointer data, GClosure *) { delete static_cast<Invocation *>(data); },
                GConnectFlags(0));
            nautilus_menu_append_item(submenu, item);
            g_object_unref(item);
        }
        nautilus_menu_item_set_submenu(root, submenu);
        g_object_unref(submenu);
        return g_list_append(nullptr, root);
    } catch (...) {
        return nullptr;
    }
}
void interfaceInit(gpointer value, gpointer) {
    static_cast<NautilusMenuProviderInterface *>(value)->get_file_items = items;
}
} // namespace
extern "C" G_MODULE_EXPORT void nautilus_module_initialize(GTypeModule *module) {
    const GTypeInfo info{sizeof(ExtensionClass), nullptr, nullptr, nullptr, nullptr, nullptr,
                         sizeof(Extension),      0,       nullptr, nullptr};
    extensionType = g_type_module_register_type(module, G_TYPE_OBJECT, "XdzipShellExtension", &info,
                                                GTypeFlags(0));
    const GInterfaceInfo interface{interfaceInit, nullptr, nullptr};
    g_type_module_add_interface(module, extensionType, NAUTILUS_TYPE_MENU_PROVIDER, &interface);
}
extern "C" G_MODULE_EXPORT void nautilus_module_shutdown() {}
extern "C" G_MODULE_EXPORT void nautilus_module_list_types(const GType **types, int *count) {
    *types = &extensionType;
    *count = 1;
}
