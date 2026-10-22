// apps/desktop/MacUpdateProvider.mm

#include "MacUpdateProvider.h"
#include "UpdateController.h"
#import <AppKit/AppKit.h>
#include <QCoreApplication>
#include <QDir>
#include <QFileInfo>
#include <QPointer>
#import <Sparkle/Sparkle.h>
#include <cstdint>
#import <dispatch/dispatch.h>

// Sparkle owns extraction, authorization, replacement, and relaunch. Only the
// application's completed join barrier is allowed to release installReply.
@interface XdzipUpdateDriver : NSObject <SPUUserDriver, SPUUpdaterDelegate> {
  @public
    QPointer<UpdateController> owner;
    QJsonObject candidate;
    std::function<void(QString)> prepared;
    SPUUpdater *updater;
    void (^installReply)(SPUUserUpdateChoice);
    void (^cancelDownload)(void);
    uint64_t received;
    uint64_t total;
    BOOL preparationReported;
    BOOL cancelled;
    BOOL committed;
}
- (BOOL)matches:(SUAppcastItem *)item;
- (void)failure;
@end

namespace {
XdzipUpdateDriver *driver = nil;
NSString *native(const QString &value) {
    return [NSString stringWithUTF8String:value.toUtf8().constData()];
}
QString text(NSString *value) {
    return value ? QString::fromUtf8(value.UTF8String) : QString{};
}
} // namespace

