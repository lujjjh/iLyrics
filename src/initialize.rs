use std::process::exit;
use std::ptr::null_mut;

use bindings::Windows::Win32::System::Com::*;
use bindings::Windows::Win32::System::Diagnostics::Debug::*;
use bindings::Windows::Win32::System::Threading::*;
use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
use windows::*;

#[cfg(debug_assertions)]
const UUID: &str = "c727e9a0-c71a-4001-ad0b-be20fd8e7971";

#[cfg(not(debug_assertions))]
const UUID: &str = "0a0e643d-e484-486c-a4d6-1eef4cf6f499";

pub fn initialize() -> Result<()> {
    unsafe {
        SetProcessDPIAware();
        CoInitialize(null_mut())?;

        let mutex_handle = CreateMutexW(null_mut(), true, UUID);
        if mutex_handle.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            MessageBoxW(
                None,
                "iLyrics is already running.",
                "iLyrics",
                MB_ICONINFORMATION | MB_OK,
            );
            exit(1);
        }

        Ok(())
    }
}
