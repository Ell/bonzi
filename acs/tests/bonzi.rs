use acs::reader::AcsReader;
use acs::Acs;

const BONZI_ACS: &[u8] = include_bytes!("../../notes/files/Bonzi.acs");
const CLIPPIT_ACS: &[u8] = include_bytes!("../../notes/files/clippit.acs");

#[test]
fn test_read_header() {
    let mut reader = AcsReader::new(BONZI_ACS);
    let header = reader.read_header().expect("Failed to read header");

    println!("Header: {:?}", header);
    println!("Character info at offset: {}, size: {}", header.character_info.offset, header.character_info.size);
    println!("Animation info at offset: {}, size: {}", header.animation_info.offset, header.animation_info.size);
    println!("Image info at offset: {}, size: {}", header.image_info.offset, header.image_info.size);
    println!("Audio info at offset: {}, size: {}", header.audio_info.offset, header.audio_info.size);
}

#[test]
fn test_read_character_info_step_by_step() {
    let mut reader = AcsReader::new(BONZI_ACS);
    let header = reader.read_header().expect("Failed to read header");

    println!("File size: {} bytes", BONZI_ACS.len());
    println!("Character info at offset {}, size {}", header.character_info.offset, header.character_info.size);

    // Manually read character info step by step
    reader.seek(header.character_info.offset as u64);

    let minor_version = reader.read_u16().expect("minor_version");
    let major_version = reader.read_u16().expect("major_version");
    println!("Version: {}.{}", major_version, minor_version);

    // Localized info locator
    let loc_offset = reader.read_u32().expect("loc_offset");
    let loc_size = reader.read_u32().expect("loc_size");
    println!("Localized info locator: offset={}, size={}", loc_offset, loc_size);

    // GUID
    let mut guid = [0u8; 16];
    for i in 0..16 {
        guid[i] = reader.read_u8().expect("guid byte");
    }
    println!("GUID: {:02x?}", guid);

    let width = reader.read_u16().expect("width");
    let height = reader.read_u16().expect("height");
    println!("Dimensions: {}x{}", width, height);

    let transparent_color = reader.read_u8().expect("transparent_color");
    println!("Transparent color: {}", transparent_color);

    let flags = reader.read_u32().expect("flags");
    println!("Flags: 0x{:08x}", flags);

    let anim_set_major = reader.read_u16().expect("anim_set_major");
    let anim_set_minor = reader.read_u16().expect("anim_set_minor");
    println!("Anim set version: {}.{}", anim_set_major, anim_set_minor);

    // Check flags for optional sections
    println!("Bit 4 (voice output): {}", flags & 0x10 != 0);
    println!("Bit 5 (unknown/voice?): {}", flags & 0x20 != 0);
    println!("Bit 8: {}", flags & 0x100 != 0);
    println!("Bit 9: {}", flags & 0x200 != 0);

    println!("\nCurrent position: {}", reader.position());

    // Dump raw bytes to see what's there
    let pos = reader.position();
    print!("\nRaw bytes at position {}: ", pos);
    for _ in 0..64 { print!("{:02x} ", reader.read_u8().unwrap_or(0)); }
    println!();
    reader.seek(pos);

    // Let's try reading VOICEINFO regardless of flags
    println!("\n--- Attempting to read VOICEINFO ---");
    let mut guid1 = [0u8; 16];
    for i in 0..16 { guid1[i] = reader.read_u8().expect("guid1"); }
    println!("TTS Engine GUID: {:02x?}", guid1);

    let mut guid2 = [0u8; 16];
    for i in 0..16 { guid2[i] = reader.read_u8().expect("guid2"); }
    println!("TTS Mode GUID: {:02x?}", guid2);

    let speed = reader.read_u32().expect("speed");
    let pitch = reader.read_u16().expect("pitch");
    let extra_flag = reader.read_u8().expect("extra_flag");
    println!("Speed: {}, Pitch: {}, Extra flag: {}", speed, pitch, extra_flag);

    if extra_flag != 0 {
        println!("\n--- Reading VOICEINFO extra data ---");
        let lang_id = reader.read_u16().expect("lang_id");
        println!("Lang ID: {}", lang_id);

        // lang_dialect STRING
        let dialect_len = reader.read_u32().expect("dialect_len");
        println!("Dialect string length: {}", dialect_len);
        if dialect_len > 0 {
            // Skip dialect string bytes: (dialect_len + 1) * 2 for UTF-16 + null
            for _ in 0..((dialect_len + 1) * 2) {
                reader.read_u8().expect("dialect byte");
            }
        }

        let gender = reader.read_u16().expect("gender");
        let age = reader.read_u16().expect("age");
        println!("Gender: {}, Age: {}", gender, age);

        // style STRING
        let style_len = reader.read_u32().expect("style_len");
        println!("Style string length: {}", style_len);
        if style_len > 0 {
            // Read the style string (including null terminator)
            let mut style_bytes = Vec::new();
            for _ in 0..((style_len + 1) * 2) {
                style_bytes.push(reader.read_u8().expect("style byte"));
            }
            let utf16: Vec<u16> = style_bytes[..style_len as usize * 2]
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();
            let style = String::from_utf16_lossy(&utf16);
            println!("Style: {:?}", style);
        }
    }

    println!("Position after voice info: {}", reader.position());

    // Next should be BALLOONINFO (if bit 9 is balloon enabled)
    println!("\n--- Attempting to read BALLOONINFO ---");
    let num_lines = reader.read_u8().expect("num_lines");
    let chars_per_line = reader.read_u8().expect("chars_per_line");
    println!("Num lines: {}, Chars per line: {}", num_lines, chars_per_line);

    // Colors are RGBQUAD (4 bytes each: R, G, B, Reserved)
    let fg_r = reader.read_u8().expect("fg_r");
    let fg_g = reader.read_u8().expect("fg_g");
    let fg_b = reader.read_u8().expect("fg_b");
    let _fg_reserved = reader.read_u8().expect("fg_reserved");
    println!("FG color: ({}, {}, {})", fg_r, fg_g, fg_b);

    let bg_r = reader.read_u8().expect("bg_r");
    let bg_g = reader.read_u8().expect("bg_g");
    let bg_b = reader.read_u8().expect("bg_b");
    let _bg_reserved = reader.read_u8().expect("bg_reserved");
    println!("BG color: ({}, {}, {})", bg_r, bg_g, bg_b);

    let border_r = reader.read_u8().expect("border_r");
    let border_g = reader.read_u8().expect("border_g");
    let border_b = reader.read_u8().expect("border_b");
    let _border_reserved = reader.read_u8().expect("border_reserved");
    println!("Border color: ({}, {}, {})", border_r, border_g, border_b);

    // Font name (STRING)
    let font_len = reader.read_u32().expect("font_len");
    println!("Font name length: {}", font_len);
    if font_len > 0 && font_len < 1000 {
        let mut font_bytes = Vec::new();
        for _ in 0..((font_len + 1) * 2) {
            font_bytes.push(reader.read_u8().expect("font byte"));
        }
        let utf16: Vec<u16> = font_bytes[..font_len as usize * 2]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        let font = String::from_utf16_lossy(&utf16);
        println!("Font name: {:?}", font);
    }

    let font_height = reader.read_i32().expect("font_height");
    let font_weight = reader.read_i32().expect("font_weight");
    let italic = reader.read_u8().expect("italic");
    let unknown = reader.read_u8().expect("unknown");
    println!("Font height: {}, weight: {}, italic: {}, unknown: {}", font_height, font_weight, italic, unknown);

    println!("Position after balloon info: {}", reader.position());

    // Palette (count is ULONG, each color is RGBQUAD = 4 bytes)
    println!("\n--- Attempting to read PALETTE ---");
    let palette_count = reader.read_u32().expect("palette_count");
    println!("Palette count: {}", palette_count);
    if palette_count > 0 && palette_count < 1000 {
        for i in 0..palette_count {
            let r = reader.read_u8().expect("r");
            let g = reader.read_u8().expect("g");
            let b = reader.read_u8().expect("b");
            let _reserved = reader.read_u8().expect("reserved");
            if i < 5 {
                println!("  Color {}: ({}, {}, {})", i, r, g, b);
            }
        }
        if palette_count > 5 {
            println!("  ... ({} more colors)", palette_count - 5);
        }
    }

    println!("Position after palette: {}", reader.position());

    // Tray icon flag
    println!("\n--- Attempting to read TRAY ICON ---");
    let has_tray_icon = reader.read_u8().expect("tray_icon_flag");
    println!("Has tray icon: {}", has_tray_icon);

    if has_tray_icon != 0 {
        let mono_size = reader.read_u32().expect("mono_size");
        println!("Mono bitmap size: {}", mono_size);
        // Skip mono bitmap
        for _ in 0..mono_size {
            reader.read_u8().expect("mono byte");
        }
        let color_size = reader.read_u32().expect("color_size");
        println!("Color bitmap size: {}", color_size);
        // Skip color bitmap
        for _ in 0..color_size {
            reader.read_u8().expect("color byte");
        }
    }

    println!("Position after tray icon: {}", reader.position());

    // States
    println!("\n--- Attempting to read STATES ---");
    let state_count = reader.read_u16().expect("state_count");
    println!("State count: {}", state_count);

    println!("Position after reading state count: {}", reader.position());

    // Expected end: character_info.offset + character_info.size = 5246214 + 3581 = 5249795
    let expected_end_approx = 5246214 + 3581;
    println!("Expected end (char info offset + size): {}", expected_end_approx);
}

