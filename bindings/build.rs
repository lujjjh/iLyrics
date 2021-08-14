fn main() {
    windows::build! {
        Windows::Win32::Foundation::*,
        Windows::Win32::Graphics::Dxgi::*,
        Windows::Win32::Graphics::Direct2D::*,
        Windows::Win32::Graphics::Direct3D11::*,
        Windows::Win32::Graphics::DirectWrite::*,
        Windows::Win32::Graphics::DirectComposition::*,
        Windows::Win32::System::Com::*,
        Windows::Win32::System::Diagnostics::Debug::*,
        Windows::Win32::System::OleAutomation::*,
        Windows::Win32::System::SystemServices::*,
        Windows::Win32::System::LibraryLoader::*,
        Windows::Win32::System::Threading::*,
        Windows::Win32::UI::WindowsAndMessaging::*,
    };
}
