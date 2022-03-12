package app

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"sync"
	"time"

	"github.com/golang/groupcache/lru"
	"golang.org/x/sync/singleflight"

	"github.com/lujjjh/ilyrics/app/lrc"
	"github.com/lujjjh/ilyrics/app/media"
)

const (
	defaultLyricsAPIEntry  = "https://lyrics-api.lujjjh.com/"
	defaultMaxCachedLyrics = 32
)

var (
	ErrLyricsNotFound = errors.New("lyrics not found")
)

type LyricsStore struct {
	endpoint *url.URL
	client   *http.Client
	group    singleflight.Group
	cache    *lru.Cache
	cacheMu  sync.RWMutex
}

func NewLyricsStore() (*LyricsStore, error) {
	endpoint, err := url.Parse(defaultLyricsAPIEntry)
	if err != nil {
		return nil, err
	}
	return &LyricsStore{
		endpoint: endpoint,
		client: &http.Client{
			Transport: &http.Transport{
				IdleConnTimeout: 5 * time.Minute,
			},
		},
		cache: lru.New(defaultMaxCachedLyrics),
	}, nil
}

func (l *LyricsStore) QueryByNowPlayingInfo(ctx context.Context, nowPlayingInfo *media.NowPlayingInfo) (*lrc.Lyrics, error) {
	if nowPlayingInfo == nil || nowPlayingInfo.Title == "" || nowPlayingInfo.Artist == "" {
		return nil, ErrLyricsNotFound
	}
	key := l.keyOfNowPlayingInfo(nowPlayingInfo)
	result, err, _ := l.group.Do(key, func() (interface{}, error) {
		if lyrics, err, ok := l.getCachedResult(key); ok {
			return lyrics, err
		}
		u := *l.endpoint
		q := u.Query()
		q.Set("name", nowPlayingInfo.Title)
		q.Set("artist", nowPlayingInfo.Artist)
		u.RawQuery = q.Encode()
		req, err := http.NewRequest(http.MethodGet, u.String(), nil)
		if err != nil {
			return nil, err
		}
		req = req.WithContext(ctx)
		resp, err := l.client.Do(req)
		if err != nil {
			return nil, err
		}
		defer resp.Body.Close()
		var lyrics *lrc.Lyrics
		switch resp.StatusCode {
		case 200:
			lyrics, err = lrc.Parse(resp.Body)
			if err != nil {
				return nil, err
			}
		case 404:
			// cacheable
			err = ErrLyricsNotFound
		default:
			return nil, fmt.Errorf("unexpected status code: %d", resp.StatusCode)
		}
		l.putCachedResult(key, lyrics, err)
		return lyrics, err
	})
	if err != nil {
		return nil, err
	}
	return result.(*lrc.Lyrics), nil
}

func (l *LyricsStore) keyOfNowPlayingInfo(nowPlayingInfo *media.NowPlayingInfo) string {
	return fmt.Sprintf("%s:%s:%s:%d", nowPlayingInfo.Title, nowPlayingInfo.Artist, nowPlayingInfo.Album, nowPlayingInfo.ITunesStoreID)
}

func (l *LyricsStore) getCachedResult(key string) (*lrc.Lyrics, error, bool) {
	l.cacheMu.RLock()
	defer l.cacheMu.RUnlock()
	if result, ok := l.cache.Get(key); ok {
		switch result := result.(type) {
		case *lrc.Lyrics:
			return result, nil, true
		case error:
			return nil, result, true
		default:
			panic("unreachable")
		}
	}
	return nil, nil, false
}

func (l *LyricsStore) putCachedResult(key string, lrc *lrc.Lyrics, err error) {
	l.cacheMu.Lock()
	defer l.cacheMu.Unlock()
	if err != nil {
		l.cache.Add(key, err)
	} else {
		l.cache.Add(key, lrc)
	}
}
