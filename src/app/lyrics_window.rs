use std::mem;
use std::ptr;
use std::time::Duration;
use std::time::SystemTime;

use bindings::Windows::Win32::Foundation::*;
use bindings::Windows::Win32::Graphics::Direct2D::*;
use bindings::Windows::Win32::Graphics::Direct3D11::*;
use bindings::Windows::Win32::Graphics::DirectComposition::*;
use bindings::Windows::Win32::Graphics::DirectWrite::*;
use bindings::Windows::Win32::Graphics::Dxgi::*;
use bindings::Windows::Win32::System::Com::*;
use bindings::Windows::Win32::System::LibraryLoader::*;
use bindings::Windows::Win32::System::Threading::*;
use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
use utf16_lit::utf16_null;
use windows::Abi;
use windows::Interface;

use super::itunes::ITunes;
use super::itunes::TrackInfo;
use super::lyrics::Lyrics;

pub struct LyricsWindow {
    hwnd: HWND,
    d2d_factory: ID2D1Factory2,
    dwrite_factory: IDWriteFactory2,
    context: ID2D1DeviceContext,
    swap_chain: IDXGISwapChain1,
    _target: IDCompositionTarget,
    itunes: ITunes,
    lyrics: Lyrics,
    last_text_to_render: Option<String>,
    last_player_position: Option<Duration>,
    last_updated_at: SystemTime,
}

const CLASS_NAME: PWSTR = PWSTR(utf16_null!("iTunesMate").as_ptr() as *mut u16);

impl LyricsWindow {
    pub fn new() -> windows::Result<Self> {
        let instance = unsafe { GetModuleHandleW(PWSTR::NULL) };
        Self::register_class(instance);

        let device = Self::create_dxgi_device()?;
        let d2d_factory = Self::create_factory()?;
        let context = Self::create_device_context(&device, &d2d_factory)?;
        let (mut dpi_x, mut dpi_y) = (0., 0.);
        unsafe { d2d_factory.GetDesktopDpi(&mut dpi_x, &mut dpi_y) };
        let hwnd = Self::create_window(instance, dpi_x, dpi_y);
        let swap_chain = Self::create_swap_chain(hwnd, &device)?;
        Self::create_bitmap(&d2d_factory, &context, &swap_chain)?;
        let target = Self::create_composition(hwnd, &device, &swap_chain)?;
        let dwrite_factory = Self::create_dwrite_factory()?;

        Ok(Self {
            hwnd,
            d2d_factory,
            context,
            swap_chain,
            _target: target,
            dwrite_factory,
            itunes: ITunes::new()?,
            lyrics: Lyrics::new(),
            last_text_to_render: None,
            last_player_position: None,
            last_updated_at: SystemTime::now(),
        })
    }

    fn register_class(instance: HINSTANCE) {
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            hCursor: unsafe { LoadCursorW(None, IDI_APPLICATION) },
            hInstance: instance,
            lpfnWndProc: Some(Self::window_proc),
            lpszClassName: CLASS_NAME,
            ..Default::default()
        };

