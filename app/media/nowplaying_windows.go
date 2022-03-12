package media

import "context"

func WatchNowPlayingInfo(ctx context.Context, immediate bool) <-chan *NowPlayingInfo {
	ch := make(chan *NowPlayingInfo, 1)
	// go func() {
	// 	mu.Lock()
	// 	element := subscriptions.PushBack(ch)
	// 	mu.Unlock()
	// 	if immediate {
	// 		mu.RLock()
	// 		ch <- currentNowPlayingInfo
	// 		mu.RUnlock()
	// 	}
	// 	<-ctx.Done()
	// 	mu.Lock()
	// 	subscriptions.Remove(element)
	// 	mu.Unlock()
	// }()
	return ch
}
