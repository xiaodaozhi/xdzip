// Owned Sparkle transport regression, stops before extraction or installation.
#import <AppKit/AppKit.h>
#import <Sparkle/Sparkle.h>
#include <unistd.h>

static NSMutableDictionary *evidence;
static NSDictionary *parameters;
static NSMutableArray *events;

static void finish(NSString *outcome, NSError *error) {
    evidence[@"outcome"] = outcome;
    NSMutableArray *chain = [NSMutableArray array];
    for (int depth = 0; error && depth < 5; depth++) {
        [chain addObject:@{@"domain" : error.domain ?: @"", @"code" : @(error.code)}];
        error = error.userInfo[NSUnderlyingErrorKey];
    }
    evidence[@"errors"] = chain;
    evidence[@"events"] = events;
    NSData *json = [NSJSONSerialization dataWithJSONObject:evidence
                                                   options:NSJSONWritingPrettyPrinted
                                                     error:nil];
    [json writeToFile:parameters[@"result"] atomically:YES];
    // This owned probe ends before extraction or any installer can be started.
    _exit(0);
}

@interface ResearchDriver : NSObject <SPUUserDriver, SPUUpdaterDelegate>
@end
@implementation ResearchDriver
- (NSString *)feedURLStringForUpdater:(SPUUpdater *)updater {
    return parameters[@"feed_url"];
}
- (BOOL)updaterShouldPromptForPermissionToCheckForUpdates:(SPUUpdater *)updater {
    return NO;
}
- (NSArray *)allowedSystemProfileKeysForUpdater:(SPUUpdater *)updater {
    return @[];
}
- (BOOL)updater:(SPUUpdater *)updater shouldDownloadReleaseNotesForUpdate:(SUAppcastItem *)item {
    return NO;
}
- (BOOL)updater:(SPUUpdater *)updater
    shouldProceedWithUpdate:(SUAppcastItem *)item
                updateCheck:(SPUUpdateCheck)check
                      error:(NSError **)error {
    BOOL valid = item.signingValidationStatus == SPUAppcastSigningValidationStatusSucceeded &&
                 [item.fileURL.absoluteString isEqualToString:parameters[@"original_asset_url"]] &&
                 [item.versionString isEqualToString:@"101"] &&
                 item.contentLength == [parameters[@"bytes"] unsignedLongLongValue];
    evidence[@"candidate_matched"] = @(valid);
    evidence[@"feed_signature_valid"] =
        @(item.signingValidationStatus == SPUAppcastSigningValidationStatusSucceeded);
    if (!valid)
        *error = [NSError errorWithDomain:@"org.xdzip.owned-research" code:1 userInfo:nil];
    return valid;
}
- (void)updater:(SPUUpdater *)updater
    willDownloadUpdate:(SUAppcastItem *)item
           withRequest:(NSMutableURLRequest *)request {
    BOOL original = [request.URL.absoluteString isEqualToString:parameters[@"original_asset_url"]];
    evidence[@"original_request_matched"] = @(original);
    request.URL = [NSURL URLWithString:parameters[@"asset_transport_url"]];
    evidence[@"mapped_scheme"] = request.URL.scheme;
    [events addObject:@"archive-request-mapped"];
}
- (void)updater:(SPUUpdater *)updater didDownloadUpdate:(SUAppcastItem *)item {
    evidence[@"archive_downloaded"] = @YES;
    finish(@"downloaded_before_extraction", nil);
}
- (void)updater:(SPUUpdater *)updater didAbortWithError:(NSError *)error {
    finish(@"native_error", error);
}
- (void)updater:(SPUUpdater *)updater
    failedToDownloadUpdate:(SUAppcastItem *)item
                     error:(NSError *)error {
    finish(@"download_error", error);
}
- (void)updater:(SPUUpdater *)updater
    didFinishUpdateCycleForUpdateCheck:(SPUUpdateCheck)check
                                 error:(NSError *)error {
    finish(@"cycle_finished", error);
}
- (void)showUpdatePermissionRequest:(SPUUpdatePermissionRequest *)request
                              reply:(void (^)(SUUpdatePermissionResponse *))reply {
    reply([[SUUpdatePermissionResponse alloc] initWithAutomaticUpdateChecks:NO
                                                          sendSystemProfile:NO]);
}
- (void)showUserInitiatedUpdateCheckWithCancellation:(void (^)(void))cancellation {
    [events addObject:@"checking"];
}
- (void)showUpdateFoundWithAppcastItem:(SUAppcastItem *)item
                                 state:(SPUUserUpdateState *)state
                                 reply:(void (^)(SPUUserUpdateChoice))reply {
    if (state.stage != SPUUserUpdateStageNotDownloaded)
        finish(@"unexpected_cached_stage", nil);
    [events addObject:@"update-found"];
    reply(SPUUserUpdateChoiceInstall);
}
- (void)showUpdateReleaseNotesWithDownloadData:(SPUDownloadData *)data {
}
- (void)showUpdateReleaseNotesFailedToDownloadWithError:(NSError *)error {
    finish(@"unexpected_release_notes", error);
}
- (void)showUpdateNotFoundWithError:(NSError *)error
                    acknowledgement:(void (^)(void))acknowledgement {
    finish(@"no_update", error);
}
- (void)showUpdaterError:(NSError *)error acknowledgement:(void (^)(void))acknowledgement {
    finish(@"updater_error", error);
}
- (void)showDownloadInitiatedWithCancellation:(void (^)(void))cancellation {
    [events addObject:@"downloading"];
}
- (void)showDownloadDidReceiveExpectedContentLength:(uint64_t)length {
    evidence[@"expected_content_length"] = @(length);
}
- (void)showDownloadDidReceiveDataOfLength:(uint64_t)length {
    evidence[@"received_bytes"] = @([evidence[@"received_bytes"] unsignedLongLongValue] + length);
}
- (void)showDownloadDidStartExtractingUpdate {
    finish(@"unexpected_extraction", nil);
}
- (void)showExtractionReceivedProgress:(double)progress {
    finish(@"unexpected_extraction", nil);
}
- (void)showReadyToInstallAndRelaunch:(void (^)(SPUUserUpdateChoice))reply {
    reply(SPUUserUpdateChoiceSkip);
    finish(@"unexpected_installer", nil);
}
- (void)showInstallingUpdateWithApplicationTerminated:(BOOL)terminated
                          retryTerminatingApplication:(void (^)(void))retry {
    finish(@"unexpected_install", nil);
}
- (void)showUpdateInstalledAndRelaunched:(BOOL)relaunched
                         acknowledgement:(void (^)(void))acknowledgement {
    finish(@"unexpected_install", nil);
}
- (void)dismissUpdateInstallation {
}
@end

