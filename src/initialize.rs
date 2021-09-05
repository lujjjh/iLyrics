use std::path::Path;
use std::process::exit;
use std::ptr::null_mut;

use bindings::Windows::Win32::System::Com::*;
use bindings::Windows::Win32::System::Diagnostics::Debug::*;
use bindings::Windows::Win32::System::Threading::*;
use bindings::Windows::Win32::UI::Shell::*;
use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
use flexi_logger::detailed_format;
use flexi_logger::Cleanup;
use flexi_logger::Criterion;
use flexi_logger::FileSpec;
use flexi_logger::Logger;
use flexi_logger::Naming;
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

        let roaming_app_data_path = known_folder_path(&FOLDERID_RoamingAppData)?;
        let log_directory = Path::new(&roaming_app_data_path)
            .join("iLyrics")
            .join("logs");
        Logger::try_with_str("info")
            .unwrap()
            .log_to_file(
                FileSpec::default()
                    .directory(log_directory)
                    .basename("iLyrics"),
            )
            .format(detailed_format)
            .rotate(
                Criterion::Size(5 * 1_024 * 1_024),
                Naming::Timestamps,
                Cleanup::KeepLogFiles(0),
            )
            .start()
            .unwrap();

        Ok(())
    }
}

fn known_folder_path(id: &windows::Guid) -> Result<String> {
    unsafe {
        let path = SHGetKnownFolderPath(id, 0, None)?;
        if path.0.is_null() {
            return Ok(String::new());
        }
        let mut end = path.0;
        while *end != 0 {
            end = end.add(1);
        }
        let result = String::from_utf16_lossy(std::slice::from_raw_parts(
            path.0,
            end.offset_from(path.0) as _,
        ));
        CoTaskMemFree(path.0 as _);
        Ok(result)
    }
}
