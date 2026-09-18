"""
Temas de Archito — la estética que la comunidad de ricing ama.

8 paletas oscuras cuidadosamente calibradas + modo «auto» que lee
los colores generados por pywal (~/.cache/wal/colors.json) o matugen,
para que Archito SIEMPRE combine con tu wallpaper sin configurar nada.

Cada paleta define: fondo (gradiente), borde, texto, acento y los
tres colores semánticos (ok / warn / danger).
"""

import json
import os
from dataclasses import dataclass

from archito.core.memory import Memory


@dataclass(frozen=True)
class Palette:
    key: str
    label: str
    bg_top: str
    bg_bottom: str
    border: str
    title: str
    text: str
    text_dim: str
    meta: str
    accent: str
    ok: str
    warn: str
    danger: str
    code_bg: str


PALETTES: dict[str, Palette] = {
    "archito-noche": Palette(
        key="archito-noche", label="Archito Noche ★",
        bg_top="#131926", bg_bottom="#0c101a",
        border="#6fb6ff", title="#f0f5ff",
        text="#dde6f6", text_dim="#a9b6cf", meta="#66738f",
        accent="#6fb6ff", ok="#79d68a", warn="#f2c14e",
        danger="#ff6b81", code_bg="#05080e",
    ),
    "catppuccin": Palette(
        key="catppuccin", label="Catppuccin Mocha",
        bg_top="#1e1e2e", bg_bottom="#181825",
        border="#89b4fa", title="#cba6f7",
        text="#cdd6f4", text_dim="#a6adc8", meta="#6c7086",
        accent="#89b4fa", ok="#a6e3a1", warn="#f9e2af",
        danger="#f38ba8", code_bg="#11111b",
    ),
    "tokyo-night": Palette(
        key="tokyo-night", label="Tokyo Night",
        bg_top="#1a1b26", bg_bottom="#16161e",
        border="#7aa2f7", title="#c0caf5",
        text="#c0caf5", text_dim="#9aa5ce", meta="#565f89",
        accent="#7aa2f7", ok="#9ece6a", warn="#e0af68",
        danger="#f7768e", code_bg="#101017",
    ),
    "nord": Palette(
        key="nord", label="Nord",
        bg_top="#2e3440", bg_bottom="#292e39",
        border="#88c0d0", title="#eceff4",
        text="#d8dee9", text_dim="#b9c2d0", meta="#6d7a8c",
        accent="#88c0d0", ok="#a3be8c", warn="#ebcb8b",
        danger="#bf616a", code_bg="#232936",
    ),
    "rose-pine": Palette(
        key="rose-pine", label="Rosé Pine",
        bg_top="#1f1d2e", bg_bottom="#191724",
        border="#ebbcba", title="#e0def4",
        text="#e0def4", text_dim="#908caa", meta="#6e6a86",
        accent="#c4a7e7", ok="#9ccfd8", warn="#f6c177",
        danger="#eb6f92", code_bg="#14121f",
    ),
    "gruvbox": Palette(
        key="gruvbox", label="Gruvbox Dark",
        bg_top="#32302f", bg_bottom="#282828",
        border="#8ec07c", title="#fbf1c7",
        text="#ebdbb2", text_dim="#bdae93", meta="#7c6f64",
        accent="#83a598", ok="#b8bb26", warn="#fabd2f",
        danger="#fb4934", code_bg="#1d2021",
    ),
    "dracula": Palette(
        key="dracula", label="Dracula",
        bg_top="#2b2d3e", bg_bottom="#21222c",
        border="#bd93f9", title="#f8f8f2",
        text="#f8f8f2", text_dim="#b5bcc9", meta="#6272a4",
        accent="#bd93f9", ok="#50fa7b", warn="#f1fa8c",
        danger="#ff5555", code_bg="#191a24",
    ),
    "everforest": Palette(
        key="everforest", label="Everforest",
        bg_top="#34393f", bg_bottom="#2d353b",
        border="#a7c080", title="#d3c6aa",
        text="#d3c6aa", text_dim="#9da9a0", meta="#78847d",
        accent="#a7c080", ok="#a7c080", warn="#dbbc7f",
        danger="#e67e80", code_bg="#272e33",
    ),
}

ALIASES = {
    "noche": "archito-noche", "archito": "archito-noche",
    "catppuccin": "catppuccin", "mocha": "catppuccin",
    "tokyo": "tokyo-night", "tokio": "tokyo-night",
    "nord": "nord", "norte": "nord",
    "rose": "rose-pine", "rosé": "rose-pine", "rose pine": "rose-pine",
    "gruvbox": "gruvbox", "dracula": "dracula",
    "everforest": "everforest", "bosque": "everforest",
}


# ──────────────────────────────────────────────
# Modo AUTO — colores del wallpaper vía pywal/matugen
# ──────────────────────────────────────────────

WAL_PATHS = [
    os.path.expanduser("~/.cache/wal/colors.json"),           # pywal
    os.path.expanduser("~/.config/matugen/colors.json"),       # matugen estándar
]


def _palette_from_wal(data: dict) -> Palette | None:
    """Convierte el JSON de pywal/matugen en una Palette."""
    try:
        colors = data.get("colors", data)
        special = data.get("special", {})

        bg = special.get("background") or colors.get("background") or "#12151c"
        fg = special.get("foreground") or colors.get("foreground") or "#e6ebf5"
        c1 = colors.get("color1", "#ff6b81")   # rojo
        c2 = colors.get("color2", "#79d68a")   # verde
        c3 = colors.get("color3", "#f2c14e")   # amarillo
        c4 = colors.get("color4", "#6fb6ff")   # azul
        c6 = colors.get("color6", c4)          # cyan
        c8 = colors.get("color8", c4)          # gris brillante

        def hx(v):
            v = str(v).strip()
            return v if v.startswith("#") else f"#{v}"

        return Palette(
            key="auto", label=f"Auto ({'pywal'})",
            bg_top=hx(bg), bg_bottom=hx(bg),
            border=hx(c4), title=hx(fg),
            text=hx(fg), text_dim=hx(c8), meta=hx(c8),
            accent=hx(c6) if hx(c6).lower() != hx(bg).lower() else hx(c4),
            ok=hx(c2), warn=hx(c3), danger=hx(c1),
            code_bg=hx(bg),
        )
    except Exception:
        return None


def resolve_auto() -> tuple[Palette | None, str]:
    """Busca pywal/matugen. Devuelve (paleta, fuente) o (None, '')."""
    for path in WAL_PATHS:
        try:
            if os.path.exists(path):
                with open(path, encoding="utf-8") as f:
                    pal = _palette_from_wal(json.load(f))
                if pal:
                    source = "pywal" if "wal" in path else "matugen"
                    return pal, source
        except (json.JSONDecodeError, OSError):
            continue
    return None, ""


def active_palette() -> Palette:
    """La paleta actualmente seleccionada en memoria."""
    memory = Memory()
    name = getattr(memory, "theme", "archito-noche")
    if name == "auto":
        pal, _ = resolve_auto()
        if pal:
            return pal
        return PALETTES["archito-noche"]
    return PALETTES.get(name, PALETTES["archito-noche"])


def normalize_theme_name(n: str) -> str | None:
    """De texto libre a clave de paleta ('tema tokyo' → 'tokyo-night')."""
    n = n.lower().strip()
    if n in ("auto", "automatico"):
        return "auto"
    for alias, key in ALIASES.items():
        if alias in n:
            return key
    return None
