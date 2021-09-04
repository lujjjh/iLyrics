use std::ptr::null;
use std::ptr::null_mut;
use std::time::Duration;

use bindings::Windows;
use bindings::Windows::Foundation::Numerics::*;
use bindings::Windows::Win32::Foundation::*;
use bindings::Windows::Win32::Graphics::Direct2D::*;
use bindings::Windows::Win32::Graphics::DirectComposition::*;
use bindings::Windows::Win32::Graphics::DirectWrite::*;
use bindings::Windows::Win32::Graphics::Dxgi::*;
use bindings::Windows::Win32::UI::Animation::*;
use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
use lrc::Lyrics;
use once_cell::sync::OnceCell;
use windows::*;

use crate::lyrics::Query;
use crate::player::itunes::ITunes;
use crate::player::Player;
use crate::player::PlayerState;
use crate::ui::get_window_instance;
use crate::ui::utils::*;
use crate::ui::Window;

const WINDOW_HEIGHT: i32 = 80;
const PADDING_HORIZONTAL: f64 = 10.;
const PADDING_VERTICAL: f64 = 5.;

struct Resources {
    d2d_factory: ID2D1Factory2,
    dc: ID2D1DeviceContext,
    _dcomp_device: IDCompositionDevice,
    _target: IDCompositionTarget,
    swap_chain: IDXGISwapChain1,
    dwrite_factory: IDWriteFactory2,
    brush: ID2D1SolidColorBrush,
    text_format: IDWriteTextFormat1,
    animation_manager: IUIAnimationManager,
    animation_timer: IUIAnimationTimer,
    animation_transition_library: IUIAnimationTransitionLibrary,
    bg_width: IUIAnimationVariable,
    bg_height: IUIAnimationVariable,
    line_vertical_offset: IUIAnimationVariable,
}

pub struct LyricsWindow {
    hwnd: HWND,
    resources: OnceCell<Resources>,
    player: ITunes,
    query: Query,
    lyrics: Option<Lyrics>,
    player_position: Option<Duration>,
    line_current: Option<String>,
    line_next: Option<String>,
}

impl Window for LyricsWindow {
    const CLASS_NAME: &'static str = "iLyrics";
    const STYLE: WINDOW_STYLE = WINDOW_STYLE(WS_CLIPSIBLINGS.0 | WS_CLIPCHILDREN.0 | WS_POPUP.0);
    const EX_STYLE: WINDOW_EX_STYLE = WINDOW_EX_STYLE(
        WS_EX_NOREDIRECTIONBITMAP.0
            | WS_EX_LAYERED.0
            | WS_EX_TRANSPARENT.0
            | WS_EX_TOPMOST.0
            | WS_EX_TOOLWINDOW.0,
    );

    fn get_hwnd(&self) -> HWND {
        self.hwnd
    }

    fn window_proc(&mut self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_TIMER => self.on_timer(hwnd, msg, wparam, lparam),
            WM_DESTROY => self.on_destroy(hwnd, msg, wparam, lparam),
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    }
}

impl LyricsWindow {
    pub fn new() -> Result<Self> {
        let (_scale_x, scale_y) = get_scale_factor()?;
        let mut rect = get_workarea_rect()?;
        rect.top = rect.bottom - (WINDOW_HEIGHT as f32 * scale_y).round() as i32;
        let hwnd = Self::create_window("iLyrics", &rect, None)?;
        let player = ITunes::new()?;
        let query = Query::new();
        Ok(Self {
            hwnd,
            resources: OnceCell::new(),
            player,
            query,
            lyrics: None,
            player_position: None,
            line_current: None,
            line_next: None,
        })
    }

    pub fn show(&mut self) -> Result<()> {
        Window::show(self, SW_SHOWNOACTIVATE)?;
        self.draw()?;
        self.set_lyrics_timer()?;
        Ok(())
    }

    fn set_lyrics_timer(&self) -> Result<()> {
        if unsafe { SetTimer(self.hwnd, 1, 100, None) } > 0 {
            Ok(())
        } else {
            Err(HRESULT::from_thread().into())
        }
    }

