#pragma description Microsoft Agent Character File (ACS) v2
#pragma magic [ C3 AB CD AB ] @ 0x00
#pragma endian little

import std.mem;
import std.string;

// =============================================================================
// Primitive Types
// =============================================================================

using BYTE = u8;
using WCHAR = char16;
using SHORT = s16;
using USHORT = u16;
using LONG = s32;
using ULONG = u32;
using LANGID = u16;
using BOOL = u8;

// =============================================================================
// Basic Structures
// =============================================================================

struct ACSLOCATOR {
    ULONG offset;   // 0-based byte offset from start of file
    ULONG size;     // Size of data in bytes
} [[static, format("format_locator")]];

fn format_locator(ACSLOCATOR loc) {
    if (loc.offset == 0 && loc.size == 0)
        return "NULL";
    return std::format("0x{:X} ({} bytes)", loc.offset, loc.size);
};

struct GUID {
    ULONG  Data1;
    USHORT Data2;
    USHORT Data3;
    BYTE   Data4[8];
} [[static, format("format_guid")]];

fn format_guid(GUID g) {
    return std::format("{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        g.Data1, g.Data2, g.Data3,
        g.Data4[0], g.Data4[1], g.Data4[2], g.Data4[3],
        g.Data4[4], g.Data4[5], g.Data4[6], g.Data4[7]);
};

struct RGBQUAD {
    BYTE blue;
    BYTE green;
    BYTE red;
    BYTE reserved;
} [[static, color(std::format("{:02X}{:02X}{:02X}", this.red, this.green, this.blue))]];

struct DATABLOCK {
    ULONG size;
    BYTE  data[size];
};

// String with length prefix (UTF-16LE)
// Length does NOT include null terminator, but null IS stored in file
struct STRING {
    ULONG length;   // Character count (NOT including null terminator)
    if (length > 0) {
        WCHAR chars[length];
        WCHAR nullTerminator;   // Always present when length > 0
    }
} [[format("format_string")]];

fn format_string(STRING s) {
    if (s.length == 0)
        return "<empty>";
    return std::string::to_string(s.chars);
};

// =============================================================================
// Voice and Balloon Info
// =============================================================================

struct VOICEINFO {
    GUID   ttsEngineId;
    GUID   ttsModeId;
    ULONG  speed;
    USHORT pitch;
    BOOL   hasExtraData;

    if (hasExtraData == 0x01) {
        LANGID langId;
        STRING dialect;
        USHORT gender;      // 0=neutral, 1=female, 2=male
        USHORT age;
        STRING style;
    }
};

struct BALLOONINFO {
    BYTE    numLines;
    BYTE    charsPerLine;
    RGBQUAD foreground;
    RGBQUAD background;
    RGBQUAD border;
    STRING  fontName;
    LONG    fontHeight;
    LONG    fontWeight;     // 0-1000, 400=normal, 700=bold
    BYTE    italic;
    BYTE    unknown;
};

// =============================================================================
// Animation Structures
// =============================================================================

struct BRANCHINFO {
    USHORT frameIndex;      // Target frame (0-based)
    USHORT probability;     // Percentage (0-100)
} [[format("format_branch")]];

fn format_branch(BRANCHINFO b) {
    return std::format("{}% -> frame {}", b.probability, b.frameIndex);
};

struct ACSFRAMEIMAGE {
    USHORT imageIndex;
    SHORT  offsetX;
    SHORT  offsetY;
} [[format("format_frameimage")]];

fn format_frameimage(ACSFRAMEIMAGE fi) {
    return std::format("img[{}] @ ({}, {})", fi.imageIndex, fi.offsetX, fi.offsetY);
};

struct ACSOVERLAYINFO {
    BYTE   overlayType;
    BOOL   replaceFlag;
    USHORT imageIndex;
    SHORT  offsetX;
    SHORT  offsetY;
};

struct ACSFRAMEINFO {
    USHORT imageCount;
    ACSFRAMEIMAGE images[imageCount];
    USHORT soundIndex;      // 0xFFFF = no sound
    USHORT duration;        // In 1/100 seconds
    SHORT  exitFrame;       // -1 (0xFFFF) = no exit branch
    BYTE   branchCount;
    BRANCHINFO branches[branchCount];
    BYTE   overlayCount;
    ACSOVERLAYINFO overlays[overlayCount];
} [[format("format_frame")]];

fn format_frame(ACSFRAMEINFO f) {
    str snd = f.soundIndex == 0xFFFF ? "none" : std::format("{}", f.soundIndex);
    return std::format("{}ms, {} imgs, snd={}", f.duration * 10, f.imageCount, snd);
};

struct ANIMATIONDATA {
    STRING name;
    BYTE   transitionType;  // 0=return anim, 1=exit branches, 2=none
    STRING returnAnimation;
    USHORT frameCount;
    ACSFRAMEINFO frames[frameCount];
};

struct ACSANIMATIONINFO {
    STRING     name;
    ACSLOCATOR animationData;
} [[format("format_animinfo")]];

fn format_animinfo(ACSANIMATIONINFO a) {
    return format_string(a.name);
};

struct ACSANIMATIONINFO_LIST {
    ULONG count;
    ACSANIMATIONINFO entries[count];
};

// =============================================================================
// Image Structures
// =============================================================================

