// Owned GitHub-hosted fixture only. The product never changes system proxies.
#import <CFNetwork/CFNetwork.h>
#import <Foundation/Foundation.h>
#import <SystemConfiguration/SystemConfiguration.h>
#include <fcntl.h>
#include <sys/select.h>
#include <sys/stat.h>
#include <unistd.h>

static NSString *const fixtureHost = @"xdzip-owned-direct.invalid";
static int hostsFD = -1;
static struct stat hostsIdentity;
static NSData *originalHosts;
static SCPreferencesRef preferences;
static SCNetworkProtocolRef proxyProtocol;
static NSDictionary *originalConfiguration;
static NSDictionary *originalGlobal;
static BOOL originalEnabled;
static BOOL preferencesChanged;

static void requireOwned(BOOL valid, NSString *stage) {
    if (!valid)
        @throw [NSException exceptionWithName:@"OwnedFixtureFailure" reason:stage userInfo:nil];
}
static void emit(NSDictionary *value) {
    NSData *data = [NSJSONSerialization dataWithJSONObject:value options:0 error:nil];
    requireOwned(data != nil, @"encode_result");
    fwrite(data.bytes, 1, data.length, stdout);
    fputc('\n', stdout);
    fflush(stdout);
}
static BOOL hosted(void) {
    NSDictionary *env = NSProcessInfo.processInfo.environment;
    return [env[@"CI"] isEqual:@"true"] && [env[@"GITHUB_ACTIONS"] isEqual:@"true"] &&
           [env[@"RUNNER_ENVIRONMENT"] isEqual:@"github-hosted"] &&
           [env[@"XDZIP_ISOLATED_UPDATE_FIXTURE"] isEqual:@"1"];
}
static void writeHosts(NSData *data) {
    struct stat current;
    // Hold one no-follow descriptor to the fixed root-owned regular file. A
    // changed inode aborts writes; a partial failure is restored in @finally.
    requireOwned(hostsFD >= 0 && lstat("/etc/hosts", &current) == 0 &&
                     current.st_dev == hostsIdentity.st_dev &&
                     current.st_ino == hostsIdentity.st_ino,
                 @"hosts_identity_changed");
    const uint8_t *bytes = data.bytes;
    size_t offset = 0;
    while (offset < data.length) {
        ssize_t count = pwrite(hostsFD, bytes + offset, data.length - offset, (off_t)offset);
        requireOwned(count > 0, @"hosts_write");
        offset += (size_t)count;
    }
    requireOwned(ftruncate(hostsFD, (off_t)data.length) == 0 && fsync(hostsFD) == 0,
                 @"hosts_commit");
}
static void apply(NSDictionary *configuration, BOOL enabled) {
    // The held preference lock scopes changes to this one active service's
    // proxy protocol. All original fields and the enabled bit are restored.
    preferencesChanged = YES;
    requireOwned(
        SCNetworkProtocolSetConfiguration(proxyProtocol, (__bridge CFDictionaryRef)configuration) &&
            SCNetworkProtocolSetEnabled(proxyProtocol, enabled) &&
            SCPreferencesCommitChanges(preferences) && SCPreferencesApplyChanges(preferences),
        @"proxy_apply");
}
static NSDictionary *globalProxies(void) {
    return CFBridgingRelease(SCDynamicStoreCopyProxies(NULL)) ?: @{};
}
static BOOL globalFlagsRestored(void) {
    NSDictionary *current = globalProxies();
    for (NSString *key in @[
             @"HTTPEnable", @"HTTPSEnable", @"SOCKSEnable", @"ProxyAutoConfigEnable",
             @"ProxyAutoDiscoveryEnable"
         ])
        if ([current[key] boolValue] != [originalGlobal[key] boolValue])
            return NO;
    return YES;
}
static BOOL profileVisible(NSString *profile, NSInteger port, NSString *pacURL) {
    NSDictionary *global = globalProxies();
    if ([profile isEqual:@"static"])
        return [global[(__bridge NSString *)kCFNetworkProxiesHTTPEnable] boolValue] &&
               [global[(__bridge NSString *)kCFNetworkProxiesHTTPProxy] isEqual:@"127.0.0.1"] &&
               [global[(__bridge NSString *)kCFNetworkProxiesHTTPPort] integerValue] == port &&
               [global[(__bridge NSString *)kCFNetworkProxiesHTTPSEnable] boolValue] &&
               ![global[(__bridge NSString *)kCFNetworkProxiesProxyAutoConfigEnable] boolValue];
    return [global[(__bridge NSString *)kCFNetworkProxiesProxyAutoConfigEnable] boolValue] &&
           [global[(__bridge NSString *)kCFNetworkProxiesProxyAutoConfigURLString] isEqual:pacURL];
}

