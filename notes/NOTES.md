# Microsoft Agent Character File (.ACS) Format Notes

This document describes the structure of Microsoft Agent Character files (version 2), based on reverse engineering of `Bonzi.acs` and the unofficial specification by Remy Lebeau.

## File Overview

| Property | Value |
|----------|-------|
| File Extension | `.acs` |
| Magic Signature | `0xABCDABC3` (little-endian: `C3 AB CD AB`) |
| Format Version | 2 (version 1 also exists but is undocumented) |
| Encoding | Little-endian throughout |
| String Encoding | UTF-16LE with null terminator |

### Related Formats

| Format | Extension | Magic | Description |
|--------|-----------|-------|-------------|
| ACS | `.acs` | `0xABCDABC3` | Single file containing character + all animations |
| ACF | `.acf` | `0xABCDABC4` | Character definition only (for HTTP delivery) |
| ACA | `.aca` | (version bytes) | Single animation file (paired with ACF) |

---

## Primitive Data Types

```c
typedef uint8_t   BYTE;      // 8-bit unsigned
typedef int16_t   SHORT;     // 16-bit signed
typedef uint16_t  USHORT;    // 16-bit unsigned
typedef int16_t   WCHAR;     // 16-bit signed (UTF-16LE character)
typedef int32_t   LONG;      // 32-bit signed
typedef uint32_t  ULONG;     // 32-bit unsigned
typedef uint16_t  LANGID;    // Windows language identifier
typedef uint8_t   BOOL;      // 0x00 = false, 0x01 = true
```

---

## Core Structures

### ACSLOCATOR

Points to data elsewhere in the file. Used extensively throughout the format.

```c
struct ACSLOCATOR {
    ULONG offset;    // 0-based byte offset from start of file
    ULONG size;      // Size of data block in bytes
};
```

**Notes:**
- If `offset == 0` and `size == 0`, the locator points to nothing (null)
- Always 8 bytes total

---

### STRING

Variable-length UTF-16LE string.

```c
struct STRING {
    ULONG length;           // Character count (NOT including null terminator)
    WCHAR chars[length];    // UTF-16LE characters (if length > 0)
    WCHAR nullTerminator;   // Always present when length > 0
};
```

**Notes:**
- Byte size = 4 + ((length + 1) * 2) when length > 0, otherwise just 4
- The length field does NOT include the null terminator
- The null terminator IS stored in the file (2 bytes) after the characters
- Empty strings have `length = 0` with no character data and no null terminator

---

### GUID

Standard 128-bit Windows GUID.

```c
struct GUID {
    ULONG  Data1;       // 4 bytes
    USHORT Data2;       // 2 bytes
    USHORT Data3;       // 2 bytes
    BYTE   Data4[8];    // 8 bytes
};
```

**Notes:**
- Total size: 16 bytes
- Used for character unique ID and TTS engine/mode IDs

---

### RGBQUAD

Standard Windows color structure.

```c
struct RGBQUAD {
    BYTE blue;
    BYTE green;
    BYTE red;
    BYTE reserved;    // Always 0x00
};
```

---

### DATABLOCK

Generic variable-length data container.

```c
struct DATABLOCK {
    ULONG size;         // Size in bytes
    BYTE  data[size];   // Raw data
};
```

---

## File Structure

### ACSHEADER

The file begins with this header structure.

```c
struct ACSHEADER {
    ULONG      signature;        // Always 0xABCDABC3
    ACSLOCATOR characterInfo;    // -> ACSCHARACTERINFO
    ACSLOCATOR animationInfo;    // -> ACSANIMATIONINFO list
    ACSLOCATOR imageInfo;        // -> ACSIMAGEINFO list
    ACSLOCATOR audioInfo;        // -> ACSAUDIOINFO list
};
```

**Notes:**
- Total size: 36 bytes (4 + 8 + 8 + 8 + 8)
- All locators point to absolute file offsets

**Example from Bonzi.acs:**
```
Offset 0x00: C3 AB CD AB     (signature)
Offset 0x04: 06 0D 50 00     (characterInfo.offset = 0x500D06)
Offset 0x08: FD 0D 00 00     (characterInfo.size = 0x0DFD = 3581 bytes)
Offset 0x0C: CA BD 4F 00     (animationInfo.offset = 0x4FBDCA)
Offset 0x10: 70 13 00 00     (animationInfo.size = 0x1370 = 4976 bytes)
...
```

---

### ACSCHARACTERINFO

Describes the character's properties and appearance settings.

