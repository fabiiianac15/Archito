"""
Dock lateral de Archito — la flecha que vive en el borde.

Cuando Archito se oculta, no desaparece: se convierte en una pequeña
lengüeta pegada al borde derecho, centrada verticalmente. Un clic y
el panel se despliega con rebote (ease-out-back).

Detalle del protocolo layer-shell: si NO anclas el eje vertical,
la superficie queda CENTRADA verticalmente — exactamente lo que
queremos para el dock, sin conocer la resolución.
"""

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gtk, Gdk, GLib

try:
    gi.require_version("Gtk4LayerShell", "1.0")
    from gi.repository import Gtk4LayerShell
    LAYER_SHELL = True
except (ImportError, ValueError):
    Gtk4LayerShell = None
    LAYER_SHELL = False

from archito.ui.style import ensure_css


def ease_out_back(t: float, s: float = 1.9) -> float:
    t -= 1
    return 1 + t * t * ((s + 1) * t + s)


def ease_out_cubic(t: float) -> float:
    return 1 - (1 - t) ** 3


class DockWindow(Gtk.ApplicationWindow):
    """Lengüeta minimalista en el borde derecho de la pantalla."""

    WIDTH = 38
    HEIGHT = 112
    OFFSCREEN = -(38 + 16)

    def __init__(self, app, on_activate):
        super().__init__(application=app)
        self.add_css_class("archito-layer-window")
        self.on_activate_cb = on_activate
        self._anim_sources: list[int] = []
        self._breath_source = None

        self.set_decorated(False)
        self._layered = False
        if LAYER_SHELL:
            try:
                Gtk4LayerShell.init_for_window(self)
                Gtk4LayerShell.set_layer(self, Gtk4LayerShell.Layer.OVERLAY)
                # Solo RIGHT anclado → centrado verticalmente por protocolo
                Gtk4LayerShell.set_anchor(self, Gtk4LayerShell.Edge.RIGHT, True)
                Gtk4LayerShell.set_namespace(self, "archito-dock")
                self._layered = True
            except Exception:
                self._layered = False

        ensure_css()

        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        box.add_css_class("archito-dock")
        box.set_valign(Gtk.Align.CENTER)

        btn = Gtk.Button(label="❮")
        btn.add_css_class("dock-btn")
        btn.set_tooltip_text("Desplegar Archito")
        btn.connect("clicked", lambda *_: self._safe_activate())
        box.append(btn)

        self.set_child(box)

        # Nace fuera de pantalla y oculto
        self._apply_margin(self.OFFSCREEN)
        self.set_visible(False)

    def _safe_activate(self):
        try:
            self.on_activate_cb()
        except Exception:
            pass

    def _apply_margin(self, value: int):
        if self._layered:
            Gtk4LayerShell.set_margin(
                self, Gtk4LayerShell.Edge.RIGHT, round(value))

    def _animate(self, duration_ms, ease, step_fn, done_fn=None):
        frame_ms = 16
        frames = max(1, duration_ms // frame_ms)
        state = {"f": 0}
        holder = {"sid": None}

        def _tick():
            state["f"] += 1
            t = min(1.0, state["f"] / frames)
            step_fn(ease(t))
            if t >= 1.0:
                sid = holder["sid"]
                if sid in self._anim_sources:
                    self._anim_sources.remove(sid)
                if done_fn:
                    done_fn()
                return False
            return True

        holder["sid"] = GLib.timeout_add(frame_ms, _tick)
        self._anim_sources.append(holder["sid"])

    def _cancel_all(self):
        for sid in list(self._anim_sources):
            GLib.source_remove(sid)
        self._anim_sources.clear()
        self._stop_breath()

    def _stop_breath(self):
        if self._breath_source is not None:
            GLib.source_remove(self._breath_source)
            self._breath_source = None
        self.set_opacity(1.0)

    def pop_in(self):
        """Aparece desde fuera con REBOTE y luego respira suavemente."""
        self._cancel_all()
        self.set_visible(True)
        self.set_opacity(1.0)

        def step(p):
            # p pasa de 0→1 con overshoot >1: la lengüeta entra,
            # se pasa unos px hacia dentro y vuelve a asentarse
            value = self.OFFSCREEN * (1 - p)
            self._apply_margin(value)

        def done():
            self._start_breath()

        self._animate(560, ease_out_back, step, done)

    def pop_out(self):
        """Se desliza fuera rápido y desaparece."""
        self._cancel_all()
        if not self.get_visible():
            self._apply_margin(self.OFFSCREEN)
            return
        current = {"start": 0}

        def step(p):
            value = current["start"] * (1 - p) + self.OFFSCREEN * p
            self._apply_margin(value)

        def done():
            self.set_visible(False)
            self._apply_margin(self.OFFSCREEN)

        self._animate(240, ease_out_cubic, step, done)

    def _start_breath(self):
        """Respiración lenta: la lengüeta 'invita' a ser pulsada."""
        phase = {"on": False}

        def pulse_step():
            phase["on"] = not phase["on"]

            def fade(p):
                start = 0.55 if phase["on"] else 1.0
                end = 1.0 if phase["on"] else 0.55
                self.set_opacity(start + (end - start) * p)

            self._animate(800, ease_out_cubic, fade)
            return True

        self._breath_source = GLib.timeout_add(2600, pulse_step)
