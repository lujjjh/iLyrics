use std::intrinsics::transmute;
use std::mem;
use std::ptr::null_mut;
use std::time::Duration;
use std::time::SystemTime;

use bindings::Windows::Win32::Foundation::*;
use bindings::Windows::Win32::System::Com::*;
use windows::*;

use super::itunes_events::ITunesImplementation;
use super::itunes_events::I_ITUNES_EVENTS_IID;

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
