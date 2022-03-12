package app

import (
	"context"
	"sync"
	"time"

	"github.com/lujjjh/ilyrics/app/lrc"
	"github.com/lujjjh/ilyrics/app/media"
	"github.com/lujjjh/ilyrics/app/ui/window/lyrics"
)

type App struct {
	lyricsWindow      *lyrics.Window
	lyricsStore       *LyricsStore
	nowPlayingInfo    *media.NowPlayingInfo
	nowPlayingInfoMu  sync.RWMutex
	currentLyrics     *lrc.Lyrics
	currentLyricsMu   sync.RWMutex
	currentLyricsLine lrc.LyricsLine
}

func New() (*App, error) {
	lyricsWindow, err := lyrics.NewWindow()
	if err != nil {
		return nil, err
	}
	lyricsStore, err := NewLyricsStore()
	if err != nil {
		return nil, err
	}
	app := &App{
		lyricsWindow: lyricsWindow,
		lyricsStore:  lyricsStore,
	}
	go app.watchNowPlayingInfo()
	go app.updateLyricsWorker()
	return app, nil
}

func (a *App) Run() {
	a.lyricsWindow.Show()
	a.lyricsWindow.RunLoop()
}

func (a *App) watchNowPlayingInfo() {
	for nowPlayingInfo := range media.WatchNowPlayingInfo(context.Background(), true) {
		a.updateNowPlayingInfo(nowPlayingInfo)
	}
}

func (a *App) updateNowPlayingInfo(nowPlayingInfo *media.NowPlayingInfo) {
	// TODO: seq
	a.nowPlayingInfoMu.Lock()
	a.nowPlayingInfo = nowPlayingInfo
	a.nowPlayingInfoMu.Unlock()
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	lyrics, err := a.lyricsStore.QueryByNowPlayingInfo(ctx, nowPlayingInfo)
	if err != nil {
		a.updateCurrentLyrics(nil)
		return
	}
	a.updateCurrentLyrics(lyrics)
}

func (a *App) updateCurrentLyrics(lyrics *lrc.Lyrics) {
	a.currentLyricsMu.Lock()
	defer a.currentLyricsMu.Unlock()
	a.currentLyrics = lyrics
}

func (a *App) updateLyricsWorker() {
	// TODO: Actively update after updateNowPlayingInfo().
	for {
		time.Sleep(100 * time.Millisecond)
		var lyricsLine lrc.LyricsLine
		if a.nowPlayingInfo != nil && a.nowPlayingInfo.PlaybackState == media.PlaybackStatePlaying && a.currentLyrics != nil {
			// HACK: +500ms for animation.
			lyricsLine = a.currentLyrics.Line(a.nowPlayingInfo.PlaybackPosition() + 350*time.Millisecond)
		}
		if lyricsLine == a.currentLyricsLine {
			continue
		}
		a.lyricsWindow.SetLyrics(lyricsLine.Text)
		a.currentLyricsLine = lyricsLine
	}
}
