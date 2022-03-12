package window

import (
	"github.com/sciter-sdk/go-sciter"
	sciterwindow "github.com/sciter-sdk/go-sciter/window"
)

type Window struct {
	creationFlag sciter.WindowCreationFlag
	sciterWindow *sciterwindow.Window
}

func New(options ...NewOption) (*Window, error) {
	window := &Window{
		creationFlag: sciter.DefaultWindowCreateFlag &^ sciter.SW_MAIN,
	}
	for _, f := range options {
		f(window)
	}
	sciterWindow, err := sciterwindow.New(window.creationFlag, nil)
	if err != nil {
		return nil, err
	}
	window.sciterWindow = sciterWindow
	return window, nil
}

func (w *Window) Load(filename string) error {
	return w.sciterWindow.LoadFile(filename)
}

func (w *Window) Show() {
	w.sciterWindow.Show()
}

func (w *Window) RunLoop() {
	w.sciterWindow.Run()
}

func (w *Window) Sciter() *sciter.Sciter {
	return w.sciterWindow.Sciter
}

type NewOption func(*Window)

func createCreationFlagOption(flag sciter.WindowCreationFlag, set bool) NewOption {
	return func(window *Window) {
		if set {
			window.creationFlag |= flag
		} else {
			window.creationFlag &^= flag
		}
	}
}

func WithMain(value bool) NewOption {
	return createCreationFlagOption(sciter.SW_MAIN, value)
}

func WithTool(value bool) NewOption {
	return createCreationFlagOption(sciter.SW_TOOL, value)
}
