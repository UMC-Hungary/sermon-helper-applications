#!/usr/bin/env bash
# Presenter Receiver — one-line installer
#
# Install only:
#   curl -fsSL https://raw.githubusercontent.com/UMC-Hungary/sermon-helper-applications/main/presenter-receiver/install.sh | bash
#
# Install and run immediately:
#   curl -fsSL https://raw.githubusercontent.com/UMC-Hungary/sermon-helper-applications/main/presenter-receiver/install.sh | bash -s -- ws://YOUR_SERVER_IP:3000/ws
#
set -e

REPO="UMC-Hungary/sermon-helper-applications"
DEST="${PRESENTER_INSTALL_DIR:-/usr/local/bin}/presenter-receiver"
WS_URL="${1:-}"

# ── Find latest presenter-receiver release ────────────────────────────────────
RELEASE_TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases" \
  | grep '"tag_name"' \
  | grep 'presenter-receiver-v' \
  | head -1 \
  | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

if [ -z "$RELEASE_TAG" ]; then
    echo "Error: no presenter-receiver release found on GitHub"
    exit 1
fi

BASE_URL="https://github.com/${REPO}/releases/download/${RELEASE_TAG}"

# ── Detect platform ───────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}-${ARCH}" in
    Darwin-arm64)   BINARY="presenter-receiver-macos-arm64"  ;;
    Darwin-x86_64)  BINARY="presenter-receiver-macos-x86_64" ;;
    Linux-aarch64)  BINARY="presenter-receiver-linux-arm64"  ;;
    Linux-armv7l)   BINARY="presenter-receiver-linux-arm64"  ;;
    Linux-x86_64)   BINARY="presenter-receiver-linux-x86_64" ;;
    *)
        echo "Unsupported platform: ${OS}-${ARCH}"
        echo "Supported: macOS arm64/x86_64, Linux arm64/x86_64"
        exit 1
        ;;
esac

# ── Install Linux system dependencies ────────────────────────────────────────
if [ "$OS" = "Linux" ] && command -v apt-get &>/dev/null; then
    MISSING=""
    for lib in libcairo2 libcairo-gobject2 libpango-1.0-0 libpangocairo-1.0-0 libpangoft2-1.0-0 libglib2.0-0; do
        if ! dpkg -s "$lib" &>/dev/null 2>&1; then
            MISSING="$MISSING $lib"
        fi
    done
    if [ -n "$MISSING" ]; then
        echo "==> Installing required system libraries:$MISSING"
        sudo apt-get install -y $MISSING
    fi
fi

# ── Download ──────────────────────────────────────────────────────────────────
echo "==> Detected: ${OS}-${ARCH} → $BINARY"
echo "==> Downloading from GitHub Releases (${RELEASE_TAG})..."

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

if command -v curl &>/dev/null; then
    curl -fsSL "$BASE_URL/$BINARY" -o "$TMP"
elif command -v wget &>/dev/null; then
    wget -qO "$TMP" "$BASE_URL/$BINARY"
else
    echo "Error: curl or wget is required"
    exit 1
fi

# ── Install ───────────────────────────────────────────────────────────────────
chmod +x "$TMP"

# Try to install to /usr/local/bin, fall back to current directory
if [ -w "$(dirname "$DEST")" ]; then
    mv "$TMP" "$DEST"
    echo "==> Installed to $DEST"
else
    LOCAL_DEST="./presenter-receiver"
    cp "$TMP" "$LOCAL_DEST"
    chmod +x "$LOCAL_DEST"
    DEST="$LOCAL_DEST"
    echo "==> /usr/local/bin not writable — installed to $LOCAL_DEST"
    echo "    To install globally: sudo mv $LOCAL_DEST /usr/local/bin/presenter-receiver"
fi

# ── Run ───────────────────────────────────────────────────────────────────────
if [ -n "$WS_URL" ]; then
    echo "==> Starting: $DEST $WS_URL"
    exec "$DEST" "$WS_URL"
else
    echo ""
    echo "Done. Usage:"
    echo "  $DEST ws://YOUR_SERVER_IP:3000/ws"
fi