        unsafe { RegisterClassW(&wc) };
    }

    fn create_window(instance: HINSTANCE, _dpi_x: f32, dpi_y: f32) -> HWND {
        let scale_y = dpi_y / 96.;
        let height = (80. * scale_y) as i32;
        unsafe {
            let mut rect: RECT = mem::zeroed();
            SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut rect as *mut _ as _, 0.into());
            CreateWindowExW(
                WS_EX_TOPMOST
                    | WS_EX_NOREDIRECTIONBITMAP
                    | WS_EX_TRANSPARENT
                    | WS_EX_LAYERED
                    | WS_EX_TOOLWINDOW,
                CLASS_NAME,
                "iTunes Mate Lyrics",
                WS_CLIPSIBLINGS | WS_CLIPCHILDREN | WS_POPUP,
                rect.left,
                rect.bottom - height,
                rect.right - rect.left,
                height,
                None,
                None,
                instance,
                ptr::null_mut(),
            )
        }
    }

    fn create_dxgi_device() -> windows::Result<IDXGIDevice> {
        unsafe {
            let mut direct3d_device = None;
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                ptr::null(),
                0,
                D3D11_SDK_VERSION,
                &mut direct3d_device,
                ptr::null_mut(),
                ptr::null_mut(),
            )?;
            let direct3d_device = direct3d_device.unwrap();
            let dxgi_device = direct3d_device.cast::<IDXGIDevice>()?;
            Ok(dxgi_device)
        }
    }

    fn create_factory() -> windows::Result<ID2D1Factory2> {
        unsafe {
            let mut d2d_factory: Option<ID2D1Factory2> = None;
            D2D1CreateFactory(
                D2D1_FACTORY_TYPE_SINGLE_THREADED,
                &ID2D1Factory::IID,
                &D2D1_FACTORY_OPTIONS {
                    debugLevel: D2D1_DEBUG_LEVEL(0),
                },
                d2d_factory.set_abi(),
            )?;
            Ok(d2d_factory.unwrap())
        }
    }

    fn create_device_context(
        dxgi_device: &IDXGIDevice,
        factory: &ID2D1Factory2,
    ) -> windows::Result<ID2D1DeviceContext> {
        unsafe {
            let d2d_device = factory.CreateDevice(dxgi_device)?;
            let dc = d2d_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
            let mut dpi_x = 0.;
            let mut dpi_y = 0.;
            factory.GetDesktopDpi(&mut dpi_x, &mut dpi_y);
            dc.SetDpi(dpi_x, dpi_y);
            Ok(dc)
        }
    }

    fn create_swap_chain(hwnd: HWND, device: &IDXGIDevice) -> windows::Result<IDXGISwapChain1> {
        unsafe {
            let dxgi_factory = CreateDXGIFactory2::<IDXGIFactory2>(0)?;
            let mut rect = mem::zeroed();
            GetClientRect(hwnd, &mut rect);

            let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: (rect.right - rect.left) as u32,
                Height: (rect.bottom - rect.top) as u32,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                Stereo: BOOL(0),
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                Scaling: DXGI_SCALING_STRETCH,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
                Flags: 0,
            };

            dxgi_factory.CreateSwapChainForComposition(device, &swap_chain_desc, None)
        }
    }

    fn create_bitmap(
        factory: &ID2D1Factory2,
        dc: &ID2D1DeviceContext,
        swap_chain: &IDXGISwapChain1,
    ) -> windows::Result<ID2D1Bitmap1> {
        unsafe {
            let dxgi_buffer = swap_chain.GetBuffer::<IDXGISurface2>(0)?;

            let mut dpi_x = 0.;
            let mut dpi_y = 0.;
            factory.GetDesktopDpi(&mut dpi_x, &mut dpi_y);

            let properties = D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: dpi_x,
                dpiY: dpi_y,
                bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                colorContext: None,
            };

            let bitmap = dc.CreateBitmapFromDxgiSurface(&dxgi_buffer, &properties)?;

            dc.SetTarget(&bitmap);

            Ok(bitmap)
        }
    }

    fn create_composition(
        hwnd: HWND,
        device: &IDXGIDevice,
        swap_chain: &IDXGISwapChain1,
    ) -> windows::Result<IDCompositionTarget> {
        unsafe {
            let dcomp_device: IDCompositionDevice = DCompositionCreateDevice(device)?;
            let target = dcomp_device.CreateTargetForHwnd(hwnd, BOOL(1))?;
            let visual = dcomp_device.CreateVisual()?;
            visual.SetContent(swap_chain)?;
            target.SetRoot(&visual)?;
            dcomp_device.Commit()?;

            Ok(target)
        }
    }

    fn create_dwrite_factory() -> windows::Result<IDWriteFactory2> {
        unsafe {
            DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory2::IID)
                .unwrap()
                .cast::<IDWriteFactory2>()
        }
    }

    pub fn show(&mut self) -> windows::Result<()> {
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, self as *const _ as _);
            ShowWindow(self.hwnd, SW_SHOW);
            SendMessageW(self.hwnd, WM_PAINT, WPARAM(0), LPARAM(0));
        }

        Ok(())
    }

    pub fn run_message_loop(&mut self) {
        unsafe {
            let mut msg = MSG::default();
            let draw_interval = Duration::from_millis(50);
            let mut last_drawn_at = SystemTime::now() - draw_interval;
            loop {
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                    if msg.message == WM_QUIT {
                        return;
                    }
                }

                let last_drawn_at_elapsed = last_drawn_at.elapsed().unwrap();
                let next_draw_duration;
                if last_drawn_at_elapsed >= draw_interval {
                    self.draw().unwrap();
                    last_drawn_at = SystemTime::now();
                    next_draw_duration = draw_interval;
                } else {
                    next_draw_duration = draw_interval - last_drawn_at_elapsed;
                }
                MsgWaitForMultipleObjectsEx(
                    0,
                    ptr::null(),
                    next_draw_duration.as_millis() as u32,
                    QS_ALLEVENTS,
                    MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX_FLAGS(0),
                );
            }
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let app_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if app_ptr == 0 {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        let app = &mut *(app_ptr as *mut Self);
        app.handle_message(hwnd, msg, wparam, lparam)
    }

    unsafe fn handle_message(
        &mut self,
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_SIZE => self.handle_size(hwnd, msg, wparam, lparam),
            WM_DESTROY => self.handle_destroy(hwnd, msg, wparam, lparam),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    fn draw(&mut self) -> windows::Result<()> {
        unsafe {
            let dc = &self.context;
            let dwrite_factory = &self.dwrite_factory;
            let itunes = &self.itunes;
            let lyrics = &mut self.lyrics;

            if !itunes.check_if_alive() {
                PostQuitMessage(0);
                return Ok(());
            }

            // We should conduct the interpolation because iTunes only provides precision to seconds.
            let mut player_position = itunes.get_player_position();
            if player_position != self.last_player_position {
                self.last_player_position = player_position;
                self.last_updated_at = SystemTime::now();
            } else if let Some(player_position) = player_position.as_mut() {
                *player_position += self.last_updated_at.elapsed().unwrap();
            }

            let text_to_render = itunes
                .get_current_track_info()
                .map(|TrackInfo { name, artist }| format!("{} - {}", name, artist))
                .and_then(|query| {
                    player_position.and_then(|duration| {
                        lyrics.get_lyrics_line(&query, duration).unwrap_or_default()
                    })
                })
                .map(|s| html_escape::decode_html_entities(&s).to_string())
                .map(|s| s.trim().to_string());

            if text_to_render == self.last_text_to_render {
                return Ok(());
            }
            self.last_text_to_render = text_to_render.clone();

            dc.BeginDraw();

            dc.Clear(&D2D1_COLOR_F {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 0.,
            });

            if let Some(text_to_render) = text_to_render {
                if !text_to_render.is_empty() {
                    let text_to_render = windows::HSTRING::from(text_to_render);

                    let text_format = dwrite_factory.CreateTextFormat(
                        "Microsoft Yahei",
                        None,
                        DWRITE_FONT_WEIGHT_NORMAL,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        24.,
                        "",
                    )?;
                    text_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
                    text_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;

                    let D2D_SIZE_F { width, height } = dc.GetSize();
                    let text_layout = dwrite_factory.CreateTextLayout(
                        PWSTR(text_to_render.as_wide().as_ptr() as *mut _),
                        text_to_render.len() as u32,
                        &text_format,
                        width,
                        height,
                    )?;

                    let bg_brush = dc.CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            r: 0.,
                            g: 0.,
                            b: 0.,
                            a: 0.5,
                        },
                        ptr::null(),
                    )?;

                    let fg_brush = dc.CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            r: 1.,
                            g: 1.,
                            b: 1.,
                            a: 1.,
                        },
                        ptr::null(),
                    )?;

                    let metrics = text_layout.GetMetrics()?;
                    let (padding_horizontal, padding_vertical) = (10., 5.);
                    dc.FillRectangle(
                        &D2D_RECT_F {
                            left: metrics.left - padding_horizontal,
                            top: metrics.top - padding_vertical,
                            right: metrics.left + metrics.width + padding_horizontal,
                            bottom: metrics.top + metrics.height + padding_vertical,
                        },
                        &bg_brush,
                    );

                    dc.DrawTextLayout(
                        D2D_POINT_2F { x: 0., y: 0. },
                        &text_layout,
                        &fg_brush,
                        D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                    );
                }
            }

            dc.EndDraw(ptr::null_mut(), ptr::null_mut())?;

            self.swap_chain.Present(0, 0).unwrap();
        }

        Ok(())
    }

    fn handle_size(&mut self, _hwnd: HWND, _msg: u32, _wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let width = (lparam.0 as u32) & 0xFFFF;
        let height = (lparam.0 as u32) >> 16;
        unsafe {
            self.context.SetTarget(None);
            self.swap_chain
                .ResizeBuffers(2, width, height, DXGI_FORMAT_B8G8R8A8_UNORM, 0)
                .unwrap();
            Self::create_bitmap(&self.d2d_factory, &self.context, &self.swap_chain).unwrap();
        }
        LRESULT(0)
    }

    fn handle_destroy(&self, _hwnd: HWND, _msg: u32, _wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
        unsafe { PostQuitMessage(0) };
        LRESULT(1)
    }
}

impl Drop for LyricsWindow {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
