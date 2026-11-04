#include "FileAssociations.h"
#import <AppKit/AppKit.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>
#include <cstdio>

bool macAssociationDefault(const QString &extension);

// The probe exercises only the read-only default query, not the change callback.
void FileAssociations::nativeFinished(bool) {}

int main(int argc, char **argv) {
    @autoreleasepool {
        if (argc != 2)
            return 2;
        NSURL *file = [NSURL fileURLWithPath:[NSString stringWithUTF8String:argv[1]]];
        NSURL *handler = [[NSWorkspace sharedWorkspace] URLForApplicationToOpenURL:file];
        bool expected = handler && [[[NSBundle bundleWithURL:handler] bundleIdentifier]
                                       isEqualToString:@"org.xdzip.desktop"];
        UTType *legacyType = [UTType typeWithIdentifier:@"org.xdzip.udf-image"];
        NSURL *legacyHandler = legacyType ? [[NSWorkspace sharedWorkspace]
                                                URLForApplicationToOpenContentType:legacyType]
                                          : nil;
        bool legacy = legacyHandler && [[[NSBundle bundleWithURL:legacyHandler] bundleIdentifier]
                                           isEqualToString:@"org.xdzip.desktop"];
        bool actual = macAssociationDefault(QStringLiteral(".udf"));
        std::printf("{\"file_default_is_xdzip\":%s,\"settings_is_xdzip\":%s,"
                    "\"legacy_type_default_is_xdzip\":%s}\n",
                    expected ? "true" : "false", actual ? "true" : "false",
                    legacy ? "true" : "false");
        return actual == expected ? 0 : 1;
    }
}
