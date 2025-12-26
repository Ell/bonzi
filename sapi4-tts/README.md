# SAPI4 TTS

Text-to-speech using Microsoft Speech API 4.0 (SAPI4), featuring the classic "Microsoft Sam" voice from Windows 2000.

## Overview

This package provides a Rust CLI binary (`sapi4-rs`) that interfaces with SAPI4 TTS, running on Linux via Wine in a Docker container.

## Quick Start

```bash
# Build the Docker image
docker compose build

# List available voices
docker run --rm sapi4-tts-sapi4 list

# Synthesize speech to a WAV file
docker run --rm -v $(pwd)/output:/output sapi4-tts-sapi4 \
    speak --text "Hello world" --output /output/hello.wav
```

## Usage Examples

### Save to WAV file

```bash
# Create output directory
mkdir -p output

# Synthesize to file
docker run --rm -v $(pwd)/output:/output sapi4-tts-sapi4 \
    speak --text "Hello world" --output /output/hello.wav

# With specific voice
docker run --rm -v $(pwd)/output:/output sapi4-tts-sapi4 \
    speak --text "Hello world" --output /output/hello.wav --voice "Adult Male #1"
```

### Pipe to mpv for immediate playback

```bash
# Play directly (suppress status messages)
docker run --rm sapi4-tts-sapi4 speak --text "Hello world" --stdout 2>/dev/null | mpv -

# Play with status messages visible
docker run --rm sapi4-tts-sapi4 speak --text "Hello world" --stdout 2>&1 | mpv -
```

### Pipe to ffmpeg for format conversion

```bash
# Convert to MP3
docker run --rm sapi4-tts-sapi4 speak --text "Hello world" --stdout 2>/dev/null | \
    ffmpeg -i pipe:0 -y output.mp3

# Convert to OGG
docker run --rm sapi4-tts-sapi4 speak --text "Hello world" --stdout 2>/dev/null | \
    ffmpeg -i pipe:0 -y output.ogg

# Resample to 44.1kHz stereo
docker run --rm sapi4-tts-sapi4 speak --text "Hello world" --stdout 2>/dev/null | \
    ffmpeg -i pipe:0 -ar 44100 -ac 2 -y output.wav
```

### Pipe to ffprobe for audio analysis

```bash
docker run --rm sapi4-tts-sapi4 speak --text "Hello world" --stdout 2>/dev/null | \
    ffprobe -i pipe:0
```

### Pipe to sox for audio processing

```bash
# Add reverb effect
docker run --rm sapi4-tts-sapi4 speak --text "Hello world" --stdout 2>/dev/null | \
    sox -t wav - -t wav output.wav reverb

# Speed up 1.5x
docker run --rm sapi4-tts-sapi4 speak --text "Hello world" --stdout 2>/dev/null | \
    sox -t wav - -t wav output.wav tempo 1.5
```

### Shell function for easy access

Add to your `.bashrc` or `.zshrc`:

```bash
sapi4-say() {
    docker run --rm sapi4-tts-sapi4 speak --text "$1" --stdout 2>/dev/null | mpv --no-terminal -
}

# Usage: sapi4-say "Hello world"
```

### Batch processing

```bash
# Process multiple lines from a file
while IFS= read -r line; do
    docker run --rm sapi4-tts-sapi4 speak --text "$line" --stdout 2>/dev/null >> combined.wav
done < input.txt
```

## CLI Reference

```
sapi4-rs - SAPI4 Text-to-Speech CLI

USAGE:
    sapi4-rs.exe <COMMAND>

COMMANDS:
    list   List available SAPI4 voices
    speak  Synthesize text to a WAV file or stdout

OPTIONS:
    -h, --help     Print help
    -V, --version  Print version

SPEAK OPTIONS:
    -t, --text <TEXT>      Text to speak
    -o, --output <FILE>    Output WAV file path (omit if using --stdout)
        --stdout           Output WAV data to stdout (for piping)
    -v, --voice <NAME>     Voice name (partial match)
        --lang-id <ID>     Language ID (e.g., 1033 for English US)
        --dialect <TEXT>   Language dialect (partial match)
        --gender <NUM>     Gender: 0=neutral, 1=female, 2=male
        --age <NUM>        Speaker age
        --style <TEXT>     Voice style (partial match)
        --speed <SPEED>    Speech speed (engine-dependent)
        --pitch <PITCH>    Speech pitch (0-65535)
    -g, --gain <GAIN>      Audio gain/volume multiplier [default: 4.0]
```

### Voice Selection

You can select voices using multiple criteria that match ACS file voice parameters:

```bash
# Select by name (default behavior)
docker run --rm sapi4-tts-sapi4 speak --text "Hello" --voice "Peter" --stdout

# Select by gender (1=female, 2=male)
docker run --rm sapi4-tts-sapi4 speak --text "Hello" --gender 1 --stdout

# Select by language ID (1033 = English US)
docker run --rm sapi4-tts-sapi4 speak --text "Hello" --lang-id 1033 --stdout

# Combine multiple criteria
docker run --rm sapi4-tts-sapi4 speak --text "Hello" --gender 1 --lang-id 1033 --stdout

# If no voice criteria specified, defaults to "Adult Male #1"
docker run --rm sapi4-tts-sapi4 speak --text "Hello" --stdout
```

Common language IDs:
- 1033: English (US)
- 1041: Japanese
- 1031: German
- 1036: French

## Available Voices

The Docker image comes pre-installed with TruVoice SAPI4 voices:

| Voice Name | Speaker | Gender |
|------------|---------|--------|
| Adult Male #1, American English (TruVoice) | Peter | Male |
| Adult Male #2, American English (TruVoice) | | Male |
| Adult Female #1, American English (TruVoice) | | Female |
| ... and more | | |

Run `docker run --rm sapi4-tts-sapi4 list` to see all available voices.

## Building from Source

The Rust binary is cross-compiled from Linux to Windows using `cargo-xwin`:

```bash
# Install dependencies
cargo install cargo-xwin
rustup target add i686-pc-windows-msvc

# Build the 32-bit binary (required for SAPI4)
cd ../sapi4-rs
XWIN_ARCH=x86 cargo xwin build --target i686-pc-windows-msvc --release

# Copy to files directory
cp ../target/i686-pc-windows-msvc/release/sapi4-rs.exe files/

# Build Docker image
docker compose build
```

## Architecture

- **sapi4-rs**: Rust binary with manual COM interface bindings for SAPI4
- **Wine**: Windows compatibility layer for running the .exe on Linux
- **Docker**: Containerized environment with SAPI4 runtime pre-installed

## Output Format

The synthesized audio is:
- Format: WAV (RIFF)
- Sample rate: 11025 Hz
- Bit depth: 16-bit
- Channels: Mono
