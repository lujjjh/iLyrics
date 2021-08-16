use std::mem::transmute_copy;
use std::ptr::null_mut;
use std::ptr::NonNull;

use bindings::Windows::Win32::Foundation::*;
use bindings::Windows::Win32::System::OleAutomation::*;
use bindings::Windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;
use windows::*;

// TODO: Rewrite with windows::implement once it is ready.

pub enum ITEvent {
    AboutToPromptUserToQuitEvent = 9,
}

pub const I_ITUNES_EVENTS_IID: Guid = Guid::from_values(
    0x5846_EB78,
    0x317E,
    0x4B6F,
    [0xB0, 0xC3, 0x11, 0xEE, 0x8C, 0x8F, 0xEE, 0xF2],
);

#[repr(C)]
pub struct ITunesEvents {
    vtable: *const ITunesEvents_abi,
    implementation: ITunesImplementation,
    count: WeakRefCount,
}

impl ITunesEvents {
    pub fn new(implementation: ITunesImplementation) -> Self {
        Self {
            vtable: &Self::VTABLE,
            implementation,
            count: WeakRefCount::new(),
        }
    }

    const VTABLE: ITunesEvents_abi = ITunesEvents_abi(
        Self::query_interface,
        Self::add_ref,
        Self::release,
        Self::get_type_info_count,
        Self::get_type_info,
        Self::get_ids_of_names,
        Self::invoke,
    );

    pub unsafe extern "system" fn query_interface(
        this: RawPtr,
        iid: &Guid,
        interface: *mut RawPtr,
    ) -> HRESULT {
        *interface = match iid {
            &IUnknown::IID | &IDispatch::IID | &I_ITUNES_EVENTS_IID => this,
            _ => null_mut(),
        };
        if (*interface).is_null() {
            HRESULT(0x8000_4002) // E_NOINTERFACE
        } else {
            let this = &mut *((this as *mut ::windows::RawPtr) as *mut Self);
            this.count.add_ref();
            HRESULT(0)
        }
    }

    pub unsafe extern "system" fn add_ref(this: RawPtr) -> u32 {
        let this = &mut *((this as *mut ::windows::RawPtr) as *mut Self);
        this.count.add_ref()
    }

    pub unsafe extern "system" fn release(this: RawPtr) -> u32 {
        let this = &mut *((this as *mut ::windows::RawPtr) as *mut Self);
        let remaining = this.count.release();
        if remaining == 0 {
            Box::from_raw(this);
        }
        remaining
    }

    pub unsafe extern "system" fn get_type_info_count(
        _this: RawPtr,
        _pctinfo: *mut u32,
    ) -> HRESULT {
        E_NOTIMPL
    }

    pub unsafe extern "system" fn get_type_info(
        _this: RawPtr,
        _itinfo: u32,
        _lcid: u64,
        _pptinfo: *mut *mut RawPtr,
    ) -> HRESULT {
        E_NOTIMPL
    }

    pub unsafe extern "system" fn get_ids_of_names(
        _this: RawPtr,
        _riid: &Guid,
        _rgsz_names: *mut *mut u8,
        _c_names: u32,
        _lcid: u64,
        _rgdispid: *mut i64,
    ) -> HRESULT {
        E_NOTIMPL
    }

    pub unsafe extern "system" fn invoke(
        this: RawPtr,
        dispid_member: i64,
        _riid: &Guid,
        _lcid: u64,
        _flags: u16,
        _pdispparams: RawPtr,
        _pvar_result: *mut RawPtr,
        _pexcepinfo: *mut RawPtr,
        _pu_arg_err: *mut u32,
    ) -> HRESULT {
        let this = &mut *((this as *mut ::windows::RawPtr) as *mut Self);
        if dispid_member == ITEvent::AboutToPromptUserToQuitEvent as i64 {
            this.implementation.on_obout_to_prompt_user_to_quit();
        };
        HRESULT(0)
    }
}

pub struct ITunesImplementation;

impl ITunesImplementation {
    pub fn new() -> Self {
        Self
    }

    pub fn on_obout_to_prompt_user_to_quit(&self) {
        unsafe { PostQuitMessage(0) };
    }
}

impl From<ITunesImplementation> for IUnknown {
    fn from(implementation: ITunesImplementation) -> Self {
        let com = ITunesEvents::new(implementation);
        unsafe {
            let ptr = Box::into_raw(Box::new(com));
            transmute_copy(&NonNull::new_unchecked(&mut (*ptr).vtable as *mut _ as _))
        }
    }
}

impl<'a> IntoParam<'a, IUnknown> for ITunesImplementation {
    fn into_param(self) -> windows::Param<'a, IUnknown> {
        Param::Owned(Into::<IUnknown>::into(self))
    }
}

#[repr(C)]
#[allow(non_camel_case_types)]
struct ITunesEvents_abi(
    // IUnknown
    pub unsafe extern "system" fn(this: RawPtr, iid: &Guid, interface: *mut RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> u32,
    pub unsafe extern "system" fn(this: RawPtr) -> u32,
    // IDispatch
    pub unsafe extern "system" fn(this: RawPtr, pctinfo: *mut u32) -> HRESULT,
    pub  unsafe extern "system" fn(
        this: RawPtr,
        itinfo: u32,
        lcid: u64,
        pptinfo: *mut *mut RawPtr,
    ) -> HRESULT,
    pub  unsafe extern "system" fn(
        this: RawPtr,
        riid: &Guid,
        rgszNames: *mut *mut u8,
        cNames: u32,
        lcid: u64,
        rgdispid: *mut i64,
    ) -> HRESULT,
    pub  unsafe extern "system" fn(
        this: RawPtr,
        dispidMember: i64,
        riid: &Guid,
        lcid: u64,
        wFlags: u16,
        pdispparams: RawPtr,
        pvarResult: *mut RawPtr,
        pexcepinfo: *mut RawPtr,
        puArgErr: *mut u32,
    ) -> HRESULT,
);
