#!/usr/bin/env bash
#
# Lanzador de Archito
#
# Resuelve el problema de LD_PRELOAD con gtk4-layer-shell:
# la librería debe cargarse antes que libwayland para que
# la ventana flotante (layer surface) funcione en Hyprland.

set -e

# ── Buscar libgtk4-layer-shell.so ──
LAYER_LIB=""
for path in \
    /usr/lib/libgtk4-layer-shell.so \
    /usr/lib/gtk4-layer-shell/libgtk4-layer-shell.so \
    /usr/local/lib/libgtk4-layer-shell.so; do
    if [ -f "$path" ]; then
        LAYER_LIB="$path"
        break
    fi
done

# Búsqueda dinámica como fallback
if [ -z "$LAYER_LIB" ]; then
    found=$(find /usr/lib /usr/local/lib -name "libgtk4-layer-shell.so*" 2>/dev/null | head -1)
    if [ -n "$found" ]; then
        LAYER_LIB="$found"
    fi
fi

# ── Directorio del proyecto ──
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ -n "$LAYER_LIB" ]; then
    export LD_PRELOAD="$LAYER_LIB"
fi

# ── Ejecutar Archito ──
cd "$SCRIPT_DIR"
PYTHONPATH=src python3 -m archito
