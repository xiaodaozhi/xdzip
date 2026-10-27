// apps/desktop/NativeExportMac.mm

#include "NativeExport.h"
#import <AppKit/AppKit.h>
#include <QDir>
#import <objc/runtime.h>

static char delegateLeaseKey, dragSourceLeaseKey;
struct MacExportGesture {
    NSEvent *__strong event;
};
void captureNativeExportGesture(QWindow *, const std::shared_ptr<NativeExportState> &state) {
    state->gesture = std::make_shared<MacExportGesture>(MacExportGesture{NSApp.currentEvent});
}
@interface XdzipPromiseDelegate : NSObject <NSFilePromiseProviderDelegate> {
  @public
    std::shared_ptr<NativeExportState> state;
    QString relative;
}
@property(strong) NSOperationQueue *queue;
@end
@implementation XdzipPromiseDelegate
- (NSString *)filePromiseProvider:(NSFilePromiseProvider *)provider
                  fileNameForType:(NSString *)type {
    Q_UNUSED(provider);
    Q_UNUSED(type);
    return relative.toNSString();
}
- (NSOperationQueue *)operationQueueForFilePromiseProvider:(NSFilePromiseProvider *)provider {
    Q_UNUSED(provider);
    return self.queue;
}
- (void)filePromiseProvider:(NSFilePromiseProvider *)provider
          writePromiseToURL:(NSURL *)url
          completionHandler:(void (^)(NSError *))completion {
    Q_UNUSED(provider);
    @autoreleasepool {
        auto complete = [&](NSError *error) {
            if (--state->pendingPromises == 0)
                state->inOperation = false;
            completion(error);
        };
        if (!url.isFileURL || !state->wait()) {
            complete([NSError errorWithDomain:NSCocoaErrorDomain
                                         code:NSFileWriteUnknownError
                                     userInfo:nil]);
            return;
        }
        auto request = std::make_shared<NativeExportCopy>();
        {
            std::lock_guard<std::mutex> lock(state->mutex);
            request->source = QDir(state->directory).filePath(relative);
        }
        request->target = QString::fromNSString(url.path);
        if (state->publish)
            state->publish(request);
        else
            request->finish("runtime_closed");
        std::unique_lock<std::mutex> lock(request->mutex);
        while (!request->done && !state->cancelled.load())
            request->changed.wait_for(lock, std::chrono::milliseconds(100));
        complete(request->done && request->error.isEmpty()
                     ? nil
                     : [NSError errorWithDomain:NSCocoaErrorDomain
                                           code:NSFileWriteUnknownError
                                       userInfo:nil]);
    }
}
@end
@interface XdzipDragSource : NSObject <NSDraggingSource> {
  @public
    std::shared_ptr<NativeExportState> state;
}
@property(weak) NSView *view;
@end
@implementation XdzipDragSource
- (NSDragOperation)draggingSession:(NSDraggingSession *)session
    sourceOperationMaskForDraggingContext:(NSDraggingContext)context {
    Q_UNUSED(session);
    Q_UNUSED(context);
    return NSDragOperationCopy;
}
- (BOOL)ignoreModifierKeysForDraggingSession:(NSDraggingSession *)session {
    Q_UNUSED(session);
    return YES;
}
- (void)draggingSession:(NSDraggingSession *)session
           endedAtPoint:(NSPoint)point
              operation:(NSDragOperation)operation {
    Q_UNUSED(session);
    Q_UNUSED(point);
    if (state->dragFinished)
        state->dragFinished();
    if (operation == NSDragOperationNone) {
        state->inOperation = false;
        state->cancelled = true;
        state->changed.notify_all();
    }
    objc_setAssociatedObject(self.view, &dragSourceLeaseKey, nil,
                             OBJC_ASSOCIATION_RETAIN_NONATOMIC);
}
@end
bool startNativeExport(QWindow *owner, const std::shared_ptr<NativeExportState> &state) {
    NSView *view = (__bridge NSView *)(reinterpret_cast<void *>(owner->winId()));
    NSEvent *event =
        state->gesture ? std::static_pointer_cast<MacExportGesture>(state->gesture)->event : nil;
    if (!view || !event)
        return false;
    auto *source = [XdzipDragSource new];
    source->state = state;
    source.view = view;
    auto *queue = [NSOperationQueue new];
    queue.maxConcurrentOperationCount = 1;
    NSMutableArray<NSDraggingItem *> *items = [NSMutableArray new];
    for (const auto &entry : state->entries) {
        if (entry.name.contains('/'))
            continue;
        auto *delegate = [XdzipPromiseDelegate new];
        delegate->state = state;
        delegate->relative = entry.name;
        delegate.queue = queue;
        auto *provider = [[NSFilePromiseProvider alloc]
            initWithFileType:entry.directory ? @"public.folder" : @"public.data"
                    delegate:delegate];
        // AppKit's delegate is weak. A provider owns its delegate through this
        // association until all native consumers release the promised item.
        objc_setAssociatedObject(provider, &delegateLeaseKey, delegate,
                                 OBJC_ASSOCIATION_RETAIN_NONATOMIC);
        auto *item = [[NSDraggingItem alloc] initWithPasteboardWriter:provider];
        NSPoint point = [view convertPoint:event.locationInWindow fromView:nil];
        [item setDraggingFrame:NSMakeRect(point.x, point.y, 32, 32)
                      contents:[NSImage imageNamed:entry.directory ? NSImageNameFolder
                                                                   : NSImageNameMultipleDocuments]];
        [items addObject:item];
    }
    if (!items.count)
        return false;
    state->pendingPromises = int(items.count);
    state->inOperation = true;
    objc_setAssociatedObject(view, &dragSourceLeaseKey, source, OBJC_ASSOCIATION_RETAIN_NONATOMIC);
    [view beginDraggingSessionWithItems:items event:event source:source];
    return true;
}
