#include "ArchiveFormatsGenerated.h"
#include "FileAssociations.h"

#import <AppKit/AppKit.h>
#include <QDir>
#include <QGuiApplication>
#include <QTemporaryDir>
#include <QtTest>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>
#import <objc/runtime.h>

@interface AssociationWorkspace : NSObject
@property(nonatomic, strong) NSMutableDictionary<NSString *, NSURL *> *defaults;
@property(nonatomic, copy) NSArray<NSURL *> *candidates;
@property(nonatomic, strong) NSMutableArray<NSURL *> *requests;
@property(nonatomic, strong) NSMutableArray<NSString *> *requestedTypes;
@property(nonatomic, strong) NSError *nextError;
@property(nonatomic) BOOL ignoreNextChange;
@property(nonatomic) int nextPublicationDelayMs;
@end

@implementation AssociationWorkspace
- (NSURL *)URLForApplicationToOpenContentType:(UTType *)type {
    return self.defaults[type.identifier];
}

- (NSArray<NSURL *> *)URLsForApplicationsToOpenContentType:(UTType *)type {
    return self.candidates;
}

- (void)setDefaultApplicationAtURL:(NSURL *)url
                 toOpenContentType:(UTType *)type
                 completionHandler:(void (^)(NSError *))completion {
    [self.requests addObject:url];
    [self.requestedTypes addObject:type.identifier];
    NSError *error = self.nextError;
    if (!error && !self.ignoreNextChange) {
        const int delay = self.nextPublicationDelayMs;
        if (delay > 0) {
            dispatch_after(dispatch_time(DISPATCH_TIME_NOW, delay * NSEC_PER_MSEC),
                           dispatch_get_main_queue(), ^{
                             self.defaults[type.identifier] = url;
                           });
        } else {
            self.defaults[type.identifier] = url;
        }
    }
    self.nextError = nil;
    self.ignoreNextChange = NO;
    self.nextPublicationDelayMs = 0;
    completion(error);
}
@end

// The workspace is simulated, so an imported type in the unregistered test
// bundle must not depend on whether a real application registered it first.
@interface AssociationFixtureType : NSObject
@property(nonatomic, copy) NSString *identifier;
@end

@implementation AssociationFixtureType
@end