@implementation XdzipUpdateDriver
- (BOOL)matches:(SUAppcastItem *)item {
    return item.signingValidationStatus == SPUAppcastSigningValidationStatusSucceeded &&
           !item.informationOnlyUpdate &&
           text(item.versionString) == QString::number(candidate.value("build").toInt()) &&
           text(item.displayVersionString) == candidate.value("version").toString() &&
           text(item.fileURL.absoluteString) == candidate.value("asset_url").toString() &&
           item.contentLength == quint64(candidate.value("bytes").toDouble());
}
- (void)failure {
    if (cancelled)
        return;
    if (owner)
        owner->reportNativeProgress(QStringLiteral("error"), -1);
    if (prepared) {
        auto callback = std::move(prepared);
        callback(QStringLiteral("native_update"));
    }
}
- (NSString *)feedURLStringForUpdater:(SPUUpdater *)sender {
    (void)sender;
    return native(candidate.value("appcast_url").toString());
}
- (BOOL)updaterShouldPromptForPermissionToCheckForUpdates:(SPUUpdater *)sender {
    (void)sender;
    return NO;
}
- (NSArray<NSString *> *)allowedSystemProfileKeysForUpdater:(SPUUpdater *)sender {
    (void)sender;
    return @[];
}
- (NSSet<NSString *> *)allowedChannelsForUpdater:(SPUUpdater *)sender {
    (void)sender;
    NSString *channel = native(candidate.value("channel").toString());
    return [NSSet setWithObject:channel];
}
- (BOOL)updater:(SPUUpdater *)sender
    shouldProceedWithUpdate:(SUAppcastItem *)item
                updateCheck:(SPUUpdateCheck)check
                      error:(NSError *__autoreleasing *)error {
    (void)sender;
    (void)check;
    if (!cancelled && [self matches:item])
        return YES;
    if (error)
        *error = [NSError errorWithDomain:@"org.xdzip.update"
                                     code:1
                                 userInfo:@{
                                     NSLocalizedDescriptionKey :
                                         @"The selected update does not match its verified release."
                                 }];
    return NO;
}
- (BOOL)updater:(SPUUpdater *)sender shouldDownloadReleaseNotesForUpdate:(SUAppcastItem *)item {
    (void)sender;
    (void)item;
    return NO;
}
- (void)showUpdatePermissionRequest:(SPUUpdatePermissionRequest *)request
                              reply:(void (^)(SUUpdatePermissionResponse *))reply {
    (void)request;
    reply([[SUUpdatePermissionResponse alloc] initWithAutomaticUpdateChecks:NO
                                                          sendSystemProfile:NO]);
}
- (void)showUserInitiatedUpdateCheckWithCancellation:(void (^)(void))cancellation {
    if (cancelled) {
        cancellation();
        return;
    }
    cancelDownload = [cancellation copy];
    if (owner)
        owner->reportNativeProgress(QStringLiteral("checking"), -1);
}
- (void)showUpdateFoundWithAppcastItem:(SUAppcastItem *)item
                                 state:(SPUUserUpdateState *)state
                                 reply:(void (^)(SPUUserUpdateChoice))reply {
    if (cancelled || !owner || ![self matches:item]) {
        reply(SPUUserUpdateChoiceSkip);
        [self failure];
        return;
    }
    if (state.stage == SPUUserUpdateStageInstalling) {
        [self showReadyToInstallAndRelaunch:reply];
    } else {
        reply(SPUUserUpdateChoiceInstall);
    }
}
- (void)showUpdateReleaseNotesWithDownloadData:(SPUDownloadData *)data {
    (void)data;
}
- (void)showUpdateReleaseNotesFailedToDownloadWithError:(NSError *)error {
    (void)error;
}
- (void)showUpdateNotFoundWithError:(NSError *)error
                    acknowledgement:(void (^)(void))acknowledgement {
    (void)error;
    [self failure];
    acknowledgement();
}
- (void)showUpdaterError:(NSError *)error acknowledgement:(void (^)(void))acknowledgement {
    (void)error;
    [self failure];
    acknowledgement();
}
- (void)showDownloadInitiatedWithCancellation:(void (^)(void))cancellation {
    if (cancelled) {
        cancellation();
        return;
    }
    cancelDownload = [cancellation copy];
    received = 0;
    total = 0;
    if (owner)
        owner->reportNativeProgress(QStringLiteral("downloading"), -1);
}
- (void)showDownloadDidReceiveExpectedContentLength:(uint64_t)length {
    total = length;
}
- (void)showDownloadDidReceiveDataOfLength:(uint64_t)length {
    if (cancelled)
        return;
    if (UINT64_MAX - received < length) {
        [self failure];
        return;
    }
    received += length;
    if (owner)
        owner->reportNativeProgress(QStringLiteral("downloading"),
                                    total ? qMin(qreal(1), qreal(received) / qreal(total)) : -1);
}
- (void)showDownloadDidStartExtractingUpdate {
    cancelDownload = nil;
    if (!cancelled && owner)
        owner->reportNativeProgress(QStringLiteral("verifying"), -1);
}
- (void)showExtractionReceivedProgress:(double)progress {
    if (!cancelled && owner)
        owner->reportNativeProgress(QStringLiteral("verifying"), progress);
}
- (void)showReadyToInstallAndRelaunch:(void (^)(SPUUserUpdateChoice))reply {
    // Cancellation can arrive during extraction, when Sparkle offers no download
    // cancellation block. Cancel the pending installer when it next becomes ready.
    if (cancelled || !owner) {
        reply(SPUUserUpdateChoiceSkip);
        return;
    }
    if (installReply) {
        reply(SPUUserUpdateChoiceSkip);
        [self failure];
        return;
    }
    installReply = [reply copy];
    if (!preparationReported && prepared) {
        preparationReported = YES;
        auto callback = std::move(prepared);
        callback({});
    }
}
- (void)showInstallingUpdateWithApplicationTerminated:(BOOL)terminated
                          retryTerminatingApplication:(void (^)(void))retry {
    if (owner)
        owner->reportNativeProgress(QStringLiteral("installing"), -1);
    (void)retry;
    // Sparkle has already started its installer before this callback. The Rust
    // join barrier released commitMacUpdate(), so the normal Qt quit is now safe.
    // Do not repeatedly invoke retry while Cocoa is processing its quit request.
    if (!terminated && committed && owner)
        emit owner->quitForUpdate();
}
- (void)showUpdateInstalledAndRelaunched:(BOOL)relaunched
                         acknowledgement:(void (^)(void))acknowledgement {
    (void)relaunched;
    acknowledgement();
}
- (void)dismissUpdateInstallation {
    cancelDownload = nil;
    installReply = nil;
}
- (BOOL)updaterShouldRelaunchApplication:(SPUUpdater *)sender {
    (void)sender;
    return committed && !cancelled;
}
- (void)updater:(SPUUpdater *)sender
    didFinishUpdateCycleForUpdateCheck:(SPUUpdateCheck)check
                                 error:(NSError *)error {
    (void)check;
    (void)error;
    // SPUUpdater retains its user driver. Release our reverse reference after
    // Sparkle returns from the completed cycle, so retries do not leak a pair.
    dispatch_async(dispatch_get_main_queue(), ^{
      if (self->updater == sender)
          self->updater = nil;
    });
}
@end