int main(int argc, char **argv) {
    @autoreleasepool {
        if (argc != 2)
            return 2;
        parameters =
            [NSJSONSerialization JSONObjectWithData:[NSData dataWithContentsOfFile:@(argv[1])]
                                            options:0
                                              error:nil];
        if (![NSHomeDirectory() isEqualToString:parameters[@"isolated_home"]])
            return 3;
        evidence = [NSMutableDictionary dictionaryWithDictionary:@{
            @"home_isolated" : @YES,
            @"case" : parameters[@"case"]
        }];
        events = [NSMutableArray array];
        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyProhibited];
        ResearchDriver *driver = [ResearchDriver new];
        SPUUpdater *updater = [[SPUUpdater alloc] initWithHostBundle:NSBundle.mainBundle
                                                   applicationBundle:NSBundle.mainBundle
                                                          userDriver:driver
                                                            delegate:driver];
        updater.httpHeaders = @{@"X-Xdzip-Research-Token" : parameters[@"token"]};
        NSError *error;
        NSURL *proxy =
            parameters[@"proxy_url"] ? [NSURL URLWithString:parameters[@"proxy_url"]] : nil;
        if (![updater xdzip_setDownloadProxyURL:proxy error:&error])
            finish(@"configuration_rejected", error);
        if (![updater startUpdater:&error])
            finish(@"start_error", error);
        evidence[@"proxy_change_rejected_after_start"] =
            @(![updater xdzip_setDownloadProxyURL:nil error:&error]);
        [updater checkForUpdates];
        dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 15 * NSEC_PER_SEC),
                       dispatch_get_main_queue(), ^{
                         finish(@"timeout", nil);
                       });
        [[NSRunLoop mainRunLoop] run];
    }
    return 0;
}
