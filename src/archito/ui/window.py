"""Ventana flotante de Archito: una presencia suave sobre Wayland.

No intenta imitar un terminal ni un HUD. Es una superficie de conversación
ligera: la mascota es el punto focal y el fondo del escritorio sigue siendo
parte de la escena. Wayland vía GTK4 + gtk4-layer-shell.
"""

import shlex

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Gdk", "4.0")

from gi.repository import Gtk, Gdk, GLib

try:
    gi.require_version("Gtk4LayerShell", "1.0")
    from gi.repository import Gtk4LayerShell
    LAYER_SHELL = True
except (ImportError, ValueError):
    Gtk4LayerShell = None
    LAYER_SHELL = False

import re

from archito.core.actions import (
    Action,
    Response,
    CONFIRM,
    DANGEROUS,
    MODE_INLINE,
    MODE_SPAWN,
    MODE_TERMINAL,
    open_in_terminal,
    run_inline,
)
from archito.core.backend import Brain
from archito.ui.assets import get_asset_path
from archito.ui.style import ensure_css


def esc(text: str) -> str:
    return GLib.markup_escape_text(text)


# ══════════════════════════════════════════════
# Easing — movimiento fluido tipo juego
# ══════════════════════════════════════════════

def ease_out_cubic(t: float) -> float:
    return 1 - (1 - t) ** 3


def ease_out_back(t: float, s: float = 1.35) -> float:
    t -= 1
    return 1 + t * t * ((s + 1) * t + s)


# ══════════════════════════════════════════════
# Render markdown-lite → Pango
# ══════════════════════════════════════════════

_INLINE_RE = re.compile(r"(\*\*.+?\*\*|`[^`]+?`)")

ACCENT = "#8cc8ff"
DIM = "#7e8aa5"


def render_segments(md: str) -> list[dict]:
    segments = []
    for line in md.split("\n"):
        if line.startswith("$ "):
            segments.append({"t": line[2:],
                             "pre": f'<span foreground="{ACCENT}"><tt>',
                             "post": "</tt></span>"})
        elif line.startswith("> "):
            segments.append({"t": line[2:],
                             "pre": f'<span foreground="{DIM}"><i>&gt; ',
                             "post": "</i></span>"})
        else:
            for part in _INLINE_RE.split(line):
                if not part:
                    continue
                if part.startswith("**") and part.endswith("**"):
                    segments.append({"t": part[2:-2], "pre": "<b>", "post": "</b>"})
                elif part.startswith("`") and part.endswith("`") and len(part) > 2:
                    segments.append({"t": part[1:-1],
                                     "pre": f'<span foreground="{ACCENT}"><tt>',
                                     "post": "</tt></span>"})
                else:
                    segments.append({"t": part, "pre": "", "post": ""})
        segments.append({"t": "\n", "pre": "", "post": ""})
    if segments and segments[-1]["t"] == "\n":
        segments.pop()
    return segments


def segments_markup(segments) -> str:
    return "".join(s["pre"] + esc(s["t"]) + s["post"] for s in segments)


# ══════════════════════════════════════════════

