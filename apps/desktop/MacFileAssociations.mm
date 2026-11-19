#include "ArchiveFormatsGenerated.h"
#include "FileAssociations.h"
#include "UiText.h"

#import <AppKit/AppKit.h>
#include <QPointer>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

namespace {
NSString *const bundleIdentifier = @"org.xdzip.desktop";

NSDictionary<NSString *, NSArray<NSString *> *> *documentTypes() {
    static NSDictionary<NSString *, NSArray<NSString *> *> *const types = [] {
        NSMutableDictionary<NSString *, NSArray<NSString *> *> *result =
            [NSMutableDictionary dictionary];
        NSArray *documents =
            [NSBundle.mainBundle objectForInfoDictionaryKey:@"CFBundleDocumentTypes"];
        for (NSDictionary *document in documents) {
            if (![document[@"LSHandlerRank"] isEqualToString:@"Alternate"] ||
                ![document[@"CFBundleTypeRole"] isEqualToString:@"Viewer"])
                continue;
            for (NSString *suffix in document[@"CFBundleTypeExtensions"])
                result[suffix] = document[@"LSItemContentTypes"] ?: @[];
        }
        return [result copy];
    }();
    return types;
}

NSString *filenameExtension(const QString &suffix) {
    // Launch Services matches the filename's final extension. For example,
    // .tar.gz shares the GZIP type, and .tgz has its own compressed TAR type.
    return suffix.section(QLatin1Char('.'), -1).toNSString();
}

UTType *contentType(const QString &suffix) {
    NSString *extension = filenameExtension(suffix);
    UTType *preferred = [UTType typeWithFilenameExtension:extension];
    NSArray<NSString *> *declared = documentTypes()[extension];
    // Extension-only declarations (including UDF's older-install compatibility)
    // follow the system's actual type, including a dynamically assigned type.
    if (!preferred && [extension isEqualToString:@"udf"])
        return [UTType typeWithIdentifier:@"org.xdzip.udf-image"];
    if (declared.count == 0 || (preferred && [declared containsObject:preferred.identifier]))
        return preferred;
    return [UTType typeWithIdentifier:declared.lastObject];
}

NSString *applicationIdentifier(NSURL *url) {
    return url ? [NSBundle bundleWithURL:url].bundleIdentifier : nil;
}

NSString *defaultIdentifier(UTType *type) {
    return applicationIdentifier(
        [NSWorkspace.sharedWorkspace URLForApplicationToOpenContentType:type]);
}

NSArray<NSURL *> *alternativeApplications(UTType *type) {
    NSMutableArray<NSURL *> *result = [NSMutableArray array];
    NSMutableSet<NSString *> *seen = [NSMutableSet setWithObject:bundleIdentifier];
    for (NSURL *url in [NSWorkspace.sharedWorkspace URLsForApplicationsToOpenContentType:type]) {
        NSString *identifier = applicationIdentifier(url);
        if (identifier.length && ![seen containsObject:identifier]) {
            [result addObject:url];
            [seen addObject:identifier];
        }
    }
    return result;
}

enum class ChangeResult { Applied, Cancelled, Failed };

void requestDefault(NSURL *url, UTType *type, void (^completion)(ChangeResult)) {
    NSString *target = applicationIdentifier(url);
    if (!target.length) {
        completion(ChangeResult::Failed);
        return;
    }
    [NSWorkspace.sharedWorkspace
        setDefaultApplicationAtURL:url
                 toOpenContentType:type
                 completionHandler:^(NSError *error) {
                   // AppKit may complete off the GUI thread. Read Launch Services and
                   // touch the Qt service and its settings only on the main thread.
                   dispatch_async(dispatch_get_main_queue(), ^{
                     if (!error && [defaultIdentifier(type) isEqualToString:target]) {
                         completion(ChangeResult::Applied);
                     } else if (([error.domain isEqualToString:NSCocoaErrorDomain] &&
                                 error.code == NSUserCancelledError) ||
                                ([error.domain isEqualToString:NSOSStatusErrorDomain] &&
                                 error.code == userCanceledErr)) {
                         completion(ChangeResult::Cancelled);
                     } else {
                         completion(ChangeResult::Failed);
                     }
                   });
                 }];
}

QString historyPath() {
    // Leave the transactional UI preferences file independent of this history.
    return uiSettingsPath() + QStringLiteral(".associations.ini");
}

bool savePreviousApplication(UTType *type, NSString *identifier) {
    QSettings history(historyPath(), QSettings::IniFormat);
    const auto key = QString::fromNSString(type.identifier);
    if (identifier.length)
        history.setValue(key, QString::fromNSString(identifier));
    else
        history.remove(key);
    history.sync();
    return history.status() == QSettings::NoError;
}

void finish(QPointer<FileAssociations> service, const QString &suffix, const char *error = "") {
    if (service)
        service->nativeFinished(QString::fromLatin1(error), suffix);
}

void finishReplacement(QPointer<FileAssociations> service, const QString &suffix, UTType *type,
                       ChangeResult result) {
    if (!service)
        return;
    if (result == ChangeResult::Applied && !savePreviousApplication(type, nil)) {
        finish(service, suffix, "settings.associationHistoryFailed");
        return;
    }
    finish(service, suffix, result == ChangeResult::Failed ? "settings.associationFailed" : "");
}

void chooseAlternative(QPointer<FileAssociations> service, QString suffix, UTType *type,
                       NSWindow *owner) {
    if (!service)
        return;
    if (![defaultIdentifier(type) isEqualToString:bundleIdentifier]) {
        finish(service, suffix);
        return;
    }
    NSArray<NSURL *> *applications = alternativeApplications(type);
    if (applications.count == 0) {
        finish(service, suffix, "settings.associationNoAlternative");
        return;
    }

    NSAlert *alert = [[NSAlert alloc] init];
    alert.messageText = systemDialogText("settings.chooseDefaultApp").toNSString();
    alert.informativeText = systemDialogText("settings.chooseDefaultAppDescription").toNSString();
    [alert addButtonWithTitle:systemDialogText("settings.useDefaultApp").toNSString()];
    [alert addButtonWithTitle:systemDialogText("common.cancel").toNSString()];

    NSPopUpButton *choice = [[NSPopUpButton alloc] initWithFrame:NSMakeRect(0, 0, 280, 24)
                                                       pullsDown:NO];
    choice.accessibilityLabel = alert.informativeText;
    for (NSURL *url in applications) {
        NSBundle *bundle = [NSBundle bundleWithURL:url];
        NSString *name = [bundle objectForInfoDictionaryKey:@"CFBundleDisplayName"] ?:
                         [bundle objectForInfoDictionaryKey:@"CFBundleName"] ?:
                         url.URLByDeletingPathExtension.lastPathComponent;
        // NSMenu keeps distinct entries even if two apps have the same display name.
        NSMenuItem *item = [choice.menu addItemWithTitle:name action:nil keyEquivalent:@""];
        item.representedObject = url;
        item.toolTip = bundle.bundleIdentifier;
    }
    [choice selectItemAtIndex:0];
    alert.accessoryView = choice;

    void (^respond)(NSModalResponse) = ^(NSModalResponse response) {
      if (!service)
          return;
      if (response != NSAlertFirstButtonReturn ||
          ![defaultIdentifier(type) isEqualToString:bundleIdentifier]) {
          finish(service, suffix);
          return;
      }
      NSURL *selected = choice.selectedItem.representedObject;
      if (![alternativeApplications(type) containsObject:selected]) {
          finish(service, suffix, "settings.associationFailed");
          return;
      }
      requestDefault(selected, type, ^(ChangeResult result) {
        finishReplacement(service, suffix, type, result);
      });
    };
    if (owner.visible && !owner.attachedSheet)
        [alert beginSheetModalForWindow:owner completionHandler:respond];
    else
        respond([alert runModal]);
}
} // namespace

