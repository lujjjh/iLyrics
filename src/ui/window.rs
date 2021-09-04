use std::ptr::null_mut;

use bindings::Windows::Win32::Foundation::*;
use bindings::Windows::Win32::System::LibraryLoader::*;
use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
use once_cell::sync::OnceCell;
use windows::*;

fn get_instance() -> HINSTANCE {
    static INSTANCE: OnceCell<HINSTANCE> = OnceCell::new();
    *INSTANCE.get_or_init(|| unsafe { GetModuleHandleW(None) })
}

pub trait Window: Sized {
    const CLASS_NAME: &'static str;
    const STYLE: WINDOW_STYLE;
    const EX_STYLE: WINDOW_EX_STYLE;

    fn get_hwnd(&self) -> HWND;
    fn window_proc(&mut self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;

    fn register_class() -> Result<()> {
        static RESULT: OnceCell<()> = OnceCell::new();
        RESULT
            .get_or_try_init(|| unsafe {
                let instance = get_instance();
                let class_name = HSTRING::from(Self::CLASS_NAME);
                let wc = WNDCLASSW {
                    style: CS_HREDRAW | CS_VREDRAW,
                    hCursor: LoadCursorW(None, IDI_APPLICATION),
                    hInstance: instance,
                    lpfnWndProc: Some(window_proc::<Self>),
                    lpszClassName: PWSTR(class_name.as_wide().as_ptr() as *mut _),
                    ..Default::default()
                };
                let atom = RegisterClassW(&wc);
                if atom == 0 {
                    Err(HRESULT::from_thread().into())
                } else {
                    Ok(())
                }
            })
            .map(|()| ())
    }

    fn create_window(title: &str, rect: &RECT, parent: Option<HWND>) -> Result<HWND> {
        Self::register_class()?;
        let hwnd = unsafe {
            CreateWindowExW(
                Self::EX_STYLE,
                Self::CLASS_NAME,
                title,
                Self::STYLE,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                parent,
                None,
                get_instance(),
                null_mut(),
            )
        };
        if hwnd == HWND(0) {
            Err(HRESULT::from_thread().into())
        } else {
            Ok(hwnd)
        }
    }

    fn show(&self, cmd: SHOW_WINDOW_CMD) -> Result<()> {
        unsafe {
            SetWindowLongPtrW(self.get_hwnd(), GWLP_USERDATA, self as *const _ as _);
            ShowWindow(self.get_hwnd(), cmd);
        }
        Ok(())
    }
}

pub unsafe fn get_window_instance<'a, T: Window>(hwnd: HWND) -> Option<&'a mut T> {
    let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    if window_ptr == 0 {
        None
    } else {
        Some(&mut *(window_ptr as *mut T))
    }
}

unsafe extern "system" fn window_proc<T: Window>(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let window: Option<&mut T> = get_window_instance(hwnd);
    match window {
        Some(window) => window.window_proc(hwnd, msg, wparam, lparam),
        None => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