QString macUpdateInstallRoot() {
    NSBundle *bundle = NSBundle.mainBundle;
    NSURL *url = bundle.bundleURL;
    if (![bundle.bundleIdentifier isEqualToString:@"org.xdzip.desktop"] ||
        ![[bundle objectForInfoDictionaryKey:@"XdzipUpdateInstallable"] boolValue] ||
        ![[bundle objectForInfoDictionaryKey:@"XdzipUpdateInstallId"]
            isEqualToString:@"org.xdzip.desktop.release.v1"] ||
        ![[bundle objectForInfoDictionaryKey:@"SUPublicEDKey"] length])
        return {};
    const auto path = text(url.path);
    const QFileInfo installation(path);
    const auto parent = installation.dir().canonicalPath();
    if (installation.isSymLink() ||
        (parent != QStringLiteral("/Applications") &&
         parent !=
             QFileInfo(QDir::homePath() + QStringLiteral("/Applications")).canonicalFilePath()))
        return {};
    if (!path.endsWith(".app") || path.startsWith("/Volumes/") ||
        path.contains("/AppTranslocation/"))
        return {};
    NSNumber *readOnly = nil;
    if (![url getResourceValue:&readOnly forKey:NSURLVolumeIsReadOnlyKey error:nil] ||
        readOnly.boolValue)
        return {};
    // Sparkle handles normal authorization for an installed, non-writable app.
    return path;
}
void startMacUpdate(UpdateController *owner, const QJsonObject &candidate,
                    std::function<void(QString)> prepared) {
    if (driver && driver->updater.sessionInProgress) {
        prepared("update_busy");
        return;
    }
    driver = [[XdzipUpdateDriver alloc] init];
    driver->owner = owner;
    driver->candidate = candidate;
    driver->prepared = std::move(prepared);
    driver->updater = [[SPUUpdater alloc] initWithHostBundle:NSBundle.mainBundle
                                           applicationBundle:NSBundle.mainBundle
                                                  userDriver:driver
                                                    delegate:driver];
    driver->updater.automaticallyChecksForUpdates = NO;
    driver->updater.automaticallyDownloadsUpdates = NO;
    NSError *error = nil;
    if (![driver->updater startUpdater:&error]) {
        [driver failure];
        driver->updater = nil;
        return;
    }
    [driver->updater checkForUpdates];
}
bool commitMacUpdate() {
    if (!driver || driver->cancelled || driver->committed || !driver->installReply)
        return false;
    driver->committed = YES;
    auto reply = driver->installReply;
    driver->installReply = nil;
    reply(SPUUserUpdateChoiceInstall);
    return true;
}
void cancelMacUpdate() {
    if (!driver)
        return;
    if (driver->committed)
        return;
    driver->cancelled = YES;
    driver->prepared = {};
    if (driver->installReply) {
        auto reply = driver->installReply;
        driver->installReply = nil;
        // Dismiss would leave an installation scheduled for the next quit.
        reply(SPUUserUpdateChoiceSkip);
    }
    if (driver->cancelDownload) {
        auto cancel = driver->cancelDownload;
        driver->cancelDownload = nil;
        cancel();
    }
}
#ifdef XDZIP_UPDATE_TESTS
bool macUpdateLifecycleProbe(UpdateController *owner) {
    // Exercise the real Sparkle user-driver callbacks, without starting a
    // network session, authorizing a helper, or modifying an installed app.
    auto previous = driver;
    driver = [[XdzipUpdateDriver alloc] init];
    driver->owner = owner;
    __block int cancelledReplies = 0;
    __block bool cancelledWithSkip = false;
    bool preparedAfterCancellation = false;
    driver->prepared = [&](QString) { preparedAfterCancellation = true; };
    [driver showDownloadDidStartExtractingUpdate];
    cancelMacUpdate();
    [driver showReadyToInstallAndRelaunch:^(SPUUserUpdateChoice choice) {
      ++cancelledReplies;
      cancelledWithSkip = choice == SPUUserUpdateChoiceSkip;
    }];
    const bool cancellationPassed = cancelledReplies == 1 && cancelledWithSkip &&
                                    !preparedAfterCancellation && !commitMacUpdate();
    driver = [[XdzipUpdateDriver alloc] init];
    driver->owner = owner;
    int preparedCount = 0;
    __block int installReplies = 0;
    __block bool installed = false;
    driver->prepared = [&](QString error) {
        if (error.isEmpty())
            ++preparedCount;
    };
    [driver showReadyToInstallAndRelaunch:^(SPUUserUpdateChoice choice) {
      ++installReplies;
      installed = choice == SPUUserUpdateChoiceInstall;
    }];
    const bool held = preparedCount == 1 && installReplies == 0;
    const bool committedOnce = commitMacUpdate() && !commitMacUpdate();
    const bool installationPassed = held && committedOnce && installed && installReplies == 1;
    driver = previous;
    return cancellationPassed && installationPassed;
}
#endif
