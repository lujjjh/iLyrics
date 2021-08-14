#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use std::ptr;

use app::LyricsWindow;
use bindings::Windows::Win32::System::Com::*;
use bindings::Windows::Win32::System::Diagnostics::Debug::*;
use bindings::Windows::Win32::System::Threading::*;
use bindings::Windows::Win32::UI::WindowsAndMessaging::*;

fn main() -> windows::Result<()> {
    unsafe {
        SetProcessDPIAware();
        CoInitialize(ptr::null_mut())?;

        if CreateMutexW(
            ptr::null_mut(),
            true,
            "0a0e643d-e484-486c-a4d6-1eef4cf6f499",
        )
        .is_null()
            || GetLastError() == ERROR_ALREADY_EXISTS
        {
            MessageBoxW(
                None,
                "iTunes Desktop Lyrics is already running.",
                "iTunes Desktop Lyrics",
                MB_ICONINFORMATION | MB_OK,
            );
            return Ok(());
        }
    }

    let app = &mut LyricsWindow::new()?;

    app.show()?;

    app.run_message_loop();

    Ok(())
}
