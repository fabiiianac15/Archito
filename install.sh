#!/usr/bin/env bash
#
# Script de instalación de Archito — Milestone 0.0.1
#
# Ejecuta: ./install.sh
#
# Hace lo siguiente:
#   1. Instala las dependencias del sistema (necesita sudo)
#   2. Copia las imágenes a ~/.local/share/archito/img/
#   3. Instala el lanzador `archito` en ~/.local/bin/ (con LD_PRELOAD)
#   4. Crea un acceso .desktop
#   5. Ofrece instalar la IA local (Ollama + modelo) una sola vez
#   6. Muestra cómo configurar el atajo en Hyprland

set -e

echo "════════════════════════════════════════"
echo "  Archito v0.2 — Instalación"
echo "════════════════════════════════════════"
echo ""

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# ── 1. Dependencias del sistema ──
echo "▸ Verificando dependencias del sistema..."

NEED_INSTALL=0

for pkg in gtk4 python-gobject gtk4-layer-shell; do
    if ! pacman -Qi "$pkg" &>/dev/null; then
        echo "  ✗ Falta: $pkg"
        NEED_INSTALL=1
    else
        echo "  ✓ $pkg ya está instalado"
    fi
done

if [ "$NEED_INSTALL" -eq 1 ]; then
    echo ""
    echo "▸ Instalando dependencias (necesita sudo)..."
    sudo pacman -S --needed gtk4 python-gobject gtk4-layer-shell
else
    echo "  Todas las dependencias están instaladas."
fi

echo ""

# ── 2. Copiar imágenes a ~/.local/share/archito/img/ ──
echo "▸ Copiando imágenes..."
SHARE_DIR="$HOME/.local/share/archito/img"
mkdir -p "$SHARE_DIR"

for img in Archito.png Archito1.png Archito2.png Archito3.png; do
    if [ -f "$SCRIPT_DIR/img/$img" ]; then
        cp "$SCRIPT_DIR/img/$img" "$SHARE_DIR/$img"
        echo "  ✓ $img"
    elif [ -f "$SCRIPT_DIR/assets/img/$img" ]; then
        cp "$SCRIPT_DIR/assets/img/$img" "$SHARE_DIR/$img"
        echo "  ✓ $img (desde assets)"
    else
        echo "  ✗ No se encontró $img"
    fi
done

# También copiar a assets/img del proyecto (para desarrollo)
mkdir -p "$SCRIPT_DIR/assets/img"
for img in Archito.png Archito1.png Archito2.png Archito3.png; do
    if [ -f "$SCRIPT_DIR/img/$img" ]; then
        cp "$SCRIPT_DIR/img/$img" "$SCRIPT_DIR/assets/img/$img"
    fi
done

echo ""

# ── 3. Buscar libgtk4-layer-shell.so para LD_PRELOAD ──
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

if [ -z "$LAYER_LIB" ]; then
    found=$(find /usr/lib /usr/local/lib -name "libgtk4-layer-shell.so*" 2>/dev/null | head -1)
    if [ -n "$found" ]; then
        LAYER_LIB="$found"
    fi
fi

if [ -n "$LAYER_LIB" ]; then
    echo "  ✓ libgtk4-layer-shell: $LAYER_LIB"
else
    echo "  ⚠ No se encontró libgtk4-layer-shell.so"
fi

echo ""

# ── 4. Instalar el paquete Python ──
echo "▸ Instalando paquete Python..."
pip install --user "$SCRIPT_DIR" 2>/dev/null && echo "  ✓ Paquete instalado" || {
    echo "  ⚠ pip install falló, se usará PYTHONPATH en el lanzador"
}

echo ""

# ── 5. Crear el lanzador `archito` en ~/.local/bin/ ──
echo "▸ Creando lanzador 'archito'..."
BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"

cat > "$BIN_DIR/archito" << LAUNCHER_EOF
#!/usr/bin/env bash
# Lanzador de Archito — generado por install.sh
# Resuelve LD_PRELOAD para gtk4-layer-shell

