#!/bin/bash
set -e

# Get the repo root (parent of tools/)
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERUS_DIR="$REPO_ROOT/tools/verus-bin"

echo "Finding latest Verus release..."

# Get the latest rolling release tag
LATEST_TAG=$(curl -s https://api.github.com/repos/verus-lang/verus/releases | \
    grep '"tag_name"' | \
    grep 'release/rolling' | \
    head -1 | \
    sed 's/.*"tag_name": "\(.*\)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    echo "❌ Could not find latest release"
    exit 1
fi

# Extract version from tag
VERUS_VERSION=$(echo "$LATEST_TAG" | sed 's/.*\///')

echo "Latest version: $VERUS_VERSION"
echo "Installing Verus to $VERUS_DIR..."

# Remove old installation if it exists
if [ -d "$VERUS_DIR" ]; then
    echo "Removing old Verus installation..."
    rm -rf "$VERUS_DIR"
fi

# Detect OS and architecture
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
    Linux*)
        if [ "$ARCH" = "x86_64" ]; then
            PLATFORM="x86-linux"
        else
            echo "Unsupported Linux architecture: $ARCH"
            echo "Only x86_64 Linux is supported via pre-built binaries"
            exit 1
        fi
        ;;
    Darwin*)
        if [ "$ARCH" = "x86_64" ]; then
            PLATFORM="x86-macos"
        elif [ "$ARCH" = "arm64" ]; then
            PLATFORM="arm64-macos"
        else
            echo "Unsupported macOS architecture: $ARCH"
            exit 1
        fi
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM="x86-win"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Detected platform: $PLATFORM"

# Download URL
DOWNLOAD_URL="https://github.com/verus-lang/verus/releases/download/$LATEST_TAG/verus-$VERUS_VERSION-$PLATFORM.zip"

echo "Downloading from: $DOWNLOAD_URL"

# Download to temp file
TEMP_FILE=$(mktemp).zip

if ! curl -L -f "$DOWNLOAD_URL" -o "$TEMP_FILE"; then
    echo ""
    echo "❌ Failed to download Verus"
    rm -f "$TEMP_FILE"
    exit 1
fi

# Extract to repo root tools/
echo "Extracting..."
unzip -q "$TEMP_FILE" -d "$REPO_ROOT/tools/"

# The zip extracts to a directory like "verus-0.2026.02.02.2d820df-x86-linux"
# Rename it to verus-bin
EXTRACTED_DIR=$(ls -d "$REPO_ROOT/tools/verus-"* 2>/dev/null | grep -v verus-bin | head -1)

if [ -z "$EXTRACTED_DIR" ]; then
    echo "❌ Could not find extracted Verus directory"
    ls -la "$REPO_ROOT/tools/"
    rm -f "$TEMP_FILE"
    exit 1
fi

mv "$EXTRACTED_DIR" "$VERUS_DIR"
rm -f "$TEMP_FILE"

echo "✓ Verus $VERUS_VERSION installed to $VERUS_DIR"

# Find and test the binary
if [ -f "$VERUS_DIR/verus" ]; then
    VERUS_BIN="$VERUS_DIR/verus"
elif [ -f "$VERUS_DIR/verus.exe" ]; then
    VERUS_BIN="$VERUS_DIR/verus.exe"
else
    VERUS_BIN=$(find "$VERUS_DIR" -name "verus" -o -name "verus.exe" | head -1)
fi

if [ -n "$VERUS_BIN" ]; then
    echo "Verus binary: $VERUS_BIN"
    "$VERUS_BIN" --version
fi
