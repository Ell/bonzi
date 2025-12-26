//! SAPI4 TTS Synthesizer
//!
//! High-level interface for SAPI4 text-to-speech synthesis

#![cfg(windows)]
#![allow(non_snake_case)]

use std::ffi::c_void;
use std::path::Path;
use std::ptr;

use windows::core::{IUnknown, Interface, GUID};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
};

use super::guids::*;
use super::interfaces::*;
use super::types::*;

/// Error types for SAPI4 operations
#[derive(Debug, thiserror::Error)]
pub enum Sapi4Error {
    #[error("COM initialization failed: {0}")]
    ComInit(String),
    #[error("Failed to create TTS enumerator: {0}")]
    EnumeratorCreate(String),
    #[error("Failed to enumerate voices: {0}")]
    EnumerateVoices(String),
    #[error("Voice not found: {0}")]
    VoiceNotFound(String),
    #[error("Failed to select voice: {0}")]
    SelectVoice(String),
    #[error("Failed to create audio destination: {0}")]
    AudioDestCreate(String),
    #[error("Failed to set output file: {0}")]
    SetOutputFile(String),
    #[error("Failed to synthesize text: {0}")]
    Synthesize(String),
    #[error("Failed to get/set attributes: {0}")]
    Attributes(String),
}

pub type Result<T> = std::result::Result<T, Sapi4Error>;

/// Information about an available TTS voice
#[derive(Debug, Clone)]
pub struct VoiceInfo {
    pub mode_id: GUID,
    pub mode_name: String,
    pub speaker: String,
    pub gender: u16,
    pub age: u16,
    pub language_id: u16,
    pub dialect: String,
    pub style: String,
}

/// Criteria for selecting a voice (all fields are optional filters)
#[derive(Debug, Clone, Default)]
pub struct VoiceCriteria {
    pub name: Option<String>,
    pub gender: Option<u16>,
    pub age: Option<u16>,
    pub language_id: Option<u16>,
    pub dialect: Option<String>,
    pub style: Option<String>,
}

/// SAPI4 TTS Synthesizer
pub struct Synthesizer {
    _com_initialized: bool,
}

impl Synthesizer {
    /// Create a new synthesizer, initializing COM
    pub fn new() -> Result<Self> {
        unsafe {
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            if hr.is_err() {
                return Err(Sapi4Error::ComInit(format!("HRESULT: {:?}", hr)));
            }
        }
        Ok(Self {
            _com_initialized: true,
        })
    }

    /// List all available SAPI4 voices
    pub fn list_voices(&self) -> Result<Vec<VoiceInfo>> {
        unsafe {
            // Create TTS enumerator
            let enumerator: ITTSEnumA =
                CoCreateInstance(&CLSID_TTSENUMERATOR, None, CLSCTX_ALL)
                    .map_err(|e| Sapi4Error::EnumeratorCreate(format!("{:?}", e)))?;

            let mut voices = Vec::new();
            let mut mode_info = TtsModeInfoA::default();
            let mut fetched: u32 = 0;

            loop {
                let hr = enumerator.Next(1, &mut mode_info, &mut fetched);
                if hr.is_err() || fetched == 0 {
                    break;
                }

                voices.push(VoiceInfo {
                    mode_id: mode_info.mode_id,
                    mode_name: mode_info.mode_name_str(),
                    speaker: mode_info.speaker_str(),
                    gender: mode_info.gender,
                    age: mode_info.age,
                    language_id: mode_info.language_id(),
                    dialect: mode_info.dialect_str(),
                    style: mode_info.style_str(),
                });
            }

            Ok(voices)
        }
    }

    /// Find a voice by name (partial match)
    pub fn find_voice(&self, name: &str) -> Result<VoiceInfo> {
        self.find_voice_by_criteria(&VoiceCriteria {
            name: Some(name.to_string()),
            ..Default::default()
        })
    }

    /// Find a voice by multiple criteria (ACS-style matching)
    /// Returns the first voice that matches ALL specified criteria
    pub fn find_voice_by_criteria(&self, criteria: &VoiceCriteria) -> Result<VoiceInfo> {
        let voices = self.list_voices()?;

        // Score each voice based on how well it matches the criteria
        let mut best_match: Option<(VoiceInfo, u32)> = None;

        for voice in voices {
            let mut score = 0u32;
            let mut matched = true;

            // Name matching (partial, case-insensitive)
            if let Some(ref name) = criteria.name {
                let name_lower = name.to_lowercase();
                if voice.mode_name.to_lowercase().contains(&name_lower)
                    || voice.speaker.to_lowercase().contains(&name_lower)
                {
                    score += 10;
                } else {
                    matched = false;
                }
            }

            // Gender matching (exact)
            if let Some(gender) = criteria.gender {
                if voice.gender == gender {
                    score += 20;
                } else {
                    matched = false;
                }
            }

            // Age matching (exact)
            if let Some(age) = criteria.age {
                if voice.age == age {
                    score += 15;
                } else {
                    matched = false;
                }
            }

            // Language ID matching (exact)
            if let Some(lang_id) = criteria.language_id {
                if voice.language_id == lang_id {
                    score += 25;
                } else {
                    matched = false;
                }
            }

            // Dialect matching (partial, case-insensitive)
            if let Some(ref dialect) = criteria.dialect {
                let dialect_lower = dialect.to_lowercase();
                if voice.dialect.to_lowercase().contains(&dialect_lower) {
                    score += 15;
                } else {
                    matched = false;
                }
            }

            // Style matching (partial, case-insensitive)
            if let Some(ref style) = criteria.style {
                let style_lower = style.to_lowercase();
                if voice.style.to_lowercase().contains(&style_lower) {
                    score += 10;
                } else {
                    matched = false;
                }
            }

            if matched {
                if let Some((_, best_score)) = &best_match {
                    if score > *best_score {
                        best_match = Some((voice, score));
                    }
                } else {
                    best_match = Some((voice, score));
                }
            }
        }

        best_match
            .map(|(voice, _)| voice)
            .ok_or_else(|| Sapi4Error::VoiceNotFound(format!("{:?}", criteria)))
    }