```c
struct ACSCHARACTERINFO {
    USHORT     minorVersion;      // Format minor version
    USHORT     majorVersion;      // Format major version (usually 2)
    ACSLOCATOR localizedInfo;     // -> LOCALIZEDINFO list
    GUID       characterGuid;     // Unique character identifier
    USHORT     width;             // Character width in pixels
    USHORT     height;            // Character height in pixels
    BYTE       transparentIndex;  // Palette index for transparency
    ULONG      flags;             // Feature flags (see below)
    USHORT     animSetMajor;      // Animation set major version
    USHORT     animSetMinor;      // Animation set minor version
    VOICEINFO  voiceInfo;         // TTS voice settings
    BALLOONINFO balloonInfo;      // Word balloon settings
    ULONG      paletteCount;      // Number of palette entries (usually 256)
    RGBQUAD    palette[paletteCount]; // Color palette
    BYTE       hasTrayIcon;       // 0x00 = no, 0x01 = yes
    TRAYICON   trayIcon;          // (if hasTrayIcon == 0x01)
    STATEINFO_LIST states;        // Animation state mappings
};
```

**Flags (bitmask):**
```c
#define FLAG_VOICE_OUTPUT_ENABLED    0x00000001
#define FLAG_BALLOON_AUTO_HIDE       0x00000002
#define FLAG_BALLOON_AUTO_PACE       0x00000004
#define FLAG_STD_ANIM_SET_SUPPORT    0x00000020
```

**Example from Bonzi.acs:**
- Version: 2.1
- Dimensions: 200 x 160 pixels (0xC8 x 0xA0)
- Transparent color index: 0xFE (254)

---

### LOCALIZEDINFO

Character info localized to a specific language.

```c
struct LOCALIZEDINFO_LIST {
    USHORT count;                   // Number of locales
    LOCALIZEDINFO entries[count];
};

struct LOCALIZEDINFO {
    LANGID langId;          // Windows language ID
    STRING name;            // Character name (e.g., "Bonzi")
    STRING description;     // Character description
    STRING extraData;       // Additional data (often version string)
};
```

---

### VOICEINFO

Text-to-speech voice settings.

```c
struct VOICEINFO {
    GUID   ttsEngineId;     // TTS engine CLSID
    GUID   ttsModeId;       // TTS mode GUID
    ULONG  speed;           // Speech speed
    USHORT pitch;           // Voice pitch
    BYTE   hasExtraData;    // 0x00 = no, 0x01 = yes
    // If hasExtraData == 0x01:
    LANGID langId;          // Language ID
    STRING dialect;         // Language dialect
    USHORT gender;          // 0 = neutral, 1 = female, 2 = male
    USHORT age;             // Voice age
    STRING style;           // Voice style
};
```

---

### BALLOONINFO

Word balloon (speech bubble) settings.

```c
struct BALLOONINFO {
    BYTE    numLines;         // Number of text lines
    BYTE    charsPerLine;     // Characters per line
    RGBQUAD foreground;       // Text color
    RGBQUAD background;       // Background color
    RGBQUAD border;           // Border color
    STRING  fontName;         // Font family (e.g., "MS Sans Serif")
    LONG    fontHeight;       // Font height in logical units
    LONG    fontWeight;       // 0-1000 (400 = normal, 700 = bold)
    BYTE    italic;           // 0x00 = no, 0x01 = yes
    BYTE    unknown;          // Purpose unknown
};
```

---

### PALETTECOLOR / RGBQUAD

Color palette entry. The palette is prefixed with a count in ACSCHARACTERINFO.

```c
struct RGBQUAD {
    BYTE blue;
    BYTE green;
    BYTE red;
    BYTE reserved;    // Always 0x00
};

// In ACSCHARACTERINFO:
ULONG paletteCount;              // Usually 256
RGBQUAD palette[paletteCount];
```

**Notes:**
- Palette is stored in ACSCHARACTERINFO with a ULONG count prefix
- The count is typically 256 for 8-bit indexed color
- All images in the character share this single palette
- Entry at `transparentIndex` is the transparent color
- Color order is BGR (blue, green, red) - standard Windows RGBQUAD format

---

## Animation Structures

### ACSANIMATIONINFO List

```c
struct ACSANIMATIONINFO_LIST {
    ULONG count;                        // Number of animations
    ACSANIMATIONINFO entries[count];
};

struct ACSANIMATIONINFO {
    STRING     name;            // Animation name (e.g., "DECLINE")
    ACSLOCATOR animationData;   // -> Animation frame data
};
```

