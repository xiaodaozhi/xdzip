#import <AppKit/AppKit.h>

#include "DesktopIntegration.h"
#include <QPointer>

// Handle the standard Quit item at the native action boundary. Qt can disable
// it for an owned task prompt, or queue its activation until an unrelated
// modal has closed. Recheck the existing quit policy before either can happen.
@interface XdzipQuitMenuBridge : NSResponder <NSMenuItemValidation> {
    QPointer<DesktopIntegration> _owner;
    __weak NSMenuItem *_item;
    __weak id _previousTarget;
    SEL _previousAction;
}
- (instancetype)initWithOwner:(DesktopIntegration *)owner;
- (void)sync;
- (void)restore;
@end

@implementation XdzipQuitMenuBridge
- (instancetype)initWithOwner:(DesktopIntegration *)owner {
    if ((self = [super init]))
        _owner = owner;
    return self;
}

- (BOOL)worksWhenModal {
    return YES;
}

- (BOOL)validateMenuItem:(NSMenuItem *)item {
    return item == _item && _owner && _owner->nativeQuitMenuAllowed();
}

- (void)requestQuit:(id)sender {
    Q_UNUSED(sender);
    if (_owner && _owner->nativeQuitMenuAllowed())
        _owner->requestQuit();
}

- (void)restore {
    NSMenuItem *item = _item;
    if (item && item.target == self && item.action == @selector(requestQuit:)) {
        item.target = _previousTarget;
        item.action = _previousAction;
    }
    _item = nil;
    _previousTarget = nil;
    _previousAction = nullptr;
}

- (void)sync {
    NSMenu *bar = NSApp.mainMenu;
    NSMenu *menu = bar.numberOfItems ? [bar itemAtIndex:0].submenu : nil;
    NSMenuItem *quit = nil;
    for (NSMenuItem *item in menu.itemArray) {
        const NSEventModifierFlags keys =
            item.keyEquivalentModifierMask & NSEventModifierFlagDeviceIndependentFlagsMask;
        if (!item.hidden && [item.keyEquivalent isEqualToString:@"q"] &&
            keys == NSEventModifierFlagCommand) {
            quit = item;
            break;
        }
    }
    if (!quit) {
        [self restore];
        return;
    }
    if (_item && _item != quit)
        [self restore];
    if (quit.target == self && quit.action == @selector(requestQuit:))
        return;
    // Do not replace an unexpected native action or another component's target.
    if (quit.target || quit.action != NSSelectorFromString(@"qt_itemFired:")) {
        [self restore];
        return;
    }
    _item = quit;
    _previousTarget = quit.target;
    _previousAction = quit.action;
    quit.target = self;
    quit.action = @selector(requestQuit:);
    quit.enabled = YES;
}
@end

// The opaque handle retains one bridge for the DesktopIntegration lifetime.
// All calls are on Qt's GUI thread. A missing or unexpected menu is left alone.
void *createMacQuitMenuBridge(DesktopIntegration *owner) {
    return owner ? (__bridge_retained void *)[[XdzipQuitMenuBridge alloc] initWithOwner:owner]
                 : nullptr;
}

void syncMacQuitMenuBridge(void *bridge) {
    if (bridge)
        [(__bridge XdzipQuitMenuBridge *)bridge sync];
}

void destroyMacQuitMenuBridge(void *bridge) {
    if (!bridge)
        return;
    XdzipQuitMenuBridge *owner = (__bridge_transfer XdzipQuitMenuBridge *)bridge;
    [owner restore];
}
