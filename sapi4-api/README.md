# SAPI4 TTS API

A simple HTTP API for Microsoft SAPI4 text-to-speech (Microsoft Sam and friends).

## Quick Start

```bash
# Build and run
docker compose up -d --build

# Test
curl http://localhost:8080/health
```

## API Endpoints

### `GET /health`
Health check.

### `GET /voices`
List available SAPI4 voices.

### `GET /agents`
List available ACS agent files.

### `POST /synthesize`
Generate speech. Returns WAV audio directly.

**Simple API:**
```bash
curl -X POST http://localhost:8080/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello world"}' \
  --output hello.wav
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `text` | string | Text to synthesize (required) |
| `voice` | string | Voice name (e.g., "Adult Male #2") |
| `agent` | string | ACS agent file (e.g., "Bonzi.acs") |
| `pitch` | number | Voice pitch |
| `speed` | number | Voice speed |
| `gain` | number | Volume gain |

**Examples:**
```bash
# Default voice (Sam)
curl -X POST http://localhost:8080/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello"}' -o hello.wav

# With Bonzi voice
curl -X POST http://localhost:8080/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello", "agent": "Bonzi.acs"}' -o bonzi.wav

# With pitch/speed
curl -X POST http://localhost:8080/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello", "pitch": 200, "speed": 150}' -o adjusted.wav
```

**Raw args API:**
```bash
curl -X POST http://localhost:8080/synthesize \
  -H "Content-Type: application/json" \
  -d '{"args": ["speak", "--text", "Hello", "--stdout"]}' -o hello.wav
```

## Files

```
sapi4-api/
├── Dockerfile
├── docker-compose.yml
├── server.py
└── files/
    ├── sapi4-rs.exe    # TTS binary
    └── *.acs           # Agent files
```

## Adding More Voices

Copy `.acs` files to `files/` and rebuild:

```bash
cp MyAgent.acs files/
docker compose up -d --build
```
