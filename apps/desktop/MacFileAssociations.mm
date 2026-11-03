#include "FileAssociations.h"
#include "UiText.h"
#include <QMetaObject>
#include <QPointer>
#import <AppKit/AppKit.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

namespace {
UTType *contentType(const QString &extension) {
    if (extension == QStringLiteral(".cab")) {
        UTType *preferred = [UTType typeWithFilenameExtension:@"cab"];
        if ([preferred.identifier isEqualToString:@"com.microsoft.cab"] ||
            [preferred.identifier isEqualToString:@"com.microsoft.cab-archive"])
            return preferred;
    }
    static const QHash<QString, QString> types{
        {".zip", "public.zip-archive"}, {".7z", "org.7-zip.7-zip-archive"},
        {".rar", "com.rarlab.rar-archive"}, {".tar", "public.tar-archive"},
        {".tar.gz", "org.gnu.gnu-zip-tar-archive"}, {".tgz", "org.gnu.gnu-zip-tar-archive"},
        {".cab", "com.microsoft.cab-archive"}, {".iso", "public.iso-image"},
        {".udf", "org.xdzip.udf-image"}};
    const QString identifier = types.value(extension);
    return identifier.isEmpty() ? nil : [UTType typeWithIdentifier:identifier.toNSString()];
}
void finish(QPointer<FileAssociations> service, bool success) {
    if (service)
        QMetaObject::invokeMethod(service, [service, success] { if (service) service->nativeFinished(success); }, Qt::QueuedConnection);
}
void chooseDefault(QPointer<FileAssociations> service, NSURL *url, UTType *type) {
    if (!url || !type) { finish(service, false); return; }
    [[NSWorkspace sharedWorkspace] setDefaultApplicationAtURL:url toOpenContentType:type
        completionHandler:^(NSError *error) { finish(service, error == nil); }];
}
}
bool macAssociationsAvailable() {
    NSArray *types = [[NSBundle mainBundle] objectForInfoDictionaryKey:@"CFBundleDocumentTypes"];
    for (NSDictionary *type in types)
        if ([type[@"LSHandlerRank"] isEqualToString:@"Alternate"]) return true;
    return false;
}
bool macAssociationDefault(const QString &extension) {
    UTType *type = contentType(extension);
    if (!type) return false;
    NSURL *url = [[NSWorkspace sharedWorkspace] URLForApplicationToOpenContentType:type];
    NSBundle *bundle = url ? [NSBundle bundleWithURL:url] : nil;
    return [bundle.bundleIdentifier isEqualToString:@"org.xdzip.desktop"];
}
void macAssociationChange(FileAssociations *object, const QString &extension, bool enable) {
    const QPointer<FileAssociations> service(object);
    UTType *type = contentType(extension);
    if (!type) { finish(service, false); return; }
    if (enable) {
        chooseDefault(service, [NSBundle mainBundle].bundleURL, type);
        return;
    }
    NSOpenPanel *panel = [NSOpenPanel openPanel];
    panel.allowedContentTypes = @[UTTypeApplication];
    panel.canChooseFiles = YES;
    panel.canChooseDirectories = NO;
    panel.allowsMultipleSelection = NO;
    panel.directoryURL = [NSURL fileURLWithPath:@"/Applications" isDirectory:YES];
    panel.message = systemDialogText("settings.chooseDefaultAppDescription").toNSString();
    [panel beginWithCompletionHandler:^(NSModalResponse response) {
        if (response != NSModalResponseOK) { finish(service, true); return; }
        NSBundle *bundle = [NSBundle bundleWithURL:panel.URL];
        if (!bundle.bundleIdentifier || [bundle.bundleIdentifier isEqualToString:@"org.xdzip.desktop"]) {
            finish(service, false); return;
        }
        chooseDefault(service, panel.URL, type);
    }];
}