---

### Animation Data (at ACSLOCATOR target)

```c
struct ANIMATIONDATA {
    STRING         name;              // Animation name (uppercase)
    BYTE           transitionType;    // 0=return, 1=exit branches, 2=none
    STRING         returnAnimation;   // Name of animation to return to
    USHORT         frameCount;
    ACSFRAMEINFO   frames[frameCount];
};
```

**Transition Types:**
- `0` - Return animation: plays `returnAnimation` when complete
- `1` - Exit branches: uses exit branching for transition
- `2` - No transition: simply ends

---

### ACSFRAMEINFO

Individual animation frame.

```c
struct ACSFRAMEINFO {
    USHORT          imageCount;
    ACSFRAMEIMAGE   images[imageCount];     // Composited images
    USHORT          soundIndex;             // 0xFFFF = no sound
    USHORT          duration;               // Frame duration in 1/100 seconds
    SHORT           exitFrame;              // Exit branch target (-1 = none)
    BYTE            branchCount;
    BRANCHINFO      branches[branchCount];  // Random branches
    BYTE            overlayCount;
    ACSOVERLAYINFO  overlays[overlayCount]; // Mouth overlays for speech
};
```

**Notes:**
- `soundIndex` of `0xFFFF` means no sound plays
- `duration` of 10 = 100ms (10 * 10ms)
- `exitFrame` of -1 (`0xFFFF` as signed) means no exit branch

---

### ACSFRAMEIMAGE

Reference to an image for compositing.

```c
struct ACSFRAMEIMAGE {
    USHORT imageIndex;    // Index into ACSIMAGEINFO list
    SHORT  offsetX;       // X offset for compositing
    SHORT  offsetY;       // Y offset for compositing
};
```

---

### BRANCHINFO

Random branch for varied playback.

```c
struct BRANCHINFO {
    USHORT frameIndex;    // Target frame (0-based)
    USHORT probability;   // Percentage chance (0-100)
};
```

**Notes:**
- Up to 3 branches per frame
- If random roll hits probability, jump to `frameIndex`
- If no branch hits, advance to next sequential frame
- Used for randomized idle behaviors and loops

---

### ACSOVERLAYINFO

Mouth position overlay for lip-sync.

```c
struct ACSOVERLAYINFO {
    BYTE   overlayType;     // Mouth shape/position type
    BOOL   replaceFlag;     // Replace or composite
    USHORT imageIndex;      // Index into ACSIMAGEINFO list
    SHORT  offsetX;         // X offset
    SHORT  offsetY;         // Y offset
};
```

---

## Image Structures

### ACSIMAGEINFO List

```c
struct ACSIMAGEINFO_LIST {
    ULONG count;                    // Number of images
    ACSIMAGEINFO entries[count];
};

struct ACSIMAGEINFO {
    ACSLOCATOR imageData;    // -> Image data
    ULONG      checksum;     // Data integrity checksum
};
```

---

### Image Data (at ACSLOCATOR target)

```c
struct IMAGEDATA {
    BYTE   unknown;              // Purpose unknown (usually 0x00)
    USHORT width;                // Image width in pixels
    USHORT height;               // Image height in pixels
    BYTE   isCompressed;         // 0x00 = raw, 0x01 = compressed

    // If isCompressed == 0x00:
    DATABLOCK rawPixels;         // Raw 8-bit indexed pixels

    // If isCompressed == 0x01:
    COMPRESSED compressedPixels; // ZLIB-compressed pixels

    // Region data (for hit testing / transparency regions)
    ULONG  regionCompressedSize;
    ULONG  regionUncompressedSize;
    BYTE   regionData[regionCompressedSize];
};
```

**Notes:**
- Images are 8-bit indexed color (256 color palette)
- Pixel data is stored bottom-up (standard Windows DIB format)
- Rows are DWORD-aligned (padded to 4-byte boundary)
- Use the `transparentIndex` from ACSCHARACTERINFO for transparency

---

### COMPRESSED

ZLIB-compressed data wrapper.

```c
struct COMPRESSED {
    ULONG compressedSize;
    ULONG uncompressedSize;
    BYTE  data[compressedSize];   // ZLIB/DEFLATE compressed
};
```

**Notes:**
- Uses standard ZLIB deflate compression
- Decompress to get raw pixel data
- Common compression ratio: 2:1 to 5:1

---

## Audio Structures

### ACSAUDIOINFO List

