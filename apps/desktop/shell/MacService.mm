#include "MacCapability.h"
#include "NativeRequest.h"
#import <AppKit/AppKit.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

@interface XdzipService : NSObject
- (void)openSelection:(NSPasteboard *)pasteboard userData:(NSString *)data error:(NSString **)error;
@end
@implementation XdzipService
- (void)openSelection:(NSPasteboard *)pasteboard
             userData:(NSString *)data
                error:(NSString **)error {
    (void)data;
    @autoreleasepool {
        try {
            const auto bundle = [NSBundle mainBundle];
            const auto location = [bundle URLForResource:@"application" withExtension:@"txt"];
            const auto appPath = [[NSString stringWithContentsOfURL:location
                                                           encoding:NSUTF8StringEncoding
                                                              error:nil]
                stringByTrimmingCharactersInSet:[NSCharacterSet newlineCharacterSet]];
            const auto executable = xdzip_shell::desktopExecutable(appPath);
            if (!executable) {
                if (error)
                    *error = @"xdzip is unavailable. Open the installed application to repair its "
                             @"service.";
                return;
            }
            const auto urls =
                [pasteboard readObjectsForClasses:@[ [NSURL class] ]
                                          options:@{NSPasteboardURLReadingFileURLsOnlyKey : @YES}];
            std::vector<xdzip_shell::fs::path> paths;
            bool archiveCandidates = true;
            for (NSURL *url in urls) {
                if (![url isFileURL]) {
                    paths.clear();
                    break;
                }
                paths.emplace_back([[url path] fileSystemRepresentation]);
                UTType *type = nil;
                [url getResourceValue:&type forKey:NSURLContentTypeKey error:nil];
                archiveCandidates &= [type conformsToType:UTTypeArchive];
            }
            if (!xdzip_shell::launch([executable fileSystemRepresentation], "choose", paths,
                                     archiveCandidates) &&
                error)
                *error = @"xdzip could not accept the selected files. Select up to 128 local items "
                         @"and try again.";
        } catch (...) {
            if (error)
                *error = @"xdzip could not accept this operation.";
        }
    }
}
@end
int main() {
    @autoreleasepool {
        auto *provider = [XdzipService new];
        NSRegisterServicesProvider(provider, @"org.xdzip.shell-service");
        [[NSRunLoop currentRunLoop] run];
    }
    return 0;
}