#[test]
fn test_read_character_info() {
    let mut reader = AcsReader::new(BONZI_ACS);
    let header = reader.read_header().expect("Failed to read header");

    println!("Reading character info from offset {}", header.character_info.offset);
    let char_info = reader
        .read_character_info(header.character_info.offset)
        .expect("Failed to read character info");

    println!("Character info: {:?}", char_info);
    println!("Name: {:?}", char_info.localized_info.first().map(|l| &l.name));
    println!("Dimensions: {}x{}", char_info.width, char_info.height);
}

#[test]
fn test_load_bonzi() {
    let acs = Acs::new(BONZI_ACS.to_vec()).expect("Failed to load Bonzi.acs");
    let info = acs.character_info();

    // Check well-known character info
    assert_eq!(info.name, "Bonzi");
    assert_eq!(info.width, 200);
    assert_eq!(info.height, 160);
}

#[test]
fn test_bonzi_has_animations() {
    let acs = Acs::new(BONZI_ACS.to_vec()).expect("Failed to load Bonzi.acs");
    let names = acs.animation_names();

    // Bonzi should have many animations
    assert!(!names.is_empty(), "Expected animations, got none");

    // Check for some well-known animation names
    let names_lower: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
    assert!(
        names_lower.iter().any(|n| n.contains("greet") || n.contains("wave")),
        "Expected a greeting/wave animation"
    );
}

