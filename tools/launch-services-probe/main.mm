#import <AppKit/AppKit.h>
#include <cstdio>

// Read the actual document-handler candidates, not all known bundle records.
// Only counts, identities and booleans leave this probe; no user paths do.
int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 3)
            return 2;
        NSURL *file = [NSURL fileURLWithPath:[NSString stringWithUTF8String:argv[1]]];
        NSURL *selected = [[NSURL fileURLWithPath:[NSString stringWithUTF8String:argv[2]]]
            URLByResolvingSymlinksInPath];
        NSWorkspace *workspace = [NSWorkspace sharedWorkspace];
        NSArray<NSURL *> *urls = [workspace URLsForApplicationsToOpenURL:file];
        NSUInteger count = 0;
        BOOL included = NO;
        for (NSURL *url in urls) {
            NSBundle *bundle = [NSBundle bundleWithURL:url];
            if ([bundle.bundleIdentifier isEqualToString:@"org.xdzip.desktop"]) {
                count++;
                included |= [[url URLByResolvingSymlinksInPath] isEqual:selected];
            }
        }
        NSURL *handler = [workspace URLForApplicationToOpenURL:file];
        NSString *identity = handler ? [NSBundle bundleWithURL:handler].bundleIdentifier : nil;
        NSDictionary *record = @{
            @"xdzip_candidates" : @(count),
            @"selected_in_candidates" : @(included),
            @"default_bundle_id" : identity ?: [NSNull null]
        };
        NSData *json = [NSJSONSerialization dataWithJSONObject:record
                                                       options:NSJSONWritingSortedKeys
                                                         error:nil];
        if (!json)
            return 3;
        std::fwrite(json.bytes, 1, json.length, stdout);
        std::fputc('\n', stdout);
    }
    return 0;
}
