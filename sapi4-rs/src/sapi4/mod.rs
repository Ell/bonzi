//! SAPI4 - Microsoft Speech API 4.0 bindings for Rust
//!
//! This module provides Rust bindings to the legacy Microsoft Speech API 4.0,
//! which includes the classic "Microsoft Sam" voice from Windows 2000.

pub mod guids;
pub mod types;

#[cfg(windows)]
pub mod interfaces;

#[cfg(windows)]
mod synthesizer;

#[cfg(windows)]
pub use synthesizer::*;