#[test]
fn test_bonzi_read_animation_info() {
    let mut reader = AcsReader::new(BONZI_ACS);
    let header = reader.read_header().expect("Failed to read header");

    println!("Animation info at offset {}, size {}", header.animation_info.offset, header.animation_info.size);

    // First, let's see the raw bytes at the animation info offset
    reader.seek(header.animation_info.offset as u64);
    print!("First 64 bytes at animation info offset: ");
    for i in 0..64 {
        if i % 16 == 0 { println!(); print!("  {:08x}: ", header.animation_info.offset + i); }
        print!("{:02x} ", reader.read_u8().unwrap_or(0));
    }
    println!();

    // Read animation list
    let anim_list = reader.read_animation_list(&header.animation_info).expect("Failed to read animation list");
    println!("Found {} animations", anim_list.len());

    for (i, entry) in anim_list.iter().take(5).enumerate() {
        println!("  {}: {} at offset {}, size {}", i, entry.name, entry.locator.offset, entry.locator.size);
    }

    // Figure out where the list entries end
    // Count + entries. Each entry has: string (4 + (len+1)*2) + locator (8)
    let mut list_end_pos = header.animation_info.offset + 4; // after count
    for entry in &anim_list {
        // String: 4 byte len + (len+1)*2 bytes
        let name_len = entry.name.len();
        list_end_pos += 4 + (name_len as u32 + 1) * 2 + 8;
    }
    println!("\nEstimated list end position: {}", list_end_pos);
    println!("Animation info block: {} to {}", header.animation_info.offset, header.animation_info.offset + header.animation_info.size);

    // The first animation entry has offset 36. If we add that to list_end_pos:
    if let Some(entry) = anim_list.first() {
        // Try interpreting offset as absolute from file start
        let abs_offset = entry.locator.offset;
        println!("\nTrying absolute offset {} for '{}'", abs_offset, entry.name);
        reader.seek(abs_offset as u64);
        print!("Bytes there: ");
        for _ in 0..20 { print!("{:02x} ", reader.read_u8().unwrap_or(0)); }
        println!();

        // Try offset relative to animation info block start
        let rel_offset = header.animation_info.offset + entry.locator.offset;
        println!("Trying relative offset {} for '{}'", rel_offset, entry.name);
        reader.seek(rel_offset as u64);
        print!("Bytes there: ");
        for _ in 0..20 { print!("{:02x} ", reader.read_u8().unwrap_or(0)); }
        println!();

        // Try reading animation info manually from absolute offset 36
        println!("\nManually reading animation info from offset 36:");
        reader.seek(36);

        let name = reader.read_string().expect("name");
        println!("  Name: {}", name);

        let trans_type = reader.read_u8().expect("trans");
        println!("  Transition type: {}", trans_type);

        let return_anim = reader.read_string().expect("return_anim");
        println!("  Return animation: {:?}", return_anim);

        let frame_count = reader.read_u16().expect("frame_count") as usize;
        println!("  Frame count: {}", frame_count);
        println!("  Current position before frames: {}", reader.position());

        // Try reading first frame
        println!("\n  Reading first frame:");

        // Read 16 bytes to see raw frame data
        let pos = reader.position();
        print!("  Raw bytes: ");
        for _ in 0..32 { print!("{:02x} ", reader.read_u8().unwrap_or(0)); }
        println!();
        reader.seek(pos);

        // Frame structure per spec
        let image_count = reader.read_u16().expect("image_count") as usize;
        println!("  Image count: {}", image_count);

        // Read each image
        for i in 0..image_count {
            let img_index = reader.read_u32().expect("img_index");
            let x = reader.read_i16().expect("x");
            let y = reader.read_i16().expect("y");
            println!("    Image {}: index={}, pos=({}, {})", i, img_index, x, y);
        }

        let sound_index = reader.read_i16().expect("sound_index");
        println!("  Sound index: {}", sound_index);

        let duration = reader.read_u16().expect("duration");
        println!("  Duration: {}", duration);

        let exit_branch = reader.read_i16().expect("exit_branch");
        println!("  Exit branch: {}", exit_branch);

        let branch_count = reader.read_u8().expect("branch_count") as usize;
        println!("  Branch count: {}", branch_count);

        // Skip branches
        for _ in 0..branch_count {
            reader.read_u16().ok(); // frame_index
            reader.read_u16().ok(); // probability
        }

        let overlay_count = reader.read_u8().expect("overlay_count") as usize;
        println!("  Overlay count: {}", overlay_count);

        println!("  Position after first frame: {}", reader.position());
    }
}

