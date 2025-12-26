//! SAPI4 type definitions
//!
//! These types are translated from Microsoft Speech API 4.0 SDK (speech.h)

#[cfg(windows)]
use windows::core::GUID;

// Constants from speech.h
pub const SVFN_LEN: usize = 262;
pub const LANG_LEN: usize = 64;
pub const TTSI_NAMELEN: usize = SVFN_LEN;
pub const TTSI_STYLELEN: usize = SVFN_LEN;

/// Voice character set for TextData
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceCharset {
    Text = 0,
    IpaPhonetic = 1,
    EnginePhonetic = 2,
}

/// Gender constants
pub const GENDER_NEUTRAL: u16 = 0;
pub const GENDER_FEMALE: u16 = 1;
pub const GENDER_MALE: u16 = 2;

/// SDATA structure - pointer to data with size
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SData {
    pub data: *const u8,
    pub size: u32,
}

impl SData {
    pub fn from_str(s: &str) -> Self {
        Self {
            data: s.as_ptr(),
            size: s.len() as u32,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: bytes.as_ptr(),
            size: bytes.len() as u32,
        }
    }
}

/// LANGUAGE structure (ANSI version)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct LanguageA {
    pub language_id: u16,
    pub dialect: [u8; LANG_LEN],
}

impl Default for LanguageA {
    fn default() -> Self {
        Self {
            language_id: 0,
            dialect: [0u8; LANG_LEN],
        }
    }
}

/// TTSMODEINFO structure (ANSI version)
/// Contains information about a TTS voice mode
#[repr(C)]
#[derive(Clone)]
#[cfg(windows)]
pub struct TtsModeInfoA {
    pub engine_id: GUID,
    pub mfg_name: [u8; TTSI_NAMELEN],
    pub product_name: [u8; TTSI_NAMELEN],
    pub mode_id: GUID,
    pub mode_name: [u8; TTSI_NAMELEN],
    pub language: LanguageA,
    pub speaker: [u8; TTSI_NAMELEN],
    pub style: [u8; TTSI_STYLELEN],
    pub gender: u16,
    pub age: u16,
    pub features: u32,
    pub interfaces: u32,
    pub engine_features: u32,
}

#[cfg(windows)]
impl Default for TtsModeInfoA {
    fn default() -> Self {
        Self {
            engine_id: GUID::zeroed(),
            mfg_name: [0u8; TTSI_NAMELEN],
            product_name: [0u8; TTSI_NAMELEN],
            mode_id: GUID::zeroed(),
            mode_name: [0u8; TTSI_NAMELEN],
            language: LanguageA::default(),
            speaker: [0u8; TTSI_NAMELEN],
            style: [0u8; TTSI_STYLELEN],
            gender: 0,
            age: 0,
            features: 0,
            interfaces: 0,
            engine_features: 0,
        }
    }
}

#[cfg(windows)]
impl TtsModeInfoA {
    pub fn mode_name_str(&self) -> String {
        let end = self.mode_name.iter().position(|&c| c == 0).unwrap_or(self.mode_name.len());
        String::from_utf8_lossy(&self.mode_name[..end]).to_string()
    }

    pub fn speaker_str(&self) -> String {
        let end = self.speaker.iter().position(|&c| c == 0).unwrap_or(self.speaker.len());
        String::from_utf8_lossy(&self.speaker[..end]).to_string()
    }

    pub fn style_str(&self) -> String {
        let end = self.style.iter().position(|&c| c == 0).unwrap_or(self.style.len());
        String::from_utf8_lossy(&self.style[..end]).to_string()
    }

    pub fn dialect_str(&self) -> String {
        let end = self.language.dialect.iter().position(|&c| c == 0).unwrap_or(self.language.dialect.len());
        String::from_utf8_lossy(&self.language.dialect[..end]).to_string()
    }

    pub fn language_id(&self) -> u16 {
        self.language.language_id
    }
}

/// TTSMOUTH structure - lip sync data
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TtsMouth {
    pub mouth_height: u8,
    pub mouth_width: u8,
    pub mouth_upturn: u8,
    pub jaw_open: u8,
    pub teeth_upper_visible: u8,
    pub teeth_lower_visible: u8,
    pub tongue_posn: u8,
    pub lip_tension: u8,
}

// Text data flags
pub const TTSDATAFLAG_TAGGED: u32 = 1;