namespace {
NSBundle *testBundle;
AssociationWorkspace *workspace;
NSArray<NSURL *> *listedApps;
int chooserCount;
NSModalResponse chooserResponse;
IMP originalFilenameType;
IMP originalIdentifierType;
bool forceDynamicRar;
bool forceMissingRar;

UTType *fixtureType(NSString *identifier) {
    // The tested code reads only identifier, and all workspace methods that
    // receive this object are replaced below. It must never reach AppKit.
    AssociationFixtureType *type = [[AssociationFixtureType alloc] init];
    type.identifier = identifier;
    return (UTType *)(id)type;
}

UTType *fixtureFilenameType(id receiver, SEL selector, NSString *extension) {
    if (forceDynamicRar && [extension isEqualToString:@"rar"])
        return fixtureType(@"dyn.xdzip-test-rar");
    return ((UTType * (*)(id, SEL, NSString *)) originalFilenameType)(receiver, selector,
                                                                      extension);
}

UTType *fixtureIdentifierType(id receiver, SEL selector, NSString *identifier) {
    if (forceMissingRar && [identifier isEqualToString:@"com.rarlab.rar-archive"])
        return nil;
    UTType *resolved = ((UTType * (*)(id, SEL, NSString *))
                            originalIdentifierType)(receiver, selector, identifier);
    if (resolved)
        return resolved;
    for (NSDictionary *declaration in
         [testBundle objectForInfoDictionaryKey:@"UTImportedTypeDeclarations"]) {
        if ([declaration[@"UTTypeIdentifier"] isEqualToString:identifier])
            return fixtureType(identifier);
    }
    return nil;
}

NSBundle *fixtureBundle(id, SEL) {
    return testBundle;
}

NSURL *fixtureDefault(id, SEL, UTType *type) {
    return [workspace URLForApplicationToOpenContentType:type];
}

NSArray<NSURL *> *fixtureCandidates(id, SEL, UTType *type) {
    return [workspace URLsForApplicationsToOpenContentType:type];
}

void fixtureChange(id, SEL, NSURL *url, UTType *type, void (^completion)(NSError *)) {
    [workspace setDefaultApplicationAtURL:url toOpenContentType:type completionHandler:completion];
}

NSModalResponse fixtureChooser(NSAlert *alert, SEL) {
    ++chooserCount;
    NSPopUpButton *choice = (NSPopUpButton *)alert.accessoryView;
    listedApps = [choice.itemArray valueForKey:@"representedObject"];
    return chooserResponse;
}

// Only the test process replaces AppKit entry points. No fixture is registered
// and no request is sent to Launch Services to change the user's default.
struct ScopedImplementation {
    Method method;
    IMP previous;
    ScopedImplementation(Method target, IMP replacement)
        : method(target), previous(method_setImplementation(target, replacement)) {}
    ~ScopedImplementation() {
        method_setImplementation(method, previous);
    }
};

NSBundle *makeBundle(const QString &root, NSString *name, NSString *identifier,
                     NSDictionary *documentInfo = nil) {
    const QString path = root + QLatin1Char('/') + QString::fromNSString(name) + ".app";
    if (!QDir().mkpath(path + "/Contents"))
        return nil;
    NSMutableDictionary *info =
        documentInfo ? [documentInfo mutableCopy] : [NSMutableDictionary dictionary];
    info[@"CFBundleIdentifier"] = identifier;
    info[@"CFBundleName"] = name;
    info[@"CFBundlePackageType"] = @"APPL";
    if (![info writeToURL:[NSURL fileURLWithPath:(path + "/Contents/Info.plist").toNSString()]
                    error:nil])
        return nil;
    return [NSBundle bundleWithPath:path.toNSString()];
}

NSString *lz4TypeIdentifier() {
    return [UTType typeWithFilenameExtension:@"lz4"].identifier;
}

bool associationIdle(FileAssociations &service) {
    // Offscreen Qt does not run Cocoa's main loop. AppKit completions still
    // reach the main dispatch queue, as they do in the native application.
    CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.001, true);
    return !service.busy();
}
} // namespace

class MacAssociationTest : public QObject {
    Q_OBJECT

  public:
    NSURL *original;
    NSURL *other;
    NSURL *duplicate;
    NSURL *xdzipCopy;

  private slots:
    void init() {
        workspace.defaults = [NSMutableDictionary dictionary];
        workspace.requests = [NSMutableArray array];
        workspace.requestedTypes = [NSMutableArray array];
        workspace.candidates = @[ testBundle.bundleURL, xdzipCopy, original, other, duplicate ];
        workspace.nextError = nil;
        workspace.ignoreNextChange = NO;
        workspace.nextPublicationDelayMs = 0;
        chooserCount = 0;
        listedApps = @[];
        chooserResponse = NSAlertFirstButtonReturn;
        forceDynamicRar = false;
        forceMissingRar = false;
        QVERIFY(QFile::remove(qEnvironmentVariable("XDZIP_SETTINGS_PATH") + ".associations.ini") ||
                !QFile::exists(qEnvironmentVariable("XDZIP_SETTINGS_PATH") + ".associations.ini"));
    }

    void firstDefaultAndNoReplacement() {
        FileAssociations service;
        QVERIFY(service.available());
        for (const auto suffix : xdzip_formats::ShellSuffixes) {
            const auto extension = QString::fromLatin1(suffix.data(), suffix.size());
            QVERIFY2(service.supportsExtension(extension), qPrintable(extension));
        }
        service.refresh();
        service.requestChange(".exe", true);
        QCOMPARE(workspace.requests.count, 0u);

        workspace.candidates = @[ testBundle.bundleURL, xdzipCopy ];
        QVERIFY(!service.isDefault(".lz4"));
        workspace.nextPublicationDelayMs = 250;
        service.requestChange(".lz4", true);
        QVERIFY(service.busy());
        QTRY_VERIFY(associationIdle(service));
        QCOMPARE(service.error(), QString());
        QCOMPARE(workspace.requests.count, 1u);
        QVERIFY(service.isDefault(".lz4"));
        QVERIFY(service.isDefault(".tar.lz4"));

        service.requestChange(".tar.lz4", false);
        QCOMPARE(service.error(), QStringLiteral("settings.associationNoAlternative"));
        QCOMPARE(service.errorExtension(), QStringLiteral(".tar.lz4"));
        QVERIFY(service.isDefault(".lz4"));
        QCOMPARE(workspace.requests.count, 1u);
        QCOMPARE(chooserCount, 0);
    }

