use std::mem::zeroed;
use std::ptr::null;
use std::ptr::null_mut;

use bindings::Windows::Win32::Foundation::*;
use bindings::Windows::Win32::Graphics::Direct2D::*;
use bindings::Windows::Win32::Graphics::Direct3D11::*;
use bindings::Windows::Win32::Graphics::DirectComposition::*;
use bindings::Windows::Win32::Graphics::DirectWrite::*;
use bindings::Windows::Win32::Graphics::Dxgi::*;
use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
use windows::*;

use crate::types::Rect;
use crate::types::Size;

use super::element::Render;
use super::state::LyricsWindowState;

const MARGIN_HORIZONTAL: f32 = 10.;
const MARGIN_VERTICAL: f32 = 20.;
const PADDING_HORIZONTAL: f32 = 10.;
const PADDING_VERTICAL: f32 = 5.;

#[derive(Debug)]
pub(crate) struct Renderer {
    d2d_factory: ID2D1Factory2,
    dwrite_factory: IDWriteFactory2,
    context: ID2D1DeviceContext,
    swap_chain: IDXGISwapChain1,
    dcomp_device: IDCompositionDevice,
    _target: IDCompositionTarget,
    text_format: IDWriteTextFormat1,
    brush: ID2D1SolidColorBrush,
}

fn rgba(r: f32, g: f32, b: f32, a: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F { r, g, b, a }
}

impl Renderer {
    pub(crate) fn new(hwnd: HWND) -> Result<Self> {
        unsafe {
            let device = create_dxgi_device()?;
            let d2d_factory = create_d2d_factory()?;
            let context = create_device_context(&device, &d2d_factory)?;
            let swap_chain = create_swap_chain(hwnd, &device)?;
            create_bitmap(&d2d_factory, &context, &swap_chain)?;
            let dcomp_device = create_dcomp_device(&device)?;
            let _target = create_composition(hwnd, &dcomp_device, &swap_chain)?;
            let dwrite_factory = create_dwrite_factory()?;
            let text_format = create_text_format(&dwrite_factory)?;
            let brush = context.CreateSolidColorBrush(&Default::default(), null())?;
            Ok(Self {
                d2d_factory,
                dwrite_factory,
                context,
                swap_chain,
                dcomp_device,
                _target,
                text_format,
                brush,
            })
        }
    }

    pub(crate) fn render(&mut self, state: &mut LyricsWindowState) -> Result<()> {
        let dc = &self.context;
        let swap_chain = &self.swap_chain;
        unsafe {
            dc.BeginDraw();
            dc.Clear(null_mut());
            let animation = self.dcomp_device.CreateAnimation()?;
            animation.End(1., 1.)?;
            state.render(dc)?;
            dc.EndDraw(null_mut(), null_mut())?;
            swap_chain.Present(0, 0)?;
        }
        Ok(())
    }

    fn get_text_rect(&self, text: &str) -> Result<Rect> {
        let client_size = unsafe { self.context.GetSize() };
        let text_rect_size = self.measure_text(text)?;
        let left = (client_size.width - text_rect_size.width) / 2.;
        let top = client_size.height - (MARGIN_VERTICAL + PADDING_VERTICAL) - text_rect_size.height;
        Ok(Rect {
            left,
            top,
            right: left + text_rect_size.width,
            bottom: top + text_rect_size.height,
        })
    }

    fn create_text_layout(
        &self,
        text: &str,
        max_width: f32,
        max_height: f32,
    ) -> Result<IDWriteTextLayout> {
        let dwrite_factory = &self.dwrite_factory;
        let text = HSTRING::from(text);
        unsafe {
            dwrite_factory.CreateTextLayout(
                PWSTR(text.as_wide().as_ptr() as *mut _),
                text.len() as u32,
                &self.text_format,
                max_width,
                max_height,
            )
        }
    }

    pub(crate) fn measure_text(&self, text: &str) -> Result<Size> {
        unsafe {
            let dc = &self.context;
            let D2D_SIZE_F { width, height } = dc.GetSize();
            let max_width = width - 2. * (MARGIN_HORIZONTAL + PADDING_HORIZONTAL);
            let max_height = height - 2. * (MARGIN_VERTICAL + PADDING_VERTICAL);
            let DWRITE_TEXT_METRICS { width, height, .. } = self
                .create_text_layout(text, max_width, max_height)?
                .GetMetrics()?;
            Ok(Size { width, height })
        }
    }

    fn draw_text(&self, text: &str, rect: &Rect, brush: ID2D1Brush) -> Result<()> {
        unsafe {
            let dc = &self.context;
            let text_layout = self.create_text_layout(text, rect.width(), rect.height())?;
            dc.DrawTextLayout(
                D2D_POINT_2F {
                    x: rect.left,
                    y: rect.top,
                },
                text_layout,
                brush,
                D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
            );
            Ok(())
        }
    }

