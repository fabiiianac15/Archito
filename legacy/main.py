#!/usr/bin/env python3
"""
Archito - Punto de entrada principal
Milestone 0.0.1 — Que aparezca.
"""
import sys

try:
    from PySide6.QtWidgets import QApplication
    from PySide6.QtGui import QFont
except ImportError:
    print("Error: PySide6 no está instalado.")
    print("Instálalo con: sudo pacman -S python-pyside6")
    sys.exit(1)

from archito_window import ArchitoWindow


def main():
    app = QApplication(sys.argv)
    app.setApplicationName("archito")
    app.setDesktopFileName("archito")

    font = QFont("Sans Serif", 10)
    app.setFont(font)

    window = ArchitoWindow()
    window.show()

    sys.exit(app.exec())


if __name__ == "__main__":
    main()