int main(int argc, char **argv) {
    @autoreleasepool {
#if !defined(__arm64__)
        return 2;
#endif
        if (argc != 3 || !hosted() || geteuid() != 0)
            return 2;
        NSInteger port = [@(argv[1]) integerValue];
        NSURL *pac = [NSURL URLWithString:@(argv[2])];
        if (port < 1 || port > 65535 || ![pac.scheme isEqual:@"http"] ||
            ![pac.host isEqual:@"127.0.0.1"] || ![pac.path isEqual:@"/proxy.pac"] || pac.user ||
            pac.password || pac.fragment || pac.port.integerValue < 1)
            return 2;
        BOOL restored = NO;
        int status = 0;
        @try {
            NSDictionary *ipv4 = CFBridgingRelease(
                SCDynamicStoreCopyValue(NULL, CFSTR("State:/Network/Global/IPv4")));
            NSString *primary = ipv4[@"PrimaryService"];
            requireOwned(primary.length > 0, @"primary_service");
            preferences = SCPreferencesCreate(NULL, CFSTR("xdzip owned proxy fixture"), NULL);
            requireOwned(preferences != NULL && SCPreferencesLock(preferences, FALSE),
                         @"preferences_lock");
            SCNetworkServiceRef service =
                SCNetworkServiceCopy(preferences, (__bridge CFStringRef)primary);
            requireOwned(service != NULL, @"service");
            proxyProtocol = SCNetworkServiceCopyProtocol(service, kSCNetworkProtocolTypeProxies);
            CFRelease(service);
            requireOwned(proxyProtocol != NULL, @"proxy_protocol");
            originalConfiguration =
                [(__bridge NSDictionary *)SCNetworkProtocolGetConfiguration(proxyProtocol) copy];
            originalEnabled = SCNetworkProtocolGetEnabled(proxyProtocol);
            originalGlobal = globalProxies();
            for (NSString *key in @[
                     @"HTTPEnable", @"HTTPSEnable", @"SOCKSEnable", @"ProxyAutoConfigEnable",
                     @"ProxyAutoDiscoveryEnable"
                 ])
                requireOwned(![originalConfiguration[key] boolValue] &&
                                 ![originalGlobal[key] boolValue],
                             @"existing_active_proxy");
            hostsFD = open("/etc/hosts", O_RDWR | O_NOFOLLOW);
            requireOwned(hostsFD >= 0 && fstat(hostsFD, &hostsIdentity) == 0 &&
                             S_ISREG(hostsIdentity.st_mode) && hostsIdentity.st_uid == 0 &&
                             hostsIdentity.st_size > 0 && hostsIdentity.st_size <= 64 * 1024,
                         @"hosts_open");
            NSMutableData *bytes = [NSMutableData dataWithLength:(NSUInteger)hostsIdentity.st_size];
            requireOwned(pread(hostsFD, bytes.mutableBytes, bytes.length, 0) ==
                             (ssize_t)bytes.length,
                         @"hosts_read");
            originalHosts = [bytes copy];
            NSString *text = [[NSString alloc] initWithData:bytes encoding:NSUTF8StringEncoding];
            requireOwned(
                text != nil &&
                    [text rangeOfString:fixtureHost options:NSCaseInsensitiveSearch].location ==
                        NSNotFound,
                @"existing_fixture_host");
            [bytes appendData:[[NSString
                                  stringWithFormat:@"\n127.0.0.1 %@ # xdzip owned proxy fixture\n",
                                                   fixtureHost]
                                  dataUsingEncoding:NSUTF8StringEncoding]];
            writeHosts(bytes);
            emit(@{@"state" : @"ready"});
            NSTimeInterval deadline = NSDate.timeIntervalSinceReferenceDate + 600;
            NSString *activeProfile = nil;
            while (NSDate.timeIntervalSinceReferenceDate < deadline) {
                fd_set readSet;
                FD_ZERO(&readSet);
                FD_SET(STDIN_FILENO, &readSet);
                struct timeval wait = {1, 0};
                int ready = select(STDIN_FILENO + 1, &readSet, NULL, NULL, &wait);
                requireOwned(ready >= 0, @"control_read");
                if (ready == 0)
                    continue;
                char buffer[32];
                if (fgets(buffer, sizeof(buffer), stdin) == NULL)
                    break; // Parent exit closes its pipe, including a killed parent.
                NSString *command = [@(buffer)
                    stringByTrimmingCharactersInSet:NSCharacterSet
                                                        .whitespaceAndNewlineCharacterSet];
                if ([command isEqual:@"restore"])
                    break;
                if ([command isEqual:@"verify"]) {
                    requireOwned(activeProfile != nil &&
                                     profileVisible(activeProfile, port, pac.absoluteString),
                                 @"profile_not_visible");
                    emit(@{@"state" : @"verified", @"profile" : activeProfile});
                    continue;
                }
                requireOwned([command isEqual:@"static"] || [command isEqual:@"pac"],
                             @"control_command");
                NSMutableDictionary *configuration = [@{
                    @"HTTPEnable" : @0,
                    @"HTTPSEnable" : @0,
                    @"SOCKSEnable" : @0,
                    @"ProxyAutoConfigEnable" : @0,
                    @"ProxyAutoDiscoveryEnable" : @0,
                    @"ExcludeSimpleHostnames" : @0,
                    @"ExceptionsList" :
                        @[ @"github.com", @"*.github.com", @"*.githubusercontent.com" ]
                } mutableCopy];
                if ([command isEqual:@"static"]) {
                    configuration[@"HTTPEnable"] = @1;
                    configuration[@"HTTPSEnable"] = @1;
                    configuration[@"HTTPProxy"] = configuration[@"HTTPSProxy"] = @"127.0.0.1";
                    configuration[@"HTTPPort"] = configuration[@"HTTPSPort"] = @(port);
                } else {
                    configuration[@"ProxyAutoConfigEnable"] = @1;
                    configuration[@"ProxyAutoConfigURLString"] = pac.absoluteString;
                }
                apply(configuration, YES);
                for (int attempt = 0;
                     attempt < 50 && !profileVisible(command, port, pac.absoluteString); attempt++)
                    usleep(100000);
                requireOwned(profileVisible(command, port, pac.absoluteString),
                             @"profile_not_visible");
                activeProfile = command;
                emit(@{@"state" : @"applied", @"profile" : command, @"verified" : @YES});
            }
        } @catch (NSException *error) {
            emit(@{@"state" : @"error", @"stage" : error.reason ?: @"fixture"});
            status = 1;
        } @finally {
            @try {
                if (preferencesChanged) {
                    apply(originalConfiguration, originalEnabled);
                    NSDictionary *actual =
                        (__bridge NSDictionary *)SCNetworkProtocolGetConfiguration(proxyProtocol);
                    requireOwned((actual == originalConfiguration ||
                                  [actual isEqual:originalConfiguration]) &&
                                     SCNetworkProtocolGetEnabled(proxyProtocol) == originalEnabled,
                                 @"proxy_restore");
                    for (int attempt = 0; attempt < 50 && !globalFlagsRestored(); attempt++)
                        usleep(100000);
                    requireOwned(globalFlagsRestored(), @"global_proxy_restore");
                }
                if (originalHosts != nil) {
                    writeHosts(originalHosts);
                    NSMutableData *actual = [NSMutableData dataWithLength:originalHosts.length];
                    struct stat end;
                    requireOwned(fstat(hostsFD, &end) == 0 && end.st_size == (off_t)actual.length &&
                                     pread(hostsFD, actual.mutableBytes, actual.length, 0) ==
                                         (ssize_t)actual.length &&
                                     [actual isEqual:originalHosts],
                                 @"hosts_restore");
                }
                restored = YES;
            } @catch (NSException *error) {
                emit(@{@"state" : @"error", @"stage" : error.reason ?: @"restore"});
                status = 1;
            }
            if (hostsFD >= 0)
                close(hostsFD);
            if (proxyProtocol)
                CFRelease(proxyProtocol);
            if (preferences) {
                SCPreferencesUnlock(preferences);
                CFRelease(preferences);
            }
            emit(@{@"state" : @"restored", @"verified" : @(restored)});
        }
        return status;
    }
}
