#include "MacFinderRequest.h"
#import <FinderSync/FinderSync.h>

@interface XdzipFinderSync : FIFinderSync {
    xdzip_shell::FinderMenuSession menuSession_;
}
@end
@implementation XdzipFinderSync
- (instancetype)init {
    self = [super init];
    if (self) {
        auto *home = [NSURL fileURLWithPath:xdzip_shell::finderHome() isDirectory:YES];
        [FIFinderSyncController defaultController].directoryURLs =
            [NSSet setWithObjects:home, [NSURL fileURLWithPath:@"/Volumes" isDirectory:YES], nil];
    }
    return self;
}
- (NSMenu *)menuForMenuKind:(FIMenuKind)kind {
    NSString *app = xdzip_shell::finderContainingApp(NSBundle.mainBundle.bundlePath);
    const auto preferences = xdzip_shell::finderPreferences(app);
    if (kind != FIMenuKindContextualMenuForItems || !preferences.enabled)
        return nil;
    return menuSession_.prepare(FIFinderSyncController.defaultController.selectedItemURLs,
                                preferences.english, self, @selector(runAction:));
}
- (void)runAction:(NSMenuItem *)item {
    NSString *app = xdzip_shell::finderContainingApp(NSBundle.mainBundle.bundlePath);
    const auto preferences = xdzip_shell::finderPreferences(app);
    auto *url = xdzip_shell::finderRequestURL(menuSession_.request(item));
    if (!preferences.enabled)
        return;
    const auto failed = ^{
      dispatch_async(dispatch_get_main_queue(), ^{
        auto *alert = [NSAlert new];
        alert.messageText = @"xdzip";
        alert.informativeText = [NSString
            stringWithUTF8String:xdzip_shell::text(preferences.english, "shell.finderHandoffFailed")
                                     .c_str()];
        [alert runModal];
      });
    };
    if (!url) {
        failed();
        return;
    }
    // Sandboxed launch arguments are ignored by LaunchServices. Send a bounded
    // URL event to our explicit, unsandboxed relay instead of inheriting our sandbox.
    auto *relay = [NSURL fileURLWithPath:[app stringByAppendingPathComponent:
                                                  @"Contents/Helpers/xdzip-shell-relay.app"]];
    auto *configuration = [NSWorkspaceOpenConfiguration configuration];
    configuration.activates = NO;
    configuration.addsToRecentItems = NO;
    configuration.allowsRunningApplicationSubstitution = NO;
    [[NSWorkspace sharedWorkspace] openURLs:@[ url ]
                       withApplicationAtURL:relay
                              configuration:configuration
                          completionHandler:^(NSRunningApplication *application, NSError *error) {
                            (void)application;
                            if (error)
                                failed();
                          }];
}
@end
