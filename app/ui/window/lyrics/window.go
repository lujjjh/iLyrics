package lyrics

import (
	"path/filepath"

	"github.com/lujjjh/ilyrics/internal/ui/window"
	"github.com/sciter-sdk/go-sciter"
)

type Window struct {
	*window.Window
}

func NewWindow() (*Window, error) {
	w, err := window.New(window.WithMain(true), window.WithTool(true))
	if err != nil {
		return nil, err
	}
	w.SetIgnoreMouseEvents(true)
	filename, _ := filepath.Abs("html/build/lyrics.html")
	if err := w.Load(filename); err != nil {
		return nil, err
	}
	return &Window{w}, nil
}

func (w *Window) SetLyrics(text string) {
	w.Sciter().Call("setLyrics", sciter.NewValue(text))
}