    /// Synthesize text to a WAV file using voice name
    pub fn synthesize_to_file(
        &self,
        text: &str,
        voice_name: &str,
        output_path: &Path,
        speed: Option<u32>,
        pitch: Option<u16>,
    ) -> Result<()> {
        self.synthesize_to_file_with_criteria(
            text,
            &VoiceCriteria {
                name: Some(voice_name.to_string()),
                ..Default::default()
            },
            output_path,
            speed,
            pitch,
        )
    }

    /// Synthesize text to a WAV file using voice criteria
    pub fn synthesize_to_file_with_criteria(
        &self,
        text: &str,
        criteria: &VoiceCriteria,
        output_path: &Path,
        speed: Option<u32>,
        pitch: Option<u16>,
    ) -> Result<()> {
        unsafe {
            // Find the voice
            let voice = self.find_voice_by_criteria(criteria)?;

            // Create TTS enumerator
            let enumerator: ITTSEnumA =
                CoCreateInstance(&CLSID_TTSENUMERATOR, None, CLSCTX_ALL)
                    .map_err(|e| Sapi4Error::EnumeratorCreate(format!("{:?}", e)))?;

            // Create audio destination file
            let audio_dest: IAudioFile =
                CoCreateInstance(&CLSID_AUDIODESTFILE, None, CLSCTX_ALL)
                    .map_err(|e| Sapi4Error::AudioDestCreate(format!("{:?}", e)))?;

            // Convert path to wide string
            let path_str = output_path.to_string_lossy();
            let wide_path: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();

            // Set the output file
            let hr = audio_dest.Set(wide_path.as_ptr(), 0);
            if hr.is_err() {
                return Err(Sapi4Error::SetOutputFile(format!("{:?}", hr)));
            }

            // Select the voice
            let mut central_ptr: *mut c_void = ptr::null_mut();
            let audio_dest_unknown: IUnknown = audio_dest.cast().unwrap();

            let hr = enumerator.Select(
                voice.mode_id,
                &mut central_ptr,
                audio_dest_unknown.as_raw(),
            );
            if hr.is_err() {
                return Err(Sapi4Error::SelectVoice(format!("{:?}", hr)));
            }

            if central_ptr.is_null() {
                return Err(Sapi4Error::SelectVoice("Got null ITTSCentral".to_string()));
            }

            // Cast to ITTSCentralA
            // NOTE: This is a raw pointer, we need to be careful about ownership
            let central_unknown = IUnknown::from_raw(central_ptr);
            let central: ITTSCentralA = central_unknown.cast()
                .map_err(|e| Sapi4Error::SelectVoice(format!("Cast to ITTSCentralA failed: {:?}", e)))?;

            // Set speed and pitch if specified
            if speed.is_some() || pitch.is_some() {
                if let Ok(attrs) = central.cast::<ITTSAttributesA>() {
                    if let Some(s) = speed {
                        let _ = attrs.SpeedSet(s);
                    }
                    if let Some(p) = pitch {
                        let _ = attrs.PitchSet(p);
                    }
                }
            }

            // Prepare text data (null-terminated for ANSI)
            let mut text_with_null = text.as_bytes().to_vec();
            text_with_null.push(0);
            let text_data = SData::from_bytes(&text_with_null);

            // Reset audio before starting
            let _ = central.AudioReset();

            // Synthesize (without notification sink for simplicity)
            // Use TTSDATAFLAG_TAGGED (1) like the reference implementation
            let hr = central.TextData(
                VoiceCharset::Text,
                TTSDATAFLAG_TAGGED,
                text_data,
                ptr::null_mut(), // no notification sink
                GUID::zeroed(),
            );
            if hr.is_err() {
                return Err(Sapi4Error::Synthesize(format!("TextData failed: {:?}", hr)));
            }

            // Run a Windows message pump to allow COM to process
            // SAPI4 synthesis is asynchronous and requires message processing
            let wait_ms = 2000 + (text.len() as u64 * 100);
            let start = std::time::Instant::now();
            let mut msg = MSG::default();

            while start.elapsed().as_millis() < wait_ms as u128 {
                // Process any pending Windows messages
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                // Small sleep to avoid busy-waiting
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            // Flush audio file to ensure all data is written
            let _ = audio_dest.Flush();

            // Process any remaining messages after flush
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Additional short wait after flush
            std::thread::sleep(std::time::Duration::from_millis(500));

            Ok(())
        }
    }
}

impl Drop for Synthesizer {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