QStringList macAssociationExtensions() {
    QStringList result;
    for (const auto suffix : xdzip_formats::ShellSuffixes) {
        const auto value = QString::fromLatin1(suffix.data(), suffix.size());
        if (documentTypes()[filenameExtension(value)] && contentType(value))
            result.append(value);
    }
    return result;
}

bool macAssociationDefault(const QString &extension) {
    UTType *type = contentType(extension);
    return type && [defaultIdentifier(type) isEqualToString:bundleIdentifier];
}

void macAssociationChange(FileAssociations *object, const QString &extension, bool enable) {
    const QPointer<FileAssociations> service(object);
    // The asynchronous AppKit blocks must own the value passed by the QML call.
    const QString suffix = extension;
    UTType *type = contentType(suffix);
    if (!type) {
        finish(service, suffix, "settings.associationFailed");
        return;
    }
    NSString *current = defaultIdentifier(type);
    if (enable == [current isEqualToString:bundleIdentifier]) {
        finish(service, suffix);
        return;
    }
    if (enable) {
        requestDefault(NSBundle.mainBundle.bundleURL, type, ^(ChangeResult result) {
          if (!service)
              return;
          if (result == ChangeResult::Applied && !savePreviousApplication(type, current))
              finish(service, suffix, "settings.associationHistoryFailed");
          else
              finish(service, suffix,
                     result == ChangeResult::Failed ? "settings.associationFailed" : "");
        });
        return;
    }

    NSWindow *owner = NSApp.keyWindow;
    QSettings history(historyPath(), QSettings::IniFormat);
    NSString *previous =
        history.value(QString::fromNSString(type.identifier)).toString().toNSString();
    if (previous.length) {
        for (NSURL *url in alternativeApplications(type)) {
            if (![applicationIdentifier(url) isEqualToString:previous])
                continue;
            requestDefault(url, type, ^(ChangeResult result) {
              if (result == ChangeResult::Failed)
                  chooseAlternative(service, suffix, type, owner);
              else
                  finishReplacement(service, suffix, type, result);
            });
            return;
        }
    }
    chooseAlternative(service, suffix, type, owner);
}
