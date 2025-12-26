//! SAPI4 TTS CLI
//!
//! Command-line interface for SAPI4 text-to-speech synthesis

use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod sapi4;

#[derive(Parser)]
#[command(name = "sapi4-rs")]
#[command(about = "SAPI4 Text-to-Speech CLI using Microsoft Speech API 4.0")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available SAPI4 voices
    List,

    /// Synthesize text to a WAV file or stdout
    Speak {
        /// Text to speak
        #[arg(short, long)]
        text: String,

        /// Output WAV file path (omit if using --stdout)
        #[arg(short, long, required_unless_present = "stdout")]
        output: Option<PathBuf>,

        /// Output WAV data to stdout (for piping to mpv, ffmpeg, etc.)
        #[arg(long, conflicts_with = "output")]
        stdout: bool,

        /// ACS file to read voice settings from (overrides other voice options)
        #[arg(long)]
        acs_file: Option<PathBuf>,

        /// Voice name (partial match)
        #[arg(short, long)]
        voice: Option<String>,

        /// Language ID (e.g., 1033 for English US, 1041 for Japanese)
        #[arg(long)]
        lang_id: Option<u16>,

        /// Language dialect (partial match, e.g., "American", "British")
        #[arg(long)]
        dialect: Option<String>,

        /// Gender: 0=neutral, 1=female, 2=male
        #[arg(long)]
        gender: Option<u16>,

        /// Speaker age
        #[arg(long)]
        age: Option<u16>,

        /// Voice style (partial match)
        #[arg(long)]
        style: Option<String>,

        /// Speech speed (engine-dependent range)
        #[arg(long)]
        speed: Option<u32>,

        /// Speech pitch (0-65535)
        #[arg(long)]
        pitch: Option<u16>,

        /// Audio gain/volume multiplier (default: 4.0 for louder output)
        #[arg(short, long, default_value = "4.0")]
        gain: f32,
    },
}

/// Amplify WAV audio data by a gain factor
/// Assumes 16-bit PCM WAV format
fn amplify_wav(wav_data: &mut [u8], gain: f32) {
    // WAV header is typically 44 bytes, but let's find the data chunk properly
    // Look for "data" marker
    let data_pos = wav_data
        .windows(4)
        .position(|w| w == b"data")
        .unwrap_or(36);

    // Skip "data" marker (4 bytes) and size (4 bytes)
    let audio_start = data_pos + 8;

    if audio_start >= wav_data.len() {
        return;
    }

    // Process 16-bit samples (2 bytes each, little-endian)
    let audio_data = &mut wav_data[audio_start..];
    for chunk in audio_data.chunks_exact_mut(2) {
        // Read 16-bit sample (little-endian)
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);

        // Apply gain with saturation (clamp to i16 range)
        let amplified = (sample as f32 * gain).clamp(i16::MIN as f32, i16::MAX as f32) as i16;

        // Write back
        let bytes = amplified.to_le_bytes();
        chunk[0] = bytes[0];
        chunk[1] = bytes[1];
    }
}

