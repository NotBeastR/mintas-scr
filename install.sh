#!/usr/bin/env bash
# Mintas Language Installer/Uninstaller
# Cross-platform: Windows (Git Bash), Linux, macOS

set -e

REPO="NotBeastR/mintas-scr"
INSTALL_DIR=""
ACTION="${1:-install}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     OS="linux";;
        Darwin*)    OS="macos";;
        MINGW*|MSYS*|CYGWIN*) OS="windows";;
        *)          error "Unsupported OS: $(uname -s)";;
    esac
    info "Detected OS: $OS"
}

# Get latest release URL
get_latest_release_url() {
    API_URL="https://api.github.com/repos/$REPO/releases/latest"
    
    case "$OS" in
        windows) ASSET_NAME="mintas-windows.zip";;
        linux)   ASSET_NAME="mintas-linux.tar.gz";;
        macos)   ASSET_NAME="mintas-macos.tar.gz";;
    esac
    
    info "Fetching latest release from GitHub..."
    DOWNLOAD_URL=$(curl -s "$API_URL" | grep "browser_download_url.*$ASSET_NAME" | cut -d '"' -f 4)
    
    if [ -z "$DOWNLOAD_URL" ]; then
        error "Failed to find $ASSET_NAME in latest release"
    fi
    
    info "Found: $DOWNLOAD_URL"
}

# Install function
install() {
    detect_os
    get_latest_release_url
    
    TEMP_DIR=$(mktemp -d)
    DOWNLOAD_FILE="$TEMP_DIR/$ASSET_NAME"
    
    info "Downloading $ASSET_NAME..."
    curl -L "$DOWNLOAD_URL" -o "$DOWNLOAD_FILE" || error "Download failed"
    
    case "$OS" in
        windows)
            # Windows installation via Git Bash
            INSTALL_DIR="$LOCALAPPDATA/Mintas"
            mkdir -p "$INSTALL_DIR"
            
            info "Extracting to $INSTALL_DIR..."
            unzip -q "$DOWNLOAD_FILE" -d "$INSTALL_DIR" || error "Extraction failed"
            
            # Add to PATH (Windows-specific)
            info "Adding to PATH..."
            WINDOWS_PATH=$(cygpath -w "$INSTALL_DIR")
            
            # Use PowerShell to add to user PATH
            powershell.exe -Command "[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';$WINDOWS_PATH', 'User')" 2>/dev/null || warn "PATH update may have failed. Please add $WINDOWS_PATH to your PATH manually."
            
            info "✅ Mintas installed to: $INSTALL_DIR"
            info "Please restart your terminal or run: export PATH=\"\$PATH:$INSTALL_DIR\""
            ;;
            
        linux|macos)
            # Unix installation
            INSTALL_DIR="/usr/local/bin"
            
            info "Extracting..."
            tar -xzf "$DOWNLOAD_FILE" -C "$TEMP_DIR" || error "Extraction failed"
            
            # Move binary to /usr/local/bin
            info "Installing to $INSTALL_DIR/mintas..."
            if [ -w "$INSTALL_DIR" ]; then
                mv "$TEMP_DIR/mintas" "$INSTALL_DIR/mintas"
                chmod +x "$INSTALL_DIR/mintas"
            else
                info "Need sudo permissions to install to $INSTALL_DIR"
                sudo mv "$TEMP_DIR/mintas" "$INSTALL_DIR/mintas"
                sudo chmod +x "$INSTALL_DIR/mintas"
            fi
            
            info "✅ Mintas installed to: $INSTALL_DIR/mintas"
            ;;
    esac
    
    # Cleanup
    rm -rf "$TEMP_DIR"
    
    info "Installation complete! Run 'mintas --help' to get started."
}

# Uninstall function
uninstall() {
    detect_os
    
    case "$OS" in
        windows)
            INSTALL_DIR="$LOCALAPPDATA/Mintas"
            
            if [ -d "$INSTALL_DIR" ]; then
                info "Removing $INSTALL_DIR..."
                rm -rf "$INSTALL_DIR"
                
                # Remove from PATH
                WINDOWS_PATH=$(cygpath -w "$INSTALL_DIR")
                powershell.exe -Command "\$path = [Environment]::GetEnvironmentVariable('Path', 'User'); \$newPath = (\$path.Split(';') | Where-Object { \$_ -ne '$WINDOWS_PATH' }) -join ';'; [Environment]::SetEnvironmentVariable('Path', \$newPath, 'User')" 2>/dev/null || warn "PATH cleanup may have failed."
                
                info "✅ Mintas uninstalled"
            else
                warn "Mintas not found at $INSTALL_DIR"
            fi
            ;;
            
        linux|macos)
            INSTALL_DIR="/usr/local/bin/mintas"
            
            if [ -f "$INSTALL_DIR" ]; then
                info "Removing $INSTALL_DIR..."
                if [ -w "/usr/local/bin" ]; then
                    rm "$INSTALL_DIR"
                else
                    sudo rm "$INSTALL_DIR"
                fi
                info "✅ Mintas uninstalled"
            else
                warn "Mintas not found at $INSTALL_DIR"
            fi
            ;;
    esac
}

# Main
case "$ACTION" in
    install)
        install
        ;;
    uninstall)
        uninstall
        ;;
    *)
        echo "Usage: $0 [install|uninstall]"
        echo "  install   - Install Mintas (default)"
        echo "  uninstall - Uninstall Mintas"
        exit 1
        ;;
esac
