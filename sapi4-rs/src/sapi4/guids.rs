//! SAPI4 COM GUIDs
//!
//! These GUIDs are extracted from the Microsoft Speech API 4.0 SDK (speech.h)

#[cfg(windows)]
use windows::core::GUID;

// CLSID_TTSEnumerator {D67C0280-C743-11cd-80E5-00AA003E4B50}
#[cfg(windows)]
pub const CLSID_TTSENUMERATOR: GUID = GUID::from_u128(
    0xd67c0280_c743_11cd_80e5_00aa003e4b50
);

// CLSID_AudioDestFile {D4623720-E4B9-11cf-8D56-00A0C9034A7E}
#[cfg(windows)]
pub const CLSID_AUDIODESTFILE: GUID = GUID::from_u128(
    0xd4623720_e4b9_11cf_8d56_00a0c9034a7e
);

// IID_ITTSEnumW {6B837B20-4A47-101B-931A-00AA0047BA4F}
#[cfg(windows)]
pub const IID_ITTSENUM: GUID = GUID::from_u128(
    0x6b837b20_4a47_101b_931a_00aa0047ba4f
);

// IID_ITTSFindW {7AA42960-4A47-101B-931A-00AA0047BA4F}
#[cfg(windows)]
pub const IID_ITTSFIND: GUID = GUID::from_u128(
    0x7aa42960_4a47_101b_931a_00aa0047ba4f
);

// IID_ITTSCentralW {28016060-4A47-101B-931A-00AA0047BA4F}
#[cfg(windows)]
pub const IID_ITTSCENTRAL: GUID = GUID::from_u128(
    0x28016060_4a47_101b_931a_00aa0047ba4f
);

// IID_ITTSAttributesW {1287A280-4A47-101B-931A-00AA0047BA4F}
#[cfg(windows)]
pub const IID_ITTSATTRIBUTES: GUID = GUID::from_u128(
    0x1287a280_4a47_101b_931a_00aa0047ba4f
);

// IID_ITTSNotifySinkW {C0FA8F40-4A46-101B-931A-00AA0047BA4F}
#[cfg(windows)]
pub const IID_ITTSNOTIFYSINK: GUID = GUID::from_u128(
    0xc0fa8f40_4a46_101b_931a_00aa0047ba4f
);

// IID_IAudioFile {FD7C2320-3D6D-11b9-C000-FED6CBA3B1A9}
#[cfg(windows)]
pub const IID_IAUDIOFILE: GUID = GUID::from_u128(
    0xfd7c2320_3d6d_11b9_c000_fed6cba3b1a9
);