    pub(crate) fn resize(&self, width: u32, height: u32) -> Result<()> {
        unsafe {
            self.context.SetTarget(None);
            self.swap_chain
                .ResizeBuffers(2, width, height, DXGI_FORMAT_B8G8R8A8_UNORM, 0)?;
            create_bitmap(&self.d2d_factory, &self.context, &self.swap_chain)?;
            Ok(())
        }
    }
}

pub(crate) fn get_desktop_dpi() -> Result<(f32, f32)> {
    unsafe {
        let d2d_factory = create_d2d_factory()?;
        let (mut dpi_x, mut dpi_y) = (0., 0.);
        d2d_factory.GetDesktopDpi(&mut dpi_x, &mut dpi_y);
        Ok((dpi_x, dpi_y))
    }
}

fn create_dxgi_device() -> Result<IDXGIDevice> {
    unsafe {
        let mut direct3d_device = None;
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            null(),
            0,
            D3D11_SDK_VERSION,
            &mut direct3d_device,
            null_mut(),
            null_mut(),
        )?;
        let direct3d_device = direct3d_device.unwrap();
        let dxgi_device = direct3d_device.cast::<IDXGIDevice>()?;
        Ok(dxgi_device)
    }
}

fn create_d2d_factory() -> Result<ID2D1Factory2> {
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
) -> Result<ID2D1DeviceContext> {
    unsafe {
        let d2d_device = factory.CreateDevice(dxgi_device)?;
        let dc = d2d_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
        let mut dpi_x = 0.;
        let mut dpi_y = 0.;
        factory.GetDesktopDpi(&mut dpi_x, &mut dpi_y);
        dc.SetDpi(dpi_x, dpi_y);
        dc.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
        Ok(dc)
    }
}

fn create_swap_chain(hwnd: HWND, device: &IDXGIDevice) -> Result<IDXGISwapChain1> {
    unsafe {
        let dxgi_factory = CreateDXGIFactory2::<IDXGIFactory2>(0)?;
        let mut rect = zeroed();
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
) -> Result<ID2D1Bitmap1> {
    unsafe {
        let dxgi_buffer = swap_chain.GetBuffer::<IDXGISurface2>(0)?;

        let properties = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: 0.,
            dpiY: 0.,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
            colorContext: None,
        };

        let bitmap = dc.CreateBitmapFromDxgiSurface(&dxgi_buffer, &properties)?;

        dc.SetTarget(&bitmap);

        Ok(bitmap)
    }
}

fn create_dcomp_device(device: &IDXGIDevice) -> Result<IDCompositionDevice> {
    unsafe { DCompositionCreateDevice(device) }
}

fn create_composition(
    hwnd: HWND,
    dcomp_device: &IDCompositionDevice,
    swap_chain: &IDXGISwapChain1,
) -> Result<IDCompositionTarget> {
    unsafe {
        let target = dcomp_device.CreateTargetForHwnd(hwnd, BOOL(1))?;
        let visual = dcomp_device.CreateVisual()?;
        visual.SetContent(swap_chain)?;
        target.SetRoot(&visual)?;
        dcomp_device.Commit()?;
        Ok(target)
    }
}

fn create_dwrite_factory() -> Result<IDWriteFactory2> {
    unsafe {
        DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory2::IID)
            .unwrap()
            .cast::<IDWriteFactory2>()
    }
}

fn create_text_format(dwrite_factory: &IDWriteFactory2) -> Result<IDWriteTextFormat1> {
    unsafe {
        let text_format: IDWriteTextFormat1 = dwrite_factory
            .CreateTextFormat(
                "Segoe UI",
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                24.,
                "",
            )?
            .cast()?;
        text_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
        text_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;

        let font_fallback_builder = dwrite_factory.CreateFontFallbackBuilder()?;
        let ranges = DWRITE_UNICODE_RANGE {
            first: 0x0,
            last: 0xffffffff,
        };
        let fallback_family_names = [
            "Segoe UI Emoji",
            "Segoe UI Symbol",
            "Helvetica",
            "Microsoft Yahei",
        ];
        // The two map-collect cannot be merged since the `name`s must live long enough.
        let fallback_family_names = fallback_family_names
            .iter()
            .map(|&name| name.encode_utf16().chain([0]).collect::<Vec<u16>>())
            .collect::<Vec<Vec<u16>>>();
        let fallback_family_names = fallback_family_names
            .iter()
            .map(|name| name.as_ptr())
            .collect::<Vec<*const u16>>();
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

        Ok(text_format)
    }
}
