#import "MediaRemoteSPI.h"
#import "NowPlaying.h"
#import <Foundation/Foundation.h>
#import <stdlib.h>

static bool playingInfoNotificationInitialized = false;

static void CopyMaybeNSStringRefToCString(id maybeAString, const char **out) {
  if (!maybeAString) {
    return;
  }
  if (![maybeAString isKindOfClass:[NSString class]]) {
    return;
  }
  *out = strdup([(__bridge NSString *)maybeAString UTF8String]);
}

static void UpdateNowPlayingInfo() {
  dispatch_queue_t queue = dispatch_queue_create(nil, nil);
  MRMediaRemoteGetNowPlayingInfo(queue, ^(CFDictionaryRef information) {
    NSDictionary *info = (NSDictionary *)information;
    if (!info) {
      nowPlayingInfoNotificationCallback(nil);
      return;
    }
    NowPlayingInfoRef item = malloc(sizeof(NowPlayingInfo));
    memset(item, 0, sizeof(NowPlayingInfo));
    CopyMaybeNSStringRefToCString(
        info[(__bridge NSString *)kMRMediaRemoteNowPlayingInfoTitle],
        &item->title);
    CopyMaybeNSStringRefToCString(
        info[(__bridge NSString *)kMRMediaRemoteNowPlayingInfoArtist],
        &item->artist);
    CopyMaybeNSStringRefToCString(
        info[(__bridge NSString *)kMRMediaRemoteNowPlayingInfoAlbum],
        &item->album);
    id playbackRate =
        info[(__bridge NSString *)kMRMediaRemoteNowPlayingInfoPlaybackRate];
    if (playbackRate && [playbackRate isKindOfClass:[NSNumber class]]) {
      item->playbackState =
          [(__bridge NSNumber *)playbackRate intValue] ? Playing : Paused;
    }
    id elapsedTime =
        info[(__bridge NSString *)kMRMediaRemoteNowPlayingInfoElapsedTime];
    if (!elapsedTime) {
      elapsedTime = 0;
    }
    if (elapsedTime && [elapsedTime isKindOfClass:[NSNumber class]]) {
      item->elapsedTime =
          (int)([(__bridge NSNumber *)elapsedTime doubleValue] * 1000);
    }
    id iTunesStoreIdentifier =
        info[@"kMRMediaRemoteNowPlayingInfoiTunesStoreIdentifier"];
    if (iTunesStoreIdentifier &&
        [iTunesStoreIdentifier isKindOfClass:[NSNumber class]]) {
      item->iTunesStoreIdentifier = (uint64_t)[(
          __bridge NSNumber *)iTunesStoreIdentifier unsignedLongLongValue];
    }
    id timestamp =
        info[(__bridge NSString *)kMRMediaRemoteNowPlayingInfoTimestamp];
    if (timestamp && [timestamp isKindOfClass:[NSDate class]]) {
      item->updatedAt =
          (uint64_t)([(NSDate *)timestamp timeIntervalSince1970] * 1000);
    }
    nowPlayingInfoNotificationCallback(item);
  });
}

void SetupPlayingInfoNotification() {
  if (playingInfoNotificationInitialized) {
    return;
  }
  [NSNotificationCenter.defaultCenter
      addObserverForName:(__bridge NSString *)
                             kMRMediaRemoteNowPlayingInfoDidChangeNotification
                  object:nil
                   queue:nil
              usingBlock:^(NSNotification *_notification) {
                UpdateNowPlayingInfo();
              }];
  dispatch_queue_t queue = dispatch_queue_create(nil, nil);
  MRMediaRemoteRegisterForNowPlayingNotifications(queue);
  UpdateNowPlayingInfo();
  playingInfoNotificationInitialized = true;
}

void FreeNowPlayingInfo(NowPlayingInfoRef nowPlayingInfo) {
  if (nowPlayingInfo->title) {
    free((void *)nowPlayingInfo->title);
  }
  if (nowPlayingInfo->artist) {
    free((void *)nowPlayingInfo->artist);
  }
  if (nowPlayingInfo->album) {
    free((void *)nowPlayingInfo->album);
  }
  free(nowPlayingInfo);
}
