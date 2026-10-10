#!/usr/bin/env bash
# Construye la AppImage de Archito y la deja en dist/Archito-x86_64.AppImage.
# Lleva dentro la isla (archito-panel) y la CLI (archito).
#
#   ./scripts/appimage.sh
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$root/app"

npm ci

# En Arch, el `strip` que trae linuxdeploy no entiende las librerías nuevas del
# sistema y aborta. Sin strip la AppImage pesa algo más, pero se construye.
NO_STRIP=1 npx tauri build --bundles appimage

built=$(ls -t "$root"/target/release/bundle/appimage/*.AppImage | head -1)
mkdir -p "$root/dist"
cp "$built" "$root/dist/Archito-x86_64.AppImage"
chmod +x "$root/dist/Archito-x86_64.AppImage"
echo "AppImage lista: dist/Archito-x86_64.AppImage ($(du -h "$root/dist/Archito-x86_64.AppImage" | cut -f1))"
