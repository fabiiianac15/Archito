"""
Ventana flotante de Archito — Milestone 0.0.1

Animación de entrada:
  Archito1.png (asomando) → Archito2.png (más) → Archito3.png (listo) → Archito.png (quieto saludando)

Después aparece el cuadro de texto. Diseñada para Hyprland/Wayland.
"""
import os

from PySide6.QtCore import Qt, QTimer, QPropertyAnimation, QPoint, QEasingCurve, QRect, QSize
from PySide6.QtGui import QPixmap, QFont, QColor, QPainter, QCursor
from PySide6.QtWidgets import (
    QWidget, QLabel, QVBoxLayout, QHBoxLayout, QGraphicsOpacityEffect
)
from PySide6.QtGui import QGuiApplication

from backend import get_response

IMG_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "img")


def load_image(name: str) -> QPixmap:
    path = os.path.join(IMG_DIR, name)
    pix = QPixmap(path)
    if pix.isNull():
        # placeholder si falta la imagen
        pix = QPixmap(160, 180)
        pix.fill(QColor("#3a3a5c"))
    return pix


class ArchitoWindow(QWidget):
    def __init__(self):
        super().__init__()

        self.setWindowFlags(
            Qt.FramelessWindowHint
            | Qt.WindowStaysOnTopHint
            | Qt.Tool
        )
        self.setAttribute(Qt.WA_TranslucentBackground, True)
        self.setAttribute(Qt.WA_DeleteOnClose, False)

        # Dimensiones del personaje
        self.char_w = 180
        self.char_h = 200

        # Cargar frames
        self.pix_main = load_image("Archito.png")
        self.pix_frame1 = load_image("Archito1.png")
        self.pix_frame2 = load_image("Archito2.png")
        self.pix_frame3 = load_image("Archito3.png")

        self._build_ui()
        self._position_bottom_right()

        # Estado de animación
        self._animating = False
        self._text_box_visible = False

        # Iniciar animación de entrada
        QTimer.singleShot(100, self._start_entrance_animation)

    # --------------------------------------------------------------
    # UI
    # --------------------------------------------------------------
    def _build_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        # Contenedor del personaje
        self.char_label = QLabel(self)
        self.char_label.setAlignment(Qt.AlignCenter)
        self.char_label.setFixedSize(self.char_w, self.char_h)
        self.char_label.setPixmap(
            self.pix_frame1.scaled(
                self.char_w, self.char_h,
                Qt.KeepAspectRatio, Qt.SmoothTransformation
            )
        )
        layout.addWidget(self.char_label)

        # Caja de texto (inicialmente oculta)
        self.text_box = QWidget(self)
        self.text_box.setObjectName("textBox")
        self.text_box.setFixedWidth(340)
        self.text_box.setVisible(False)

        box_layout = QVBoxLayout(self.text_box)
        box_layout.setContentsMargins(14, 12, 14, 12)
        box_layout.setSpacing(8)

        # Mensaje de bienvenida
        self.welcome_label = QLabel("👋 Hola, soy Archito.\n¿En qué puedo ayudarte?")
        self.welcome_label.setWordWrap(True)
        self.welcome_label.setStyleSheet(
            "color: #e8e8f0; font-size: 13px; font-weight: 500;"
        )
        box_layout.addWidget(self.welcome_label)

        # Input (simulado con QLabel editable usando QTextEdit)
        from PySide6.QtWidgets import QTextEdit
        self.input_field = QTextEdit()
        self.input_field.setPlaceholderText("Escribe aquí...")
        self.input_field.setFixedHeight(34)
        self.input_field.setStyleSheet(
            """
            QTextEdit {
                background-color: #2a2a40;
                color: #e8e8f0;
                border: 1px solid #555580;
                border-radius: 8px;
                padding: 6px 10px;
                font-size: 13px;
            }
            QTextEdit:focus {
                border: 1px solid #7b8cff;
            }
            """
        )
        box_layout.addWidget(self.input_field)

        # Respuesta de Archito
        self.response_label = QLabel("")
        self.response_label.setWordWrap(True)
        self.response_label.setStyleSheet(
            "color: #b8b8d0; font-size: 12px; font-style: italic;"
        )
        self.response_label.setVisible(False)
        box_layout.addWidget(self.response_label)

        layout.addWidget(self.text_box)

        # Estilo general de la caja
        self.text_box.setStyleSheet(
            """
            QWidget#textBox {
                background-color: rgba(30, 30, 50, 240);
                border: 1px solid #4a4a70;
                border-radius: 14px;
            }
            """
        )

        self.setLayout(layout)
        self.adjustSize()

    def _position_bottom_right(self):
        """Coloca la ventana en la esquina inferior derecha de la pantalla."""
        screen = QGuiApplication.primaryScreen().availableGeometry()
        margin = 30
        x = screen.width() - self.width() - margin
        y = screen.height() - self.height() - margin
        self.move(x, y)

    # --------------------------------------------------------------
    # Animación de entrada (Archito1 → Archito2 → Archito3 → Archito)
    # --------------------------------------------------------------
    def _start_entrance_animation(self):
        self._animating = True
        # Frame 1: apenas asomando
        self.char_label.setPixmap(
            self.pix_frame1.scaled(
                self.char_w, self.char_h,
                Qt.KeepAspectRatio, Qt.SmoothTransformation
            )
        )

        # Secuencia de frames
        QTimer.singleShot(250, self._show_frame2)
        QTimer.singleShot(550, self._show_frame3)
        QTimer.singleShot(850, self._show_main_and_box)

    def _show_frame2(self):
        self.char_label.setPixmap(
            self.pix_frame2.scaled(
                self.char_w, self.char_h,
                Qt.KeepAspectRatio, Qt.SmoothTransformation
            )
        )

    def _show_frame3(self):
        self.char_label.setPixmap(
            self.pix_frame3.scaled(
                self.char_w, self.char_h,
                Qt.KeepAspectRatio, Qt.SmoothTransformation
            )
        )

    def _show_main_and_box(self):
        """Frame final: Archito quieto saludando + aparece la caja de texto."""
        self.char_label.setPixmap(
            self.pix_main.scaled(
                self.char_w, self.char_h,
                Qt.KeepAspectRatio, Qt.SmoothTransformation
            )
        )

        # Mostrar la caja de texto con fade-in
        self.text_box.setVisible(True)
        self.text_box.adjustSize()
        self.adjustSize()
        self._position_bottom_right()

        # Fade-in de la caja
        self.opacity_effect = QGraphicsOpacityEffect(self.text_box)
        self.text_box.setGraphicsEffect(self.opacity_effect)
        self.opacity_anim = QPropertyAnimation(self.opacity_effect, b"opacity")
        self.opacity_anim.setDuration(300)
        self.opacity_anim.setStartValue(0.0)
        self.opacity_anim.setEndValue(1.0)
        self.opacity_anim.setEasingCurve(QEasingCurve.OutCubic)
        self.opacity_anim.start()

        self._animating = False
        self._text_box_visible = True
        self.input_field.setFocus()

    # --------------------------------------------------------------
    # Eventos
    # --------------------------------------------------------------
    def keyPressEvent(self, event):
        # Cerrar con Escape
        if event.key() == Qt.Key_Escape:
            self.close()
            return

        # Enter para enviar mensaje
        if event.key() == Qt.Key_Return or event.key() == Qt.Key_Enter:
            text = self.input_field.toPlainText().strip()
            if text:
                self._process_user_input(text)
            return

        super().keyPressEvent(event)

    def _process_user_input(self, text: str):
        # Mostrar lo que escribió el usuario
        self.response_label.setText(f"Tú: {text}\n")
        self.response_label.setVisible(True)

        # Obtener respuesta del backend
        response = get_response(text)
        QTimer.singleShot(400, lambda: self._show_response(response))

    def _show_response(self, response: str):
        self.response_label.setText(response)
        self.response_label.setVisible(True)
        self.input_field.clear()

    # --------------------------------------------------------------
    # Animación de salida (opcional, para futuro)
    # --------------------------------------------------------------
    def fade_out(self):
        self.opacity_effect = QGraphicsOpacityEffect(self)
        self.setGraphicsEffect(self.opacity_effect)
        self.opacity_anim = QPropertyAnimation(self.opacity_effect, b"opacity")
        self.opacity_anim.setDuration(250)
        self.opacity_anim.setStartValue(1.0)
        self.opacity_anim.setEndValue(0.0)
        self.opacity_anim.setEasingCurve(QEasingCurve.InCubic)
        self.opacity_anim.finished.connect(self.close)
        self.opacity_anim.start()


if __name__ == "__main__":
    import sys
    from PySide6.QtWidgets import QApplication
    app = QApplication(sys.argv)
    w = ArchitoWindow()
    w.show()
    sys.exit(app.exec())