if [ -n "$LAYER_LIB" ]; then
    export LD_PRELOAD="$LAYER_LIB"
fi

# Si el paquete pip está instalado, usar el módulo directamente
python3 -c "import archito" 2>/dev/null && {
    exec python3 -m archito
}

# Fallback: usar PYTHONPATH del directorio del proyecto
export PYTHONPATH="$SCRIPT_DIR/src"
exec python3 -m archito
LAUNCHER_EOF

chmod +x "$BIN_DIR/archito"
echo "  ✓ Lanzador instalado en $BIN_DIR/archito"

echo ""

# ── 6. Crear acceso .desktop ──
echo "▸ Creando acceso .desktop..."
APPS_DIR="$HOME/.local/share/applications"
mkdir -p "$APPS_DIR"

cat > "$APPS_DIR/archito.desktop" << DESKTOP_EOF
[Desktop Entry]
Name=Archito
Comment=Asistente local para Arch Linux
Exec=$BIN_DIR/archito
Icon=$SHARE_DIR/Archito.png
Terminal=false
Type=Application
Categories=Utility;System;
Keywords=archito;assistant;arch;
DESKTOP_EOF

echo "  ✓ Acceso creado en $APPS_DIR/archito.desktop"

echo ""

# ── 7. IA local opcional, guiada ──
# El modelo ocupa espacio y se descarga de Internet; por eso preguntamos una
# única vez durante instalación en lugar de hacerlo silenciosamente al abrir la app.
setup_local_ai() {
    local answer model
    model="${ARCHITO_OLLAMA_MODEL:-llama3.2:3b}"

    if [ "${ARCHITO_SKIP_AI:-0}" = "1" ]; then
        echo "▸ IA local omitida (ARCHITO_SKIP_AI=1)."
        return
    fi

    if ! command -v ollama &>/dev/null; then
        if [ ! -t 0 ]; then
            echo "▸ IA local: instala luego 'ollama' para activarla."
            return
        fi
        echo "▸ Archito puede instalar su cerebro local (Ollama)."
        read -r -p "  ¿Instalar IA local ahora? [S/n] " answer
        if [[ "$answer" =~ ^[Nn]$ ]]; then
            echo "  • Sin problema: Archito funciona también sin IA."
            return
        fi
        echo "  ▸ Instalando Ollama…"
        sudo pacman -S --needed ollama
    else
        echo "▸ Ollama ya está instalado."
    fi

    # El servicio elimina la necesidad de `ollama serve` en cada sesión.
    if systemctl list-unit-files ollama.service &>/dev/null; then
        echo "  ▸ Activando el servicio local de Ollama…"
        sudo systemctl enable --now ollama.service || \
            echo "  ⚠ No pude activar el servicio; Archito intentará iniciarlo al abrirse."
    fi

    if [ ! -t 0 ]; then
        echo "  • Para descargar un modelo: ollama pull $model"
        return
    fi
    read -r -p "  ¿Descargar el modelo recomendado ($model)? [S/n] " answer
    if [[ ! "$answer" =~ ^[Nn]$ ]]; then
        echo "  ▸ Descargando $model (puede tardar y ocupar varios GB)…"
        ollama pull "$model"
        echo "  ✓ IA local lista. Archito la usará automáticamente."
    else
        echo "  • Ollama queda instalado y arrancará solo; podrás añadir un modelo después."
    fi
}

echo ""
setup_local_ai
echo ""

# ── 8. Instrucciones para Hyprland ──
echo "════════════════════════════════════════"
echo "  ✓ Instalación completada"
echo "════════════════════════════════════════"
echo ""
echo "Para lanzar Archito:"
echo "  archito"
echo ""
echo "Atajo de teclado en Hyprland (~/.config/hypr/hyprland.conf):"
echo "  bind = SUPER, space, exec, archito"
echo ""
echo "Para desinstalar:"
echo "  ./uninstall.sh"
echo ""