#[test]
fn test_bonzi_render_frame() {
    let mut acs = Acs::new(BONZI_ACS.to_vec()).expect("Failed to load Bonzi.acs");

    // Get first animation name (clone to avoid borrow)
    let anim_name = {
        let names = acs.animation_names();
        assert!(!names.is_empty());
        names[0].to_string()
    };

    println!("Testing animation: {}", anim_name);

    // Load animation to ensure it's cached
    let frame_count = {
        let anim = acs.animation(&anim_name).expect("Failed to get animation");
        anim.frames.len()
    };
    println!("Animation has {} frames", frame_count);
    assert!(frame_count > 0, "Animation should have at least one frame");

    // Render the first frame
    let img = acs.render_frame(&anim_name, 0).expect("Failed to render frame 0");
    println!("Rendered frame: {}x{}, {} bytes", img.width, img.height, img.data.len());

    assert_eq!(img.width, 200);
    assert_eq!(img.height, 160);
    assert_eq!(img.data.len(), 200 * 160 * 4); // RGBA
}

// ============ Clippit tests ============

#[test]
fn test_clippit_read_header() {
    let mut reader = AcsReader::new(CLIPPIT_ACS);
    let header = reader.read_header().expect("Failed to read header");

    println!("Header: {:?}", header);
    println!("File size: {} bytes", CLIPPIT_ACS.len());
    println!("Character info at offset: {}, size: {}", header.character_info.offset, header.character_info.size);
    println!("Animation info at offset: {}, size: {}", header.animation_info.offset, header.animation_info.size);
    println!("Image info at offset: {}, size: {}", header.image_info.offset, header.image_info.size);
    println!("Audio info at offset: {}, size: {}", header.audio_info.offset, header.audio_info.size);
}