    void restoresPriorAppAcrossInstancesAndAliases() {
        workspace.defaults[lz4TypeIdentifier()] = original;
        {
            FileAssociations service;
            service.refresh();
            service.requestChange(".tar.lz4", true);
            QTRY_VERIFY(associationIdle(service));
            QVERIFY(service.isDefault(".lz4"));
        }
        FileAssociations service;
        service.refresh();
        service.requestChange(".lz4", false);
        QTRY_VERIFY(associationIdle(service));
        QCOMPARE(service.error(), QString());
        QVERIFY([workspace.requests.lastObject isEqual:original]);
        QVERIFY(!service.isDefault(".tar.lz4"));
        QCOMPARE(chooserCount, 0);
    }

    void missingImportedTypeDoesNotChangeDefault() {
        forceDynamicRar = true;
        forceMissingRar = true;
        FileAssociations service;
        QVERIFY(!service.supportsExtension(".rar"));
        service.refresh();
        service.requestChange(".rar", true);
        QCOMPARE(workspace.requests.count, 0u);
    }

    void importedTypeIsUsedWhenPreferredTypeIsDynamic() {
        forceDynamicRar = true;
        FileAssociations service;
        QVERIFY(service.supportsExtension(".rar"));
        service.refresh();
        service.requestChange(".rar", true);
        QTRY_VERIFY(associationIdle(service));
        QCOMPARE(service.error(), QString());
        QCOMPARE(workspace.requestedTypes.count, 1u);
        QVERIFY([workspace.requestedTypes.firstObject isEqualToString:@"com.rarlab.rar-archive"]);
        QVERIFY(service.isDefault(".rar"));
    }

    void missingOrRejectedPriorAppOffersCompatibleChoices() {
        workspace.defaults[lz4TypeIdentifier()] = original;
        FileAssociations service;
        service.refresh();
        service.requestChange(".lz4", true);
        QTRY_VERIFY(associationIdle(service));
        QVERIFY(service.isDefault(".lz4"));

        workspace.candidates = @[ testBundle.bundleURL, xdzipCopy, other, duplicate ];
        chooserResponse = NSAlertSecondButtonReturn;
        service.requestChange(".lz4", false);
        QCOMPARE(chooserCount, 1);
        QVERIFY([listedApps isEqualToArray:@[ other ]]);
        QCOMPARE(workspace.requests.count, 1u);
        QVERIFY(service.isDefault(".lz4"));

        workspace.candidates = @[ other, duplicate, original, xdzipCopy ];
        chooserResponse = NSAlertFirstButtonReturn;
        workspace.nextError = [NSError errorWithDomain:NSOSStatusErrorDomain
                                                  code:-10814
                                              userInfo:nil];
        service.requestChange(".lz4", false);
        QTRY_VERIFY(associationIdle(service));
        QCOMPARE(chooserCount, 2);
        QVERIFY(([listedApps isEqualToArray:@[ other, original ]]));
        QVERIFY([workspace.requests.lastObject isEqual:other]);
        QVERIFY(!service.isDefault(".lz4"));
    }

