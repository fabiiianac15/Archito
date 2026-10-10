#!/usr/bin/env bash
#
# Script de DESINSTALACIÓN de Archito
#
# Ejecuta: ./uninstall.sh
#
# Elimina:
#   1. El comando `archito` (instalado con pip)
#   2. Las imágenes copiadas a ~/.local/share/archito/
#   3. El acceso .desktop (si existe)

set -e

echo "════════════════════════════════════════"
echo "  Archito — Desinstalación"
echo "════════════════════════════════════════"
echo ""

# ── 1. Desinstalar el paquete pip ──
echo "▸ Desinstalando el comando 'archito'..."
if pip show archito &>/dev/null; then
    pip uninstall -y archito && echo "  ✓ Paquete Python eliminado."
else
    echo "  • Archito no está instalado con pip."
fi

echo ""

# ── 2. Eliminar lanzador ──
LAUNCHER="$HOME/.local/bin/archito"
if [ -f "$LAUNCHER" ]; then
    echo "▸ Eliminando lanzador $LAUNCHER..."
    rm -f "$LAUNCHER"
    echo "  ✓ Lanzador eliminado."
else
    echo "  • No hay lanzador en ~/.local/bin."
fi

echo ""

# ── 3. Eliminar imágenes compartidas y memoria local ──
SHARE_DIR="$HOME/.local/share/archito"
if [ -d "$SHARE_DIR" ]; then
    echo "▸ Eliminando imágenes de $SHARE_DIR..."
    rm -rf "$SHARE_DIR"
    echo "  ✓ Imágenes eliminadas."
else
    echo "  • No hay imágenes en $SHARE_DIR."
fi

CONFIG_DIR="$HOME/.config/archito"
if [ -d "$CONFIG_DIR" ]; then
    echo "▸ Eliminando memoria local ($CONFIG_DIR)..."
    rm -rf "$CONFIG_DIR"
    echo "  ✓ Memoria local eliminada."
else
    echo "  • No hay memoria local en $CONFIG_DIR."
fi

echo ""

# ── 4. Eliminar archivo .desktop ──
DESKTOP_FILE="$HOME/.local/share/applications/archito.desktop"
if [ -f "$DESKTOP_FILE" ]; then
    rm -f "$DESKTOP_FILE"
    echo "  ✓ Acceso .desktop eliminado."
else
    echo "  • No hay acceso .desktop."
fi

echo ""
echo "════════════════════════════════════════"
echo "  ✓ Desinstalación completada"
echo "════════════════════════════════════════"
echo ""
echo "Nota: Las dependencias del sistema (gtk4, python-gobject,"
echo "gtk4-layer-shell) NO se eliminan. Quítalas manualmente si"
echo "no las necesitas para otros programas:"
echo "  sudo pacman -Rns gtk4-layer-shell"
echo "Ollama y sus modelos tampoco se eliminan: pueden ser usados por otras apps."
echo ""
echo "El código fuente en esta carpeta no se borra."
echo ""
