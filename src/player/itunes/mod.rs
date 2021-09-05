use std::intrinsics::transmute;
use std::mem;
use std::mem::transmute_copy;
use std::ptr::null_mut;
use std::ptr::NonNull;
use std::time::Duration;
use std::time::SystemTime;

use bindings::Windows::Win32::Foundation::*;
use bindings::Windows::Win32::System::Com::*;
use bindings::Windows::Win32::System::OleAutomation::*;
use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
use windows::*;

use super::Player;
use super::PlayerState;

const ITUNES_CLSID: Guid = Guid::from_values(
    0xDC0C2640,
    0x1415,
    0x4644,
    [0x87, 0x5C, 0x6F, 0x4D, 0x76, 0x98, 0x39, 0xBA],
);

pub struct ITunes {
    instance: IiTunes,

    // last_player_position interpolation.
    last_player_position: Option<Duration>,
    last_player_position_updated_at: SystemTime,
}

impl Player for ITunes {
    fn get_player_state(&mut self) -> Option<PlayerState> {
        if !self.is_playing() {
            return None;
        }
        let current_track_info = self.get_current_track_info();
        let player_position = self.get_player_position();
        current_track_info.and_then(|track_info| {
            player_position.map(|player_position| PlayerState {
                song_name: track_info.name,
                song_artist: track_info.artist,
                player_position,
            })
        })
    }
}

impl ITunes {
    pub fn new() -> windows::Result<Self> {
        unsafe {
            CoInitialize(null_mut())?;
            let instance: IiTunes = CoCreateInstance(&ITUNES_CLSID, None, CLSCTX_LOCAL_SERVER)?;
            let connection_point_container = instance.cast::<IConnectionPointContainer>()?;
            let connection_point =
                connection_point_container.FindConnectionPoint(&I_ITUNES_EVENTS_IID)?;
            let itunes_events = ITunesImplementation::new();
            connection_point.Advise(itunes_events)?;
            Ok(Self {
                instance,
                last_player_position: None,
                last_player_position_updated_at: SystemTime::now(),
            })
        }
    }

    fn get_instance(&self) -> &IiTunes {
        &self.instance
    }

    pub fn is_playing(&self) -> bool {
        unsafe { self.get_instance().GetPlayerState() }
            .map(|state| state == 1)
            .unwrap_or(false)
    }

    pub fn get_current_track_info(&self) -> Option<TrackInfo> {
        unsafe {
            self.get_instance()
                .GetCurrentTrack()
                .unwrap_or(None)
                .and_then(|track| {
                    (|| -> windows::Result<TrackInfo> {
                        let name = track.GetName()?.to_string();
                        let artist = track.GetArtist()?.to_string();
                        Ok(TrackInfo { name, artist })
                    })()
                    .map(Some)
                    .unwrap_or(None)
                })
        }
    }

    pub fn get_player_position(&mut self) -> Option<Duration> {
        let player_position = unsafe {
            self.get_instance()
                .GetPlayerPositionMS()
                // There appears to be a delay in iTunes' PlayerPositionMS.
                .map(|ms| Some(Duration::from_millis(ms as u64) + Duration::from_millis(350)))
                .unwrap_or(None)
        };
        // iTunes' PlayerPositionMS is not continuous. We still have to do the interpolation.
        if player_position != self.last_player_position {
            self.last_player_position = player_position;
            self.last_player_position_updated_at = SystemTime::now();
        }
        player_position.map(|value| {
            value
                + self
                    .last_player_position_updated_at
                    .elapsed()
                    .unwrap()
                    .min(Duration::from_secs(1))
        })
    }
}

impl Drop for ITunes {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

#[derive(Debug)]
pub struct TrackInfo {
    pub name: String,
    pub artist: String,
}

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq)]
struct IiTunes(IUnknown);

#[repr(C)]
#[allow(non_camel_case_types)]
struct IiTunes_abi(
    // IUnknown
    pub unsafe extern "system" fn(this: RawPtr, iid: *const Guid, interface: *mut RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> u32,
    pub unsafe extern "system" fn(this: RawPtr) -> u32,
    // IDispatch (TODO)
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    // IiTunes
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr, value: *mut i32) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr, value: *mut i64) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr, track: *mut RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr, value: *mut i64) -> HRESULT,
);

unsafe impl Interface for IiTunes {
    type Vtable = IiTunes_abi;

    const IID: Guid = Guid::from_values(
        0x9DD6_680B,
        0x3EDC,
        0x40db,
        [0xA7, 0x71, 0xE6, 0xFE, 0x48, 0x32, 0xE3, 0x4A],
    );
}

#[allow(non_snake_case)]
impl IiTunes {
    pub unsafe fn GetPlayerState(&self) -> Result<i32> {
        let mut value: i32 = 0;
        (Interface::vtable(self).39)(Abi::abi(self), &mut value).ok()?;
        Ok(value)
    }

    pub unsafe fn GetPlayerPositionMS(&self) -> Result<i64> {
        let mut value: i64 = 0;
        (Interface::vtable(self).91)(Abi::abi(self), &mut value).ok()?;
        Ok(value)
    }

    pub unsafe fn GetCurrentTrack(&self) -> Result<Option<IITTrack>> {
        let mut abi = null_mut();
        (Interface::vtable(self).62)(Abi::abi(self), &mut abi).ok()?;
        if abi.is_null() {
            Ok(None)
        } else {
            Ok(transmute(abi))
        }
    }
}

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq)]
struct IITTrack(IUnknown);

#[repr(C)]
#[allow(non_camel_case_types)]
struct IITTrack_abi(
    // IUnknown
    pub unsafe extern "system" fn(this: RawPtr, iid: *const Guid, interface: *mut RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> u32,
    pub unsafe extern "system" fn(this: RawPtr) -> u32,
    // IDispatch (TODO)
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    // IITObject
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr, value: *mut *mut u16) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    // IITTrack
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr) -> HRESULT,
    pub unsafe extern "system" fn(this: RawPtr, value: *mut *mut u16) -> HRESULT,
);

unsafe impl Interface for IITTrack {
    type Vtable = IITTrack_abi;

    const IID: Guid = Guid::from_values(
        0x4cb0_915d,
        0x1e54,
        0x4727,
        [0xba, 0xf3, 0xce, 0x6c, 0xc9, 0xa2, 0x25, 0xa1],
    );
}

#[allow(non_snake_case)]
impl IITTrack {
    pub unsafe fn GetName(&self) -> Result<BSTR> {
        let mut abi: <BSTR as Abi>::Abi = mem::zeroed();
        (Interface::vtable(self).8)(Abi::abi(self), &mut abi).from_abi(abi)
    }

    pub unsafe fn GetArtist(&self) -> Result<BSTR> {
        let mut abi: <BSTR as Abi>::Abi = mem::zeroed();
        (Interface::vtable(self).22)(Abi::abi(self), &mut abi).from_abi(abi)
    }
}

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