    fn on_timer(&mut self, _hwnd: HWND, _msg: u32, wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
        match wparam {
            WPARAM(1) => {
                let player_state = self.player.get_player_state();
                if let Some(PlayerState {
                    song_name,
                    song_artist,
                    ..
                }) = player_state.as_ref()
                {
                    let mut lyrics = None;
                    let changed = self
                        .query
                        .get_lyrics(song_name, song_artist, &mut lyrics)
                        .unwrap_or(false);
                    if changed {
                        self.set_lyrics(lyrics).unwrap();
                    }
                };
                let player_position = player_state.map(|player_state| player_state.player_position);
                self.set_player_position(player_position).unwrap();
                LRESULT(1)
            }
            _ => LRESULT(0),
        }
    }

    fn set_lyrics(&mut self, lyrics: Option<Lyrics>) -> Result<()> {
        self.lyrics = lyrics;
        self.set_player_position(None)
    }

    fn set_player_position(&mut self, player_position: Option<Duration>) -> Result<()> {
        self.player_position = player_position;
        self.update_lines()
    }

    fn update_lines(&mut self) -> Result<()> {
        if let Some(lyrics) = self.lyrics.as_ref() {
            if let Some(player_position) = self.player_position {
                let line_current = lyrics
                    .find_timed_line_index(player_position.as_millis() as i64)
                    .map(|index| lyrics.get_timed_lines()[index].1.to_string());
                let line_next = lyrics
                    .find_timed_line_index(player_position.as_millis() as i64 + 200)
                    .map(|index| lyrics.get_timed_lines()[index].1.to_string());
                if self.line_current != line_current {
                    self.line_current = line_current;
                }
                if self.line_next != line_next {
                    self.line_next = line_next;
                    self.update_bg_rectangle(self.line_next.as_ref())?;
                }
                return Ok(());
            }
        }
        self.line_next = None;
        self.update_bg_rectangle(None)?;
        Ok(())
    }

    fn update_bg_rectangle(&self, line_next: Option<&String>) -> Result<()> {
        let Resources {
            dc,
            animation_manager,
            animation_timer,
            animation_transition_library,
            bg_width,
            bg_height,
            line_vertical_offset,
            ..
        } = self.get_or_init_resources()?;
        match line_next {
            Some(line_next) => {
                let size = unsafe { dc.GetSize() };
                let metrics = self.get_text_metrics(line_next, size.width, size.height)?;
                unsafe {
                    let transition_bg_width = animation_transition_library
                        .CreateAccelerateDecelerateTransition(
                            0.2,
                            metrics.width as f64 + 2. * PADDING_HORIZONTAL,
                            0.5,
                            0.5,
                        )?;
                    let transition_bg_height = animation_transition_library
                        .CreateAccelerateDecelerateTransition(
                            0.2,
                            metrics.height as f64 + 2. * PADDING_VERTICAL,
                            0.5,
                            0.5,
                        )?;
                    let transition_line_vertical_offset = animation_transition_library
                        .CreateAccelerateDecelerateTransition(0.2, -size.height as f64, 0.5, 0.5)?;
                    transition_line_vertical_offset.SetInitialValue(0.)?;
                    let time_now = animation_timer.GetTime()?;
                    animation_manager.ScheduleTransition(
                        bg_width,
                        &transition_bg_width,
                        time_now,
                    )?;
                    animation_manager.ScheduleTransition(
                        bg_height,
                        &transition_bg_height,
                        time_now,
                    )?;
                    animation_manager.ScheduleTransition(
                        line_vertical_offset,
                        &transition_line_vertical_offset,
                        time_now,
                    )?;
                }
            }
            None => {}
        }
        Ok(())
    }

    fn on_destroy(&self, _hwnd: HWND, _msg: u32, _wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
        unsafe { PostQuitMessage(0) };
        LRESULT(1)
    }