#[test]
fn test_clippit_character_info_step_by_step() {
    let mut reader = AcsReader::new(CLIPPIT_ACS);
    let header = reader.read_header().expect("Failed to read header");

    println!("File size: {} bytes", CLIPPIT_ACS.len());
    println!("Character info at offset {}, size {}", header.character_info.offset, header.character_info.size);

    // Manually read character info step by step
    reader.seek(header.character_info.offset as u64);

    let minor_version = reader.read_u16().expect("minor_version");
    let major_version = reader.read_u16().expect("major_version");
    println!("Version: {}.{}", major_version, minor_version);

    // Localized info locator
    let loc_offset = reader.read_u32().expect("loc_offset");
    let loc_size = reader.read_u32().expect("loc_size");
    println!("Localized info locator: offset={}, size={}", loc_offset, loc_size);

    // GUID
    let mut guid = [0u8; 16];
    for i in 0..16 {
        guid[i] = reader.read_u8().expect("guid byte");
    }
    println!("GUID: {:02x?}", guid);

    let width = reader.read_u16().expect("width");
    let height = reader.read_u16().expect("height");
    println!("Dimensions: {}x{}", width, height);

    let transparent_color = reader.read_u8().expect("transparent_color");
    println!("Transparent color: {}", transparent_color);

    let flags = reader.read_u32().expect("flags");
    println!("Flags: 0x{:08x}", flags);

    let anim_set_major = reader.read_u16().expect("anim_set_major");
    let anim_set_minor = reader.read_u16().expect("anim_set_minor");
    println!("Anim set version: {}.{}", anim_set_major, anim_set_minor);

    // Check flags for optional sections
    // Bit 5 (not bit 4) controls VOICEINFO presence
    let has_voice = flags & 0x20 != 0;
    println!("Bit 4 (voice output flag): {}", flags & 0x10 != 0);
    println!("Bit 5 (voiceinfo present): {}", has_voice);
    println!("Bit 8: {}", flags & 0x100 != 0);
    println!("Bit 9: {}", flags & 0x200 != 0);

    println!("\nCurrent position: {}", reader.position());

    // Dump raw bytes to see what's there
    let pos = reader.position();
    print!("\nRaw bytes at position {}: ", pos);
    for _ in 0..64 { print!("{:02x} ", reader.read_u8().unwrap_or(0)); }
    println!();
    reader.seek(pos);

    // Only read VOICEINFO if bit 4 is set (Clippit has it disabled)
    if has_voice {
        println!("\n--- Reading VOICEINFO (bit 4 enabled) ---");
        let mut guid1 = [0u8; 16];
        for i in 0..16 { guid1[i] = reader.read_u8().expect("guid1"); }
        println!("TTS Engine GUID: {:02x?}", guid1);

        let mut guid2 = [0u8; 16];
        for i in 0..16 { guid2[i] = reader.read_u8().expect("guid2"); }
        println!("TTS Mode GUID: {:02x?}", guid2);

        let speed = reader.read_u32().expect("speed");
        let pitch = reader.read_u16().expect("pitch");
        let extra_flag = reader.read_u8().expect("extra_flag");
        println!("Speed: {}, Pitch: {}, Extra flag: {}", speed, pitch, extra_flag);

        if extra_flag != 0 {
            println!("--- Reading VOICEINFO extra data ---");
            let lang_id = reader.read_u16().expect("lang_id");
            println!("Lang ID: {}", lang_id);

            let dialect_len = reader.read_u32().expect("dialect_len");
            println!("Dialect string length: {}", dialect_len);
            if dialect_len > 0 {
                for _ in 0..((dialect_len + 1) * 2) {
                    reader.read_u8().expect("dialect byte");
                }
            }

            let gender = reader.read_u16().expect("gender");
            let age = reader.read_u16().expect("age");
            println!("Gender: {}, Age: {}", gender, age);

            let style_len = reader.read_u32().expect("style_len");
            println!("Style string length: {}", style_len);
            if style_len > 0 {
                for _ in 0..((style_len + 1) * 2) {
                    reader.read_u8().expect("style byte");
                }
            }
        }
        println!("Position after voice info: {}", reader.position());
    } else {
        println!("\n--- Skipping VOICEINFO (bit 4 disabled) ---");
    }

    // Next should be BALLOONINFO
    println!("\n--- Attempting to read BALLOONINFO ---");
    let num_lines = reader.read_u8().expect("num_lines");
    let chars_per_line = reader.read_u8().expect("chars_per_line");
    println!("Num lines: {}, Chars per line: {}", num_lines, chars_per_line);

    // Colors are RGBQUAD (4 bytes each)
    let fg_b = reader.read_u8().expect("fg_b");
    let fg_g = reader.read_u8().expect("fg_g");
    let fg_r = reader.read_u8().expect("fg_r");
    let _fg_reserved = reader.read_u8().expect("fg_reserved");
    println!("FG color: ({}, {}, {})", fg_r, fg_g, fg_b);

    let bg_b = reader.read_u8().expect("bg_b");
    let bg_g = reader.read_u8().expect("bg_g");
    let bg_r = reader.read_u8().expect("bg_r");
    let _bg_reserved = reader.read_u8().expect("bg_reserved");
    println!("BG color: ({}, {}, {})", bg_r, bg_g, bg_b);

    let border_b = reader.read_u8().expect("border_b");
    let border_g = reader.read_u8().expect("border_g");
    let border_r = reader.read_u8().expect("border_r");
    let _border_reserved = reader.read_u8().expect("border_reserved");
    println!("Border color: ({}, {}, {})", border_r, border_g, border_b);

    // Font name (STRING)
    let font_len = reader.read_u32().expect("font_len");
    println!("Font name length: {}", font_len);
    if font_len > 0 && font_len < 1000 {
        for _ in 0..((font_len + 1) * 2) {
            reader.read_u8().expect("font byte");
        }
    }

    let font_height = reader.read_i32().expect("font_height");
    let font_weight = reader.read_i32().expect("font_weight");
    let italic = reader.read_u8().expect("italic");
    let unknown = reader.read_u8().expect("unknown");
    println!("Font height: {}, weight: {}, italic: {}, unknown: {}", font_height, font_weight, italic, unknown);

    println!("Position after balloon info: {}", reader.position());

    // Palette
    println!("\n--- Attempting to read PALETTE ---");
    let palette_count = reader.read_u32().expect("palette_count");
    println!("Palette count: {}", palette_count);
    if palette_count > 0 && palette_count < 1000 {
        for _ in 0..palette_count {
            reader.read_u8().expect("b");
            reader.read_u8().expect("g");
            reader.read_u8().expect("r");
            reader.read_u8().expect("reserved");
        }
    }

    println!("Position after palette: {}", reader.position());

    // Tray icon flag
    println!("\n--- Attempting to read TRAY ICON ---");
    let has_tray_icon = reader.read_u8().expect("tray_icon_flag");
    println!("Has tray icon: {}", has_tray_icon);

    if has_tray_icon != 0 {
        let mono_size = reader.read_u32().expect("mono_size");
        println!("Mono bitmap size: {}", mono_size);
        for _ in 0..mono_size { reader.read_u8().expect("mono byte"); }
        let color_size = reader.read_u32().expect("color_size");
        println!("Color bitmap size: {}", color_size);
        for _ in 0..color_size { reader.read_u8().expect("color byte"); }
    }

    println!("Position after tray icon: {}", reader.position());

    // States
    println!("\n--- Attempting to read STATES ---");
    let state_count = reader.read_u16().expect("state_count");
    println!("State count: {}", state_count);

    println!("Position after reading state count: {}", reader.position());
}

