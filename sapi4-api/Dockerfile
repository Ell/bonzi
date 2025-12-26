# SAPI4 TTS Container for Cloudflare Workers
#
# This container runs an HTTP server that wraps the sapi4-rs.exe binary
# The HTTP server accepts synthesis requests and returns WAV audio data

FROM debian:bookworm-slim

# Install runtime dependencies
RUN dpkg --add-architecture i386 && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        wine wine32 xvfb xauth ca-certificates cabextract wget curl unzip p7zip-full \
        python3 python3-pip && \
    rm -rf /var/lib/apt/lists/*

# Set up Wine
ENV WINEPREFIX=/root/.wine
ENV WINEARCH=win32
ENV DISPLAY=:99
ENV WINEDEBUG=-all

# Initialize Wine prefix
RUN xvfb-run -a wineboot --init && wineserver -w

# Download SAPI4 runtime and voices
WORKDIR /tmp
RUN curl -sL "https://github.com/TETYYS/SAPI4/raw/master/spchapi.exe" -o spchapi.exe && \
    curl -sL "https://github.com/TETYYS/SAPI4/raw/master/tv_enua.exe" -o tv_enua.exe

# Install SAPI4 runtime
RUN xvfb-run -a sh -c 'wine spchapi.exe /Q; sleep 15; wineserver -w' || true
RUN xvfb-run -a sh -c 'wine tv_enua.exe /Q; sleep 15; wineserver -w' || true

# Clean up installers
RUN rm -f /tmp/*.exe

# Create working directory
WORKDIR /sapi4

# Copy the pre-compiled Rust binary (32-bit)
COPY files/sapi4-rs.exe /sapi4/

# Copy ACS files for voice extraction
COPY files/*.acs /sapi4/agents/

# Copy the HTTP server script
COPY server.py /sapi4/server.py

# Expose the HTTP port
EXPOSE 8080

# Start the HTTP server
CMD ["python3", "/sapi4/server.py"]
