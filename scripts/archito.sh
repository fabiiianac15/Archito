#!/bin/bash
# Archito — Lanzador de desarrollo
# Ejecuta la app GTK4 con el PYTHONPATH correcto y resuelve
# el LD_PRELOAD de gtk4-layer-shell cuando hace falta.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

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

if [ -n "$LAYER_LIB" ]; then
    export LD_PRELOAD="$LAYER_LIB"
fi

cd "$SCRIPT_DIR"
PYTHONPATH=src exec python3 -m archito "$@"
