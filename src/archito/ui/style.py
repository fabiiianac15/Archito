"""
Estilos globales de Archito — CSS generado desde la paleta activa.

El CSS se construye dinámicamente desde core/themes.py, así que
cambiar de tema («tema catppuccin») re-estila TODO en caliente:
panel, dock, burbujas, chips y tarjetas. Incluye la variante .touch
para pantallas pequeñas — semilla del futuro Arch Mobile.
"""

from gi.repository import Gtk, Gdk

from archito.core.themes import Palette, PALETTES, active_palette

_provider = None
_loaded_key: str | None = None


def build_css(p: Palette) -> str:
    """Genera la hoja completa a partir de una paleta."""
    return f"""
/* El root de las dos layer surfaces NO puede pintar un rectángulo.
   Sin estas reglas algunos temas GTK dibujan su fondo por defecto negro. */
window.archito-layer-window,
window.archito-layer-window > box,
window.archito-layer-window > overlay {{
    background-color: transparent;
    background-image: none;
    border: none;
    box-shadow: none;
}}

/* ── Panel: una pieza de vidrio, no una tarjeta pegada ── */
.archito-panel {{
    background-image: linear-gradient(
        135deg,
        alpha({p.bg_top}, 0.34),
        alpha({p.bg_bottom}, 0.43));
    border: 1px solid alpha({p.title}, 0.12);
    border-radius: 24px;
    padding: 16px;
    box-shadow: 0 18px 52px alpha(#000000, 0.16);
}}

/* ── Identidad ── */
.assistant-header {{ padding: 2px 2px 8px; }}
.assistant-title {{ font-family: sans-serif; font-weight: 800;
                    font-size: 19px; letter-spacing: 0.2px; color: {p.title}; }}
.assistant-subtitle {{ font-size: 11px; color: {p.text_dim}; }}

.pill {{ font-size: 10px; font-weight: 700;
        border-radius: 999px; padding: 5px 9px; }}
.pill-ok   {{ background: alpha({p.ok}, 0.14); color: {p.ok};
             border: 1px solid alpha({p.ok}, 0.25); }}
.pill-busy {{ background: alpha({p.warn}, 0.14); color: {p.warn};
             border: 1px solid alpha({p.warn}, 0.45); }}
.pill-err  {{ background: alpha({p.danger}, 0.15); color: {p.danger};
             border: 1px solid alpha({p.danger}, 0.50); }}

.avatar-ring {{
    border-radius: 999px;
    border: 1px solid alpha({p.accent}, 0.44);
    background: alpha({p.accent}, 0.10);
    padding: 4px;
    box-shadow: 0 0 18px alpha({p.accent}, 0.17);
}}
.status-dot {{
    font-size: 9px;
    border-radius: 999px;
    padding: 1px 3px;
}}
.dot-ok   {{ color: {p.ok};   background: alpha({p.bg_bottom}, 0.72);
            border: 1px solid alpha({p.ok}, 0.6); }}
.dot-busy {{ color: {p.warn};  background: alpha({p.bg_bottom}, 0.9);
            border: 1px solid alpha({p.warn}, 0.7); }}
.dot-err  {{ color: {p.danger}; background: alpha({p.bg_bottom}, 0.9);
            border: 1px solid alpha({p.danger}, 0.7); }}

/* ── Dock lateral: rastro de Archito en el borde, no botón enmarcado ── */
.archito-dock {{
    background-image: linear-gradient(
        to left,
        alpha({p.bg_top}, 0.09),
        alpha({p.bg_bottom}, 0.20));
    border: 1px solid alpha({p.accent}, 0.14);
    border-right: none;
    border-radius: 18px 0 0 18px;
    padding: 17px 6px 17px 10px;
}}
.archito-dock:hover {{
    background-image: linear-gradient(to left, alpha({p.bg_top}, 0.35), alpha({p.bg_bottom}, 0.58));
    border-color: alpha({p.accent}, 0.42);
}}

button.dock-btn {{
    font-size: 15px;
    font-weight: 800;
    color: {p.accent};
    background: transparent;
    border: none;
    padding: 5px 1px;
}}
button.dock-btn:hover {{ color: {p.title}; }}

/* ── Mensajes ── */
.msg-user {{
    background: alpha({p.accent}, 0.16);
    border: 1px solid alpha({p.accent}, 0.25);
    border-radius: 16px 16px 4px 16px;
    padding: 9px 13px;
    color: {p.title};
    font-size: 13px;
}}
.msg-bot {{
    background: alpha({p.bg_top}, 0.18);
    border: 1px solid alpha({p.title}, 0.07);
    border-radius: 16px 16px 16px 4px;
    padding: 10px 13px;
    color: {p.text};
    font-size: 13px;
}}
.msg-meta {{ font-family: monospace; font-size: 9.5px; color: {p.meta}; }}

/* ── Bloques de salida ── */
.code-block {{
    background: alpha({p.code_bg}, 0.62);
    border: 1px solid alpha({p.accent}, 0.18);
    border-radius: 12px;
    padding: 8px 10px;
    font-family: monospace;
    font-size: 11.5px;
    color: {p.text};
}}
.code-err {{
    background: alpha({p.danger}, 0.12);
    border: 1px solid alpha({p.danger}, 0.55);
    color: {p.text};
}}

/* ── Tarjetas de acción ── */
.card {{
    background: alpha({p.bg_top}, 0.28);
    border-left-width: 3px;
    border-radius: 10px;
    padding: 10px 12px;
}}
.card-confirm {{ border-left-color: {p.warn};
                background: alpha({p.warn}, 0.08); }}
.card-danger  {{ border-left-color: {p.danger};
                background: alpha({p.danger}, 0.10); }}
.card-title {{ color: {p.title}; font-weight: 700; font-size: 13px; }}
.card-desc  {{ color: {p.text_dim}; font-size: 12px; }}
.card-warn  {{ color: {p.danger}; font-size: 11.5px; }}
.card-done  {{ font-family: monospace; color: {p.ok};
              font-size: 11px; font-weight: 700; }}
.cmdline {{
    background: alpha({p.code_bg}, 0.96);
    border-radius: 6px;
    padding: 5px 8px;
    font-family: monospace;
    font-size: 11.5px;
    color: {p.accent};
}}

/* ── Barras de estadísticas ── */
.stat-card {{
    background: alpha({p.bg_top}, 0.22);
    border: 1px solid alpha({p.title}, 0.07);
    border-radius: 14px;
    padding: 8px 11px;
}}
.stat-label {{ font-family: monospace; font-size: 11px;
              font-weight: 700; color: {p.text_dim}; }}
.stat-value {{ font-family: monospace; font-size: 10.5px;
              color: {p.meta}; }}
progressbar.stat-bar progress {{
    background-color: {p.accent};
    border-radius: 4px;
}}
progressbar.stat-bar trough {{
    background-color: alpha(#ffffff, 0.08);
    border-radius: 4px;
    min-height: 8px;
}}
.stat-warn progressbar.stat-bar progress {{ background-color: {p.warn}; }}
.stat-danger progressbar.stat-bar progress {{ background-color: {p.danger}; }}
.stat-good progressbar.stat-bar progress {{ background-color: {p.ok}; }}

/* ── Botones / chips ── */
button.chip {{
    border-radius: 999px;
    padding: 5px 11px;
    font-size: 11px;
    background: alpha({p.accent}, 0.09);
    color: {p.accent};
    border: 1px solid alpha({p.accent}, 0.34);
}}
button.chip:hover {{ background: alpha({p.accent}, 0.26); color: {p.title}; }}

button.btn-primary {{
    font-weight: 700;
    border-radius: 999px;
    padding: 7px 14px;
    font-size: 15px;
    background: alpha({p.accent}, 0.30);
    color: {p.title};
    border: 1px solid alpha({p.accent}, 0.58);
}}
button.btn-primary:hover {{ background: alpha({p.accent}, 0.38); }}

button.btn-danger {{
    font-family: monospace;
    font-weight: 700;
    border-radius: 8px;
    padding: 5px 13px;
    font-size: 12px;
    background: alpha({p.danger}, 0.20);
    color: {p.text};
    border: 1px solid alpha({p.danger}, 0.58);
}}
button.btn-danger:hover {{ background: alpha({p.danger}, 0.34); }}

button.btn-flat {{
    font-family: monospace;
    background: transparent;
    color: {p.text_dim};
    border: 1px solid transparent;
    border-radius: 8px;
    padding: 5px 10px;
    font-size: 11.5px;
}}
button.btn-flat:hover {{ color: {p.title}; background: alpha(#ffffff, 0.06); }}

/* ── Entrada ── */
entry.archito-entry {{
    background: alpha({p.code_bg}, 0.28);
    color: {p.title};
    border: 1px solid alpha({p.accent}, 0.40);
    border-radius: 16px;
    padding: 11px 14px;
    font-size: 13.5px;
}}
entry.archito-entry:focus {{
    border-color: {p.accent};
    box-shadow: 0 0 0 1.5px alpha({p.accent}, 0.30);
}}
entry.archito-entry image {{
    color: {p.meta};
    -gtk-icon-size: 12px;
}}

.hint-bar {{ font-size: 10px; color: alpha({p.meta}, 0.86); padding: 2px 3px 0; }}

/* ── Scrollbar fina ── */
scrollbar {{ background: transparent; }}
scrollbar slider {{
    background: alpha({p.accent}, 0.25);
    border-radius: 999px;
    min-width: 4px;
    min-height: 4px;
}}
scrollbar slider:hover {{ background: alpha({p.accent}, 0.45); }}

/* ══════════════════════════════════════════
   MODO TÁCTIL / MÓVIL — semilla de Arch Mobile
   Objetivos táctiles ≥ 40px.
   ══════════════════════════════════════════ */
.touch .msg-bot,
.touch .msg-user {{ font-size: 15px; padding: 11px 14px; }}
.touch entry.archito-entry {{ font-size: 16px; padding: 13px 15px; }}
.touch button.chip {{ font-size: 14px; padding: 10px 18px; margin-bottom: 4px; }}
.touch button.btn-primary,
.touch button.btn-flat,
.touch button.btn-danger {{ font-size: 14px; padding: 10px 18px; }}
.touch .assistant-title {{ font-size: 21px; }}
.touch .assistant-subtitle {{ font-size: 13px; }}
.touch .code-block {{ font-size: 13px; padding: 10px 12px; }}
.touch .cmdline {{ font-size: 13px; padding: 8px 10px; }}
.touch .card-title {{ font-size: 15px; }}
.touch .card-desc {{ font-size: 13.5px; }}
.touch .hint-bar {{ font-size: 11.5px; }}
.touch .pill {{ font-size: 12px; padding: 6px 12px; }}
"""


def ensure_css(force_reload: bool = False) -> None:
    """Aplica la paleta activa. Si cambió, re-estila todo en vivo."""
    global _provider, _loaded_key
    display = Gdk.Display.get_default()
    if display is None:
        return

    palette = active_palette()
    if not force_reload and _loaded_key == palette.key and _provider is not None:
        return

    old = _provider
    _provider = Gtk.CssProvider()
    _provider.load_from_string(build_css(palette))
    Gtk.StyleContext.add_provider_for_display(
        display, _provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
    if old is not None:
        Gtk.StyleContext.remove_provider_for_display(display, old)
    _loaded_key = palette.key


def current_theme_key() -> str | None:
    return _loaded_key
