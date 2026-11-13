#include "MacFinderRequest.h"

@interface XdzipFinderRelay : NSObject <NSApplicationDelegate>
@property NSUInteger generation;
@end
@implementation XdzipFinderRelay
- (void)application:(NSApplication *)application openURLs:(NSArray<NSURL *> *)urls {
    NSString *app = xdzip_shell::finderContainingApp(NSBundle.mainBundle.bundlePath);
    const auto preferences = xdzip_shell::finderPreferences(app);
    bool rejected = urls.count != 1 || !preferences.enabled || !xdzip_shell::finderCompatible(app);
    if (!rejected) {
        try {
            for (NSURL *url in urls) {
                auto *request = xdzip_shell::finderRequestFromURL(url, preferences.handoffToken);
                if (!request) {
                    rejected = true;
                    continue;
                }
                std::vector<xdzip_shell::fs::path> paths;
                for (NSString *path in request[@"paths"])
                    paths.emplace_back(path.fileSystemRepresentation);
                auto *executable = xdzip_shell::desktopExecutable(app);
                rejected |= !xdzip_shell::launch(
                    executable.fileSystemRepresentation, [request[@"action"] UTF8String], paths,
                    [request[@"archive_candidates"] boolValue], [request[@"id"] UTF8String]);
            }
        } catch (...) {
            rejected = true;
        }
    }
    if (rejected) {
        auto *alert = [NSAlert new];
        alert.messageText = @"xdzip";
        alert.informativeText =
            [NSString stringWithUTF8String:xdzip_shell::text(preferences.english,
                                                             "shell.finderRequestRejected")
                                               .c_str()];
        [alert runModal];
    }
    const NSUInteger generation = ++self.generation;
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 2 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
      if (self.generation == generation)
          [application terminate:nil];
    });
}
- (void)applicationDidFinishLaunching:(NSNotification *)notification {
    (void)notification;
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 5 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
      if (self.generation == 0)
          [NSApplication.sharedApplication terminate:nil];
    });
}
- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)application {
    (void)application;
    return NO;
}
@end
int main() {
    @autoreleasepool {
        auto *delegate = [XdzipFinderRelay new];
        NSApplication.sharedApplication.delegate = delegate;
        [NSApplication.sharedApplication run];
    }
    return 0;
}
