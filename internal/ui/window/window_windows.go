package window

/*
#include <windows.h>
#include <stdbool.h>

void SetIgnoreMouseEvents(void *w, bool ignore) {
	HWND wnd = (HWND)w;
	LONG exStyle = GetWindowLong(wnd, GWL_EXSTYLE);
	exStyle &= ~WS_EX_TRANSPARENT;
	if (ignore) {
		exStyle |= WS_EX_TRANSPARENT;
	}
	SetWindowLong(wnd, GWL_EXSTYLE, exStyle);
}
*/
import "C"
import (
	"syscall"
	"unsafe"
)

func init() {
	syscall.NewLazyDLL("Shcore.dll").NewProc("SetProcessDpiAwareness").Call(uintptr(2))
}

func (w *Window) SetIgnoreMouseEvents(ignore bool) {
	C.SetIgnoreMouseEvents(unsafe.Pointer(w.sciterWindow.GetHwnd()), C.bool(ignore))
}