```c
struct ACSAUDIOINFO_LIST {
    ULONG count;                    // Number of sounds (0x16 = 22 in Bonzi)
    ACSAUDIOINFO entries[count];
};

struct ACSAUDIOINFO {
    ACSLOCATOR audioData;    // -> Audio data
    ULONG      checksum;     // Data integrity checksum
};
```

---

### Audio Data (at ACSLOCATOR target)

The audio data is typically a complete WAV file or raw PCM audio data.

```c
struct AUDIODATA {
    // Usually a complete WAV file including RIFF header
    // Or raw PCM data with implied format
    BYTE data[];
};
```

**Notes:**
- Audio format varies; often 8-bit or 16-bit PCM
- Sample rates typically 11025Hz, 22050Hz, or 44100Hz
- Some files use compressed audio (e.g., ADPCM)

---

## State Structures

### STATEINFO

Maps character states to animations.

```c
struct STATEINFO_LIST {
    USHORT count;
    STATEINFO entries[count];
};

struct STATEINFO {
    STRING stateName;       // State name (e.g., "IDLINGLEVEL1")
    USHORT animationCount;
    STRING animations[];    // Animation names for this state
};
```

**Common States:**
- `SHOWING` - Character appearing
- `HIDING` - Character disappearing
- `SPEAKING` - Character talking
- `IDLINGLEVEL1` - Primary idle animations
- `IDLINGLEVEL2` - Secondary idle animations (less common)
- `IDLINGLEVEL3` - Tertiary idle animations (rare)
- `MOVINGLEFT`, `MOVINGRIGHT`, `MOVINGUP`, `MOVINGDOWN` - Movement

---

## Tray Icon Structure

### TRAYICON

System tray icon (if present). Consists of two DATABLOCKs containing the icon bitmap and transparency mask.

```c
struct TRAYICON {
    DATABLOCK iconData;     // Icon bitmap (includes BITMAPINFOHEADER + pixel data)
    DATABLOCK maskData;     // Icon mask (1-bit transparency mask)
};
```

**Notes:**
- The iconData contains a full Windows bitmap structure (BITMAPINFOHEADER followed by pixel data)
- The maskData contains a 1-bit per pixel transparency mask
- Typical icons are 16x16 or 32x32 pixels
- In Bonzi.acs: iconData is 112 bytes, maskData is 232 bytes

---

## Bonzi.acs Specific Information

### File Statistics
- Total size: 5,249,795 bytes (~5 MB)
- CharacterInfo: offset 0x500D06, size 3581 bytes
- AnimationInfo: offset 0x4FBDCA, size 4976 bytes
- ImageInfo: offset 0x4FD13A, size 15040 bytes
- AudioInfo: offset 0x500BFA, size 268 bytes

### Character Details
- Name: "Bonzi"
- Description: "Hi, my name is Bonzi! I am your interactive friend and traveling companion on the Internet! I can surf, talk, joke, sing, browse, and search the Internet with you!"
- Version: 3.0.7
- Dimensions: 200 x 160 pixels

### Animations Found (partial list)
```
DECLINE, IDLE1_9, EXPLAIN, IDLE1_1, ALERT, CONFUSED,
CONGRATULATE, DONTRECOGNIZE, ACKNOWLEDGE, GESTUREDOWN,
GESTURELEFT, GESTURERIGHT, GESTUREUP, SHOW, HIDE,
MOVELEFT, MOVERIGHT, MOVEUP, MOVEDOWN, SPEAKING,
RESTPOSE, IDLINGLEVEL1, IDLINGLEVEL2, IDLINGLEVEL3,
IDLE1_1 through IDLE1_26, IDLE3_1, IDLE3_2, ...
```

### Audio Count
22 sound effects (index 0-21)

## References

- [Archive Team Wiki - Microsoft Agent character](http://justsolve.archiveteam.org/wiki/Microsoft_Agent_character)
- [LeBeau Software MSAgent Specification](http://lebeausoftware.org/downloadfile.aspx?ID=25001fc7-18e9-49a4-90dc-21e8ff46aa1d) (PDF)
- [MSAgent Specification (HTML mirror)](https://uploads.s.zeid.me/ms-agent-format-spec.html)
- [Agentpedia - Agent Character File](https://agentpedia.tmafe.com/wiki/Agent_Character_File_(file_format))
- [Microsoft Learn - Creating Animations](https://learn.microsoft.com/en-us/windows/win32/lwef/creating-animations)
- [BITMAPINFOHEADER Structure](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapinfoheader)
