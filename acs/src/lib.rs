//! ACS file parser for Microsoft Agent character files.
//!
//! This crate provides both low-level and high-level APIs for reading ACS files.
//!
//! # Example
//!
//! ```ignore
//! use acs::Acs;
//!
//! let data = std::fs::read("bonzi.acs").unwrap();
//! let mut acs = Acs::new(data).unwrap();
//!
//! println!("Character: {}", acs.character_info().name);
//! println!("Animations: {:?}", acs.animation_names());
//!
//! // Get a specific image
//! let image = acs.image(0).unwrap();
//! println!("Image: {}x{}", image.width, image.height);
//! ```

mod acs;
mod bit_reader;
pub mod compression;
pub mod reader;

pub use acs::{
    Acs, AcsError, Animation, Branch, CharacterInfo, Frame, FrameImage, Image, Overlay,
    OverlayType, Sound, TransitionType,
};
pub use reader::{VoiceExtraData, VoiceInfo};
