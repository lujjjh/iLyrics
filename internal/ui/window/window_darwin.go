package window

/*
#cgo CFLAGS: -x objective-c
#cgo LDFLAGS: -framework Cocoa
#import <Cocoa/Cocoa.h>
#import <AppKit/NSApplication.h>
#include <stdlib.h>

void SetIgnoreMouseEvents(void *w, bool ignore) {
	[[(NSView*)w window] setIgnoresMouseEvents:ignore];
}
*/
import "C"
import "unsafe"

func (w *Window) SetIgnoreMouseEvents(ignore bool) {
	C.SetIgnoreMouseEvents(unsafe.Pointer(w.sciterWindow.GetHwnd()), C.bool(ignore))
}