#[cfg(windows)]
fn format_criteria_desc(criteria: &sapi4::VoiceCriteria) -> String {
    let mut parts = Vec::new();
    if let Some(ref name) = criteria.name {
        parts.push(format!("name=\"{}\"", name));
    }
    if let Some(gender) = criteria.gender {
        let g = match gender {
            0 => "neutral",
            1 => "female",
            2 => "male",
            _ => "unknown",
        };
        parts.push(format!("gender={}", g));
    }
    if let Some(age) = criteria.age {
        parts.push(format!("age={}", age));
    }
    if let Some(lang_id) = criteria.language_id {
        parts.push(format!("lang_id={}", lang_id));
    }
    if let Some(ref dialect) = criteria.dialect {
        parts.push(format!("dialect=\"{}\"", dialect));
    }
    if let Some(ref style) = criteria.style {
        parts.push(format!("style=\"{}\"", style));
    }
    if parts.is_empty() {
        "(default)".to_string()
    } else {
        parts.join(", ")
    }
}

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write};

    let cli = Cli::parse();

    let synth = sapi4::Synthesizer::new()?;

    match cli.command {
        Commands::List => {
            let voices = synth.list_voices()?;
            if voices.is_empty() {
                println!("No SAPI4 voices found. Make sure SAPI4 runtime is installed.");
            } else {
                println!("Available SAPI4 voices:");
                println!("{:-<70}", "");
                for voice in voices {
                    let gender = match voice.gender {
                        0 => "Neutral",
                        1 => "Female",
                        2 => "Male",
                        _ => "Unknown",
                    };
                    println!("  Name: {}", voice.mode_name);
                    println!("  Speaker: {}", voice.speaker);
                    println!("  Gender: {} ({}), Age: {}", gender, voice.gender, voice.age);
                    println!("  Language ID: {}, Dialect: {}", voice.language_id, voice.dialect);
                    if !voice.style.is_empty() {
                        println!("  Style: {}", voice.style);
                    }
                    println!("{:-<70}", "");
                }
            }
        }

        Commands::Speak {
            text,
            output,
            stdout,
            acs_file,
            voice,
            lang_id,
            dialect,
            gender,
            age,
            style,
            speed,
            pitch,
            gain,
        } => {
            // Determine voice criteria and speed/pitch from ACS file or CLI args
            let (criteria, effective_speed, effective_pitch) = if let Some(ref acs_path) = acs_file {
                // Read ACS file and extract voice info
                let acs_data = std::fs::read(acs_path)
                    .map_err(|e| format!("Failed to read ACS file: {}", e))?;
                let acs = acs::Acs::new(acs_data)
                    .map_err(|e| format!("Failed to parse ACS file: {}", e))?;

                let char_info = acs.character_info();
                eprintln!("Loading voice from ACS: {}", char_info.name);

                if let Some(ref voice_info) = char_info.voice_info {
                    let mut criteria = sapi4::VoiceCriteria::default();

                    // Use extra_data if available for matching
                    if let Some(ref extra) = voice_info.extra_data {
                        criteria.language_id = Some(extra.lang_id);
                        criteria.gender = Some(extra.gender);
                        criteria.age = Some(extra.age);
                        if !extra.lang_dialect.is_empty() {
                            criteria.dialect = Some(extra.lang_dialect.clone());
                        }
                        if !extra.style.is_empty() {
                            criteria.style = Some(extra.style.clone());
                        }
                    }

                    // Use ACS speed/pitch, allowing CLI to override
                    let acs_speed = Some(voice_info.speed);
                    let acs_pitch = Some(voice_info.pitch);

                    (
                        criteria,
                        speed.or(acs_speed),
                        pitch.or(acs_pitch),
                    )
                } else {
                    eprintln!("Warning: ACS file has no voice info, using defaults");
                    (
                        sapi4::VoiceCriteria {
                            name: Some("Adult Male #1".to_string()),
                            ..Default::default()
                        },
                        speed,
                        pitch,
                    )
                }
            } else {
                // Build voice criteria from CLI arguments
                let criteria = sapi4::VoiceCriteria {
                    name: voice.clone(),
                    gender,
                    age,
                    language_id: lang_id,
                    dialect: dialect.clone(),
                    style: style.clone(),
                };

                // If no criteria specified at all, default to "Adult Male #1"
                let criteria = if criteria.name.is_none()
                    && criteria.gender.is_none()
                    && criteria.age.is_none()
                    && criteria.language_id.is_none()
                    && criteria.dialect.is_none()
                    && criteria.style.is_none()
                {
                    sapi4::VoiceCriteria {
                        name: Some("Adult Male #1".to_string()),
                        ..Default::default()
                    }
                } else {
                    criteria
                };

                (criteria, speed, pitch)
            };

            // Format criteria description for status output
            let criteria_desc = format_criteria_desc(&criteria);

            if stdout {
                // Output to stdout - use temp file, then write to stdout
                let temp_dir = std::env::temp_dir();
                let temp_file = temp_dir.join(format!("sapi4_tts_{}.wav", std::process::id()));

                // Write status to stderr so it doesn't pollute the WAV stream
                eprintln!("Synthesizing...");
                eprintln!("Voice criteria: {}", criteria_desc);
                eprintln!("Text: \"{}\"", text);

                synth.synthesize_to_file_with_criteria(&text, &criteria, &temp_file, effective_speed, effective_pitch)?;

                // Read temp file and apply gain
                let mut wav_data = std::fs::read(&temp_file)?;
                let _ = std::fs::remove_file(&temp_file); // Clean up

                // Apply gain amplification
                if gain != 1.0 {
                    amplify_wav(&mut wav_data, gain);
                }

                let mut stdout_handle = io::stdout().lock();
                stdout_handle.write_all(&wav_data)?;
                stdout_handle.flush()?;

                eprintln!("Done! ({} bytes, gain: {}x)", wav_data.len(), gain);
            } else if let Some(output_path) = output {
                // Output to file
                eprintln!("Synthesizing to: {}", output_path.display());
                eprintln!("Voice criteria: {}", criteria_desc);
                eprintln!("Text: \"{}\"", text);

                synth.synthesize_to_file_with_criteria(&text, &criteria, &output_path, effective_speed, effective_pitch)?;

                // Apply gain amplification to the output file
                if gain != 1.0 {
                    let mut wav_data = std::fs::read(&output_path)?;
                    amplify_wav(&mut wav_data, gain);
                    std::fs::write(&output_path, &wav_data)?;
                }

                eprintln!("Done! (gain: {}x)", gain);
            }
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This program requires Windows with SAPI4 installed.");
    eprintln!("It can be run on Linux via Wine.");
    std::process::exit(1);
}