#[test]
fn test_clippit_read_character_info() {
    let mut reader = AcsReader::new(CLIPPIT_ACS);
    let header = reader.read_header().expect("Failed to read header");

    println!("Reading character info from offset {}", header.character_info.offset);
    let char_info = reader
        .read_character_info(header.character_info.offset)
        .expect("Failed to read character info");

    println!("Character info: {:?}", char_info);
    println!("Name: {:?}", char_info.localized_info.first().map(|l| &l.name));
    println!("Dimensions: {}x{}", char_info.width, char_info.height);
}

#[test]
fn test_load_clippit() {
    let acs = Acs::new(CLIPPIT_ACS.to_vec()).expect("Failed to load clippit.acs");
    let info = acs.character_info();

    println!("Clippit name: {:?}", info.name);
    println!("Clippit dimensions: {}x{}", info.width, info.height);
    println!("Clippit description: {:?}", info.description);
}

#[test]
fn test_clippit_render_frame() {
    let mut acs = Acs::new(CLIPPIT_ACS.to_vec()).expect("Failed to load clippit.acs");

    let anim_name = {
        let names = acs.animation_names();
        assert!(!names.is_empty());
        println!("Clippit animations: {:?}", names);
        names[0].to_string()
    };

    println!("Testing animation: {}", anim_name);

    let frame_count = {
        let anim = acs.animation(&anim_name).expect("Failed to get animation");
        anim.frames.len()
    };
    println!("Animation has {} frames", frame_count);

    let img = acs.render_frame(&anim_name, 0).expect("Failed to render frame 0");
    println!("Rendered frame: {}x{}, {} bytes", img.width, img.height, img.data.len());
}
