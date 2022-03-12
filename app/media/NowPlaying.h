#ifndef NOW_PLAYING_H_
#define NOW_PLAYING_H_

#include <stdlib.h>
#include <stdint.h>

typedef enum PlaybackState { Paused, Playing } PlaybackState;

typedef struct NowPlayingInfo {
  const char *title;
  const char *artist;
  const char *album;
  PlaybackState playbackState;
  int elapsedTime;
  uint64_t iTunesStoreIdentifier;
  uint64_t updatedAt;
} NowPlayingInfo, *NowPlayingInfoRef;

void SetupPlayingInfoNotification(void);
void FreeNowPlayingInfo(NowPlayingInfoRef);

// Go callbacks
extern void nowPlayingInfoNotificationCallback(NowPlayingInfoRef);

#endif