    void systemCancellationAndUnappliedChange() {
        workspace.defaults[lz4TypeIdentifier()] = original;
        FileAssociations service;
        service.refresh();
        workspace.ignoreNextChange = YES;
        service.requestChange(".lz4", true);
        QTRY_VERIFY(associationIdle(service));
        QCOMPARE(service.error(), QStringLiteral("settings.associationFailed"));
        QVERIFY(!service.isDefault(".lz4"));

        service.requestChange(".lz4", true);
        QTRY_VERIFY(associationIdle(service));
        workspace.nextError = [NSError errorWithDomain:NSCocoaErrorDomain
                                                  code:NSUserCancelledError
                                              userInfo:nil];
        service.requestChange(".lz4", false);
        QTRY_VERIFY(associationIdle(service));
        QCOMPARE(service.error(), QString());
        QVERIFY(service.isDefault(".lz4"));
        QCOMPARE(chooserCount, 0);
        service.requestChange(".tar.lz4", false);
        QTRY_VERIFY(associationIdle(service));
        QVERIFY([workspace.requests.lastObject isEqual:original]);
        QCOMPARE(chooserCount, 0);
    }
};

int main(int argc, char **argv) {
    @autoreleasepool {
        QGuiApplication app(argc, argv);
        if (argc != 2)
            return 2;
        QTemporaryDir fixture;
        if (!fixture.isValid())
            return 2;
        qputenv("XDZIP_SETTINGS_PATH", (fixture.path() + "/settings.ini").toUtf8());
        NSMutableDictionary *info = [NSMutableDictionary
            dictionaryWithContentsOfFile:QString::fromUtf8(argv[1]).toNSString()];
        if (!info)
            return 2;
        NSMutableArray *documents = [NSMutableArray array];
        for (NSDictionary *item in info[@"CFBundleDocumentTypes"]) {
            NSMutableDictionary *copy = [item mutableCopy];
            copy[@"LSHandlerRank"] = @"Alternate";
            [documents addObject:copy];
        }
        info[@"CFBundleDocumentTypes"] = documents;
        testBundle = makeBundle(fixture.path(), @"xdzip", @"org.xdzip.desktop", info);
        workspace = [[AssociationWorkspace alloc] init];
        MacAssociationTest tests;
        tests.original =
            makeBundle(fixture.path(), @"Original", @"org.xdzip.test.original").bundleURL;
        tests.other = makeBundle(fixture.path(), @"Other", @"org.xdzip.test.other").bundleURL;
        tests.duplicate =
            makeBundle(fixture.path(), @"Other copy", @"org.xdzip.test.other").bundleURL;
        tests.xdzipCopy = makeBundle(fixture.path(), @"xdzip copy", @"org.xdzip.desktop").bundleURL;
        if (!testBundle || !tests.original || !tests.other || !tests.duplicate || !tests.xdzipCopy)
            return 2;

        const ScopedImplementation mainBundle(
            class_getClassMethod(NSBundle.class, @selector(mainBundle)), (IMP)fixtureBundle);
        Method filenameType =
            class_getClassMethod(UTType.class, @selector(typeWithFilenameExtension:));
        Method identifierType = class_getClassMethod(UTType.class, @selector(typeWithIdentifier:));
        if (!filenameType || !identifierType)
            return 2;
        originalFilenameType = method_getImplementation(filenameType);
        originalIdentifierType = method_getImplementation(identifierType);
        const ScopedImplementation filenameTypeLookup(filenameType, (IMP)fixtureFilenameType);
        const ScopedImplementation identifierTypeLookup(identifierType, (IMP)fixtureIdentifierType);
        const Class workspaceClass = object_getClass(NSWorkspace.sharedWorkspace);
        const ScopedImplementation defaultApplication(
            class_getInstanceMethod(workspaceClass, @selector(URLForApplicationToOpenContentType:)),
            (IMP)fixtureDefault);
        const ScopedImplementation compatibleApplications(
            class_getInstanceMethod(workspaceClass,
                                    @selector(URLsForApplicationsToOpenContentType:)),
            (IMP)fixtureCandidates);
        const ScopedImplementation changeDefault(
            class_getInstanceMethod(workspaceClass,
                                    @selector(setDefaultApplicationAtURL:
                                                       toOpenContentType:completionHandler:)),
            (IMP)fixtureChange);
        const ScopedImplementation runModal(
            class_getInstanceMethod(NSAlert.class, @selector(runModal)), (IMP)fixtureChooser);
        return QTest::qExec(&tests, 1, argv);
    }
}

#include "macos_association_test.moc"
