//! SAPI4 COM Interface definitions using raw vtables
//!
//! These interfaces are translated from Microsoft Speech API 4.0 SDK (speech.h)
//! We use the ANSI versions (A suffix) for simplicity.
//!
//! Instead of using the windows crate's #[interface] macro (which has specific requirements),
//! we define the vtables manually for maximum compatibility.

#![cfg(windows)]
#![allow(non_snake_case)]

use std::ffi::c_void;
use windows::core::{GUID, HRESULT, IUnknown, Interface};

use super::types::{SData, TtsModeInfoA, TtsMouth, VoiceCharset};

/// ITTSEnumA vtable
#[repr(C)]
pub struct ITTSEnumA_Vtbl {
    pub base__: windows::core::IUnknown_Vtbl,
    pub Next: unsafe extern "system" fn(
        this: *mut c_void,
        num_to_fetch: u32,
        mode_info: *mut TtsModeInfoA,
        num_fetched: *mut u32,
    ) -> HRESULT,
    pub Skip: unsafe extern "system" fn(this: *mut c_void, num: u32) -> HRESULT,
    pub Reset: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub Clone: unsafe extern "system" fn(this: *mut c_void, enum_out: *mut *mut c_void) -> HRESULT,
    pub Select: unsafe extern "system" fn(
        this: *mut c_void,
        mode_id: GUID,
        central: *mut *mut c_void,
        audio_dest: *mut c_void,
    ) -> HRESULT,
}

/// ITTSEnumA interface wrapper
#[repr(transparent)]
pub struct ITTSEnumA(IUnknown);

impl ITTSEnumA {
    pub const IID: GUID = GUID::from_u128(0x05EB6C6D_DBAB_11CD_B3CA_00AA0047BA4F);

    #[inline]
    fn vtbl(&self) -> &ITTSEnumA_Vtbl {
        unsafe { &*(*(self.0.as_raw() as *const *const ITTSEnumA_Vtbl)) }
    }

    pub unsafe fn Next(
        &self,
        num_to_fetch: u32,
        mode_info: *mut TtsModeInfoA,
        num_fetched: *mut u32,
    ) -> HRESULT {
        (self.vtbl().Next)(self.0.as_raw(), num_to_fetch, mode_info, num_fetched)
    }

    pub unsafe fn Skip(&self, num: u32) -> HRESULT {
        (self.vtbl().Skip)(self.0.as_raw(), num)
    }

    pub unsafe fn Reset(&self) -> HRESULT {
        (self.vtbl().Reset)(self.0.as_raw())
    }

    pub unsafe fn Select(
        &self,
        mode_id: GUID,
        central: *mut *mut c_void,
        audio_dest: *mut c_void,
    ) -> HRESULT {
        (self.vtbl().Select)(self.0.as_raw(), mode_id, central, audio_dest)
    }
}

unsafe impl windows::core::Interface for ITTSEnumA {
    type Vtable = ITTSEnumA_Vtbl;
    const IID: GUID = Self::IID;
}

