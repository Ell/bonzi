#!/bin/bash
# SAPI4 TTS Entrypoint
# Runs the sapi4-rs.exe binary under Wine with xvfb for display handling

cd /sapi4

# If no arguments provided, show help
if [ $# -eq 0 ]; then
    xvfb-run -a wine sapi4-rs.exe --help 2>/dev/null
    exit 0
fi

# Run the command with xvfb for Wine display support
# Redirect Wine/xvfb stderr to /dev/null to avoid polluting stdout when piping
xvfb-run -a wine sapi4-rs.exe "$@" 2>/dev/null
