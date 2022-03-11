package media

// #include <NowPlaying.h>
import "C"
import "time"

type NowPlayingInfo struct {
	Title         string
	Artist        string
	Album         string
	PlaybackState PlaybackState
	ElapsedTime   time.Duration
	ITunesStoreID uint64
	UpdatedAt     time.Time
}

type PlaybackState int

var (
	PlaybackStatePaused  PlaybackState = C.Paused
	PlaybackStatePlaying PlaybackState = C.Playing
)

func (n *NowPlayingInfo) PlaybackPosition() time.Duration {
	if n.PlaybackState == PlaybackStatePaused {
		return n.ElapsedTime
	}
	return n.ElapsedTime + time.Since(n.UpdatedAt)
}
