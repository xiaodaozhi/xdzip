#include "MacArchiveLocation.h"
#import <AppKit/AppKit.h>
#include <QString>

bool revealArchiveInFinder(const QString &path, const QString &directory) {
    return [NSWorkspace.sharedWorkspace selectFile:path.toNSString()
                          inFileViewerRootedAtPath:directory.toNSString()];
}
