#!/usr/bin/env python3
"""
Simple HTTP server that wraps the sapi4-rs.exe binary for TTS synthesis.
Designed to run inside a Cloudflare Container.
"""

import http.server
import json
import subprocess
import os
import glob
import threading
import signal
from urllib.parse import urlparse

PORT = 8080
SAPI4_EXE = "/sapi4/sapi4-rs.exe"
AGENTS_DIR = "/sapi4/agents"

# Global Xvfb process
xvfb_process = None


class TTSHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        # Log to stderr for container visibility
        print(f"[HTTP] {args[0]}", flush=True)

    def send_json(self, data, status=200):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

    def send_audio(self, audio_data):
        self.send_response(200)
        self.send_header("Content-Type", "audio/wav")
        self.send_header("Content-Length", str(len(audio_data)))
        self.end_headers()
        self.wfile.write(audio_data)

    def do_GET(self):
        path = urlparse(self.path).path

        if path == "/health":
            self.send_json({"status": "ok"})
            return

        if path == "/voices":
            # Run sapi4-rs list command to get available voices
            try:
                result = subprocess.run(
                    ["wine", SAPI4_EXE, "list"],
                    capture_output=True,
                    timeout=30,
                    env={**os.environ, "WINEDEBUG": "-all", "DISPLAY": ":99"}
                )
                output = result.stdout.decode("utf-8", errors="replace")
                self.send_json({"voices": output, "success": True})
            except Exception as e:
                self.send_json({"error": str(e), "success": False}, 500)
            return

        if path == "/agents":
            # List available ACS files
            agents = []
            for acs_file in glob.glob(f"{AGENTS_DIR}/*.acs") + glob.glob(f"{AGENTS_DIR}/*.ACS"):
                filename = os.path.basename(acs_file)
                agents.append({"filename": filename, "name": os.path.splitext(filename)[0]})
            self.send_json({"agents": agents, "success": True})
            return

        self.send_json({"error": "Not found"}, 404)

    def do_POST(self):
        path = urlparse(self.path).path

        if path == "/synthesize":
            # Read request body
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length)

            try:
                data = json.loads(body)
            except json.JSONDecodeError:
                self.send_json({"error": "Invalid JSON"}, 400)
                return

            # Support two API styles:
            # 1. Raw args: {"args": ["speak", "--text", "hello", "--stdout"]}
            # 2. Simple: {"text": "hello", "voice": "Sam", "agent": "Bonzi.acs", ...}
            args = data.get("args")

            if not args:
                # Build args from simple parameters
                text = data.get("text")
                if not text:
                    self.send_json({"error": "Missing 'text' or 'args'"}, 400)
                    return

                args = ["speak", "--text", text, "--stdout"]

                # Optional parameters
                if data.get("agent"):
                    args.extend(["--acs-file", f"/sapi4/agents/{data['agent']}"])
                elif data.get("voice"):
                    args.extend(["--voice", data["voice"]])

                if data.get("pitch") is not None:
                    args.extend(["--pitch", str(data["pitch"])])
                if data.get("speed") is not None:
                    args.extend(["--speed", str(data["speed"])])
                if data.get("gain") is not None:
                    args.extend(["--gain", str(data["gain"])])

            # Build the full command (Xvfb is already running)
            cmd = ["wine", SAPI4_EXE] + args

            try:
                print(f"Running: {' '.join(cmd)}", flush=True)
                result = subprocess.run(
                    cmd,
                    capture_output=True,
                    timeout=60,
                    env={**os.environ, "WINEDEBUG": "-all", "DISPLAY": ":99"}
                )

                if result.returncode != 0:
                    stderr = result.stderr.decode("utf-8", errors="replace")
                    print(f"Error: {stderr}", flush=True)
                    self.send_json({"error": stderr, "success": False}, 500)
                    return

                # stdout should contain the WAV data
                audio_data = result.stdout

                if len(audio_data) < 44:  # Minimum WAV header size
                    self.send_json({
                        "error": "No audio data generated",
                        "stderr": result.stderr.decode("utf-8", errors="replace"),
                        "success": False
                    }, 500)
                    return

                self.send_audio(audio_data)

            except subprocess.TimeoutExpired:
                self.send_json({"error": "Synthesis timeout"}, 504)
            except Exception as e:
                self.send_json({"error": str(e)}, 500)
            return

        self.send_json({"error": "Not found"}, 404)


def start_xvfb():
    """Start a persistent Xvfb instance."""
    global xvfb_process
    print("Starting Xvfb on display :99...", flush=True)
    xvfb_process = subprocess.Popen(
        ["Xvfb", ":99", "-screen", "0", "1024x768x24", "-nolisten", "tcp"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL
    )
    # Give Xvfb time to start
    import time
    time.sleep(1)
    print("Xvfb started.", flush=True)
    return xvfb_process


def prewarm_wine():
    """Pre-warm Wine by running a quick TTS synthesis."""
    print("Pre-warming Wine and SAPI4...", flush=True)
    env = {**os.environ, "WINEDEBUG": "-all", "DISPLAY": ":99"}

    try:
        # Run a quick synthesis to load all DLLs
        result = subprocess.run(
            ["wine", SAPI4_EXE, "speak", "--text", ".", "--stdout"],
            capture_output=True,
            timeout=60,
            env=env
        )
        if result.returncode == 0:
            print(f"Wine pre-warmed successfully ({len(result.stdout)} bytes)", flush=True)
        else:
            print(f"Wine pre-warm had errors: {result.stderr.decode('utf-8', errors='replace')[:200]}", flush=True)
    except Exception as e:
        print(f"Wine pre-warm failed: {e}", flush=True)


def cleanup(signum, frame):
    """Clean up Xvfb on shutdown."""
    global xvfb_process
    print("Shutting down...", flush=True)
    if xvfb_process:
        xvfb_process.terminate()
        xvfb_process.wait()
    exit(0)


def main():
    global xvfb_process

    print(f"Starting SAPI4 TTS server on port {PORT}...", flush=True)

    # Set up signal handlers
    signal.signal(signal.SIGTERM, cleanup)
    signal.signal(signal.SIGINT, cleanup)

    # Start persistent Xvfb
    start_xvfb()

    # List available agents
    agents = glob.glob(f"{AGENTS_DIR}/*.acs") + glob.glob(f"{AGENTS_DIR}/*.ACS")
    print(f"Available agents: {len(agents)}", flush=True)
    for agent in agents:
        print(f"  - {os.path.basename(agent)}", flush=True)

    # Pre-warm Wine in a separate thread so server starts quickly
    warmup_thread = threading.Thread(target=prewarm_wine)
    warmup_thread.daemon = True
    warmup_thread.start()

    server = http.server.HTTPServer(("0.0.0.0", PORT), TTSHandler)
    print(f"Server listening on http://0.0.0.0:{PORT}", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