    fn get_or_init_resources(&self) -> Result<&Resources> {
        self.resources.get_or_try_init(|| {
            let (dpi_x, dpi_y) = get_desktop_dpi()?;
            let d2d_factory = create_d2d_factory()?;
            let dxgi_device = create_dxgi_device()?;
            let _dcomp_device: IDCompositionDevice =
                unsafe { DCompositionCreateDevice(&dxgi_device) }?;
            let visual = unsafe { _dcomp_device.CreateVisual() }?;
            let dc = create_device_context(&dxgi_device, &d2d_factory, dpi_x, dpi_y)?;
            let swap_chain = create_swap_chain(self.hwnd, &dxgi_device)?;
            create_bitmap_from_swap_chain(&dc, &swap_chain, dpi_x, dpi_y)?;
            unsafe { visual.SetContent(&swap_chain) }?;
            let _target = unsafe { _dcomp_device.CreateTargetForHwnd(self.hwnd, BOOL(1)) }?;
            unsafe {
                _target.SetRoot(&visual)?;
                _dcomp_device.Commit()?;
            }
            let brush = unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F::default(), null()) }?;
            let dwrite_factory = create_dwrite_factory()?;
            let text_format: IDWriteTextFormat1 = unsafe {
                dwrite_factory
                    .CreateTextFormat(
                        "Segoe UI",
                        None,
                        DWRITE_FONT_WEIGHT_NORMAL,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        24.,
                        "",
                    )?
                    .cast()
            }?;
            {
                unsafe {
                    text_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
                    text_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
                }
                let font_fallback_builder = unsafe { dwrite_factory.CreateFontFallbackBuilder() }?;
                let ranges = DWRITE_UNICODE_RANGE {
                    first: 0x0,
                    last: 0xffffffff,
                };
                let fallback_family_names = [
                    HSTRING::from("Segoe UI Emoji"),
                    HSTRING::from("Segoe UI Symbol"),
                    HSTRING::from("Helvetica"),
                    HSTRING::from("Microsoft Yahei"),
                ];
                let fallback_family_names = fallback_family_names
                    .iter()
                    .map(|name| name.as_wide().as_ptr())
                    .collect::<Vec<*const u16>>();
                unsafe {
                    font_fallback_builder.AddMapping(
                        &ranges,
                        1,
                        fallback_family_names.as_ptr(),
                        fallback_family_names.len() as u32,
                        None,
                        None,
                        None,
                        1.,
                    )?;
                    font_fallback_builder.AddMappings(dwrite_factory.GetSystemFontFallback()?)?;
                    let font_fallback = font_fallback_builder.CreateFontFallback()?;
                    text_format.SetFontFallback(font_fallback)?;
                }
            }
            let animation_manager = create_animation_manager()?;
            let animation_timer = create_animation_timer()?;
            let animation_timer_handler: IUIAnimationTimerEventHandler =
                UIAnimationTimerEventHandler::new(self.hwnd).into();
            unsafe { animation_timer.SetTimerEventHandler(animation_timer_handler) }?;
            let timer_update_handler: IUIAnimationTimerUpdateHandler = animation_manager.cast()?;
            unsafe {
                animation_timer.SetTimerUpdateHandler(
                    &timer_update_handler,
                    UI_ANIMATION_IDLE_BEHAVIOR_DISABLE,
                )
            }?;
            let animation_transition_library = create_animation_transition_library()?;
            let bg_width = unsafe { animation_manager.CreateAnimationVariable(0.) }?;
            let bg_height = unsafe { animation_manager.CreateAnimationVariable(0.) }?;
            let line_vertical_offset = unsafe { animation_manager.CreateAnimationVariable(0.) }?;
            Ok(Resources {
                d2d_factory,
                dc,
                _dcomp_device,
                _target,
                swap_chain,
                brush,
                dwrite_factory,
                text_format,
                animation_manager,
                animation_timer,
                animation_transition_library,
                bg_width,
                bg_height,
                line_vertical_offset,
            })
        })
    }

    fn draw(&self) -> Result<()> {
        let Resources {
            d2d_factory,
            dc,
            swap_chain,
            brush,
            bg_width,
            bg_height,
            line_vertical_offset,
            ..
        } = self.get_or_init_resources()?;
        let Self {
            line_current,
            line_next,
            ..
        } = self;
        unsafe {
            dc.BeginDraw();
            dc.Clear(null_mut());
            brush.SetColor(&D2D1_COLOR_F {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 0.5,
            });
            let bg_width = bg_width.GetValue()? as f32;
            let bg_height = bg_height.GetValue()? as f32;
            let size = dc.GetSize();
            let left = (size.width - bg_width) / 2.;
            let top = (size.height - bg_height) / 2.;
            let right = left + bg_width;
            let bottom = top + bg_height;
            let bg_rounded_rect = &D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left,
                    top,
                    right,
                    bottom,
                },
                radiusX: 4.,
                radiusY: 4.,
            };
            let bg_geometry = d2d_factory.CreateRoundedRectangleGeometry(bg_rounded_rect)?;
            dc.FillGeometry(&bg_geometry, brush, None);
            dc.PushLayer(
                &D2D1_LAYER_PARAMETERS {
                    contentBounds: D2D_RECT_F {
                        left: -f32::INFINITY,
                        top: -f32::INFINITY,
                        right: f32::INFINITY,
                        bottom: f32::INFINITY,
                    },
                    geometricMask: Some(bg_geometry.into()),
                    maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
                    maskTransform: Matrix3x2::identity(),
                    opacity: 1.,
                    opacityBrush: None,
                    layerOptions: D2D1_LAYER_OPTIONS_NONE,
                },
                None,
            );
            let line_vertical_offset = line_vertical_offset.GetValue()? as f32;
            if let Some(line_current) = line_current {
                self.draw_text(
                    line_current,
                    &D2D_RECT_F {
                        left: 0.,
                        top: line_vertical_offset,
                        right: size.width,
                        bottom: line_vertical_offset + size.height,
                    },
                )?;
            }
            if let Some(line_next) = line_next {
                self.draw_text(
                    line_next,
                    &D2D_RECT_F {
                        left: 0.,
                        top: line_vertical_offset + size.height,
                        right: size.width,
                        bottom: line_vertical_offset + size.height * 2.,
                    },
                )?;
            }
            dc.PopLayer();
            dc.EndDraw(null_mut(), null_mut())?;
            swap_chain.Present(0, 0)?;
        }
        Ok(())
    }
    fn create_text_layout(
        &self,
        text: &str,
        max_width: f32,
        max_height: f32,
    ) -> Result<IDWriteTextLayout> {
        let Resources {
            dwrite_factory,
            text_format,
            ..
        } = self.get_or_init_resources()?;
        let string = HSTRING::from(text);
        unsafe {
            dwrite_factory.CreateTextLayout(
                PWSTR(string.as_wide().as_ptr() as *mut _),
                string.len() as u32,
                text_format,
                max_width,
                max_height,
            )
        }
    }

    fn draw_text(&self, text: &str, rect: &D2D_RECT_F) -> Result<()> {
        let Resources { dc, brush, .. } = self.get_or_init_resources()?;
        let text_layout =
            self.create_text_layout(text, rect.right - rect.left, rect.bottom - rect.top)?;
        unsafe {
            brush.SetColor(&D2D1_COLOR_F {
                r: 1.,
                g: 1.,
                b: 1.,
                a: 1.,
            });
            dc.DrawTextLayout(
                &D2D_POINT_2F {
                    x: rect.left,
                    y: rect.top,
                },
                &text_layout,
                brush,
                D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
            );
        }
        Ok(())
    }

    fn get_text_metrics(
        &self,
        text: &str,
        max_width: f32,
        max_height: f32,
    ) -> Result<DWRITE_TEXT_METRICS> {
        let text_layout = self.create_text_layout(text, max_width, max_height)?;
        unsafe { text_layout.GetMetrics() }
    }
}

#[implement(Windows::Win32::UI::Animation::IUIAnimationTimerEventHandler)]
struct UIAnimationTimerEventHandler {
    hwnd: HWND,
}

#[allow(non_snake_case)]
impl UIAnimationTimerEventHandler {
    fn new(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    fn OnPreUpdate(&self) -> HRESULT {
        S_OK
    }

    fn OnPostUpdate(&self) -> HRESULT {
        let window: &mut LyricsWindow = unsafe { get_window_instance(self.hwnd) }.unwrap();
        window.draw().unwrap();
        S_OK
    }

    fn OnRenderingTooSlow(&self, _fps: u32) -> HRESULT {
        S_OK
    }
}
