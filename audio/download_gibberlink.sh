#!/bin/bash
# Gibberlink Audio Downloader

# Check dependencies
if ! command -v yt-dlp &> /dev/null || ! command -v ffmpeg &> /dev/null; then
    echo "Error: This script requires yt-dlp and ffmpeg"
    exit 1
fi

URL="https://youtu.be/EtNagNezo8w"
TEMP_DIR="/tmp/gibberlink_$$"
mkdir -p "$TEMP_DIR"

TEMP_BASE="$TEMP_DIR/video_$$"

echo "Downloading from: $URL"
echo "Trying format: bestvideo[height<=720]+bestaudio/best[height<=720]"

# Use the same format strategy that worked for you before
yt-dlp -f "bestvideo[height<=720]+bestaudio/best[height<=720]" -o "$TEMP_BASE.%(ext)s" "$URL"

# Find the downloaded file
TEMP_FILE=$(find "$TEMP_DIR" -type f -not -name "*.part" | head -n 1)

if [ -z "$TEMP_FILE" ] || [ ! -f "$TEMP_FILE" ]; then
    echo "Error: Download failed. Trying fallback format..."
    
    # Try format 18 as fallback (360p with audio)
    yt-dlp -f 18 -o "$TEMP_BASE.%(ext)s" "$URL"
    TEMP_FILE=$(find "$TEMP_DIR" -type f -not -name "*.part" | head -n 1)
    
    if [ -z "$TEMP_FILE" ] || [ ! -f "$TEMP_FILE" ]; then
        echo "Error: All download attempts failed"
        rm -rf "$TEMP_DIR"
        exit 1
    fi
fi

echo "Downloaded: $TEMP_FILE"

# Extract audio to WAV
OUTPUT_FILE="gibberlink_demo.wav"
echo "Converting to WAV (48kHz mono)..."

ffmpeg -i "$TEMP_FILE" \
    -vn \
    -ac 1 \
    -ar 48000 \
    -acodec pcm_s16le \
    "$OUTPUT_FILE" -y

if [ $? -eq 0 ]; then
    echo "✓ Audio saved to: $OUTPUT_FILE"
    echo "✓ File size: $(du -h "$OUTPUT_FILE" | cut -f1)"
    echo "✓ Duration: $(ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 "$OUTPUT_FILE" | awk '{printf "%.2f seconds\n", $1}')"
else
    echo "Error: Failed to convert audio"
fi

# Clean up
rm -rf "$TEMP_DIR"
