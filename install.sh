
#!/usr/bin/env bash
set -euo pipefail

REPO="Raman-Khiva/recall"
VERSION="v0.1.0"
ARCHIVE="rc-v0.1.0-linux-x86_64.tar.gz"
CHECKSUM="${ARCHIVE}.sha256"
INSTALL_DIR="${HOME}/.local/bin"

echo "Installing Recall ${VERSION}..."

# Check operating system
if [[ "$(uname -s)" != "Linux" ]]; then
    echo "Error: This installer currently supports Linux only."
    exit 1
fi

# Check architecture
ARCH="$(uname -m)"
if [[ "$ARCH" != "x86_64" ]]; then
    echo "Error: Unsupported architecture: ${ARCH}"
    echo "Currently supported: Linux x86_64"
    exit 1
fi

# Check required tools
for cmd in curl tar sha256sum; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "Error: Required command not found: $cmd"
        exit 1
    fi
done

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

echo "Downloading Recall..."
curl -fL "${BASE_URL}/${ARCHIVE}" -o "${TMP_DIR}/${ARCHIVE}"
curl -fL "${BASE_URL}/${CHECKSUM}" -o "${TMP_DIR}/${CHECKSUM}"

echo "Verifying checksum..."
(
    cd "$TMP_DIR"
    sha256sum --check "$CHECKSUM"
)

echo "Extracting binary..."
tar -xzf "${TMP_DIR}/${ARCHIVE}" \
    -C "$TMP_DIR"

BINARY="${TMP_DIR}/recall-v0.1.0-linux-x86_64/rc"

if [[ ! -f "$BINARY" ]]; then
    echo "Error: Binary not found in release archive."
    exit 1
fi

mkdir -p "$INSTALL_DIR"
install -m 755 "$BINARY" "${INSTALL_DIR}/rc"

echo ""
echo "Recall installed successfully!"
echo "Location: ${INSTALL_DIR}/rc"

if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
    echo ""
    echo "Add Recall to your PATH:"
    echo '  echo '\''export PATH="$HOME/.local/bin:$PATH"'\'' >> ~/.bashrc'
    echo "  source ~/.bashrc"
fi

echo ""
echo "Run: rc --version"
