#pragma once
#import <Foundation/Foundation.h>

namespace xdzip_shell {
// Read current metadata on every request. NSBundle caches Info.plist across
// replacements, so it cannot decide whether a rolled-back executable understands us.
inline NSString *desktopExecutable(NSString *path) {
    NSString *infoPath = [path stringByAppendingPathComponent:@"Contents/Info.plist"];
    auto *attributes = [[NSFileManager defaultManager] attributesOfItemAtPath:infoPath error:nil];
    if (![attributes[NSFileType] isEqual:NSFileTypeRegular] ||
        [attributes[NSFileSize] unsignedLongLongValue] > 1024 * 1024)
        return nil;
    NSDictionary *info = [NSDictionary dictionaryWithContentsOfFile:infoPath];
    NSString *name = info[@"CFBundleExecutable"];
    if (![@"org.xdzip.desktop" isEqual:info[@"CFBundleIdentifier"]] ||
        ![@1 isEqual:info[@"XdzipShellProtocol"]] || ![name isKindOfClass:[NSString class]] ||
        !name.length || ![[name lastPathComponent] isEqual:name] || [name isEqual:@"."] ||
        [name isEqual:@".."])
        return nil;
    return [[path stringByAppendingPathComponent:@"Contents/MacOS"]
        stringByAppendingPathComponent:name];
}
} // namespace xdzip_shell
