#include "MacCapability.h"
#include "NativeRequest.h"
#include <iostream>

int main(int argc, char **argv) {
    if (argc != 3)
        return 2;
    @autoreleasepool {
        if (!xdzip_shell::desktopExecutable([NSString stringWithUTF8String:argv[2]]))
            return 1;
        auto directory = xdzip_shell::fs::path(argv[1]) / xdzip_shell::randomHex();
        xdzip_shell::fs::create_directories(directory / "fixture.app/Contents");
        NSString *path = [NSString stringWithUTF8String:(directory / "fixture.app").c_str()];
        NSString *plist = [path stringByAppendingPathComponent:@"Contents/Info.plist"];
        NSMutableDictionary *info =
            [@{@"CFBundleIdentifier" : @"org.xdzip.desktop", @"CFBundleExecutable" : @"xdzip"}
                mutableCopy];
        int checks = 1;
        const auto check = [&](bool expected) {
            ++checks;
            return [info writeToFile:plist atomically:YES] &&
                   bool(xdzip_shell::desktopExecutable(path)) == expected;
        };
        bool passed = check(false);
        info[@"XdzipShellProtocol"] = @1;
        passed &= check(true);
        [info removeObjectForKey:@"XdzipShellProtocol"];
        passed &= check(false);
        info[@"XdzipShellProtocol"] = @2;
        passed &= check(false);
        info[@"XdzipShellProtocol"] = @1;
        passed &= check(true);
        info[@"CFBundleExecutable"] = @"../old";
        passed &= check(false);
        info[@"CFBundleExecutable"] = @"xdzip";
        info[@"CFBundleIdentifier"] = @"foreign.product";
        passed &= check(false);
        // Only these exact fixture objects are owned by this native test.
        xdzip_shell::fs::remove(directory / "fixture.app/Contents/Info.plist");
        xdzip_shell::fs::remove(directory / "fixture.app/Contents");
        xdzip_shell::fs::remove(directory / "fixture.app");
        xdzip_shell::fs::remove(directory);
        std::cout << "Mac shell capability: " << checks << " replacement/rollback checks\n";
        return passed ? 0 : 1;
    }
}