class ArchitoWindow(Gtk.ApplicationWindow):

    IDLE_FRAME = "Archito.png"
    THINKING_FRAME = "Archito2.png"   # expresión "pensando"
    HAPPY_FRAME = "Archito3.png"      # expresión "¡hecho!"

    # Un poco más ancho para que una respuesta se lea como una conversación,
    # no como una consola comprimida. La altura, en cambio, es natural hasta
    # el tope de chat: crece sola y luego aparece scroll.
    PANEL_W = 510
    AVATAR_SIZE = 56

    MARGIN_RIGHT = 20
    MARGIN_BOTTOM = 20
    SLIDE_OFFSET = 260

    SLIDE_MS = 700          # entrada del panel: suave
    FADE_PANEL_MS = 420
    AVATAR_POP_MS = 520
    HISTORY_MAX = 60

    TEXTURE_CACHE: dict = {}


    # ──────────────────────────────────────────────
    # Ciclo de vida
    # ──────────────────────────────────────────────

    def __init__(self, app, dock=None):
        super().__init__(application=app)
        # Da al root del layer surface una identidad CSS propia. Es crucial:
        # cualquier píxel fuera del panel debe permanecer realmente transparente.
        self.add_css_class("archito-layer-window")
        self.app = app
        self.brain = Brain()
        self.dock = dock

        self._type_source = None
        self._pulse_source = None
        self._dots_source = None
        self._breath_source = None
        self._avatar_restore_source = None
        self._anim_sources: list[int] = []
        self._busy = False
        self._introduced = False
        self._touch_applied: bool | None = None
        self._history: list[str] = []
        self._hist_idx: int | None = None

        # Movimiento reducido: ARCHITO_CALM=1 desactiva lo decorativo
        import os
        self.calm_mode = os.environ.get("ARCHITO_CALM", "").lower() in ("1", "true")

        # Modo táctil: ARCHITO_TOUCH=1 fuerza; si no, se respeta la memoria
        if os.environ.get("ARCHITO_TOUCH", "").lower() in ("1", "true"):
            self.brain.memory.touch_mode = True

        self._setup_window()
        ensure_css()
        self._build_ui()
        self._apply_touch_class()
        self._start_intro()

    def _setup_window(self):
        self.set_decorated(False)
        self._layered = False
        if LAYER_SHELL:
            try:
                Gtk4LayerShell.init_for_window(self)
                Gtk4LayerShell.set_layer(self, Gtk4LayerShell.Layer.OVERLAY)
                Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.BOTTOM, True)
                Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.RIGHT, True)
                Gtk4LayerShell.set_namespace(self, "archito")
                Gtk4LayerShell.set_keyboard_mode(
                    self, Gtk4LayerShell.KeyboardMode.ON_DEMAND)
                self._layered = True
            except Exception:
                self._layered = False

        key_controller = Gtk.EventControllerKey()
        key_controller.connect("key-pressed", self._on_key_pressed)
        self.add_controller(key_controller)

        click_gesture = Gtk.GestureClick()
        click_gesture.connect("pressed", self._on_window_click)
        self.add_controller(click_gesture)

    def _load_css(self):
        ensure_css()

    def _apply_touch_class(self):
        """Aplica/quita la variante .touch según la preferencia guardada."""
        want = self.brain.memory.touch_mode
        if self._touch_applied == want:
            return
        if want:
            self.panel.add_css_class("touch")
        else:
            self.panel.remove_css_class("touch")
        self._touch_applied = want

    def _sync_touch(self):
        self._touch_applied = None
        self._apply_touch_class()

    # ──────────────────────────────────────────────
    # Construcción de la interfaz — una superficie de conversación
    # ──────────────────────────────────────────────

    def _build_ui(self):
        self.panel = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        self.panel.add_css_class("archito-panel")
        self.panel.set_size_request(self.PANEL_W, -1)

        self.panel.append(self._build_header())
        self.panel.append(self._build_chat_area())
        self.panel.append(self._build_chips())
        self.panel.append(self._build_input_row())

        hint = Gtk.Label(label="↵ enviar   ·   ↑↓ historial   ·   Esc minimizar")
        hint.add_css_class("hint-bar")
        hint.set_halign(Gtk.Align.START)
        self.panel.append(hint)

        self.set_child(self.panel)

    def _get_texture(self, name: str):
        from gi.repository import Gdk as _Gdk
        if name in self.TEXTURE_CACHE:
            return self.TEXTURE_CACHE[name]
        path = get_asset_path(name)
        if path is None:
            return None
        tex = _Gdk.Texture.new_from_filename(str(path))
        self.TEXTURE_CACHE[name] = tex
        return tex

    def _make_avatar(self, size: int) -> Gtk.Image:
        img = Gtk.Image()
        img.set_pixel_size(size)
        tex = self._get_texture(self.IDLE_FRAME)
        if tex is not None:
            img.set_from_paintable(tex)
        return img

    def _build_header(self):
        header = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        header.add_css_class("assistant-header")

        # Archito no es un icono de aplicación: es el centro de la conversación.
        ring = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        ring.add_css_class("avatar-ring")

        overlay = Gtk.Overlay()
        self.avatar = self._make_avatar(self.AVATAR_SIZE)
        self.avatar.set_valign(Gtk.Align.CENTER)
        overlay.set_child(self.avatar)

        self.status_dot = Gtk.Label(label="●")
        self.status_dot.add_css_class("status-dot")
        self.status_dot.add_css_class("dot-ok")
        self.status_dot.set_halign(Gtk.Align.END)
        self.status_dot.set_valign(Gtk.Align.END)
        overlay.add_overlay(self.status_dot)

        ring.append(overlay)
        header.append(ring)

        title_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=1)
        title = Gtk.Label(label="Archito")
        title.add_css_class("assistant-title")
        title.set_halign(Gtk.Align.START)
        sub = Gtk.Label(label="En tu equipo · privado por diseño")
        sub.add_css_class("assistant-subtitle")
        sub.set_halign(Gtk.Align.START)
        title_box.append(title)
        title_box.append(sub)

        spacer = Gtk.Box()
        spacer.set_hexpand(True)

        self.status_pill = Gtk.Label(label="● listo")
        self.status_pill.add_css_class("pill")
        self.status_pill.add_css_class("pill-ok")

        header.append(title_box)
        header.append(spacer)
        header.append(self.status_pill)
        return header

    def _build_chat_area(self):
        self.chat_scroll = Gtk.ScrolledWindow()
        self.chat_scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        # propagate_natural_height hace que la ventana acompañe la conversación.
        # El límite evita que el panel invada toda la pantalla en respuestas largas.
        self.chat_scroll.set_min_content_height(190)
        self.chat_scroll.set_max_content_height(470)
        self.chat_scroll.set_propagate_natural_height(True)

        self.chat_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.chat_box.set_valign(Gtk.Align.END)
        self.chat_scroll.set_child(self.chat_box)
        return self.chat_scroll

    def _build_chips(self):
        self.chips_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        return self.chips_box

    def _build_input_row(self):
        row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)

        self.entry = Gtk.Entry()
        self.entry.set_placeholder_text("Escribe lo que necesitas…")
        self.entry.add_css_class("archito-entry")
        self.entry.set_hexpand(True)
        self.entry.connect("activate", self._on_entry_activate)
        self.entry.connect("changed", self._on_entry_changed)

        entry_keys = Gtk.EventControllerKey()
        entry_keys.connect("key-pressed", self._on_entry_key)
        self.entry.add_controller(entry_keys)

        # Botón de limpiar (solo cuando hay texto) — GTK4 no tiene iconos de Entry
        self.clear_btn = Gtk.Button(label="✕")
        self.clear_btn.add_css_class("btn-flat")
        self.clear_btn.set_tooltip_text("Limpiar")
        self.clear_btn.set_visible(False)
        self.clear_btn.connect(
            "clicked", lambda *_: (self.entry.set_text(""),
                                   self.entry.grab_focus()))

        self.send_btn = Gtk.Button(label="↑")
        self.send_btn.add_css_class("btn-primary")
        self.send_btn.set_tooltip_text("Enviar (Enter)")
        self.send_btn.connect(
            "clicked", lambda *_: self._on_entry_activate(self.entry))

        row.append(self.entry)
        row.append(self.clear_btn)
        row.append(self.send_btn)
        return row

    def _on_entry_changed(self, entry):
        has_text = bool(entry.get_text().strip())
        if self.clear_btn.get_visible() != has_text:
            self.clear_btn.set_visible(has_text)

    # ──────────────────────────────────────────────
    # Motor de animación genérico (easing real)
    # ──────────────────────────────────────────────

    def _animate(self, duration_ms: int, ease, step_fn, done_fn=None):
        """
        Anima durante `duration_ms` llamando a step_fn(progress_eased).
        progress va de 0.0 a 1.0 con la función de easing indicada.
        """
        frame_ms = 16
        frames = max(1, duration_ms // frame_ms)
        state = {"f": 0}
        source: list = []

        def _tick():
            state["f"] += 1
            t = min(1.0, state["f"] / frames)
            step_fn(ease(t))
            if t >= 1.0:
                if source:
                    self._anim_sources.remove(source[0])
                if done_fn:
                    done_fn()
                return False
            return True

        sid = GLib.timeout_add(frame_ms, _tick)
        source.append(sid)
        self._anim_sources.append(sid)

    def _cancel_animations(self):
        for attr in ("_type_source", "_pulse_source", "_dots_source",
                     "_breath_source", "_avatar_restore_source"):
            src = getattr(self, attr, None)
            if src is not None:
                GLib.source_remove(src)
                setattr(self, attr, None)
        self._cancel_move()

    def _cancel_move(self):
        """Cancela SOLO las animaciones de movimiento (no el typewriter)."""
        for sid in list(self._anim_sources):
            GLib.source_remove(sid)
        self._anim_sources.clear()

    # ──────────────────────────────────────────────
    # Despliegue / repliegue (con dock lateral)
    # ──────────────────────────────────────────────

    def _start_intro(self):
        self.panel.set_opacity(0.0)
        self.avatar.set_opacity(0.0)

        if self._layered:
            Gtk4LayerShell.set_margin(
                self, Gtk4LayerShell.Edge.RIGHT,
                self.MARGIN_RIGHT + self.SLIDE_OFFSET)

        def slide(p):
            if not self._layered:
                return
            offset = self.SLIDE_OFFSET * max(0.0, 1 - p)
            Gtk4LayerShell.set_margin(
                self, Gtk4LayerShell.Edge.RIGHT,
                self.MARGIN_RIGHT + offset)

        def fade_panel(p):
            self.panel.set_opacity(min(1.0, p * 1.15))

        GLib.timeout_add(80, lambda: (
            self._animate(self.SLIDE_MS, ease_out_cubic, slide),
            self._animate(self.FADE_PANEL_MS, ease_out_cubic, fade_panel),
            False)[2])

        # El personaje hace un "pop" con rebote al final del deslizamiento
        GLib.timeout_add(420, self._avatar_pop)

        # Bienvenida cuando el panel ya está en escena
        GLib.timeout_add(1050, self._add_welcome)

        # Y desde entonces, respira
        GLib.timeout_add(1400, self._start_breath_idle)

        def _mark():
            self._introduced = True
        GLib.timeout_add(1100, _mark)

    def expand(self):
        """
        Despliega Archito desde el borde derecho con REBOTE.

        El panel entra con ease-out-back: se pasa unos píxeles hacia
        dentro y se asienta. Ese pequeño 'overshoot' es lo que da la
        sensación de juego.
        """
        if self.get_visible():
            return
        self._cancel_move()
        self.dock and self.dock.pop_out()

        if not self._introduced:
            self.set_visible(True)
            self._start_intro()
            return

        self.set_visible(True)
        self.panel.set_opacity(1.0)
        start = self.PANEL_W + 60

        def slide(p):
            if not self._layered:
                return
            offset = start * (1 - p)          # p puede pasar de 1 → rebote
            offset = max(-14.0, offset)       # nunca más allá del borde
            Gtk4LayerShell.set_margin(
                self, Gtk4LayerShell.Edge.RIGHT,
                self.MARGIN_RIGHT + offset)

        def done():
            self.entry.grab_focus()

        self._animate(620, ease_out_back, slide, done)

    def collapse(self):
        """Se desliza fuera y deja solo la lengüeta en el borde."""
        if not self.get_visible():
            self.dock and self.dock.pop_in()
            return
        self._cancel_move()

        def slide(p):
            if not self._layered:
                return
            offset = (self.PANEL_W + 60) * p
            Gtk4LayerShell.set_margin(
                self, Gtk4LayerShell.Edge.RIGHT,
                self.MARGIN_RIGHT + offset)

        def done():
            self.set_visible(False)
            self._introduced = True
            self.dock and self.dock.pop_in()

        self._animate(430, ease_out_cubic, slide, done)

    # Compatibilidad con llamadas anteriores
    def hide_smooth(self, after=None):
        def _after():
            if after:
                after()
        self.collapse()
        GLib.timeout_add(450, _after)

    def reveal_quick(self):
        self.expand()

    def _add_welcome(self):
        if not self.chat_box.get_first_child():
            welcome = self.brain.welcome()
            self.add_bot_message(welcome.text, animate=False)
            self._set_chips(welcome.chips)
        return False

    def _avatar_pop(self):
        base = self.AVATAR_SIZE

        def pop(p):
            # Rebote de tamaño: crece con overshoot y se asienta
            size = max(8, round(base * (0.55 + 0.45 * p)))
            self.avatar.set_pixel_size(size)
            self.avatar.set_opacity(min(1.0, p * 1.6))

        self._animate(self.AVATAR_POP_MS, ease_out_back, pop)
        return False

    # ──────────────────────────────────────────────
    # Estado visual — el personaje VIVE
    # ──────────────────────────────────────────────

    def set_status(self, kind: str, text: str):
        for cls in ("pill-ok", "pill-busy", "pill-err"):
            self.status_pill.remove_css_class(cls)
        self.status_pill.add_css_class(f"pill-{kind}")
        self.status_pill.set_text(text)

        for cls in ("dot-ok", "dot-busy", "dot-err"):
            self.status_dot.remove_css_class(cls)
        dot = {"ok": "dot-ok", "busy": "dot-busy", "err": "dot-err"}.get(kind, "dot-ok")
        self.status_dot.add_css_class(dot)

    def set_avatar_frame(self, name: str):
        tex = self._get_texture(name)
        if tex is not None:
            self.avatar.set_from_paintable(tex)

    def _cancel_avatar_restore(self):
        if self._avatar_restore_source is not None:
            GLib.source_remove(self._avatar_restore_source)
            self._avatar_restore_source = None

    def _avatar_express(self, frame: str, restore_after_ms: int | None = None):
        """Cambia la expresión del personaje; vuelve al idle si toca."""
        if self.calm_mode:
            return
        self._cancel_avatar_restore()
        self.set_avatar_frame(frame)
        if restore_after_ms:
            def _back():
                self._avatar_restore_source = None
                self.set_avatar_frame(self.IDLE_FRAME)
                return False
            self._avatar_restore_source = GLib.timeout_add(
                restore_after_ms, _back)

    def _start_pulse(self):
        """Pensando: expresión + parpadeo de opacidad."""
        self._stop_pulse()
        self._avatar_express(self.THINKING_FRAME)
        if self.calm_mode:
            return
        phase = {"on": True}

        def _tick():
            phase["on"] = not phase["on"]
            self.avatar.set_opacity(1.0 if phase["on"] else 0.5)
            return True

        self._pulse_source = GLib.timeout_add(400, _tick)

    def _stop_pulse(self):
        if self._pulse_source is not None:
            GLib.source_remove(self._pulse_source)
            self._pulse_source = None
        self.avatar.set_opacity(1.0)

    def celebrate(self):
        """¡Éxito! El personaje sonríe un momento (frame feliz)."""
        if self.calm_mode:
            return
        self._stop_breath()
        self._avatar_express(self.HAPPY_FRAME, restore_after_ms=1100)
        GLib.timeout_add(1200, self._start_breath_idle)

    def _stop_breath(self):
        if getattr(self, "_breath_source", None) is not None:
            GLib.source_remove(self._breath_source)
            self._breath_source = None
        self.avatar.set_opacity(1.0)

    # ── Respiración en reposo — Archito nunca está muerto ──

    def _start_breath_idle(self, *_):
        if self.calm_mode or self._busy or self._breath_source is not None:
            return False
        phase = {"up": False}

        def breathe():
            if self._busy:
                self._breath_source = None
                return False
            phase["up"] = not phase["up"]

            def fade(p):
                start = 0.86 if phase["up"] else 1.0
                end = 1.0 if phase["up"] else 0.86
                self.avatar.set_opacity(start + (end - start) * p)

            self._animate(1500, ease_out_cubic, fade)
            return True

        self._breath_source = GLib.timeout_add(1600, breathe)
        return False

    # ──────────────────────────────────────────────
    # Mensajes
    # ──────────────────────────────────────────────

    def _scroll_bottom(self):
        adj = self.chat_scroll.get_vadjustment()
        adj.set_value(adj.get_upper())
        return False

    def _now_hm(self) -> str:
        import datetime
        return datetime.datetime.now().strftime("%H:%M")

    def _fade_in_widget(self, widget: Gtk.Widget, done=None):
        """Entrada con vida: desliza desde abajo mientras aparece."""
        widget.set_opacity(0.0)
        start_margin = 12

        def step(p):
            widget.set_opacity(p)
            widget.set_margin_top(round(start_margin * (1 - p)))

        def fin():
            widget.set_opacity(1.0)
            widget.set_margin_top(0)
            if done:
                done()
        self._animate(280, ease_out_cubic, step, fin)

    def add_user_message(self, text: str):
        label = Gtk.Label(label=text)
        label.set_wrap(True)
        label.set_xalign(0.0)
        label.add_css_class("msg-user")
        label.set_margin_start(70)

        wrap = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        wrap.set_halign(Gtk.Align.END)
        meta = Gtk.Label(label=f"tú · {self._now_hm()}")
        meta.add_css_class("msg-meta")
        meta.set_halign(Gtk.Align.END)
        wrap.append(meta)
        wrap.append(label)

        self.chat_box.append(wrap)
        self._fade_in_widget(wrap)
        GLib.idle_add(self._scroll_bottom)

    def add_bot_message(self, text: str, animate: bool = True,
                        done_cb=None) -> Gtk.Label:
        # Meta con mini-avatar del personaje: todo acoplado, sin cajas sueltas
        meta_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL,
                           spacing=6, valign=Gtk.Align.CENTER)
        mini = self._make_avatar(18)
        meta_row.append(mini)
        meta = Gtk.Label(label=f"archito · {self._now_hm()}")
        meta.add_css_class("msg-meta")
        meta_row.append(meta)

        label = Gtk.Label()
        label.set_use_markup(True)
        label.add_css_class("msg-bot")
        label.set_wrap(True)
        label.set_xalign(0.0)
        label.set_margin_end(50)

        bubble = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=3)
        bubble.set_halign(Gtk.Align.START)
        bubble.append(meta_row)
        bubble.append(label)

        self.chat_box.append(bubble)
        GLib.idle_add(self._scroll_bottom)

        segments = render_segments(text)
        if animate and segments:
            self._start_typewriter(bubble, label, segments, done_cb)
        else:
            label.set_markup(segments_markup(segments))
            self._fade_in_widget(bubble, done_cb)
        return label

    def add_code_block(self, code: str, error: bool = False):
        box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        box.set_halign(Gtk.Align.START)
        box.set_margin_end(50)
        label = Gtk.Label()
        label.set_selectable(True)
        label.set_wrap(True)
        label.set_xalign(0.0)
        label.add_css_class("code-block")
        if error:
            label.add_css_class("code-err")
        label.set_markup("<tt>" + esc(code) + "</tt>")
        box.append(label)
        self.chat_box.append(box)
        GLib.idle_add(lambda: (self._scroll_bottom(), False)[1])
        return box

    def _add_stat_cards(self, stats):
        """Tarjetas con barra animada — estilo widget AGS."""
        for item in stats[:4]:
            label_text, pct, value, sev = (list(item) + [None, "", None])[:4]
            card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=5)
            card.set_halign(Gtk.Align.FILL)
            card.add_css_class("stat-card")
            if sev in ("warn", "danger", "good"):
                card.add_css_class(f"stat-{sev}")

            row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
            lbl = Gtk.Label(label=label_text)
            lbl.add_css_class("stat-label")
            lbl.set_halign(Gtk.Align.START)
            spacer = Gtk.Box()
            spacer.set_hexpand(True)
            val = Gtk.Label(label=f"{value} · {round(pct)}%")
            val.add_css_class("stat-value")
            val.set_halign(Gtk.Align.END)
            row.append(lbl)
            row.append(spacer)
            row.append(val)

            bar = Gtk.ProgressBar()
            bar.add_css_class("stat-bar")
            bar.set_fraction(0.0)

            card.append(row)
            card.append(bar)
            self.chat_box.append(card)

            # La barra se llena con vida
            self._animate(650, ease_out_cubic,
                          lambda p, b=bar, target=pct / 100.0:
                          b.set_fraction(min(1.0, p * target)))
            GLib.idle_add(lambda: (self._scroll_bottom(), False)[1])

    # ──────────────────────────────────────────────
    # Efecto máquina de escribir (markup-aware)
    # ──────────────────────────────────────────────

    def _start_typewriter(self, bubble, label, segments, done_cb=None):
        self._stop_typewriter()
        self._tw_segments = segments
        self._tw_si = 0
        self._tw_ci = 0
        self._tw_done_cb = done_cb
        self._tw_skip = False

        def _render():
            partial = []
            for j, seg in enumerate(self._tw_segments):
                if j < self._tw_si:
                    partial.append(seg["pre"] + esc(seg["t"]) + seg["post"])
                elif j == self._tw_si:
                    shown = seg["t"][:self._tw_ci]
                    if shown:
                        partial.append(seg["pre"] + esc(shown) + seg["post"])
            label.set_markup("".join(partial))

        def _advance(budget: int) -> bool:
            while budget > 0:
                seg = self._tw_segments[self._tw_si]
                remaining = len(seg["t"]) - self._tw_ci
                take = min(budget, remaining)
                self._tw_ci += take
                budget -= take
                if self._tw_ci >= len(seg["t"]):
                    self._tw_si += 1
                    self._tw_ci = 0
                    if self._tw_si >= len(self._tw_segments):
                        return False
            return True

        def _finish():
            self._type_source = None
            label.set_markup(segments_markup(self._tw_segments))
            self._finish_typewriter()

        def _tick():
            if getattr(self, "_tw_skip", False):
                _finish()
                return False
            if not _advance(2):
                _finish()
                return False
            _render()
            return True

        self._fade_in_widget(bubble)
        self._type_source = GLib.timeout_add(9, _tick)

    def _finish_typewriter(self):
        cb = getattr(self, "_tw_done_cb", None)
        self._type_source = None
        self._tw_done_cb = None
        GLib.idle_add(self._scroll_bottom)
        if cb:
            cb()

    def _skip_typewriter(self):
        self._tw_skip = True

    def _stop_typewriter(self):
        if getattr(self, "_type_source", None) is not None:
            GLib.source_remove(self._type_source)
            self._type_source = None

    # ──────────────────────────────────────────────
    # Chips de sugerencia
    # ──────────────────────────────────────────────

    def _set_chips(self, chips: list[str]):
        while self.chips_box.get_first_child():
            self.chips_box.remove(self.chips_box.get_first_child())
        for text in chips[:5]:
            btn = Gtk.Button(label=text)
            btn.add_css_class("chip")
            btn.connect("clicked", self._on_chip_clicked, text)
            self.chips_box.append(btn)

    def _on_chip_clicked(self, btn, text: str):
        if not self._busy:
            self._submit(text)

    # ──────────────────────────────────────────────
    # Tarjetas de acción (FASE 11)
    # ──────────────────────────────────────────────

    def _add_action_card(self, action: Action):
        card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=7)
        card.set_halign(Gtk.Align.START)
        card.set_margin_end(50)
        card.add_css_class("card")
        level = "confirm" if action.level == CONFIRM else \
            ("danger" if action.level == DANGEROUS else "safe")
        if action.level != "safe":
            card.add_css_class(f"card-{level}")

        icon = {"confirm": "🛈", "danger": "⚠️"}.get(level, "")
        title = Gtk.Label(label=f"{icon} {action.label}".strip())
        title.add_css_class("card-title")
        title.set_xalign(0.0)
        title.set_wrap(True)
        title.set_halign(Gtk.Align.START)
        card.append(title)

        if action.description:
            desc = Gtk.Label(label=action.description)
            desc.add_css_class("card-desc")
            desc.set_wrap(True)
            desc.set_xalign(0.0)
            desc.set_halign(Gtk.Align.START)
            card.append(desc)

        cmd = Gtk.Label()
        cmd.set_markup("<tt>$ " + esc(action.command) + "</tt>")
        cmd.add_css_class("cmdline")
        cmd.set_selectable(True)
        cmd.set_wrap(True)
        cmd.set_xalign(0.0)
        cmd.set_halign(Gtk.Align.START)
        card.append(cmd)

        if action.danger_note:
            warn = Gtk.Label(label="⚠ " + action.danger_note)
            warn.add_css_class("card-warn")
            warn.set_wrap(True)
            warn.set_xalign(0.0)
            warn.set_halign(Gtk.Align.START)
            card.append(warn)

        buttons = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)

        run_btn = Gtk.Button(label="Ejecutar")
        run_btn.add_css_class("btn-danger" if action.level == DANGEROUS
                              else "btn-primary")
        term_btn = Gtk.Button(label="En terminal")
        term_btn.add_css_class("btn-flat")
        explain_btn = Gtk.Button(label="Explicar")
        explain_btn.add_css_class("btn-flat")
        cancel_btn = Gtk.Button(label="Cancelar")
        cancel_btn.add_css_class("btn-flat")

        state = {"done": False}

        def _disable_all():
            for b in (run_btn, term_btn, explain_btn, cancel_btn):
                b.set_sensitive(False)

        def _mark(text_done: str):
            done_label = Gtk.Label(label=text_done)
            done_label.add_css_class("card-done")
            done_label.set_valign(Gtk.Align.CENTER)
            buttons.append(done_label)

        def _on_cancel(_):
            if state["done"]:
                return
            state["done"] = True
            _disable_all()
            _mark("✖ cancelado")
            self.brain.activity.record_action(action.command, action.label, "cancelled")
            self.add_bot_message("> Cancelado. No ejecuté nada.", animate=False)

        def _on_explain(_):
            resp = self.brain.explain(action.command)
            self.add_bot_message(resp.text)

        def _execute(via_terminal: bool):
            if action.mode == MODE_INLINE and not via_terminal:
                result = run_inline(action.command)
                if result.ok:
                    self.brain.activity.record_action(action.command, action.label, "completed")
                    self.add_bot_message("**Resultado:**", animate=False)
                    if result.output:
                        self.add_code_block(result.output)
                else:
                    self.brain.activity.record_action(action.command, action.label, "failed")
                    self.add_bot_message("**Falló la ejecución:**", animate=False)
                    if result.error or result.output:
                        self.add_code_block(result.error or result.output,
                                            error=True)
                    self.set_status("err", "● error")
            else:
                opened = open_in_terminal(action.command)
                if opened:
                    self.brain.activity.record_action(action.command, action.label, "terminal_opened")
                    self.add_bot_message(
                        "> Terminal abierta. Escribe tu contraseña si te la pide; "
                        "la ventana espera hasta que pulses Enter.",
                        animate=False)
                else:
                    self.brain.activity.record_action(action.command, action.label, "terminal_failed")
                    self.add_bot_message(
                        "⚠ No encontré un emulador de terminal instalado.",
                        animate=False)

        def _finish_ok(note: str):
            state["done"] = True
            _disable_all()
            _mark(note)

        def _on_run(_):
            if state["done"]:
                return
            if action.mode == MODE_INLINE:
                _finish_ok("✔ ejecutado")
                _execute(via_terminal=False)
            elif action.mode == MODE_SPAWN:
                import subprocess as _sp
                _finish_ok("✔ lanzado")
                try:
                    from archito.core.actions import clean_env
                    _sp.Popen(shlex.split(action.command),
                              stdout=_sp.DEVNULL, stderr=_sp.DEVNULL,
                        start_new_session=True, env=clean_env())
                    self.brain.activity.record_action(action.command, action.label, "spawned")
                except Exception:
                    self.brain.activity.record_action(action.command, action.label, "failed")
                    self.add_bot_message("⚠ No pude lanzar el proceso.",
                                         animate=False)
            else:
                _finish_ok("✔ enviado a terminal")
                _execute(via_terminal=True)
            GLib.idle_add(self._scroll_bottom)

        run_btn.connect("clicked", _on_run)
        term_btn.connect("clicked",
                         lambda _: (_finish_ok("✔ en terminal"),
                                    _execute(via_terminal=True)))
        explain_btn.connect("clicked", _on_explain)
        cancel_btn.connect("clicked", _on_cancel)

        buttons.append(run_btn)
        buttons.append(term_btn)
        buttons.append(explain_btn)
        buttons.append(cancel_btn)
        card.append(buttons)

        self.chat_box.append(card)
        GLib.idle_add(lambda: (self._scroll_bottom(), False)[1])

    # ──────────────────────────────────────────────
    # Flujo de conversación
    # ──────────────────────────────────────────────

    def _on_entry_activate(self, entry):
        text = entry.get_text().strip()
        if not text:
            return
        entry.set_text("")
        if self._busy:
            self._skip_typewriter()
            return
        self._submit(text)

    def _submit(self, text: str):
        if self._busy:
            return
        self._busy = True
        self.add_user_message(text)
        self._push_history(text)
        self._hist_idx = None
        self._set_chips([])
        self.set_status("busy", "● pensando…")
        self._start_pulse()
        self._show_typing_dots()
        self.entry.set_sensitive(False)
        self.send_btn.set_sensitive(False)

        # Las sondas del sistema y, si se activa, el modelo local pueden tardar.
        # Se ejecutan fuera del hilo de GTK para conservar las animaciones vivas.
        def _work():
            try:
                resp = self.brain.process(text)
            except Exception as exc:
                resp = Response(
                    text=f"Ups, algo se rompió dentro de mí. 🫣\n\n`{exc}`",
                    status="error")
            GLib.idle_add(self._render_response, resp)

        import threading
        threading.Thread(target=_work, name="archito-brain", daemon=True).start()

    def _show_typing_dots(self):
        """Tres puntos en onda — la espera también es bonita."""
        import math
        self.dots_labels = []
        row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=3)
        row.set_halign(Gtk.Align.START)
        row.set_margin_start(4)
        for _ in range(3):
            d = Gtk.Label(label="●")
            d.add_css_class("msg-meta")
            row.append(d)
            self.dots_labels.append(d)
        wrap = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        wrap.set_halign(Gtk.Align.START)
        wrap.append(row)
        self.chat_box.append(wrap)
        self.dots_widget = wrap
        GLib.idle_add(self._scroll_bottom)

        t = {"i": 0}

        def _tick():
            t["i"] += 1
            for j, lbl in enumerate(self.dots_labels):
                phase = math.sin((t["i"] * 0.45) - (j * 1.05))
                lbl.set_opacity(0.25 + 0.75 * (0.5 + 0.5 * phase))
            return True

        self._dots_source = GLib.timeout_add(60, _tick)

    def _hide_typing_dots(self):
        if getattr(self, "_dots_source", None) is not None:
            GLib.source_remove(self._dots_source)
            self._dots_source = None
        w = getattr(self, "dots_widget", None)
        if w is not None and w.get_parent() is not None:
            self.chat_box.remove(w)
        self.dots_widget = None
        self.dots_labels = []

    def _render_response(self, resp: Response):
        self._hide_typing_dots()
        self._stop_pulse()
        self._busy = False

        if resp.clear_chat:
            self._clear_chat(keep_welcome=False)

        status_map = {
            "ok": ("ok", "● online"),
            "warn": ("ok", "● online"),
            "error": ("err", "● error"),
            "bye": ("ok", "● online"),
        }
        pill_kind, pill_text = status_map.get(resp.status, ("ok", "● online"))
        self.set_status(pill_kind, pill_text)

        chips = list(resp.chips) or ["ayuda"]

        def after_text():
            if resp.code:
                self.add_code_block(resp.code,
                                    error=(resp.status == "error"))
            if resp.stats:
                self._add_stat_cards(resp.stats)
            for action in resp.actions:
                self._add_action_card(action)
            if resp.followup:
                self.add_bot_message(resp.followup, animate=False)
            self._set_chips(chips)
            self.entry.set_sensitive(True)
            self.send_btn.set_sensitive(True)
            self.entry.grab_focus()
            GLib.idle_add(self._scroll_bottom)

            # Órdenes a la UI: cambio de tema en caliente / modo táctil
            if resp.ui_command == "sync-touch":
                self._sync_touch()
            elif resp.ui_command and resp.ui_command.startswith("theme:"):
                ensure_css(force_reload=True)

            if resp.status == "ok":
                self.celebrate()          # el personaje sonríe
                self._start_breath_idle()
            if resp.status == "bye":
                # Se despide y se repliega al dock
                GLib.timeout_add(1500, self.collapse)

        self.entry.set_sensitive(False)
        self.send_btn.set_sensitive(False)
        self.add_bot_message(resp.text, animate=True, done_cb=after_text)

    def _clear_chat(self, keep_welcome: bool = True):
        while self.chat_box.get_first_child():
            self.chat_box.remove(self.chat_box.get_first_child())
        if keep_welcome:
            welcome = self.brain.welcome()
            self.add_bot_message(welcome.text, animate=False)
            self._set_chips(welcome.chips)

    # ──────────────────────────────────────────────
    # Historial del Entry (↑/↓)
    # ──────────────────────────────────────────────

    def _push_history(self, text: str):
        if self._history and self._history[-1] == text:
            return
        self._history.append(text)
        if len(self._history) > self.HISTORY_MAX:
            self._history.pop(0)

    def _on_entry_key(self, controller, keyval, keycode, state):
        if keyval == Gdk.KEY_Up and self._history:
            if self._hist_idx is None:
                self._hist_idx = len(self._history) - 1
            elif self._hist_idx > 0:
                self._hist_idx -= 1
            self.entry.set_text(self._history[self._hist_idx])
            self.entry.set_position(-1)
            return True
        if keyval == Gdk.KEY_Down:
            if self._hist_idx is None:
                return False
            if self._hist_idx < len(self._history) - 1:
                self._hist_idx += 1
                self.entry.set_text(self._history[self._hist_idx])
            else:
                self._hist_idx = None
                self.entry.set_text("")
            self.entry.set_position(-1)
            return True
        return False

    # ──────────────────────────────────────────────
    # Eventos de ventana
    # ──────────────────────────────────────────────

    def _on_key_pressed(self, controller, keyval, keycode, state):
        if keyval == Gdk.KEY_Escape:
            self.collapse()          # se convierte en la flecha del borde
            return True
        ctrl = state & Gdk.ModifierType.CONTROL_MASK
        if ctrl and keyval == Gdk.KEY_q:
            self.app.quit()
            return True
        if ctrl and keyval == Gdk.KEY_l:
            self._clear_chat()
            return True
        return False

    def _on_window_click(self, gesture, n_press, x, y):
        if getattr(self, "_type_source", None) is not None:
            self._skip_typewriter()
        self.entry.grab_focus()

    def close_archito(self):
        """Cierra con animación y sale."""
        self.hide_smooth(after=lambda: self.app.quit())
