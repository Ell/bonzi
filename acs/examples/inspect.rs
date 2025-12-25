use std::fs;
use acs::Acs;

fn main() {
    let path = std::env::args().nth(1).expect("Usage: inspect <file.acs>");
    let data = fs::read(&path).expect("read file");
    let mut acs = Acs::new(data).expect("parse");

    println!("Character: {}", acs.character_info().name);

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
