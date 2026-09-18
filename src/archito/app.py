"""
Aplicación principal de Archito.

Instancia única con tres estados:

    EXPANDIDO  → panel de chat visible
    REPLEGADO  → solo la lengüeta-flecha en el borde derecho
    OCULTO     → nada (Ctrl+Q)

SUPER+ESPACIO alterna expandido ↔ replegado. El dock y el panel
son dos superficies layer-shell independientes que se coordinan
desde aquí.
"""

import sys

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Gdk", "4.0")

from gi.repository import Gtk, Gio

from archito.ui.dock import DockWindow
from archito.ui.window import ArchitoWindow


class ArchitoApplication(Gtk.Application):
    """Aplicación Archito — un panel, una flecha, un cerebro."""

    def __init__(self):
        super().__init__(
            application_id="archito.Archito",
            flags=Gio.ApplicationFlags.FLAGS_NONE,
        )
        self.window = None
        self.dock = None

    def do_activate(self):
        if self.window is None:
            # El dock se crea primero; el panel lo usa para repliegues
            self.dock = DockWindow(app=self, on_activate=self.toggle_archito)
            self.window = ArchitoWindow(self, dock=self.dock)
            self.window.present()
            return
        self.toggle_archito()

    def toggle_archito(self):
        if self.window.get_visible():
            self.window.collapse()
        else:
            self.window.expand()


def main():
    app = ArchitoApplication()
    return app.run(sys.argv)
