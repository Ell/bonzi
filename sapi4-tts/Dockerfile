# SAPI4 TTS Docker Container
#
# Uses a pre-compiled Rust binary (sapi4-rs) to interface with SAPI4 TTS via Wine
# The binary is cross-compiled from Linux to Windows using cargo-xwin
#
# Build requirements:
#   1. Build the Rust binary first: XWIN_ARCH=x86 cargo xwin build --target i686-pc-windows-msvc --release
#   2. Copy the binary here: cp ../target/i686-pc-windows-msvc/release/sapi4-rs.exe files/
#   3. Build the Docker image: docker build -t sapi4-tts .

FROM debian:bookworm-slim

# Install runtime dependencies
RUN dpkg --add-architecture i386 && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        wine wine32 xvfb xauth ca-certificates cabextract wget curl unzip p7zip-full && \
    rm -rf /var/lib/apt/lists/*

# Set up Wine
ENV WINEPREFIX=/root/.wine
ENV WINEARCH=win32
ENV DISPLAY=:99
ENV WINEDEBUG=-all

# Initialize Wine prefix
RUN xvfb-run -a wineboot --init && wineserver -w

# Download SAPI4 runtime and voices from TETYYS/SAPI4 repo
WORKDIR /tmp
RUN curl -sL "https://github.com/TETYYS/SAPI4/raw/master/spchapi.exe" -o spchapi.exe && \
    curl -sL "https://github.com/TETYYS/SAPI4/raw/master/tv_enua.exe" -o tv_enua.exe && \
    ls -la *.exe

# Install SAPI4 runtime - these are InstallShield installers
# Run them with Wine and give them time to complete
RUN xvfb-run -a sh -c ' \
    wine spchapi.exe /Q; \
    sleep 15; \
    wineserver -w' || true

RUN xvfb-run -a sh -c ' \
    wine tv_enua.exe /Q; \
    sleep 15; \
    wineserver -w' || true

# Clean up installers
RUN rm -f /tmp/*.exe

# Verify SAPI4 installation - check for key files
RUN echo "=== Checking SAPI4 installation ===" && \
    find /root/.wine -name "*.dll" | xargs -I{} basename {} | grep -i "sp\|vtxt\|tts" | sort -u || echo "No speech DLLs found"

# Create working directory
WORKDIR /sapi4

# Copy the pre-compiled Rust binary (32-bit)
COPY files/sapi4-rs.exe /sapi4/

# Copy ACS files for voice extraction (optional, for --acs-file support)
COPY files/*.acs /sapi4/agents/

# Copy entrypoint script
COPY scripts/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
