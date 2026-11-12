#include "NativeRequest.h"
#include <gmodule.h>
#include <iostream>
#include <nautilus-extension.h>

struct TestFile {
    GObject parent;
    GFile *location;
};
struct TestFileClass {
    GObjectClass parent;
};
static void fileInterface(gpointer value, gpointer) {
    static_cast<NautilusFileInfoInterface *>(value)->get_location = [](NautilusFileInfo *file) {
        return G_FILE(g_object_ref(reinterpret_cast<TestFile *>(file)->location));
    };
}
G_DEFINE_TYPE_WITH_CODE(TestFile, test_file, G_TYPE_OBJECT,
                        G_IMPLEMENT_INTERFACE(NAUTILUS_TYPE_FILE_INFO, fileInterface))
static void test_file_init(TestFile *file) {
    file->location = nullptr;
}
static void test_file_class_init(TestFileClass *klass) {
    G_OBJECT_CLASS(klass)->finalize = [](GObject *object) {
        g_clear_object(&reinterpret_cast<TestFile *>(object)->location);
        G_OBJECT_CLASS(test_file_parent_class)->finalize(object);
    };
}
struct TestModule {
    GTypeModule parent;
};
struct TestModuleClass {
    GTypeModuleClass parent;
};
G_DEFINE_TYPE(TestModule, test_module, G_TYPE_TYPE_MODULE)
static void test_module_init(TestModule *) {}
static void test_module_class_init(TestModuleClass *klass) {
    G_TYPE_MODULE_CLASS(klass)->load = [](GTypeModule *) -> gboolean { return TRUE; };
    G_TYPE_MODULE_CLASS(klass)->unload = [](GTypeModule *) {};
}
int main(int argc, char **argv) {
    if (argc != 2)
        return 2;
    auto *library = g_module_open(argv[1], G_MODULE_BIND_LOCAL);
    if (!library) {
        std::cerr << g_module_error() << '\n';
        return 1;
    }
    void (*initialize)(GTypeModule *) = nullptr;
    void (*list)(const GType **, int *) = nullptr;
    if (!g_module_symbol(library, "nautilus_module_initialize",
                         reinterpret_cast<gpointer *>(&initialize)) ||
        !g_module_symbol(library, "nautilus_module_list_types",
                         reinterpret_cast<gpointer *>(&list)))
        return 1;
    auto *module = G_TYPE_MODULE(g_object_new(test_module_get_type(), nullptr));
    if (!g_type_module_use(module))
        return 1;
    initialize(module);
    const GType *types = nullptr;
    int count = 0;
    list(&types, &count);
    if (count != 1 || !g_type_is_a(types[0], NAUTILUS_TYPE_MENU_PROVIDER))
        return 1;
    auto *provider = NAUTILUS_MENU_PROVIDER(g_object_new(types[0], nullptr));
    int checks = 0;
    const auto check = [&](const std::vector<std::string> &uris, int expected) {
        GList *files = nullptr;
        for (const auto &uri : uris) {
            auto *file = reinterpret_cast<TestFile *>(g_object_new(test_file_get_type(), nullptr));
            file->location = g_file_new_for_uri(uri.c_str());
            files = g_list_append(files, file);
        }
        auto *items = nautilus_menu_provider_get_file_items(provider, files);
        int found = 0;
        if (items && !items->next) {
            NautilusMenu *submenu = nullptr;
            g_object_get(items->data, "menu", &submenu, nullptr);
            auto *children = nautilus_menu_get_items(submenu);
            found = g_list_length(children);
            for (auto *it = children; it; it = it->next) {
                char *label = nullptr;
                g_object_get(it->data, "label", &label, nullptr);
                if (!label || !g_utf8_validate(label, -1, nullptr))
                    found = -1;
                g_free(label);
            }
            g_list_free_full(children, g_object_unref);
            g_object_unref(submenu);
        }
        g_list_free_full(items, g_object_unref);
        g_list_free_full(files, g_object_unref);
        ++checks;
        return found == expected;
    };
    if (!check({"file:///tmp/xdzip-test.txt"}, 3) || !check({"file:///tmp/xdzip-test.zip"}, 7) ||
        !check({"file:///tmp/xdzip-test.zip", "file:///tmp/xdzip-test.7z"}, 8) ||
        !check({"file:///tmp/xdzip-test.zip", "file:///tmp/xdzip-test.txt"}, 4) ||
        !check({"smb://example.invalid/archive.zip"}, 0) || !check({}, 0) ||
        !check(std::vector<std::string>(129, "file:///tmp/xdzip-test.txt"), 0))
        return 1;
    g_object_unref(provider);
    // GLib owns the registered dynamic types until process teardown.
    g_module_make_resident(library);
    std::cout << "Nautilus native provider: " << checks << " selection and submenu cases passed\n";
    return 0;
}
