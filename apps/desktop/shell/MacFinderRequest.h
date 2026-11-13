#pragma once
#include "MacCapability.h"
#include "MenuModel.h"
#import <AppKit/AppKit.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>
#include <cstring>
#include <limits>
#include <pwd.h>
#include <unistd.h>

namespace xdzip_shell {
constexpr NSUInteger finderRequestLimit = 256 * 1024;
inline NSString *finderHome() {
    std::array<char, 16384> buffer{};
    passwd user{};
    passwd *result = nullptr;
    if (getpwuid_r(getuid(), &user, buffer.data(), buffer.size(), &result) != 0 || !result)
        return nil;
    return [[NSFileManager defaultManager] stringWithFileSystemRepresentation:user.pw_dir
                                                                       length:strlen(user.pw_dir)];
}
inline bool finderSchema(id value) {
    return value && CFGetTypeID((__bridge CFTypeRef)value) == CFNumberGetTypeID() &&
           [value isEqual:@1];
}
inline NSString *finderConfigurationPath() {
    return [finderHome()
        stringByAppendingPathComponent:@"Library/Application Support/xdzip/finder-sync.json"];
}
inline NSString *finderContainingApp(NSString *bundlePath) {
    NSString *container = bundlePath.stringByDeletingLastPathComponent;
    NSString *contents = container.stringByDeletingLastPathComponent;
    NSString *app = contents.stringByDeletingLastPathComponent;
    // The extension can read its own bundle, not its containing app's metadata.
    // Derive only our fixed bundle layout here; the unsandboxed relay re-reads
    // and validates the desktop's current capability before every handoff.
    return ([@[ @"PlugIns", @"Helpers" ] containsObject:container.lastPathComponent] &&
            [contents.lastPathComponent isEqual:@"Contents"] && [app.pathExtension isEqual:@"app"])
               ? app
               : nil;
}
inline bool finderCompatible(NSString *app) {
    if (!desktopExecutable(app))
        return false;
    auto *info = [NSDictionary
        dictionaryWithContentsOfFile:[app stringByAppendingPathComponent:@"Contents/Info.plist"]];
    return finderSchema(info[@"XdzipFinderSyncProtocol"]);
}
inline bool validFinderToken(id value) {
    if (![value isKindOfClass:[NSString class]] || [value length] != 64 ||
        ![value canBeConvertedToEncoding:NSASCIIStringEncoding])
        return false;
    const char *bytes = [value UTF8String];
    for (int i = 0; i < 64; ++i)
        if (!((bytes[i] >= '0' && bytes[i] <= '9') || (bytes[i] >= 'a' && bytes[i] <= 'f')))
            return false;
    return true;
}
inline bool finderTokensEqual(id left, id right) {
    if (!validFinderToken(left) || !validFinderToken(right))
        return false;
    const char *a = [left UTF8String], *b = [right UTF8String];
    unsigned char difference = 0;
    for (int i = 0; i < 64; ++i)
        difference |= static_cast<unsigned char>(a[i] ^ b[i]);
    return difference == 0;
}
struct FinderPreferences {
    bool enabled = false;
    bool english = false;
    bool valid = false;
    NSString *handoffToken = nil;
};
inline FinderPreferences finderPreferences(NSString *app,
                                           NSString *path = finderConfigurationPath()) {
    auto *manager = [NSFileManager defaultManager];
    auto *attributes = [manager attributesOfItemAtPath:path error:nil];
    if (![attributes[NSFileType] isEqual:NSFileTypeRegular] ||
        [attributes[NSFileOwnerAccountID] unsignedLongLongValue] != getuid() ||
        [attributes[NSFilePosixPermissions] unsignedLongLongValue] & 0077 ||
        [attributes[NSFileSize] unsignedLongLongValue] > 8192)
        return {};
    auto *bytes = [NSData dataWithContentsOfFile:path];
    if (!bytes || bytes.length > 8192)
        return {};
    id value = [NSJSONSerialization JSONObjectWithData:bytes options:0 error:nil];
    if (![value isKindOfClass:[NSDictionary class]] || !finderSchema(value[@"schema_version"]) ||
        ![value[@"product"] isEqual:@"org.xdzip.finder-sync"] ||
        ![value[@"application"] isKindOfClass:[NSString class]] ||
        ![[value[@"application"] stringByResolvingSymlinksInPath]
            isEqual:[app stringByResolvingSymlinksInPath]] ||
        !value[@"enabled"] ||
        CFGetTypeID((__bridge CFTypeRef)value[@"enabled"]) != CFBooleanGetTypeID() ||
        !validFinderToken(value[@"handoff_token"]))
        return {};
    return {[value[@"enabled"] boolValue],
            [value[@"language"] isEqual:@"en-US"],
            true,
            value[@"handoff_token"]};
}
inline NSDictionary *finderRequest(NSString *action, NSArray<NSURL *> *urls) {
    if (urls.count == 0 || urls.count > 128)
        return nil;
    auto *paths = [NSMutableArray new];
    bool archives = true;
    for (NSURL *url in urls) {
        if (![url isKindOfClass:[NSURL class]] || !url.isFileURL || !url.path.isAbsolutePath ||
            [url.path lengthOfBytesUsingEncoding:NSUTF8StringEncoding] > 4096)
            return nil;
        [paths addObject:url.path];
        auto *type = [UTType typeWithFilenameExtension:url.pathExtension];
        NSNumber *directory = nil;
        [url getResourceValue:&directory forKey:NSURLIsDirectoryKey error:nil];
        archives &= !url.hasDirectoryPath && !directory.boolValue &&
                    ([type conformsToType:UTTypeArchive] ||
                     archiveCandidate(fs::path(url.path.fileSystemRepresentation)));
    }
    return @{
        @"schema_version" : @1,
        @"id" : NSUUID.UUID.UUIDString,
        @"action" : action,
        @"paths" : paths,
        @"archive_candidates" : @(archives)
    };
}
inline bool validFinderRequest(id value) {
    if (![value isKindOfClass:[NSDictionary class]] || !finderSchema(value[@"schema_version"]) ||
        ![value[@"id"] isKindOfClass:[NSString class]] || [value[@"id"] length] != 36 ||
        ![value[@"id"] canBeConvertedToEncoding:NSASCIIStringEncoding] ||
        !validRequestIdentity([value[@"id"] UTF8String]) ||
        ![value[@"paths"] isKindOfClass:[NSArray class]] || [value[@"paths"] count] == 0 ||
        [value[@"paths"] count] > 128 || ![value[@"action"] isKindOfClass:[NSString class]] || ![@[
            @"zip", @"7z", @"create", @"separate-zip", @"extract-here", @"extract-smart",
            @"extract-separate", @"extract-to"
        ] containsObject:value[@"action"]] ||
        !value[@"archive_candidates"] ||
        CFGetTypeID((__bridge CFTypeRef)value[@"archive_candidates"]) != CFBooleanGetTypeID())
        return false;
    for (id path in value[@"paths"])
        if (![path isKindOfClass:[NSString class]] || ![path isAbsolutePath] ||
            ![path canBeConvertedToEncoding:NSUTF8StringEncoding] ||
            [path lengthOfBytesUsingEncoding:NSUTF8StringEncoding] > 4096 ||
            [path rangeOfString:[NSString stringWithFormat:@"%C", (unichar)0]].location !=
                NSNotFound)
            return false;
    return true;
}
inline NSURL *finderRequestURL(NSDictionary *request, NSString *token) {
    if (!validFinderRequest(request) || !validFinderToken(token))
        return nil;
    NSMutableDictionary *envelope = [request mutableCopy];
    envelope[@"handoff_token"] = token;
    auto *data = [NSJSONSerialization dataWithJSONObject:envelope options:0 error:nil];
    if (!data || data.length > finderRequestLimit)
        return nil;
    NSString *encoded = [data base64EncodedStringWithOptions:0];
    encoded = [[encoded stringByReplacingOccurrencesOfString:@"+" withString:@"-"]
        stringByReplacingOccurrencesOfString:@"/"
                                  withString:@"_"];
    encoded = [encoded stringByReplacingOccurrencesOfString:@"=" withString:@""];
    return [NSURL URLWithString:[@"xdzip-shell://v1/" stringByAppendingString:encoded]];
}
inline NSDictionary *finderRequestFromURL(NSURL *url, NSString *token) {
    if (![url.scheme isEqual:@"xdzip-shell"] || ![url.host isEqual:@"v1"] || url.query ||
        url.fragment || url.user || url.password || url.port || url.path.length < 2 ||
        url.path.length > (finderRequestLimit + 2) / 3 * 4 + 1)
        return nil;
    NSString *encoded = [url.path substringFromIndex:1];
    auto *allowed =
        [NSCharacterSet characterSetWithCharactersInString:
                            @"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"];
    if ([encoded rangeOfCharacterFromSet:allowed.invertedSet].location != NSNotFound)
        return nil;
    encoded = [[encoded stringByReplacingOccurrencesOfString:@"-" withString:@"+"]
        stringByReplacingOccurrencesOfString:@"_"
                                  withString:@"/"];
    while (encoded.length % 4)
        encoded = [encoded stringByAppendingString:@"="];
    auto *data = [[NSData alloc] initWithBase64EncodedString:encoded options:0];
    if (!data || data.length > finderRequestLimit)
        return nil;
    id value = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];
    if (![value isKindOfClass:[NSDictionary class]] ||
        !finderTokensEqual(value[@"handoff_token"], token))
        return nil;
    // The private file and desktop single-instance packet never carry this secret.
    NSMutableDictionary *request = [value mutableCopy];
    [request removeObjectForKey:@"handoff_token"];
    return validFinderRequest(request) ? request : nil;
}
inline NSMenu *finderMenu(NSArray<NSURL *> *urls, bool english, id target, SEL action) {
    auto *snapshot = finderRequest(@"zip", urls);
    if (!snapshot)
        return nil;
    std::vector<fs::path> paths;
    for (NSString *path in snapshot[@"paths"])
        paths.emplace_back(path.fileSystemRepresentation);
    const auto entries = menu(paths, english, [snapshot[@"archive_candidates"] boolValue]);
    auto *menu = [[NSMenu alloc] initWithTitle:@""];
    auto *root = [[NSMenuItem alloc]
        initWithTitle:[NSString stringWithUTF8String:text(english, "shell.title").c_str()]
               action:nullptr
        keyEquivalent:@""];
    auto *submenu = [[NSMenu alloc] initWithTitle:root.title];
    for (const auto &entry : entries) {
        auto *item =
            [[NSMenuItem alloc] initWithTitle:[NSString stringWithUTF8String:entry.label.c_str()]
                                       action:action
                                keyEquivalent:@""];
        item.target = target;
        NSMutableDictionary *request = [snapshot mutableCopy];
        request[@"action"] = [NSString stringWithUTF8String:entry.action.c_str()];
        item.representedObject = request;
        [submenu addItem:item];
    }
    root.submenu = submenu;
    [menu addItem:root];
    return menu;
}
class FinderMenuSession {
    NSDictionary *requests_ = nil;
    NSInteger nextTag_ = 0;

  public:
    NSMenu *prepare(NSArray<NSURL *> *urls, bool english, id target, SEL action) {
        if (nextTag_ > std::numeric_limits<int>::max() - 8) {
            requests_ = nil;
            return nil;
        }
        auto *menu = finderMenu(urls, english, target, action);
        auto *requests = [NSMutableDictionary new];
        for (NSMenuItem *item in menu.itemArray.firstObject.submenu.itemArray) {
            item.tag = ++nextTag_;
            requests[@(item.tag)] = item.representedObject;
        }
        requests_ = requests;
        return menu;
    }
    NSDictionary *request(NSMenuItem *item) const {
        // Finder forwards the tag, but does not preserve representedObject.
        // Retain one bounded menu snapshot; stale actions cannot select new paths.
        return requests_[@(item.tag)];
    }
};
} // namespace xdzip_shell
