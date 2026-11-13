#include "../MacFinderIntegration.h"
#include "MacFinderRequest.h"
#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QTemporaryDir>
#include <iostream>

int main(int argc, char **argv) {
    QCoreApplication application(argc, argv);
    if (argc != 3)
        return 2;
    @autoreleasepool {
        int checks = 0, failures = 0;
        const auto check = [&](bool pass, const char *name) {
            ++checks;
            if (!pass) {
                ++failures;
                std::cerr << "FAIL " << name << '\n';
            }
        };
        QDir().mkpath(QString::fromUtf8(argv[1]));
        QTemporaryDir temporary(QString::fromUtf8(argv[1]) + "/finder-XXXXXX");
        if (!temporary.isValid())
            return 2;
        NSString *root = temporary.path().toNSString();
        NSString *app = [root stringByAppendingPathComponent:@"fixture.app"];
        NSString *plist = [app stringByAppendingPathComponent:@"Contents/Info.plist"];
        [[NSFileManager defaultManager]
                  createDirectoryAtPath:plist.stringByDeletingLastPathComponent
            withIntermediateDirectories:YES
                             attributes:nil
                                  error:nil];
        NSMutableDictionary *info = [@{
            @"CFBundleIdentifier" : @"org.xdzip.desktop",
            @"CFBundleExecutable" : @"xdzip",
            @"XdzipShellProtocol" : @1,
            @"XdzipFinderSyncProtocol" : @1
        } mutableCopy];
        [info writeToFile:plist atomically:YES];
        check(xdzip_shell::finderCompatible([NSString stringWithUTF8String:argv[2]]),
              "built capability");
        check([xdzip_shell::finderContainingApp(
                  [app stringByAppendingPathComponent:@"Contents/PlugIns/xdzip-finder-sync.appex"])
                  isEqual:app],
              "extension containing app");
        check([xdzip_shell::finderContainingApp(
                  [app stringByAppendingPathComponent:@"Contents/Helpers/xdzip-shell-relay.app"])
                  isEqual:app],
              "relay containing app");
        check(!xdzip_shell::finderContainingApp(@"/tmp/foreign.appex"), "foreign layout");
        check(xdzip_shell::finderCompatible(app), "fresh capability");
        [info removeObjectForKey:@"XdzipFinderSyncProtocol"];
        [info writeToFile:plist atomically:YES];
        check(!xdzip_shell::finderCompatible(app), "rollback rejects cached capability");
        info[@"XdzipFinderSyncProtocol"] = @2;
        [info writeToFile:plist atomically:YES];
        check(!xdzip_shell::finderCompatible(app), "unknown capability");

        auto *file = [NSURL fileURLWithPath:[root stringByAppendingPathComponent:@"a & 中文.txt"]];
        auto *archive = [NSURL fileURLWithPath:[root stringByAppendingPathComponent:@"a.zip"]];
        auto *volume = [NSURL fileURLWithPath:[root stringByAppendingPathComponent:@"b.7z.001"]];
        auto *directory = [NSURL fileURLWithPath:[root stringByAppendingPathComponent:@"folder.zip"]
                                     isDirectory:YES];
        [[NSFileManager defaultManager] createDirectoryAtURL:directory
                                 withIntermediateDirectories:YES
                                                  attributes:nil
                                                       error:nil];
        const auto entries = [&](NSArray<NSURL *> *urls, bool english, NSInteger expected) {
            auto *menu = xdzip_shell::finderMenu(urls, english, nil, @selector(description));
            auto *submenu = menu.itemArray.firstObject.submenu;
            check(menu.numberOfItems == 1 && submenu.numberOfItems == expected,
                  "dynamic menu count");
            for (NSMenuItem *item in submenu.itemArray) {
                auto *url = xdzip_shell::finderRequestURL(item.representedObject);
                auto *request = xdzip_shell::finderRequestFromURL(url);
                check(request && [request isEqual:item.representedObject], "menu relay roundtrip");
            }
            return submenu;
        };
        auto *single = entries(@[ file ], false, 3);
        check([single.itemArray.firstObject.title containsString:@"a & 中文.txt.zip"],
              "full filename");
        entries(@[ archive ], true, 7);
        entries(@[ archive, volume ], false, 8);
        entries(@[ archive, file ], true, 4);
        entries(@[ directory ], false, 3);
        check(!xdzip_shell::finderMenu(@[], false, nil, nullptr), "empty selection");
        xdzip_shell::FinderMenuSession session;
        auto *snapshot = session.prepare(@[ file ], false, nil, @selector(description));
        auto *originalItem = snapshot.itemArray.firstObject.submenu.itemArray.firstObject;
        auto *proxy = [[NSMenuItem alloc] initWithTitle:originalItem.title
                                                 action:nullptr
                                          keyEquivalent:@""];
        proxy.tag = originalItem.tag;
        check([session.request(proxy) isEqual:originalItem.representedObject],
              "Finder tag-only proxy");
        session.prepare(@[ archive ], true, nil, @selector(description));
        check(!session.request(proxy), "stale tag cannot select another file");
        check(!xdzip_shell::finderRequest(
                  @"zip", @[ [NSURL URLWithString:@"https://example.invalid/a.zip"] ]),
              "local selection only");
        NSMutableDictionary *request = [xdzip_shell::finderRequest(@"zip", @[ file ]) mutableCopy];
        NSString *identity = request[@"id"];
        check([xdzip_shell::finderRequestFromURL(xdzip_shell::finderRequestURL(request))[@"id"]
                  isEqual:identity],
              "identity preserved");
        request[@"id"] = @"00000000-0000-0000-0000-000000000000";
        check(!xdzip_shell::finderRequestURL(request), "null identity");
        request[@"id"] = identity;
        request[@"action"] = @"unknown";
        check(!xdzip_shell::finderRequestURL(request), "unknown action");
        request[@"action"] = @"zip";
        request[@"schema_version"] = @YES;
        check(!xdzip_shell::finderRequestURL(request), "boolean schema rejected");
        request[@"schema_version"] = @1;
        request[@"paths"] = @[ @"relative" ];
        check(!xdzip_shell::finderRequestURL(request), "relative path");
        request[@"paths"] = @[ [NSString stringWithFormat:@"/a%Cb", (unichar)0] ];
        check(!xdzip_shell::finderRequestURL(request), "embedded null");
        auto *many = [NSMutableArray new];
        for (int i = 0; i < 128; ++i)
            [many addObject:file];
        check(xdzip_shell::finderRequestURL(xdzip_shell::finderRequest(@"zip", many)), "128 paths");
        [many addObject:file];
        check(!xdzip_shell::finderRequest(@"zip", many), "129 paths");
        auto *large = [NSMutableArray new];
        NSString *longPath = [@"/" stringByPaddingToLength:4096 withString:@"x" startingAtIndex:0];
        for (int i = 0; i < 128; ++i)
            [large addObject:longPath];
        request[@"paths"] = large;
        check(!xdzip_shell::finderRequestURL(request), "256 KiB JSON bound");
        for (NSString *url in @[
                 @"xdzip-shell://v2/e30", @"https://v1/e30", @"xdzip-shell://v1/@@",
                 @"xdzip-shell://v1/e30?query=1", @"xdzip-shell://user@v1/e30",
                 @"xdzip-shell://v1/e30#fragment"
             ]) {
            check(!xdzip_shell::finderRequestFromURL([NSURL URLWithString:url]),
                  "malformed envelope");
        }
        NSString *config = [root stringByAppendingPathComponent:@"finder-sync.json"];
        const auto write = [&](bool enabled, const QString &language) {
            return writeMacFinderConfiguration(QString::fromNSString(app),
                                               QString::fromNSString(config), enabled, language);
        };
        check(write(true, "en-US"), "write preferences");
        auto prefs = xdzip_shell::finderPreferences(app, config);
        check(prefs.enabled && prefs.english, "enabled language");
        check(!xdzip_shell::finderPreferences(@"/foreign.app", config).enabled,
              "foreign installation");
        check(write(false, "zh-CN") && !xdzip_shell::finderPreferences(app, config).enabled,
              "disable immediately");
        check(xdzip_shell::finderPreferences(app, config).valid,
              "explicit integration remains configurable after disable");
        check(write(true, "zh-CN"), "re-enable");
        check(writeMacFinderConfiguration("/old.app", QString::fromNSString(config), false,
                                          "en-US") &&
                  xdzip_shell::finderPreferences(app, config).enabled,
              "old uninstall keeps active copy");
        QFile::setPermissions(QString::fromNSString(config),
                              QFile::ReadOwner | QFile::WriteOwner | QFile::ReadGroup);
        check(!xdzip_shell::finderPreferences(app, config).enabled, "private preferences required");
        QFile::remove(QString::fromNSString(config));
        QFile malformed(QString::fromNSString(config));
        malformed.open(QIODevice::WriteOnly);
        malformed.write("{}");
        malformed.close();
        check(!write(true, "zh-CN"), "foreign configuration preserved");
        QFile::remove(QString::fromNSString(config));
        [[NSFileManager defaultManager] createSymbolicLinkAtPath:config
                                             withDestinationPath:plist
                                                           error:nil];
        check(!write(true, "zh-CN") && !xdzip_shell::finderPreferences(app, config).enabled,
              "symlink configuration refused");
        // QTemporaryDir owns only this invocation's newly created fixture subtree.
        std::cout << "Mac Finder: " << checks << " checks, " << failures << " failures\n";
        return failures ? 1 : 0;
    }
}
