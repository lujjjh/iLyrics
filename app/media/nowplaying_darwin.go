package media

/*
#cgo CFLAGS: -x objective-c
#cgo LDFLAGS: -framework Cocoa -F/System/Library/PrivateFrameworks -framework MediaRemote

#import "NowPlaying.h"
*/
import "C"
import (
	"container/list"
	"context"
	"sync"
	"time"
)

var (
	currentNowPlayingInfo *NowPlayingInfo
	subscriptions         list.List
	mu                    sync.RWMutex
)

//export nowPlayingInfoNotificationCallback
func nowPlayingInfoNotificationCallback(nowPlayingInfoRef C.NowPlayingInfoRef) {
	var nowPlayingInfo *NowPlayingInfo
	if nowPlayingInfoRef != nil {
		defer C.FreeNowPlayingInfo(nowPlayingInfoRef)
		nowPlayingInfo = new(NowPlayingInfo)
		if p := nowPlayingInfoRef.title; p != nil {
			nowPlayingInfo.Title = C.GoString(p)
		}
		if p := nowPlayingInfoRef.artist; p != nil {
			nowPlayingInfo.Artist = C.GoString(p)
		}
		if p := nowPlayingInfoRef.album; p != nil {
			nowPlayingInfo.Album = C.GoString(p)
		}
		nowPlayingInfo.PlaybackState = PlaybackState(nowPlayingInfoRef.playbackState)
		nowPlayingInfo.ElapsedTime = time.Duration(nowPlayingInfoRef.elapsedTime) * time.Millisecond
		nowPlayingInfo.ITunesStoreID = uint64(nowPlayingInfoRef.iTunesStoreIdentifier)
		nowPlayingInfo.UpdatedAt = time.Unix(0, int64(nowPlayingInfoRef.updatedAt)*int64(time.Millisecond))
	}
	mu.Lock()
	currentNowPlayingInfo = nowPlayingInfo
	mu.Unlock()
	for element := subscriptions.Front(); element != nil; element = element.Next() {
		element.Value.(chan *NowPlayingInfo) <- nowPlayingInfo
	}
}

func init() {
	C.SetupPlayingInfoNotification()
}

func WatchNowPlayingInfo(ctx context.Context, immediate bool) <-chan *NowPlayingInfo {
	ch := make(chan *NowPlayingInfo, 1)
	go func() {
		mu.Lock()
		element := subscriptions.PushBack(ch)
		mu.Unlock()
		if immediate {
			mu.RLock()
			ch <- currentNowPlayingInfo
			mu.RUnlock()
		}
		<-ctx.Done()
		mu.Lock()
		subscriptions.Remove(element)
		mu.Unlock()
	}()
	return ch
}
