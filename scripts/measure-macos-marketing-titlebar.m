// Measure AppKit's own button geometry without capturing the user's desktop.
#import <Cocoa/Cocoa.h>

static void report(const char *name, NSWindowStyleMask mask) {
    NSWindow *window = [[NSWindow alloc] initWithContentRect:NSMakeRect(0, 0, 1024, 622)
        styleMask:mask backing:NSBackingStoreBuffered defer:NO];
    NSView *view = window.contentView;
    NSArray<NSNumber *> *buttons = @[@(NSWindowCloseButton), @(NSWindowMiniaturizeButton), @(NSWindowZoomButton)];
    printf("%s\n", name);
    for (NSNumber *kind in buttons) {
        NSButton *button = [window standardWindowButton:(NSWindowButton)kind.integerValue];
        if (!button) {
            printf("  %ld: unavailable\n", (long)kind.integerValue);
            continue;
        }
        NSRect bounds = [view convertRect:button.bounds fromView:button];
        // Convert AppKit's view coordinate origin into a top-left offset.
        double top = view.isFlipped ? NSMinY(bounds) : NSHeight(view.bounds) - NSMaxY(bounds);
        printf("  %ld: x=%.3f top=%.3f width=%.3f height=%.3f enabled=%s\n",
               (long)kind.integerValue, NSMinX(bounds), top, NSWidth(bounds), NSHeight(bounds),
               button.enabled ? "yes" : "no");
    }
}

int main(void) {
    @autoreleasepool {
        [NSApplication sharedApplication];
        printf("macOS %s; coordinates are logical points\n", NSProcessInfo.processInfo.operatingSystemVersionString.UTF8String);
        NSWindowStyleMask common = NSWindowStyleMaskTitled | NSWindowStyleMaskClosable;
        report("main before application titlebar alignment", common | NSWindowStyleMaskMiniaturizable |
               NSWindowStyleMaskResizable | NSWindowStyleMaskFullSizeContentView);
        report("operation window", common);
    }
    return 0;
}