struct ACSIMAGEINFO {
    ACSLOCATOR imageData;
    ULONG      checksum;
} [[format("format_imginfo")]];

fn format_imginfo(ACSIMAGEINFO i) {
    return std::format("@ 0x{:X}, chk=0x{:08X}", i.imageData.offset, i.checksum);
};

struct ACSIMAGEINFO_LIST {
    ULONG count;
    ACSIMAGEINFO entries[count];
};

// Image data structure (parsed at imageData.offset)
struct IMAGEDATA {
    BYTE   unknown;
    USHORT width;
    USHORT height;
    BYTE   isCompressed;

    ULONG dataSize;
    BYTE  pixelData[dataSize];

    // Region data for hit testing
    ULONG regionCompressedSize;
    ULONG regionUncompressedSize;
    if (regionCompressedSize > 0) {
        BYTE regionData[regionCompressedSize];
    }
} [[format("format_imagedata")]];

fn format_imagedata(IMAGEDATA img) {
    str comp = img.isCompressed ? "compressed" : "raw";
    return std::format("{}x{} ({})", img.width, img.height, comp);
};

// =============================================================================
// Audio Structures
// =============================================================================

struct ACSAUDIOINFO {
    ACSLOCATOR audioData;
    ULONG      checksum;
} [[format("format_audioinfo")]];

fn format_audioinfo(ACSAUDIOINFO a) {
    return std::format("@ 0x{:X}, {} bytes", a.audioData.offset, a.audioData.size);
};

struct ACSAUDIOINFO_LIST {
    ULONG count;
    ACSAUDIOINFO entries[count];
};

// =============================================================================
// Localized Info
// =============================================================================

struct LOCALIZEDINFO {
    LANGID langId;
    STRING name;
    STRING description;
    STRING extraData;
} [[format("format_locinfo")]];

fn format_locinfo(LOCALIZEDINFO li) {
    return std::format("lang=0x{:04X}: {}", li.langId, format_string(li.name));
};

struct LOCALIZEDINFO_LIST {
    USHORT count;
    LOCALIZEDINFO entries[count];
};

// =============================================================================
// State Info
// =============================================================================

struct STATEANIMATION {
    STRING animationName;
};

struct STATEINFO {
    STRING stateName;
    USHORT animationCount;
    STATEANIMATION animations[animationCount];
} [[format("format_stateinfo")]];

fn format_stateinfo(STATEINFO s) {
    return std::format("{} ({} anims)", format_string(s.stateName), s.animationCount);
};

struct STATEINFO_LIST {
    USHORT count;
    STATEINFO entries[count];
};

// =============================================================================
// Tray Icon
// =============================================================================

// TRAYICON is simply two DATABLOCKs: icon bitmap data and mask data
struct TRAYICON {
    DATABLOCK iconData;     // Icon bitmap (includes BITMAPINFOHEADER + pixels)
    DATABLOCK maskData;     // Icon mask (1-bit transparency mask)
} [[format("format_trayicon")]];

fn format_trayicon(TRAYICON t) {
    return std::format("icon={} bytes, mask={} bytes", t.iconData.size, t.maskData.size);
};

// =============================================================================
// Character Info
// =============================================================================

bitfield CharacterFlags {
    voiceOutputEnabled : 1;
    balloonAutoHide : 1;
    balloonAutoPace : 1;
    padding1 : 2;
    stdAnimSetSupport : 1;
    padding2 : 26;
};

struct ACSCHARACTERINFO {
    USHORT         minorVersion;
    USHORT         majorVersion;
    ACSLOCATOR     localizedInfo;
    GUID           characterGuid;
    USHORT         width;
    USHORT         height;
    BYTE           transparentIndex;
    CharacterFlags flags;
    USHORT         animSetMajor;
    USHORT         animSetMinor;
    VOICEINFO      voiceInfo;
    BALLOONINFO    balloonInfo;
    ULONG          paletteCount;           // Usually 256
    RGBQUAD        palette[paletteCount];
    BOOL           hasTrayIcon;
    if (hasTrayIcon == 0x01) {
        TRAYICON   trayIcon;
    }
    STATEINFO_LIST states;
};

// =============================================================================
// File Header
// =============================================================================

struct ACSHEADER {
    ULONG      signature;       // 0xABCDABC3
    ACSLOCATOR characterInfo;
    ACSLOCATOR animationInfo;
    ACSLOCATOR imageInfo;
    ACSLOCATOR audioInfo;
} [[static]];

// =============================================================================
// Main Pattern Entry Point
// =============================================================================

ACSHEADER header @ 0x00;

// Parse the lists at their respective offsets
ACSANIMATIONINFO_LIST animationList @ header.animationInfo.offset;
ACSIMAGEINFO_LIST imageList @ header.imageInfo.offset;
ACSAUDIOINFO_LIST audioList @ header.audioInfo.offset;
ACSCHARACTERINFO characterInfo @ header.characterInfo.offset;

// Parse localized info from character info
LOCALIZEDINFO_LIST localizedList @ characterInfo.localizedInfo.offset;

// =============================================================================
// Optional: Parse individual animation data
// Uncomment specific animations to examine their frame data
// =============================================================================

// Example: Parse first animation's frame data
// ANIMATIONDATA anim0 @ animationList.entries[0].animationData.offset;

// Example: Parse specific image data
// IMAGEDATA img0 @ imageList.entries[0].imageData.offset;
