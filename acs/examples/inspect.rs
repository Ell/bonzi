use std::fs;
use acs::Acs;

fn format_guid(bytes: &[u8; 16]) -> String {
    // GUID format: {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}
    format!(
        "{{{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        bytes[3], bytes[2], bytes[1], bytes[0],
        bytes[5], bytes[4],
        bytes[7], bytes[6],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

fn main() {
    let path = std::env::args().nth(1).expect("Usage: inspect <file.acs>");
    let data = fs::read(&path).expect("read file");
    let mut acs = Acs::new(data).expect("parse");

    println!("Character: {}", acs.character_info().name);

    // Print voice info
    if let Some(ref voice) = acs.character_info().voice_info {
        println!("\nVoice Info:");
        println!("  TTS Engine ID: {}", format_guid(&voice.tts_engine_id));
        println!("  TTS Mode ID: {}", format_guid(&voice.tts_mode_id));
        println!("  Speed: {}", voice.speed);
        println!("  Pitch: {}", voice.pitch);
        if let Some(ref extra) = voice.extra_data {
            println!("  Language ID: {}", extra.lang_id);
            println!("  Dialect: {}", extra.lang_dialect);
            println!("  Gender: {} (0=neutral, 1=female, 2=male)", extra.gender);
            println!("  Age: {}", extra.age);
            println!("  Style: {}", extra.style);
        }
    } else {
        println!("\nNo voice info in this ACS file");
    }

    // Show specific animation details
    let filter = std::env::args().nth(2);

    let names: Vec<String> = acs.animation_names().iter().map(|s| s.to_string()).collect();

    println!("\nAnimations with transitions:");
    for name in names {
        if let Some(ref f) = filter {
            if !name.to_lowercase().contains(&f.to_lowercase()) {
                continue;
            }
        }
        if let Ok(anim) = acs.animation(&name) {
            let return_anim = anim.return_animation.as_deref().unwrap_or("(none)");
            let trans_type = match anim.transition_type {
                acs::TransitionType::UseReturnAnimation => "UseReturn",
                acs::TransitionType::UseExitBranch => "UseExitBranch",
                acs::TransitionType::None => "None",
            };
            println!("  {} ({} frames) -> {} (type: {})", name, anim.frames.len(), return_anim, trans_type);

            // Show exit branches for last few frames if using exit branches
            if anim.transition_type == acs::TransitionType::UseExitBranch {
                for (i, frame) in anim.frames.iter().enumerate() {
                    if frame.exit_branch.is_some() || !frame.branches.is_empty() {
                        println!("    frame {}: exit_branch={:?}, branches={:?}",
                            i, frame.exit_branch,
                            frame.branches.iter().map(|b| (b.frame_index, b.probability)).collect::<Vec<_>>());
                    }
                }
            }
        }
    }
}
