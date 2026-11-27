#!/usr/bin/env bash
set -euo pipefail

REPO="Raman-Khiva/recall"
VERSION="v0.1.0"
ARCHIVE="rc-${VERSION}-linux-x86_64.tar.gz"
CHECKSUM="${ARCHIVE}.sha256"
INSTALL_DIR="${RECALL_INSTALL_DIR:-$HOME/.local/bin}"
MARKER="# recall"

echo "Installing Recall ${VERSION}..."

# --- Platform checks ---------------------------------------------------------
if [[ "$(uname -s)" != "Linux" ]]; then
    echo "Error: This installer currently supports Linux only." >&2
    exit 1
fi

ARCH="$(uname -m)"
if [[ "$ARCH" != "x86_64" ]]; then
    echo "Error: Unsupported architecture: ${ARCH}" >&2
    echo "Currently supported: Linux x86_64" >&2
    exit 1
fi

for cmd in curl tar sha256sum install find; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "Error: Required command not found: $cmd" >&2
        exit 1
    fi
done

if [[ -z "${HOME:-}" ]]; then
    echo "Error: \$HOME is not set." >&2
    exit 1
fi

# --- Download and verify -----------------------------------------------------
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

echo "Downloading Recall..."
curl -fL "${BASE_URL}/${ARCHIVE}" -o "${TMP_DIR}/${ARCHIVE}"
curl -fL "${BASE_URL}/${CHECKSUM}" -o "${TMP_DIR}/${CHECKSUM}"

echo "Verifying checksum..."
( cd "$TMP_DIR" && sha256sum --check "$CHECKSUM" )

echo "Extracting binary..."
tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "$TMP_DIR"

# Don't depend on the exact folder name inside the archive
BINARY="$(find "$TMP_DIR" -type f -name rc | head -n 1)"
if [[ -z "$BINARY" ]]; then
    echo "Error: Binary not found in release archive." >&2
    exit 1
fi

# --- Install -----------------------------------------------------------------
mkdir -p "$INSTALL_DIR"
install -m 755 "$BINARY" "${INSTALL_DIR}/rc"

echo ""
echo "Recall installed to ${INSTALL_DIR}/rc"

# --- PATH setup --------------------------------------------------------------
RESTART_HINT=""

path_has_dir() {
    case ":$PATH:" in *":$INSTALL_DIR:"*) return 0 ;; *) return 1 ;; esac
}

add_posix_line() {  # $1 = rc file (bash/zsh)
    local rc="$1"
    if grep -qsF "$MARKER" "$rc"; then
        return 0
    fi
    printf '\n%s\nexport PATH="%s:$PATH"\n' "$MARKER" "$INSTALL_DIR" >> "$rc"
}

if path_has_dir; then
    :  # already usable in this session
elif [[ -n "${RECALL_NO_MODIFY_PATH:-}" ]]; then
    echo "Skipping PATH changes (RECALL_NO_MODIFY_PATH is set)."
    echo "Add ${INSTALL_DIR} to your PATH manually."
else
    SHELL_NAME="$(basename "${SHELL:-}")"
    case "$SHELL_NAME" in
        zsh)
            RC="${ZDOTDIR:-$HOME}/.zshrc"
            add_posix_line "$RC"
            RESTART_HINT="source \"$RC\""
            ;;
        bash)
            RC="$HOME/.bashrc"
            add_posix_line "$RC"
            # Login shells (ssh, tty) read .bash_profile/.profile, which
            # may not source .bashrc. Cover the file that exists.
            if [[ -f "$HOME/.bash_profile" ]]; then
                add_posix_line "$HOME/.bash_profile"
            elif [[ -f "$HOME/.profile" ]]; then
                add_posix_line "$HOME/.profile"
            fi
            RESTART_HINT="source \"$RC\""
            ;;
        fish)
            FISH_CONF="$HOME/.config/fish/conf.d/recall.fish"
            mkdir -p "$(dirname "$FISH_CONF")"
            cat > "$FISH_CONF" <<EOF
$MARKER
contains "$INSTALL_DIR" \$PATH; or set -gx PATH "$INSTALL_DIR" \$PATH
EOF
            RESTART_HINT="source \"$FISH_CONF\""
            ;;
        *)
            echo "" >&2
            echo "Warning: unsupported shell '${SHELL_NAME:-unknown}'." >&2
            echo "Add ${INSTALL_DIR} to your PATH manually, e.g.:" >&2
            echo "  export PATH=\"${INSTALL_DIR}:\$PATH\"" >&2
            ;;
    esac
fi

# --- Shadowing check (e.g. older install in /usr/local/bin) ------------------
for old in /usr/local/bin/rc /usr/bin/rc; do
    if [[ -x "$old" && "$old" != "${INSTALL_DIR}/rc" ]]; then
        echo "" >&2
        echo "Warning: another 'rc' exists at ${old}." >&2
        echo "It may take priority over this install. Remove it if it's an old Recall:" >&2
        echo "  sudo rm ${old}" >&2
    fi
done

# --- Done --------------------------------------------------------------------
echo ""
if [[ -n "$RESTART_HINT" ]]; then
    echo "Restart your terminal, or run:"
    echo "  ${RESTART_HINT}"
    echo ""
fi
echo "Then run: rc --version"
