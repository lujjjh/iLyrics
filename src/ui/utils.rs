use std::ptr::null;
use std::ptr::null_mut;

use bindings::Windows::Win32::Foundation::*;
use bindings::Windows::Win32::Graphics::Direct2D::*;
use bindings::Windows::Win32::Graphics::Direct3D11::*;
use bindings::Windows::Win32::Graphics::DirectWrite::*;
use bindings::Windows::Win32::Graphics::Dxgi::*;
use bindings::Windows::Win32::System::Com::*;
use bindings::Windows::Win32::UI::Animation::*;
use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
use windows::*;

pub fn get_desktop_dpi() -> Result<(f32, f32)> {
    let d2d_factory = create_d2d_factory()?;
    let (mut dpi_x, mut dpi_y) = (0., 0.);
    unsafe { d2d_factory.GetDesktopDpi(&mut dpi_x, &mut dpi_y) };
    Ok((dpi_x, dpi_y))
}

pub fn get_scale_factor() -> Result<(f32, f32)> {
    let (dpi_x, dpi_y) = get_desktop_dpi()?;
    Ok((dpi_x / 96., dpi_y / 96.))
}

fn check_result(result: BOOL) -> Result<()> {
    if result.as_bool() {
        Ok(())
    } else {
        Err(HRESULT::from_thread().into())
    }
}

pub fn get_workarea_rect() -> Result<RECT> {
    let mut rect: RECT = Default::default();
    check_result(unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            &mut rect as *mut _ as _,
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    })?;
    Ok(rect)
}

pub fn get_client_rect(hwnd: HWND) -> Result<RECT> {
    let mut rect = RECT::default();
    check_result(unsafe { GetClientRect(hwnd, &mut rect) })?;
    Ok(rect)
}

pub fn create_dxgi_device() -> Result<IDXGIDevice> {
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

pub fn create_d2d_factory() -> Result<ID2D1Factory2> {
    let mut d2d_factory: Option<ID2D1Factory2> = None;
    unsafe {
        D2D1CreateFactory(
            D2D1_FACTORY_TYPE_SINGLE_THREADED,
            &ID2D1Factory::IID,
            &D2D1_FACTORY_OPTIONS {
                debugLevel: D2D1_DEBUG_LEVEL(0),
            },
            d2d_factory.set_abi(),
        )?;
    }
    Ok(d2d_factory.unwrap())
}

pub fn create_device_context(
    dxgi_device: &IDXGIDevice,
    factory: &ID2D1Factory2,
    dpi_x: f32,
    dpi_y: f32,
) -> Result<ID2D1DeviceContext> {
    unsafe {
        let d2d_device = factory.CreateDevice(dxgi_device)?;
        let dc = d2d_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
        dc.SetDpi(dpi_x, dpi_y);
        dc.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
        Ok(dc)
    }
}

pub fn create_swap_chain(hwnd: HWND, device: &IDXGIDevice) -> Result<IDXGISwapChain1> {
    let dxgi_factory = unsafe { CreateDXGIFactory2::<IDXGIFactory2>(0) }?;
    let rect = get_client_rect(hwnd)?;
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
    unsafe { dxgi_factory.CreateSwapChainForComposition(device, &swap_chain_desc, None) }
}

pub fn create_bitmap_from_swap_chain<'a>(
    dc: &ID2D1DeviceContext,
    swap_chain: &IDXGISwapChain1,
    dpi_x: f32,
    dpi_y: f32,
) -> Result<()> {
    let dxgi_buffer = unsafe { swap_chain.GetBuffer::<IDXGISurface2>(0) }?;
    let properties = D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
        },
        bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
        dpiX: dpi_x,
        dpiY: dpi_y,
        ..Default::default()
    };
    let bitmap = unsafe { dc.CreateBitmapFromDxgiSurface(&dxgi_buffer, &properties) }?;
    unsafe { dc.SetTarget(&bitmap) };
    Ok(())
}

pub fn create_dwrite_factory() -> Result<IDWriteFactory2> {
    unsafe {
        DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory2::IID)
            .unwrap()
            .cast::<IDWriteFactory2>()
    }
}

pub fn create_animation_manager() -> Result<IUIAnimationManager> {
    unsafe { CoCreateInstance(&UIAnimationManager, None, CLSCTX_INPROC_SERVER) }
}

pub fn create_animation_timer() -> Result<IUIAnimationTimer> {
    unsafe { CoCreateInstance(&UIAnimationTimer, None, CLSCTX_INPROC_SERVER) }
}

pub fn create_animation_transition_library() -> Result<IUIAnimationTransitionLibrary> {
    unsafe { CoCreateInstance(&UIAnimationTransitionLibrary, None, CLSCTX_INPROC_SERVER) }
}
