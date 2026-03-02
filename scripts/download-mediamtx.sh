#!/usr/bin/env bash
# Downloads the mediamtx binary for the current platform and places it in
# src-tauri/binaries/ with the Tauri sidecar naming convention.
#
# Usage: bash scripts/download-mediamtx.sh [version]
# Example: bash scripts/download-mediamtx.sh v1.12.2

set -euo pipefail

VERSION="${1:-v1.12.2}"
BINARIES_DIR="$(dirname "$0")/../src-tauri/binaries"
mkdir -p "$BINARIES_DIR"

download() {
    local os="$1" arch="$2" triple="$3"
    local archive="mediamtx_${VERSION}_${os}_${arch}.tar.gz"
    local url="https://github.com/bluenviron/mediamtx/releases/download/${VERSION}/${archive}"
    local dest="${BINARIES_DIR}/mediamtx-${triple}"

    echo "Downloading mediamtx ${VERSION} for ${triple}..."
    curl -fsSL "$url" | tar -xz -O mediamtx > "$dest"
    chmod +x "$dest"
    echo "  -> ${dest}"
}

case "$(uname -s)" in
    Darwin)
        case "$(uname -m)" in
            arm64)  download darwin arm64 aarch64-apple-darwin ;;
            x86_64) download darwin amd64 x86_64-apple-darwin  ;;
            *)      echo "Unsupported macOS arch: $(uname -m)"; exit 1 ;;
        esac
        ;;
    Linux)
        case "$(uname -m)" in
            x86_64)  download linux amd64 x86_64-unknown-linux-gnu  ;;
            aarch64) download linux arm64 aarch64-unknown-linux-gnu  ;;
            *)       echo "Unsupported Linux arch: $(uname -m)"; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $(uname -s)"
        echo "For Windows, download mediamtx_${VERSION}_windows_amd64.zip from:"
        echo "  https://github.com/bluenviron/mediamtx/releases/tag/${VERSION}"
        echo "Extract mediamtx.exe to src-tauri/binaries/mediamtx-x86_64-pc-windows-msvc.exe"
        exit 1
        ;;
esac

echo "Done. Run 'pnpm tauri dev' to use it."