impl std::ops::Deref for ITTSEnumA {
    type Target = IUnknown;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for ITTSEnumA {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// ITTSCentralA vtable
#[repr(C)]
pub struct ITTSCentralA_Vtbl {
    pub base__: windows::core::IUnknown_Vtbl,
    pub Inject: unsafe extern "system" fn(this: *mut c_void, text: *const u8) -> HRESULT,
    pub ModeGet: unsafe extern "system" fn(this: *mut c_void, mode_info: *mut TtsModeInfoA) -> HRESULT,
    pub Phoneme: unsafe extern "system" fn(
        this: *mut c_void,
        charset: VoiceCharset,
        flags: u32,
        input: SData,
        output: *mut SData,
    ) -> HRESULT,
    pub PosnGet: unsafe extern "system" fn(this: *mut c_void, pos: *mut u64) -> HRESULT,
    pub TextData: unsafe extern "system" fn(
        this: *mut c_void,
        charset: VoiceCharset,
        flags: u32,
        text_data: SData,
        notify_interface: *mut c_void,
        notify_iid: GUID,
    ) -> HRESULT,
    pub ToFileTime: unsafe extern "system" fn(this: *mut c_void, pos: *mut u64, file_time: *mut u64) -> HRESULT,
    pub AudioPause: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub AudioResume: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub AudioReset: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub Register: unsafe extern "system" fn(
        this: *mut c_void,
        notify_interface: *mut c_void,
        notify_iid: GUID,
        key: *mut u32,
    ) -> HRESULT,
    pub UnRegister: unsafe extern "system" fn(this: *mut c_void, key: u32) -> HRESULT,
}

/// ITTSCentralA interface wrapper
#[repr(transparent)]
pub struct ITTSCentralA(IUnknown);

impl ITTSCentralA {
    pub const IID: GUID = GUID::from_u128(0x05EB6C6A_DBAB_11CD_B3CA_00AA0047BA4F);

    #[inline]
    fn vtbl(&self) -> &ITTSCentralA_Vtbl {
        unsafe { &*(*(self.0.as_raw() as *const *const ITTSCentralA_Vtbl)) }
    }

    pub unsafe fn TextData(
        &self,
        charset: VoiceCharset,
        flags: u32,
        text_data: SData,
        notify_interface: *mut c_void,
        notify_iid: GUID,
    ) -> HRESULT {
        (self.vtbl().TextData)(self.0.as_raw(), charset, flags, text_data, notify_interface, notify_iid)
    }

    pub unsafe fn Register(
        &self,
        notify_interface: *mut c_void,
        notify_iid: GUID,
        key: *mut u32,
    ) -> HRESULT {
        (self.vtbl().Register)(self.0.as_raw(), notify_interface, notify_iid, key)
    }

    pub unsafe fn UnRegister(&self, key: u32) -> HRESULT {
        (self.vtbl().UnRegister)(self.0.as_raw(), key)
    }

    pub unsafe fn AudioReset(&self) -> HRESULT {
        (self.vtbl().AudioReset)(self.0.as_raw())
    }
}

unsafe impl windows::core::Interface for ITTSCentralA {
    type Vtable = ITTSCentralA_Vtbl;
    const IID: GUID = Self::IID;
}

impl std::ops::Deref for ITTSCentralA {
    type Target = IUnknown;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for ITTSCentralA {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// ITTSAttributesA vtable
#[repr(C)]
pub struct ITTSAttributesA_Vtbl {
    pub base__: windows::core::IUnknown_Vtbl,
    pub PitchGet: unsafe extern "system" fn(this: *mut c_void, pitch: *mut u16) -> HRESULT,
    pub PitchSet: unsafe extern "system" fn(this: *mut c_void, pitch: u16) -> HRESULT,
    pub RealTimeGet: unsafe extern "system" fn(this: *mut c_void, real_time: *mut u32) -> HRESULT,
    pub RealTimeSet: unsafe extern "system" fn(this: *mut c_void, real_time: u32) -> HRESULT,
    pub SpeedGet: unsafe extern "system" fn(this: *mut c_void, speed: *mut u32) -> HRESULT,
    pub SpeedSet: unsafe extern "system" fn(this: *mut c_void, speed: u32) -> HRESULT,
    pub VolumeGet: unsafe extern "system" fn(this: *mut c_void, volume: *mut u32) -> HRESULT,
    pub VolumeSet: unsafe extern "system" fn(this: *mut c_void, volume: u32) -> HRESULT,
}

/// ITTSAttributesA interface wrapper
#[repr(transparent)]
pub struct ITTSAttributesA(IUnknown);

impl ITTSAttributesA {
    pub const IID: GUID = GUID::from_u128(0x0FD6E2A1_E77D_11CD_B3CA_00AA0047BA4F);

    #[inline]
    fn vtbl(&self) -> &ITTSAttributesA_Vtbl {
        unsafe { &*(*(self.0.as_raw() as *const *const ITTSAttributesA_Vtbl)) }
    }

    pub unsafe fn PitchSet(&self, pitch: u16) -> HRESULT {
        (self.vtbl().PitchSet)(self.0.as_raw(), pitch)
    }

    pub unsafe fn SpeedSet(&self, speed: u32) -> HRESULT {
        (self.vtbl().SpeedSet)(self.0.as_raw(), speed)
    }
}

unsafe impl windows::core::Interface for ITTSAttributesA {
    type Vtable = ITTSAttributesA_Vtbl;
    const IID: GUID = Self::IID;
}

impl std::ops::Deref for ITTSAttributesA {
    type Target = IUnknown;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for ITTSAttributesA {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// ITTSNotifySinkA vtable
#[repr(C)]
pub struct ITTSNotifySinkA_Vtbl {
    pub base__: windows::core::IUnknown_Vtbl,
    pub AttribChanged: unsafe extern "system" fn(this: *mut c_void, attrib: u32) -> HRESULT,
    pub AudioStart: unsafe extern "system" fn(this: *mut c_void, pos: u64) -> HRESULT,
    pub AudioStop: unsafe extern "system" fn(this: *mut c_void, pos: u64) -> HRESULT,
    pub Visual: unsafe extern "system" fn(
        this: *mut c_void,
        pos: u64,
        phoneme: u8,
        eng_phoneme: u8,
        hints: u32,
        mouth: *const TtsMouth,
    ) -> HRESULT,
}

pub const IID_ITTSNOTIFYSINKA: GUID = GUID::from_u128(0x05EB6C6F_DBAB_11CD_B3CA_00AA0047BA4F);

/// IAudioFile vtable
#[repr(C)]
pub struct IAudioFile_Vtbl {
    pub base__: windows::core::IUnknown_Vtbl,
    pub Register: unsafe extern "system" fn(this: *mut c_void, notify_sink: *mut c_void) -> HRESULT,
    pub Set: unsafe extern "system" fn(this: *mut c_void, file_path: *const u16, id: u32) -> HRESULT,
    pub Add: unsafe extern "system" fn(this: *mut c_void, file_path: *const u16, id: u32) -> HRESULT,
    pub Flush: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub RealTimeSet: unsafe extern "system" fn(this: *mut c_void, time: u16) -> HRESULT,
    pub RealTimeGet: unsafe extern "system" fn(this: *mut c_void, time: *mut u16) -> HRESULT,
}

/// IAudioFile interface wrapper
#[repr(transparent)]
pub struct IAudioFile(IUnknown);

impl IAudioFile {
    pub const IID: GUID = GUID::from_u128(0xFD7C2320_3D6D_11B9_C000_FED6CBA3B1A9);

    #[inline]
    fn vtbl(&self) -> &IAudioFile_Vtbl {
        unsafe { &*(*(self.0.as_raw() as *const *const IAudioFile_Vtbl)) }
    }

    pub unsafe fn Set(&self, file_path: *const u16, id: u32) -> HRESULT {
        (self.vtbl().Set)(self.0.as_raw(), file_path, id)
    }

    pub unsafe fn Flush(&self) -> HRESULT {
        (self.vtbl().Flush)(self.0.as_raw())
    }
}

unsafe impl windows::core::Interface for IAudioFile {
    type Vtable = IAudioFile_Vtbl;
    const IID: GUID = Self::IID;
}

impl std::ops::Deref for IAudioFile {
    type Target = IUnknown;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for IAudioFile {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
